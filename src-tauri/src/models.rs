use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebAppWindowConfig {
    pub width: f64,
    pub height: f64,
    pub maximized: Option<bool>,
    #[serde(default)]
    pub transparent: bool,
    #[serde(default)]
    pub decorations: bool,
    #[serde(default = "default_true")]
    pub stable_fallback: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebAppWindowState {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebAppPermissions {
    #[serde(default = "default_true")]
    pub page: bool,
    #[serde(default)]
    pub clipboard: bool,
    #[serde(default)]
    pub shell: bool,
    #[serde(default)]
    pub filesystem: bool,
    #[serde(default)]
    pub network: bool,
    #[serde(default)]
    pub notification: bool,
}

fn default_true() -> bool {
    true
}

fn default_titlebar_height() -> f64 {
    44.0
}

fn default_background_color() -> String {
    "#111827".to_string()
}

fn default_foreground_color() -> String {
    "#f8fafc".to_string()
}

fn default_opacity() -> f64 {
    1.0
}

fn default_corner_radius() -> f64 {
    12.0
}

fn default_controls_position() -> String {
    "right".to_string()
}

fn default_controls_style() -> String {
    "windows".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebAppChromeConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_titlebar_height")]
    pub titlebar_height: f64,
    #[serde(default = "default_background_color")]
    pub background_color: String,
    #[serde(default = "default_foreground_color")]
    pub foreground_color: String,
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    #[serde(default = "default_corner_radius")]
    pub corner_radius: f64,
    #[serde(default = "default_true")]
    pub shadow: bool,
    #[serde(default = "default_controls_position")]
    pub controls_position: String,
    #[serde(default = "default_controls_style")]
    pub controls_style: String,
    #[serde(default = "default_true")]
    pub show_title: bool,
    #[serde(default = "default_true")]
    pub show_icon: bool,
    #[serde(default)]
    pub show_url: bool,
    #[serde(default)]
    pub theme_preset_id: Option<String>,
}

impl Default for WebAppChromeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            titlebar_height: default_titlebar_height(),
            background_color: default_background_color(),
            foreground_color: default_foreground_color(),
            opacity: default_opacity(),
            corner_radius: default_corner_radius(),
            shadow: true,
            controls_position: default_controls_position(),
            controls_style: default_controls_style(),
            show_title: true,
            show_icon: true,
            show_url: false,
            theme_preset_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebAppScriptConfig {
    #[serde(default)]
    pub inject_bridge: bool,
    #[serde(default)]
    pub custom_script_enabled: bool,
    #[serde(default)]
    pub custom_script: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebApp {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub url: String,
    pub user_agent: Option<String>,
    pub start_on_boot: Option<bool>,
    pub tray: Option<bool>,
    pub window_config: WebAppWindowConfig,
    pub last_window_state: Option<WebAppWindowState>,
    pub permissions: WebAppPermissions,
    #[serde(default)]
    pub script_config: Option<WebAppScriptConfig>,
    #[serde(default)]
    pub chrome_config: Option<WebAppChromeConfig>,
    pub created_at: u64,
    pub updated_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationTrigger {
    pub kind: String,
    pub shortcut: Option<String>,
    pub url: Option<String>,
    pub menu_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationCondition {
    pub kind: String,
    pub value: Option<String>,
    #[serde(default)]
    pub negate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationAction {
    pub kind: String,
    pub selector: Option<String>,
    pub text: Option<String>,
    pub script: Option<String>,
    pub value: Option<String>,
    pub timeout_ms: Option<u64>,
    pub continue_on_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationConfig {
    pub id: String,
    pub web_app_id: String,
    pub name: String,
    #[serde(default)]
    pub enabled: bool,
    pub trigger: AutomationTrigger,
    #[serde(default)]
    pub conditions: Vec<AutomationCondition>,
    #[serde(default)]
    pub actions: Vec<AutomationAction>,
    pub created_at: u64,
    pub updated_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserScriptConfig {
    pub id: String,
    pub web_app_id: String,
    pub name: String,
    #[serde(default)]
    pub enabled: bool,
    pub code: String,
    #[serde(default = "default_script_language")]
    pub language: String,
    #[serde(default)]
    pub compiled_code: Option<String>,
    #[serde(default = "default_script_run_at")]
    pub run_at: String,
    #[serde(default)]
    pub match_patterns: Vec<String>,
    #[serde(default)]
    pub required_permissions: Vec<String>,
    pub created_at: u64,
    pub updated_at: Option<u64>,
}

fn default_script_language() -> String {
    "javascript".to_string()
}

fn default_script_run_at() -> String {
    "manual".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub category: String,
    pub instruction: String,
    pub created_at: u64,
    pub updated_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemePreset {
    pub id: String,
    pub name: String,
    pub chrome_config: WebAppChromeConfig,
    pub created_at: u64,
    pub updated_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub default_theme_preset_id: Option<String>,
    pub default_chrome_config: WebAppChromeConfig,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_theme_preset_id: Some("bandoo-default".to_string()),
            default_chrome_config: WebAppChromeConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopIntegrationResult {
    pub path: String,
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopIntegrationStatus {
    pub target: String,
    pub path: String,
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationStepResult {
    pub index: usize,
    pub action_kind: String,
    pub status: String,
    pub message: String,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationRunResult {
    pub run_id: String,
    pub automation_id: String,
    pub web_app_id: String,
    pub dispatched: bool,
    pub message: String,
    pub steps: Vec<AutomationStepResult>,
    pub started_at: u64,
    pub finished_at: Option<u64>,
    pub duration_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserScriptRunResult {
    pub run_id: String,
    pub script_id: String,
    pub web_app_id: String,
    pub dispatched: bool,
    pub message: String,
    pub started_at: u64,
    pub finished_at: Option<u64>,
    pub duration_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationRunLog {
    pub id: String,
    pub source_id: String,
    pub web_app_id: String,
    pub kind: String,
    pub status: String,
    pub message: String,
    #[serde(default)]
    pub steps: Vec<AutomationStepResult>,
    pub started_at: u64,
    pub finished_at: Option<u64>,
    pub duration_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeInfo {
    pub os: &'static str,
    pub family: &'static str,
    pub arch: &'static str,
    pub linux_primary: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeRequest {
    pub web_app_id: String,
    pub capability: String,
    pub operation: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeResponse {
    pub ok: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}
