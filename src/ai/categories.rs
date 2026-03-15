
//! Scanners for querying the specific award categories.

use super::{SYSTEM_MESSAGE, LlmClient, Message};
use super::scanner::{VideoGame, remove_author_name_from_title};

use anyhow::Context;

use std::collections::HashMap;

pub async fn extract_categories(client: &LlmClient, categories: &[String], post_content: &str) -> anyhow::Result<HashMap<String, VideoGame>> {
  const BASE_USER_MESSAGE: &str = r#"
    You will be provided with a forum post which consists of plaintext content.
    This post contains votes for a video game contest. You will focus on the "Best Of" categories.
    Ignore the base rankings and any additional comments, reviews, or notes, and focus only on the
    "Best Of" categories. You will extract the games that this user voted for in each category.

    Ignore games which are explicitly marked as "Unjudged" or "Unranked" and do NOT include them
    in your output.

    Provide your output as valid JSON. Do not output ANYTHING other than valid JSON.
    ```
    {
      <category>: { "title": <game name>, "author": <game author> },
      <category>: { "title": <game name>, "author": <game author> },
      ...
    }
    ```

    Do NOT output null for missing data. Simply omit the key from the output entirely in that case.

    Do not include the author's name in the "title" field. Keep in mind that a game might have multiple
    authors. In that case, include all authors in one string, separated by commas (NOT in a JSON sublist).

    If the post does not attempt to award any games, just ignore its contents and return "{}".
  "#;

  let categories_message = format!("The specific categories you are looking for are: {}", categories.join(", "));

  let messages = [
    Message::system(SYSTEM_MESSAGE),
    Message::user(BASE_USER_MESSAGE),
    Message::user(&categories_message),
    Message::user(post_content),
  ];
  let response = client.chat(&messages).await?.replace("```json", "").replace("```", "");
  let response: HashMap<String, VideoGame> = serde_json::from_str(&response)
    .with_context(|| format!("Response: {response}"))?;

  // Do a tiny bit of data cleanup here, based on common responses
  // I've seen from the AI.
  let response: HashMap<String, VideoGame> = response.into_iter()
    .map(|(category, game)| (category, remove_author_name_from_title(game)))
    .collect();

  Ok(response)
}
