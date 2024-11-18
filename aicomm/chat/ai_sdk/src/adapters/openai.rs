use crate::{AiAdapter, AiService, Message};
use anyhow::anyhow;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenaiAdapter {
    host: String,
    api_key: String,
    model: String,
    client: Client,
}

#[derive(Serialize)]
pub struct OpenaiChatCompletionRequest {
    pub model: String,
    pub messages: Vec<OpenaiMessage>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct OpenaiMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct OpenaiChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub system_fingerprint: String,
    pub choices: Vec<OpenaiChoice>,
    pub usage: OpenaiUsage,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct OpenaiChoice {
    pub index: u32,
    pub message: OpenaiMessage,
    pub logprobs: Option<i64>,
    pub finish_reason: String,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct OpenaiUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub completion_tokens_details: Option<OpenaiCompletionTokensDetails>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct OpenaiCompletionTokensDetails {
    pub reasoning_tokens: u32,
}

impl OpenaiAdapter {
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

impl AiService for OpenaiAdapter {
    async fn complete(&self, messages: &[Message]) -> anyhow::Result<String> {
        let req = OpenaiChatCompletionRequest {
            model: self.model.clone(),
            messages: messages.iter().map(OpenaiMessage::from).collect(),
        };
        let url = format!("{}/chat/completions", self.host);
        let res = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req)
            .send()
            .await?;

        let mut response: OpenaiChatCompletionResponse = res.json().await?;
        let content = response.choices.pop().ok_or(anyhow!("No response"))?.message.content;
        Ok(content)
    }
}

impl From<OpenaiAdapter> for AiAdapter {
    fn from(value: OpenaiAdapter) -> Self {
        AiAdapter::Openai(value)
    }
}

impl From<Message> for OpenaiMessage {
    fn from(msg: Message) -> Self {
        Self {
            role: msg.role.to_string(),
            content: msg.content,
        }
    }
}

impl From<&Message> for OpenaiMessage {
    fn from(msg: &Message) -> Self {
        Self {
            role: msg.role.to_string(),
            content: msg.content.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::adapters::OpenaiAdapter;
    use crate::{AiService, Message, Role};
    use anyhow::Result;
    use std::env;

    #[ignore]
    #[tokio::test]
    async fn openai_complete_should_work() -> Result<()> {
        // get the api key from system environment
        let api_key = env::var("OPENAI_API_KEY")?;
        let host = "https://vip.apiyi.com/v1";
        let adapter = OpenaiAdapter::new(host, api_key, "gpt-4o-mini");
        let message = vec![Message {
            role: Role::User,
            content: "hello chat".to_string(),
        }];
        let response = adapter.complete(&message).await?;
        println!("response: {}", response);
        Ok(())
    }
}