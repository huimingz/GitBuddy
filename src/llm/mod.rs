mod openai_compatible;
mod openai_compatible_builder;

use crate::config;
use crate::config::ModelParameters;
use crate::prompt::Prompt;
use anyhow::Result;
use clap::ValueEnum;
use colored::Colorize;
use openai_compatible_builder::OpenAICompatibleBuilder;
use serde::{Deserialize, Serialize};
use std::io::Write;

/// Prompt model
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Deserialize, Serialize)]
pub enum PromptModel {
    #[clap(name = "openai")]
    #[serde(rename = "openai")]
    OpenAI,
    #[clap(name = "deepseek")]
    #[serde(rename = "deepseek")]
    DeepSeek,
    #[clap(name = "ollama")]
    #[serde(rename = "ollama")]
    Ollama,
}

impl PromptModel {
    pub fn default_model(&self) -> String {
        match self {
            PromptModel::OpenAI => "gpt-3.5-turbo".to_string(),
            PromptModel::DeepSeek => "deepseek-chat".to_string(),
            PromptModel::Ollama => "ollama".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct LLMResult {
    pub commit_message: String,
    pub completion_tokens: i64,
    pub prompt_tokens: i64,
    pub total_tokens: i64,
}

struct RequestsWrap {
    vendor: PromptModel,
    model: String,
    api_key: String,
}

impl RequestsWrap {
    fn new(vendor: PromptModel, model: String, api_key: String) -> Self {
        RequestsWrap { vendor, model, api_key }
    }
}

pub fn llm_request(
    diff_content: &str,
    vendor: Option<PromptModel>,
    model: Option<String>,
    prompt: Prompt,
    prefix: Option<String>,
) -> Result<LLMResult> {
    let config = config::get_config()?;

    let (model_config, prompt_model) = config
        .model(vendor)
        .expect("must load model config and prompt template");

    let model = model.unwrap_or(model_config.model.clone());
    println!("use model: {model}");

    get_commit_message(
        prompt_model,
        model.as_str(),
        model_config.api_key.clone().unwrap_or("".into()).as_str(),
        diff_content,
        config.model_params(),
        prompt,
        prefix,
    )
}

fn get_commit_message(
    vendor: PromptModel,
    model: &str,
    api_key: &str,
    diff_content: &str,
    option: ModelParameters,
    prompt: Prompt,
    prefix: Option<String>,
) -> Result<LLMResult> {
    let builder = OpenAICompatibleBuilder::new(vendor, model, api_key);

    // generate http request
    let m = builder.build(prompt.value().to_string());
    let result = m.request(diff_content, option, prefix)?;
    Ok(result)
}

pub enum Confirm {
    Ok,
    Retry,
    Exit,
}

pub fn confirm_commit(commit_message: &str) -> Result<Confirm, &str> {
    println!("--------------------------------------");
    println!("{}", commit_message.cyan().bold());
    println!("--------------------------------------");
    print!("Are you sure you want to commit? (Y/n/c) ");
    let mut input = String::new();

    // flush
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut input).expect("Failed to read line");

    match input.trim().to_lowercase().as_str() {
        "y" => Ok(Confirm::Ok),
        "n" => Ok(Confirm::Exit),
        "c" => Ok(Confirm::Retry),
        _ => Ok(Confirm::Ok),
    }
}
