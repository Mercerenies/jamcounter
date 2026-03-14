
use figment::Figment;
use figment::providers::Env;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
  #[serde(default)]
  pub openai_api_key: String,
  #[serde(default = "default_openai_url")]
  pub openai_url: String,
  #[serde(default = "default_model")]
  pub model: String,
}

fn default_openai_url() -> String {
  "https://api.openai.com/v1".into()
}

fn default_model() -> String {
  "gpt-3.5-turbo".into()
}

pub fn read_config() -> figment::Result<Config> {
  Figment::new()
    .merge(Env::prefixed("JAM_"))
    .extract::<Config>()
}
