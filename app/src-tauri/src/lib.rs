//! JSeek desktop app: Tauri shell around the C++ engine (via `jseek-engine`).

mod content;
mod settings;

use jseek_core::SearchHit;
use jseek_engine::Engine;
use serde::Serialize;
use settings::{tab_query, Settings};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct Status {
    /// loading | indexing | ready
    state: String,
    count: u64,
    root: String,
    indexed_at: i64,
    scan_ms: u64,
}

struct AppState {
    engine: RwLock<Engine>,
    settings: RwLock<Settings>,
    status: Mutex<Status>,
    settings_path: PathBuf,
    index_path: PathBuf,
}

type Shared<'a> = State<'a, Arc<AppState>>;

#[derive(Serialize)]
struct AppInfo {
    version: String,
    platform: String,
    backend: String,
}

#[derive(Serialize)]
struct Hit {
    name: String,
    dir: String,
    path: String,
    kind: jseek_core::EntryKind,
    size: u64,
    modified: i64,
}

impl From<SearchHit> for Hit {
    fn from(h: SearchHit) -> Self {
        Hit {
            path: h.full_path(),
            name: h.name,
            dir: h.dir,
            kind: h.kind,
            size: h.size,
            modified: h.modified,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchResult {
    hits: Vec<Hit>,
    content: Vec<content::ContentHit>,
    query: String,
    ms: f64,
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn set_status(app: &AppHandle, st: &AppState, f: impl FnOnce(&mut Status)) {
    let snapshot = {
        let mut s = st.status.lock().unwrap();
        f(&mut s);
        s.clone()
    };
    let _ = app.emit("status", snapshot);
}

/// Full scan on a background thread, then swap the new index in and save it.
fn start_index(app: AppHandle, st: Arc<AppState>) {
    std::thread::spawn(move || {
        let root = st.settings.read().unwrap().scan_root.clone();
        set_status(&app, &st, |s| {
            s.state = "indexing".into();
            s.root = root.clone();
        });
        let t = Instant::now();
        let mut fresh = Engine::new();
        let count = fresh.scan(&root);
        let _ = std::fs::create_dir_all(st.index_path.parent().unwrap());
        fresh.save(&st.index_path);
        *st.engine.write().unwrap() = fresh;
        set_status(&app, &st, |s| {
            s.state = "ready".into();
            s.count = count;
            s.indexed_at = now();
            s.scan_ms = t.elapsed().as_millis() as u64;
        });
    });
}

#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        backend: jseek_engine::backend().to_string(),
    }
}

#[tauri::command]
fn get_status(st: Shared) -> Status {
    st.status.lock().unwrap().clone()
}

#[tauri::command]
fn get_settings(st: Shared) -> Settings {
    st.settings.read().unwrap().clone()
}

#[tauri::command]
fn default_settings() -> Settings {
    Settings::default()
}

#[tauri::command]
fn save_settings(app: AppHandle, st: Shared, settings: Settings) -> Result<(), String> {
    settings
        .save(&st.settings_path)
        .map_err(|e| e.to_string())?;
    let root_changed = {
        let mut cur = st.settings.write().unwrap();
        let changed = cur.scan_root != settings.scan_root;
        *cur = settings;
        changed
    };
    if root_changed {
        start_index(app, st.inner().clone());
    }
    Ok(())
}

#[tauri::command]
fn reindex(app: AppHandle, st: Shared) {
    start_index(app, st.inner().clone());
}

#[tauri::command]
async fn search(st: Shared<'_>, query: String, tab: String) -> Result<SearchResult, String> {
    let st = st.inner().clone();
    // Engine work runs off the main thread so typing stays smooth.
    tauri::async_runtime::spawn_blocking(move || run_search(&st, &query, &tab))
        .await
        .map_err(|e| e.to_string())
}

fn run_search(st: &AppState, query: &str, tab_id: &str) -> SearchResult {
    let t = Instant::now();
    let settings = st.settings.read().unwrap().clone();
    let engine = st.engine.read().unwrap();
    if tab_id == "content" {
        // Words are the text to find; tokens with ':' narrow which files are read.
        let (filters, words): (Vec<&str>, Vec<&str>) =
            query.split_whitespace().partition(|w| w.contains(':'));
        let needle = words.join(" ");
        let mut q = filters.join(" ");
        if !settings.content_exts.is_empty() {
            q = format!("{q} ext:{}", settings.content_exts.join(";"));
        }
        let q = format!("{} file:", q.trim());
        let files = if needle.is_empty() {
            Vec::new()
        } else {
            engine.search(&q, 20_000)
        };
        drop(engine);
        let cands = files
            .into_iter()
            .map(|h| (h.full_path(), h.name, h.dir, h.size, h.modified))
            .collect();
        let content = content::search(cands, &needle, settings.content_max_mb << 20, 200);
        return SearchResult {
            hits: Vec::new(),
            content,
            query: q,
            ms: t.elapsed().as_secs_f64() * 1000.0,
        };
    }
    let tab = settings.tabs.iter().find(|x| x.id == tab_id);
    let q = tab_query(query, tab);
    let hits = if q.is_empty() {
        Vec::new()
    } else {
        engine.search(&q, settings.result_limit)
    };
    SearchResult {
        hits: hits.into_iter().map(Hit::from).collect(),
        content: Vec::new(),
        query: q,
        ms: t.elapsed().as_secs_f64() * 1000.0,
    }
}

fn spawn(cmd: &str, args: &[&str]) -> Result<(), String> {
    std::process::Command::new(cmd)
        .args(args)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Open a file or folder with its default app.
#[tauri::command]
fn open_path(path: String) -> Result<(), String> {
    if cfg!(target_os = "macos") {
        spawn("open", &[&path])
    } else if cfg!(windows) {
        spawn("explorer", &[&path.replace('/', "\\")])
    } else {
        spawn("xdg-open", &[&path])
    }
}

/// Show the item in Finder / Explorer.
#[tauri::command]
fn reveal_path(path: String) -> Result<(), String> {
    if cfg!(target_os = "macos") {
        spawn("open", &["-R", &path])
    } else if cfg!(windows) {
        spawn(
            "explorer",
            &[&format!("/select,{}", path.replace('/', "\\"))],
        )
    } else {
        let parent = std::path::Path::new(&path)
            .parent()
            .map(|p| p.to_string_lossy().into_owned());
        spawn("xdg-open", &[parent.as_deref().unwrap_or("/")])
    }
}

/// Esc in the window: hide it (it stays resident; the Dock icon or shortcut brings it back).
#[tauri::command]
fn hide_main(app: AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .setup(|app| {
            let cfg = app.path().app_config_dir()?;
            let data = app.path().app_data_dir()?;
            let settings_path = cfg.join("settings.json");
            let index_path = data.join("index.bin");
            let settings = Settings::load(&settings_path);
            let mut engine = Engine::new();
            let loaded = index_path.exists() && engine.load(&index_path);
            let count = engine.len();
            let st = Arc::new(AppState {
                engine: RwLock::new(engine),
                status: Mutex::new(Status {
                    state: if loaded { "ready" } else { "loading" }.into(),
                    count,
                    root: settings.scan_root.clone(),
                    ..Default::default()
                }),
                settings: RwLock::new(settings),
                settings_path,
                index_path,
            });
            app.manage(st.clone());
            // Search the saved index right away; refresh it in the background.
            start_index(app.handle().clone(), st);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let hide = window
                    .app_handle()
                    .try_state::<Arc<AppState>>()
                    .map(|s| s.settings.read().unwrap().hide_on_close)
                    .unwrap_or(false);
                if hide {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            get_status,
            get_settings,
            default_settings,
            save_settings,
            reindex,
            search,
            open_path,
            reveal_path,
            hide_main
        ])
        .build(tauri::generate_context!())
        .expect("error while building JSeek");

    app.run(|app, event| {
        // macOS: clicking the Dock icon brings the hidden window back.
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen { .. } = event {
            show_main(app);
        }
        #[cfg(not(target_os = "macos"))]
        let _ = (app, event);
    });
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}
