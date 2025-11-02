pub mod circle_detector;
pub mod question_extractor;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Represents a region of interest on the screen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Represents a detected circled region with its associated question
#[derive(Debug, Clone)]
pub struct QuestionContext {
    pub circled_region: BoundingBox,
    pub question_text: String,
    pub question_region: Option<BoundingBox>,
    pub full_screenshot_base64: String,
}

impl QuestionContext {
    pub fn new(
        circled_region: BoundingBox,
        question_text: String,
        question_region: Option<BoundingBox>,
        full_screenshot_base64: String,
    ) -> Self {
        Self {
            circled_region,
            question_text,
            question_region,
            full_screenshot_base64,
        }
    }
}

