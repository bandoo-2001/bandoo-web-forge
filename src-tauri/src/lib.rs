mod automation;
mod bridge;
mod models;
mod platform;
mod runtime;
mod storage;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle,
};

use crate::models::{
    AppSettings, AutomationConfig, AutomationRunLog, AutomationRunResult, BridgeRequest,
    BridgeResponse, DesktopIntegrationResult, DesktopIntegrationStatus, PromptTemplate,
    RuntimeInfo, ThemePreset, UserScriptConfig, UserScriptRunResult, WebApp,
};

#[tauri::command]
fn runtime_info() -> RuntimeInfo {
    platform::runtime_info()
}

#[tauri::command]
fn list_webapps(app: AppHandle) -> Result<Vec<WebApp>, String> {
    storage::read_webapps(&app)
}

#[tauri::command]
fn upsert_webapp(app: AppHandle, webapp: WebApp) -> Result<Vec<WebApp>, String> {
    storage::upsert_webapp(&app, webapp)
}

#[tauri::command]
fn delete_webapp(app: AppHandle, id: String) -> Result<Vec<WebApp>, String> {
    let items = storage::delete_webapp(&app, &id)?;
    let _ = platform::remove_all_desktop_entries(&id);
    Ok(items)
}

#[tauri::command]
fn install_desktop_entry(
    app: AppHandle,
    id: String,
    target: String,
) -> Result<DesktopIntegrationResult, String> {
    let target = platform::DesktopIntegrationTarget::parse(&target)?;
    let webapp = storage::read_webapps(&app)?
        .into_iter()
        .find(|candidate| candidate.id == id)
        .ok_or_else(|| "WebApp not found".to_string())?;
    platform::install_desktop_entry(&app, &webapp, target)
}

#[tauri::command]
fn remove_desktop_entry(id: String, target: String) -> Result<DesktopIntegrationResult, String> {
    let target = platform::DesktopIntegrationTarget::parse(&target)?;
    platform::remove_desktop_entry(&id, target)
}

#[tauri::command]
fn desktop_integration_statuses(id: String) -> Result<Vec<DesktopIntegrationStatus>, String> {
    platform::desktop_integration_statuses(&id)
}

#[tauri::command]
async fn launch_webapp(app: AppHandle, id: String) -> Result<(), String> {
    runtime::launch_webapp(app, id)
}

#[tauri::command]
fn start_selector_capture(app: AppHandle, web_app_id: String) -> Result<(), String> {
    runtime::start_selector_capture(app, web_app_id)
}

#[tauri::command]
fn start_action_recording(app: AppHandle, web_app_id: String) -> Result<(), String> {
    runtime::start_action_recording(app, web_app_id)
}

#[tauri::command]
fn list_automations(app: AppHandle) -> Result<Vec<AutomationConfig>, String> {
    storage::read_automations(&app)
}

#[tauri::command]
fn upsert_automation(
    app: AppHandle,
    automation: AutomationConfig,
) -> Result<Vec<AutomationConfig>, String> {
    let mut items = storage::read_automations(&app)?;
    items.retain(|item| item.id != automation.id);
    items.insert(0, automation);
    automation::validate_shortcuts(&items)?;
    storage::write_automations(&app, &items)?;
    automation::refresh_shortcuts(&app)?;
    Ok(items)
}

#[tauri::command]
fn delete_automation(app: AppHandle, id: String) -> Result<Vec<AutomationConfig>, String> {
    let items = storage::delete_automation(&app, &id)?;
    automation::refresh_shortcuts(&app)?;
    Ok(items)
}

#[tauri::command]
async fn execute_automation(
    app: AppHandle,
    automation: AutomationConfig,
) -> Result<AutomationRunResult, String> {
    runtime::execute_automation(app, automation)
}

#[tauri::command]
fn list_user_scripts(app: AppHandle) -> Result<Vec<UserScriptConfig>, String> {
    storage::read_user_scripts(&app)
}

#[tauri::command]
fn upsert_user_script(
    app: AppHandle,
    script: UserScriptConfig,
) -> Result<Vec<UserScriptConfig>, String> {
    storage::upsert_user_script(&app, script)
}

#[tauri::command]
fn delete_user_script(app: AppHandle, id: String) -> Result<Vec<UserScriptConfig>, String> {
    storage::delete_user_script(&app, &id)
}

#[tauri::command]
async fn execute_user_script(
    app: AppHandle,
    script: UserScriptConfig,
) -> Result<UserScriptRunResult, String> {
    runtime::execute_user_script(app, script)
}

#[tauri::command]
fn list_prompt_templates(app: AppHandle) -> Result<Vec<PromptTemplate>, String> {
    storage::read_prompt_templates(&app)
}

#[tauri::command]
fn upsert_prompt_template(
    app: AppHandle,
    template: PromptTemplate,
) -> Result<Vec<PromptTemplate>, String> {
    storage::upsert_prompt_template(&app, template)
}

#[tauri::command]
fn delete_prompt_template(app: AppHandle, id: String) -> Result<Vec<PromptTemplate>, String> {
    storage::delete_prompt_template(&app, &id)
}

#[tauri::command]
fn list_theme_presets(app: AppHandle) -> Result<Vec<ThemePreset>, String> {
    storage::read_theme_presets(&app)
}

#[tauri::command]
fn upsert_theme_preset(app: AppHandle, preset: ThemePreset) -> Result<Vec<ThemePreset>, String> {
    storage::upsert_theme_preset(&app, preset)
}

#[tauri::command]
fn delete_theme_preset(app: AppHandle, id: String) -> Result<Vec<ThemePreset>, String> {
    storage::delete_theme_preset(&app, &id)
}

#[tauri::command]
fn app_settings(app: AppHandle) -> Result<AppSettings, String> {
    storage::read_app_settings(&app)
}

#[tauri::command]
fn save_app_settings(app: AppHandle, settings: AppSettings) -> Result<AppSettings, String> {
    storage::write_app_settings(&app, &settings)
}

#[tauri::command]
fn list_run_logs(app: AppHandle) -> Result<Vec<AutomationRunLog>, String> {
    storage::read_run_logs(&app)
}

#[tauri::command]
fn bridge_request(app: AppHandle, request: BridgeRequest) -> Result<BridgeResponse, String> {
    bridge::handle(app, request)
}

fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show-main", "Show Bandoo WebForge", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    let mut builder = TrayIconBuilder::with_id("bandoo-webforge")
        .menu(&menu)
        .tooltip("Bandoo WebForge")
        .on_menu_event(|app, event| {
            if event.id() == "show-main" {
                let _ = runtime::show_main_window(app);
            } else if event.id() == "quit" {
                app.exit(0);
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            automation::init_global_shortcuts(&handle).map_err(|error| {
                Box::<dyn std::error::Error>::from(std::io::Error::other(error))
            })?;
            setup_tray(&handle)?;
            runtime::launch_cli_webapp(handle).map_err(|error| {
                Box::<dyn std::error::Error>::from(std::io::Error::other(error))
            })?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            runtime_info,
            list_webapps,
            upsert_webapp,
            delete_webapp,
            install_desktop_entry,
            remove_desktop_entry,
            desktop_integration_statuses,
            launch_webapp,
            start_selector_capture,
            start_action_recording,
            list_automations,
            upsert_automation,
            delete_automation,
            execute_automation,
            list_user_scripts,
            upsert_user_script,
            delete_user_script,
            execute_user_script,
            list_prompt_templates,
            upsert_prompt_template,
            delete_prompt_template,
            list_theme_presets,
            upsert_theme_preset,
            delete_theme_preset,
            app_settings,
            save_app_settings,
            list_run_logs,
            bridge_request
        ])
        .run(tauri::generate_context!())
        .expect("error while running Bandoo WebForge");
}
