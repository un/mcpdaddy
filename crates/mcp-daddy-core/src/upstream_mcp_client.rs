use std::time::Duration;

use serde_json::{json, Value};

use crate::runtime_state::RuntimeStateStore;
use crate::stdio_jsonrpc_client::{JsonRpcStdioClient, JsonRpcStdioClientError};
use crate::upstream_process::{RunningUpstreamProcess, UpstreamProcessError, UpstreamProcessSpec};

pub const MCP_PROTOCOL_VERSION: &str = "2025-11-25";

#[derive(Debug, thiserror::Error)]
pub enum UpstreamMcpError {
    #[error("process error: {0}")]
    Process(#[from] UpstreamProcessError),

    #[error("json-rpc error: {0}")]
    JsonRpc(#[from] JsonRpcStdioClientError),

    #[error("unsupported protocol version: {0}")]
    UnsupportedProtocolVersion(String),

    #[error("invalid initialize response")]
    InvalidInitializeResponse,
}

#[derive(Debug, Clone)]
pub struct UpstreamInitializeResult {
    pub protocol_version: String,
    pub capabilities: Value,
    pub server_info: Value,
    pub instructions: Option<String>,
}

pub struct UpstreamMcpClient {
    pub upstream_id: String,
    rpc: JsonRpcStdioClient,
    runtime: RuntimeStateStore,
}

impl UpstreamMcpClient {
    pub fn spawn(
        upstream_id: impl Into<String>,
        spec: &UpstreamProcessSpec,
        runtime: RuntimeStateStore,
    ) -> Result<Self, UpstreamMcpError> {
        let process = RunningUpstreamProcess::spawn(spec)?;
        let rpc = JsonRpcStdioClient::new(process)?;
        Ok(Self {
            upstream_id: upstream_id.into(),
            rpc,
            runtime,
        })
    }

    pub fn initialize(
        &mut self,
        timeout: Duration,
    ) -> Result<UpstreamInitializeResult, UpstreamMcpError> {
        let params = json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": {
                "name": "mcp-daddy",
                "version": crate::build_version(),
            }
        });

        let result = self.rpc.request("initialize", Some(params), timeout)?;
        let parsed = parse_initialize_result(result)?;

        if parsed.protocol_version != MCP_PROTOCOL_VERSION {
            return Err(UpstreamMcpError::UnsupportedProtocolVersion(
                parsed.protocol_version,
            ));
        }

        // Must send notifications/initialized after successful initialize.
        self.rpc
            .send_notification("notifications/initialized", None)?;

        self.runtime.record_upstream_initialized(
            self.upstream_id.clone(),
            parsed.protocol_version.clone(),
            parsed.capabilities.clone(),
            parsed.server_info.clone(),
            parsed.instructions.clone(),
        );

        Ok(parsed)
    }

    pub fn stderr_lines_snapshot(&self) -> Vec<String> {
        self.rpc.stderr_lines_snapshot()
    }
}

fn parse_initialize_result(result: Value) -> Result<UpstreamInitializeResult, UpstreamMcpError> {
    let protocol_version = result
        .get("protocolVersion")
        .and_then(|v| v.as_str())
        .ok_or(UpstreamMcpError::InvalidInitializeResponse)?
        .to_string();

    let capabilities = result
        .get("capabilities")
        .cloned()
        .ok_or(UpstreamMcpError::InvalidInitializeResponse)?;

    let server_info = result
        .get("serverInfo")
        .cloned()
        .ok_or(UpstreamMcpError::InvalidInitializeResponse)?;

    let instructions = result
        .get("instructions")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(UpstreamInitializeResult {
        protocol_version,
        capabilities,
        server_info,
        instructions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[cfg(unix)]
    #[test]
    fn initialize_succeeds_and_records_capabilities() {
        let script = r#"read line; echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"fake","version":"0.0.0"}}}'; read line2; exit 0"#;

        let mut spec = UpstreamProcessSpec::new("sh");
        spec.args = vec!["-c".into(), script.into()];

        let runtime = RuntimeStateStore::default();
        let mut client = UpstreamMcpClient::spawn("fake-upstream", &spec, runtime.clone()).unwrap();

        let res = client.initialize(Duration::from_millis(500)).unwrap();
        assert_eq!(res.protocol_version, MCP_PROTOCOL_VERSION);
        assert!(res.capabilities.get("tools").is_some());

        let snap = runtime.snapshot();
        let u = snap
            .upstreams
            .iter()
            .find(|u| u.upstream_id == "fake-upstream")
            .unwrap();
        assert_eq!(u.protocol_version.as_deref(), Some(MCP_PROTOCOL_VERSION));
        assert!(u.capabilities.as_ref().unwrap().get("tools").is_some());
    }

    #[cfg(unix)]
    #[test]
    fn initialize_times_out() {
        let mut spec = UpstreamProcessSpec::new("sh");
        spec.args = vec!["-c".into(), "sleep 5".into()];

        let runtime = RuntimeStateStore::default();
        let mut client = UpstreamMcpClient::spawn("sleepy", &spec, runtime).unwrap();

        let err = client.initialize(Duration::from_millis(50)).unwrap_err();
        let UpstreamMcpError::JsonRpc(JsonRpcStdioClientError::Timeout { .. }) = err else {
            panic!("expected timeout, got: {err:?}");
        };
    }
}
