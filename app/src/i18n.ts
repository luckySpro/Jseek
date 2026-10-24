import { ref, computed } from "vue";

// Series baseline: zh-Hans, zh-Hant, en, ja. English text is the key; missing keys fall back to English.
export type Lang = "system" | "zh-Hans" | "zh-Hant" | "en" | "ja";
type Row = [hans: string, hant: string, ja: string];

const table: Record<string, Row> = {
  "Search files and folders…": ["搜索文件和文件夹…", "搜尋檔案和檔案夾…", "ファイルとフォルダを検索…"],
  "All": ["全部", "全部", "すべて"],
  "Apps": ["应用", "應用程式", "アプリ"],
  "Folders": ["文件夹", "檔案夾", "フォルダ"],
  "Documents": ["文档", "文件", "書類"],
  "Images": ["图片", "圖片", "画像"],
  "Code": ["代码", "程式碼", "コード"],
  "Find anything on your computer": ["在电脑上找到任何东西", "在電腦上找到任何東西", "コンピュータのあらゆるものを見つける"],
  "Preview build — the search engine arrives in the next milestones.": [
    "预览版——搜索引擎将在后续里程碑接入。",
    "預覽版——搜尋引擎將在後續里程碑接入。",
    "プレビュー版 — 検索エンジンは次のマイルストーンで追加されます。",
  ],
  "Language": ["语言", "語言", "言語"],
  "Follow System": ["跟随系统", "跟隨系統", "システムに従う"],
};

const STORAGE_KEY = "jseek.lang";
function readSaved(): Lang {
  try {
    return (localStorage.getItem(STORAGE_KEY) as Lang) || "system";
  } catch {
    return "system";
  }
}

export const lang = ref<Lang>(readSaved());

export function setLang(l: Lang) {
  lang.value = l;
  try {
    localStorage.setItem(STORAGE_KEY, l);
  } catch {
    /* storage unavailable: keep in memory */
  }
}

function resolve(l: Lang): Exclude<Lang, "system"> {
  if (l !== "system") return l;
  for (const p of navigator.languages.map((x) => x.toLowerCase())) {
    if (/^zh-(hant|tw|hk|mo)/.test(p)) return "zh-Hant";
    if (p.startsWith("zh")) return "zh-Hans";
    if (p.startsWith("ja")) return "ja";
    if (p.startsWith("en")) return "en";
  }
  return "en";
}

export const resolved = computed(() => resolve(lang.value));

export function t(english: string): string {
  const row = table[english];
  switch (resolved.value) {
    case "zh-Hans": return row?.[0] ?? english;
    case "zh-Hant": return row?.[1] ?? english;
    case "ja": return row?.[2] ?? english;
    default: return english;
  }
}

export const languages: { id: Lang; label: () => string }[] = [
  { id: "system", label: () => t("Follow System") },
  { id: "zh-Hans", label: () => "简体中文" },
  { id: "zh-Hant", label: () => "繁體中文" },
  { id: "en", label: () => "English" },
  { id: "ja", label: () => "日本語" },
];
