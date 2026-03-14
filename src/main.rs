
use jamcounter::config::read_config;
use jamcounter::scraping::{read_and_parse_page};
use jamcounter::ai::LlmClient;
use jamcounter::ai::scanner::extract_names;

use anyhow::anyhow;

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
  for post in posts {
    println!("{}", post.author);
    let names = extract_names(&llm, &post.text).await?;
    dbg!(names);
  }

  Ok(ExitCode::SUCCESS)
}
