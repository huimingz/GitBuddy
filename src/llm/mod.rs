mod formatter;
mod openai_compatible;
mod openai_compatible_builder;
mod theme;

use crate::config;
use crate::config::{ModelConfig, ModelParameters};
use crate::prompt::Prompt;
use anyhow::Result;
use clap::ValueEnum;
use colored::Colorize;
use openai_compatible_builder::OpenAICompatibleBuilder;
use serde::{Deserialize, Serialize};
use std::io::Write;

/// Prompt model
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Deserialize, Serialize, Hash)]
pub enum PromptModelVendor {
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

impl PromptModelVendor {
    pub fn default_model(&self) -> String {
        match self {
            PromptModelVendor::OpenAI => "gpt-3.5-turbo".to_string(),
            PromptModelVendor::DeepSeek => "deepseek-chat".to_string(),
            PromptModelVendor::Ollama => "ollama".to_string(),
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

pub fn llm_request(
    diff_content: &str,
    vendor: Option<PromptModelVendor>,
    model: Option<String>,
    prompt: Prompt,
    prefix: Option<String>,
) -> Result<LLMResult> {
    let config = config::get_config()?;
    let model_config = config
        .load_model(vendor)
        .expect("must load model config and prompt template");

    let mut mc = model_config.clone();
    if let Some(m) = model {
        mc.model = m
    }
    get_commit_message(diff_content, prompt, prefix, &mc, config.model_params())
}

fn get_commit_message(
    diff_content: &str,
    prompt: Prompt,
    prefix: Option<String>,
    model_config: &ModelConfig,
    model_option: ModelParameters,
) -> Result<LLMResult> {
    let builder = OpenAICompatibleBuilder::new(model_config);

    // generate http request
    let m = builder.build(prompt.value().to_string());
    let result = m.request(diff_content, model_option, prefix)?;
    Ok(result)
}

pub enum Confirm<'a> {
    Ok(&'a String),
    Retry,
    Exit,
}

pub fn confirm_commit<'a>(result: &'a LLMResult, _commit_message: &'a str) -> Result<Confirm<'a>, &'static str> {
    theme::print_commit_options(result, theme::DEFAULT_COMMIT_OPTION_STYLE);
    let input = user_choice(result);
    match input.as_str() {
        "" => Ok(Confirm::Ok(&result.commit_messages[0])),
        "n" => Ok(Confirm::Exit),
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

fn user_choice(result: &LLMResult) -> String {
    print!(
        "\n{} {} {} {} {}\n{} ",
        "🎯".bright_yellow(),
        "Select Your Commit".bright_cyan().bold(),
        format!("[1-{}]", result.commit_messages.len()).bright_green(),
        "•".bright_yellow(),
        "(n: cancel)".bright_red(),
        "⌲ Enter your choice (default: 1): ".bright_yellow()
    );
    let mut input = String::new();

    // flush
    std::io::stdout().flush().expect("Failed to flush stdout");
    std::io::stdin().read_line(&mut input).expect("Failed to read line");

    input.trim().to_lowercase()
}
