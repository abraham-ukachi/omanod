use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderKind {
  OpenAiCompatible,
  Anthropic,
  Gemini,
  Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
  pub id: String,
  pub name: String,
  pub kind: ProviderKind,
  pub base_url: String,
  pub api_key_env: Option<String>,
  pub note: Option<String>,
}

impl ProviderConfig {
  pub fn api_key(&self) -> String {
    self
      .api_key_env
      .as_deref()
      .and_then(|name| name.var_ok())
      .unwrap_or_default()
  }
}

trait VarOk {
  fn var_ok(self) -> Option<String>;
}

impl VarOk for &str {
  fn var_ok(self) -> Option<String> {
    std::env::var(self).ok().filter(|v| !v.trim().is_empty())
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
  pub id: String,
  pub name: String,
  pub provider: String,
}

fn p(
  id: &str,
  name: &str,
  kind: ProviderKind,
  base_url: &str,
  api_key_env: Option<&str>,
  note: Option<&str>,
) -> ProviderConfig {
  ProviderConfig {
    id: id.to_string(),
    name: name.to_string(),
    kind,
    base_url: base_url.to_string(),
    api_key_env: api_key_env.map(str::to_string),
    note: note.map(str::to_string),
  }
}

pub fn catalog() -> Vec<ProviderConfig> {
  let mut list = vec![
    p("ollama", "Ollama (local)", ProviderKind::OpenAiCompatible, "http://127.0.0.1:11434/v1", None, None),
    p("xai", "xAI Grok", ProviderKind::OpenAiCompatible, "https://api.x.ai/v1", Some("XAI_API_KEY"), None),
    p("openai", "OpenAI", ProviderKind::OpenAiCompatible, "https://api.openai.com/v1", Some("OPENAI_API_KEY"), None),
    p("anthropic", "Anthropic", ProviderKind::Anthropic, "https://api.anthropic.com/v1", Some("ANTHROPIC_API_KEY"), None),
    p("google", "Google Gemini", ProviderKind::Gemini, "https://generativelanguage.googleapis.com/v1beta", Some("GEMINI_API_KEY"), None),
    p("openrouter", "OpenRouter", ProviderKind::OpenAiCompatible, "https://openrouter.ai/api/v1", Some("OPENROUTER_API_KEY"), None),
    p("groq", "Groq", ProviderKind::OpenAiCompatible, "https://api.groq.com/openai/v1", Some("GROQ_API_KEY"), None),
    p("deepseek", "DeepSeek", ProviderKind::OpenAiCompatible, "https://api.deepseek.com/v1", Some("DEEPSEEK_API_KEY"), None),
    p("mistral", "Mistral", ProviderKind::OpenAiCompatible, "https://api.mistral.ai/v1", Some("MISTRAL_API_KEY"), None),
    p("together", "Together", ProviderKind::OpenAiCompatible, "https://api.together.xyz/v1", Some("TOGETHER_API_KEY"), None),
    p("fireworks", "Fireworks", ProviderKind::OpenAiCompatible, "https://api.fireworks.ai/inference/v1", Some("FIREWORKS_API_KEY"), None),
    p("cerebras", "Cerebras", ProviderKind::OpenAiCompatible, "https://api.cerebras.ai/v1", Some("CEREBRAS_API_KEY"), None),
    p("baseten", "Baseten", ProviderKind::OpenAiCompatible, "https://bridge.baseten.co/v1/direct", Some("BASETEN_API_KEY"), None),
    p("moonshotai", "Moonshot", ProviderKind::OpenAiCompatible, "https://api.moonshot.ai/v1", Some("MOONSHOT_API_KEY"), None),
    p("moonshotai-cn", "Moonshot CN", ProviderKind::OpenAiCompatible, "https://api.moonshot.cn/v1", Some("MOONSHOT_API_KEY"), None),
    p("minimax", "MiniMax", ProviderKind::OpenAiCompatible, "https://api.minimax.io/v1", Some("MINIMAX_API_KEY"), None),
    p("minimax-cn", "MiniMax CN", ProviderKind::OpenAiCompatible, "https://api.minimaxi.com/v1", Some("MINIMAX_API_KEY"), None),
    p("nvidia", "NVIDIA NIM", ProviderKind::OpenAiCompatible, "https://integrate.api.nvidia.com/v1", Some("NVIDIA_API_KEY"), None),
    p("huggingface", "Hugging Face", ProviderKind::OpenAiCompatible, "https://router.huggingface.co/v1", Some("HF_TOKEN"), None),
    p("zai", "Z.ai", ProviderKind::OpenAiCompatible, "https://open.bigmodel.cn/api/paas/v4", Some("ZAI_API_KEY"), None),
    p("zai-coding-cn", "Z.ai Coding CN", ProviderKind::OpenAiCompatible, "https://open.bigmodel.cn/api/coding/paas/v4", Some("ZAI_API_KEY"), None),
    p("vercel-ai-gateway", "Vercel AI Gateway", ProviderKind::OpenAiCompatible, "https://ai-gateway.vercel.sh/v1", Some("AI_GATEWAY_API_KEY"), None),
    p("cloudflare-ai-gateway", "Cloudflare AI Gateway", ProviderKind::Unsupported, "", Some("CLOUDFLARE_API_KEY"), Some("needs an account gateway path in ~/.config/omanod/config.toml")),
    p("cloudflare-workers-ai", "Cloudflare Workers AI", ProviderKind::Unsupported, "", Some("CLOUDFLARE_API_KEY"), Some("needs an account id in ~/.config/omanod/config.toml")),
    p("azure", "Azure OpenAI", ProviderKind::Unsupported, "", Some("AZURE_OPENAI_API_KEY"), Some("needs AZURE_OPENAI_ENDPOINT in the environment")),
    p("amazon-bedrock", "Amazon Bedrock", ProviderKind::Unsupported, "", None, Some("AWS credentials are not wired yet")),
    p("google-vertex", "Google Vertex AI", ProviderKind::Unsupported, "", None, Some("Google Application Credentials are not wired yet")),
    p("ant-ling", "Ant Ling", ProviderKind::Unsupported, "", None, Some("provider needs its own endpoint")),
    p("kimi-coding", "Kimi Coding", ProviderKind::Unsupported, "", Some("KIMI_API_KEY"), Some("provider needs its own endpoint")),
    p("meta", "Meta", ProviderKind::Unsupported, "", None, Some("provider needs its own endpoint")),
    p("opencode", "OpenCode", ProviderKind::Unsupported, "", Some("OPENCODE_API_KEY"), Some("subscription login is not wired yet")),
    p("opencode-go", "OpenCode Go", ProviderKind::Unsupported, "", Some("OPENCODE_API_KEY"), Some("subscription login is not wired yet")),
    p("qwen-token-plan", "Qwen Token Plan", ProviderKind::Unsupported, "", None, Some("subscription login is not wired yet")),
    p("qwen-token-plan-cn", "Qwen Token Plan CN", ProviderKind::Unsupported, "", None, Some("subscription login is not wired yet")),
    p("qwen-token-plan-individual", "Qwen Individual", ProviderKind::Unsupported, "", None, Some("subscription login is not wired yet")),
    p("radius", "Radius", ProviderKind::Unsupported, "", Some("RADIUS_API_KEY"), Some("provider needs its own endpoint")),
    p("xiaomi", "Xiaomi", ProviderKind::Unsupported, "", Some("XIAOMI_API_KEY"), Some("provider needs its own endpoint")),
    p("xiaomi-token-plan-ams", "Xiaomi AMS", ProviderKind::Unsupported, "", None, Some("subscription login is not wired yet")),
    p("xiaomi-token-plan-cn", "Xiaomi CN", ProviderKind::Unsupported, "", None, Some("subscription login is not wired yet")),
    p("xiaomi-token-plan-sgp", "Xiaomi SGP", ProviderKind::Unsupported, "", None, Some("subscription login is not wired yet")),
    p("compatible", "OpenAI-compatible", ProviderKind::OpenAiCompatible, "", Some("OMANOD_COMPATIBLE_API_KEY"), Some("set base URL in ~/.config/omanod/config.toml")),
  ];
  list.sort_by(|a, b| a.name.cmp(&b.name));
  list
}

pub fn provider(id: &str) -> Option<ProviderConfig> {
  catalog().into_iter().find(|p| p.id == id)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn covers_local_grok_and_nod_providers() {
    let catalog = catalog();
    let ids: Vec<&str> = catalog.iter().map(|p| p.id.as_str()).collect();
    for id in ["ollama", "xai", "openai", "anthropic", "google", "openrouter", "minimax", "compatible"] {
      assert!(ids.contains(&id), "{id} is missing from the catalog");
    }
  }
}
