use crate::llm::openai_compatible::OpenAICompatible;
use crate::llm::PromptModelVendor;

pub(crate) struct OpenAICompatibleBuilder {
    url: String,
    model: String,
    api_key: String,
}

impl OpenAICompatibleBuilder {
    pub fn new(vendor: PromptModelVendor, model: &str, api_key: &str) -> Self {
        match vendor {
            PromptModelVendor::OpenAI => OpenAICompatibleBuilder {
                url: String::from("https://api.openai.com"),
                model: model.to_string(),
                api_key: api_key.to_string(),
            },
            PromptModelVendor::DeepSeek => OpenAICompatibleBuilder {
                url: String::from("https://api.deepseek.com"),
                model: model.to_string(),
                api_key: api_key.to_string(),
            },
            PromptModelVendor::Ollama => OpenAICompatibleBuilder {
                url: String::from("http://localhost:11434"),
                model: model.to_string(),
                api_key: api_key.to_string(),
            },
        }
    }

    pub fn build(self, prompt: String) -> OpenAICompatible {
        OpenAICompatible {
            url: self.url,
            model: self.model,
            prompt: prompt,
            api_key: self.api_key,
        }
    }
}
