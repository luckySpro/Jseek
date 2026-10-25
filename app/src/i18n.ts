import { ref, computed } from "vue";

// Series baseline: zh-Hans, zh-Hant, en, ja. English text is the key; missing keys fall back to English.
export type Lang = "system" | "zh-Hans" | "zh-Hant" | "en" | "ja";
type Row = [hans: string, hant: string, ja: string];

const table: Record<string, Row> = {
  "Search files and folders…": ["搜索文件和文件夹…", "搜尋檔案和檔案夾…", "ファイルとフォルダを検索…"],
  "Search inside files…": ["搜索文件内容…", "搜尋檔案內容…", "ファイルの中身を検索…"],
  "All": ["全部", "全部", "すべて"],
  "Apps": ["应用", "應用程式", "アプリ"],
  "Folders": ["文件夹", "檔案夾", "フォルダ"],
  "Documents": ["文档", "文件", "書類"],
  "Images": ["图片", "圖片", "画像"],
  "Audio & Video": ["音视频", "影音", "音声・動画"],
  "Code": ["代码", "程式碼", "コード"],
  "Archives": ["压缩包", "壓縮檔", "アーカイブ"],
  "Content": ["内容", "內容", "内容"],
  "Today": ["今天", "今天", "今日"],
  "This week": ["本周", "本週", "今週"],
  "Large files": ["大文件", "大型檔案", "大きなファイル"],
  "Downloads": ["下载", "下載", "ダウンロード"],
  "Screenshots": ["截图", "截圖", "スクリーンショット"],
  "Find anything on your computer": ["在电脑上找到任何东西", "在電腦上找到任何東西", "コンピュータのあらゆるものを見つける"],
  "Search inside your files": ["在文件内容里搜索", "在檔案內容裡搜尋", "ファイルの中身を検索"],
  "Language": ["语言", "語言", "言語"],
  "Follow System": ["跟随系统", "跟隨系統", "システムに従う"],
  "Settings": ["设置", "設定", "設定"],
  "General": ["通用", "一般", "一般"],
  "Index": ["索引", "索引", "インデックス"],
  "Tabs": ["标签", "標籤", "タブ"],
  "Content search": ["内容搜索", "內容搜尋", "内容検索"],
  "Color card": ["色卡", "色卡", "カラーカード"],
  "Appearance": ["外观", "外觀", "外観"],
  "Light": ["浅色", "淺色", "ライト"],
  "Dark": ["深色", "深色", "ダーク"],
  "Closing the window keeps JSeek running": ["关闭窗口后 JSeek 继续常驻", "關閉視窗後 JSeek 繼續常駐", "ウインドウを閉じても JSeek は常駐する"],
  "Maximum results": ["最多显示结果", "最多顯示結果", "最大表示件数"],
  "Folder to index": ["索引的文件夹", "索引的檔案夾", "インデックスするフォルダ"],
  "Use / (Mac) or C:/ (Windows) for the whole disk. Changing it rebuilds the index.": ["整盘索引填 /（Mac）或 C:/（Windows）。修改后会重建索引。", "整碟索引填 /（Mac）或 C:/（Windows）。修改後會重建索引。", "ディスク全体は /（Mac）または C:/（Windows）。変更するとインデックスを再作成します。"],
  "Status": ["状态", "狀態", "状態"],
  "Ready": ["就绪", "就緒", "準備完了"],
  "Indexing…": ["正在索引…", "正在索引…", "インデックス作成中…"],
  "items": ["项", "項", "件"],
  "Rebuild index": ["重建索引", "重建索引", "インデックスを再作成"],
  "Building index…": ["正在建立索引…", "正在建立索引…", "インデックスを作成中…"],
  "Updating index…": ["正在更新索引…", "正在更新索引…", "インデックスを更新中…"],
  "Loading index…": ["正在加载索引…", "正在載入索引…", "インデックスを読み込み中…"],
  "Each tab adds its extensions and filter to what you type. Filters use the search syntax, e.g. folder: or size:>10mb.": ["每个标签会把它的扩展名和筛选条件加到你输入的内容后面。筛选条件使用搜索语法，例如 folder: 或 size:>10mb。", "每個標籤會把它的副檔名和篩選條件加到你輸入的內容後面。篩選條件使用搜尋語法，例如 folder: 或 size:>10mb。", "各タブは入力内容に拡張子とフィルタを追加します。フィルタは検索構文で書きます（例：folder:、size:>10mb）。"],
  "Name": ["名称", "名稱", "名前"],
  "Move up": ["上移", "上移", "上へ"],
  "Move down": ["下移", "下移", "下へ"],
  "Remove": ["删除", "刪除", "削除"],
  "Extensions": ["扩展名", "副檔名", "拡張子"],
  "Filter": ["筛选", "篩選", "フィルタ"],
  "e.g. pdf docx md": ["例如 pdf docx md", "例如 pdf docx md", "例：pdf docx md"],
  "Searches file contents; its file types are set under Content search.": ["搜索文件内容；文件类型在「内容搜索」中设置。", "搜尋檔案內容；檔案類型在「內容搜尋」中設定。", "ファイルの中身を検索します。対象の種類は「内容検索」で設定します。"],
  "Add tab": ["添加标签", "新增標籤", "タブを追加"],
  "New tab": ["新标签", "新標籤", "新しいタブ"],
  "QUICK chips add a query with one click or ⌃⌥1–9.": ["QUICK 快捷词：点击或按 ⌃⌥1–9 加入查询。", "QUICK 快捷詞：點擊或按 ⌃⌥1–9 加入查詢。", "QUICK チップ：クリックまたは ⌃⌥1–9 でクエリを追加します。"],
  "Query": ["查询", "查詢", "クエリ"],
  "New": ["新建", "新增", "新規"],
  "Add chip": ["添加快捷词", "新增快捷詞", "チップを追加"],
  "File types to search inside": ["要搜索内容的文件类型", "要搜尋內容的檔案類型", "中身を検索するファイルの種類"],
  "Skip files larger than (MB)": ["跳过大于此大小的文件（MB）", "略過大於此大小的檔案（MB）", "これより大きいファイルは読まない（MB）"],
  "Tip: add filters to narrow the files read, e.g. TODO path:src ext:rs": ["提示：加上筛选条件可缩小读取范围，例如 TODO path:src ext:rs", "提示：加上篩選條件可縮小讀取範圍，例如 TODO path:src ext:rs", "ヒント：フィルタで読むファイルを絞れます（例：TODO path:src ext:rs）"],
  "Reset to defaults": ["恢复默认", "恢復預設", "初期設定に戻す"],
  "Cancel": ["取消", "取消", "キャンセル"],
  "Save": ["保存", "儲存", "保存"],
  "Syntax": ["语法", "語法", "構文"],
  "by extension": ["按扩展名", "按副檔名", "拡張子で"],
  "by size": ["按大小", "按大小", "サイズで"],
  "modified this week": ["本周修改过", "本週修改過", "今週変更"],
  "inside a folder name": ["路径中含某文件夹名", "路徑中含某檔案夾名", "フォルダ名を含むパス"],
  "either / exclude": ["或 / 排除", "或 / 排除", "いずれか / 除外"],
  "exact phrase": ["精确匹配", "精確比對", "完全一致"],
  "Keyboard": ["键盘", "鍵盤", "キーボード"],
  "move": ["移动", "移動", "移動"],
  "open": ["打开", "開啟", "開く"],
  "show in Finder": ["在访达中显示", "在 Finder 中顯示", "Finder で表示"],
  "show in Explorer": ["在资源管理器中显示", "在檔案總管中顯示", "エクスプローラーで表示"],
  "copy path": ["复制路径", "拷貝路徑", "パスをコピー"],
  "switch tab": ["切换标签", "切換標籤", "タブ切り替え"],
  "QUICK chips": ["QUICK 快捷词", "QUICK 快捷詞", "QUICK チップ"],
  "clear / hide": ["清空 / 隐藏", "清空 / 隱藏", "クリア / 隠す"],
  "No matches.": ["没有匹配的结果。", "沒有符合的結果。", "一致する項目はありません。"],
  "Path copied": ["已复制路径", "已拷貝路徑", "パスをコピーしました"],
  "Could not copy": ["无法复制", "無法拷貝", "コピーできません"],
  "days ago": ["天前", "天前", "日前"],
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
