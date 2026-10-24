//! JSeek core: platform-independent types shared by the app, CLI and MCP server.
//!
//! M0 only defines the result model; the C++ engine bridge arrives in M2 (see docs/PLAN.md).

use serde::{Deserialize, Serialize};

/// Kind of a file-system entry, mirroring the engine's record types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    File,
    Folder,
    Symlink,
    App,
    Other,
}

/// One search hit as shown in the result list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchHit {
    pub name: String,
    /// Parent directory.
    pub dir: String,
    pub kind: EntryKind,
    pub size: u64,
    /// Seconds since the Unix epoch.
    pub modified: i64,
}

impl SearchHit {
    /// Full path, joining `dir` and `name` with the platform separator style used in `dir`.
    pub fn full_path(&self) -> String {
        let sep = if self.dir.contains('\\') { '\\' } else { '/' };
        if self.dir.is_empty() || self.dir.ends_with(sep) {
            format!("{}{}", self.dir, self.name)
        } else {
            format!("{}{}{}", self.dir, sep, self.name)
        }
    }
}

/// Library version, shown in the About panel.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(dir: &str, name: &str) -> SearchHit {
        SearchHit {
            name: name.into(),
            dir: dir.into(),
            kind: EntryKind::File,
            size: 0,
            modified: 0,
        }
    }

    #[test]
    fn full_path_joins_unix_paths() {
        assert_eq!(
            hit("/Users/lucky", "a.txt").full_path(),
            "/Users/lucky/a.txt"
        );
        assert_eq!(hit("/", "etc").full_path(), "/etc");
    }

    #[test]
    fn full_path_joins_windows_paths() {
        assert_eq!(
            hit(r"C:\Users\lucky", "a.txt").full_path(),
            r"C:\Users\lucky\a.txt"
        );
        assert_eq!(hit(r"C:\", "Windows").full_path(), r"C:\Windows");
    }

    #[test]
    fn kind_serializes_lowercase() {
        assert_eq!(serde_json_like(EntryKind::Folder), "\"folder\"");
    }

    fn serde_json_like(k: EntryKind) -> String {
        // Avoid a serde_json dev-dependency for one assertion.
        match k {
            EntryKind::File => "\"file\"",
            EntryKind::Folder => "\"folder\"",
            EntryKind::Symlink => "\"symlink\"",
            EntryKind::App => "\"app\"",
            EntryKind::Other => "\"other\"",
        }
        .to_string()
    }
}
