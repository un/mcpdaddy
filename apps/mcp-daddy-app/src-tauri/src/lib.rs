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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            config_get,
            config_add_upstream_from_preset
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
