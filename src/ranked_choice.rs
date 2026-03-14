
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct Voter<T> {
  pub rankings: Vec<T>
}

pub fn calculate_ranked_choice<'a, 'b: 'a, T: Hash + Eq>(entries: &'a [T], voters: &'b [Voter<T>]) -> HashMap<&'a T, f64> {
  let mut scores: HashMap<&'a T, f64> = entries.iter().map(|e| (e, 0.0)).collect();

  for voter in voters {
    for (i, entry) in voter.rankings.iter().enumerate() {
      let score = scores.get_mut(&entry).unwrap();
      *score += 1.0 / (i as f64 + 2.0);
    }
  }

  scores
}

pub fn calculate_adjusted_ranked_choice<'a, 'b: 'a, T: Hash + Eq>(
  entries: &'a [T],
  voters: &'b [Voter<T>],
) -> HashMap<&'a T, f64> {
  let base_scores = calculate_ranked_choice(entries, voters);
  let presence_count = count_presence_in_voter_lists(voters);
  let total_voters_count = voters.len() as f64;

  base_scores.into_iter()
    .map(|(k, base_score)| (k, base_score * total_voters_count / (*presence_count.get(k).unwrap_or(&0)) as f64))
    .collect()
}

fn count_presence_in_voter_lists<T: Hash + Eq>(voters: &[Voter<T>]) -> HashMap<&T, usize> {
  let mut presence_count = HashMap::new();
  for voter in voters {
    for entry in &voter.rankings {
      *presence_count.entry(entry).or_insert(0) += 1;
    }
  }
  presence_count
}
