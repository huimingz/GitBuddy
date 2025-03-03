use crate::prompt::Prompt;

pub struct CommandArgs {
    pub push: bool,
    pub dry_run: bool,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub prompt: Prompt,
    pub hint: Option<String>,
    pub number_of_commit_options: u8,
}

impl CommandArgs {
    pub fn new(
        push: bool,
        dry_run: bool,
        vendor: Option<String>,
        model: Option<String>,
        prompt: Prompt,
        hint: Option<String>,
        number_of_commit_options: u8,
    ) -> Self {
        Self {
            push,
            dry_run,
            vendor,
            model,
            prompt,
            hint,
            number_of_commit_options,
        }
    }
} 