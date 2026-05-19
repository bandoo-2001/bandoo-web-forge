use tauri::{
    AppHandle, Manager, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};
use url::Url;

use crate::{
    models::{WebApp, WebAppWindowState},
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
