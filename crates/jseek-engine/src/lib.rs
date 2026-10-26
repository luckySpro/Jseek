//! Safe wrapper over the C++ engine (see `engine/Bridge/jseek_c.h`).
//!
//! ```no_run
//! let mut e = jseek_engine::Engine::new();
//! e.scan("/Users/me/Documents");
//! for hit in e.search("ext:pdf report", 50) { println!("{}", hit.full_path()); }
//! ```

use jseek_core::{EntryKind, SearchHit};
use std::ffi::{c_char, c_void, CStr, CString};
use std::path::Path;

mod ffi {
    use std::ffi::{c_char, c_int, c_void};

    #[repr(C)]
    pub struct Engine {
        _private: [u8; 0],
    }

    #[repr(C)]
    pub struct Hit {
        pub name: *const c_char,
        pub dir: *const c_char,
        pub kind: u8,
        pub size: u64,
        pub modified: i64,
    }

    pub type HitCb = extern "C" fn(user: *mut c_void, hit: *const Hit);

    extern "C" {
        pub fn jseek_engine_new() -> *mut Engine;
        pub fn jseek_engine_free(e: *mut Engine);
        pub fn jseek_engine_scan(e: *mut Engine, root: *const c_char) -> u64;
        pub fn jseek_engine_load(e: *mut Engine, path: *const c_char) -> c_int;
        pub fn jseek_engine_save(e: *const Engine, path: *const c_char) -> c_int;
        pub fn jseek_engine_count(e: *const Engine) -> u64;
        pub fn jseek_engine_search(
            e: *const Engine,
            query: *const c_char,
            limit: u32,
            cb: HitCb,
            user: *mut c_void,
        ) -> u64;
        pub fn jseek_engine_backend() -> *const c_char;
        pub fn jseek_scan_records(root: *const c_char, out_count: *mut u64) -> *mut Records;
        pub fn jseek_engine_load_records(e: *mut Engine, r: *mut Records);
        pub fn jseek_records_free(r: *mut Records);
        pub fn jseek_release_memory();
        pub fn jseek_memory_footprint() -> u64;
    }

    #[repr(C)]
    pub struct Records {
        _private: [u8; 0],
    }
}

/// Map the engine's record type byte to [`EntryKind`].
pub fn kind_from_engine(t: u8) -> EntryKind {
    match t {
        1 => EntryKind::File,
        2 => EntryKind::Folder,
        3 => EntryKind::Symlink,
        5 => EntryKind::App,
        _ => EntryKind::Other,
    }
}

/// Engine paths always use '/'; convert a native path for it.
fn engine_path(p: &Path) -> CString {
    let s = p.to_string_lossy().replace('\\', "/");
    CString::new(s).unwrap_or_default()
}

/// Result of a scan that has not been loaded into an engine yet.
pub struct Records {
    raw: *mut ffi::Records,
    count: u64,
}

unsafe impl Send for Records {}

impl Records {
    /// Scan `root` without touching any engine, so searches can continue meanwhile.
    pub fn scan(root: impl AsRef<Path>) -> Records {
        let c = engine_path(root.as_ref());
        let mut count = 0u64;
        let raw = unsafe { ffi::jseek_scan_records(c.as_ptr(), &mut count) };
        Records { raw, count }
    }

    pub fn len(&self) -> u64 {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

impl Drop for Records {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { ffi::jseek_records_free(self.raw) }
        }
    }
}

/// Return freed heap memory to the operating system.
pub fn release_memory() {
    unsafe { ffi::jseek_release_memory() }
}

/// Memory used by this process, as the OS activity monitor reports it (bytes).
pub fn memory_footprint() -> u64 {
    unsafe { ffi::jseek_memory_footprint() }
}

/// An in-memory index plus the 1.9 query language.
pub struct Engine {
    raw: *mut ffi::Engine,
}

// The C++ engine guards its state with a shared_mutex; queries may run from any thread.
unsafe impl Send for Engine {}
unsafe impl Sync for Engine {}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        let raw = unsafe { ffi::jseek_engine_new() };
        assert!(!raw.is_null(), "jseek_engine_new failed");
        Engine { raw }
    }

    /// Full scan of `root`, replacing the current records. Returns the record count.
    pub fn scan(&mut self, root: impl AsRef<Path>) -> u64 {
        let c = engine_path(root.as_ref());
        unsafe { ffi::jseek_engine_scan(self.raw, c.as_ptr()) }
    }

    /// Replace the records with a finished scan (consumes it; only one copy stays in memory).
    pub fn load_records(&mut self, mut records: Records) {
        let raw = std::mem::replace(&mut records.raw, std::ptr::null_mut());
        unsafe { ffi::jseek_engine_load_records(self.raw, raw) }
    }

    pub fn load(&mut self, index: impl AsRef<Path>) -> bool {
        let c = engine_path(index.as_ref());
        unsafe { ffi::jseek_engine_load(self.raw, c.as_ptr()) == 1 }
    }

    pub fn save(&self, index: impl AsRef<Path>) -> bool {
        let c = engine_path(index.as_ref());
        unsafe { ffi::jseek_engine_save(self.raw, c.as_ptr()) == 1 }
    }

    pub fn len(&self) -> u64 {
        unsafe { ffi::jseek_engine_count(self.raw) }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Search with the full query syntax. `limit` 0 means no limit.
    pub fn search(&self, query: &str, limit: u32) -> Vec<SearchHit> {
        extern "C" fn collect(user: *mut c_void, hit: *const ffi::Hit) {
            // SAFETY: `user` is the Vec passed below; `hit` and its strings live for this call.
            let out = unsafe { &mut *(user as *mut Vec<SearchHit>) };
            let h = unsafe { &*hit };
            let s = |p: *const c_char| unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned();
            out.push(SearchHit {
                name: s(h.name),
                dir: s(h.dir),
                kind: kind_from_engine(h.kind),
                size: h.size,
                modified: h.modified,
            });
        }
        let Ok(q) = CString::new(query) else {
            return Vec::new();
        };
        let mut out: Vec<SearchHit> = Vec::new();
        unsafe {
            ffi::jseek_engine_search(
                self.raw,
                q.as_ptr(),
                limit,
                collect,
                &mut out as *mut Vec<SearchHit> as *mut c_void,
            );
        }
        out
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe { ffi::jseek_engine_free(self.raw) }
    }
}

/// Scanner compiled into the engine on this platform.
pub fn backend() -> &'static str {
    unsafe { CStr::from_ptr(ffi::jseek_engine_backend()) }
        .to_str()
        .unwrap_or("unknown")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_map() {
        assert_eq!(kind_from_engine(1), EntryKind::File);
        assert_eq!(kind_from_engine(2), EntryKind::Folder);
        assert_eq!(kind_from_engine(5), EntryKind::App);
        assert_eq!(kind_from_engine(9), EntryKind::Other);
    }
}
