use tauri::{
    AppHandle, Manager, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};
use url::Url;

use crate::{
    models::{
        AutomationAction, AutomationConfig, AutomationRunResult, AutomationStepResult,
        UserScriptConfig, UserScriptRunResult, WebApp, WebAppPermissions, WebAppWindowState,
    },
    storage,
};

fn window_label(id: &str) -> String {
    format!("webapp-{id}")
}

fn validate_url(raw_url: &str) -> Result<Url, String> {
    let url: Url = raw_url
        .parse()
        .map_err(|error: url::ParseError| error.to_string())?;

    match url.scheme() {
        "http" | "https" => Ok(url),
        scheme => Err(format!("Unsupported URL scheme: {scheme}")),
    }
}

fn capture_window_state<R: Runtime>(
    window: &WebviewWindow<R>,
) -> Result<WebAppWindowState, String> {
    let position = window.outer_position().map_err(|error| error.to_string())?;
    let size = window.inner_size().map_err(|error| error.to_string())?;
    let maximized = window.is_maximized().map_err(|error| error.to_string())?;

    Ok(WebAppWindowState {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
        maximized,
    })
}

fn persist_window_state<R: Runtime>(app: &AppHandle, id: &str, window: &WebviewWindow<R>) {
    if let Ok(state) = capture_window_state(window) {
        let _ = storage::update_window_state(app, id, state);
    }
}

fn attach_window_state_tracking<R: Runtime>(app: AppHandle, id: String, window: &WebviewWindow<R>) {
    let tracked_window = window.clone();
    window.on_window_event(move |event| match event {
        WindowEvent::Moved(_)
        | WindowEvent::Resized(_)
        | WindowEvent::CloseRequested { .. }
        | WindowEvent::Destroyed => persist_window_state(&app, &id, &tracked_window),
        _ => {}
    });
}

fn configured_builder<'a>(
    app: &'a AppHandle,
    label: String,
    webapp: &WebApp,
) -> Result<WebviewWindowBuilder<'a, tauri::Wry, AppHandle>, String> {
    let url = validate_url(&webapp.url)?;
    let mut builder = WebviewWindowBuilder::new(app, label, WebviewUrl::External(url))
        .title(&webapp.name)
        .inner_size(webapp.window_config.width, webapp.window_config.height)
        .min_inner_size(480.0, 360.0)
        .initialization_script(bridge_script(webapp)?);

    if let Some(state) = &webapp.last_window_state {
        builder = builder
            .position(f64::from(state.x), f64::from(state.y))
            .inner_size(f64::from(state.width), f64::from(state.height))
            .maximized(state.maximized);
    } else if webapp.window_config.maximized.unwrap_or(false) {
        builder = builder.maximized(true);
    } else {
        builder = builder.center();
    }

    if let Some(user_agent) = webapp
        .user_agent
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder = builder.user_agent(user_agent);
    }

    Ok(builder)
}

fn bridge_script(webapp: &WebApp) -> Result<String, String> {
    let app_json = serde_json::to_string(&serde_json::json!({
        "id": webapp.id,
        "name": webapp.name,
        "url": webapp.url,
    }))
    .map_err(|error| error.to_string())?;
    let permissions_json =
        serde_json::to_string(&webapp.permissions).map_err(|error| error.to_string())?;
    let user_script = webapp
        .script_config
        .as_ref()
        .filter(|config| config.inject_bridge && config.custom_script_enabled)
        .map(|config| config.custom_script.as_str())
        .unwrap_or_default();

    Ok(format!(
        r#"
(() => {{
  const app = {app_json};
  const permissions = {permissions_json};
  const emitRoute = () => {{
    window.dispatchEvent(new CustomEvent('bandoo:route-change', {{
      detail: {{
        href: location.href,
        pathname: location.pathname,
        search: location.search,
        hash: location.hash
      }}
    }}));
  }};

  const notify = async (title, body) => {{
    if (!permissions.notification || !('Notification' in window)) return false;
    if (Notification.permission === 'default') await Notification.requestPermission();
    if (Notification.permission !== 'granted') return false;
    new Notification(String(title || app.name), {{ body: String(body || '') }});
    return true;
  }};

  Object.defineProperty(window, '__BANDOO__', {{
    configurable: false,
    enumerable: false,
    value: Object.freeze({{
      app,
      permissions: Object.freeze(permissions),
      version: '0.1.0',
      getTitle: () => document.title,
      getRoute: () => ({{
        href: location.href,
        pathname: location.pathname,
        search: location.search,
        hash: location.hash
      }}),
      notify,
      notification: Object.freeze({{
        send: notify
      }}),
      clipboard: Object.freeze({{
        readText: async () => {{
          if (!permissions.clipboard || !navigator.clipboard?.readText) throw new Error('Clipboard read is not permitted');
          return await navigator.clipboard.readText();
        }},
        writeText: async (text) => {{
          if (!permissions.clipboard || !navigator.clipboard?.writeText) throw new Error('Clipboard write is not permitted');
          await navigator.clipboard.writeText(String(text ?? ''));
          return true;
        }}
      }}),
      page: Object.freeze({{
        query: (selector) => {{
          if (!permissions.page) throw new Error('Page access is not permitted');
          return document.querySelector(String(selector));
        }},
        focus: (selector) => {{
          if (!permissions.page) throw new Error('Page access is not permitted');
          const element = document.querySelector(String(selector));
          if (!element) throw new Error(`Element not found: ${{selector}}`);
          element.focus();
          return true;
        }},
        click: (selector) => {{
          if (!permissions.page) throw new Error('Page access is not permitted');
          const element = document.querySelector(String(selector));
          if (!element) throw new Error(`Element not found: ${{selector}}`);
          element.click();
          return true;
        }},
        type: (selector, text) => {{
          if (!permissions.page) throw new Error('Page access is not permitted');
          const element = document.querySelector(String(selector));
          if (!element) throw new Error(`Element not found: ${{selector}}`);
          element.focus();
          const value = String(text ?? '');
          if ('value' in element) {{
            element.value = value;
            element.dispatchEvent(new InputEvent('input', {{ bubbles: true, inputType: 'insertText', data: value }}));
            element.dispatchEvent(new Event('change', {{ bubbles: true }}));
          }} else {{
            element.textContent = value;
            element.dispatchEvent(new InputEvent('input', {{ bubbles: true, inputType: 'insertText', data: value }}));
          }}
          return true;
        }}
      }}),
      workflow: Object.freeze({{
        runActions: async (actions) => window.__BANDOO__.automation.run(actions),
        sleep: (milliseconds) => new Promise((resolve) => setTimeout(resolve, Number(milliseconds || 0))),
        log: (...args) => console.log('[Bandoo workflow]', ...args)
      }}),
      automation: Object.freeze({{
        run: async (actions) => {{
          let clipboard = '';
          const results = [];
          for (const [index, action] of actions.entries()) {{
            try {{
              switch (action.kind) {{
                case 'clipboard-read':
                  clipboard = await window.__BANDOO__.clipboard.readText();
                  if (!clipboard) throw new Error('Clipboard is empty');
                  break;
                case 'clipboard-write':
                  await window.__BANDOO__.clipboard.writeText(action.text ?? action.value ?? '');
                  break;
                case 'page-focus':
                  window.__BANDOO__.page.focus(action.selector);
                  break;
                case 'page-click':
                  window.__BANDOO__.page.click(action.selector);
                  break;
                case 'page-type':
                  window.__BANDOO__.page.type(action.selector || ':focus', String(action.text ?? '').replaceAll('{{clipboard}}', clipboard));
                  break;
                case 'notify':
                  await window.__BANDOO__.notify(action.text || 'Bandoo WebForge', action.value || '');
                  break;
                case 'js':
                  Function(action.script || '')();
                  break;
                default:
                  throw new Error(`Unsupported action kind: ${{action.kind}}`);
              }}
              results.push({{
                index: index + 1,
                actionKind: action.kind,
                status: 'completed',
                message: 'Completed'
              }});
            }} catch (error) {{
              const message = `Step ${{index + 1}} failed (${{action.kind}}): ${{error.message || error}}`;
              results.push({{
                index: index + 1,
                actionKind: action.kind,
                status: 'failed',
                message
              }});
              const runError = new Error(message);
              runError.steps = results;
              await notify('Bandoo 自动化失败', message).catch(() => false);
              throw runError;
            }}
          }}
          return {{ ok: true, steps: results }};
        }}
      }}),
      onRouteChange: (handler) => {{
        window.addEventListener('bandoo:route-change', (event) => handler(event.detail));
      }}
    }})
  }});

  const patchHistory = (name) => {{
    const original = history[name];
    history[name] = function (...args) {{
      const result = original.apply(this, args);
      queueMicrotask(emitRoute);
      return result;
    }};
  }};
  patchHistory('pushState');
  patchHistory('replaceState');
  window.addEventListener('popstate', emitRoute);
  window.addEventListener('hashchange', emitRoute);
  queueMicrotask(emitRoute);

  {user_script}
}})();
"#
    ))
}

pub fn launch_cli_webapp(app: AppHandle) -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--launch-webapp" {
            if let Some(id) = args.next() {
                return launch_webapp(app, id);
            }
            return Err("--launch-webapp requires a WebApp id".to_string());
        }
    }
    Ok(())
}

pub fn show_main_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|error| error.to_string())?;
        window.set_focus().map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn launch_webapp(app: AppHandle, id: String) -> Result<(), String> {
    let items = storage::read_webapps(&app)?;
    let webapp = items
        .into_iter()
        .find(|candidate| candidate.id == id)
        .ok_or_else(|| "WebApp not found".to_string())?;

    let label = window_label(&webapp.id);
    if let Some(window) = app.get_webview_window(&label) {
        window.set_focus().map_err(|error| error.to_string())?;
        return Ok(());
    }

    let window = configured_builder(&app, label, &webapp)?
        .build()
        .map_err(|error| error.to_string())?;
    attach_window_state_tracking(app, webapp.id, &window);

    Ok(())
}

pub fn execute_automation(
    app: AppHandle,
    automation: AutomationConfig,
) -> Result<AutomationRunResult, String> {
    if !automation.enabled {
        return Ok(AutomationRunResult {
            automation_id: automation.id,
            web_app_id: automation.web_app_id,
            dispatched: false,
            message: "Automation is disabled".to_string(),
            steps: Vec::new(),
        });
    }

    let webapp_id = automation.web_app_id.clone();
    if webapp_id.trim().is_empty() {
        return Err("Automation must be bound to a WebApp before execution".to_string());
    }

    let webapp = storage::read_webapps(&app)?
        .into_iter()
        .find(|candidate| candidate.id == webapp_id)
        .ok_or_else(|| "Bound WebApp was not found".to_string())?;

    if !conditions_match(&automation, &webapp) {
        return Ok(AutomationRunResult {
            automation_id: automation.id,
            web_app_id: webapp_id,
            dispatched: false,
            message: "Automation conditions were not met".to_string(),
            steps: Vec::new(),
        });
    }

    let actions = automation.actions.clone();
    let preflight_steps = preflight_actions(&actions, &webapp.permissions);
    if let Some(failed) = preflight_steps
        .iter()
        .find(|step| step.status == "failed")
        .cloned()
    {
        return Ok(AutomationRunResult {
            automation_id: automation.id,
            web_app_id: webapp_id,
            dispatched: false,
            message: failed.message,
            steps: preflight_steps,
        });
    }

    launch_webapp(app.clone(), webapp_id.clone())?;
    let label = window_label(&webapp_id);
    let window = app
        .get_webview_window(&label)
        .ok_or_else(|| "WebApp window was not found after launch".to_string())?;

    let actions_json = serde_json::to_string(&actions).map_err(|error| error.to_string())?;
    let script = format!(
        r#"
(async () => {{
  if (!window.__BANDOO__?.automation?.run) throw new Error('Bandoo automation bridge is not available');
  const result = await window.__BANDOO__.automation.run({actions_json});
  console.info('[Bandoo automation]', result);
}})().catch((error) => {{
  console.error('[Bandoo automation]', error);
  if (error.steps) console.table(error.steps);
}});
"#
    );
    window.eval(script).map_err(|error| error.to_string())?;

    Ok(AutomationRunResult {
        automation_id: automation.id,
        web_app_id: webapp_id,
        dispatched: true,
        message: "Automation was dispatched to the WebApp window".to_string(),
        steps: preflight_steps
            .into_iter()
            .map(|step| AutomationStepResult {
                status: "dispatched".to_string(),
                ..step
            })
            .collect(),
    })
}

pub fn execute_user_script(
    app: AppHandle,
    script: UserScriptConfig,
) -> Result<UserScriptRunResult, String> {
    if !script.enabled {
        return Ok(UserScriptRunResult {
            script_id: script.id,
            web_app_id: script.web_app_id,
            dispatched: false,
            message: "User script is disabled".to_string(),
        });
    }

    let webapp_id = script.web_app_id.clone();
    if webapp_id.trim().is_empty() {
        return Err("User script must be bound to a WebApp before execution".to_string());
    }

    let webapp = storage::read_webapps(&app)?
        .into_iter()
        .find(|candidate| candidate.id == webapp_id)
        .ok_or_else(|| "Bound WebApp was not found".to_string())?;

    if let Some(message) =
        missing_permission_message(&script.required_permissions, &webapp.permissions)
    {
        return Ok(UserScriptRunResult {
            script_id: script.id,
            web_app_id: webapp_id,
            dispatched: false,
            message,
        });
    }

    launch_webapp(app.clone(), webapp_id.clone())?;
    let label = window_label(&webapp_id);
    let window = app
        .get_webview_window(&label)
        .ok_or_else(|| "WebApp window was not found after launch".to_string())?;

    let script_id_json = serde_json::to_string(&script.id).map_err(|error| error.to_string())?;
    let script_name_json =
        serde_json::to_string(&script.name).map_err(|error| error.to_string())?;
    let code_json = serde_json::to_string(&script.code).map_err(|error| error.to_string())?;
    let eval_script = format!(
        r#"
(async () => {{
  const scriptId = {script_id_json};
  const scriptName = {script_name_json};
  const source = {code_json};
  const bandoo = window.__BANDOO__;
  if (!bandoo) throw new Error('Bandoo bridge is not available');

  try {{
    const runner = new Function(
      'bandoo',
      'app',
      'page',
      'clipboard',
      'notification',
      'workflow',
      `"use strict"; return (async () => {{\n${{source}}\n}})();`
    );
    await runner(
      bandoo,
      bandoo.app,
      bandoo.page,
      bandoo.clipboard,
      bandoo.notification,
      bandoo.workflow
    );
    console.info('[Bandoo user script]', {{ scriptId, scriptName, ok: true }});
  }} catch (error) {{
    const message = error?.message || String(error);
    console.error('[Bandoo user script]', {{ scriptId, scriptName, message, error }});
    await bandoo.notification?.send?.('Bandoo 用户脚本失败', message).catch(() => false);
  }}
}})().catch((error) => {{
  console.error('[Bandoo user script]', error);
}});
"#
    );
    window
        .eval(eval_script)
        .map_err(|error| error.to_string())?;

    Ok(UserScriptRunResult {
        script_id: script.id,
        web_app_id: webapp_id,
        dispatched: true,
        message: "User script was dispatched to the WebApp window".to_string(),
    })
}

fn preflight_actions(
    actions: &[AutomationAction],
    permissions: &WebAppPermissions,
) -> Vec<AutomationStepResult> {
    actions
        .iter()
        .enumerate()
        .map(|(index, action)| preflight_action(index + 1, action, permissions))
        .collect()
}

fn conditions_match(automation: &AutomationConfig, webapp: &WebApp) -> bool {
    automation.conditions.iter().all(|condition| {
        let matched = match condition.kind.as_str() {
            "url-contains" => condition
                .value
                .as_deref()
                .map(|value| webapp.url.contains(value))
                .unwrap_or(true),
            "current-webapp" => condition
                .value
                .as_deref()
                .map(|value| value == webapp.id)
                .unwrap_or(true),
            "platform" => condition
                .value
                .as_deref()
                .map(|value| value == std::env::consts::OS)
                .unwrap_or(true),
            _ => true,
        };

        if condition.negate {
            !matched
        } else {
            matched
        }
    })
}

fn preflight_action(
    index: usize,
    action: &AutomationAction,
    permissions: &WebAppPermissions,
) -> AutomationStepResult {
    let action_kind = action.kind.clone();

    let failure = match action.kind.as_str() {
        kind if !is_supported_action(kind) => Some("Unsupported action kind"),
        "clipboard-read" | "clipboard-write" if !permissions.clipboard => {
            Some("Clipboard permission is disabled")
        }
        "page-focus" | "page-click" | "page-type" | "js" if !permissions.page => {
            Some("Page permission is disabled")
        }
        "notify" if !permissions.notification => Some("Notification permission is disabled"),
        "page-focus" | "page-click"
            if action.selector.as_deref().unwrap_or("").trim().is_empty() =>
        {
            Some("Page selector is required")
        }
        "page-type"
            if action
                .selector
                .as_deref()
                .unwrap_or(":focus")
                .trim()
                .is_empty() =>
        {
            Some("Page selector is required")
        }
        _ => None,
    };

    if let Some(message) = failure {
        return AutomationStepResult {
            index,
            action_kind,
            status: "failed".to_string(),
            message: format!("Step {index} failed ({}): {message}", action.kind),
        };
    }

    AutomationStepResult {
        index,
        action_kind,
        status: "ready".to_string(),
        message: "Ready".to_string(),
    }
}

fn is_supported_action(kind: &str) -> bool {
    matches!(
        kind,
        "clipboard-read"
            | "clipboard-write"
            | "page-focus"
            | "page-click"
            | "page-type"
            | "notify"
            | "js"
    )
}

fn missing_permission_message(
    required_permissions: &[String],
    permissions: &WebAppPermissions,
) -> Option<String> {
    required_permissions.iter().find_map(|permission| {
        let allowed = match permission.as_str() {
            "page" => permissions.page,
            "clipboard" => permissions.clipboard,
            "notification" => permissions.notification,
            "network" => permissions.network,
            "shell" => false,
            "filesystem" => false,
            unknown => {
                return Some(format!("Unsupported user script permission: {unknown}"));
            }
        };

        if allowed {
            None
        } else {
            Some(format!("Missing WebApp permission: {permission}"))
        }
    })
}
