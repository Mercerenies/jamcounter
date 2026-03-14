
use crate::ai::LlmClient;
use crate::ai::scanner::{extract_names, VideoGame};
use crate::scraping::{ForumPost, read_and_parse_page};
use crate::clustering::{ClusterSet, Cluster, cluster_data};
use crate::clustering::games::{game_comparison_score, COMPARE_THRESHOLD};
use crate::ranked_choice::{Voter, calculate_adjusted_ranked_choice};

use serde::{Deserialize, Serialize};
use anyhow::anyhow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JamResults {
  pub posts: Vec<ExtractedPost>,
  pub final_rankings: Vec<RankedGame>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct ExtractedPost {
  #[serde(alias = "author")]
  pub post_author: String,
  pub ranks: Vec<VideoGame>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedGame {
  #[serde(flatten)]
  pub game: VideoGame,
  pub score: f64,
}

impl RankedGame {
  pub fn title(&self) -> &str {
    self.game.title.as_str()
  }

  pub fn author(&self) -> &str {
    self.game.author.as_str()
  }
}

pub async fn run_vote_counts_pipeline(llm: &LlmClient, vote_page_url: &str) -> anyhow::Result<JamResults> {
  let games = read_and_extract_from_posts(&llm, vote_page_url).await?;
  let flattened_games = games.clone().into_iter().flat_map(|e| e.ranks).collect::<Vec<_>>();
  let cluster_set = cluster_data(flattened_games, game_comparison_score, COMPARE_THRESHOLD);
  let all_entry_ids = cluster_set.cluster_indices().collect::<Vec<_>>();
  let voters = organize_posts_into_voters(&cluster_set, &games);
  let ranked_choice = calculate_adjusted_ranked_choice(&all_entry_ids, &voters);

  let mut ranked_games: Vec<_> = cluster_set.clusters()
    .map(|(idx, cluster)| {
      RankedGame {
        game: choose_representative_entry_for_game(cluster).clone(),
        score: ranked_choice[&idx],
      }
    })
    .collect();
  ranked_games.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

  Ok(JamResults {
    posts: games,
    final_rankings: ranked_games,
  })
}

async fn read_and_extract_from_posts(llm: &LlmClient, url: &str) -> anyhow::Result<Vec<ExtractedPost>> {
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
  Ok(games)
}

async fn extract_games_from_posts(llm: &LlmClient, posts: Vec<ForumPost>) -> anyhow::Result<Vec<ExtractedPost>> {
  let mut extracted = Vec::with_capacity(posts.len());
  for post in posts {
    let names = extract_names(llm, &post.text).await?;
    extracted.push(ExtractedPost { post_author: post.author, ranks: names });
  }
  Ok(extracted)
}

fn organize_posts_into_voters(cluster_set: &ClusterSet<VideoGame>, posts: &[ExtractedPost]) -> Vec<Voter<usize>> {
  let game_to_cluster_id = |game: &VideoGame| cluster_set.get_cluster_index(game).unwrap();

  posts
    .iter()
    .map(|post| Voter {
      rankings: post.ranks.iter().map(game_to_cluster_id).collect(),
    })
    .collect()
}

fn choose_representative_entry_for_game(cluster: &Cluster<VideoGame>) -> &VideoGame {
  // Pick the one with the longest title (assume some people might
  // have abbreviated it in their post)
  cluster.as_slice()
    .iter()
    .max_by_key(|game| game.title.len())
    .unwrap()
}
