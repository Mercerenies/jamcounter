
use super::{LlmClient, Message};

use openai::OpenAiError;

const SYSTEM_MESSAGE: &str = r"
You are a helpful assistant. The user will provide you
with instructions and text. Follow the instructions. Provide
output using the exact format specified. Do not provide
any additional output other than what is requested.
";

pub async fn ask_if_voting_post(client: &LlmClient, post_content: &str) -> Result<bool, OpenAiError> {
  const BASE_USER_MESSAGE: &str = r#"
    You will be provided with a forum post which consists of BBCode-styled content.
    This post originates from a thread where users are ranking video games in a list.
    Some of these posts contain a ranked list of games. Others are administrative or serve
    other purposes. If the post you are given is a ranked list of games, respond "Yes". If
    the post serves any other purpose, respond "No".
  "#;

  let messages = [
    Message::system(SYSTEM_MESSAGE),
    Message::user(BASE_USER_MESSAGE),
    Message::user(post_content),
  ];
  let response = client.chat(&messages).await?.to_ascii_lowercase();
  Ok(response.contains("yes"))
}
