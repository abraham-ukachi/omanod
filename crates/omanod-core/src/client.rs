use crate::{Error, Message, ModelInfo, ProviderConfig, ProviderKind, Result, Role};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Usage {
  pub input: u64,
  pub output: u64,
}

#[derive(Debug)]
pub enum StreamEvent {
  Delta(String),
  Done(Usage),
  Error(String),
}

fn redact(mut text: String) -> String {
  for config in crate::models::catalog() {
    let Some(name) = config.api_key_env.as_deref() else {
      continue;
    };
    if let Ok(value) = std::env::var(name) {
      if value.len() >= 6 {
        text = text.replace(&value, "[redacted]");
      }
    }
  }
  text
}

fn response_body(response: ureq::Response) -> String {
  let mut body = String::new();
  let mut reader = response.into_reader();
  let _ = reader.read_to_string(&mut body);
  body.truncate(8192);
  redact(body)
}

fn send_get(url: &str, headers: &[(&str, String)]) -> Result<Box<dyn Read + Send + Sync>> {
  let mut request = ureq::get(url).timeout(std::time::Duration::from_secs(30));
  for (name, value) in headers {
    request = request.set(name, value);
  }
  match request.call() {
    Ok(response) => Ok(response.into_reader()),
    Err(ureq::Error::Status(status, response)) => Err(Error::Http {
      status,
      body: response_body(response),
    }),
    Err(error) => Err(Error::Message(redact(error.to_string()))),
  }
}

fn send_post(url: &str, headers: &[(&str, String)], body: &str) -> Result<Box<dyn Read + Send + Sync>> {
  let mut request = ureq::post(url)
    .timeout(std::time::Duration::from_secs(120))
    .set("Content-Type", "application/json");
  for (name, value) in headers {
    request = request.set(name, value);
  }
  match request.send_string(body) {
    Ok(response) => Ok(response.into_reader()),
    Err(ureq::Error::Status(status, response)) => Err(Error::Http {
      status,
      body: response_body(response),
    }),
    Err(error) => Err(Error::Message(redact(error.to_string()))),
  }
}

fn bearer(key: &str) -> Vec<(&'static str, String)> {
  if key.is_empty() {
    Vec::new()
  } else {
    vec![("Authorization", format!("Bearer {key}"))]
  }
}

fn each_sse(reader: impl Read, mut visit: impl FnMut(&str) -> Result<()>) -> Result<()> {
  let mut reader = BufReader::new(reader);
  let mut line = String::new();
  loop {
    line.clear();
    if reader.read_line(&mut line)? == 0 {
      break;
    }
    let Some(data) = line.trim().strip_prefix("data:") else {
      continue;
    };
    let data = data.trim();
    if data.is_empty() || data == "[DONE]" {
      continue;
    }
    visit(data)?;
  }
  Ok(())
}

pub fn list_models(config: &ProviderConfig) -> Result<Vec<ModelInfo>> {
  match config.kind {
    ProviderKind::OpenAiCompatible => {
      if config.base_url.is_empty() {
        return Ok(Vec::new());
      }
      let url = format!("{}/models", config.base_url.trim_end_matches('/'));
      let reader = send_get(&url, &bearer(&config.api_key()))?;
      let value: Value = serde_json::from_reader(reader)?;
      let Some(entries) = value.get("data").and_then(Value::as_array) else {
        return Ok(Vec::new());
      };
      Ok(
        entries
          .iter()
          .filter_map(|entry| entry.get("id").and_then(Value::as_str))
          .map(|id| ModelInfo {
            id: id.to_string(),
            name: id.to_string(),
            provider: config.id.clone(),
          })
          .collect(),
      )
    }
    ProviderKind::Anthropic => Ok(
      [
        "claude-sonnet-4-5",
        "claude-sonnet-4-0",
        "claude-opus-4-1",
        "claude-opus-4-0",
        "claude-3-7-sonnet-latest",
        "claude-3-5-haiku-latest",
      ]
      .iter()
      .map(|id| ModelInfo {
        id: id.to_string(),
        name: id.to_string(),
        provider: config.id.clone(),
      })
      .collect(),
    ),
    ProviderKind::Gemini => {
      let key = config.api_key();
      if key.is_empty() {
        return Ok(Vec::new());
      }
      let url = format!("{}/models", config.base_url.trim_end_matches('/'));
      let reader = send_get(&url, &[("x-goog-api-key", key)])?;
      let value: Value = serde_json::from_reader(reader)?;
      let Some(entries) = value.get("models").and_then(Value::as_array) else {
        return Ok(Vec::new());
      };
      Ok(
        entries
          .iter()
          .filter_map(|entry| entry.get("name").and_then(Value::as_str))
          .filter(|name| name.contains("generateContent") || !name.contains(':'))
          .map(|name| {
            let id = name.strip_prefix("models/").unwrap_or(name).to_string();
            ModelInfo {
              id: id.clone(),
              name: id,
              provider: config.id.clone(),
            }
          })
          .collect(),
      )
    }
    ProviderKind::Unsupported => Ok(Vec::new()),
  }
}

pub fn stream(
  config: &ProviderConfig,
  model: &str,
  messages: &[Message],
  on_event: &(dyn Fn(StreamEvent) + Send + Sync),
) -> Result<()> {
  match config.kind {
    ProviderKind::OpenAiCompatible => stream_openai(config, model, messages, on_event),
    ProviderKind::Anthropic => stream_anthropic(config, model, messages, on_event),
    ProviderKind::Gemini => stream_gemini(config, model, messages, on_event),
    ProviderKind::Unsupported => Err(Error::Config(format!(
      "{} is not wired yet: {}",
      config.name,
      config.note.as_deref().unwrap_or("unsupported provider")
    ))),
  }
}

fn role_name(role: Role) -> &'static str {
  match role {
    Role::System => "system",
    Role::User => "user",
    Role::Assistant => "assistant",
  }
}

fn stream_openai(
  config: &ProviderConfig,
  model: &str,
  messages: &[Message],
  on_event: &(dyn Fn(StreamEvent) + Send + Sync),
) -> Result<()> {
  let body = json!({
    "model": model,
    "messages": messages.iter().map(|m| json!({"role": role_name(m.role), "content": m.content})).collect::<Vec<_>>(),
    "stream": true,
    "stream_options": {"include_usage": true},
    "temperature": 0.7
  });
  let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
  let reader = send_post(&url, &bearer(&config.api_key()), &body.to_string())?;
  let mut usage = Usage::default();
  each_sse(reader, |data| {
    let value: Value = serde_json::from_str(data)?;
    if let Some(delta) = value
      .get("choices")
      .and_then(Value::as_array)
      .and_then(|choices| choices.first())
      .and_then(|choice| choice.get("delta"))
      .and_then(|delta| delta.get("content"))
      .and_then(Value::as_str)
    {
      if !delta.is_empty() {
        on_event(StreamEvent::Delta(delta.to_string()));
      }
    }
    if let Some(usage_value) = value.get("usage") {
      usage.input = usage_value.get("prompt_tokens").and_then(Value::as_u64).unwrap_or(0);
      usage.output = usage_value.get("completion_tokens").and_then(Value::as_u64).unwrap_or(0);
    }
    Ok(())
  })?;
  on_event(StreamEvent::Done(usage));
  Ok(())
}

fn stream_anthropic(
  config: &ProviderConfig,
  model: &str,
  messages: &[Message],
  on_event: &(dyn Fn(StreamEvent) + Send + Sync),
) -> Result<()> {
  let system = messages
    .iter()
    .filter(|m| m.role == Role::System)
    .map(|m| m.content.as_str())
    .collect::<Vec<_>>()
    .join("\n\n");
  let turns = messages
    .iter()
    .filter(|m| m.role != Role::System)
    .map(|m| {
      let role = if m.role == Role::Assistant { "assistant" } else { "user" };
      json!({"role": role, "content": m.content})
    })
    .collect::<Vec<_>>();
  let mut body = json!({
    "model": model,
    "messages": turns,
    "max_tokens": 4096,
    "stream": true
  });
  if !system.is_empty() {
    body["system"] = json!(system);
  }
  let url = format!("{}/messages", config.base_url.trim_end_matches('/'));
  let headers: Vec<(&str, String)> = vec![
    ("x-api-key", config.api_key()),
    ("anthropic-version", "2023-06-01".to_string()),
  ];
  let reader = send_post(&url, &headers, &body.to_string())?;
  let mut usage = Usage::default();
  each_sse(reader, |data| {
    let value: Value = serde_json::from_str(data)?;
    match value.get("type").and_then(Value::as_str) {
      Some("content_block_delta") => {
        if let Some(text) = value
          .get("delta")
          .and_then(|d| d.get("text"))
          .and_then(Value::as_str)
        {
          if !text.is_empty() {
            on_event(StreamEvent::Delta(text.to_string()));
          }
        }
      }
      Some("message_delta") => {
        if let Some(usage_value) = value.get("usage") {
          usage.output = usage_value.get("output_tokens").and_then(Value::as_u64).unwrap_or(0);
        }
      }
      Some("message_start") => {
        if let Some(usage_value) = value.get("message").and_then(|m| m.get("usage")) {
          usage.input = usage_value.get("input_tokens").and_then(Value::as_u64).unwrap_or(0);
        }
      }
      _ => {}
    }
    Ok(())
  })?;
  on_event(StreamEvent::Done(usage));
  Ok(())
}

fn stream_gemini(
  config: &ProviderConfig,
  model: &str,
  messages: &[Message],
  on_event: &(dyn Fn(StreamEvent) + Send + Sync),
) -> Result<()> {
  let contents = messages
    .iter()
    .filter(|m| m.role != Role::System)
    .map(|m| {
      let role = if m.role == Role::Assistant { "model" } else { "user" };
      json!({"role": role, "parts": [{"text": m.content}]})
    })
    .collect::<Vec<_>>();
  let mut body = json!({"contents": contents});
  if let Some(system) = messages.iter().find(|m| m.role == Role::System) {
    body["systemInstruction"] = json!({"parts": [{"text": system.content}]});
  }
  let model = model.strip_prefix("models/").unwrap_or(model);
  let key = config.api_key();
  let url = format!(
    "{}/models/{}:streamGenerateContent?alt=sse",
    config.base_url.trim_end_matches('/'),
    model
  );
  let reader = send_post(&url, &[("x-goog-api-key", key)], &body.to_string())?;
  let mut usage = Usage::default();
  each_sse(reader, |data| {
    let value: Value = serde_json::from_str(data)?;
    if let Some(parts) = value
      .get("candidates")
      .and_then(Value::as_array)
      .and_then(|c| c.first())
      .and_then(|c| c.get("content"))
      .and_then(|c| c.get("parts"))
      .and_then(Value::as_array)
    {
      for part in parts {
        if let Some(text) = part.get("text").and_then(Value::as_str) {
          if !text.is_empty() {
            on_event(StreamEvent::Delta(text.to_string()));
          }
        }
      }
    }
    if let Some(metadata) = value.get("usageMetadata") {
      usage.input = metadata.get("promptTokenCount").and_then(Value::as_u64).unwrap_or(0);
      usage.output = metadata.get("candidatesTokenCount").and_then(Value::as_u64).unwrap_or(0);
    }
    Ok(())
  })?;
  on_event(StreamEvent::Done(usage));
  Ok(())
}
