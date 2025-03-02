use serde_json::json;
use anyhow::anyhow;
use std::io::{BufRead, BufReader};
use serde::{Deserialize, Serialize};
use crate::config::{ModelConfig, ModelParameters};
use crate::llm::llm;

pub struct OpenAIClient {
    pub(crate) base_url: String,
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

    pub fn chat(&self, messages: Vec<llm::Message>, option: ModelParameters) -> anyhow::Result<impl Iterator<Item = String>> {
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
pub struct OpenAIStreamResponse {
    pub id: String,
    pub model: String, // 生成该 completion 的模型名
    pub object: String,
    pub system_fingerprint: Option<String>, // This fingerprint represents the backend configuration that the model runs with.
    pub choices: Vec<OpenAIStreamChoice>,
    pub usage: Option<OpenAIResponseUsage>,
    pub created: i64, // 创建聊天完成时的 Unix 时间戳（以秒为单位）
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct OpenAIStreamChoice {
    pub index: i64,
    pub delta: OpenAIChoiceDelta,
    pub finish_reason: Option<String>, // 模型停止生成 token 的原因:stop/length/content_filter
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct OpenAIChoiceDelta {
    pub role: Option<String>,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct OpenAIResponse {
    pub id: String,
    pub model: String, // 生成该 completion 的模型名
    pub object: String,
    pub system_fingerprint: String, // This fingerprint represents the backend configuration that the model runs with.
    pub choices: Vec<OpenAIResponseChoice>,
    pub usage: OpenAIResponseUsage, // 该对话补全请求的用量信息
    pub created: i64,               // 创建聊天完成时的 Unix 时间戳（以秒为单位）
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIResponseChoice {
    pub index: i64, // 该 completion 在模型生成的 completion 的选择列表中的索引。
    pub message: OpenAIResponseChoiceMessage,
    pub finish_reason: String, // 模型停止生成 token 的原因:stop/length/content_filter
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIResponseChoiceMessage {
    pub role: String, // 角色:assistant
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct OpenAIResponseUsage {
    pub completion_tokens: i64,
    pub prompt_tokens: i64,
    pub total_tokens: i64,
}