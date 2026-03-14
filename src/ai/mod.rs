
pub mod classifier;
pub mod message;

pub use message::{Message, Role};

use crate::config::Config;

use openai::{Credentials, OpenAiError};
use openai::chat::{ChatCompletion, ChatCompletionMessage};

#[derive(Debug)]
pub struct LlmClient {
  credentials: Credentials,
  model: String,
}

impl LlmClient {
  pub fn new(credentials: Credentials, model: String) -> LlmClient {
    LlmClient { credentials, model }
  }

  pub fn from_config(config: &Config) -> LlmClient {
    LlmClient::new(
      Credentials::new(&config.openai_api_key, &config.openai_url),
      config.model.clone(),
    )
  }

  pub async fn chat(&self, messages: &[Message]) -> Result<String, OpenAiError> {
    let messages: Vec<_> = messages.iter().map(|msg| ChatCompletionMessage::from(msg.clone())).collect();
    let completion = ChatCompletion::builder(&self.model, messages)
      .credentials(self.credentials.clone())
      .create()
      .await?;
    let returned_message = completion.choices.first().unwrap().message.content.as_ref().unwrap();
    Ok(returned_message.trim().to_owned())
  }
}
