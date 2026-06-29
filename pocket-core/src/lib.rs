use thiserror::Error;

#[derive(Error, Debug)]
pub enum PocketError {
    #[error("I/O Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Configuration Error: {0}")]
    ConfigError(String),

    #[error("Engine Inference Error: {0}")]
    EngineError(String),

    #[error("Insufficient System RAM: {0}")]
    MemoryInsufficient(String),
}

pub type Result<T> = std::result::Result<T, PocketError>;

/// Represents the role of the speaker in a conversation turn
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    System,
    User,
    Assistant,
}

/// A single turn of a multi-turn conversation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

/// Manages rolling conversation context history and chat template formatting
#[derive(Debug, Clone)]
pub struct ConversationManager {
    pub system_prompt: String,
    pub messages: Vec<Message>,
}

impl ConversationManager {
    /// Instantiates a new ConversationManager with an optional system prompt override
    pub fn new(system_prompt: Option<String>) -> Self {
        Self {
            system_prompt: system_prompt
                .unwrap_or_else(|| "You are a helpful, highly capable AI assistant.".to_string()),
            messages: Vec::new(),
        }
    }

    /// Appends a new turn to the message log
    pub fn add_message(&mut self, role: Role, content: String) {
        self.messages.push(Message { role, content });
    }

    /// Resets the conversation logs
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Stitches system prompt and history into a ChatML/Llama-3 hybrid template string,
    /// leaving the final assistant turn block open.
    pub fn get_formatted_prompt(&self) -> String {
        let mut prompt = String::new();

        // 1. Append system prompt block
        if !self.system_prompt.is_empty() {
            prompt.push_str("<|system|>\n");
            prompt.push_str(&self.system_prompt);
            prompt.push_str("<|end|>\n");
        }

        // 2. Append history turns
        for message in &self.messages {
            match message.role {
                Role::System => {
                    prompt.push_str("<|system|>\n");
                    prompt.push_str(&message.content);
                    prompt.push_str("<|end|>\n");
                }
                Role::User => {
                    prompt.push_str("<|user|>\n");
                    prompt.push_str(&message.content);
                    prompt.push_str("<|end|>\n");
                }
                Role::Assistant => {
                    prompt.push_str("<|assistant|>\n");
                    prompt.push_str(&message.content);
                    prompt.push_str("<|end|>\n");
                }
            }
        }

        // 3. Open assistant response turn
        prompt.push_str("<|assistant|>\n");

        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_manager_formatting() {
        let mut manager =
            ConversationManager::new(Some("You are a helpful assistant.".to_string()));

        // Assert initial formatting contains only system prompt
        let initial_prompt = manager.get_formatted_prompt();
        assert_eq!(
            initial_prompt,
            "<|system|>\nYou are a helpful assistant.<|end|>\n<|assistant|>\n"
        );

        // Append user turn and verify
        manager.add_message(Role::User, "Hello!".to_string());
        let after_user = manager.get_formatted_prompt();
        assert_eq!(
            after_user,
            "<|system|>\nYou are a helpful assistant.<|end|>\n<|user|>\nHello!<|end|>\n<|assistant|>\n"
        );

        // Append assistant turn and verify
        manager.add_message(Role::Assistant, "Hi there!".to_string());
        manager.add_message(Role::User, "Tell me a story.".to_string());
        let final_prompt = manager.get_formatted_prompt();
        assert_eq!(
            final_prompt,
            "<|system|>\nYou are a helpful assistant.<|end|>\n<|user|>\nHello!<|end|>\n<|assistant|>\nHi there!<|end|>\n<|user|>\nTell me a story.<|end|>\n<|assistant|>\n"
        );
    }
}
