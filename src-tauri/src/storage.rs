use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::{params, Connection};
use tauri::{AppHandle, Manager};

use crate::models::{
    AppSettings, AutomationConfig, AutomationRunLog, PromptTemplate, ThemePreset, UserScriptConfig,
    WebApp, WebAppWindowState,
};

const DB_FILE: &str = "bandoo-webforge.sqlite3";
const SCHEMA_VERSION: i64 = 1;

trait StoredItem {
    fn id(&self) -> &str;
}

impl StoredItem for WebApp {
    fn id(&self) -> &str {
        &self.id
    }
}

impl StoredItem for AutomationConfig {
    fn id(&self) -> &str {
        &self.id
    }
}

impl StoredItem for UserScriptConfig {
    fn id(&self) -> &str {
        &self.id
    }
}

impl StoredItem for PromptTemplate {
    fn id(&self) -> &str {
        &self.id
    }
}

impl StoredItem for ThemePreset {
    fn id(&self) -> &str {
        &self.id
    }
}

impl StoredItem for AutomationRunLog {
    fn id(&self) -> &str {
        &self.id
    }
}

pub fn store_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir)
}

fn db_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(store_dir(app)?.join(DB_FILE))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn connect(app: &AppHandle) -> Result<Connection, String> {
    let conn = Connection::open(db_path(app)?).map_err(|error| error.to_string())?;
    migrate(&conn)?;
    import_legacy_json(app, &conn)?;
    seed_theme_presets(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS webapps (
            id TEXT PRIMARY KEY NOT NULL,
            payload TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS automations (
            id TEXT PRIMARY KEY NOT NULL,
            payload TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS user_scripts (
            id TEXT PRIMARY KEY NOT NULL,
            payload TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS prompt_templates (
            id TEXT PRIMARY KEY NOT NULL,
            payload TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS theme_presets (
            id TEXT PRIMARY KEY NOT NULL,
            payload TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS automation_run_logs (
            id TEXT PRIMARY KEY NOT NULL,
            payload TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );
        ",
    )
    .map_err(|error| error.to_string())?;

    set_setting(conn, "schemaVersion", &SCHEMA_VERSION.to_string())?;
    Ok(())
}

fn setting(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE key = ?1")
        .map_err(|error| error.to_string())?;
    let mut rows = stmt
        .query(params![key])
        .map_err(|error| error.to_string())?;
    if let Some(row) = rows.next().map_err(|error| error.to_string())? {
        let value = row.get(0).map_err(|error| error.to_string())?;
        Ok(Some(value))
    } else {
        Ok(None)
    }
}

fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn read_legacy_collection<T>(path: &Path) -> Result<Vec<T>, String>
where
    T: serde::de::DeserializeOwned,
{
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&raw).map_err(|error| error.to_string())
}

fn backup_legacy_file(dir: &Path, backup_dir: &Path, file_name: &str) -> Result<(), String> {
    let source = dir.join(file_name);
    if source.exists() {
        fs::create_dir_all(backup_dir).map_err(|error| error.to_string())?;
        fs::copy(&source, backup_dir.join(file_name)).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn import_legacy_json(app: &AppHandle, conn: &Connection) -> Result<(), String> {
    if setting(conn, "legacyJsonImported")?.as_deref() == Some("true") {
        return Ok(());
    }

    let dir = store_dir(app)?;
    let backup_dir = dir.join("legacy-json-backup").join(now_ms().to_string());

    for item in read_legacy_collection::<WebApp>(&dir.join("webapps.json"))? {
        upsert_item(conn, "webapps", &item)?;
    }
    for item in read_legacy_collection::<AutomationConfig>(&dir.join("automations.json"))? {
        upsert_item(conn, "automations", &item)?;
    }
    for item in read_legacy_collection::<UserScriptConfig>(&dir.join("user-scripts.json"))? {
        upsert_item(conn, "user_scripts", &item)?;
    }
    for item in read_legacy_collection::<PromptTemplate>(&dir.join("prompt-templates.json"))? {
        upsert_item(conn, "prompt_templates", &item)?;
    }

    backup_legacy_file(&dir, &backup_dir, "webapps.json")?;
    backup_legacy_file(&dir, &backup_dir, "automations.json")?;
    backup_legacy_file(&dir, &backup_dir, "user-scripts.json")?;
    backup_legacy_file(&dir, &backup_dir, "prompt-templates.json")?;
    set_setting(conn, "legacyJsonImported", "true")?;
    Ok(())
}

fn seed_theme_presets(conn: &Connection) -> Result<(), String> {
    let existing: i64 = conn
        .query_row("SELECT COUNT(*) FROM theme_presets", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    if existing > 0 {
        return Ok(());
    }

    let now = now_ms();
    let presets = [
        ThemePreset {
            id: "bandoo-default".to_string(),
            name: "Bandoo Dark".to_string(),
            chrome_config: Default::default(),
            created_at: now,
            updated_at: None,
        },
        ThemePreset {
            id: "graphite-light".to_string(),
            name: "Graphite Light".to_string(),
            chrome_config: crate::models::WebAppChromeConfig {
                background_color: "#f8fafc".to_string(),
                foreground_color: "#18202a".to_string(),
                controls_style: "minimal".to_string(),
                shadow: true,
                ..Default::default()
            },
            created_at: now,
            updated_at: None,
        },
        ThemePreset {
            id: "teal-focus".to_string(),
            name: "Teal Focus".to_string(),
            chrome_config: crate::models::WebAppChromeConfig {
                background_color: "#0f766e".to_string(),
                foreground_color: "#f8fafc".to_string(),
                controls_position: "left".to_string(),
                corner_radius: 16.0,
                ..Default::default()
            },
            created_at: now,
            updated_at: None,
        },
    ];

    for preset in presets {
        upsert_item(conn, "theme_presets", &preset)?;
    }

    let settings = AppSettings::default();
    let raw = serde_json::to_string(&settings).map_err(|error| error.to_string())?;
    set_setting(conn, "appSettings", &raw)?;
    Ok(())
}

fn read_collection<T>(app: &AppHandle, table: &str) -> Result<Vec<T>, String>
where
    T: serde::de::DeserializeOwned,
{
    let conn = connect(app)?;
    let sql = format!("SELECT payload FROM {table} ORDER BY updated_at DESC");
    let mut stmt = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| error.to_string())?;
    rows.map(|row| {
        let raw = row.map_err(|error| error.to_string())?;
        serde_json::from_str(&raw).map_err(|error| error.to_string())
    })
    .collect()
}

fn upsert_item<T>(conn: &Connection, table: &str, item: &T) -> Result<(), String>
where
    T: serde::Serialize + StoredItem,
{
    let raw = serde_json::to_string_pretty(item).map_err(|error| error.to_string())?;
    let sql = format!(
        "INSERT INTO {table} (id, payload, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(id) DO UPDATE SET payload = excluded.payload, updated_at = excluded.updated_at"
    );
    conn.execute(&sql, params![item.id(), raw, now_ms()])
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn upsert_collection_item<T>(app: &AppHandle, table: &str, item: T) -> Result<Vec<T>, String>
where
    T: serde::Serialize + serde::de::DeserializeOwned + StoredItem,
{
    let conn = connect(app)?;
    upsert_item(&conn, table, &item)?;
    read_collection(app, table)
}

fn delete_item<T>(app: &AppHandle, table: &str, id: &str) -> Result<Vec<T>, String>
where
    T: serde::de::DeserializeOwned,
{
    let conn = connect(app)?;
    let sql = format!("DELETE FROM {table} WHERE id = ?1");
    conn.execute(&sql, params![id])
        .map_err(|error| error.to_string())?;
    read_collection(app, table)
}

pub fn read_webapps(app: &AppHandle) -> Result<Vec<WebApp>, String> {
    read_collection(app, "webapps")
}

pub fn upsert_webapp(app: &AppHandle, webapp: WebApp) -> Result<Vec<WebApp>, String> {
    upsert_collection_item(app, "webapps", webapp)
}

pub fn delete_webapp(app: &AppHandle, id: &str) -> Result<Vec<WebApp>, String> {
    delete_item(app, "webapps", id)
}

pub fn update_window_state(
    app: &AppHandle,
    id: &str,
    state: WebAppWindowState,
) -> Result<(), String> {
    let mut items = read_webapps(app)?;
    if let Some(item) = items.iter_mut().find(|candidate| candidate.id == id) {
        item.last_window_state = Some(state);
        upsert_webapp(app, item.clone())?;
    }
    Ok(())
}

pub fn read_automations(app: &AppHandle) -> Result<Vec<AutomationConfig>, String> {
    read_collection(app, "automations")
}

pub fn write_automations(app: &AppHandle, items: &[AutomationConfig]) -> Result<(), String> {
    let conn = connect(app)?;
    conn.execute("DELETE FROM automations", [])
        .map_err(|error| error.to_string())?;
    for item in items {
        upsert_item(&conn, "automations", item)?;
    }
    Ok(())
}

pub fn delete_automation(app: &AppHandle, id: &str) -> Result<Vec<AutomationConfig>, String> {
    delete_item(app, "automations", id)
}

pub fn read_user_scripts(app: &AppHandle) -> Result<Vec<UserScriptConfig>, String> {
    read_collection(app, "user_scripts")
}

pub fn upsert_user_script(
    app: &AppHandle,
    script: UserScriptConfig,
) -> Result<Vec<UserScriptConfig>, String> {
    upsert_collection_item(app, "user_scripts", script)
}

pub fn delete_user_script(app: &AppHandle, id: &str) -> Result<Vec<UserScriptConfig>, String> {
    delete_item(app, "user_scripts", id)
}

pub fn read_prompt_templates(app: &AppHandle) -> Result<Vec<PromptTemplate>, String> {
    read_collection(app, "prompt_templates")
}

pub fn upsert_prompt_template(
    app: &AppHandle,
    template: PromptTemplate,
) -> Result<Vec<PromptTemplate>, String> {
    upsert_collection_item(app, "prompt_templates", template)
}

pub fn delete_prompt_template(app: &AppHandle, id: &str) -> Result<Vec<PromptTemplate>, String> {
    delete_item(app, "prompt_templates", id)
}

pub fn read_theme_presets(app: &AppHandle) -> Result<Vec<ThemePreset>, String> {
    read_collection(app, "theme_presets")
}

pub fn upsert_theme_preset(
    app: &AppHandle,
    preset: ThemePreset,
) -> Result<Vec<ThemePreset>, String> {
    upsert_collection_item(app, "theme_presets", preset)
}

pub fn delete_theme_preset(app: &AppHandle, id: &str) -> Result<Vec<ThemePreset>, String> {
    delete_item(app, "theme_presets", id)
}

pub fn read_app_settings(app: &AppHandle) -> Result<AppSettings, String> {
    let conn = connect(app)?;
    let Some(raw) = setting(&conn, "appSettings")? else {
        return Ok(AppSettings::default());
    };
    serde_json::from_str(&raw).map_err(|error| error.to_string())
}

pub fn write_app_settings(app: &AppHandle, settings: &AppSettings) -> Result<AppSettings, String> {
    let conn = connect(app)?;
    let raw = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
    set_setting(&conn, "appSettings", &raw)?;
    Ok(settings.clone())
}

pub fn append_run_log(
    app: &AppHandle,
    log: AutomationRunLog,
) -> Result<Vec<AutomationRunLog>, String> {
    upsert_collection_item(app, "automation_run_logs", log)
}

pub fn read_run_logs(app: &AppHandle) -> Result<Vec<AutomationRunLog>, String> {
    read_collection(app, "automation_run_logs")
}
