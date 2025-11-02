pub mod openai;

use anyhow::Result;
use serde_json::Value as JsonValue;

pub trait LLMEngine {
    fn add_text_content(&mut self, text: &str);
    fn add_image_content(&mut self, base64_image: &str);
    fn clear_content(&mut self);
    fn execute(&mut self) -> Result<String>;
}

