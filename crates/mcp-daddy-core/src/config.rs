use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const SCHEMA_VERSION_V1: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigV1 {
    pub schema_version: u32,
    pub upstream_servers: Vec<UpstreamServerV1>,
    pub client_profiles: Vec<ClientProfileV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpstreamServerV1 {
    pub upstream_id: String,
    pub display_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientProfileV1 {
    pub profile_id: String,
    pub display_name: String,
    pub exposure_mode: ExposureMode,
    pub allowed_upstream_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExposureMode {
    Full,
    Compact,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip_config_v1() {
        let cfg = ConfigV1 {
            schema_version: SCHEMA_VERSION_V1,
            upstream_servers: vec![UpstreamServerV1 {
                upstream_id: "github".to_string(),
                display_name: "GitHub".to_string(),
                command: None,
                args: vec![],
                env: Default::default(),
                cwd: None,
            }],
            client_profiles: vec![ClientProfileV1 {
                profile_id: "default".to_string(),
                display_name: "Default".to_string(),
                exposure_mode: ExposureMode::Compact,
                allowed_upstream_ids: vec!["github".to_string()],
            }],
        };

        let json = serde_json::to_string(&cfg).expect("serialize");
        let decoded: ConfigV1 = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(cfg, decoded);
    }

    #[test]
    fn exposure_mode_serializes_as_lowercase() {
        let full = serde_json::to_string(&ExposureMode::Full).unwrap();
        let compact = serde_json::to_string(&ExposureMode::Compact).unwrap();
        assert_eq!(full, "\"full\"");
        assert_eq!(compact, "\"compact\"");
    }
}
