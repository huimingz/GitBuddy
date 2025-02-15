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
    pub commit_messages: Vec<String>,
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

pub enum Confirm<'a> {
    Ok(&'a String),
    Retry,
    Exit,
}

pub fn confirm_commit<'a>(result: &'a LLMResult, _commit_message: &'a str) -> Result<Confirm<'a>, &'static str> {
    println!("----------------------- Commit Message -----------------------");
    for (idx, message) in result.commit_messages.iter().enumerate() {
        if idx < result.commit_messages.len() - 1 {
            println!(
                "{}\n{}\n",
                format!("Option {}:", idx + 1).bold().bright_cyan(),
                message.cyan()
            );
        } else {
            println!(
                "{}\n{}",
                format!("Option {}:", idx + 1).bold().bright_cyan(),
                message.cyan()
            );
        }
    }
    println!("--------------------------------------------------------------");
    print!(
        "Choose a commit message (1-{}, default: 1, n: cancel, c: customize): ",
        result.commit_messages.len()
    );
    let mut input = String::new();

    // flush
    std::io::stdout().flush().expect("Failed to flush stdout");
    std::io::stdin().read_line(&mut input).expect("Failed to read line");

    let input = input.trim().to_lowercase();
    match input.as_str() {
        "" => Ok(Confirm::Ok(&result.commit_messages[0])),
        "n" => Ok(Confirm::Exit),
        "c" => Ok(Confirm::Retry),
        num => {
            if let Ok(choice) = num.parse::<usize>() {
                if choice > 0 && choice <= result.commit_messages.len() {
                    Ok(Confirm::Ok(&result.commit_messages[choice - 1]))
                } else {
                    Err("Invalid input choice")
                }
            } else {
                Err("Input must be a number")
            }
        }
    }
}
