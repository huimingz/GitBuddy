use crate::config::ModelConfig;
use crate::llm::openai_compatible::OpenAICompatible;

pub(crate) struct OpenAICompatibleBuilder {
    url: String,
    model: String,
}

impl OpenAICompatibleBuilder {
    pub fn new(model_config: &ModelConfig) -> Self {
        OpenAICompatibleBuilder {
            url: model_config.base_url.clone(),
            model: model_config.model.clone(),
        }
    }

    pub fn build(self, prompt: String) -> OpenAICompatible {
        OpenAICompatible {
            url: self.url,
            model: self.model,
            prompt: prompt,
        }
    }
}
