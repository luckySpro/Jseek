import { t } from "./i18n";
import type { Kind } from "./api";

export function fmtSize(n: number): string {
  if (n < 1024) return `${n} B`;
  const u = ["KB", "MB", "GB", "TB"];
  let v = n / 1024, i = 0;
  while (v >= 1024 && i < u.length - 1) { v /= 1024; i++; }
  return `${v < 10 ? v.toFixed(1) : Math.round(v)} ${u[i]}`;
}

export function fmtDate(sec: number): string {
  if (!sec) return "";
  const d = new Date(sec * 1000), now = new Date();
  const days = Math.floor((now.getTime() - d.getTime()) / 86400000);
  const pad = (x: number) => String(x).padStart(2, "0");
  if (d.toDateString() === now.toDateString()) return `${t("Today")} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
  if (days < 7 && days >= 0) return `${days || 1} ${t("days ago")}`;
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

const esc = (s: string) => s.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]!);

/** Plain words of a query (filters like ext:pdf and operators dropped), for highlighting. */
export function terms(q: string): string[] {
  return q
    .split(/\s+/)
    .filter((w) => w && !w.includes(":") && !/^(and|or|not)$/i.test(w))
    .map((w) => w.replace(/^[!-]|["*?]/g, ""))
    .filter((w) => w.length > 0);
}

/** HTML-escaped text with each term wrapped in <mark>, case-insensitive. */
export function highlight(text: string, words: string[]): string {
  if (!words.length) return esc(text);
  const lower = text.toLowerCase();
  const marks: [number, number][] = [];
  for (const w of words) {
    const lw = w.toLowerCase();
    for (let i = lower.indexOf(lw); i >= 0; i = lower.indexOf(lw, i + lw.length)) marks.push([i, i + lw.length]);
  }
  if (!marks.length) return esc(text);
  marks.sort((a, b) => a[0] - b[0]);
  let out = "", pos = 0;
  for (const [s, e] of marks) {
    if (e <= pos) continue;
    const start = Math.max(s, pos);
    out += esc(text.slice(pos, start)) + "<mark>" + esc(text.slice(start, e)) + "</mark>";
    pos = e;
  }
  return out + esc(text.slice(pos));
}

const groups: Record<string, string[]> = {
  doc: ["pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "key", "pages", "numbers", "txt", "md", "rtf", "csv", "epub"],
  img: ["jpg", "jpeg", "png", "gif", "heic", "webp", "svg", "bmp", "tif", "tiff", "psd", "raw", "ico"],
  media: ["mp3", "m4a", "wav", "flac", "aac", "ogg", "mp4", "mov", "mkv", "avi", "webm", "m4v"],
  code: ["rs", "ts", "tsx", "js", "jsx", "vue", "py", "go", "c", "cc", "cpp", "h", "hpp", "swift", "kt", "java", "rb", "php", "cs", "sh", "sql", "json", "yaml", "yml", "toml", "html", "css"],
  zip: ["zip", "rar", "7z", "tar", "gz", "tgz", "bz2", "xz", "dmg", "iso"],
};

/** Small colored label for a row: the extension, or a kind glyph. */
export function badge(name: string, kind: Kind): { k: string; label: string } {
  if (kind === "folder") return { k: "folder", label: "DIR" };
  if (kind === "app") return { k: "app", label: "APP" };
  if (kind === "symlink") return { k: "other", label: "LNK" };
  const dot = name.lastIndexOf(".");
  const ext = dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
  const k = Object.keys(groups).find((g) => groups[g].includes(ext)) ?? "other";
  return { k, label: ext ? ext.slice(0, 4) : "—" };
}
