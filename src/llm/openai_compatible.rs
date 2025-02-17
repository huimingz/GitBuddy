use crate::config::ModelParameters;
use crate::llm::{formatter, LLMResult};
use anyhow::{anyhow, Result};
use colored::Colorize;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::time::Duration;

#[derive(Debug)]
pub(crate) struct OpenAICompatible {
    pub(crate) url: String,
    pub(crate) model: String,
    pub(crate) prompt: String,
    pub(crate) api_key: String,
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
        option: ModelParameters,
        hint: Option<String>,
    ) -> Result<LLMResult, anyhow::Error> {
        let client = reqwest::blocking::Client::new();

        let api_key = self.api_key.clone();
        let mut messages: Vec<Message> = vec![
            Message {
                role: String::from("system"),
                content: self.prompt.clone(),
            },
            Message {
                role: String::from("user"),
                content: format!("Generate commit message for these changes. If it's a new file, focus on its purpose rather than analyzing its content:\n```diff\n{diff_content}\n```"),
            },
        ];
        if let Some(p) = hint {
            println!("expect prefix: {p}");
            messages.push(Message {
                role: String::from("user"),
                content: format!("hint: {p}"),
            })
        }
        OpenAICompatible::print_configuration(&self.model, diff_content, &option, &self.url);

        let response = client
            .post(&self.url)
            .timeout(Duration::from_secs(120))
            .header("Accept", "text/event-stream")
            .header("Authorization", format!("Bearer {api_key}",))
            .json(&json!({
                "model": &self.model,
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
            }))
            .send()
            .expect("Error sending request");

        return if response.status().is_success() {
            let mut message = String::new();
            let reader = BufReader::new(response);
            let (start_separator, end_separator) = formatter::get_stream_separator(3); // 使用方案2，可以改为1或3尝试其他效果
            let mut usage = OpenAIResponseUsage::default();
            println!("{}", start_separator);
            for line in reader.lines() {
                let line = line?;
                if line.starts_with("data: ") {
                    let payload = line.trim_start_matches("data: ");
                    if payload == "[DONE]" {
                        break;
                    }

                    // println!("sse data: {}", payload);
                    let data: OpenAIStreamResponse = serde_json::from_str(payload)?;
                    for choice in data.choices {
                        print!("{}", choice.delta.content.cyan());
                        io::stdout().flush()?; // 强制刷新到终端，确保每次打印都显示
                        message.push_str(choice.delta.content.as_str());
                    }
                    if let Some(u) = data.usage {
                        usage.total_tokens += u.total_tokens;
                        usage.prompt_tokens += u.prompt_tokens;
                        usage.completion_tokens += u.completion_tokens;
                    }
                }
            }
            println!("\n{}", end_separator);

            let re = Regex::new(r"(?s)<think>.*?</think>")
                .map_err(|e| format!("invalid regex, err: {e}"))
                .unwrap();
            let message = re.replace_all(&message.trim(), "").trim().to_string();
            let messages = process_llm_response(message.clone())?;

            Ok(LLMResult {
                completion_tokens: usage.completion_tokens,
                prompt_tokens: usage.prompt_tokens,
                total_tokens: usage.total_tokens,
                commit_message: message,
                commit_messages: messages,
            })
        } else {
            let status_code = response.status();
            let reason = match response.text() {
                Ok(text) => text,
                Err(e) => {
                    return Err(anyhow!("Error: {:?}", e.to_string().truncate(100)));
                }
            };
            return Err(anyhow!(
                "Error occurred in request, reason: '{}', status code: {}",
                reason,
                status_code
            ));
        };
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
                    commit.push_str(&formatter::wrap_text(body.trim(), 80));
                }

                // 添加可选的页脚
                if let Some(footer) = msg.footer.filter(|s| !s.trim().is_empty()) {
                    commit.push_str("\n\n");
                    commit.push_str(&formatter::wrap_text(footer.trim(), 80));
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
