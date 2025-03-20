use crate::args::CommandArgs;
use crate::config::{ModelConfig, ModelParameters};
use crate::llm::openai::{OpenAIClient, OpenAIResponseUsage};
use crate::llm::{llm, theme, LLMResult};
use anyhow::{Error, Result};
use colored::Colorize;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io;
use std::io::{BufRead, Write};

pub(crate) fn generate_git_commit_messages(
    diff_content: &str,
    model_config: &ModelConfig,
    option: ModelParameters,
    args: &CommandArgs,
    prompt: String,
) -> Result<LLMResult, anyhow::Error> {
    let client = OpenAIClient::new_from_config(model_config, None);
    print_configuration(&model_config.model, diff_content, &option, &client.base_url);

    let messages = git_commit_prompt(diff_content, args.hint.as_ref(), prompt);

    let (output, usage) = stream_chat_response(option, client, messages)?;

    let re = Regex::new(r"(?s)<think>.*?</think>")
        .map_err(|e| format!("invalid regex, err: {e}"))
        .unwrap();
    let message = re.replace_all(&output.trim(), "").trim().to_string();
    let messages = process_llm_response(message.clone(), args.reference.as_ref())?;

    Ok(LLMResult {
        completion_tokens: usage.completion_tokens,
        prompt_tokens: usage.prompt_tokens,
        total_tokens: usage.total_tokens,
        commit_message: message,
        commit_messages: messages,
    })
}

fn stream_chat_response(
    option: ModelParameters,
    client: OpenAIClient,
    messages: Vec<llm::Message>,
) -> Result<(String, OpenAIResponseUsage), Error> {
    let mut output = String::new();
    let mut usage = OpenAIResponseUsage::default();

    let (start_separator, end_separator) = theme::get_stream_separator(3); // 使用方案2，可以改为1或3尝试其他效果
    println!("{}", start_separator);
    for (data, _line) in client.stream_chat(messages, option)? {
        for choice in data.choices {
            print!("{}", choice.delta.content.cyan());
            io::stdout().flush()?; // flush to terminal, ensure each print is visible
            output.push_str(choice.delta.content.as_str());
        }
        if let Some(u) = data.usage {
            usage.total_tokens += u.total_tokens;
            usage.prompt_tokens += u.prompt_tokens;
            usage.completion_tokens += u.completion_tokens;
        }
    }
    println!("\n{}", end_separator);
    Ok((output, usage))
}

fn git_commit_prompt(diff_content: &str, hint: Option<&String>, prompt: String) -> Vec<llm::Message> {
    let mut messages = Vec::new();
    messages.push(llm::Message::new_system(prompt));
    messages.push(llm::Message::new_user(format!("Generate commit message for these changes. If it's a new file, focus on its purpose rather than analyzing its content:\n```diff\n{diff_content}\n```")));
    if let Some(p) = hint {
        messages.push(llm::Message::new_user(format!("hint: {p}")));
    }
    messages
}

fn print_configuration(model: &String, diff_content: &str, option: &ModelParameters, url: &String) {
    println!(
        "\n{} {} {}",
        "⚙️".bright_cyan(),
        "LLM Configuration".bright_cyan().bold(),
        "🔮".bright_cyan()
    );
    println!("  {} Model: {}", "🚀".bright_yellow(), model.bright_green().bold());
    println!(
        "  {} Max Tokens: {}",
        "⚡".bright_yellow(),
        option.max_tokens.to_string().bright_green().bold()
    );
    println!(
        "  {} Temperature: {}",
        "🎲".bright_yellow(),
        option.temperature.to_string().bright_green().bold()
    );
    println!(
        "  {} Top P: {}",
        "🎯".bright_yellow(),
        option.top_p.to_string().bright_green().bold()
    );
    println!(
        "  {} Diff Length: {} chars",
        "📏".bright_yellow(),
        diff_content.len().to_string().bright_green().bold()
    );
    println!(
        "  {} Diff Lines: {} lines",
        "📑".bright_yellow(),
        diff_content.lines().count().to_string().bright_green().bold()
    );
    println!("  {} Endpoint: {}\n", "🌐".bright_yellow(), url.bright_green());
}
fn fix_json_response(text: &str) -> String {
    // 使用正则表达式匹配 JSON 数组部分
    let re = Regex::new(r"\[\s*\{.*\}\s*\]").unwrap();

    if let Some(json_match) = re.find(text) {
        let json_str = json_match.as_str();

        // 清理常见的 JSON 格式问题
        let cleaned = json_str
            .replace("\\n", "\n") // 处理转义的换行符
            .replace("\\\"", "\"") // 处理转义的引号
            .replace("\\\\", "\\") // 处理转义的反斜杠
            .replace("\\'", "'"); // 处理转义的单引号

        // 处理数组末尾的多余逗号
        let comma_fixed = Regex::new(r",(\s*\])").unwrap().replace_all(&cleaned, "$1").to_string();

        // 尝试解析和重新格式化 JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&comma_fixed) {
            return serde_json::to_string(&json).unwrap_or(comma_fixed);
        }
        comma_fixed
    } else {
        text.to_string()
    }
}

fn extract_json_content(text: &str) -> String {
    // 首先尝试匹配 ```json
    let json_re = Regex::new(r"```json\s*([\s\S]*?)\s*```").unwrap();
    if let Some(captures) = json_re.captures(text) {
        if let Some(json_content) = captures.get(1) {
            return json_content.as_str().trim().to_string();
        }
    }

    // 如果没找到 ```json，尝试匹配普通的 ```
    let code_re = Regex::new(r"```\s*([\s\S]*?)\s*```").unwrap();
    if let Some(captures) = code_re.captures(text) {
        if let Some(code_content) = captures.get(1) {
            return code_content.as_str().trim().to_string();
        }
    }

    // 如果都没找到，返回原始文本
    text.to_string()
}

fn process_llm_response(response: String, reference: Option<&String>) -> Result<Vec<String>> {
    // 首先尝试提取代码块内容
    let content = extract_json_content(&response);

    // 尝试修复和解析 JSON
    let fixed_json = fix_json_response(&content);

    match serde_json::from_str::<Vec<CommitMessage>>(&fixed_json) {
        Ok(messages) => Ok(messages
            .into_iter()
            .map(|msg| {
                let mut commit = String::new();

                // 构建提交消息头
                if let Some(scope) = msg.scope.filter(|s| !s.trim().is_empty()) {
                    commit.push_str(&format!("{}({}): {}", msg.r#type, scope.trim(), msg.subject.trim()));
                } else {
                    commit.push_str(&format!("{}: {}", msg.r#type, msg.subject.trim()));
                }

                // 添加 issue 引用
                if let Some(r) = reference {
                    commit.push_str(" ");
                    commit.push_str(r.as_str());
                }

                // 添加可选的消息体
                if let Some(body) = msg.body.filter(|s| !s.trim().is_empty()) {
                    commit.push_str("\n\n");
                    commit.push_str(&theme::wrap_text(body.trim(), 100));
                }

                // 添加可选的页脚
                if let Some(footer) = msg.footer.filter(|s| !s.trim().is_empty()) {
                    commit.push_str("\n\n");
                    commit.push_str(&theme::wrap_text(footer.trim(), 100));
                }

                commit
            })
            .collect()),
        Err(e) => {
            println!("Parse JSON failed: {}", e);
            Err(anyhow::anyhow!("Parse JSON failed: {}", e))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CommitMessage {
    r#type: String,
    scope: Option<String>,
    subject: String,
    body: Option<String>,
    footer: Option<String>,
}
