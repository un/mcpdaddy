use crate::config::{ClientProfileV1, ConfigV1};

pub const PROFILE_ENV_VAR: &str = "MCP_DADDY_PROFILE";

#[derive(Debug, thiserror::Error)]
pub enum ProfileSelectionError {
    #[error("no profile selected (use --profile <id> or set {PROFILE_ENV_VAR})")]
    MissingProfile,

    #[error("unknown profile: {requested}")]
    UnknownProfile {
        requested: String,
        known: Vec<String>,
    },
}

pub fn resolve_profile_id(args: &[String]) -> Option<String> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--profile" {
            return args.get(i + 1).cloned();
        }
        i += 1;
    }
    None
}

pub fn resolve_profile_id_with_env(args: &[String]) -> Option<String> {
    resolve_profile_id(args).or_else(|| std::env::var(PROFILE_ENV_VAR).ok())
}

pub fn validate_profile_exists(
    config: &ConfigV1,
    profile_id: &str,
) -> Result<ClientProfileV1, ProfileSelectionError> {
    match config
        .client_profiles
        .iter()
        .find(|p| p.profile_id == profile_id)
    {
        Some(p) => Ok(p.clone()),
        None => Err(ProfileSelectionError::UnknownProfile {
            requested: profile_id.to_string(),
            known: config
                .client_profiles
                .iter()
                .map(|p| p.profile_id.clone())
                .collect(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ConfigV1, ExposureMode};

    fn cfg() -> ConfigV1 {
        ConfigV1 {
            schema_version: crate::config::SCHEMA_VERSION_V1,
            upstream_servers: vec![],
            client_profiles: vec![
                ClientProfileV1 {
                    profile_id: "default".into(),
                    display_name: "Default".into(),
                    exposure_mode: ExposureMode::Compact,
                    allowed_upstream_ids: vec![],
                },
                ClientProfileV1 {
                    profile_id: "full".into(),
                    display_name: "Full".into(),
                    exposure_mode: ExposureMode::Full,
                    allowed_upstream_ids: vec![],
                },
            ],
        }
    }

    #[test]
    fn parses_profile_flag() {
        let args = vec!["mcp-daddy".into(), "--profile".into(), "full".into()];
        assert_eq!(resolve_profile_id(&args), Some("full".into()));
    }

    #[test]
    fn validate_profile_exists_ok() {
        let out = validate_profile_exists(&cfg(), "default").unwrap();
        assert_eq!(out.profile_id, "default");
    }

    #[test]
    fn validate_profile_exists_err_lists_known() {
        let err = validate_profile_exists(&cfg(), "nope").unwrap_err();
        let ProfileSelectionError::UnknownProfile { known, .. } = err else {
            panic!("expected unknown profile");
        };
        assert!(known.contains(&"default".to_string()));
    }
}
