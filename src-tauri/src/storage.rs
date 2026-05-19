use std::{fs, path::PathBuf};

use tauri::{AppHandle, Manager};

use crate::models::{
    AutomationConfig, PromptTemplate, UserScriptConfig, WebApp, WebAppWindowState,
};

pub fn store_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir)
}

fn store_path(app: &AppHandle, file_name: &str) -> Result<PathBuf, String> {
    Ok(store_dir(app)?.join(file_name))
}

fn read_collection<T>(app: &AppHandle, file_name: &str) -> Result<Vec<T>, String>
where
    T: serde::de::DeserializeOwned,
{
    let path = store_path(app, file_name)?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&raw).map_err(|error| error.to_string())
}

fn write_collection<T>(app: &AppHandle, file_name: &str, items: &[T]) -> Result<(), String>
where
    T: serde::Serialize,
{
    let path = store_path(app, file_name)?;
    let raw = serde_json::to_string_pretty(items).map_err(|error| error.to_string())?;
    fs::write(path, raw).map_err(|error| error.to_string())
}

pub fn read_webapps(app: &AppHandle) -> Result<Vec<WebApp>, String> {
    read_collection(app, "webapps.json")
}

pub fn write_webapps(app: &AppHandle, items: &[WebApp]) -> Result<(), String> {
    write_collection(app, "webapps.json", items)
}

pub fn upsert_webapp(app: &AppHandle, webapp: WebApp) -> Result<Vec<WebApp>, String> {
    let mut items = read_webapps(app)?;
    items.retain(|item| item.id != webapp.id);
    items.insert(0, webapp);
    write_webapps(app, &items)?;
    Ok(items)
}

pub fn delete_webapp(app: &AppHandle, id: &str) -> Result<Vec<WebApp>, String> {
    let mut items = read_webapps(app)?;
    items.retain(|item| item.id != id);
    write_webapps(app, &items)?;
    Ok(items)
}

pub fn update_window_state(
    app: &AppHandle,
    id: &str,
    state: WebAppWindowState,
) -> Result<(), String> {
    let mut items = read_webapps(app)?;
    if let Some(item) = items.iter_mut().find(|candidate| candidate.id == id) {
        item.last_window_state = Some(state);
        write_webapps(app, &items)?;
    }
    Ok(())
}

pub fn read_automations(app: &AppHandle) -> Result<Vec<AutomationConfig>, String> {
    read_collection(app, "automations.json")
}

pub fn upsert_automation(
    app: &AppHandle,
    automation: AutomationConfig,
) -> Result<Vec<AutomationConfig>, String> {
    let mut items = read_automations(app)?;
    items.retain(|item| item.id != automation.id);
    items.insert(0, automation);
    write_collection(app, "automations.json", &items)?;
    Ok(items)
}

pub fn delete_automation(app: &AppHandle, id: &str) -> Result<Vec<AutomationConfig>, String> {
    let mut items = read_automations(app)?;
    items.retain(|item| item.id != id);
    write_collection(app, "automations.json", &items)?;
    Ok(items)
}

pub fn read_user_scripts(app: &AppHandle) -> Result<Vec<UserScriptConfig>, String> {
    read_collection(app, "user-scripts.json")
}

pub fn upsert_user_script(
    app: &AppHandle,
    script: UserScriptConfig,
) -> Result<Vec<UserScriptConfig>, String> {
    let mut items = read_user_scripts(app)?;
    items.retain(|item| item.id != script.id);
    items.insert(0, script);
    write_collection(app, "user-scripts.json", &items)?;
    Ok(items)
}

pub fn delete_user_script(app: &AppHandle, id: &str) -> Result<Vec<UserScriptConfig>, String> {
    let mut items = read_user_scripts(app)?;
    items.retain(|item| item.id != id);
    write_collection(app, "user-scripts.json", &items)?;
    Ok(items)
}

pub fn read_prompt_templates(app: &AppHandle) -> Result<Vec<PromptTemplate>, String> {
    read_collection(app, "prompt-templates.json")
}

pub fn upsert_prompt_template(
    app: &AppHandle,
    template: PromptTemplate,
) -> Result<Vec<PromptTemplate>, String> {
    let mut items = read_prompt_templates(app)?;
    items.retain(|item| item.id != template.id);
    items.insert(0, template);
    write_collection(app, "prompt-templates.json", &items)?;
    Ok(items)
}

pub fn delete_prompt_template(app: &AppHandle, id: &str) -> Result<Vec<PromptTemplate>, String> {
    let mut items = read_prompt_templates(app)?;
    items.retain(|item| item.id != id);
    write_collection(app, "prompt-templates.json", &items)?;
    Ok(items)
}
