
use super::{SYSTEM_MESSAGE, LlmClient, Message};

use anyhow::Context;
use serde::{Serialize, Deserialize};

use std::fmt::{self, Display};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct VideoGame {
  pub title: String,
  pub author: String,
}

impl Display for VideoGame {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} by {}", self.title, self.author)
  }
}

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
    ```

    Do not include the author's name in the "title" field. Keep in mind that a game might have multiple
    authors. In that case, include all authors in one string, separated by commas (NOT in a JSON sublist).

    If the post does not attempt to rank any games, just ignore its contents and return "[]".
  "#;

  let messages = [
    Message::system(SYSTEM_MESSAGE),
    Message::user(BASE_USER_MESSAGE),
    Message::user(post_content),
  ];
  let response = client.chat(&messages).await?.replace("```json", "").replace("```", "");
  let response: Vec<VideoGame> = serde_json::from_str(&response)
    .with_context(|| format!("Response: {response}"))?;

  // Do a tiny bit of data cleanup here, based on common responses
  // I've seen from the AI.
  let response: Vec<VideoGame> = response.into_iter()
    .map(remove_author_name_from_title)
    .collect();

  Ok(response)
}

// Cleanup function for AI
pub(super) fn remove_author_name_from_title(mut game: VideoGame) -> VideoGame {
  if game.title.ends_with(format!(" by {}", game.author).as_str()) {
    game.title.truncate(game.title.len() - game.author.len() - 4);
  }
  game
}
