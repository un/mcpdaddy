use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

#[derive(Debug, thiserror::Error)]
pub enum ConfigStoreError {
    #[error("unable to resolve OS config directory")]
    NoProjectDirs,

    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

fn project_dirs() -> Result<ProjectDirs, ConfigStoreError> {
    ProjectDirs::from("com", "un", "mcp-daddy").ok_or(ConfigStoreError::NoProjectDirs)
}

pub fn config_dir() -> Result<PathBuf, ConfigStoreError> {
    Ok(project_dirs()?.config_dir().to_path_buf())
}

pub fn config_file_path() -> Result<PathBuf, ConfigStoreError> {
    Ok(config_dir()?.join("config.json"))
}

pub fn read_config_string() -> Result<Option<String>, ConfigStoreError> {
    let path = config_file_path()?;
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(fs::read_to_string(path)?))
}

pub fn write_config_string(contents: &str) -> Result<(), ConfigStoreError> {
    let path = config_file_path()?;
    ensure_parent_dir(&path)?;

    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, contents.as_bytes())?;
    fs::rename(&tmp_path, &path)?;
    Ok(())
}

fn ensure_parent_dir(path: &Path) -> Result<(), ConfigStoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}
