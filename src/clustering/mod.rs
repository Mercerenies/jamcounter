
pub mod games;
pub mod text;

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Index, Range};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "ClusterSetDeserializeHelper<T>", bound(deserialize = "T: Clone + Eq + Hash + Deserialize<'de>"))]
pub struct ClusterSet<T> {
  clusters: Vec<Cluster<T>>,
  #[serde(skip)]
  clusters_by_contents: HashMap<T, usize>,
}

#[derive(Deserialize)]
struct ClusterSetDeserializeHelper<T> {
  clusters: Vec<Cluster<T>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Cluster<T> {
  values: Vec<T>,
}

impl<T> ClusterSet<T> {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn is_empty(&self) -> bool {
    self.clusters.is_empty()
  }

  pub fn len(&self) -> usize {
    self.clusters.len()
  }

  pub fn clusters(&self) -> impl Iterator<Item = (usize, &Cluster<T>)> + '_ {
    self.clusters.iter().enumerate()
  }

  pub fn cluster_indices(&self) -> Range<usize> {
    0..self.len()
  }
}

impl<T: Eq + Hash + Clone> ClusterSet<T> {
  pub fn get_cluster_index(&self, value: &T) -> Option<usize> {
    self.clusters_by_contents.get(value).copied()
  }

  pub fn contains(&self, value: &T) -> bool {
    self.get_cluster_index(value).is_some()
  }

  pub fn add_to_cluster(&mut self, cluster_index: usize, value: T) {
    assert!(
      !self.clusters_by_contents.contains_key(&value),
      "Cluster already contains value",
    );

    self.clusters[cluster_index].values.push(value.clone());
    self.clusters_by_contents.insert(value, cluster_index);
  }

  pub fn append_new_cluster(&mut self) -> usize {
    self.clusters.push(Cluster::default());
    self.clusters.len() - 1
  }
}

impl<T> Cluster<T> {
  pub fn is_empty(&self) -> bool {
    self.values.is_empty()
  }

  pub fn len(&self) -> usize {
    self.values.len()
  }

  pub fn iter(&self) -> impl Iterator<Item = &T> {
    self.values.iter()
  }

  pub fn as_slice(&self) -> &[T] {
    self.values.as_slice()
  }
}

impl<T> Default for ClusterSet<T> {
  fn default() -> Self {
    Self {
      clusters: Vec::new(),
      clusters_by_contents: HashMap::new(),
    }
  }
}

impl<T> Default for Cluster<T> {
  fn default() -> Self {
    Self {
      values: Vec::new(),
    }
  }
}

impl<T> Index<usize> for ClusterSet<T> {
  type Output = Cluster<T>;

  fn index(&self, index: usize) -> &Self::Output {
    &self.clusters[index]
  }
}

impl<T: Clone + Eq + Hash> From<ClusterSetDeserializeHelper<T>> for ClusterSet<T> {
  fn from(helper: ClusterSetDeserializeHelper<T>) -> Self {
    let clusters_by_contents = helper.clusters.iter()
      .enumerate()
      .flat_map(|(i, cluster)| cluster.iter().map(move |v| (v.clone(), i)))
      .collect();

    Self {
      clusters: helper.clusters,
      clusters_by_contents,
    }
  }
}

pub fn cluster_data<T, F>(data: Vec<T>, mut cmp: F, threshold: f64) -> ClusterSet<T>
where T: Eq + Hash + Clone,
      F: FnMut(&T, &T) -> f64 {
  let mut compare_to_cluster = |value: &T, cluster: &Cluster<T>| -> f64 {
    cluster.iter()
      .map(|elem| OrderedFloat(cmp(value, elem)))
      .max()
      .map(|v| v.0)
      .unwrap_or(0.0)
  };

  let mut set = ClusterSet::new();
  for elem in data {
    if set.contains(&elem) {
      continue;
    }

    let best = set.clusters()
      .map(|(idx, cluster)| (idx, compare_to_cluster(&elem, cluster)))
      .max_by_key(|(_, v)| OrderedFloat(*v));
    if let Some((best_idx, best_value)) = best && best_value > threshold {
      set.add_to_cluster(best_idx, elem);
    } else {
      let new_cluster_idx = set.append_new_cluster();
      set.add_to_cluster(new_cluster_idx, elem);
    }
  }

  set
}
