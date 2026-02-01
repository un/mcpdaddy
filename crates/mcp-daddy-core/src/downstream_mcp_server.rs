use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::time::Duration;

use crate::config::UpstreamServerV1;
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
    upstream_display_names: HashMap<String, String>,
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
            upstream_display_names: HashMap::new(),
            upstreams: HashMap::new(),
        }
    }

    pub fn with_upstream_servers(mut self, upstream_servers: Vec<UpstreamServerV1>) -> Self {
        self.upstream_display_names = upstream_servers
            .into_iter()
            .map(|u| (u.upstream_id, u.display_name))
            .collect();
        self
    }

    pub fn with_tools_cache(mut self, tools_cache: UpstreamToolsCacheStore) -> Self {
        self.tools_cache = tools_cache;
        self
    }

    pub fn add_upstream_client(&mut self, client: UpstreamMcpClient) {
        self.upstreams.insert(client.upstream_id.clone(), client);
    }

    fn handle_compact_meta_tool_call(
        &self,
        id: Option<Value>,
        tool_name: &str,
        _arguments: Value,
    ) -> Value {
        match tool_name {
            "mcpdaddy.integrations.list" => {
                let integrations = self
                    .profile
                    .allowed_upstream_ids
                    .iter()
                    .map(|upstream_id| {
                        let display_name = self
                            .upstream_display_names
                            .get(upstream_id)
                            .cloned()
                            .unwrap_or_else(|| upstream_id.clone());
                        json!({
                            "upstreamId": upstream_id,
                            "displayName": display_name
                        })
                    })
                    .collect::<Vec<_>>();

                let structured = json!({"integrations": integrations});
                let text = serde_json::to_string(&structured).unwrap_or_else(|_| "{}".to_string());
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{"type": "text", "text": text}],
                        "structuredContent": structured,
                        "isError": false
                    }
                })
            }

            "mcpdaddy.tools.search" | "mcpdaddy.tools.call" => {
                tool_call_error(id, "Not implemented")
            }

            _ => jsonrpc_error(id, -32602, "Unknown tool"),
        }
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
                let cursor = msg
                    .get("params")
                    .and_then(|p| p.get("cursor"))
                    .and_then(|v| v.as_str());
                let offset = match cursor {
                    Some(c) => match decode_cursor(c) {
                        Some(o) => o,
                        None => return Some(jsonrpc_error(id, -32602, "Invalid cursor")),
                    },
                    None => 0,
                };

                match self.profile.exposure_mode {
                    crate::config::ExposureMode::Full => {
                        for upstream_id in &self.profile.allowed_upstream_ids {
                            if let Some(cached) = self.tools_cache.get(upstream_id) {
                                tools.extend(namespace_tools(upstream_id, cached.tools));
                            }
                        }
                    }
                    crate::config::ExposureMode::Compact => {
                        tools = compact_meta_tools();
                    }
                }

                if offset > tools.len() {
                    return Some(jsonrpc_error(id, -32602, "Invalid cursor"));
                }

                let (page, next_cursor) = paginate(tools, offset, 50);

                let mut result = serde_json::Map::new();
                result.insert("tools".to_string(), Value::Array(page));
                if let Some(next_cursor) = next_cursor {
                    result.insert("nextCursor".to_string(), Value::String(next_cursor));
                }

                Some(json!({"jsonrpc": "2.0", "id": id, "result": Value::Object(result)}))
            }

            "tools/call" => {
                let params = msg.get("params");
                let tool_name = params.and_then(|p| p.get("name")).and_then(|v| v.as_str());
                let arguments = params.and_then(|p| p.get("arguments")).cloned();
                let (Some(tool_name), Some(arguments)) = (tool_name, arguments) else {
                    return Some(jsonrpc_error(id, -32602, "Invalid params"));
                };

                if self.profile.exposure_mode == crate::config::ExposureMode::Compact {
                    return Some(self.handle_compact_meta_tool_call(id, tool_name, arguments));
                }

                if self.profile.exposure_mode != crate::config::ExposureMode::Full {
                    return Some(jsonrpc_error(id, -32602, "Unknown tool"));
                }

                let (upstream_id, tool_name) = match parse_namespaced_tool_name(tool_name) {
                    Some((upstream_id, tool_name)) => {
                        if !self.profile.allowed_upstream_ids.contains(&upstream_id) {
                            return Some(jsonrpc_error(id, -32602, "Unknown tool"));
                        }
                        (upstream_id, tool_name)
                    }
                    None => return Some(jsonrpc_error(id, -32602, "Invalid params")),
                };

                if !tool_exists_in_cache(&self.tools_cache, &upstream_id, &tool_name) {
                    return Some(jsonrpc_error(id, -32602, "Unknown tool"));
                }

                let client = match self.upstreams.get_mut(&upstream_id) {
                    Some(c) => c,
                    None => {
                        return Some(tool_call_error(
                            id,
                            &format!("Upstream not connected: {upstream_id}"),
                        ));
                    }
                };

                match client.call_tool(&tool_name, arguments, Duration::from_secs(5)) {
                    Ok(result) => Some(json!({"jsonrpc": "2.0", "id": id, "result": result})),
                    Err(e) => Some(tool_call_error(id, &format!("Upstream error: {e}"))),
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

fn tool_call_error(id: Option<Value>, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": [{"type": "text", "text": message}],
            "isError": true
        }
    })
}

fn namespace_tools(upstream_id: &str, tools: Vec<Value>) -> Vec<Value> {
    tools
        .into_iter()
        .map(|mut t| {
            if let Some(name) = t.get("name").and_then(|v| v.as_str()) {
                t["name"] = json!(format!("{upstream_id}.{name}"));
            }
            t
        })
        .collect()
}

fn compact_meta_tools() -> Vec<Value> {
    vec![
        json!({
            "name": "mcpdaddy.integrations.list",
            "description": "List allowed upstream integrations.",
            "inputSchema": {"type": "object", "additionalProperties": false}
        }),
        json!({
            "name": "mcpdaddy.tools.search",
            "description": "Search tools across allowed upstream integrations.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "limit": {"type": "integer", "minimum": 1, "maximum": 200}
                },
                "required": ["query"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": "mcpdaddy.tools.call",
            "description": "Call an upstream tool by qualified name.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "qualifiedName": {"type": "string", "description": "<upstreamId>.<toolName>"},
                    "arguments": {"type": "object"}
                },
                "required": ["qualifiedName", "arguments"],
                "additionalProperties": false
            }
        }),
    ]
}

fn paginate(
    mut items: Vec<Value>,
    offset: usize,
    page_size: usize,
) -> (Vec<Value>, Option<String>) {
    let end = (offset + page_size).min(items.len());
    let next = if end < items.len() {
        Some(encode_cursor(end))
    } else {
        None
    };
    let page = items.drain(offset..end).collect();
    (page, next)
}

fn encode_cursor(offset: usize) -> String {
    format!("o:{offset}")
}

fn decode_cursor(cursor: &str) -> Option<usize> {
    let (prefix, rest) = cursor.split_once(':')?;
    if prefix != "o" {
        return None;
    }
    rest.parse::<usize>().ok()
}

fn parse_namespaced_tool_name(name: &str) -> Option<(String, String)> {
    let (prefix, rest) = name.split_once('.')?;
    if prefix.is_empty() || rest.is_empty() {
        return None;
    }
    Some((prefix.to_string(), rest.to_string()))
}

fn tool_exists_in_cache(
    cache: &UpstreamToolsCacheStore,
    upstream_id: &str,
    tool_name: &str,
) -> bool {
    let Some(cached) = cache.get(upstream_id) else {
        return false;
    };
    cached
        .tools
        .iter()
        .any(|t| t.get("name").and_then(|v| v.as_str()) == Some(tool_name))
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
        assert_eq!(tools[0]["name"], "a.allowed");
    }

    #[test]
    fn compact_mode_tools_list_is_meta_only() {
        let profile = ClientProfileV1 {
            profile_id: "p".to_string(),
            display_name: "P".to_string(),
            exposure_mode: ExposureMode::Compact,
            allowed_upstream_ids: vec!["a".to_string()],
        };

        let mut server = DownstreamMcpServer::new(profile);
        let resp = server
            .handle_message(json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}))
            .unwrap();

        let tools = resp["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 3);
        for t in tools {
            let name = t["name"].as_str().unwrap();
            assert!(name.starts_with("mcpdaddy."));
            assert!(t.get("inputSchema").is_some());
        }
    }

    #[test]
    fn compact_integrations_list_returns_allowed_only() {
        let profile = ClientProfileV1 {
            profile_id: "p".to_string(),
            display_name: "P".to_string(),
            exposure_mode: ExposureMode::Compact,
            allowed_upstream_ids: vec!["github".to_string(), "notion".to_string()],
        };
        let upstreams = vec![
            UpstreamServerV1 {
                upstream_id: "github".to_string(),
                display_name: "GitHub".to_string(),
            },
            UpstreamServerV1 {
                upstream_id: "notion".to_string(),
                display_name: "Notion".to_string(),
            },
            UpstreamServerV1 {
                upstream_id: "vercel".to_string(),
                display_name: "Vercel".to_string(),
            },
        ];

        let mut server = DownstreamMcpServer::new(profile).with_upstream_servers(upstreams);
        let resp = server
            .handle_message(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "mcpdaddy.integrations.list",
                    "arguments": {}
                }
            }))
            .unwrap();

        assert_eq!(resp["result"]["isError"], false);
        let list = resp["result"]["structuredContent"]["integrations"]
            .as_array()
            .unwrap();
        let ids: Vec<String> = list
            .iter()
            .map(|i| i["upstreamId"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"github".to_string()));
        assert!(ids.contains(&"notion".to_string()));
    }

    #[test]
    fn tools_list_namespaces_and_preserves_metadata() {
        let profile = ClientProfileV1 {
            profile_id: "p".to_string(),
            display_name: "P".to_string(),
            exposure_mode: ExposureMode::Full,
            allowed_upstream_ids: vec!["a".to_string(), "b".to_string()],
        };
        let cache = UpstreamToolsCacheStore::default();
        cache.set(
            "a",
            vec![json!({"name":"same","title":"T","description":"D","inputSchema":{"type":"object"}})],
        );
        cache.set(
            "b",
            vec![json!({"name":"same","title":"T2","description":"D2","inputSchema":{"type":"object"}})],
        );

        let mut server = DownstreamMcpServer::new(profile).with_tools_cache(cache);
        let resp = server
            .handle_message(json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}))
            .unwrap();
        let tools = resp["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 2);
        let names: Vec<String> = tools
            .iter()
            .map(|t| t["name"].as_str().unwrap().to_string())
            .collect();
        assert!(names.contains(&"a.same".to_string()));
        assert!(names.contains(&"b.same".to_string()));

        let a_tool = tools.iter().find(|t| t["name"] == "a.same").unwrap();
        assert_eq!(a_tool["title"], "T");
        assert_eq!(a_tool["description"], "D");
        assert_eq!(a_tool["inputSchema"]["type"], "object");
    }

    #[test]
    fn tools_list_paginates_with_cursor() {
        let profile = ClientProfileV1 {
            profile_id: "p".to_string(),
            display_name: "P".to_string(),
            exposure_mode: ExposureMode::Full,
            allowed_upstream_ids: vec!["a".to_string()],
        };
        let cache = UpstreamToolsCacheStore::default();
        let tools = (0..60)
            .map(|i| json!({"name": format!("t{i}"), "description": "d", "inputSchema": {"type":"object"}}))
            .collect::<Vec<_>>();
        cache.set("a", tools);

        let mut server = DownstreamMcpServer::new(profile).with_tools_cache(cache);
        let first = server
            .handle_message(json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}))
            .unwrap();
        let first_tools = first["result"]["tools"].as_array().unwrap();
        assert_eq!(first_tools.len(), 50);
        let cursor = first["result"]["nextCursor"].as_str().unwrap().to_string();

        let second = server
            .handle_message(
                json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{"cursor": cursor}}),
            )
            .unwrap();
        let second_tools = second["result"]["tools"].as_array().unwrap();
        assert_eq!(second_tools.len(), 10);
        assert!(second["result"].get("nextCursor").is_none());
        assert!(second_tools[0]["name"].as_str().unwrap().starts_with("a."));
    }

    #[test]
    fn tools_list_invalid_cursor_returns_invalid_params() {
        let profile = ClientProfileV1 {
            profile_id: "p".to_string(),
            display_name: "P".to_string(),
            exposure_mode: ExposureMode::Full,
            allowed_upstream_ids: vec![],
        };
        let mut server = DownstreamMcpServer::new(profile);
        let resp = server
            .handle_message(
                json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{"cursor":"bad"}}),
            )
            .unwrap();
        assert_eq!(resp["error"]["code"], -32602);
    }

    #[test]
    fn tools_call_rejects_disallowed_tool() {
        let profile = ClientProfileV1 {
            profile_id: "p".to_string(),
            display_name: "P".to_string(),
            exposure_mode: ExposureMode::Full,
            allowed_upstream_ids: vec!["a".to_string()],
        };

        let mut server = DownstreamMcpServer::new(profile);
        let resp = server
            .handle_message(json!({
                "jsonrpc":"2.0",
                "id": 10,
                "method": "tools/call",
                "params": {"name": "b.denied", "arguments": {}}
            }))
            .unwrap();

        assert_eq!(resp["error"]["code"], -32602);
    }

    #[test]
    fn tools_call_unknown_tool_returns_invalid_params() {
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
        let resp = server
            .handle_message(json!({
                "jsonrpc":"2.0",
                "id": 10,
                "method": "tools/call",
                "params": {"name": "a.nope", "arguments": {}}
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
                "params": {"name": "a.allowed", "arguments": {}}
            }))
            .unwrap();
        assert_eq!(resp["result"]["isError"], false);
        assert_eq!(resp["result"]["content"][0]["text"], "ok");
    }

    #[cfg(unix)]
    #[test]
    fn tools_call_upstream_failure_maps_to_is_error_true() {
        let script = r#"read l1; echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"fake","version":"0.0.0"}}}'; read l2; exit 0"#;
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
                "params": {"name": "a.allowed", "arguments": {}}
            }))
            .unwrap();

        assert!(resp.get("error").is_none());
        assert_eq!(resp["result"]["isError"], true);
    }

    #[cfg(unix)]
    #[test]
    fn tools_call_routes_to_correct_upstream_by_prefix() {
        let script_a = r#"read l1; echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"a","version":"0"}}}'; read l2; read l3; echo '{"jsonrpc":"2.0","id":2,"result":{"content":[{"type":"text","text":"from-a"}],"isError":false}}'; exit 0"#;
        let script_b = r#"read l1; echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"b","version":"0"}}}'; read l2; read l3; echo '{"jsonrpc":"2.0","id":2,"result":{"content":[{"type":"text","text":"from-b"}],"isError":false}}'; exit 0"#;

        let mut spec_a = UpstreamProcessSpec::new("sh");
        spec_a.args = vec!["-c".into(), script_a.into()];
        let mut spec_b = UpstreamProcessSpec::new("sh");
        spec_b.args = vec!["-c".into(), script_b.into()];

        let runtime = RuntimeStateStore::default();
        let mut upstream_a = UpstreamMcpClient::spawn("a", &spec_a, runtime.clone()).unwrap();
        let mut upstream_b = UpstreamMcpClient::spawn("b", &spec_b, runtime).unwrap();
        upstream_a.initialize(Duration::from_millis(500)).unwrap();
        upstream_b.initialize(Duration::from_millis(500)).unwrap();

        let profile = ClientProfileV1 {
            profile_id: "p".to_string(),
            display_name: "P".to_string(),
            exposure_mode: ExposureMode::Full,
            allowed_upstream_ids: vec!["a".to_string(), "b".to_string()],
        };

        let cache = UpstreamToolsCacheStore::default();
        cache.set(
            "a",
            vec![json!({"name":"allowed","description":"d","inputSchema":{"type":"object"}})],
        );
        cache.set(
            "b",
            vec![json!({"name":"allowed","description":"d","inputSchema":{"type":"object"}})],
        );

        let mut server = DownstreamMcpServer::new(profile).with_tools_cache(cache);
        server.add_upstream_client(upstream_a);
        server.add_upstream_client(upstream_b);

        let resp = server
            .handle_message(json!({
                "jsonrpc":"2.0",
                "id": 10,
                "method": "tools/call",
                "params": {"name": "b.allowed", "arguments": {}}
            }))
            .unwrap();
        assert_eq!(resp["result"]["content"][0]["text"], "from-b");
    }
}
