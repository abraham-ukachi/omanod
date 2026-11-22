use crate::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
  #[serde(default = "default_theme")]
  pub theme: String,
  #[serde(default = "default_provider")]
  pub provider: String,
  #[serde(default)]
  pub model: String,
  #[serde(default)]
  pub system_prompt: Option<String>,
  #[serde(default)]
  pub compatible_base_url: Option<String>,
}

fn default_theme() -> String {
  "Omarchy".into()
}

fn default_provider() -> String {
  "ollama".into()
}

impl Default for Config {
  fn default() -> Self {
    Self {
      theme: default_theme(),
      provider: default_provider(),
      model: String::new(),
      system_prompt: None,
      compatible_base_url: None,
    }
  }
}

impl Config {
  pub fn path() -> std::path::PathBuf {
    let config_home = std::env::var("XDG_CONFIG_HOME")
      .map(std::path::PathBuf::from)
      .unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/abrahamukachi".into());
        std::path::PathBuf::from(home).join(".config")
      });
    config_home.join("omanod").join("config.toml")
  }

  pub fn load() -> Result<Config> {
    let path = Self::path();
    if !path.exists() {
      let config = Config::default();
      config.save()?;
      return Ok(config);
    }
    let source = std::fs::read_to_string(&path)?;
    toml::from_str(&source).map_err(Error::from)
  }

  pub fn save(&self) -> Result<()> {
    let path = Self::path();
    if let Some(parent) = path.parent() {
      std::fs::create_dir_all(parent)?;
    }
    let source = toml::to_string_pretty(self).map_err(|e| Error::Message(e.to_string()))?;
    std::fs::write(path, source)?;
    Ok(())
  }
}
