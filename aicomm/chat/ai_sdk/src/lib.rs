use std::fmt;

mod adapters;

pub use adapters::*;

pub enum AiAdapter {
    Openai(OpenaiAdapter),
    Ollama(OllamaAdapter),
    Cloudflare(CloudflareAdapter),
}

#[derive(Debug, Clone)]
pub enum Role {
    User,
    Assistant,
    System,
}
#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[allow(async_fn_in_trait)]
pub trait AiService {
    async fn complete(&self, message: &[Message]) -> anyhow::Result<String>;
}

impl AiService for AiAdapter {
    async fn complete(&self, message: &[Message]) -> anyhow::Result<String> {
        match self {
            AiAdapter::Openai(adapter) => adapter.complete(message).await,
            AiAdapter::Ollama(adapter) => adapter.complete(message).await,
            AiAdapter::Cloudflare(adapter) => adapter.complete(message).await,
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
            Role::System => write!(f, "system"),
        }
    }
}

impl Message {
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }
    pub fn user(content: impl Into<String>) -> Self {
        Self::new(Role::User, content)
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(Role::Assistant, content)
    }
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(Role::System, content)
    }
}