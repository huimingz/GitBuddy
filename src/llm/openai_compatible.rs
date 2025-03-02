use crate::config::{ModelConfig, ModelParameters};
use crate::llm::{llm, theme, LLMResult};
use anyhow::{anyhow, Error, Result};
use colored::Colorize;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io;
use std::io::{BufRead, BufReader, Write};

#[derive(Debug)]
pub(crate) struct OpenAICompatible {
    pub(crate) model: String,
    pub(crate) prompt: String,
}

pub struct OpenAIClient {
    base_url: String,
    model: String,
    api_key: Option<String>,
    client: reqwest::blocking::Client,
}

impl OpenAIClient {
    pub fn new_from_config(conf: &ModelConfig, model: Option<String>) -> OpenAIClient {
        let model = model.unwrap_or(conf.model.clone());
        OpenAIClient {
            base_url: conf.base_url.clone(),
            model,
            api_key: conf.api_key.clone(),
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn new(base_url: String, model: String, api_key: String) -> OpenAIClient {
        OpenAIClient {
            base_url,
            model,
            api_key: Some(api_key),
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn new_with_client(
        client: reqwest::blocking::Client,
        base_url: String,
        model: String,
        api_key: String,
    ) -> OpenAIClient {
        OpenAIClient {
            base_url,
            model,
            api_key: Some(api_key),
            client,
        }
    }

    pub fn chat(&self, messages: Vec<llm::Message>, option: ModelParameters) -> Result<impl Iterator<Item = String>> {
        let payload = &json!({
            "model": self.model,
            "messages": messages,
            "options": {
                "temperature": option.temperature,
                "top_p": option.top_p,
                "top_k": option.top_k,
            },
            "options": option,
            "keep_alive": "30m",
            "max_tokens": option.max_tokens,
            "stream": true,
        });

        let mut builder = self.client.post(format!("{}/chat/completions", self.base_url));
        if let Some(key) = &self.api_key {
            builder = builder.header("Authorization", format!("Bearer {}", key));
        }
        let response = builder
            .header("Accept", "text/event-stream")
            .header("Content-Type", "application/json")
            .json(payload)
            .send()?;

        if !response.status().is_success() {
            return Err(anyhow!("HTTP request failed with status code {}", response.status()));
        }

        let reader = BufReader::new(response);
        Ok(reader
            .lines()
            .filter_map(|l| {
                l.ok().and_then(|s| {
                    if s.starts_with("data: ") {
                        Some(s["data: ".len()..].to_string())
                    } else {
                        None
                    }
                })
            })
            .filter(|s| s != "[DONE]"))
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct OpenAIStreamResponse {
    id: String,
    model: String, // 生成该 completion 的模型名
    object: String,
    system_fingerprint: Option<String>, // This fingerprint represents the backend configuration that the model runs with.
    choices: Vec<OpenAIStreamChoice>,
    usage: Option<OpenAIResponseUsage>,
    created: i64, // 创建聊天完成时的 Unix 时间戳（以秒为单位）
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct OpenAIStreamChoice {
    index: i64,
    delta: OpenAIChoiceDelta,
    finish_reason: Option<String>, // 模型停止生成 token 的原因:stop/length/content_filter
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct OpenAIChoiceDelta {
    role: Option<String>,
    content: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct OpenAIResponse {
    id: String,
    model: String, // 生成该 completion 的模型名
    object: String,
    system_fingerprint: String, // This fingerprint represents the backend configuration that the model runs with.
    choices: Vec<OpenAIResponseChoice>,
    usage: OpenAIResponseUsage, // 该对话补全请求的用量信息
    created: i64,               // 创建聊天完成时的 Unix 时间戳（以秒为单位）
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponseChoice {
    index: i64, // 该 completion 在模型生成的 completion 的选择列表中的索引。
    message: OpenAIResponseChoiceMessage,
    finish_reason: String, // 模型停止生成 token 的原因:stop/length/content_filter
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponseChoiceMessage {
    role: String, // 角色:assistant
    content: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct OpenAIResponseUsage {
    completion_tokens: i64,
    prompt_tokens: i64,
    total_tokens: i64,
}

#[derive(Serialize, Deserialize)]
struct Message {
    #[serde(rename = "role")]
    role: String,
    #[serde(rename = "content")]
    content: String,
}

impl OpenAICompatible {
    pub(crate) fn request(
        &self,
        diff_content: &str,
        model_config: &ModelConfig,
        option: ModelParameters,
        hint: Option<String>,
    ) -> Result<LLMResult, anyhow::Error> {
        let client = OpenAIClient::new_from_config(model_config, None);
        OpenAICompatible::print_configuration(&self.model, diff_content, &option, &client.base_url);

        let messages = self.git_commit_prompt(diff_content, hint);

        let (output, usage) = self.stream_chat_response(option, client, messages)?;

        let re = Regex::new(r"(?s)<think>.*?</think>")
            .map_err(|e| format!("invalid regex, err: {e}"))
            .unwrap();
        let message = re.replace_all(&output.trim(), "").trim().to_string();
        let messages = process_llm_response(message.clone())?;

        Ok(LLMResult {
            completion_tokens: usage.completion_tokens,
            prompt_tokens: usage.prompt_tokens,
            total_tokens: usage.total_tokens,
            commit_message: message,
            commit_messages: messages,
        })
    }

    fn stream_chat_response(
        &self,
        option: ModelParameters,
        client: OpenAIClient,
        messages: Vec<llm::Message>,
    ) -> Result<(String, OpenAIResponseUsage), Error> {
        let mut output = String::new();
        let mut usage = OpenAIResponseUsage::default();

        let (start_separator, end_separator) = theme::get_stream_separator(3); // 使用方案2，可以改为1或3尝试其他效果
        println!("{}", start_separator);
        for line in client.chat(messages, option)? {
            if line.is_empty() {
                continue;
            }
            let data: OpenAIStreamResponse = serde_json::from_str(&line)?;
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

    fn git_commit_prompt(&self, diff_content: &str, hint: Option<String>) -> Vec<llm::Message> {
        let mut messages = Vec::new();
        messages.push(llm::Message::new_system(self.prompt.clone()));
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

fn process_llm_response(response: String) -> Result<Vec<String>> {
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

                // 添加可选的消息体
                if let Some(body) = msg.body.filter(|s| !s.trim().is_empty()) {
                    commit.push_str("\n\n");
                    commit.push_str(&theme::wrap_text(body.trim(), 80));
                }

                // 添加可选的页脚
                if let Some(footer) = msg.footer.filter(|s| !s.trim().is_empty()) {
                    commit.push_str("\n\n");
                    commit.push_str(&theme::wrap_text(footer.trim(), 80));
                }

                commit
            })
            .collect()),
        Err(e) => {
            println!("Parse JSON failed: {}", e);
            Err(anyhow::anyhow!("Parse JSON failed: {}", e))
            //  // 如果 JSON 解析失败，回退到原始的分隔符处理方式
            //  response
            //  .replace("---", "===")
            //  .replace("___", "===")
            //  .replace("***", "===")
            //  .replace("```", "")
            //  .split("===")
            //  .map(|s| s.trim().to_string())
            //  .filter(|s| !s.is_empty())
            //  .collect()
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
