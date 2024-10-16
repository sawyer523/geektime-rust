use crate::{AiService, Message};
use anyhow::anyhow;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenAIAdapter {
    host: String,
    api_key: String,
    model: String,
    client: Client,
}

#[derive(Serialize)]
pub struct OpenAIChatCompletionRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct OpenAIChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub system_fingerprint: String,
    pub choices: Vec<OpenAIChoice>,
    pub usage: OpenAIUsage,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct OpenAIChoice {
    pub index: u32,
    pub message: OpenAIMessage,
    pub logprobs: Option<i64>,
    pub finish_reason: String,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct OpenAIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub completion_tokens_details: Option<OpenAICompletionTokensDetails>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct OpenAICompletionTokensDetails {
    pub reasoning_tokens: u32,
}

impl OpenAIAdapter {
    pub fn deafult(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new("", api_key, model)
    }

    pub fn new(host: impl Into<String>, api_key: impl Into<String>, model: impl Into<String>) -> Self {
        let mut host = host.into();
        if host.is_empty() {
            host = "https://api.openai.com/v1".to_string();
        }
        Self {
            host,
            api_key: api_key.into(),
            model: model.into(),
            client: Client::new(),
        }
    }
}

impl AiService for OpenAIAdapter {
    async fn complete(&self, messages: &[Message]) -> anyhow::Result<String> {
        let req = OpenAIChatCompletionRequest {
            model: self.model.clone(),
            messages: messages.iter().map(OpenAIMessage::from).collect(),
        };
        let url = format!("{}/chat/completions", self.host);
        let res = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req)
            .send()
            .await?;

        let mut response: OpenAIChatCompletionResponse = res.json().await?;
        let content = response.choices.pop().ok_or(anyhow!("No response"))?.message.content;
        Ok(content)
    }
}

impl From<Message> for OpenAIMessage {
    fn from(msg: Message) -> Self {
        Self {
            role: msg.role.to_string(),
            content: msg.content,
        }
    }
}

impl From<&Message> for OpenAIMessage {
    fn from(msg: &Message) -> Self {
        Self {
            role: msg.role.to_string(),
            content: msg.content.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::adapters::OpenAIAdapter;
    use crate::{AiService, Message, Role};
    use anyhow::Result;
    use std::env;

    #[ignore]
    #[tokio::test]
    async fn openai_complete_should_work() -> Result<()> {
        // get the api key from system environment
        let api_key = env::var("OPENAI_API_KEY")?;
        let host = "https://vip.apiyi.com/v1";
        let adapter = OpenAIAdapter::new(host, api_key, "gpt-4o-mini");
        let message = vec![Message {
            role: Role::User,
            content: "hello chat".to_string(),
        }];
        let response = adapter.complete(&message).await?;
        Ok(())
    }
}