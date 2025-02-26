use serde::{Deserialize, Serialize};

// pub trait LLM {
//     async fn chat_stream(&self, messages: Vec<Message>) -> Result<Iterator<Item=String>>
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn new_system(content: String) -> Message {
        Message {
            role: "system".to_string(),
            content,
        }
    }

    pub fn new_user(content: String) -> Message {
        Message {
            role: "user".to_string(),
            content,
        }
    }
}
