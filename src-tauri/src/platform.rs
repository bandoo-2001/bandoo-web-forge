use std::{env, fs, path::PathBuf};

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
    if !cfg!(target_os = "linux") {
        return Err("Desktop integration is currently implemented for Linux only".to_string());
    }

    let path = desktop_entry_path(target, &webapp.id)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let entry = desktop_entry(app, webapp, target)?;
    fs::write(&path, entry).map_err(|error| error.to_string())?;
    mark_executable(&path)?;

    Ok(DesktopIntegrationResult {
        path: path.display().to_string(),
        installed: true,
    })
}

pub fn remove_desktop_entry(
    webapp_id: &str,
    target: DesktopIntegrationTarget,
) -> Result<DesktopIntegrationResult, String> {
    if !cfg!(target_os = "linux") {
        return Err("Desktop integration is currently implemented for Linux only".to_string());
    }

    let path = desktop_entry_path(target, webapp_id)?;
    if path.exists() {
        fs::remove_file(&path).map_err(|error| error.to_string())?;
    }

    Ok(DesktopIntegrationResult {
        path: path.display().to_string(),
        installed: false,
    })
}

pub fn desktop_integration_statuses(
    webapp_id: &str,
) -> Result<Vec<DesktopIntegrationStatus>, String> {
    if !cfg!(target_os = "linux") {
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

fn mark_executable(path: &PathBuf) -> Result<(), String> {
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(path)
            .map_err(|error| error.to_string())?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).map_err(|error| error.to_string())?;
    }

    Ok(())
}
