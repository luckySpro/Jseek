//! Resident shell: tray / menu-bar icon, global shortcut, launch at login, start hidden.

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

const PANEL_WIDTH: f64 = 300.0;

pub fn hide_panel(app: &AppHandle) {
    if let Some(p) = app.get_webview_window("panel") {
        let _ = p.hide();
    }
}

/// Show the menu-bar panel under the tray icon (right click), or hide it if it is open.
fn toggle_panel(app: &AppHandle, rect: tauri::Rect) {
    let Some(panel) = app.get_webview_window("panel") else {
        return;
    };
    if panel.is_visible().unwrap_or(false) {
        let _ = panel.hide();
        return;
    }
    let scale = panel.scale_factor().unwrap_or(1.0);
    let pos = rect.position.to_physical::<f64>(scale);
    let size = rect.size.to_physical::<f64>(scale);
    let width = PANEL_WIDTH * scale;
    let mut x = pos.x + size.width / 2.0 - width / 2.0;
    // Keep the panel on the monitor the icon is on.
    if let Ok(Some(m)) = panel.current_monitor() {
        let left = m.position().x as f64;
        let right = left + m.size().width as f64;
        x = x.clamp(left + 8.0, right - width - 8.0);
    }
    // Menu bar at the top (macOS) → open below; taskbar at the bottom (Windows) → open above.
    let below = pos.y < 200.0;
    let height = panel
        .outer_size()
        .map(|s| s.height as f64)
        .unwrap_or(560.0 * scale);
    let y = if below {
        pos.y + size.height + 4.0
    } else {
        pos.y - height - 4.0
    };
    let _ = panel.set_position(tauri::PhysicalPosition::new(x, y));
    let _ = panel.show();
    let _ = panel.set_focus();
    let _ = app.emit_to("panel", "panel-opened", ());
}

pub fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("JSeek")
        .on_tray_icon_event(|tray, e| {
            if let TrayIconEvent::Click {
                button,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = e
            {
                let app = tray.app_handle();
                match button {
                    MouseButton::Right => toggle_panel(app, rect),
                    MouseButton::Left => {
                        hide_panel(app);
                        toggle_main(app);
                    }
                    _ => {}
                }
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
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
