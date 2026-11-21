use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
  #[error("{0}")]
  Message(String),

  #[error("I/O failed: {0}")]
  Io(#[from] std::io::Error),

  #[error("JSON failed: {0}")]
  Json(#[from] serde_json::Error),

  #[error("TOML failed: {0}")]
  Toml(#[from] toml::de::Error),

  #[error("storage failed: {0}")]
  Db(#[from] rusqlite::Error),

  #[error("HTTP {status}: {body}")]
  Http { status: u16, body: String },

  #[error("configuration: {0}")]
  Config(String),
}

pub type Result<T> = std::result::Result<T, Error>;
