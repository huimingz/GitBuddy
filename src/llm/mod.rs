mod git_commit;
mod interaction;
mod llm;
mod openai;
mod theme;

use crate::args::CommandArgs;
use crate::config;
use crate::config::{ModelConfig, ModelParameters};
use crate::llm::git_commit::generate_git_commit_messages;
use crate::prompt::Prompt;
use anyhow::{anyhow, Error, Result};
use clap::ValueEnum;
use colored::Colorize;
use minijinja::{context, Environment};
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

pub fn llm_request(diff_content: &str, _prompt: Prompt, args: &CommandArgs) -> Result<LLMResult> {
    let config = config::get_config()?;
    let model_config = config
        .load_model(args.vendor.clone())
        .expect("must load model config and prompt template");

    let mut mc = model_config.clone();
    if let Some(m) = args.model.as_ref() {
        mc.model = m.clone()
    }
    get_commit_message(diff_content, &mc, config.model_params(), args)
}

fn get_commit_message(
    diff_content: &str,
    model_config: &ModelConfig,
    model_option: ModelParameters,
    args: &CommandArgs,
) -> Result<LLMResult> {
    let rendered_prompt = render_prompt(args.prompt, args.number_of_commit_options, &args.language)?;
    let result = generate_git_commit_messages(diff_content, model_config, model_option, args, rendered_prompt)
        .map_err(|e| anyhow!("request failed: {:?}", e))?;
    Ok(result)
}

fn render_prompt(prompt: Prompt, number: u8, language: &String) -> Result<String, Error> {
    let p = prompt.value();

    let mut env = Environment::new();
    env.add_template("prompt", p)?;
    let tmpl = env.get_template("prompt")?;
    let rendered = tmpl.render(context! {
        number => number,
        language => map_language(language),
    })?;
    Ok(rendered)
}

fn map_language(lang: &String) -> &str {
    match lang.as_str().to_lowercase().as_str() {
        "en" => "English",
        "zh" => "Chinese",
        "ja" => "Japanese",
        "ko" => "Korean",
        "es" => "Spanish",
        "fr" => "French",
        "de" => "German",
        "it" => "Italian",
        "pt" => "Portuguese",
        "ru" => "Russian",
        "tr" => "Turkish",
        "pl" => "Polish",
        "nl" => "Dutch",
        "sv" => "Swedish",
        "fi" => "Finnish",
        "hu" => "Hungarian",
        _ => lang.as_str(),
    }
}

pub enum Confirm<'a> {
    Ok(&'a String),
    Retry,
    Exit,
}

pub fn confirm_commit<'a>(result: &'a LLMResult) -> Result<Confirm<'a>, &'static str> {
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
