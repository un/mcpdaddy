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

    #[error("invalid tools/list response")]
    InvalidToolsListResponse,
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

    pub fn refresh_tools_cache(
        &mut self,
        cache: &crate::upstream_tools_cache::UpstreamToolsCacheStore,
        timeout: Duration,
    ) -> Result<Vec<Value>, UpstreamMcpError> {
        match self.fetch_all_tools(timeout) {
            Ok(tools) => {
                cache.set(self.upstream_id.clone(), tools.clone());
                Ok(tools)
            }
            Err(e) => {
                self.runtime.set_upstream_status(
                    self.upstream_id.clone(),
                    crate::runtime_state::UpstreamStatus::Unhealthy,
                    Some(e.to_string()),
                );
                Err(e)
            }
        }
    }

    pub fn cached_tools(
        &self,
        cache: &crate::upstream_tools_cache::UpstreamToolsCacheStore,
    ) -> Option<Vec<Value>> {
        cache.get(&self.upstream_id).map(|c| c.tools)
    }

    pub fn fetch_all_tools(&mut self, timeout: Duration) -> Result<Vec<Value>, UpstreamMcpError> {
        let mut all = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let (tools, next) = self.fetch_tools_page(cursor, timeout)?;
            all.extend(tools);
            cursor = next;
            if cursor.is_none() {
                break;
            }
        }

        Ok(all)
    }

    fn fetch_tools_page(
        &mut self,
        cursor: Option<String>,
        timeout: Duration,
    ) -> Result<(Vec<Value>, Option<String>), UpstreamMcpError> {
        self.runtime
            .record_upstream_tool_call(self.upstream_id.clone());

        let params = cursor.map(|cursor| json!({ "cursor": cursor }));
        let result = self.rpc.request("tools/list", params, timeout)?;
        let tools = result
            .get("tools")
            .and_then(|v| v.as_array())
            .ok_or(UpstreamMcpError::InvalidToolsListResponse)?
            .iter()
            .cloned()
            .collect::<Vec<_>>();

        let next_cursor = result
            .get("nextCursor")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok((tools, next_cursor))
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
    use crate::upstream_tools_cache::UpstreamToolsCacheStore;
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

    #[cfg(unix)]
    #[test]
    fn tools_list_is_cached_and_refreshable() {
        let script = r#"read l1; echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"fake","version":"0.0.0"}}}'; read l2; read l3; echo '{"jsonrpc":"2.0","id":2,"result":{"tools":[{"name":"t1","description":"d","inputSchema":{"type":"object"}}]}}'; read l4; exit 0"#;

        let mut spec = UpstreamProcessSpec::new("sh");
        spec.args = vec!["-c".into(), script.into()];
        let runtime = RuntimeStateStore::default();
        let mut client = UpstreamMcpClient::spawn("fake-upstream", &spec, runtime).unwrap();
        client.initialize(Duration::from_millis(500)).unwrap();

        let cache = UpstreamToolsCacheStore::default();
        assert!(client.cached_tools(&cache).is_none());

        let tools = client
            .refresh_tools_cache(&cache, Duration::from_millis(500))
            .unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "t1");

        let cached = client.cached_tools(&cache).unwrap();
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0]["name"], "t1");
    }

    #[cfg(unix)]
    #[test]
    fn tools_list_failure_marks_upstream_unhealthy() {
        let script = r#"read l1; echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"fake","version":"0.0.0"}}}'; read l2; exit 0"#;

        let mut spec = UpstreamProcessSpec::new("sh");
        spec.args = vec!["-c".into(), script.into()];

        let runtime = RuntimeStateStore::default();
        let mut client = UpstreamMcpClient::spawn("fake-upstream", &spec, runtime.clone()).unwrap();
        client.initialize(Duration::from_millis(500)).unwrap();

        let cache = UpstreamToolsCacheStore::default();
        let err = client
            .refresh_tools_cache(&cache, Duration::from_millis(200))
            .unwrap_err();

        // Error type can vary (stdout closed / timeout), but status must be unhealthy.
        let _ = err;
        let snap = runtime.snapshot();
        let u = snap
            .upstreams
            .iter()
            .find(|u| u.upstream_id == "fake-upstream")
            .unwrap();
        assert_eq!(u.status, crate::runtime_state::UpstreamStatus::Unhealthy);
        assert!(u.last_error.as_ref().is_some());
    }
}
