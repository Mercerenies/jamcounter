
use super::text::token_containment_score;
use crate::ai::scanner::VideoGame;

pub const COMPARE_THRESHOLD: f64 = 0.6;

pub fn game_comparison_score(a: &VideoGame, b: &VideoGame) -> f64 {
  let title_score = token_containment_score(&a.title, &b.title);
  let author_score = token_containment_score(&a.author, &b.author);
  title_score * 0.7 + author_score * 0.3
}
