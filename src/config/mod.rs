use crate::llm::PromptModelVendor;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod storage;
mod vendor;

/// Update or create configuration for a specific model
pub fn handler(vendor: &PromptModelVendor, api_key: &str, model: &str) -> Result<()> {
    let mut config = GlobalConfig::load().unwrap_or_else(|| create_default_config());

    let model_config = ModelConfig {
        api_key: Some(api_key.to_string()),
        model: model.to_string(),
        base_url: get_default_base_url(vendor),
    };

    match vendor {
        PromptModelVendor::DeepSeek => config.deepseek = Some(model_config),
        PromptModelVendor::OpenAI => config.openai = Some(model_config),
        PromptModelVendor::Ollama => config.ollama = Some(model_config),
    }

    config.save()?;
    println!("Config saved.");
    Ok(())
}

/// Returns the default base URL for a given model vendor
fn get_default_base_url(vendor: &PromptModelVendor) -> String {
    match vendor {
        PromptModelVendor::OpenAI => "https://api.openai.com/v1".to_string(),
        PromptModelVendor::DeepSeek => "https://api.deepseek.com/v1".to_string(),
        PromptModelVendor::Ollama => "http://localhost:11434".to_string(),
    }
}

/// Creates a default configuration with predefined settings
fn create_default_config() -> GlobalConfig {
    GlobalConfig {
        default: DefaultConfig {
            default_vendor: PromptModelVendor::DeepSeek,
            timeout: 30,
        },
        vendors: HashMap::new(),
        openai: None,
        deepseek: None,
        ollama: None,
        model_parameters: Some(ModelParameters {
            temperature: 0.1,
            top_p: 0.75,
            top_k: 5,
            max_tokens: 1024,
        }),
    }
}

/// Retrieves the global configuration from storage
///
/// # Returns
/// * `Ok(GlobalConfig)` if configuration was found and loaded successfully
/// * `Err` if configuration was not found or could not be loaded
pub fn get_config() -> Result<GlobalConfig> {
    GlobalConfig::load().ok_or_else(|| anyhow!("Config not found."))
}

/// Global configuration structure for GitBuddy
#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Default configuration settings
    pub default: DefaultConfig,

    #[serde(rename = "vendor", default = "HashMap::new")]
    pub vendors: HashMap<String, ModelConfig>,

    /// OpenAI model configuration
    pub openai: Option<ModelConfig>,
    /// DeepSeek model configuration
    pub deepseek: Option<ModelConfig>,
    /// Ollama model configuration
    pub ollama: Option<ModelConfig>,
    /// Model parameters for inference
    pub model_parameters: Option<ModelParameters>,
}

impl GlobalConfig {
    /// Creates a new configuration with default values
    pub fn new() -> Self {
        create_default_config()
    }

    /// Saves the current configuration to storage
    ///
    /// # Returns
    /// * `Ok(())` if save was successful
    /// * `Err` if save failed
    pub fn save(&self) -> Result<()> {
        let content = toml::to_string(self)?;
        storage::save_config(&content)?;
        Ok(())
    }

    /// Loads configuration from storage
    ///
    /// # Returns
    /// * `Some(GlobalConfig)` if load was successful
    /// * `None` if load failed or config was not found
    pub fn load() -> Option<Self> {
        let content = storage::read_config().unwrap_or_default();
        match toml::from_str(content.as_str()) {
            Ok(config) => Some(config),
            Err(err) => {
                eprintln!("Load config error: {}", err);
                None
            }
        }
    }

    /// Gets the model configuration for a specified vendor
    pub fn load_model(&self, vendor: Option<String>) -> Option<&ModelConfig> {
        match vendor {
            Some(v) => self.vendors.get(&v),
            None => self.vendors.get(&self.default.default_vendor),
        }
    }

    pub fn model_params(&self) -> ModelParameters {
        match &self.model_parameters {
            Some(mp) => mp.clone(),
            None => ModelParameters {
                max_tokens: 1024,
                temperature: 0.0,
                top_p: 0.75,
                top_k: 10,
            },
        }
    }
}

/// Model-specific configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelConfig {
    /// API key for the model vendor
    pub api_key: Option<String>,
    /// Model identifier/name
    pub model: String,
    /// Base URL for API requests
    pub base_url: String,
}

impl ModelConfig {
    pub fn must_api_key(&self) -> String {
        self.api_key.clone().unwrap_or(String::new())
    }
}

/// Default configuration settings
#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultConfig {
    /// Default model vendor to use
    pub default_vendor: PromptModelVendor,
    /// Request timeout in seconds
    pub timeout: u64,
}

/// Parameters for model inference
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelParameters {
    /// Temperature for controlling randomness (0.0 to 1.0)
    pub temperature: f32,
    /// Top-p sampling parameter (0.0 to 1.0)
    pub top_p: f32,
    /// Top-k sampling parameter
    pub top_k: u32,
    /// Maximum number of tokens to generate
    pub max_tokens: u32,
}

#[cfg(test)]
mod test {
    use crate::config::vendor::ModelConfig;

    use super::*;

    #[test]
    fn test_config() {
        let params = ModelConfig {
            model: String::from("gpt-3.5-turbo"),
            api_key: Some(String::from("sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")),
        };

        let mut cfg = GlobalConfig::new();
        // cfg.set_model(UseModel::DeepSeek(params));

        let toml_str = toml::to_string(&cfg).unwrap();
        println!("{}", toml_str);
    }

    #[test]
    fn config_serialization() {
        let toml_str = r#"
[model.DeepSeek]
model = "gpt-3.5-turbo"
api_key = "sk-12345678"
        "#;

        // let cfg: GlobalConfig = toml::from_str(toml_str).unwrap();
    }

    #[test]
    fn save_config() {
        let mut cfg = GlobalConfig::new();

        // let params = OpenAILikeParams {
        //     model: String::from("gpt-3.5-turbo"),
        //     api_key: String::from("sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"),
        // };

        // cfg.set_model(UseModel::DeepSeek(params));
        // cfg.save();
    }
}
