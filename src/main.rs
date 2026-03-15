
use jamcounter::config::read_config;
use jamcounter::ai::LlmClient;
use jamcounter::runner::{scrape_posts_from_web, run_vote_counts_pipeline};

use std::fs::{self, File};
use std::process::ExitCode;

#[tokio::main]
async fn main() -> anyhow::Result<ExitCode> {
  let config = read_config()?;
  let llm = LlmClient::from_config(&config);

  println!("Using {} model {}", config.openai_url, config.llm_model);

  if fs::exists(&config.output_path)? {
    anyhow::bail!("Output file already exists: {}", config.output_path);
  }
  let url = &config.voting_post_url;

  let posts = scrape_posts_from_web(&url).await?;
  let result = run_vote_counts_pipeline(&llm, &posts).await?;

  println!("Jam Results");
  for (i, game) in result.final_rankings.iter().enumerate() {
    println!("{}. {} by {} ({})", i + 1, game.title(), game.author(), game.score);
  }

  println!("Writing full results to {}", config.output_path);
  serde_json::to_writer_pretty(File::create(&config.output_path)?, &result)?;

  Ok(ExitCode::SUCCESS)
}
