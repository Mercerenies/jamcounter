
use jamcounter::config::read_config;
use jamcounter::ai::LlmClient;
use jamcounter::runner::run_vote_counts_pipeline;

use std::env;
use std::fs::{self, File};
use std::process::ExitCode;

#[tokio::main]
async fn main() -> anyhow::Result<ExitCode> {
  let config = read_config()?;
  let llm = LlmClient::from_config(&config);

  if fs::exists(&config.output_path)? {
    anyhow::bail!("Output file already exists: {}", config.output_path);
  }

  let args = env::args().collect::<Vec<_>>();
  if args.len() < 2 {
    eprintln!("Usage: jamscraper <url>");
    return Ok(ExitCode::FAILURE);
  }
  let url = &args[1];
  let result = run_vote_counts_pipeline(&llm, url).await?;

  println!("Jam Results");
  for (i, game) in result.final_rankings.iter().enumerate() {
    println!("{}. {} by {} ({})", i + 1, game.title(), game.author(), game.score);
  }

  println!("Writing full results to {}", config.output_path);
  serde_json::to_writer_pretty(File::create(&config.output_path)?, &result)?;

  Ok(ExitCode::SUCCESS)
}
