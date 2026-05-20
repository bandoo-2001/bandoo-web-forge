use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebAppWindowConfig {
    pub width: f64,
    pub height: f64,
    pub maximized: Option<bool>,
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
    #[serde(default)]
    pub required_permissions: Vec<String>,
    pub created_at: u64,
    pub updated_at: Option<u64>,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationRunResult {
    pub automation_id: String,
    pub web_app_id: String,
    pub dispatched: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeInfo {
    pub os: &'static str,
    pub family: &'static str,
    pub arch: &'static str,
    pub linux_primary: bool,
}
