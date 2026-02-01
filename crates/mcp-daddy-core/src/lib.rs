pub const APP_NAME: &str = "mcp-daddy";

pub fn build_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub mod config;
pub mod config_migration;
pub mod config_store;
pub mod runtime_state;
pub mod stdio_framing;
pub mod stdio_jsonrpc_client;
pub mod upstream_mcp_client;
pub mod upstream_process;
pub mod upstream_tools_cache;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_nonempty() {
        assert!(!build_version().is_empty());
    }
}
