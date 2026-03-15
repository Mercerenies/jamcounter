
use crate::ai::LlmClient;
use crate::ai::scanner::{extract_names, VideoGame};
use crate::ai::categories::{extract_categories, extract_author_categories};
use crate::scraping::{ForumPost, read_and_parse_page};
use crate::clustering::{ClusterSet, Cluster, cluster_data, add_to_correct_cluster_set};
use crate::clustering::games::{game_comparison_score, COMPARE_THRESHOLD};
use crate::ranked_choice::{Voter, calculate_adjusted_ranked_choice};

use serde::{Deserialize, Serialize};
use anyhow::anyhow;
use futures::future::try_join_all;
use itertools::Itertools;

use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JamResults {
  #[serde(flatten)]
  pub rankings_data: RankingsResults,
  pub best_ofs: BestOfResults<VideoGame>,
  pub best_of_users: BestOfResults<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingsResults {
  pub posts: Vec<ExtractedPost>,
  pub clusters: ClusterSet<VideoGame>,
  pub final_rankings: Vec<RankedGame>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestOfResults<T> {
  pub posts: Vec<ExtractedBestOfPost<T>>,
  pub winners: HashMap<String, Vec<T>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct ExtractedPost {
  #[serde(alias = "author")]
  pub post_author: String,
  pub ranks: Vec<VideoGame>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtractedBestOfPost<T> {
  #[serde(alias = "author")]
  pub post_author: String,
  pub votes: HashMap<String, T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedGame {
  #[serde(flatten)]
  pub game: VideoGame,
  pub score: f64,
}

#[derive(Debug, Clone)]
pub struct CategoryVoter<T> {
  pub votes: HashMap<String, T>,
}

impl RankedGame {
  pub fn title(&self) -> &str {
    self.game.title.as_str()
  }

  pub fn author(&self) -> &str {
    self.game.author.as_str()
  }
}

pub async fn run_vote_counts_pipeline(llm: &LlmClient, posts: &[ForumPost]) -> anyhow::Result<RankingsResults> {
  let posts = extract_games_from_posts(llm, posts.to_vec()).await?;
  let flattened_games = posts.clone().into_iter().flat_map(|e| e.ranks).collect::<Vec<_>>();
  let cluster_set = cluster_data(flattened_games, game_comparison_score, COMPARE_THRESHOLD);
  let all_entry_ids = cluster_set.cluster_indices().collect::<Vec<_>>();
  let voters = organize_posts_into_voters(&cluster_set, &posts);
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

  Ok(RankingsResults {
    posts,
    clusters: cluster_set,
    final_rankings: ranked_games,
  })
}

pub async fn run_best_ofs_pipeline(llm: &LlmClient, categories: &[String], posts: &[ForumPost], clusters: &mut ClusterSet<VideoGame>) -> anyhow::Result<BestOfResults<VideoGame>> {
  let posts = extract_bestofs_from_posts(llm, categories, posts.to_vec()).await?;

  // Add any games missing from the cluster set to the cluster set.
  for post in &posts {
    for game in post.votes.values() {
      add_to_correct_cluster_set(clusters, game.clone(), game_comparison_score, COMPARE_THRESHOLD);
    }
  }

  let voters = organize_best_of_posts_into_voters(&clusters, &posts);
  let winners: HashMap<String, Vec<VideoGame>> = categories.iter()
    .cloned()
    .map(|category| {
      let vote_counts: HashMap<usize, usize> = voters.iter().flat_map(|voter| voter.votes.get(&category).copied()).counts();
      let winners_for_category = vote_counts.keys().copied().max_set_by_key(|cluster_idx| vote_counts[cluster_idx]);
      let winners_for_category = winners_for_category.into_iter()
        .map(|cluster_idx| choose_representative_entry_for_game(&clusters[cluster_idx]))
        .cloned()
        .collect();

      (category, winners_for_category)
    })
    .collect();

  Ok(BestOfResults { posts, winners })
}

pub async fn run_best_of_users_pipeline(llm: &LlmClient, categories: &[String], posts: &[ForumPost]) -> anyhow::Result<BestOfResults<String>> {
  let posts = extract_bestof_users_from_posts(llm, categories, posts.to_vec()).await?;

  let voters: Vec<_> = posts.iter().map(|post| CategoryVoter { votes: post.votes.clone() }).collect();
  let winners: HashMap<String, Vec<String>> = categories.iter()
    .cloned()
    .map(|category| {
      let vote_counts: HashMap<String, usize> = voters.iter().flat_map(|voter| voter.votes.get(&category).cloned()).counts();
      let winners_for_category = vote_counts.keys().cloned().max_set_by_key(|cluster_idx| vote_counts[cluster_idx]);
      (category, winners_for_category)
    })
    .collect();

  Ok(BestOfResults { posts, winners })
}

pub async fn scrape_posts_from_web(url: &str) -> anyhow::Result<Vec<ForumPost>> {
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

  Ok(posts)
}

async fn extract_games_from_posts(llm: &LlmClient, posts: Vec<ForumPost>) -> anyhow::Result<Vec<ExtractedPost>> {
  try_join_all(
    posts.into_iter()
      .map(|post| async move {
        let names = extract_names(llm, &post.text).await?;
        Ok(ExtractedPost { post_author: post.author, ranks: names })
      }),
  ).await
}

async fn extract_bestofs_from_posts(llm: &LlmClient, categories: &[String], posts: Vec<ForumPost>) -> anyhow::Result<Vec<ExtractedBestOfPost<VideoGame>>> {
  try_join_all(
    posts.into_iter()
      .map(|post| async move {
        let votes = extract_categories(llm, categories, &post.text).await?;
        Ok(ExtractedBestOfPost { post_author: post.author, votes })
      }),
  ).await
}

async fn extract_bestof_users_from_posts(llm: &LlmClient, categories: &[String], posts: Vec<ForumPost>) -> anyhow::Result<Vec<ExtractedBestOfPost<String>>> {
  try_join_all(
    posts.into_iter()
      .map(|post| async move {
        let votes = extract_author_categories(llm, categories, &post.text).await?;
        let votes = votes.into_iter()
          // The AI picks up self as "best reviewer" a weird amount of
          // the time, so clean that up.
          .filter(|(_category, vote)| *vote != post.author)
          .collect();
        Ok(ExtractedBestOfPost { post_author: post.author, votes })
      }),
  ).await
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

fn organize_best_of_posts_into_voters(cluster_set: &ClusterSet<VideoGame>, posts: &[ExtractedBestOfPost<VideoGame>]) -> Vec<CategoryVoter<usize>> {
  let game_to_cluster_id = |game: &VideoGame| cluster_set.get_cluster_index(game).unwrap();

  posts
    .iter()
    .map(|post| CategoryVoter {
      votes: post.votes.iter().map(|(k, v)| (k.clone(), game_to_cluster_id(v))).collect(),
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
