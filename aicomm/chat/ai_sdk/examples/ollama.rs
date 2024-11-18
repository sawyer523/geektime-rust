use ai_sdk::{AiService, CloudflareAdapter, Message, OllamaAdapter, Role};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let adapter = OllamaAdapter::default();
    let messages = vec![Message { role: Role::User, content: "世界上最长的河流是什么？".to_string() }];
    let response = adapter.complete(&messages).await.unwrap();
    println!("response: {}", response);
    let api_key = env::var("CF_API_KEY")?;
    let account = env::var("CF_API_ACCOUNT")?;
    let adapter = CloudflareAdapter::new(api_key, account, "llama-3-8b-instruct");
    let messages = vec![Message { role: Role::User, content: "hello".to_string() }];
    let response = adapter.complete(&messages).await.unwrap();
    println!("response: {}", response);
    Ok(())
}