use crate::config::ModelParameters;
use crate::llm::LLMResult;
use crate::prompt::Prompt;
use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::{BufRead, BufReader};
use std::time::Duration;
use toml::Value::String;

#[derive(Debug)]
pub(crate) struct OpenAICompatible {
    pub(crate) url: String,
    pub(crate) model: String,
    pub(crate) prompt: String,
    pub(crate) api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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
    pub(crate) fn stream_request(
        &self,
        diff_content: &str,
        option: ModelParameters,
        prefix: Option<String>,
    ) -> Result<LLMResult> {
        let client = reqwest::blocking::Client::new();
    }

    pub(crate) fn request(
        &self,
        diff_content: &str,
        option: ModelParameters,
        prefix: Option<String>,
    ) -> Result<LLMResult> {
        let client = reqwest::blocking::Client::new();

        let api_key = self.api_key.clone();
        let mut messages: Vec<Message> = vec![
            Message {
                role: String::from("system"),
                content: self.prompt.clone(),
            },
            Message {
                role: String::from("user"),
                content: format!("diff content: \n{diff_content}"),
            },
        ];
        if let Some(p) = prefix {
            println!("expect prefix: {p}");

            messages.push(Message {
                role: String::from("user"),
                content: format!("commit message must be prefix with: {p}"),
            })
        }

        let url = format!("{}/v1/chat/completions", self.url);
        println!("Vendor Endpoint: {}", url);
        let response = client
            .post(url)
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
                // "format": {
                //     "type": "object",
                //     "properties": {"subject": {"type":"string"}, "scope": {"type":"string"}, "summary": {"type":"string"}},
                //     "required": ["subject", "scope", "summary"]
                // },
            }))
            .send()
            .expect("Error sending request");

        return if response.status().is_success() {
            let reader = BufReader::new(response);
            let mut buffer = String::new();
            for line in reader.lines() {
                let line = line?;
                if line.starts_with("data: ") {
                    let payload = line.trim_start_matches("data: ");
                    println!("收到事件流: {}", payload);
                }
            }

            let _response_json = OpenAIResponse {
                id: "".to_string(),
                model: "".to_string(),
                object: "".to_string(),
                system_fingerprint: "".to_string(),
                choices: vec![],
                usage: OpenAIResponseUsage {
                    completion_tokens: 0,
                    prompt_tokens: 0,
                    total_tokens: 0,
                },
                created: 0,
            };
            let response_json: OpenAIResponse = response.json().expect("Failed to parse response as JSON");

            if response_json.choices.is_empty() {
                panic!("No choices returned from OpenAI API");
            }
            let choice = &response_json.choices[0];
            let message = choice.message.content.clone().trim().to_owned();
            let re = Regex::new(r"(?s)<think>.*?</think>")
                .map_err(|e| format!("invalid regex, err: {e}"))
                .unwrap();
            for cap in re.captures_iter(&message) {
                println!("Think: {}\n------------------", &cap[0])
            }
            let message = re.replace_all(&message, "").trim().to_string();

            Ok(LLMResult {
                commit_message: message,
                total_tokens: response_json.usage.total_tokens,
                prompt_tokens: response_json.usage.prompt_tokens,
                completion_tokens: response_json.usage.completion_tokens,
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
}
