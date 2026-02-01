use crate::config::{ClientProfileV1, ConfigV1, ExposureMode, SCHEMA_VERSION_V1};
use crate::config_migration::{parse_and_migrate_to_latest, ConfigMigrationError};
use crate::config_store::{read_config_string, ConfigStoreError};

#[derive(Debug, thiserror::Error)]
pub enum ConfigLoadError {
    #[error("config store error: {0}")]
    Store(#[from] ConfigStoreError),

    #[error("config migration error: {0}")]
    Migration(#[from] ConfigMigrationError),
}

pub fn default_config_v1() -> ConfigV1 {
    ConfigV1 {
        schema_version: SCHEMA_VERSION_V1,
        upstream_servers: vec![],
        client_profiles: vec![ClientProfileV1 {
            profile_id: "default".to_string(),
            display_name: "Default".to_string(),
            exposure_mode: ExposureMode::Compact,
            allowed_upstream_ids: vec![],
        }],
    }
}

pub fn load_config_or_default() -> Result<ConfigV1, ConfigLoadError> {
    match read_config_string()? {
        Some(s) => Ok(parse_and_migrate_to_latest(&s)?),
        None => Ok(default_config_v1()),
    }
}
