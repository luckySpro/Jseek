// Color cards. Each has a light and a dark set; "appearance" picks one or follows the system.
export interface Palette { paper: string; surface: string; ink: string; muted: string; rule: string; accent: string; header: string; headerInk: string; hl: string }
export interface Card { id: string; name: string; light: Palette; dark: Palette }

export const cards: Card[] = [
  { id: "paper", name: "Claude Paper",
    light: { paper: "#f0eee6", surface: "#faf9f5", ink: "#1f1e1d", muted: "#73726c", rule: "#dad7cc", accent: "#c9512f", header: "#1f1e1d", headerInk: "#f0eee6", hl: "#f3d9c9" },
    dark: { paper: "#1f1e1d", surface: "#2a2927", ink: "#eceae3", muted: "#9c9a92", rule: "#3b3a37", accent: "#d97757", header: "#141413", headerInk: "#eceae3", hl: "#5a3426" } },
  { id: "ink", name: "Ink",
    light: { paper: "#f4f4f2", surface: "#ffffff", ink: "#16181d", muted: "#6b6f78", rule: "#dcdde1", accent: "#2f5fd0", header: "#16181d", headerInk: "#f4f4f2", hl: "#d6e0fa" },
    dark: { paper: "#15171b", surface: "#1e2026", ink: "#e8e9ec", muted: "#9297a1", rule: "#30333b", accent: "#7a9cf0", header: "#0e0f12", headerInk: "#e8e9ec", hl: "#27365c" } },
  { id: "sage", name: "Sage",
    light: { paper: "#eef0e8", surface: "#f9faf5", ink: "#1d221c", muted: "#6c7468", rule: "#d5dacb", accent: "#3e7a4f", header: "#22301f", headerInk: "#eef0e8", hl: "#d3e6d2" },
    dark: { paper: "#1a1e19", surface: "#232822", ink: "#e5e9e1", muted: "#949c8f", rule: "#343b32", accent: "#7fbf8c", header: "#111410", headerInk: "#e5e9e1", hl: "#2c4630" } },
  { id: "sky", name: "Sky",
    light: { paper: "#ebf0f4", surface: "#f8fbfd", ink: "#17212b", muted: "#66737f", rule: "#d1dbe3", accent: "#1c7fae", header: "#14324a", headerInk: "#ebf0f4", hl: "#cfe6f2" },
    dark: { paper: "#151c22", surface: "#1d262e", ink: "#e3eaf0", muted: "#8e9ba6", rule: "#2d3943", accent: "#5bb4dd", header: "#0d1318", headerInk: "#e3eaf0", hl: "#1f4257" } },
  { id: "plum", name: "Plum",
    light: { paper: "#f1edf0", surface: "#fbf9fb", ink: "#221a21", muted: "#75697a", rule: "#ddd3dc", accent: "#8a3f86", header: "#2c1a2b", headerInk: "#f1edf0", hl: "#ecd3ea" },
    dark: { paper: "#1d181c", surface: "#272026", ink: "#ece4ea", muted: "#a093a0", rule: "#3a3139", accent: "#d08ccb", header: "#130f13", headerInk: "#ece4ea", hl: "#4d2a4a" } },
];

const vars: [keyof Palette, string][] = [
  ["paper", "--paper"], ["surface", "--surface"], ["ink", "--ink"], ["muted", "--muted"], ["rule", "--rule"],
  ["accent", "--accent"], ["header", "--header"], ["headerInk", "--header-ink"], ["hl", "--hl"],
];

let media: MediaQueryList | null = null;

export function applyTheme(id: string, appearance: "system" | "light" | "dark") {
  const card = cards.find((c) => c.id === id) ?? cards[0];
  media ??= window.matchMedia("(prefers-color-scheme: dark)");
  const dark = appearance === "dark" || (appearance === "system" && media.matches);
  const p = dark ? card.dark : card.light;
  const root = document.documentElement;
  for (const [k, v] of vars) root.style.setProperty(v, p[k]);
  root.style.colorScheme = dark ? "dark" : "light";
  root.dataset.theme = dark ? "dark" : "light";
}

export function watchSystem(cb: () => void) {
  media ??= window.matchMedia("(prefers-color-scheme: dark)");
  media.addEventListener("change", cb);
}
