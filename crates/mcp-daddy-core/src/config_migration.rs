use serde_json::Value;

use crate::config::{ConfigV1, SCHEMA_VERSION_V1};

pub const SUPPORTED_SCHEMA_VERSIONS: &[u32] = &[SCHEMA_VERSION_V1];

#[derive(Debug, thiserror::Error)]
pub enum ConfigMigrationError {
    #[error("invalid json: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("missing or invalid schemaVersion")]
    MissingOrInvalidSchemaVersion,

    #[error("unsupported schemaVersion={found}; supported versions: {supported:?}")]
    UnsupportedSchemaVersion {
        found: u32,
        supported: &'static [u32],
    },
}

/// Parse config JSON and migrate to the latest supported schema.
///
/// Supported schema versions: v1.
pub fn parse_and_migrate_to_latest(json: &str) -> Result<ConfigV1, ConfigMigrationError> {
    let v: Value = serde_json::from_str(json)?;
    let schema_version = v
        .get("schemaVersion")
        .and_then(|sv| sv.as_u64())
        .and_then(|sv| u32::try_from(sv).ok())
        .ok_or(ConfigMigrationError::MissingOrInvalidSchemaVersion)?;

    match schema_version {
        SCHEMA_VERSION_V1 => Ok(serde_json::from_value(v)?),
        other => Err(ConfigMigrationError::UnsupportedSchemaVersion {
            found: other,
            supported: SUPPORTED_SCHEMA_VERSIONS,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ClientProfileV1, ExposureMode, UpstreamServerV1};

    #[test]
    fn rejects_unknown_schema_version() {
        let json = r#"{"schemaVersion":999,"upstreamServers":[],"clientProfiles":[]}"#;
        let err = parse_and_migrate_to_latest(json).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("unsupported schemaVersion=999"));
    }

    #[test]
    fn accepts_v1() {
        let cfg = ConfigV1 {
            schema_version: SCHEMA_VERSION_V1,
            upstream_servers: vec![UpstreamServerV1 {
                upstream_id: "github".to_string(),
                display_name: "GitHub".to_string(),
            }],
            client_profiles: vec![ClientProfileV1 {
                profile_id: "default".to_string(),
                display_name: "Default".to_string(),
                exposure_mode: ExposureMode::Compact,
                allowed_upstream_ids: vec!["github".to_string()],
            }],
        };

        let json = serde_json::to_string(&cfg).unwrap();
        let out = parse_and_migrate_to_latest(&json).unwrap();
        assert_eq!(cfg, out);
    }
}
