use crate::{AiAdapter, AiService, Message};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct CloudflareAdapter {
    pub api_key: String,
    pub account: String,
    pub model: String,
    pub client: Client,
}

#[derive(Serialize)]
pub struct CloudflareChatCompletionRequest {
    pub messages: Vec<CloudflareMessage>,
}
#[derive(Serialize, Deserialize)]
pub struct CloudflareMessage {
    pub role: String,
    pub content: String,
}
#[derive(Deserialize)]
pub struct CloudflareChatCompletionResponse {
    pub result: CloudflareChatCompletionResult,
    pub success: bool,
    pub errors: Vec<String>,
    pub messages: Vec<String>,
}

#[derive(Deserialize)]
pub struct CloudflareChatCompletionResult {
    pub response: String,
}

impl CloudflareAdapter {
    pub fn new(api_key: impl Into<String>, account: impl Into<String>, model: impl Into<String>) -> Self {
        let api_key = api_key.into();
        let account = account.into();
        let model = model.into();
        let client = Client::new();
        Self { api_key, account, model, client }
    }
}

impl AiService for CloudflareAdapter {
    async fn complete(&self, messages: &[Message]) -> anyhow::Result<String> {
        let request = CloudflareChatCompletionRequest {
            messages: messages.iter().map(|m| m.into()).collect(),
        };

        let url = format!("https://api.cloudflare.com/client/v4/accounts/{}/ai/run/@cf/meta/{}", self.account, self.model);
        let response = self.client.post(url)
            .header("Authorization", format!("Bearer {}", self.api_key.clone()))
            .json(&request)
            .send()
            .await?;

        let response: CloudflareChatCompletionResponse = response.json()
            .await?;
        Ok(response.result.response)
    }
}

impl From<CloudflareAdapter> for AiAdapter {
    fn from(value: CloudflareAdapter) -> Self {
        AiAdapter::Cloudflare(value)
    }
}

impl From<Message> for CloudflareMessage {
    fn from(message: Message) -> Self {
        CloudflareMessage { role: message.role.to_string(), content: message.content }
    }
}
impl From<&Message> for CloudflareMessage {
    fn from(message: &Message) -> Self {
        CloudflareMessage { role: message.role.to_string(), content: message.content.clone() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Role;
    use std::env;

    #[tokio::test]
    async fn test_complete() -> anyhow::Result<()> {
        let api_key = env::var("CF_API_KEY")?;
        let account = env::var("CF_API_ACCOUNT")?;
        let adapter = CloudflareAdapter::new(api_key, account, "llama-3-8b-instruct");
        let messages = vec![Message { role: Role::User, content: "hello".to_string() }];
        let response = adapter.complete(&messages).await.unwrap();
        println!("response: {}", response);
        Ok(())
    }
}