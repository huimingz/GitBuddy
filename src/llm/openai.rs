use serde_json::json;
use anyhow::anyhow;
use std::io::{BufRead, BufReader};
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