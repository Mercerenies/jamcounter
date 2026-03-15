
use jamcounter::config::read_config;
use jamcounter::ai::LlmClient;
use jamcounter::runner::{scrape_posts_from_web, run_vote_counts_pipeline, run_best_ofs_pipeline, JamResults};

use itertools::Itertools;

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
  let mut ranked_result = run_vote_counts_pipeline(&llm, &posts).await?;
  let best_of_result = run_best_ofs_pipeline(&llm, &config.award_categories, &posts, &mut ranked_result.clusters).await?;

  let compiled_results = JamResults {
    rankings_data: ranked_result,
    best_ofs: best_of_result,
  };

  println!("Jam Results:");
  for (i, ranked_game) in compiled_results.rankings_data.final_rankings.iter().enumerate() {
    println!("{}. {} ({})", i + 1, ranked_game.game, ranked_game.score);
  }

  println!("Best ofs:");
  for (category, winners) in &compiled_results.best_ofs.winners {
    println!("{}: {}", category, winners.iter().join("; "));
  }

  println!("Writing full results to {}", config.output_path);
  serde_json::to_writer_pretty(File::create(&config.output_path)?, &compiled_results)?;

  Ok(ExitCode::SUCCESS)
}
