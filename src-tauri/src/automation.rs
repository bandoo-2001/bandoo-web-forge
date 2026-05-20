use std::collections::{HashMap, HashSet};

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::{
    models::{AutomationConfig, AutomationRunResult},
    runtime, storage,
};

fn shortcut_label(raw: &str) -> String {
    raw.split('+')
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("+")
}

fn shortcut_for(automation: &AutomationConfig) -> Option<String> {
    if !automation.enabled || automation.trigger.kind != "shortcut" {
        return None;
    }

    automation
        .trigger
        .shortcut
        .as_deref()
        .map(shortcut_label)
        .filter(|value| !value.is_empty())
}

fn validate_shortcut(value: &str) -> Result<(), String> {
    Shortcut::try_from(value).map_err(|error| format!("Invalid shortcut `{value}`: {error}"))?;
    Ok(())
}

pub fn validate_shortcuts(automations: &[AutomationConfig]) -> Result<(), String> {
    let mut seen = HashSet::new();

    for automation in automations {
        if let Some(shortcut) = shortcut_for(automation) {
            validate_shortcut(&shortcut)?;
            if !seen.insert(shortcut.clone()) {
                return Err(format!(
                    "Shortcut `{shortcut}` is already used by another automation"
                ));
            }
        }
    }

    Ok(())
}

pub fn refresh_shortcuts(app: &AppHandle) -> Result<(), String> {
    let automations = storage::read_automations(app)?;
    validate_shortcuts(&automations)?;

    let shortcuts = automations
        .iter()
        .filter_map(shortcut_for)
        .collect::<Vec<_>>();

    app.global_shortcut()
        .unregister_all()
        .map_err(|error| error.to_string())?;

    for shortcut in shortcuts {
        app.global_shortcut()
            .register(shortcut.as_str())
            .map_err(|error| format!("Failed to register `{shortcut}`: {error}"))?;
    }

    Ok(())
}

pub fn init_global_shortcuts(app: &AppHandle) -> Result<(), String> {
    app.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(|app, shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    if let Err(error) = execute_shortcut(app, shortcut) {
                        eprintln!("[Bandoo automation shortcut] {error}");
                    }
                }
            })
            .build(),
    )
    .map_err(|error| error.to_string())?;

    refresh_shortcuts(app)?;

    Ok(())
}

pub fn execute_shortcut(
    app: &AppHandle,
    shortcut: &Shortcut,
) -> Result<AutomationRunResult, String> {
    let triggered = *shortcut;
    let automations = storage::read_automations(app)?;
    let shortcut_map = automations
        .iter()
        .filter_map(|automation| {
            let shortcut = shortcut_for(automation)?;
            let parsed = Shortcut::try_from(shortcut.as_str()).ok()?;
            Some((parsed, automation.clone()))
        })
        .collect::<HashMap<_, _>>();

    let automation = shortcut_map
        .get(&triggered)
        .ok_or_else(|| format!("No automation is bound to shortcut `{shortcut}`"))?
        .clone();

    runtime::execute_automation(app.clone(), automation)
}
