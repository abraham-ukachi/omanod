mod client;
mod config;
mod error;
mod models;
mod store;
mod theme;
mod themes_gen;

pub use client::{list_models, stream, StreamEvent, Usage};
pub use config::Config;
pub use error::{Error, Result};
pub use models::{catalog, provider, ModelInfo, ProviderConfig, ProviderKind};
pub use store::{Store, Thread};
pub use theme::{Rgb, Theme};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
  System,
  User,
  Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
  pub role: Role,
  pub content: String,
}

impl Message {
  pub fn user(content: impl Into<String>) -> Message {
    Message {
      role: Role::User,
      content: content.into(),
    }
  }

  pub fn assistant(content: impl Into<String>) -> Message {
    Message {
      role: Role::Assistant,
      content: content.into(),
    }
  }

  pub fn system(content: impl Into<String>) -> Message {
    Message {
      role: Role::System,
      content: content.into(),
    }
  }
}
