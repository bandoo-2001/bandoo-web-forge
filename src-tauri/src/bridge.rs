use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};

use crate::{
    models::{
        AutomationRunLog, AutomationStepResult, BridgeRequest, BridgeResponse, WebAppPermissions,
    },
    storage,
};

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn response(message: impl Into<String>, data: Option<Value>) -> BridgeResponse {
    BridgeResponse {
        ok: true,
        message: message.into(),
        data,
    }
}

fn permission_enabled(permissions: &WebAppPermissions, capability: &str) -> bool {
    match capability {
        "shell" => permissions.shell,
        "filesystem" | "fs" => permissions.filesystem,
        "network" => permissions.network,
        "notification" => permissions.notification,
        "clipboard" => permissions.clipboard,
        "page" => permissions.page,
        "workflow" => true,
        _ => false,
    }
}

fn string_payload(payload: &Value, key: &str) -> Result<String, String> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("Missing `{key}`"))
}

fn record(app: &AppHandle, request: &BridgeRequest, status: &str, message: &str) {
    let _ = storage::append_run_log(
        app,
        AutomationRunLog {
            id: format!("bridge-{}-{}", request.web_app_id, now_ms()),
            source_id: request.operation.clone(),
            web_app_id: request.web_app_id.clone(),
            kind: "bridge".to_string(),
            status: status.to_string(),
            message: message.to_string(),
            steps: Vec::new(),
            started_at: now_ms(),
            finished_at: Some(now_ms()),
            duration_ms: Some(0),
            error: if status == "failed" {
                Some(message.to_string())
            } else {
                None
            },
        },
    );
}

pub fn handle(app: AppHandle, request: BridgeRequest) -> Result<BridgeResponse, String> {
    let webapp = storage::read_webapps(&app)?
        .into_iter()
        .find(|candidate| candidate.id == request.web_app_id)
        .ok_or_else(|| "WebApp not found".to_string())?;

    if !permission_enabled(&webapp.permissions, &request.capability) {
        let message = format!("Permission `{}` is disabled", request.capability);
        record(&app, &request, "failed", &message);
        return Err(message);
    }

    let result = match request.capability.as_str() {
        "shell" => shell(&request),
        "filesystem" | "fs" => filesystem(&request),
        "network" => network(&request),
        "workflow" => workflow(&app, &request),
        capability => Err(format!("Unsupported bridge capability: {capability}")),
    };

    match result {
        Ok(value) => {
            record(&app, &request, "completed", &value.message);
            Ok(value)
        }
        Err(error) => {
            record(&app, &request, "failed", &error);
            Err(error)
        }
    }
}

fn payload_string(payload: &Value, key: &str, fallback: &str) -> String {
    payload
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn payload_u64(payload: &Value, key: &str, fallback: u64) -> u64 {
    payload.get(key).and_then(Value::as_u64).unwrap_or(fallback)
}

fn workflow(app: &AppHandle, request: &BridgeRequest) -> Result<BridgeResponse, String> {
    match request.operation.as_str() {
        "log" => workflow_log(app, request),
        "selectorCaptured" => workflow_selector_captured(app, request),
        "actionsRecorded" => workflow_actions_recorded(app, request),
        operation => Err(format!("Unsupported workflow operation: {operation}")),
    }
}

fn workflow_log(app: &AppHandle, request: &BridgeRequest) -> Result<BridgeResponse, String> {
    let now = now_ms();
    let id = payload_string(
        &request.payload,
        "id",
        &format!("workflow-{}-{now}", request.web_app_id),
    );
    let source_id = payload_string(&request.payload, "sourceId", "workflow.log");
    let kind = payload_string(&request.payload, "kind", "workflow");
    let status = payload_string(&request.payload, "status", "completed");
    let message = payload_string(&request.payload, "message", "Workflow log");
    let started_at = payload_u64(&request.payload, "startedAt", now);
    let finished_at = request
        .payload
        .get("finishedAt")
        .and_then(Value::as_u64)
        .or(Some(now));
    let duration_ms = request
        .payload
        .get("durationMs")
        .and_then(Value::as_u64)
        .or_else(|| finished_at.map(|finished| finished.saturating_sub(started_at)));
    let steps = request
        .payload
        .get("steps")
        .cloned()
        .map(serde_json::from_value::<Vec<AutomationStepResult>>)
        .transpose()
        .map_err(|error| error.to_string())?
        .unwrap_or_default();
    let error = request
        .payload
        .get("error")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            if status == "failed" {
                Some(message.clone())
            } else {
                None
            }
        });

    storage::append_run_log(
        app,
        AutomationRunLog {
            id,
            source_id,
            web_app_id: request.web_app_id.clone(),
            kind,
            status,
            message,
            steps,
            started_at,
            finished_at,
            duration_ms,
            error,
        },
    )?;

    Ok(response("Workflow log recorded", None))
}

fn workflow_selector_captured(
    app: &AppHandle,
    request: &BridgeRequest,
) -> Result<BridgeResponse, String> {
    let selector = string_payload(&request.payload, "selector")?;
    app.emit(
        "bandoo-selector-captured",
        json!({
            "webAppId": request.web_app_id,
            "selector": selector
        }),
    )
    .map_err(|error| error.to_string())?;

    storage::append_run_log(
        app,
        AutomationRunLog {
            id: format!("selector-{}-{}", request.web_app_id, now_ms()),
            source_id: "selector-capture".to_string(),
            web_app_id: request.web_app_id.clone(),
            kind: "selector-capture".to_string(),
            status: "completed".to_string(),
            message: selector,
            steps: Vec::new(),
            started_at: now_ms(),
            finished_at: Some(now_ms()),
            duration_ms: Some(0),
            error: None,
        },
    )?;

    Ok(response("Selector captured", None))
}

fn workflow_actions_recorded(
    app: &AppHandle,
    request: &BridgeRequest,
) -> Result<BridgeResponse, String> {
    let actions = request
        .payload
        .get("actions")
        .cloned()
        .unwrap_or_else(|| json!([]));
    let count = actions.as_array().map(|items| items.len()).unwrap_or(0);
    app.emit(
        "bandoo-actions-recorded",
        json!({
            "webAppId": request.web_app_id,
            "actions": actions
        }),
    )
    .map_err(|error| error.to_string())?;

    storage::append_run_log(
        app,
        AutomationRunLog {
            id: format!("recording-{}-{}", request.web_app_id, now_ms()),
            source_id: "recording".to_string(),
            web_app_id: request.web_app_id.clone(),
            kind: "recording".to_string(),
            status: "completed".to_string(),
            message: format!("Recorded {count} actions"),
            steps: Vec::new(),
            started_at: now_ms(),
            finished_at: Some(now_ms()),
            duration_ms: Some(0),
            error: None,
        },
    )?;

    Ok(response("Actions recorded", None))
}

fn shell(request: &BridgeRequest) -> Result<BridgeResponse, String> {
    if request.operation != "exec" {
        return Err(format!(
            "Unsupported shell operation: {}",
            request.operation
        ));
    }

    let command = string_payload(&request.payload, "command")?;
    let mut process = if cfg!(target_os = "windows") {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", &command]);
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.args(["-lc", &command]);
        cmd
    };

    if let Some(cwd) = request.payload.get("cwd").and_then(Value::as_str) {
        process.current_dir(cwd);
    }

    let output = process.output().map_err(|error| error.to_string())?;
    Ok(response(
        "Shell command completed",
        Some(json!({
            "status": output.status.code(),
            "success": output.status.success(),
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr)
        })),
    ))
}

fn filesystem(request: &BridgeRequest) -> Result<BridgeResponse, String> {
    match request.operation.as_str() {
        "readText" => {
            let path = string_payload(&request.payload, "path")?;
            let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
            Ok(response("File read", Some(json!({ "text": text }))))
        }
        "writeText" => {
            let path = string_payload(&request.payload, "path")?;
            let text = string_payload(&request.payload, "text")?;
            fs::write(path, text).map_err(|error| error.to_string())?;
            Ok(response("File written", None))
        }
        "readDir" => {
            let path = string_payload(&request.payload, "path")?;
            let entries = fs::read_dir(path)
                .map_err(|error| error.to_string())?
                .filter_map(Result::ok)
                .map(|entry| {
                    let path = entry.path();
                    json!({
                        "name": entry.file_name().to_string_lossy(),
                        "path": path.display().to_string(),
                        "isDir": path.is_dir(),
                        "isFile": path.is_file()
                    })
                })
                .collect::<Vec<_>>();
            Ok(response(
                "Directory read",
                Some(json!({ "entries": entries })),
            ))
        }
        "exists" => {
            let path = string_payload(&request.payload, "path")?;
            Ok(response(
                "Path checked",
                Some(json!({ "exists": fs::metadata(path).is_ok() })),
            ))
        }
        "mkdir" => {
            let path = string_payload(&request.payload, "path")?;
            fs::create_dir_all(path).map_err(|error| error.to_string())?;
            Ok(response("Directory created", None))
        }
        "remove" => {
            let path = string_payload(&request.payload, "path")?;
            let metadata = fs::metadata(&path).map_err(|error| error.to_string())?;
            if metadata.is_dir() {
                fs::remove_dir_all(path).map_err(|error| error.to_string())?;
            } else {
                fs::remove_file(path).map_err(|error| error.to_string())?;
            }
            Ok(response("Path removed", None))
        }
        operation => Err(format!("Unsupported filesystem operation: {operation}")),
    }
}

fn network(request: &BridgeRequest) -> Result<BridgeResponse, String> {
    if request.operation != "fetch" {
        return Err(format!(
            "Unsupported network operation: {}",
            request.operation
        ));
    }

    let url = string_payload(&request.payload, "url")?;
    let method = request
        .payload
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or("GET")
        .to_ascii_uppercase();
    let client = reqwest::blocking::Client::new();
    let mut request_builder = match method.as_str() {
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "PATCH" => client.patch(url),
        "DELETE" => client.delete(url),
        _ => client.get(url),
    };

    if let Some(headers) = request.payload.get("headers").and_then(Value::as_object) {
        for (key, value) in headers {
            if let Some(value) = value.as_str() {
                request_builder = request_builder.header(key, value);
            }
        }
    }

    if let Some(body) = request.payload.get("body").and_then(Value::as_str) {
        request_builder = request_builder.body(body.to_string());
    }

    let response_value = request_builder.send().map_err(|error| error.to_string())?;
    let status = response_value.status().as_u16();
    let text = response_value.text().map_err(|error| error.to_string())?;
    Ok(response(
        "Network request completed",
        Some(json!({
            "status": status,
            "body": text
        })),
    ))
}
