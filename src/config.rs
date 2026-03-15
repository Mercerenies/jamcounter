
use figment::Figment;
use figment::providers::{Format, Env, Toml};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
  #[serde(default = "default_output_path")]
  pub output_path: String,
  #[serde(default)]
  pub openai_api_key: String,
  #[serde(default = "default_openai_url")]
  pub openai_url: String,
  #[serde(default = "default_model")]
  pub llm_model: String,
  pub voting_post_url: String,
  pub award_categories: Vec<String>,
  pub author_award_categories: Vec<String>,
}

fn default_output_path() -> String {
  "results.json".into()
}

fn default_openai_url() -> String {
  "https://api.openai.com/v1".into()
}

fn default_model() -> String {
  "gpt-4o-mini".into()
}

pub fn read_config() -> figment::Result<Config> {
  Figment::new()
    .merge(Toml::file("config.toml"))
    .merge(Env::prefixed("JAM_"))
    .extract::<Config>()
}
