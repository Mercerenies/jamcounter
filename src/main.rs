
use jamcounter::config::read_config;
use jamcounter::scraping::{ForumPost, read_and_parse_page};
use jamcounter::ai::LlmClient;
use jamcounter::ai::scanner::{extract_names, VideoGame};

use anyhow::anyhow;

use std::env;
use std::process::ExitCode;

#[derive(Debug, Clone)]
pub struct ExtractedPost {
  pub author: String,
  pub ranks: Vec<VideoGame>,
}

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
  for post in &mut posts {
    if let Some(idx) = post.text.find("Comments:") {
      post.text = post.text[..idx].to_string();
    }
    post.text = post.text.chars().take(6_000).collect();
  }

  let games = extract_games_from_posts(&llm, posts).await?;
  dbg!(&games);

  Ok(ExitCode::SUCCESS)
}

async fn extract_games_from_posts(llm: &LlmClient, posts: Vec<ForumPost>) -> anyhow::Result<Vec<ExtractedPost>> {
  let mut extracted = Vec::with_capacity(posts.len());
  for post in posts {
    let names = extract_names(llm, &post.text).await?;
    extracted.push(ExtractedPost { author: post.author, ranks: names });
  }
  Ok(extracted)
}
