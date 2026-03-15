use crate::config::cli::ProviderType;
use crate::error::LumenError;
use dirs::home_dir;
use serde::{Deserialize, Deserializer};
use serde_json::from_reader;
use std::fs::File;
use std::io::BufReader;

use crate::Cli;

#[derive(Debug, Deserialize)]
pub struct LumenConfig {
    #[serde(
        default = "default_ai_provider",
        deserialize_with = "deserialize_ai_provider"
    )]
    pub provider: ProviderType,

    #[serde(default = "default_model")]
    pub model: Option<String>,

    #[serde(default = "default_api_key")]
    pub api_key: Option<String>,

    #[serde(default = "default_base_url")]
    pub base_url: Option<String>,

    #[serde(default)]
    pub theme: Option<String>,
}

fn default_ai_provider() -> ProviderType {
    std::env::var("LUMEN_AI_PROVIDER")
        .unwrap_or_else(|_| "openai".to_string())
        .parse()
        .unwrap_or(ProviderType::Openai)
}

fn deserialize_ai_provider<'de, D>(deserializer: D) -> Result<ProviderType, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

fn default_model() -> Option<String> {
    std::env::var("LUMEN_AI_MODEL").ok()
}

fn default_api_key() -> Option<String> {
    std::env::var("LUMEN_API_KEY").ok()
}

fn default_base_url() -> Option<String> {
    std::env::var("LUMEN_BASE_URL").ok()
}

fn default_config_path() -> Option<String> {
    home_dir().and_then(|mut path| {
        path.push(".config/lumen/lumen.config.json");
        path.exists()
            .then_some(path)
            .and_then(|p| p.to_str().map(|s| s.to_string()))
    })
}

impl LumenConfig {
    pub fn build(cli: &Cli) -> Result<Self, LumenError> {
        let config = if let Some(config_path) = &cli.config {
            LumenConfig::from_file(config_path)?
        } else {
            match default_config_path() {
                Some(path) => LumenConfig::from_file(&path)?,
                None => LumenConfig::default(),
            }
        };

        let provider = cli.provider.as_ref().cloned().unwrap_or(config.provider);
        let api_key = cli.api_key.clone().or(config.api_key);
        let model = cli.model.clone().or(config.model);
        let base_url = cli.base_url.clone().or(config.base_url);

        Ok(LumenConfig {
            provider,
            model,
            api_key,
            base_url,
            theme: config.theme,
        })
    }

    pub fn from_file(file_path: &str) -> Result<Self, LumenError> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        // Deserialize JSON data into the LumenConfig struct
        let config: LumenConfig = match from_reader(reader) {
            Ok(config) => config,
            Err(e) => return Err(LumenError::InvalidConfiguration(e.to_string())),
        };

        Ok(config)
    }
}

impl Default for LumenConfig {
    fn default() -> Self {
        LumenConfig {
            provider: default_ai_provider(),
            model: default_model(),
            api_key: default_api_key(),
            base_url: default_base_url(),
            theme: None,
        }
    }
}
