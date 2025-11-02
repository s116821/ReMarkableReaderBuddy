use super::{BoundingBox, QuestionContext};
use anyhow::Result;
use log::{debug, info};

/// Extracts question text near circled regions
pub struct QuestionExtractor;

impl QuestionExtractor {
    /// Extract question context for each circled region
    /// Uses LLM vision capabilities to identify and extract handwritten questions
    pub fn extract_question_for_circle(
        circled_region: &BoundingBox,
        full_screenshot_base64: &str,
    ) -> Result<QuestionContext> {
        debug!(
            "Extracting question for circled region at ({}, {})",
            circled_region.x, circled_region.y
        );

        // TODO: Implement LLM-based question extraction
        // For now, return a placeholder
        let question_text = "What is this?".to_string();
        let question_region = None;

        Ok(QuestionContext::new(
            circled_region.clone(),
            question_text,
            question_region,
            full_screenshot_base64.to_string(),
        ))
    }

    /// Use vision LLM to detect handwritten text near the circled region
    /// Returns the extracted question text and its bounding box
    fn extract_handwritten_question_via_llm(
        circled_region: &BoundingBox,
        base64_image: &str,
    ) -> Result<(String, Option<BoundingBox>)> {
        // TODO: Implement LLM vision-based text extraction
        // Prompt: "Look at this image. There is a circled region near coordinates (x, y).
        //          Find the handwritten question text closest to this circle.
        //          Return the question text and its approximate location."

        info!("LLM-based question extraction not yet fully implemented");
        Ok(("Placeholder question".to_string(), None))
    }

    /// Find the nearest text region to a given circled area
    /// Uses spatial heuristics to identify likely question locations
    fn find_nearest_text_region(
        circled_region: &BoundingBox,
        text_regions: &[BoundingBox],
    ) -> Option<BoundingBox> {
        // TODO: Implement spatial proximity calculation
        // Consider:
        // - Distance from circle center
        // - Text orientation (horizontal vs vertical)
        // - Typical reading patterns (left-to-right, top-to-bottom)

        debug!("Finding nearest text region to circle");
        None
    }
}
