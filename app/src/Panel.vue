<script setup lang="ts">
// Menu-bar control panel (right click on the tray icon), same layout as 1.x.
import { ref, computed, onMounted, onBeforeUnmount } from "vue";
import { api, native, type PanelState, type Settings } from "./api";
import { t, setLang, languages } from "./i18n";
import { applyTheme, cards, watchSystem } from "./themes";
import { getCurrentWindow } from "@tauri-apps/api/window";

const state = ref<PanelState | null>(null);
const settings = ref<Settings | null>(null);
const isMac = navigator.platform.toLowerCase().includes("mac");

const live = computed(() => state.value?.status.state === "ready");
const memory = computed(() => {
  const b = state.value?.memory ?? 0;
  return b >= 1 << 30 ? `${(b / (1 << 30)).toFixed(2)} GB` : `${Math.round(b / (1 << 20))} MB`;
});
const hotkey = computed(() => {
  const h = settings.value?.hotkey ?? "";
  if (!isMac) return h;
  return h.replace(/CommandOrControl|Command|Cmd|Super/gi, "⌘").replace(/Control|Ctrl/gi, "⌃")
    .replace(/Alt|Option/gi, "⌥").replace(/Shift/gi, "⇧").replace(/Space/gi, "Space").replace(/\+/g, "");
});

async function refresh() {
  const [s, st] = await Promise.all([api.settings(), api.panelState()]);
  settings.value = s; state.value = st;
  setLang(s.lang as any);
  applyTheme(s.theme, s.appearance);
}

async function update(change: Partial<Settings>) {
  if (!settings.value) return;
  const next = { ...settings.value, ...change };
  settings.value = next;
  setLang(next.lang as any);
  applyTheme(next.theme, next.appearance);
  await api.saveSettings(next);
}

const run = (fn: () => Promise<unknown>) => () => { fn(); };
const rows = computed(() => [
  { icon: state.value?.windowVisible ? "◌" : "▢", title: state.value?.windowVisible ? t("Hide JSeek") : t("Show JSeek"), hint: hotkey.value, act: run(api.toggleMain) },
  { icon: "↻", title: t("Rebuild Index"), hint: "", act: run(async () => { await api.reindex(); await refresh(); }) },
  { icon: "⌕", title: t("Content Settings…"), hint: "", act: run(() => api.openMainAt("content")) },
  { icon: "⌘", title: t("Shortcut Settings…"), hint: "", act: run(() => api.openMainAt("general")) },
  { icon: "?", title: t("Search Syntax Help"), hint: "", act: run(() => api.openMainAt("help")) },
]);

let timer: number | undefined;
const unlisteners: (() => void)[] = [];
onMounted(async () => {
  await refresh();
  watchSystem(() => settings.value && applyTheme(settings.value.theme, settings.value.appearance));
  // Refresh counts and memory while the panel is open.
  timer = window.setInterval(() => { if (document.visibilityState === "visible") api.panelState().then((s) => (state.value = s)); }, 2000);
  unlisteners.push(
    await api.on("panel-opened", refresh),
    await api.onSettings((s) => { settings.value = s; setLang(s.lang as any); applyTheme(s.theme, s.appearance); }),
  );
  if (native) {
    // Click outside → close, like a menu.
    unlisteners.push(await getCurrentWindow().onFocusChanged(({ payload }) => { if (!payload) api.hidePanel(); }));
  }
  window.addEventListener("keydown", (e) => { if (e.key === "Escape") api.hidePanel(); });
});
onBeforeUnmount(() => { clearInterval(timer); unlisteners.forEach((u) => u()); });
</script>

<template>
  <div class="panel">
    <header>
      <div class="top">
        <span class="mark">◎</span><span class="word">JSEEK</span>
        <span class="grow"></span>
        <span class="live"><i :class="{ on: live }"></i>{{ live ? t("Live") : t("Scanning") }}</span>
      </div>
      <div class="stats">
        <span><b>{{ (state?.status.count ?? 0).toLocaleString() }}</b> {{ t("files") }}</span>
        <span><b>{{ memory }}</b> {{ t("Memory") }}</span>
      </div>
    </header>

    <div class="body">
      <div class="group">
        <button v-for="r in rows" :key="r.title" class="row" @click="r.act">
          <span class="ic">{{ r.icon }}</span><span class="title">{{ r.title }}</span><span v-if="r.hint" class="hint">{{ r.hint }}</span>
        </button>
      </div>

      <div v-if="settings" class="group">
        <h4>{{ t("Startup") }}</h4>
        <label class="toggle">
          <span>{{ t("Launch at Login") }}</span>
          <input type="checkbox" :checked="settings.launchAtLogin" @change="update({ launchAtLogin: ($event.target as HTMLInputElement).checked })" />
        </label>
        <label class="toggle">
          <span>{{ isMac ? t("Keep Running in Menu Bar") : t("Keep Running in Tray") }}</span>
          <input type="checkbox" :checked="settings.hideOnClose" @change="update({ hideOnClose: ($event.target as HTMLInputElement).checked })" />
        </label>
      </div>

      <div v-if="settings" class="group">
        <h4>{{ t("Appearance") }}</h4>
        <div class="look">
          <button
            v-for="c in cards" :key="c.id" class="dot" :class="{ on: settings.theme === c.id }" :title="c.name"
            :style="{ background: c.light.paper, '--a': c.light.accent }" @click="update({ theme: c.id })"
          ><i></i></button>
          <span class="grow"></span>
          <select :value="settings.lang" :aria-label="t('Language')" @change="update({ lang: ($event.target as HTMLSelectElement).value })">
            <option v-for="l in languages" :key="l.id" :value="l.id">{{ l.label() }}</option>
          </select>
        </div>
      </div>
    </div>

    <footer>
      <button class="quit" @click="api.quit()">⏻ {{ t("Quit JSeek") }}</button>
      <span class="grow"></span>
      <span class="ver">v{{ state?.version }}</span>
    </footer>
  </div>
</template>

<style scoped>
.panel { height: 100%; display: flex; flex-direction: column; background: var(--paper); color: var(--ink); user-select: none; overflow: hidden; }
header { background: var(--header); color: var(--header-ink); padding: 12px 16px; }
.top { display: flex; align-items: center; gap: 7px; }
.mark { color: var(--accent); font-weight: 700; }
.word { font-size: 11.5px; font-weight: 800; letter-spacing: 1.6px; }
.grow { flex: 1; }
.live { font-size: 10.5px; opacity: .85; display: flex; align-items: center; gap: 5px; }
.live i { width: 6px; height: 6px; border-radius: 50%; background: color-mix(in srgb, var(--header-ink) 40%, transparent); }
.live i.on { background: #3fb36b; }
.stats { display: flex; gap: 14px; margin-top: 8px; font-size: 10.5px; }
.stats span { opacity: .6; }
.stats b { font: 700 15px ui-monospace, monospace; opacity: 1; color: var(--header-ink); }
.stats span:has(b) { opacity: 1; }
.body { flex: 1; overflow: auto; padding: 12px 10px; display: flex; flex-direction: column; gap: 14px; }
.group { display: flex; flex-direction: column; gap: 2px; }
h4 { margin: 0 10px 3px; font: 600 9.5px ui-monospace, monospace; letter-spacing: 1.1px; text-transform: uppercase; color: var(--muted); }
.row { all: unset; cursor: pointer; display: flex; align-items: center; gap: 10px; padding: 6px 10px; border-radius: 5px; font-size: 13px; position: relative; }
.row:hover { background: var(--surface); box-shadow: inset 2.5px 0 var(--accent); }
.ic { width: 18px; text-align: center; color: var(--muted); font-size: 12px; }
.row:hover .ic { color: var(--accent); }
.title { flex: 1; }
.hint { font: 10.5px ui-monospace, monospace; color: var(--muted); }
.toggle { display: flex; align-items: center; justify-content: space-between; padding: 5px 10px; font-size: 13px; cursor: pointer; }
.toggle input { appearance: none; width: 30px; height: 17px; border-radius: 999px; background: var(--rule); position: relative; cursor: pointer; transition: background .15s; }
.toggle input::after { content: ""; position: absolute; top: 2px; left: 2px; width: 13px; height: 13px; border-radius: 50%; background: #fff; transition: left .15s; box-shadow: 0 1px 2px rgba(0,0,0,.25); }
.toggle input:checked { background: var(--accent); }
.toggle input:checked::after { left: 15px; }
.look { display: flex; align-items: center; gap: 8px; padding: 2px 10px; }
.dot { all: unset; cursor: pointer; width: 22px; height: 22px; border-radius: 50%; display: grid; place-items: center; border: 1px solid var(--rule); }
.dot i { width: 10px; height: 10px; border-radius: 50%; background: var(--a); }
.dot.on { border: 2px solid var(--accent); }
select { font-size: 11.5px; background: var(--surface); color: var(--ink); border: 1px solid var(--rule); border-radius: 5px; padding: 2px 4px; }
footer { display: flex; align-items: center; padding: 10px 18px; border-top: 1px solid var(--rule); }
.quit { all: unset; cursor: pointer; color: var(--accent); font-size: 12.5px; font-weight: 500; }
.quit:hover { text-decoration: underline; }
.ver { font: 10.5px ui-monospace, monospace; color: var(--muted); }
</style>
