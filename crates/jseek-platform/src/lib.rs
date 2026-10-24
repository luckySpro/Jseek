//! Platform adapters that feed the index. M0 defines the contract; scanners land in M2 (macOS) and M4 (Windows).

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("not supported on this platform")]
    Unsupported,
    #[error("{0}")]
    Other(String),
}

/// Which mechanism a source uses — shown in diagnostics so users know the real backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// macOS: getattrlistbulk full scan + FSEvents.
    MacFsEvents,
    /// Windows: NTFS MFT enumeration + USN journal via the jseek-indexer service.
    WindowsMftUsn,
    /// Fallback: directory walk + change notifications (slower).
    Walk,
}

/// A source of file records for the index.
pub trait FileSource {
    fn backend(&self) -> Backend;
    /// Human-readable name for diagnostics.
    fn describe(&self) -> String;
}

/// The best source available on this platform today.
pub fn default_backend() -> Backend {
    if cfg!(target_os = "macos") {
        Backend::MacFsEvents
    } else if cfg!(target_os = "windows") {
        Backend::WindowsMftUsn
    } else {
        Backend::Walk
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_backend_is_defined() {
        let _ = super::default_backend();
    }
}
