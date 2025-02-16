use crate::config::ModelConfig;
use crate::llm::openai_compatible::OpenAICompatible;

pub(crate) struct OpenAICompatibleBuilder {
    url: String,
    model: String,
    api_key: String,
}

impl OpenAICompatibleBuilder {
    pub fn new(model_config: &ModelConfig) -> Self {
        OpenAICompatibleBuilder {
            url: model_config.base_url.clone(),
            model: model_config.model.clone(),
            api_key: model_config.must_api_key(),
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
