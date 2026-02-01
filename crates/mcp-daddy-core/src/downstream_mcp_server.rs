use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::time::Duration;

use crate::stdio_framing::{write_jsonrpc_line, JsonRpcLineReader, StdioFramingError};
use crate::upstream_mcp_client::MCP_PROTOCOL_VERSION;
use crate::upstream_tools_cache::UpstreamToolsCacheStore;
use crate::{config::ClientProfileV1, upstream_mcp_client::UpstreamMcpClient};

#[derive(Debug, thiserror::Error)]
pub enum DownstreamServerError {
    #[error("stdio framing error: {0}")]
    Framing(#[from] StdioFramingError),

    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

#[derive(Debug, Clone)]
pub struct DownstreamServerInfo {
    pub name: String,
    pub version: String,
}

pub struct DownstreamMcpServer {
    pub server_info: DownstreamServerInfo,
    pub profile: ClientProfileV1,
    pub tools_cache: UpstreamToolsCacheStore,
    upstreams: HashMap<String, UpstreamMcpClient>,
}

impl DownstreamMcpServer {
    pub fn new(profile: ClientProfileV1) -> Self {
        Self {
            server_info: DownstreamServerInfo {
                name: "mcp-daddy".to_string(),
                version: crate::build_version().to_string(),
            },
            profile,
            tools_cache: UpstreamToolsCacheStore::default(),
            upstreams: HashMap::new(),
        }
    }

    pub fn with_tools_cache(mut self, tools_cache: UpstreamToolsCacheStore) -> Self {
        self.tools_cache = tools_cache;
        self
    }

    pub fn add_upstream_client(&mut self, client: UpstreamMcpClient) {
        self.upstreams.insert(client.upstream_id.clone(), client);
    }

    pub fn handle_message(&mut self, msg: Value) -> Option<Value> {
        let method = msg.get("method")?.as_str()?;
        let id = msg.get("id").cloned();

        match method {
            "initialize" => {
                let requested = msg
                    .get("params")
                    .and_then(|p| p.get("protocolVersion"))
                    .and_then(|v| v.as_str());

                // Version negotiation: respond with a supported version.
                let protocol_version = match requested {
                    Some(MCP_PROTOCOL_VERSION) => MCP_PROTOCOL_VERSION,
                    _ => MCP_PROTOCOL_VERSION,
                };

                Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": protocol_version,
                        "capabilities": {
                            "tools": {}
                        },
                        "serverInfo": {
                            "name": self.server_info.name,
                            "version": self.server_info.version,
                        }
                    }
                }))
            }

            "notifications/initialized" => None,

            "tools/list" => {
                let mut tools: Vec<Value> = Vec::new();
                for upstream_id in &self.profile.allowed_upstream_ids {
                    if let Some(cached) = self.tools_cache.get(upstream_id) {
                        tools.extend(cached.tools);
                    }
                }

                Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": tools
                    }
                }))
            }

            "tools/call" => {
                let params = msg.get("params");
                let tool_name = params.and_then(|p| p.get("name")).and_then(|v| v.as_str());
                let arguments = params.and_then(|p| p.get("arguments")).cloned();
                let (Some(tool_name), Some(arguments)) = (tool_name, arguments) else {
                    return Some(jsonrpc_error(id, -32602, "Invalid params"));
                };

                let matches = find_tool_upstreams(
                    &self.tools_cache,
                    &self.profile.allowed_upstream_ids,
                    tool_name,
                );
                let upstream_id = match matches.as_slice() {
                    [] => return Some(jsonrpc_error(id, -32602, "Unknown tool")),
                    [only] => only.clone(),
                    _ => {
                        return Some(jsonrpc_error(
                            id,
                            -32602,
                            "Ambiguous tool name; use namespaced tool name",
                        ));
                    }
                };

                let client = match self.upstreams.get_mut(&upstream_id) {
                    Some(c) => c,
                    None => return Some(jsonrpc_error(id, -32000, "Upstream not connected")),
                };

                match client.call_tool(tool_name, arguments, Duration::from_secs(5)) {
                    Ok(result) => Some(json!({"jsonrpc": "2.0", "id": id, "result": result})),
                    Err(e) => Some(jsonrpc_error(id, -32000, &format!("Upstream error: {e}"))),
                }
            }

            _ => {
                // Method not found.
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32601,
                        "message": "Method not found"
                    }
                }))
            }
        }
    }

    pub fn serve_stdio<R: BufRead, W: Write>(
        &mut self,
        mut reader: R,
        mut writer: W,
    ) -> Result<(), DownstreamServerError> {
        let mut framing = JsonRpcLineReader::new(&mut reader);
        while let Some(msg) = framing.next_message()? {
            if let Some(resp) = self.handle_message(msg) {
                write_jsonrpc_line(&mut writer, &resp)?;
            }
        }
        Ok(())
    }
}

fn jsonrpc_error(id: Option<Value>, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

fn find_tool_upstreams(
    cache: &UpstreamToolsCacheStore,
    allowed_upstream_ids: &[String],
    tool_name: &str,
) -> Vec<String> {
    let mut out = Vec::new();
    for upstream_id in allowed_upstream_ids {
        let Some(cached) = cache.get(upstream_id) else {
            continue;
        };
        if cached
            .tools
            .iter()
            .any(|t| t.get("name").and_then(|v| v.as_str()) == Some(tool_name))
        {
            out.push(upstream_id.clone());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ExposureMode;
    use crate::runtime_state::RuntimeStateStore;
    use crate::upstream_process::UpstreamProcessSpec;
    use std::io::Cursor;

    #[test]
    fn initialize_advertises_tools_and_protocol_version() {
        let profile = ClientProfileV1 {
            profile_id: "default".to_string(),
            display_name: "Default".to_string(),
            exposure_mode: ExposureMode::Compact,
            allowed_upstream_ids: vec![],
        };
        let mut server = DownstreamMcpServer::new(profile);
        let req = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "0" }
            }
        });

        let resp = server.handle_message(req).unwrap();
        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 1);
        assert_eq!(resp["result"]["protocolVersion"], MCP_PROTOCOL_VERSION);
        assert!(resp["result"]["capabilities"].get("tools").is_some());
    }

    #[test]
    fn serve_stdio_writes_initialize_response() {
        let profile = ClientProfileV1 {
            profile_id: "default".to_string(),
            display_name: "Default".to_string(),
            exposure_mode: ExposureMode::Compact,
            allowed_upstream_ids: vec![],
        };
        let mut server = DownstreamMcpServer::new(profile);
        let input = format!(
            "{}\n",
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {"protocolVersion": MCP_PROTOCOL_VERSION, "capabilities": {}, "clientInfo": {"name":"x","version":"0"}}
            })
        );

        let mut out = Vec::new();
        server
            .serve_stdio(Cursor::new(input.as_bytes()), &mut out)
            .unwrap();
        let out_s = std::str::from_utf8(&out).unwrap();
        assert!(out_s.contains("\"protocolVersion\""));
        assert!(out_s.contains(MCP_PROTOCOL_VERSION));
    }

    #[test]
    fn tools_list_respects_allowlist() {
        let profile = ClientProfileV1 {
            profile_id: "p".to_string(),
            display_name: "P".to_string(),
            exposure_mode: ExposureMode::Full,
            allowed_upstream_ids: vec!["a".to_string()],
        };
        let cache = UpstreamToolsCacheStore::default();
        cache.set(
            "a",
            vec![json!({"name":"allowed","description":"d","inputSchema":{"type":"object"}})],
        );
        cache.set(
            "b",
            vec![json!({"name":"denied","description":"d","inputSchema":{"type":"object"}})],
        );

        let mut server = DownstreamMcpServer::new(profile).with_tools_cache(cache);
        let resp = server
            .handle_message(json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}))
            .unwrap();
        let tools = resp["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "allowed");
    }

    #[test]
    fn tools_call_rejects_disallowed_tool() {
        let profile = ClientProfileV1 {
            profile_id: "p".to_string(),
            display_name: "P".to_string(),
            exposure_mode: ExposureMode::Full,
            allowed_upstream_ids: vec!["a".to_string()],
        };
        let cache = UpstreamToolsCacheStore::default();
        cache.set(
            "b",
            vec![json!({"name":"denied","description":"d","inputSchema":{"type":"object"}})],
        );

        let mut server = DownstreamMcpServer::new(profile).with_tools_cache(cache);
        let resp = server
            .handle_message(json!({
                "jsonrpc":"2.0",
                "id": 10,
                "method": "tools/call",
                "params": {"name": "denied", "arguments": {}}
            }))
            .unwrap();

        assert_eq!(resp["error"]["code"], -32602);
    }

    #[cfg(unix)]
    #[test]
    fn tools_call_routes_to_allowed_upstream() {
        let script = r#"read l1; echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"fake","version":"0.0.0"}}}'; read l2; read l3; echo '{"jsonrpc":"2.0","id":2,"result":{"content":[{"type":"text","text":"ok"}],"isError":false}}'; exit 0"#;
        let mut spec = UpstreamProcessSpec::new("sh");
        spec.args = vec!["-c".into(), script.into()];

        let runtime = RuntimeStateStore::default();
        let mut upstream = UpstreamMcpClient::spawn("a", &spec, runtime).unwrap();
        upstream.initialize(Duration::from_millis(500)).unwrap();

        let profile = ClientProfileV1 {
            profile_id: "p".to_string(),
            display_name: "P".to_string(),
            exposure_mode: ExposureMode::Full,
            allowed_upstream_ids: vec!["a".to_string()],
        };

        let cache = UpstreamToolsCacheStore::default();
        cache.set(
            "a",
            vec![json!({"name":"allowed","description":"d","inputSchema":{"type":"object"}})],
        );

        let mut server = DownstreamMcpServer::new(profile).with_tools_cache(cache);
        server.add_upstream_client(upstream);

        let resp = server
            .handle_message(json!({
                "jsonrpc":"2.0",
                "id": 10,
                "method": "tools/call",
                "params": {"name": "allowed", "arguments": {}}
            }))
            .unwrap();
        assert_eq!(resp["result"]["isError"], false);
        assert_eq!(resp["result"]["content"][0]["text"], "ok");
    }
}
