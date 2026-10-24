use serde::Serialize;

#[derive(Serialize)]
struct AppInfo {
    version: String,
    platform: String,
    backend: String,
}

/// Version and backend shown in the preview window and the About panel.
#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        backend: format!("{:?}", jseek_platform::default_backend()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![app_info])
        .run(tauri::generate_context!())
        .expect("error while running JSeek");
}
