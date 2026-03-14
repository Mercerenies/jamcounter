
use super::{LlmClient, Message};

use anyhow::Context;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct VideoGame {
  pub title: String,
  pub author: String,
}

const SYSTEM_MESSAGE: &str = r"
You are a helpful assistant. The user will provide you
with instructions and text. Follow the instructions. Provide
output using the exact format specified. Do not provide
any additional output other than what is requested.
";

pub async fn extract_names(client: &LlmClient, post_content: &str) -> anyhow::Result<Vec<VideoGame>> {
  const BASE_USER_MESSAGE: &str = r#"
    You will be provided with a forum post which consists of plaintext content.
    This post contains a ranked list of video games. Ignore any additional comments, reviews,
    notes, or "Best Of" categories, and focus only on the list of games. You will extract this list,
    providing (in order) the title and author of each game.

    Ignore games which are explicitly marked as "Unjudged" or "Unranked" and do NOT include them
    in your output.

    Provide your output as valid JSON. Do not output ANYTHING other than valid JSON.
    ```
    [
      { "title": ..., "author": ... },
      ...
    ]

    If the post does not list any games, just return "[]".
    ```
  "#;

  let messages = [
    Message::system(SYSTEM_MESSAGE),
    Message::user(BASE_USER_MESSAGE),
    Message::user(post_content),
  ];
  let response = client.chat(&messages).await?.replace("```json", "").replace("```", "");
  let response: Vec<VideoGame> = serde_json::from_str(&response)
    .with_context(|| format!("Response: {response}"))?;
  Ok(response)
}
