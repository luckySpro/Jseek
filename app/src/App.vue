<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount, nextTick } from "vue";
import { api, type Hit, type ContentHit, type Settings, type Status, type AppInfo, type Quick } from "./api";
import { t, setLang, resolved } from "./i18n";
import { applyTheme, watchSystem } from "./themes";
import { fmtSize, fmtDate, highlight, badge, terms } from "./format";
import SettingsPane from "./SettingsPane.vue";

const settings = ref<Settings | null>(null);
const status = ref<Status | null>(null);
const info = ref<AppInfo | null>(null);
const tabIndex = ref(0);
const query = ref("");
/** QUICK filters in use, shown as removable tags in the search box. */
const active = ref<Quick[]>([]);
const fullQuery = computed(() => [query.value.trim(), ...active.value.map((q) => q.query)].filter(Boolean).join(" "));
const hits = ref<Hit[]>([]);
const content = ref<ContentHit[]>([]);
const ms = ref(0);
const selected = ref(0);
const showSettings = ref(false);
const toast = ref("");
const input = ref<HTMLInputElement | null>(null);
const list = ref<HTMLElement | null>(null);
const navEl = ref<HTMLElement | null>(null);

const tabs = computed(() => settings.value?.tabs ?? []);
const tab = computed(() => tabs.value[tabIndex.value]);
const isContent = computed(() => tab.value?.id === "content");
const rowCount = computed(() => (isContent.value ? content.value.length : hits.value.length));
const words = computed(() => terms(fullQuery.value));
const isMac = navigator.platform.toLowerCase().includes("mac");
const mod = isMac ? "⌘" : "Ctrl";

// ── search ──
let seq = 0;
let timer: number | undefined;
async function runSearch() {
  const my = ++seq;
  const q = fullQuery.value;
  const id = tab.value?.id ?? "all";
  if (!q.trim() && (id === "all" || id === "content")) {
    hits.value = []; content.value = []; ms.value = 0;
    return;
  }
  try {
    const r = await api.search(q, id);
    if (my !== seq) return; // a newer search started while this one ran
    hits.value = r.hits; content.value = r.content; ms.value = r.ms; selected.value = 0;
    list.value?.scrollTo({ top: 0 });
  } catch (e) {
    if (my === seq) flash(String(e));
  }
}
watch([fullQuery, tabIndex], () => {
  clearTimeout(timer);
  timer = window.setTimeout(runSearch, isContent.value ? 180 : 40);
});

function flash(msg: string) {
  toast.value = msg;
  window.setTimeout(() => { if (toast.value === msg) toast.value = ""; }, 1800);
}

// ── actions ──
function currentPath(): string | undefined {
  return isContent.value ? content.value[selected.value]?.path : hits.value[selected.value]?.path;
}
async function open(path?: string) { if (path) await api.open(path).catch((e) => flash(String(e))); }
async function reveal(path?: string) { if (path) await api.reveal(path).catch((e) => flash(String(e))); }
async function copy(path?: string) {
  if (!path) return;
  try { await navigator.clipboard.writeText(path); flash(t("Path copied")); } catch { flash(t("Could not copy")); }
}
/** Filter kinds a chip uses (dm, size, ext, …); chips of the same kind replace each other. */
const kinds = (q: string) => new Set(q.split(/\s+/).filter((w) => w.includes(":")).map((w) => w.split(":")[0].replace(/^[!-]/, "").toLowerCase()));
function isActive(q: Quick) {
  return active.value.some((a) => a.query === q.query);
}
function toggleQuick(q: Quick) {
  if (isActive(q)) {
    active.value = active.value.filter((a) => a.query !== q.query);
  } else {
    const k = kinds(q.query);
    active.value = [...active.value.filter((a) => ![...kinds(a.query)].some((x) => k.has(x))), q];
  }
  input.value?.focus();
}
function removeQuick(i: number) {
  active.value = active.value.filter((_, j) => j !== i);
  input.value?.focus();
}
// Keep the active tab visible when the tab row is wider than the window.
watch(tabIndex, () => nextTick(() => navEl.value?.querySelector("button.on")?.scrollIntoView({ inline: "nearest", block: "nearest" })));
function wheelTabs(e: WheelEvent) {
  if (navEl.value && Math.abs(e.deltaY) > Math.abs(e.deltaX)) { navEl.value.scrollLeft += e.deltaY; e.preventDefault(); }
}
function pickTab(i: number) {
  if (i >= 0 && i < tabs.value.length) tabIndex.value = i;
}
function move(delta: number) {
  if (!rowCount.value) return;
  selected.value = Math.max(0, Math.min(rowCount.value - 1, selected.value + delta));
  nextTick(() => list.value?.querySelector(".row.sel")?.scrollIntoView({ block: "nearest" }));
}

function onKey(e: KeyboardEvent) {
  const cmd = isMac ? e.metaKey : e.ctrlKey;
  if (showSettings.value) {
    if (e.key === "Escape") showSettings.value = false;
    return;
  }
  if (e.ctrlKey && e.altKey && /^Digit[1-9]$/.test(e.code)) {
    const q = settings.value?.quick[Number(e.code.slice(5)) - 1];
    if (q) { e.preventDefault(); toggleQuick(q); }
    return;
  }
  if (cmd && /^[1-9]$/.test(e.key)) { e.preventDefault(); pickTab(Number(e.key) - 1); return; }
  if (cmd && e.key === ",") { e.preventDefault(); showSettings.value = true; return; }
  if (e.key === "Tab" && !e.altKey) {
    e.preventDefault();
    const n = tabs.value.length || 1;
    tabIndex.value = (tabIndex.value + (e.shiftKey ? n - 1 : 1)) % n;
    return;
  }
  switch (e.key) {
    case "ArrowDown": e.preventDefault(); move(1); break;
    case "ArrowUp": e.preventDefault(); move(-1); break;
    case "PageDown": e.preventDefault(); move(10); break;
    case "PageUp": e.preventDefault(); move(-10); break;
    case "Enter":
      e.preventDefault();
      if (cmd) reveal(currentPath()); else open(currentPath());
      break;
    case "Escape":
      e.preventDefault();
      if (query.value) query.value = "";
      else if (active.value.length) active.value = [];
      else api.hide();
      break;
    case "Backspace":
      // Empty box: Backspace removes the last QUICK tag.
      if (!query.value && active.value.length && document.activeElement === input.value) {
        e.preventDefault(); active.value = active.value.slice(0, -1);
      }
      break;
    case "c": case "C": {
      const sel = window.getSelection()?.toString() || (input.value && input.value.selectionStart !== input.value.selectionEnd);
      if (cmd && !sel && rowCount.value) { e.preventDefault(); copy(currentPath()); }
      break;
    }
  }
}

// ── settings & theme ──
function applyLook() {
  const s = settings.value;
  if (s) applyTheme(s.theme, s.appearance);
}
async function saveSettings(s: Settings) {
  settings.value = s;
  applyLook();
  setLang(s.lang as any);
  if (tabIndex.value >= s.tabs.length) tabIndex.value = 0;
  const problems = await api.saveSettings(s).catch((e) => [String(e)]);
  if (problems.length) flash(problems.join(" · "));
  runSearch();
}

const unlisteners: (() => void)[] = [];
watch(resolved, (l) => api.setUiLang(l), { immediate: true });
onMounted(async () => {
  window.addEventListener("keydown", onKey);
  watchSystem(applyLook);
  const [s, st, i] = await Promise.all([api.settings(), api.status(), api.info()]);
  settings.value = s; status.value = st; info.value = i;
  setLang(s.lang as any);
  applyLook();
  unlisteners.push(
    await api.onStatus((x) => { status.value = x; if (x.state === "ready") runSearch(); }),
    await api.on("open-settings", () => { showSettings.value = true; }),
    await api.on("focus-search", () => { input.value?.focus(); input.value?.select(); }),
  );
  input.value?.focus();
});
onBeforeUnmount(() => { window.removeEventListener("keydown", onKey); unlisteners.forEach((u) => u()); });

const statusLine = computed(() => {
  const s = status.value;
  if (!s) return "";
  const n = s.count.toLocaleString();
  if (s.state === "indexing") return s.count ? `${t("Updating index…")} · ${n} ${t("items")}` : t("Building index…");
  if (s.state === "loading") return t("Loading index…");
  return `${n} ${t("items")} · ${s.root}`;
});
</script>

<template>
  <header class="bar" data-tauri-drag-region>
    <span class="mark" data-tauri-drag-region>◎</span>
    <span class="wordmark" data-tauri-drag-region>JSEEK</span>
    <nav ref="navEl" @wheel="wheelTabs">
      <button v-for="(x, i) in tabs" :key="x.id" :class="{ on: i === tabIndex }" :title="`${mod}${i + 1}`" @click="pickTab(i)">
        <b>{{ String(i + 1).padStart(2, "0") }}</b>{{ t(x.name) }}
      </button>
    </nav>
    <button class="gear" :title="`${t('Settings')} (${mod},)`" :aria-label="t('Settings')" @click="showSettings = true">⚙</button>
  </header>

  <main>
    <div class="searchbox" @click="input?.focus()">
      <span v-for="(q, i) in active" :key="q.query" class="tag" :title="q.query">
        {{ t(q.name) }}<button :aria-label="t('Remove')" @click.stop="removeQuick(i)">×</button>
      </span>
      <input
        ref="input" v-model="query" class="search" spellcheck="false" autocomplete="off"
        :placeholder="active.length ? '' : isContent ? t('Search inside files…') : t('Search files and folders…')"
      />
      <span v-if="fullQuery" class="count">{{ rowCount.toLocaleString() }} · {{ ms.toFixed(1) }} ms</span>
    </div>
    <div class="quick">
      <span class="qlabel">QUICK</span>
      <button v-for="(q, i) in settings?.quick ?? []" :key="i" :class="{ on: isActive(q) }" :title="`⌃⌥${i + 1} · ${q.query}`" @click="toggleQuick(q)">
        {{ t(q.name) }}<kbd v-if="i < 9">{{ i + 1 }}</kbd>
      </button>
    </div>
    <div class="rule"></div>

    <!-- start panel -->
    <section v-if="!rowCount && !fullQuery" class="start">
      <div class="lead">
        <h1>{{ isContent ? t("Search inside your files") : t("Find anything on your computer") }}</h1>
        <p class="st">
          <span class="dot" :class="status?.state"></span>{{ statusLine }}
          <button class="link" @click="api.reindex()">{{ t("Rebuild index") }}</button>
        </p>
      </div>
      <div class="grid">
        <div class="card">
          <h3>{{ t("Syntax") }}</h3>
          <dl>
            <dt>ext:pdf;docx</dt><dd>{{ t("by extension") }}</dd>
            <dt>size:>100mb</dt><dd>{{ t("by size") }}</dd>
            <dt>dm:thisweek</dt><dd>{{ t("modified this week") }}</dd>
            <dt>path:Downloads</dt><dd>{{ t("inside a folder name") }}</dd>
            <dt>a OR b · !tmp</dt><dd>{{ t("either / exclude") }}</dd>
            <dt>"exact name"</dt><dd>{{ t("exact phrase") }}</dd>
          </dl>
        </div>
        <div class="card">
          <h3>{{ t("Keyboard") }}</h3>
          <dl>
            <dt>↑ ↓</dt><dd>{{ t("move") }}</dd>
            <dt>↩</dt><dd>{{ t("open") }}</dd>
            <dt>{{ mod }} ↩</dt><dd>{{ isMac ? t("show in Finder") : t("show in Explorer") }}</dd>
            <dt>{{ mod }} C</dt><dd>{{ t("copy path") }}</dd>
            <dt>Tab · {{ mod }}1–9</dt><dd>{{ t("switch tab") }}</dd>
            <dt>⌃⌥1–9</dt><dd>{{ t("QUICK chips") }}</dd>
            <dt>Esc</dt><dd>{{ t("clear / hide") }}</dd>
          </dl>
        </div>
      </div>
    </section>

    <section v-else-if="!rowCount" class="none">
      <p>{{ t("No matches.") }}</p>
      <p v-if="status?.state !== 'ready'" class="muted">{{ statusLine }}</p>
    </section>

    <!-- results -->
    <section v-else ref="list" class="list">
      <template v-if="isContent">
        <div
          v-for="(c, i) in content" :key="c.path" class="row crow" :class="{ sel: i === selected }"
          @click="selected = i" @dblclick="open(c.path)"
        >
          <span class="badge" :data-k="badge(c.name, 'file').k">{{ badge(c.name, "file").label }}</span>
          <div class="body">
            <div class="name"><span v-html="highlight(c.name, words)"></span><span class="dir">{{ c.dir }}</span></div>
            <div v-for="l in c.lines" :key="l.line" class="line"><b>{{ l.line }}</b><span v-html="highlight(l.text, words)"></span></div>
          </div>
          <span class="meta">{{ fmtDate(c.modified) }}</span>
        </div>
      </template>
      <template v-else>
        <div
          v-for="(h, i) in hits" :key="h.path" class="row" :class="{ sel: i === selected }"
          @click="selected = i" @dblclick="open(h.path)"
        >
          <span class="badge" :data-k="badge(h.name, h.kind).k">{{ badge(h.name, h.kind).label }}</span>
          <div class="body">
            <div class="name" v-html="highlight(h.name, words)"></div>
            <div class="dir">{{ h.dir }}</div>
          </div>
          <span class="meta size">{{ h.kind === "file" ? fmtSize(h.size) : "" }}</span>
          <span class="meta">{{ fmtDate(h.modified) }}</span>
          <span class="acts">
            <button :title="isMac ? t('show in Finder') : t('show in Explorer')" @click.stop="reveal(h.path)">↗</button>
            <button :title="t('copy path')" @click.stop="copy(h.path)">⧉</button>
          </span>
        </div>
      </template>
    </section>

    <footer class="foot">
      <span><span class="dot" :class="status?.state"></span>{{ statusLine }}</span>
      <span v-if="info">JSeek {{ info.version }} · {{ info.backend }}</span>
    </footer>
  </main>

  <div v-if="toast" class="toast">{{ toast }}</div>
  <SettingsPane v-if="showSettings && settings" :settings="settings" :status="status" @save="saveSettings" @close="showSettings = false" />
</template>

<style scoped>
.bar {
  display: flex; align-items: center; gap: 10px; height: 44px; padding: 0 8px 0 78px; flex: none;
  background: var(--header); color: var(--header-ink); user-select: none;
}
.mark { color: var(--accent); font-weight: 700; }
.wordmark { font-weight: 800; letter-spacing: 2px; font-size: 12.5px; margin-right: 6px; }
nav {
  display: flex; height: 100%; min-width: 0; overflow-x: auto; flex: 1; scrollbar-width: none;
  mask-image: linear-gradient(to right, #000 calc(100% - 28px), transparent);
}
nav::-webkit-scrollbar { display: none; }
nav button, .gear {
  all: unset; cursor: pointer; display: flex; align-items: center; gap: 5px; padding: 0 9px;
  color: var(--header-ink); opacity: .65; font-size: 13px; white-space: nowrap;
}
nav button:hover, .gear:hover { opacity: .9; }
nav button b { font: 500 10px ui-monospace, monospace; opacity: .7; }
nav button.on { opacity: 1; font-weight: 600; box-shadow: inset 0 -2.5px var(--accent); }
nav button.on b { color: var(--accent); opacity: 1; }
.gear { font-size: 16px; height: 100%; }

main { flex: 1; display: flex; flex-direction: column; min-height: 0; padding: 16px 22px 0; }
.searchbox {
  display: flex; align-items: center; flex-wrap: wrap; gap: 6px; padding: 7px 12px; border-radius: 6px; cursor: text;
  background: var(--surface); border: 1px solid var(--rule);
}
.searchbox:focus-within { border-color: var(--accent); box-shadow: 0 0 0 1px var(--accent); }
.search { flex: 1; min-width: 160px; font-size: 20px; padding: 3px 2px; color: var(--ink); background: transparent; border: 0; outline: none; }
.tag {
  display: inline-flex; align-items: center; gap: 2px; padding: 2px 4px 2px 10px; border-radius: 999px; font-size: 13px; font-weight: 600;
  background: var(--accent); color: #fff; white-space: nowrap;
}
.tag button { all: unset; cursor: pointer; width: 18px; text-align: center; border-radius: 50%; opacity: .8; }
.tag button:hover { opacity: 1; background: rgba(255, 255, 255, .22); }
.count { flex: none; font: 12px ui-monospace, monospace; color: var(--muted); }

.quick { display: flex; gap: 6px; align-items: center; margin-top: 10px; flex-wrap: wrap; }
.qlabel { font: 700 10px ui-monospace, monospace; letter-spacing: 1.5px; color: var(--accent); margin-right: 2px; }
.quick button {
  all: unset; cursor: pointer; font-size: 12px; padding: 2px 8px; border: 1px solid var(--rule); border-radius: 999px;
  background: var(--surface); display: inline-flex; gap: 5px; align-items: center;
}
.quick button:hover { border-color: var(--accent); color: var(--accent); }
.quick button.on { background: var(--accent); border-color: var(--accent); color: #fff; }
.quick button.on kbd { color: #fff; opacity: .8; }
.quick kbd { font: 10px ui-monospace, monospace; color: var(--muted); }
.rule { margin-top: 12px; border-top: 1.5px solid var(--ink); border-bottom: 1px solid var(--ink); height: 3px; flex: none; }

.start { padding: 22px 2px; overflow: auto; }
.lead h1 { font-size: 24px; margin: 0 0 6px; font-weight: 800; letter-spacing: -.2px; }
.st { color: var(--muted); margin: 0; display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
.link { all: unset; cursor: pointer; color: var(--accent); margin-left: 8px; font-size: 13px; }
.link:hover { text-decoration: underline; }
.grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 14px; margin-top: 22px; }
.card { border: 1px solid var(--rule); background: var(--surface); border-radius: 6px; padding: 12px 14px; }
.card h3 { margin: 0 0 8px; font: 700 11px ui-monospace, monospace; letter-spacing: 1.5px; text-transform: uppercase; color: var(--accent); }
dl { display: grid; grid-template-columns: max-content 1fr; gap: 4px 14px; margin: 0; font-size: 13px; }
dt { font-family: ui-monospace, monospace; font-size: 12px; }
dd { margin: 0; color: var(--muted); }

.none { padding: 26px 2px; }
.muted { color: var(--muted); }

.list { flex: 1; overflow: auto; min-height: 0; margin: 0 -22px; padding: 4px 0; }
.row {
  display: flex; align-items: center; gap: 12px; padding: 6px 22px; cursor: default;
  border-bottom: 1px solid color-mix(in srgb, var(--rule) 55%, transparent);
}
.row:hover { background: color-mix(in srgb, var(--surface) 70%, transparent); }
.row.sel { background: var(--surface); box-shadow: inset 3px 0 var(--accent); }
.badge {
  flex: none; width: 34px; height: 22px; border-radius: 4px; display: grid; place-items: center;
  font: 700 9px ui-monospace, monospace; color: #fff; background: var(--muted); text-transform: uppercase;
}
.badge[data-k="folder"] { background: var(--accent); }
.badge[data-k="app"] { background: var(--ink); color: var(--paper); }
.badge[data-k="doc"] { background: #4a6fa5; }
.badge[data-k="img"] { background: #3f8a5e; }
.badge[data-k="media"] { background: #8a5a9e; }
.badge[data-k="code"] { background: #b0782a; }
.badge[data-k="zip"] { background: #7b6f5c; }
.body { flex: 1; min-width: 0; }
.name { font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.dir { color: var(--muted); font-size: 12px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.crow { align-items: flex-start; }
.crow .badge { margin-top: 2px; }
.crow .name .dir { margin-left: 10px; font-weight: 400; }
.line { font: 12px/1.6 ui-monospace, monospace; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.line b { display: inline-block; min-width: 38px; color: var(--muted); font-weight: 400; }
.meta { flex: none; font: 12px ui-monospace, monospace; color: var(--muted); min-width: 92px; text-align: right; }
.meta.size { min-width: 70px; }
.acts { flex: none; display: flex; gap: 2px; visibility: hidden; }
.row:hover .acts, .row.sel .acts { visibility: visible; }
.acts button { all: unset; cursor: pointer; padding: 2px 6px; border-radius: 4px; color: var(--muted); }
.acts button:hover { color: var(--accent); background: var(--paper); }
:deep(mark) { background: var(--hl); color: inherit; border-radius: 2px; padding: 0 1px; }

.foot {
  flex: none; display: flex; justify-content: space-between; gap: 12px; padding: 6px 0 8px; font: 11.5px ui-monospace, monospace;
  color: var(--muted); border-top: 1px solid var(--rule);
}
.dot { display: inline-block; width: 7px; height: 7px; border-radius: 50%; background: var(--muted); margin-right: 6px; }
.dot.ready { background: #3f8a5e; }
.dot.indexing, .dot.loading { background: var(--accent); animation: pulse 1.2s infinite; }
@keyframes pulse { 50% { opacity: .35; } }
.toast {
  position: fixed; bottom: 40px; left: 50%; transform: translateX(-50%); background: var(--ink); color: var(--paper);
  padding: 6px 14px; border-radius: 6px; font-size: 13px;
}
@media (max-width: 760px) { .meta.size { display: none; } }
</style>
