//! Resident shell: tray / menu-bar icon, global shortcut, launch at login, start hidden.

use crate::AppState;
use std::sync::Arc;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

/// Command-line flag used by the login item so JSeek starts in the tray.
pub const HIDDEN_FLAG: &str = "--hidden";

pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
        let _ = app.emit("focus-search", ());
    }
}

pub fn toggle_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let visible = w.is_visible().unwrap_or(false) && w.is_focused().unwrap_or(false);
        if visible {
            let _ = w.hide();
        } else {
            show_main(app);
        }
    }
}

/// Tray menu labels in the series languages (en, zh-Hans, zh-Hant, ja).
fn labels(lang: &str) -> [&'static str; 4] {
    match lang {
        "zh-Hans" => ["显示 JSeek", "重建索引", "设置…", "退出 JSeek"],
        "zh-Hant" => ["顯示 JSeek", "重建索引", "設定…", "結束 JSeek"],
        "ja" => [
            "JSeek を表示",
            "インデックスを再作成",
            "設定…",
            "JSeek を終了",
        ],
        _ => ["Show JSeek", "Rebuild Index", "Settings…", "Quit JSeek"],
    }
}

pub struct TrayItems {
    show: MenuItem<tauri::Wry>,
    reindex: MenuItem<tauri::Wry>,
    settings: MenuItem<tauri::Wry>,
    quit: MenuItem<tauri::Wry>,
}

impl TrayItems {
    pub fn set_lang(&self, lang: &str) {
        let l = labels(lang);
        let _ = self.show.set_text(l[0]);
        let _ = self.reindex.set_text(l[1]);
        let _ = self.settings.set_text(l[2]);
        let _ = self.quit.set_text(l[3]);
    }
}

pub fn build_tray(app: &AppHandle) -> tauri::Result<TrayItems> {
    let l = labels("en");
    let show = MenuItem::with_id(app, "show", l[0], true, None::<&str>)?;
    let reindex = MenuItem::with_id(app, "reindex", l[1], true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", l[2], true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", l[3], true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &show,
            &PredefinedMenuItem::separator(app)?,
            &reindex,
            &settings,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;
    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("JSeek")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, e| match e.id.as_ref() {
            "show" => show_main(app),
            "reindex" => {
                if let Some(st) = app.try_state::<Arc<AppState>>() {
                    crate::start_index(app.clone(), st.inner().clone());
                }
            }
            "settings" => {
                show_main(app);
                let _ = app.emit("open-settings", ());
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, e| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = e
            {
                toggle_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(TrayItems {
        show,
        reindex,
        settings,
        quit,
    })
}

/// Replace the global shortcut. Empty string = none.
pub fn set_hotkey(app: &AppHandle, hotkey: &str) -> Result<(), String> {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    if hotkey.trim().is_empty() {
        return Ok(());
    }
    gs.register(hotkey.trim())
        .map_err(|e| format!("{hotkey}: {e}"))
}

pub fn set_autostart(app: &AppHandle, on: bool) -> Result<(), String> {
    let al = app.autolaunch();
    let cur = al.is_enabled().unwrap_or(false);
    if on == cur {
        return Ok(());
    }
    if on { al.enable() } else { al.disable() }.map_err(|e| e.to_string())
}
