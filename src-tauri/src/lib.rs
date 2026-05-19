use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebAppWindowConfig {
    width: f64,
    height: f64,
    maximized: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebAppPermissions {
    clipboard: bool,
    shell: bool,
    filesystem: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebApp {
    id: String,
    name: String,
    icon: Option<String>,
    url: String,
    user_agent: Option<String>,
    start_on_boot: Option<bool>,
    tray: Option<bool>,
    window_config: WebAppWindowConfig,
    permissions: WebAppPermissions,
    created_at: u64,
}

fn store_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir.join("webapps.json"))
}

fn read_webapps(app: &AppHandle) -> Result<Vec<WebApp>, String> {
    let path = store_path(app)?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&raw).map_err(|error| error.to_string())
}

fn write_webapps(app: &AppHandle, items: &[WebApp]) -> Result<(), String> {
    let path = store_path(app)?;
    let raw = serde_json::to_string_pretty(items).map_err(|error| error.to_string())?;
    fs::write(path, raw).map_err(|error| error.to_string())
}

#[tauri::command]
fn list_webapps(app: AppHandle) -> Result<Vec<WebApp>, String> {
    read_webapps(&app)
}

#[tauri::command]
fn upsert_webapp(app: AppHandle, webapp: WebApp) -> Result<Vec<WebApp>, String> {
    let mut items = read_webapps(&app)?;
    items.retain(|item| item.id != webapp.id);
    items.insert(0, webapp);
    write_webapps(&app, &items)?;
    Ok(items)
}

#[tauri::command]
fn delete_webapp(app: AppHandle, id: String) -> Result<Vec<WebApp>, String> {
    let mut items = read_webapps(&app)?;
    items.retain(|item| item.id != id);
    write_webapps(&app, &items)?;
    Ok(items)
}

#[tauri::command]
fn launch_webapp(app: AppHandle, id: String) -> Result<(), String> {
    let items = read_webapps(&app)?;
    let item = items
        .into_iter()
        .find(|candidate| candidate.id == id)
        .ok_or_else(|| "WebApp not found".to_string())?;

    let url: Url = item.url.parse().map_err(|error: url::ParseError| error.to_string())?;
    let label = format!("webapp-{}", item.id);

    if let Some(window) = app.get_webview_window(&label) {
        window.set_focus().map_err(|error| error.to_string())?;
        return Ok(());
    }

    let mut builder = WebviewWindowBuilder::new(&app, label, WebviewUrl::External(url))
        .title(item.name)
        .inner_size(item.window_config.width, item.window_config.height);

    if item.window_config.maximized.unwrap_or(false) {
        builder = builder.maximized(true);
    }

    builder.build().map_err(|error| error.to_string())?;
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_webapps,
            upsert_webapp,
            delete_webapp,
            launch_webapp
        ])
        .run(tauri::generate_context!())
        .expect("error while running Bandoo WebForge");
}
