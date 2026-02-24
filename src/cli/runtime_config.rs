use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};

use crate::core::errors::{ChacrabError, ChacrabResult};

const CONFIG_DIR: &str = ".config/chacrab";
const CONFIG_FILE: &str = "config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub backend: String,
    pub database_url: String,
}

pub fn cli_flag_present(args: &[String], flag: &str) -> bool {
    args.iter()
        .any(|arg| arg == flag || arg.starts_with(&format!("{flag}=")))
}

pub fn load() -> ChacrabResult<Option<RuntimeConfig>> {
    let path = config_file_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path)
        .map_err(|_| ChacrabError::Config("failed to read runtime config".to_owned()))?;
    let config = serde_json::from_str::<RuntimeConfig>(&content)
        .map_err(|_| ChacrabError::Config("invalid runtime config format".to_owned()))?;
    Ok(Some(config))
}

pub fn save(config: &RuntimeConfig) -> ChacrabResult<()> {
    let path = config_file_path()?;
    let Some(parent) = path.parent() else {
        return Err(ChacrabError::Config(
            "invalid runtime config path".to_owned(),
        ));
    };

    fs::create_dir_all(parent)
        .map_err(|_| ChacrabError::Config("failed to create config directory".to_owned()))?;
    let serialized = serde_json::to_string_pretty(config)
        .map_err(|_| ChacrabError::Config("failed to serialize runtime config".to_owned()))?;
    fs::write(path, serialized)
        .map_err(|_| ChacrabError::Config("failed to persist runtime config".to_owned()))?;
    Ok(())
}

fn config_file_path() -> ChacrabResult<PathBuf> {
    if let Ok(path) = env::var("CHACRAB_CONFIG_PATH") {
        return Ok(PathBuf::from(path));
    }

    let home = env::var("HOME")
        .map_err(|_| ChacrabError::Config("HOME environment variable is not set".to_owned()))?;
    Ok(PathBuf::from(home).join(CONFIG_DIR).join(CONFIG_FILE))
}
