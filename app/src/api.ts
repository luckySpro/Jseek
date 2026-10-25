// Typed wrappers over the Tauri commands. In a plain browser (vite dev) a small mock answers,
// so the layout can be worked on without the native shell.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type Kind = "file" | "folder" | "symlink" | "app" | "other";
export interface Hit { name: string; dir: string; path: string; kind: Kind; size: number; modified: number }
export interface LineHit { line: number; text: string }
export interface ContentHit { path: string; name: string; dir: string; size: number; modified: number; lines: LineHit[] }
export interface SearchResult { hits: Hit[]; content: ContentHit[]; query: string; ms: number }
export interface Tab { id: string; name: string; exts: string[]; filter: string }
export interface Quick { name: string; query: string }
export interface Settings {
  scanRoot: string; tabs: Tab[]; quick: Quick[]; contentExts: string[]; contentMaxMb: number;
  theme: string; appearance: "system" | "light" | "dark"; lang: string; hideOnClose: boolean; resultLimit: number;
}
export interface Status { state: "loading" | "indexing" | "ready"; count: number; root: string; indexedAt: number; scanMs: number }
export interface AppInfo { version: string; platform: string; backend: string }

export const native = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

const mockSettings: Settings = {
  scanRoot: "/", contentExts: ["txt", "md", "rs"], contentMaxMb: 4, theme: "paper", appearance: "system",
  lang: "system", hideOnClose: true, resultLimit: 500,
  tabs: [
    { id: "all", name: "All", exts: [], filter: "" },
    { id: "folders", name: "Folders", exts: [], filter: "folder:" },
    { id: "docs", name: "Documents", exts: ["pdf", "docx", "md"], filter: "" },
    { id: "content", name: "Content", exts: [], filter: "" },
  ],
  quick: [{ name: "Today", query: "dm:today" }, { name: "PDF", query: "ext:pdf" }],
};

async function mock(cmd: string, args?: any): Promise<any> {
  const now = Date.now() / 1000;
  switch (cmd) {
    case "app_info": return { version: "dev", platform: "browser", backend: "mock" };
    case "get_status": return { state: "ready", count: 3, root: "/", indexedAt: now, scanMs: 12 };
    case "get_settings": case "default_settings": return structuredClone(mockSettings);
    case "search":
      if (args.tab === "content") return { hits: [], ms: 1, query: args.query, content: [
        { path: "/Users/me/notes/todo.md", name: "todo.md", dir: "/Users/me/notes", size: 812, modified: now - 3600,
          lines: [{ line: 4, text: `- check ${args.query} before release` }] }] };
      return { content: [], ms: 0.4, query: args.query, hits: [
        { name: `${args.query || "report"}.pdf`, dir: "/Users/me/Documents", path: "/Users/me/Documents/report.pdf", kind: "file", size: 248_000, modified: now - 7200 },
        { name: "Projects", dir: "/Users/me", path: "/Users/me/Projects", kind: "folder", size: 0, modified: now - 86400 * 3 },
        { name: "Safari.app", dir: "/Applications", path: "/Applications/Safari.app", kind: "app", size: 0, modified: now - 86400 * 40 },
      ] };
    default: return null;
  }
}

function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return native ? invoke<T>(cmd, args) : mock(cmd, args);
}

export const api = {
  info: () => call<AppInfo>("app_info"),
  status: () => call<Status>("get_status"),
  settings: () => call<Settings>("get_settings"),
  defaults: () => call<Settings>("default_settings"),
  saveSettings: (settings: Settings) => call<void>("save_settings", { settings }),
  reindex: () => call<void>("reindex"),
  search: (query: string, tab: string) => call<SearchResult>("search", { query, tab }),
  open: (path: string) => call<void>("open_path", { path }),
  reveal: (path: string) => call<void>("reveal_path", { path }),
  hide: () => call<void>("hide_main"),
  onStatus: (cb: (s: Status) => void) =>
    native ? listen<Status>("status", (e) => cb(e.payload)) : Promise.resolve(() => {}),
};
