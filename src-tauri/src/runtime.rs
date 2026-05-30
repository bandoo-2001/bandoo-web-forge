use tauri::webview::WebviewBuilder;
use tauri::window::{Window, WindowBuilder};
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Runtime, WebviewUrl, WindowEvent};
use url::Url;

use crate::{
    models::{
        AutomationAction, AutomationConfig, AutomationRunLog, AutomationRunResult,
        AutomationStepResult, UserScriptConfig, UserScriptRunResult, WebApp, WebAppPermissions,
        WebAppWindowState,
    },
    storage,
};

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn window_label(id: &str) -> String {
    format!("webapp-{id}")
}

fn shell_label(id: &str) -> String {
    format!("webapp-{id}-shell")
}

fn content_label(id: &str) -> String {
    format!("webapp-{id}-content")
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

fn capture_window_state<R: Runtime>(window: &Window<R>) -> Result<WebAppWindowState, String> {
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

fn persist_window_state<R: Runtime>(app: &AppHandle, id: &str, window: &Window<R>) {
    if let Ok(state) = capture_window_state(window) {
        let _ = storage::update_window_state(app, id, state);
    }
}

fn attach_window_state_tracking<R: Runtime>(app: AppHandle, id: String, window: &Window<R>) {
    let tracked_window = window.clone();
    window.on_window_event(move |event| match event {
        WindowEvent::Moved(_)
        | WindowEvent::Resized(_)
        | WindowEvent::CloseRequested { .. }
        | WindowEvent::Destroyed => persist_window_state(&app, &id, &tracked_window),
        _ => {}
    });
}

fn configured_shell_window(
    app: &AppHandle,
    label: String,
    webapp: &WebApp,
) -> Result<Window<tauri::Wry>, String> {
    validate_url(&webapp.url)?;

    let chrome = webapp.chrome_config.clone().unwrap_or_default();
    let mut builder = WindowBuilder::new(app, label.clone())
        .title(&webapp.name)
        .inner_size(webapp.window_config.width, webapp.window_config.height)
        .min_inner_size(640.0, 420.0)
        .decorations(webapp.window_config.decorations)
        .shadow(chrome.shadow);

    #[cfg(not(target_os = "macos"))]
    {
        builder = builder.transparent(webapp.window_config.transparent);
    }

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

    let window = builder.build().map_err(|error| error.to_string())?;
    let titlebar_height = if chrome.enabled {
        chrome.titlebar_height.max(32.0)
    } else {
        0.0
    };
    let width = webapp.window_config.width;
    let height = webapp.window_config.height;

    let user_scripts_json = user_scripts_json(app, webapp)?;
    let automations_json = automations_json(app, webapp)?;
    let shell_url = WebviewUrl::App(format!("index.html#/shell/{}", webapp.id).into());
    let mut shell = WebviewBuilder::new(shell_label(&webapp.id), shell_url);
    #[cfg(not(target_os = "macos"))]
    {
        shell = shell.transparent(webapp.window_config.transparent);
    }
    let shell = shell.auto_resize();
    window
        .add_child(
            shell,
            LogicalPosition::new(0.0, 0.0),
            LogicalSize::new(width, titlebar_height),
        )
        .map_err(|error| error.to_string())?;

    let mut content = WebviewBuilder::new(
        content_label(&webapp.id),
        WebviewUrl::External(validate_url(&webapp.url)?),
    )
    .initialization_script(bridge_script_with_payloads(
        webapp,
        &user_scripts_json,
        &automations_json,
    )?)
    .auto_resize();

    if let Some(user_agent) = webapp
        .user_agent
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        content = content.user_agent(user_agent);
    }

    window
        .add_child(
            content,
            LogicalPosition::new(0.0, titlebar_height),
            LogicalSize::new(width, (height - titlebar_height).max(320.0)),
        )
        .map_err(|error| error.to_string())?;

    Ok(window)
}

fn user_scripts_json(app: &AppHandle, webapp: &WebApp) -> Result<String, String> {
    let scripts = storage::read_user_scripts(app)?
        .into_iter()
        .filter(|script| script.enabled && script.web_app_id == webapp.id)
        .map(|script| {
            serde_json::json!({
                "id": script.id,
                "name": script.name,
                "source": script.compiled_code
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or(script.code),
                "runAt": script.run_at,
                "matchPatterns": script.match_patterns,
                "requiredPermissions": script.required_permissions,
            })
        })
        .collect::<Vec<_>>();
    serde_json::to_string(&scripts).map_err(|error| error.to_string())
}

fn automations_json(app: &AppHandle, webapp: &WebApp) -> Result<String, String> {
    let automations = storage::read_automations(app)?
        .into_iter()
        .filter(|automation| automation.enabled && automation.web_app_id == webapp.id)
        .collect::<Vec<_>>();
    serde_json::to_string(&automations).map_err(|error| error.to_string())
}

fn bridge_script_with_payloads(
    webapp: &WebApp,
    user_scripts_json: &str,
    automations_json: &str,
) -> Result<String, String> {
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
  const userScripts = {user_scripts_json};
  const automations = {automations_json};
  const emitRoute = () => {{
    window.dispatchEvent(new CustomEvent('bandoo:route-change', {{
      detail: {{
        href: location.href,
        pathname: location.pathname,
        search: location.search,
        hash: location.hash
      }}
    }}));
    queueMicrotask(() => runUserScripts('url-change'));
    queueMicrotask(() => runAutomations('url-change'));
  }};

  const notify = async (title, body) => {{
    if (!permissions.notification || !('Notification' in window)) return false;
    if (Notification.permission === 'default') await Notification.requestPermission();
    if (Notification.permission !== 'granted') return false;
    new Notification(String(title || app.name), {{ body: String(body || '') }});
    return true;
  }};
  const invoke = window.__TAURI__?.core?.invoke || window.__TAURI_INTERNALS__?.invoke;
  const bridgeRequest = async (capability, operation, payload = {{}}) => {{
    if (!invoke) throw new Error('Bandoo bridge IPC is not available');
    return await invoke('bridge_request', {{
      request: {{
        webAppId: app.id,
        capability,
        operation,
        payload
      }}
    }});
  }};
  const scriptMatches = (script) => {{
    const patterns = script.matchPatterns?.length ? script.matchPatterns : ['*'];
    return patterns.some((pattern) => {{
      if (pattern === '*') return true;
      if (pattern.startsWith('/') && pattern.endsWith('/')) {{
        try {{ return new RegExp(pattern.slice(1, -1)).test(location.href); }} catch (_) {{ return false; }}
      }}
      return location.href.includes(pattern);
    }});
  }};
  const scriptHasPermissions = (script) => {{
    return (script.requiredPermissions || []).every((permission) => Boolean(permissions[permission]));
  }};
  const runUserScripts = async (runAt) => {{
    for (const script of userScripts) {{
      if (script.runAt !== runAt || !scriptMatches(script) || !scriptHasPermissions(script)) continue;
      const startedAt = Date.now();
      const runId = `script-${{script.id}}-${{startedAt}}`;
      try {{
        const runner = new Function(
          'bandoo',
          'app',
          'page',
          'clipboard',
          'notification',
          'workflow',
          'shell',
          'fs',
          'network',
          `"use strict"; return (async () => {{\n${{script.source || ''}}\n}})();`
        );
        await runner(
          window.__BANDOO__,
          window.__BANDOO__.app,
          window.__BANDOO__.page,
          window.__BANDOO__.clipboard,
          window.__BANDOO__.notification,
          window.__BANDOO__.workflow,
          window.__BANDOO__.shell,
          window.__BANDOO__.fs,
          window.__BANDOO__.network
        );
        await workflowLog({{
          id: runId,
          sourceId: script.id,
          kind: 'user-script',
          status: 'completed',
          message: `User script completed: ${{script.name || script.id}}`,
          startedAt,
          finishedAt: Date.now()
        }});
        console.info('[Bandoo user script]', {{ scriptId: script.id, runAt, ok: true }});
      }} catch (error) {{
        const message = error?.message || String(error);
        await workflowLog({{
          id: runId,
          sourceId: script.id,
          kind: 'user-script',
          status: 'failed',
          message,
          startedAt,
          finishedAt: Date.now(),
          error: message
        }});
        console.error('[Bandoo user script]', {{ scriptId: script.id, runAt, error }});
      }}
    }}
  }};
  const conditionMatches = (condition) => {{
    const value = condition.value || '';
    let matched = true;
    switch (condition.kind) {{
      case 'url-contains':
        matched = !value || location.href.includes(value);
        break;
      case 'url-regex':
        try {{ matched = !value || new RegExp(value).test(location.href); }} catch (_) {{ matched = false; }}
        break;
      case 'current-webapp':
        matched = !value || value === app.id;
        break;
      case 'element-exists':
        matched = !value || Boolean(document.querySelector(value));
        break;
      case 'permission-enabled':
        matched = !value || Boolean(permissions[value]);
        break;
      default:
        matched = true;
    }}
    return condition.negate ? !matched : matched;
  }};
  const runAutomations = async (triggerKind) => {{
    for (const automation of automations) {{
      if (automation.trigger?.kind !== triggerKind) continue;
      if (!(automation.conditions || []).every(conditionMatches)) continue;
      const startedAt = Date.now();
      const runId = `automation-${{automation.id}}-${{startedAt}}`;
      try {{
        const result = await window.__BANDOO__.automation.run(automation.actions || []);
        await window.__BANDOO__.workflow.log({{
          id: runId,
          sourceId: automation.id,
          kind: 'automation',
          status: 'completed',
          message: `Automation completed: ${{automation.name || automation.id}}`,
          steps: result.steps || [],
          startedAt,
          finishedAt: Date.now()
        }});
        console.info('[Bandoo automation]', {{ automationId: automation.id, triggerKind, result }});
      }} catch (error) {{
        const message = error?.message || String(error);
        await window.__BANDOO__?.workflow?.log?.({{
          id: runId,
          sourceId: automation.id,
          kind: 'automation',
          status: 'failed',
          message,
          steps: error?.steps || [],
          startedAt,
          finishedAt: Date.now(),
          error: message
        }});
        console.error('[Bandoo automation]', {{ automationId: automation.id, triggerKind, error }});
      }}
    }}
  }};
  const workflowLog = async (...args) => {{
    const payload =
      args.length === 1 && args[0] && typeof args[0] === 'object' && !Array.isArray(args[0])
        ? args[0]
        : {{
            sourceId: 'workflow.log',
            kind: 'workflow',
            status: 'completed',
            message: args.map((item) => {{
              if (typeof item === 'string') return item;
              try {{ return JSON.stringify(item); }} catch (_) {{ return String(item); }}
            }}).join(' ')
          }};
    try {{
      return await bridgeRequest('workflow', 'log', payload);
    }} catch (error) {{
      console.warn('[Bandoo workflow log]', error);
      return null;
    }}
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
      shell: Object.freeze({{
        exec: async (payload) => bridgeRequest('shell', 'exec', payload)
      }}),
      fs: Object.freeze({{
        readText: async (path) => bridgeRequest('filesystem', 'readText', {{ path }}),
        writeText: async (path, text) => bridgeRequest('filesystem', 'writeText', {{ path, text }}),
        readDir: async (path) => bridgeRequest('filesystem', 'readDir', {{ path }}),
        exists: async (path) => bridgeRequest('filesystem', 'exists', {{ path }}),
        mkdir: async (path) => bridgeRequest('filesystem', 'mkdir', {{ path }}),
        remove: async (path) => bridgeRequest('filesystem', 'remove', {{ path }})
      }}),
      network: Object.freeze({{
        fetch: async (payload) => bridgeRequest('network', 'fetch', payload)
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
        selectorCaptured: (selector) => bridgeRequest('workflow', 'selectorCaptured', {{ selector }}),
        actionsRecorded: (actions) => bridgeRequest('workflow', 'actionsRecorded', {{ actions }}),
        log: (...args) => {{
          console.log('[Bandoo workflow]', ...args);
          return workflowLog(...args);
        }}
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
                case 'wait':
                case 'sleep':
                  await window.__BANDOO__.workflow.sleep(action.timeoutMs ?? Number(action.value || 250));
                  break;
                case 'js':
                  Function(action.script || '')();
                  break;
                case 'shell':
                  await window.__BANDOO__.shell.exec({{ command: action.value || action.text || '' }});
                  break;
                case 'fs-read':
                  await window.__BANDOO__.fs.readText(action.value || action.text || '');
                  break;
                case 'fs-write':
                  await window.__BANDOO__.fs.writeText(action.value || '', action.text || '');
                  break;
                case 'network-fetch':
                  await window.__BANDOO__.network.fetch({{ url: action.value || action.text || '' }});
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
  queueMicrotask(() => runUserScripts('page-load'));
  queueMicrotask(() => runAutomations('page-load'));
  for (const automation of automations) {{
    if (automation.trigger?.kind === 'timer') {{
      const interval = Number(automation.trigger?.url || automation.trigger?.shortcut || 60000);
      setInterval(async () => {{
        if (!(automation.conditions || []).every(conditionMatches)) return;
        const startedAt = Date.now();
        const runId = `automation-${{automation.id}}-${{startedAt}}`;
        try {{
          const result = await window.__BANDOO__.automation.run(automation.actions || []);
          await window.__BANDOO__.workflow.log({{
            id: runId,
            sourceId: automation.id,
            kind: 'automation',
            status: 'completed',
            message: `Automation completed: ${{automation.name || automation.id}}`,
            steps: result.steps || [],
            startedAt,
            finishedAt: Date.now()
          }});
          console.info('[Bandoo automation]', {{ automationId: automation.id, triggerKind: 'timer', result }});
        }} catch (error) {{
          const message = error?.message || String(error);
          await window.__BANDOO__?.workflow?.log?.({{
            id: runId,
            sourceId: automation.id,
            kind: 'automation',
            status: 'failed',
            message,
            steps: error?.steps || [],
            startedAt,
            finishedAt: Date.now(),
            error: message
          }});
          console.error('[Bandoo automation]', {{ automationId: automation.id, triggerKind: 'timer', error }});
        }}
      }}, Math.max(interval, 1000));
    }}
  }}

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
    if let Some(window) = app.get_window(&label) {
        window.set_focus().map_err(|error| error.to_string())?;
        return Ok(());
    }

    let window = configured_shell_window(&app, label, &webapp)?;
    attach_window_state_tracking(app, webapp.id, &window);

    Ok(())
}

fn selector_capture_script() -> &'static str {
    r#"
(() => {
  if (!window.__BANDOO__) return;
  window.__BANDOO_SELECTOR_CAPTURE_CLEANUP__?.();
  const escapeCss = (value) => window.CSS?.escape
    ? window.CSS.escape(String(value))
    : String(value).replace(/[^a-zA-Z0-9_-]/g, '\\$&');
  const selectorFor = (target) => {
    if (!(target instanceof Element)) return '';
    if (target.id) return `#${escapeCss(target.id)}`;
    const testId = target.getAttribute('data-testid') || target.getAttribute('data-test');
    if (testId) return `[data-testid="${String(testId).replaceAll('"', '\\"')}"]`;
    const aria = target.getAttribute('aria-label');
    if (aria) return `${target.localName.toLowerCase()}[aria-label="${String(aria).replaceAll('"', '\\"')}"]`;
    const path = [];
    let element = target;
    while (element && element.nodeType === 1 && element !== document.documentElement) {
      let part = element.localName.toLowerCase();
      const name = element.getAttribute('name');
      if (name) {
        part += `[name="${String(name).replaceAll('"', '\\"')}"]`;
      } else {
        const stableClass = [...element.classList].find((item) => !/^(active|selected|open|focus|hover|css-|sc-)/.test(item));
        if (stableClass) part += `.${escapeCss(stableClass)}`;
      }
      const parent = element.parentElement;
      if (parent) {
        const same = [...parent.children].filter((item) => item.localName === element.localName);
        if (same.length > 1) part += `:nth-of-type(${same.indexOf(element) + 1})`;
      }
      path.unshift(part);
      const selector = path.join(' > ');
      try {
        if (document.querySelectorAll(selector).length === 1) return selector;
      } catch (_) {}
      element = parent;
    }
    return path.join(' > ');
  };
  const style = document.createElement('style');
  style.textContent = '[data-bandoo-capture-hover="true"]{outline:2px solid #0f766e !important;outline-offset:2px !important;cursor:crosshair !important;}';
  document.documentElement.append(style);
  let current = null;
  const cleanup = () => {
    current?.removeAttribute('data-bandoo-capture-hover');
    style.remove();
    document.removeEventListener('mouseover', onMouseOver, true);
    document.removeEventListener('click', onClick, true);
    document.removeEventListener('keydown', onKeyDown, true);
    delete window.__BANDOO_SELECTOR_CAPTURE_CLEANUP__;
  };
  const finish = (selector) => {
    cleanup();
    if (selector) {
      window.__BANDOO__.workflow.selectorCaptured(selector);
      navigator.clipboard?.writeText?.(selector).catch(() => {});
    }
  };
  const onMouseOver = (event) => {
    if (!(event.target instanceof Element)) return;
    current?.removeAttribute('data-bandoo-capture-hover');
    current = event.target;
    current.setAttribute('data-bandoo-capture-hover', 'true');
  };
  const onClick = (event) => {
    event.preventDefault();
    event.stopPropagation();
    finish(selectorFor(event.target));
  };
  const onKeyDown = (event) => {
    if (event.key === 'Escape') cleanup();
  };
  window.__BANDOO_SELECTOR_CAPTURE_CLEANUP__ = cleanup;
  document.addEventListener('mouseover', onMouseOver, true);
  document.addEventListener('click', onClick, true);
  document.addEventListener('keydown', onKeyDown, true);
})();
"#
}

fn action_recording_script() -> &'static str {
    r#"
(() => {
  if (!window.__BANDOO__) return;
  window.__BANDOO_ACTION_RECORDING_CLEANUP__?.();
  const actions = [];
  const escapeCss = (value) => window.CSS?.escape
    ? window.CSS.escape(String(value))
    : String(value).replace(/[^a-zA-Z0-9_-]/g, '\\$&');
  const selectorFor = (target) => {
    if (!(target instanceof Element)) return '';
    if (target.id) return `#${escapeCss(target.id)}`;
    const testId = target.getAttribute('data-testid') || target.getAttribute('data-test');
    if (testId) return `[data-testid="${String(testId).replaceAll('"', '\\"')}"]`;
    const name = target.getAttribute('name');
    if (name) return `${target.localName.toLowerCase()}[name="${String(name).replaceAll('"', '\\"')}"]`;
    const path = [];
    let element = target;
    while (element && element.nodeType === 1 && element !== document.documentElement) {
      let part = element.localName.toLowerCase();
      const stableClass = [...element.classList].find((item) => !/^(active|selected|open|focus|hover|css-|sc-)/.test(item));
      if (stableClass) part += `.${escapeCss(stableClass)}`;
      const parent = element.parentElement;
      if (parent) {
        const same = [...parent.children].filter((item) => item.localName === element.localName);
        if (same.length > 1) part += `:nth-of-type(${same.indexOf(element) + 1})`;
      }
      path.unshift(part);
      const selector = path.join(' > ');
      try {
        if (document.querySelectorAll(selector).length === 1) return selector;
      } catch (_) {}
      element = parent;
    }
    return path.join(' > ');
  };
  const badge = document.createElement('div');
  badge.textContent = 'Bandoo recording';
  badge.style.cssText = 'position:fixed;z-index:2147483647;right:14px;bottom:14px;padding:8px 10px;border-radius:8px;background:#0f766e;color:#fff;font:12px system-ui,sans-serif;box-shadow:0 8px 24px rgba(15,23,42,.24);';
  document.documentElement.append(badge);
  const pushOrReplaceType = (selector, text) => {
    const last = actions[actions.length - 1];
    if (last?.kind === 'page-type' && last.selector === selector) {
      last.text = text;
    } else {
      actions.push({ kind: 'page-type', selector, text });
    }
  };
  const onClick = (event) => {
    const selector = selectorFor(event.target);
    if (selector) actions.push({ kind: 'page-click', selector });
  };
  const onInput = (event) => {
    const selector = selectorFor(event.target);
    if (!selector) return;
    const target = event.target;
    const text = 'value' in target ? target.value : target.textContent || '';
    pushOrReplaceType(selector, String(text));
  };
  const stop = () => {
    badge.remove();
    document.removeEventListener('click', onClick, true);
    document.removeEventListener('input', onInput, true);
    document.removeEventListener('keydown', onKeyDown, true);
    delete window.__BANDOO_ACTION_RECORDING_CLEANUP__;
    window.__BANDOO__.workflow.actionsRecorded(actions);
  };
  const onKeyDown = (event) => {
    if (event.key === 'Escape') stop();
  };
  window.__BANDOO_ACTION_RECORDING_CLEANUP__ = stop;
  document.addEventListener('click', onClick, true);
  document.addEventListener('input', onInput, true);
  document.addEventListener('keydown', onKeyDown, true);
  setTimeout(stop, 20000);
})();
"#
}

fn eval_content_script(app: AppHandle, web_app_id: String, script: &str) -> Result<(), String> {
    launch_webapp(app.clone(), web_app_id.clone())?;
    let label = content_label(&web_app_id);
    let webview = app
        .get_webview(&label)
        .ok_or_else(|| "WebApp content webview was not found after launch".to_string())?;
    webview.eval(script).map_err(|error| error.to_string())
}

pub fn start_selector_capture(app: AppHandle, web_app_id: String) -> Result<(), String> {
    eval_content_script(app, web_app_id, selector_capture_script())
}

pub fn start_action_recording(app: AppHandle, web_app_id: String) -> Result<(), String> {
    eval_content_script(app, web_app_id, action_recording_script())
}

pub fn execute_automation(
    app: AppHandle,
    automation: AutomationConfig,
) -> Result<AutomationRunResult, String> {
    let started_at = now_ms();
    let run_id = format!("automation-{}-{started_at}", automation.id);
    if !automation.enabled {
        return Ok(AutomationRunResult {
            run_id,
            automation_id: automation.id,
            web_app_id: automation.web_app_id,
            dispatched: false,
            message: "Automation is disabled".to_string(),
            steps: Vec::new(),
            started_at,
            finished_at: Some(now_ms()),
            duration_ms: Some(0),
            error: Some("Automation is disabled".to_string()),
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
            run_id,
            automation_id: automation.id,
            web_app_id: webapp_id,
            dispatched: false,
            message: "Automation conditions were not met".to_string(),
            steps: Vec::new(),
            started_at,
            finished_at: Some(now_ms()),
            duration_ms: Some(now_ms().saturating_sub(started_at)),
            error: Some("Automation conditions were not met".to_string()),
        });
    }

    let actions = automation.actions.clone();
    let preflight_steps = preflight_actions(&actions, &webapp.permissions);
    if let Some(failed) = preflight_steps
        .iter()
        .find(|step| step.status == "failed")
        .cloned()
    {
        let message = failed.message.clone();
        return Ok(AutomationRunResult {
            run_id,
            automation_id: automation.id,
            web_app_id: webapp_id,
            dispatched: false,
            message: message.clone(),
            steps: preflight_steps,
            started_at,
            finished_at: Some(now_ms()),
            duration_ms: Some(now_ms().saturating_sub(started_at)),
            error: Some(message),
        });
    }

    launch_webapp(app.clone(), webapp_id.clone())?;
    let label = content_label(&webapp_id);
    let webview = app
        .get_webview(&label)
        .ok_or_else(|| "WebApp content webview was not found after launch".to_string())?;

    let actions_json = serde_json::to_string(&actions).map_err(|error| error.to_string())?;
    let run_id_json = serde_json::to_string(&run_id).map_err(|error| error.to_string())?;
    let automation_id_json =
        serde_json::to_string(&automation.id).map_err(|error| error.to_string())?;
    let script = format!(
        r#"
(async () => {{
  const runId = {run_id_json};
  const sourceId = {automation_id_json};
  const startedAt = Date.now();
  if (!window.__BANDOO__?.automation?.run) throw new Error('Bandoo automation bridge is not available');
  try {{
    const result = await window.__BANDOO__.automation.run({actions_json});
    const finishedAt = Date.now();
    await window.__BANDOO__?.workflow?.log?.({{
      id: runId,
      sourceId,
      kind: 'automation',
      status: 'completed',
      message: 'Automation completed',
      steps: result.steps || [],
      startedAt,
      finishedAt,
      durationMs: finishedAt - startedAt
    }});
    console.info('[Bandoo automation]', result);
  }} catch (error) {{
    const finishedAt = Date.now();
    const message = error?.message || String(error);
    await window.__BANDOO__?.workflow?.log?.({{
      id: runId,
      sourceId,
      kind: 'automation',
      status: 'failed',
      message,
      steps: error?.steps || [],
      startedAt,
      finishedAt,
      durationMs: finishedAt - startedAt,
      error: message
    }});
    throw error;
  }}
}})().catch((error) => {{
  console.error('[Bandoo automation]', error);
  if (error.steps) console.table(error.steps);
}});
"#
    );
    webview.eval(script).map_err(|error| error.to_string())?;

    let dispatched_steps = preflight_steps
        .into_iter()
        .map(|step| AutomationStepResult {
            status: "dispatched".to_string(),
            ..step
        })
        .collect::<Vec<_>>();
    let message = "Automation was dispatched to the WebApp window".to_string();
    let _ = storage::append_run_log(
        &app,
        AutomationRunLog {
            id: run_id.clone(),
            source_id: automation.id.clone(),
            web_app_id: webapp_id.clone(),
            kind: "automation".to_string(),
            status: "dispatched".to_string(),
            message: message.clone(),
            steps: dispatched_steps.clone(),
            started_at,
            finished_at: None,
            duration_ms: None,
            error: None,
        },
    );

    Ok(AutomationRunResult {
        run_id,
        automation_id: automation.id,
        web_app_id: webapp_id,
        dispatched: true,
        message,
        steps: dispatched_steps,
        started_at,
        finished_at: None,
        duration_ms: None,
        error: None,
    })
}

pub fn execute_user_script(
    app: AppHandle,
    script: UserScriptConfig,
) -> Result<UserScriptRunResult, String> {
    let started_at = now_ms();
    let run_id = format!("script-{}-{started_at}", script.id);
    if !script.enabled {
        return Ok(UserScriptRunResult {
            run_id,
            script_id: script.id,
            web_app_id: script.web_app_id,
            dispatched: false,
            message: "User script is disabled".to_string(),
            started_at,
            finished_at: Some(now_ms()),
            duration_ms: Some(0),
            error: Some("User script is disabled".to_string()),
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
            run_id,
            script_id: script.id,
            web_app_id: webapp_id,
            dispatched: false,
            message: message.clone(),
            started_at,
            finished_at: Some(now_ms()),
            duration_ms: Some(now_ms().saturating_sub(started_at)),
            error: Some(message),
        });
    }

    launch_webapp(app.clone(), webapp_id.clone())?;
    let label = content_label(&webapp_id);
    let webview = app
        .get_webview(&label)
        .ok_or_else(|| "WebApp content webview was not found after launch".to_string())?;

    let script_id_json = serde_json::to_string(&script.id).map_err(|error| error.to_string())?;
    let script_name_json =
        serde_json::to_string(&script.name).map_err(|error| error.to_string())?;
    let run_id_json = serde_json::to_string(&run_id).map_err(|error| error.to_string())?;
    let source = script
        .compiled_code
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(&script.code);
    let code_json = serde_json::to_string(source).map_err(|error| error.to_string())?;
    let eval_script = format!(
        r#"
(async () => {{
  const runId = {run_id_json};
  const scriptId = {script_id_json};
  const scriptName = {script_name_json};
  const source = {code_json};
  const startedAt = Date.now();
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
      'shell',
      'fs',
      'network',
      `"use strict"; return (async () => {{\n${{source}}\n}})();`
    );
    await runner(
      bandoo,
      bandoo.app,
      bandoo.page,
      bandoo.clipboard,
      bandoo.notification,
      bandoo.workflow,
      bandoo.shell,
      bandoo.fs,
      bandoo.network
    );
    const finishedAt = Date.now();
    await bandoo.workflow?.log?.({{
      id: runId,
      sourceId: scriptId,
      kind: 'user-script',
      status: 'completed',
      message: `User script completed: ${{scriptName}}`,
      startedAt,
      finishedAt,
      durationMs: finishedAt - startedAt
    }});
    console.info('[Bandoo user script]', {{ scriptId, scriptName, ok: true }});
  }} catch (error) {{
    const finishedAt = Date.now();
    const message = error?.message || String(error);
    await bandoo.workflow?.log?.({{
      id: runId,
      sourceId: scriptId,
      kind: 'user-script',
      status: 'failed',
      message,
      startedAt,
      finishedAt,
      durationMs: finishedAt - startedAt,
      error: message
    }});
    console.error('[Bandoo user script]', {{ scriptId, scriptName, message, error }});
    await bandoo.notification?.send?.('Bandoo 用户脚本失败', message).catch(() => false);
  }}
}})().catch((error) => {{
  console.error('[Bandoo user script]', error);
}});
"#
    );
    webview
        .eval(eval_script)
        .map_err(|error| error.to_string())?;

    let message = "User script was dispatched to the WebApp window".to_string();
    let _ = storage::append_run_log(
        &app,
        AutomationRunLog {
            id: run_id.clone(),
            source_id: script.id.clone(),
            web_app_id: webapp_id.clone(),
            kind: "user-script".to_string(),
            status: "dispatched".to_string(),
            message: message.clone(),
            steps: Vec::new(),
            started_at,
            finished_at: None,
            duration_ms: None,
            error: None,
        },
    );

    Ok(UserScriptRunResult {
        run_id,
        script_id: script.id,
        web_app_id: webapp_id,
        dispatched: true,
        message,
        started_at,
        finished_at: None,
        duration_ms: None,
        error: None,
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
        "shell" if !permissions.shell => Some("Shell permission is disabled"),
        "fs-read" | "fs-write" if !permissions.filesystem => {
            Some("Filesystem permission is disabled")
        }
        "network-fetch" if !permissions.network => Some("Network permission is disabled"),
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
            duration_ms: None,
        };
    }

    AutomationStepResult {
        index,
        action_kind,
        status: "ready".to_string(),
        message: "Ready".to_string(),
        duration_ms: None,
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
            | "wait"
            | "sleep"
            | "shell"
            | "fs-read"
            | "fs-write"
            | "network-fetch"
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
            "shell" => permissions.shell,
            "filesystem" => permissions.filesystem,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_url_accepts_http_and_https() {
        assert!(validate_url("https://chatgpt.com").is_ok());
        assert!(validate_url("http://localhost:5173").is_ok());
    }

    #[test]
    fn validate_url_rejects_unsafe_schemes() {
        assert!(validate_url("file:///etc/passwd").is_err());
        assert!(validate_url("javascript:alert(1)").is_err());
    }

    #[test]
    fn preflight_rejects_disabled_high_risk_permissions() {
        let permissions = WebAppPermissions {
            page: true,
            clipboard: false,
            shell: false,
            filesystem: false,
            network: false,
            notification: false,
        };
        let step = preflight_action(
            1,
            &AutomationAction {
                kind: "shell".to_string(),
                selector: None,
                text: None,
                script: None,
                value: Some("echo hi".to_string()),
                timeout_ms: None,
                continue_on_error: None,
            },
            &permissions,
        );
        assert_eq!(step.status, "failed");
    }
}
