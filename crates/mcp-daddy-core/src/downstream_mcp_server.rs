use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

use crate::stdio_framing::{write_jsonrpc_line, JsonRpcLineReader, StdioFramingError};
use crate::upstream_mcp_client::MCP_PROTOCOL_VERSION;

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

#[derive(Debug, Clone)]
pub struct DownstreamMcpServer {
    pub server_info: DownstreamServerInfo,
}

impl DownstreamMcpServer {
    pub fn new() -> Self {
        Self {
            server_info: DownstreamServerInfo {
                name: "mcp-daddy".to_string(),
                version: crate::build_version().to_string(),
            },
        }
    }

    pub fn handle_message(&self, msg: Value) -> Option<Value> {
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
        &self,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn initialize_advertises_tools_and_protocol_version() {
        let server = DownstreamMcpServer::new();
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
        let server = DownstreamMcpServer::new();
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
}
