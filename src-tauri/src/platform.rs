use std::{env, fs, path::PathBuf, process::Command};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use tauri::AppHandle;

use crate::{
    models::{DesktopIntegrationResult, DesktopIntegrationStatus, RuntimeInfo, WebApp},
    storage,
};

pub fn runtime_info() -> RuntimeInfo {
    RuntimeInfo {
        os: std::env::consts::OS,
        family: std::env::consts::FAMILY,
        arch: std::env::consts::ARCH,
        linux_primary: cfg!(target_os = "linux"),
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DesktopIntegrationTarget {
    Applications,
    Desktop,
    Autostart,
}

impl DesktopIntegrationTarget {
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw {
            "applications" => Ok(Self::Applications),
            "desktop" => Ok(Self::Desktop),
            "autostart" => Ok(Self::Autostart),
            value => Err(format!("Unsupported desktop integration target: {value}")),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Applications => "applications",
            Self::Desktop => "desktop",
            Self::Autostart => "autostart",
        }
    }

    fn all() -> [Self; 3] {
        [Self::Applications, Self::Desktop, Self::Autostart]
    }
}

pub fn install_desktop_entry(
    app: &AppHandle,
    webapp: &WebApp,
    target: DesktopIntegrationTarget,
) -> Result<DesktopIntegrationResult, String> {
    let path = desktop_entry_path(target, &webapp.id)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    if cfg!(target_os = "windows") {
        create_windows_shortcut(app, webapp, &path)?;
    } else if cfg!(target_os = "linux") {
        let entry = desktop_entry(app, webapp, target)?;
        fs::write(&path, entry).map_err(|error| error.to_string())?;
        mark_executable(&path)?;
    } else {
        return Err(
            "Desktop integration is currently implemented for Linux and Windows".to_string(),
        );
    }

    Ok(DesktopIntegrationResult {
        path: path.display().to_string(),
        installed: true,
    })
}

pub fn remove_desktop_entry(
    webapp_id: &str,
    target: DesktopIntegrationTarget,
) -> Result<DesktopIntegrationResult, String> {
    let path = desktop_entry_path(target, webapp_id)?;
    if path.exists() {
        fs::remove_file(&path).map_err(|error| error.to_string())?;
    }

    Ok(DesktopIntegrationResult {
        path: path.display().to_string(),
        installed: false,
    })
}

pub fn remove_all_desktop_entries(webapp_id: &str) -> Result<(), String> {
    for target in DesktopIntegrationTarget::all() {
        let _ = remove_desktop_entry(webapp_id, target);
    }
    Ok(())
}

pub fn desktop_integration_statuses(
    webapp_id: &str,
) -> Result<Vec<DesktopIntegrationStatus>, String> {
    if !cfg!(any(target_os = "linux", target_os = "windows")) {
        return Ok(Vec::new());
    }

    DesktopIntegrationTarget::all()
        .into_iter()
        .map(|target| {
            let path = desktop_entry_path(target, webapp_id)?;
            Ok(DesktopIntegrationStatus {
                target: target.as_str().to_string(),
                installed: path.exists(),
                path: path.display().to_string(),
            })
        })
        .collect()
}

fn desktop_entry_path(
    target: DesktopIntegrationTarget,
    webapp_id: &str,
) -> Result<PathBuf, String> {
    let file_name = format!("bandoo-webforge-{webapp_id}.desktop");
    if cfg!(target_os = "windows") {
        let file_name = format!("Bandoo WebForge {webapp_id}.lnk");
        return match target {
            DesktopIntegrationTarget::Applications => Ok(appdata_dir()?
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs")
                .join(file_name)),
            DesktopIntegrationTarget::Autostart => Ok(appdata_dir()?
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs")
                .join("Startup")
                .join(file_name)),
            DesktopIntegrationTarget::Desktop => Ok(home_dir()?.join("Desktop").join(file_name)),
        };
    }

    match target {
        DesktopIntegrationTarget::Applications => Ok(home_dir()?
            .join(".local/share/applications")
            .join(file_name)),
        DesktopIntegrationTarget::Autostart => {
            Ok(home_dir()?.join(".config/autostart").join(file_name))
        }
        DesktopIntegrationTarget::Desktop => Ok(desktop_dir()?.join(file_name)),
    }
}

fn appdata_dir() -> Result<PathBuf, String> {
    env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "APPDATA is not set".to_string())
}

fn desktop_entry(
    app: &AppHandle,
    webapp: &WebApp,
    target: DesktopIntegrationTarget,
) -> Result<String, String> {
    let exe = env::current_exe().map_err(|error| error.to_string())?;
    let icon = webapp
        .icon
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| "bandoo-web-forge".to_string());
    let autostart_enabled = matches!(target, DesktopIntegrationTarget::Autostart);
    let config_dir = storage::store_dir(app)?;

    Ok(format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name={}\n\
         Comment=Managed by Bandoo WebForge\n\
         Exec={} --launch-webapp {}\n\
         Icon={}\n\
         Terminal=false\n\
         Categories=Network;Utility;\n\
         StartupNotify=true\n\
         X-Bandoo-WebForge-Id={}\n\
         X-Bandoo-WebForge-ConfigDir={}\n\
         X-GNOME-Autostart-enabled={}\n",
        escape_desktop_value(&webapp.name),
        quote_exec_arg(&exe.display().to_string()),
        quote_exec_arg(&webapp.id),
        escape_desktop_value(&icon),
        escape_desktop_value(&webapp.id),
        escape_desktop_value(&config_dir.display().to_string()),
        if autostart_enabled { "true" } else { "false" }
    ))
}

fn home_dir() -> Result<PathBuf, String> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".to_string())
}

fn desktop_dir() -> Result<PathBuf, String> {
    if let Some(raw) = env::var_os("XDG_DESKTOP_DIR") {
        return Ok(PathBuf::from(raw));
    }

    let home = home_dir()?;
    let localized = home.join("桌面");
    if localized.exists() {
        return Ok(localized);
    }
    Ok(home.join("Desktop"))
}

fn escape_desktop_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\n', " ")
}

fn quote_exec_arg(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn create_windows_shortcut(
    _app: &AppHandle,
    webapp: &WebApp,
    path: &PathBuf,
) -> Result<(), String> {
    let exe = env::current_exe().map_err(|error| error.to_string())?;
    let script = format!(
        "$shell = New-Object -ComObject WScript.Shell; \
         $shortcut = $shell.CreateShortcut({}); \
         $shortcut.TargetPath = {}; \
         $shortcut.Arguments = {}; \
         $shortcut.WorkingDirectory = {}; \
         $shortcut.Description = {}; \
         $shortcut.Save()",
        ps_string(&path.display().to_string()),
        ps_string(&exe.display().to_string()),
        ps_string(&format!("--launch-webapp {}", webapp.id)),
        ps_string(
            &exe.parent()
                .map(|path| path.display().to_string())
                .unwrap_or_default()
        ),
        ps_string(&format!("Managed by Bandoo WebForge: {}", webapp.name))
    );
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .output()
        .map_err(|error| error.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn ps_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn mark_executable(_path: &PathBuf) -> Result<(), String> {
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(_path)
            .map_err(|error| error.to_string())?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(_path, permissions).map_err(|error| error.to_string())?;
    }

    Ok(())
}
