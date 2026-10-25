//! User settings, stored as JSON in the app config folder. Everything here is editable in the
//! Settings page; unknown or missing fields fall back to defaults so old files keep working.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// A header tab. `exts` empty + `filter` empty = everything.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Tab {
    pub id: String,
    /// English label (translated in the UI) or the user's own text.
    pub name: String,
    /// Extension list without dots, e.g. ["pdf", "docx"].
    #[serde(default)]
    pub exts: Vec<String>,
    /// Extra engine filter, e.g. "folder:".
    #[serde(default)]
    pub filter: String,
}

/// A QUICK chip: one click (or Ctrl+Alt+N) inserts its query.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Quick {
    pub name: String,
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Folder that is indexed ("/" or "C:/" = whole disk).
    pub scan_root: String,
    pub tabs: Vec<Tab>,
    pub quick: Vec<Quick>,
    /// Extensions whose contents the Content tab searches.
    pub content_exts: Vec<String>,
    /// Largest file read by content search, in MB.
    pub content_max_mb: u64,
    /// Color card id: paper, ink, sage, sky, plum.
    pub theme: String,
    /// light | dark | system
    pub appearance: String,
    pub lang: String,
    pub hide_on_close: bool,
    pub result_limit: u32,
    /// Global shortcut that shows/hides the window, e.g. "Alt+Space". Empty = none.
    pub hotkey: String,
    pub launch_at_login: bool,
}

pub fn default_hotkey() -> String {
    if cfg!(target_os = "macos") {
        "Alt+Space"
    } else {
        "Ctrl+Alt+Space"
    }
    .into()
}

fn strs(s: &str) -> Vec<String> {
    s.split_whitespace().map(String::from).collect()
}

fn tab(id: &str, name: &str, exts: &str, filter: &str) -> Tab {
    Tab {
        id: id.into(),
        name: name.into(),
        exts: strs(exts),
        filter: filter.into(),
    }
}

fn quick(name: &str, query: &str) -> Quick {
    Quick {
        name: name.into(),
        query: query.into(),
    }
}

pub fn default_root() -> String {
    if cfg!(windows) {
        "C:/".into()
    } else if cfg!(target_os = "macos") {
        "/".into()
    } else {
        std::env::var("HOME").unwrap_or_else(|_| "/".into())
    }
}

impl Default for Settings {
    fn default() -> Self {
        let apps = if cfg!(windows) { "exe lnk msi" } else { "app" };
        Settings {
            scan_root: default_root(),
            tabs: vec![
                tab("all", "All", "", ""),
                tab("apps", "Apps", apps, ""),
                tab("folders", "Folders", "", "folder:"),
                tab("docs", "Documents", "pdf doc docx xls xlsx ppt pptx key pages numbers txt md rtf csv epub", ""),
                tab("images", "Images", "jpg jpeg png gif heic webp svg bmp tif tiff psd raw ico", ""),
                tab("media", "Audio & Video", "mp3 m4a wav flac aac ogg mp4 mov mkv avi webm m4v", ""),
                tab("code", "Code", "rs ts tsx js jsx vue py go c cc cpp h hpp swift kt java rb php cs sh sql json yaml yml toml", ""),
                tab("archives", "Archives", "zip rar 7z tar gz tgz bz2 xz dmg iso", ""),
                tab("content", "Content", "", ""),
            ],
            quick: vec![
                quick("Today", "dm:today"),
                quick("This week", "dm:thisweek"),
                quick("Large files", "file: size:>100mb"),
                quick("PDF", "ext:pdf"),
                quick("Downloads", "path:Downloads"),
                quick("Screenshots", "screenshot pic:"),
            ],
            content_exts: strs("txt md markdown rs ts tsx js jsx vue py go c cc cpp h hpp swift kt java rb php cs sh sql json yaml yml toml ini cfg conf log csv html css xml"),
            content_max_mb: 4,
            theme: "paper".into(),
            appearance: "system".into(),
            lang: "system".into(),
            hide_on_close: true,
            result_limit: 500,
            hotkey: default_hotkey(),
            launch_at_login: false,
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Settings {
        std::fs::read(path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(self).unwrap_or_default())?;
        std::fs::rename(tmp, path)
    }
}

/// Engine query for a tab: user text, then the tab's extension list and filter.
pub fn tab_query(user: &str, tab: Option<&Tab>) -> String {
    let mut q = user.trim().to_string();
    if let Some(t) = tab {
        if !t.exts.is_empty() {
            q.push_str(" ext:");
            q.push_str(&t.exts.join(";"));
        }
        if !t.filter.is_empty() {
            q.push(' ');
            q.push_str(&t.filter);
        }
    }
    q.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_query_appends_exts_and_filter() {
        let t = tab("x", "X", "pdf docx", "file:");
        assert_eq!(tab_query(" report ", Some(&t)), "report ext:pdf;docx file:");
        assert_eq!(tab_query("a", None), "a");
        assert_eq!(
            tab_query("", Some(&tab("f", "F", "", "folder:"))),
            "folder:"
        );
    }

    #[test]
    fn partial_json_keeps_defaults() {
        let s: Settings = serde_json::from_str(r#"{"theme":"sage"}"#).unwrap();
        assert_eq!(s.theme, "sage");
        assert!(!s.tabs.is_empty());
        assert_eq!(s.result_limit, 500);
    }
}
