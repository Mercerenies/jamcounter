
use jamcounter::config::read_config;
use jamcounter::scraping::{read_and_parse_page, ForumPost};
use jamcounter::ai::LlmClient;
use jamcounter::ai::classifier::ask_if_voting_post;

use anyhow::anyhow;
use openai::OpenAiError;

use std::env;
use std::process::ExitCode;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<ExitCode> {
  let config = read_config()?;
  let llm = LlmClient::from_config(&config);

  let args = env::args().collect::<Vec<_>>();
  if args.len() < 2 {
    eprintln!("Usage: jamscraper <url>");
    return Ok(ExitCode::FAILURE);
  }
  let url = &args[1];
  let posts = read_and_parse_page(url).await?;
  let mut posts = posts.into_iter()
    .map(|opt_post| opt_post.ok_or_else(|| anyhow!("Failed to parse post")))
    .collect::<Result<Vec<_>, _>>()?;
  posts.remove(0); // Remove the instructions post
  let posts = filter_to_relevant_posts(&llm, &posts).await?;

  Ok(ExitCode::SUCCESS)
}

async fn filter_to_relevant_posts(client: &LlmClient, posts: &[ForumPost]) -> Result<Vec<ForumPost>, OpenAiError> {
  let mut new_posts = Vec::with_capacity(posts.len());
  for post in posts {
    println!("{}", &post.author);
    if ask_if_voting_post(client, &post.text).await? {
      println!("Yes");
      new_posts.push(post.clone());
    } else {
      println!("No");
    }
  }
  Ok(new_posts)
}
