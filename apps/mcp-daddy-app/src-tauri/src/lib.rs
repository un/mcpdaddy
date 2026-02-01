// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddUpstreamFromPresetInput {
    upstream_id: String,
    display_name: String,
    command: String,
    args: Vec<String>,
    env: std::collections::HashMap<String, String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpsertUpstreamInput {
    upstream_id: String,
    display_name: String,
    command: String,
    args: Vec<String>,
    env: std::collections::HashMap<String, String>,
    cwd: Option<String>,
}

#[tauri::command]
fn config_get() -> Result<mcp_daddy_core::config::ConfigV1, String> {
    mcp_daddy_core::config_loader::load_config_or_default().map_err(|e| e.to_string())
}

#[tauri::command]
fn config_add_upstream_from_preset(
    input: AddUpstreamFromPresetInput,
) -> Result<mcp_daddy_core::config::ConfigV1, String> {
    let mut cfg =
        mcp_daddy_core::config_loader::load_config_or_default().map_err(|e| e.to_string())?;

    let upstream = mcp_daddy_core::config::UpstreamServerV1 {
        upstream_id: input.upstream_id,
        display_name: input.display_name,
        command: Some(input.command),
        args: input.args,
        env: input.env,
        cwd: None,
    };

    if let Some(existing) = cfg
        .upstream_servers
        .iter_mut()
        .find(|u| u.upstream_id == upstream.upstream_id)
    {
        *existing = upstream;
    } else {
        cfg.upstream_servers.push(upstream);
    }

    let json = serde_json::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
    mcp_daddy_core::config_store::write_config_string(&json).map_err(|e| e.to_string())?;
    Ok(cfg)
}

#[tauri::command]
fn config_upsert_upstream(
    input: UpsertUpstreamInput,
) -> Result<mcp_daddy_core::config::ConfigV1, String> {
    let mut cfg =
        mcp_daddy_core::config_loader::load_config_or_default().map_err(|e| e.to_string())?;

    let upstream = mcp_daddy_core::config::UpstreamServerV1 {
        upstream_id: input.upstream_id,
        display_name: input.display_name,
        command: Some(input.command),
        args: input.args,
        env: input.env,
        cwd: input.cwd,
    };

    if let Some(existing) = cfg
        .upstream_servers
        .iter_mut()
        .find(|u| u.upstream_id == upstream.upstream_id)
    {
        *existing = upstream;
    } else {
        cfg.upstream_servers.push(upstream);
    }

    let json = serde_json::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
    mcp_daddy_core::config_store::write_config_string(&json).map_err(|e| e.to_string())?;
    Ok(cfg)
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TestConnectionResult {
    ok: bool,
    tool_count: usize,
    stderr: Vec<String>,
    error: Option<String>,
}

#[tauri::command]
fn upstream_test_connection(upstream_id: String) -> TestConnectionResult {
    use std::path::PathBuf;
    use std::time::Duration;

    let cfg = match mcp_daddy_core::config_loader::load_config_or_default() {
        Ok(c) => c,
        Err(e) => {
            return TestConnectionResult {
                ok: false,
                tool_count: 0,
                stderr: vec![],
                error: Some(format!("failed to load config: {e}")),
            }
        }
    };

    let upstream = match cfg
        .upstream_servers
        .iter()
        .find(|u| u.upstream_id == upstream_id)
    {
        Some(u) => u,
        None => {
            return TestConnectionResult {
                ok: false,
                tool_count: 0,
                stderr: vec![],
                error: Some("unknown upstreamId".to_string()),
            }
        }
    };

    let Some(command) = upstream.command.clone() else {
        return TestConnectionResult {
            ok: false,
            tool_count: 0,
            stderr: vec![],
            error: Some("upstream has no command configured".to_string()),
        };
    };

    let mut spec = mcp_daddy_core::upstream_process::UpstreamProcessSpec::new(command);
    spec.args = upstream.args.clone();
    spec.env = upstream.env.clone();
    spec.cwd = upstream.cwd.as_ref().map(PathBuf::from);

    let runtime = mcp_daddy_core::runtime_state::RuntimeStateStore::default();
    let mut client = match mcp_daddy_core::upstream_mcp_client::UpstreamMcpClient::spawn(
        upstream.upstream_id.clone(),
        &spec,
        runtime,
    ) {
        Ok(c) => c,
        Err(e) => {
            return TestConnectionResult {
                ok: false,
                tool_count: 0,
                stderr: vec![],
                error: Some(format!("failed to spawn: {e}")),
            }
        }
    };

    if let Err(e) = client.initialize(Duration::from_secs(2)) {
        return TestConnectionResult {
            ok: false,
            tool_count: 0,
            stderr: client.stderr_lines_snapshot(),
            error: Some(format!("initialize failed: {e}")),
        };
    }

    match client.fetch_all_tools(Duration::from_secs(3)) {
        Ok(tools) => TestConnectionResult {
            ok: true,
            tool_count: tools.len(),
            stderr: client.stderr_lines_snapshot(),
            error: None,
        },
        Err(e) => TestConnectionResult {
            ok: false,
            tool_count: 0,
            stderr: client.stderr_lines_snapshot(),
            error: Some(format!("tools/list failed: {e}")),
        },
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            config_get,
            config_add_upstream_from_preset,
            config_upsert_upstream,
            upstream_test_connection
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
