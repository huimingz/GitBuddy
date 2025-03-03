use crate::config::ModelConfig;
use crate::llm::openai_compatible::OpenAICompatible;

pub(crate) struct OpenAICompatibleBuilder {
    url: String,
}

impl OpenAICompatibleBuilder {
    pub fn new(model_config: &ModelConfig) -> Self {
        OpenAICompatibleBuilder {
            url: model_config.base_url.clone(),
        }
    }

    pub fn build(self) -> OpenAICompatible {
        OpenAICompatible {}
    }
}
