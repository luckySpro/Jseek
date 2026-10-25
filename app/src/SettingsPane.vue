<script setup lang="ts">
import { ref, computed } from "vue";
import { api, type Settings, type Status, type Tab } from "./api";
import { t, languages } from "./i18n";
import { cards } from "./themes";

const props = defineProps<{ settings: Settings; status: Status | null }>();
const emit = defineEmits<{ save: [s: Settings]; close: [] }>();

// JSON copy: structuredClone throws on Vue reactive proxies (that left the pane blank).
const clone = <T,>(v: T): T => JSON.parse(JSON.stringify(v));
const draft = ref<Settings>(clone(props.settings));
const section = ref("general");
const sections = [
  ["general", "General"], ["index", "Index"], ["tabs", "Tabs"], ["quick", "QUICK"], ["content", "Content search"],
] as const;
const dirty = computed(() => JSON.stringify(draft.value) !== JSON.stringify(props.settings));

const extText = (xs: string[]) => xs.join(" ");
const parseExts = (s: string) =>
  [...new Set(s.split(/[\s,;]+/).map((x) => x.trim().replace(/^\./, "").toLowerCase()).filter(Boolean))];

function moveTab(i: number, d: number) {
  const tabs = draft.value.tabs, j = i + d;
  if (j < 0 || j >= tabs.length) return;
  [tabs[i], tabs[j]] = [tabs[j], tabs[i]];
}
function addTab() {
  const tab: Tab = { id: `custom-${Date.now().toString(36)}`, name: t("New tab"), exts: [], filter: "" };
  draft.value.tabs.push(tab);
}
function moveQuick(i: number, d: number) {
  const q = draft.value.quick, j = i + d;
  if (j < 0 || j >= q.length) return;
  [q[i], q[j]] = [q[j], q[i]];
}
async function resetDefaults() {
  const d = await api.defaults();
  d.lang = draft.value.lang;
  draft.value = d;
}
function save() {
  draft.value.resultLimit = Math.max(50, Math.min(5000, Number(draft.value.resultLimit) || 500));
  draft.value.contentMaxMb = Math.max(1, Math.min(100, Number(draft.value.contentMaxMb) || 4));
  emit("save", clone(draft.value));
  emit("close");
}
const builtin = (id: string) => ["all", "content"].includes(id);
</script>

<template>
  <div class="scrim" @click.self="emit('close')">
    <div class="pane" role="dialog" :aria-label="t('Settings')">
      <aside>
        <h2>{{ t("Settings") }}</h2>
        <button v-for="[id, label] in sections" :key="id" :class="{ on: section === id }" @click="section = id">{{ t(label) }}</button>
      </aside>

      <div class="content">
        <!-- General -->
        <template v-if="section === 'general'">
          <label class="field">
            <span>{{ t("Language") }}</span>
            <select v-model="draft.lang">
              <option v-for="l in languages" :key="l.id" :value="l.id">{{ l.label() }}</option>
            </select>
          </label>
          <div class="field">
            <span>{{ t("Color card") }}</span>
            <div class="cards">
              <button v-for="c in cards" :key="c.id" class="swatch" :class="{ on: draft.theme === c.id }" @click="draft.theme = c.id">
                <i :style="{ background: c.light.header }"></i><i :style="{ background: c.light.paper }"></i><i :style="{ background: c.light.accent }"></i>
                <span>{{ c.name }}</span>
              </button>
            </div>
          </div>
          <label class="field">
            <span>{{ t("Appearance") }}</span>
            <select v-model="draft.appearance">
              <option value="system">{{ t("Follow System") }}</option>
              <option value="light">{{ t("Light") }}</option>
              <option value="dark">{{ t("Dark") }}</option>
            </select>
          </label>
          <label class="field check">
            <input v-model="draft.hideOnClose" type="checkbox" />
            <span>{{ t("Closing the window keeps JSeek running") }}</span>
          </label>
          <label class="field check">
            <input v-model="draft.launchAtLogin" type="checkbox" />
            <span>{{ t("Start JSeek when I log in") }}</span>
          </label>
          <label class="field">
            <span>{{ t("Global shortcut") }}</span>
            <input v-model="draft.hotkey" spellcheck="false" placeholder="Alt+Space" />
            <small>{{ t("Shows or hides JSeek from anywhere, e.g. Alt+Space or CommandOrControl+Shift+K. Leave empty for none.") }}</small>
          </label>
          <label class="field">
            <span>{{ t("Maximum results") }}</span>
            <input v-model.number="draft.resultLimit" type="number" min="50" max="5000" step="50" />
          </label>
        </template>

        <!-- Index -->
        <template v-else-if="section === 'index'">
          <label class="field">
            <span>{{ t("Folder to index") }}</span>
            <input v-model="draft.scanRoot" spellcheck="false" />
            <small>{{ t("Use / (Mac) or C:/ (Windows) for the whole disk. Changing it rebuilds the index.") }}</small>
          </label>
          <div class="field">
            <span>{{ t("Status") }}</span>
            <p v-if="status" class="mono">
              {{ t(status.state === "ready" ? "Ready" : "Indexing…") }} · {{ status.count.toLocaleString() }} {{ t("items") }}
              <template v-if="status.scanMs"> · {{ (status.scanMs / 1000).toFixed(1) }} s</template>
            </p>
            <button class="btn" @click="api.reindex()">{{ t("Rebuild index") }}</button>
          </div>
        </template>

        <!-- Tabs -->
        <template v-else-if="section === 'tabs'">
          <p class="hint">{{ t("Each tab adds its extensions and filter to what you type. Filters use the search syntax, e.g. folder: or size:>10mb.") }}</p>
          <div v-for="(tab, i) in draft.tabs" :key="tab.id" class="item">
            <div class="itemhead">
              <b class="num">{{ String(i + 1).padStart(2, "0") }}</b>
              <input v-model="tab.name" class="name" :aria-label="t('Name')" />
              <span class="tools">
                <button :title="t('Move up')" @click="moveTab(i, -1)">↑</button>
                <button :title="t('Move down')" @click="moveTab(i, 1)">↓</button>
                <button v-if="!builtin(tab.id)" :title="t('Remove')" @click="draft.tabs.splice(i, 1)">✕</button>
              </span>
            </div>
            <template v-if="tab.id !== 'content'">
              <label class="sub"><span>{{ t("Extensions") }}</span>
                <input :value="extText(tab.exts)" spellcheck="false" :placeholder="t('e.g. pdf docx md')" @change="tab.exts = parseExts(($event.target as HTMLInputElement).value)" />
              </label>
              <label class="sub"><span>{{ t("Filter") }}</span>
                <input v-model="tab.filter" spellcheck="false" placeholder="folder:" />
              </label>
            </template>
            <small v-else>{{ t("Searches file contents; its file types are set under Content search.") }}</small>
          </div>
          <button class="btn" @click="addTab">+ {{ t("Add tab") }}</button>
        </template>

        <!-- QUICK -->
        <template v-else-if="section === 'quick'">
          <p class="hint">{{ t("QUICK chips add a query with one click or ⌃⌥1–9.") }}</p>
          <div v-for="(q, i) in draft.quick" :key="i" class="item row">
            <b class="num">⌃⌥{{ i + 1 }}</b>
            <input v-model="q.name" class="name" :aria-label="t('Name')" />
            <input v-model="q.query" class="q" spellcheck="false" :aria-label="t('Query')" />
            <span class="tools">
              <button :title="t('Move up')" @click="moveQuick(i, -1)">↑</button>
              <button :title="t('Move down')" @click="moveQuick(i, 1)">↓</button>
              <button :title="t('Remove')" @click="draft.quick.splice(i, 1)">✕</button>
            </span>
          </div>
          <button class="btn" @click="draft.quick.push({ name: t('New'), query: '' })">+ {{ t("Add chip") }}</button>
        </template>

        <!-- Content -->
        <template v-else>
          <label class="field">
            <span>{{ t("File types to search inside") }}</span>
            <textarea :value="extText(draft.contentExts)" rows="4" spellcheck="false" @change="draft.contentExts = parseExts(($event.target as HTMLTextAreaElement).value)"></textarea>
          </label>
          <label class="field">
            <span>{{ t("Skip files larger than (MB)") }}</span>
            <input v-model.number="draft.contentMaxMb" type="number" min="1" max="100" />
          </label>
          <p class="hint">{{ t("Tip: add filters to narrow the files read, e.g. TODO path:src ext:rs") }}</p>
        </template>
      </div>

      <footer>
        <button class="link" @click="resetDefaults">{{ t("Reset to defaults") }}</button>
        <span class="grow"></span>
        <button class="btn ghost" @click="emit('close')">{{ t("Cancel") }}</button>
        <button class="btn primary" :disabled="!dirty" @click="save">{{ t("Save") }}</button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.scrim { position: fixed; inset: 0; background: color-mix(in srgb, var(--ink) 35%, transparent); display: grid; place-items: center; z-index: 10; }
.pane {
  width: min(760px, calc(100vw - 32px)); height: min(560px, calc(100vh - 60px)); background: var(--paper); border-radius: 8px;
  display: grid; grid-template-columns: 170px 1fr; grid-template-rows: 1fr auto; overflow: hidden;
  box-shadow: 0 18px 50px rgba(0, 0, 0, .3); border: 1px solid var(--rule);
}
aside { grid-row: 1 / 3; background: var(--header); color: var(--header-ink); padding: 16px 10px; display: flex; flex-direction: column; gap: 2px; }
aside h2 { font-size: 13px; letter-spacing: 1.5px; text-transform: uppercase; margin: 0 8px 12px; color: var(--accent); }
aside button { all: unset; cursor: pointer; padding: 6px 10px; border-radius: 5px; opacity: .7; font-size: 13.5px; }
aside button:hover { opacity: 1; }
aside button.on { opacity: 1; background: color-mix(in srgb, var(--header-ink) 12%, transparent); box-shadow: inset 2.5px 0 var(--accent); }
.content { overflow: auto; padding: 18px 22px; display: flex; flex-direction: column; gap: 14px; }
.field { display: flex; flex-direction: column; gap: 5px; }
.field > span { font-weight: 600; font-size: 13px; }
.field.check { flex-direction: row; align-items: center; gap: 8px; }
.field.check span { font-weight: 400; }
small, .hint { color: var(--muted); font-size: 12px; margin: 0; }
input:not([type="checkbox"]), select, textarea {
  background: var(--surface); color: var(--ink); border: 1px solid var(--rule); border-radius: 5px; padding: 6px 8px; outline: none;
}
input:focus, select:focus, textarea:focus { border-color: var(--accent); }
textarea { font-family: ui-monospace, monospace; font-size: 12px; resize: vertical; }
.mono { font: 12px ui-monospace, monospace; margin: 0; }
.cards { display: flex; gap: 8px; flex-wrap: wrap; }
.swatch { all: unset; cursor: pointer; display: grid; grid-template-columns: repeat(3, 18px); gap: 0; border: 1px solid var(--rule); border-radius: 6px; padding: 6px; background: var(--surface); }
.swatch i { height: 26px; }
.swatch span { grid-column: 1 / 4; font-size: 11px; margin-top: 4px; text-align: center; }
.swatch.on { border-color: var(--accent); box-shadow: 0 0 0 1px var(--accent); }
.item { border: 1px solid var(--rule); background: var(--surface); border-radius: 6px; padding: 8px 10px; display: flex; flex-direction: column; gap: 6px; }
.item.row { flex-direction: row; align-items: center; gap: 8px; }
.itemhead { display: flex; align-items: center; gap: 8px; }
.num { font: 600 11px ui-monospace, monospace; color: var(--accent); min-width: 30px; }
.name { flex: 1; font-weight: 600; }
.q { flex: 1.4; font-family: ui-monospace, monospace; font-size: 12px; }
.sub { display: grid; grid-template-columns: 80px 1fr; align-items: center; gap: 8px; font-size: 12px; color: var(--muted); }
.sub input { font-family: ui-monospace, monospace; font-size: 12px; }
.tools { display: flex; gap: 2px; }
.tools button { all: unset; cursor: pointer; padding: 2px 6px; border-radius: 4px; color: var(--muted); }
.tools button:hover { color: var(--accent); background: var(--paper); }
footer { display: flex; align-items: center; gap: 8px; padding: 10px 16px; border-top: 1px solid var(--rule); }
.grow { flex: 1; }
.btn { all: unset; cursor: pointer; padding: 6px 14px; border-radius: 5px; border: 1px solid var(--rule); background: var(--surface); font-size: 13px; align-self: flex-start; }
.btn:hover { border-color: var(--accent); }
.btn.primary { background: var(--accent); border-color: var(--accent); color: #fff; }
.btn.primary:disabled { opacity: .45; cursor: default; }
.link { all: unset; cursor: pointer; color: var(--muted); font-size: 12.5px; }
.link:hover { color: var(--accent); }
</style>
