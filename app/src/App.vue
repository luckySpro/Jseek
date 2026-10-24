<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { t, lang, setLang, languages } from "./i18n";

const tabs = ["All", "Apps", "Folders", "Documents", "Images", "Code"];
const active = ref(0);
const query = ref("");
const info = ref<{ version: string; platform: string; backend: string } | null>(null);

onMounted(async () => {
  try {
    info.value = await invoke("app_info");
  } catch {
    info.value = null; // running in a plain browser (vite dev)
  }
});
</script>

<template>
  <header class="bar" data-tauri-drag-region>
    <span class="mark">◎</span>
    <span class="wordmark">JSEEK</span>
    <nav>
      <button v-for="(tab, i) in tabs" :key="tab" :class="{ on: i === active }" @click="active = i">
        <b>{{ String(i + 1).padStart(2, "0") }}</b>{{ t(tab) }}
      </button>
    </nav>
    <select class="lang" :value="lang" :aria-label="t('Language')" @change="setLang(($event.target as HTMLSelectElement).value as any)">
      <option v-for="l in languages" :key="l.id" :value="l.id">{{ l.label() }}</option>
    </select>
  </header>

  <main>
    <input v-model="query" class="search" :placeholder="t('Search files and folders…')" autofocus />
    <div class="rule"></div>
    <section class="empty">
      <h1>{{ t("Find anything on your computer") }}</h1>
      <p>{{ t("Preview build — the search engine arrives in the next milestones.") }}</p>
      <p v-if="info" class="meta">v{{ info.version }} · {{ info.platform }} · {{ info.backend }}</p>
    </section>
  </main>
</template>

<style scoped>
.bar {
  display: flex; align-items: center; gap: 10px; height: 44px; padding: 0 12px 0 78px;
  background: var(--header); color: var(--header-ink); user-select: none;
}
.mark { color: var(--accent); font-weight: 700; }
.wordmark { font-weight: 800; letter-spacing: 2px; font-size: 12.5px; margin-right: 8px; }
nav { display: flex; height: 100%; min-width: 0; overflow-x: auto; flex: 1; }
nav button {
  all: unset; cursor: pointer; display: flex; align-items: center; gap: 5px; padding: 0 9px;
  color: var(--header-ink); opacity: .65; font-size: 13px; white-space: nowrap;
}
nav button b { font: 500 10px ui-monospace, monospace; opacity: .7; }
nav button.on { opacity: 1; font-weight: 600; box-shadow: inset 0 -2.5px var(--accent); }
nav button.on b { color: var(--accent); opacity: 1; }
.lang { background: transparent; color: var(--header-ink); border: 1px solid #ffffff33; border-radius: 4px; font-size: 12px; }
main { padding: 20px 24px; }
.search {
  width: 100%; font-size: 20px; padding: 10px 14px; border-radius: 6px; color: var(--ink);
  background: var(--surface); border: 1px solid var(--rule); outline: none;
}
.search:focus { border: 1.5px solid var(--accent); }
.rule { margin-top: 18px; border-top: 1.5px solid var(--ink); border-bottom: 1px solid var(--ink); height: 2px; }
.empty { padding: 28px 2px; }
.empty h1 { font-size: 24px; margin: 0 0 6px; font-weight: 800; }
.empty p { color: var(--muted); margin: 0 0 4px; }
.meta { font: 12px ui-monospace, monospace; }
</style>
