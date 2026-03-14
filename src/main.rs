
use jamcounter::config::read_config;
use jamcounter::ai::LlmClient;
use jamcounter::runner::run_vote_counts_pipeline;

use std::env;
use std::fs::File;
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
  let result = run_vote_counts_pipeline(&llm, url).await?;

  println!("Jam Results");
  for (i, game) in result.final_rankings.iter().enumerate() {
    println!("{}. {} by {} ({})", i + 1, game.title(), game.author(), game.score);
  }

  println!("Writing full results to results.json");
  serde_json::to_writer_pretty(File::create("results.json")?, &result)?;

  Ok(ExitCode::SUCCESS)
}
