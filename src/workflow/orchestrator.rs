use anyhow::Result;
use log::{debug, error, info};

use super::{
    AnswerPageType, Workflow, MASK_BOTTOM_OFFSET, MASK_LEFT_OFFSET, MASK_RIGHT_OFFSET,
    MASK_TOP_OFFSET,
};
use crate::analysis::BoundingBox;
use crate::llm::{openai::OpenAI, LLMEngine};

/// Result from LLM analysis containing question, answer, and bounding boxes
struct AnalysisResult {
    question: String,
    answer: String,
    _question_box: Option<BoundingBox>,
    _outline_box: Option<BoundingBox>,
    _screenshot_data: Vec<u8>, // PNG data for downstream processing (reserved for future use)
}

/// High-level orchestrator for the complete workflow
pub struct Orchestrator {
    workflow: Workflow,
    llm: OpenAI,
}

impl Orchestrator {
    pub fn new(workflow: Workflow, llm: OpenAI) -> Self {
        Self { workflow, llm }
    }

    /// Run one complete iteration of the reader buddy workflow
    /// NOTE: v0.1 processes ONE outline-question pair per trigger
    pub fn run_iteration(&mut self) -> Result<()> {
        info!("=== Starting Reader Buddy Iteration ===");

        // Step 1: Wait for trigger
        self.workflow.wait_for_trigger()?;

        // Step 2: Capture screenshot (of current/question page)
        let (screenshot_base64, screenshot_png_data) =
            self.workflow.capture_screenshot_with_data()?;

        // Step 3: Single LLM call does everything:
        // - Detect outlined region
        // - Extract question text
        // - Generate answer
        let result =
            self.analyze_and_answer_single_call(&screenshot_base64, screenshot_png_data)?;

        match result {
            None => {
                info!("No outlined regions or questions detected");
                // Draw failure X on current page (no text output)
                self.workflow.draw_failure_x()?;
                return Ok(());
            }
            Some(result) => {
                info!(
                    "Got Q&A - Question: {} | Answer: {}",
                    result.question, result.answer
                );

                if let Err(e) = self.render_answer(&result) {
                    error!("Error rendering answer: {}", e);
                    // On error, draw failure X (no text output)
                    self.workflow.draw_failure_x()?;
                }
            }
        }

        info!("=== Iteration Complete ===");
        Ok(())
    }

    /// Single LLM call that does everything:
    /// 1. Detects outlined content
    /// 2. Extracts handwritten question
    /// 3. Generates answer
    /// 4. Provides bounding boxes
    ///
    /// Returns None if no outline/question found, or Some((question, answer, question_box, outline_box))
    fn analyze_and_answer_single_call(
        &mut self,
        screenshot_base64: &str,
        screenshot_png_data: Vec<u8>,
    ) -> Result<Option<AnalysisResult>> {
        info!("Sending single LLM call for analysis + answer");

        self.llm.clear_content();
        self.llm.add_text_content(
            "Look at this reMarkable tablet screenshot (768x1024 pixels). The user is reading and has:\n\
             1. Drawn an outline (circle, rectangle, or any closed shape) around some content\n\
             2. Written a handwritten question nearby about that content\n\n\
             Your task:\n\
             1. Identify what content has been outlined\n\
             2. Read the handwritten question text\n\
             3. Provide a clear, helpful answer based on the outlined content\n\
             4. Provide approximate bounding boxes for the outline and question regions\n\n\
             Respond EXACTLY in this format:\n\
             QUESTION: [the extracted question text]\n\
             QUESTION_BOX: x,y,width,height (approximate pixels where the question text is)\n\
             OUTLINE_BOX: x,y,width,height (approximate pixels of the outline shape)\n\
             ---\n\
             ANSWER: [your answer]\n\n\
             If you cannot find a clear outline or question, respond with just:\n\
             NONE\n\n\
             Note: Process only ONE outline-question pair (the most prominent one if multiple exist). \
             Keep the answer concise and focused. Boxes are in pixels with origin (0,0) at top-left."
        );
        self.llm.add_image_content(screenshot_base64);

        let response = self.llm.execute()?;
        info!("LLM Response: {}", response);

        // Parse the response
        if response.trim().to_uppercase().starts_with("NONE") {
            return Ok(None);
        }

        // Parse the structured response
        let parts: Vec<&str> = response.split("---").collect();
        if parts.len() < 2 {
            // Fallback: treat whole response as answer
            return Ok(Some(AnalysisResult {
                question: "What does this mean?".to_string(),
                answer: response,
                _question_box: None,
                _outline_box: None,
                _screenshot_data: screenshot_png_data,
            }));
        }

        let header = parts[0];
        let answer_text = parts[1]
            .trim()
            .strip_prefix("ANSWER:")
            .unwrap_or(parts[1])
            .trim();

        // Extract question text
        let question_text = Self::extract_field(header, "QUESTION:");

        // Extract bounding boxes
        let question_box = Self::parse_bounding_box(&Self::extract_field(header, "QUESTION_BOX:"));
        let outline_box = Self::parse_bounding_box(&Self::extract_field(header, "OUTLINE_BOX:"));

        debug!("Parsed - Question: {}", question_text);
        debug!("Question box: {:?}", question_box);
        debug!("Outline box: {:?}", outline_box);

        Ok(Some(AnalysisResult {
            question: question_text,
            answer: answer_text.to_string(),
            _question_box: question_box,
            _outline_box: outline_box,
            _screenshot_data: screenshot_png_data,
        }))
    }

    /// Extract a field value from the response
    fn extract_field(text: &str, field_name: &str) -> String {
        for line in text.lines() {
            if let Some(value) = line.strip_prefix(field_name) {
                return value.trim().to_string();
            }
        }
        "".to_string()
    }

    /// Parse bounding box from "x,y,width,height" format
    fn parse_bounding_box(text: &str) -> Option<BoundingBox> {
        let parts: Vec<&str> = text.split(',').collect();
        if parts.len() == 4 {
            if let (Ok(x), Ok(y), Ok(w), Ok(h)) = (
                parts[0].trim().parse::<i32>(),
                parts[1].trim().parse::<i32>(),
                parts[2].trim().parse::<i32>(),
                parts[3].trim().parse::<i32>(),
            ) {
                return Some(BoundingBox {
                    x,
                    y,
                    width: w,
                    height: h,
                });
            }
        }
        None
    }

    /// Render the answer on the next page
    ///
    /// Simplified flow:
    /// 1. Store original page screenshot for later comparison
    /// 2. Navigate right to next page  
    /// 3. Compare to original to verify we actually moved
    /// 4. Check if page is valid (blank or existing QA page)
    /// 5. If not valid or didn't move → ensure we're on original and draw X
    /// 6. If valid → render Q&A on that page
    fn render_answer(&mut self, result: &AnalysisResult) -> Result<()> {
        info!("Attempting to render Q&A on next page");

        // Step 1: Store original page screenshot for comparison
        self.workflow.screenshot.take_screenshot()?;
        let original_png = self.workflow.screenshot.get_image_data().to_vec();
        let original_img = image::load_from_memory(&original_png)?;
        debug!("Stored original page screenshot for comparison");

        // Step 2: Attempt to navigate to next page
        self.workflow.navigate_to_next_page()?;
        std::thread::sleep(std::time::Duration::from_millis(800));

        // Step 3: Take screenshot and compare to original
        self.workflow.screenshot.take_screenshot()?;
        let current_png = self.workflow.screenshot.get_image_data().to_vec();
        let current_img = image::load_from_memory(&current_png)?;

        let similarity_to_original = Workflow::compute_image_similarity_masked(
            &original_img,
            &current_img,
            MASK_LEFT_OFFSET,
            MASK_RIGHT_OFFSET,
            MASK_TOP_OFFSET,
            MASK_BOTTOM_OFFSET,
            5, // Default sample rate
        );
        debug!(
            "Similarity to original page: {:.2}%",
            similarity_to_original * 100.0
        );

        // If we're still very similar to original (>99.9%), we didn't actually navigate
        const SAME_PAGE_THRESHOLD: f32 = 0.999;
        let did_navigate = similarity_to_original < SAME_PAGE_THRESHOLD;

        if !did_navigate {
            info!(
                "No page exists to the right (similarity {:.1}% >= {:.1}%) - drawing X on original",
                similarity_to_original * 100.0,
                SAME_PAGE_THRESHOLD * 100.0
            );
            // We're confirmed still on original page, draw failure X
            self.workflow.draw_failure_x()?;
            return Ok(());
        }

        info!(
            "Navigation successful (similarity {:.1}% < {:.1}%)",
            similarity_to_original * 100.0,
            SAME_PAGE_THRESHOLD * 100.0
        );

        // Step 4: Check if the page we navigated to is valid (blank or QA)
        let page_type = self.workflow.is_valid_answer_page()?;

        match page_type {
            AnswerPageType::Invalid => {
                // Page exists but is not suitable - return to original
                info!(
                    "Next page is not valid (not blank and not a QA page) - returning to original"
                );

                // Navigate back and verify we're on original
                self.return_to_original_page(&original_img)?;
                self.workflow.draw_failure_x()?;
                return Ok(());
            }
            AnswerPageType::Blank => {
                // Step 5a: Blank page - render header first, then Q&A
                info!("Blank page found, rendering header and Q&A");

                // Switch to body text mode once before all rendering
                self.workflow.set_body_text_mode()?;

                // Header with two blank lines before first Q&A block
                self.workflow
                    .render_text("=== Reader Buddy Answers ===\n\n\n")?;

                // Save header pattern for future detection (only on first blank page)
                std::thread::sleep(std::time::Duration::from_millis(500));
                self.workflow.screenshot.take_screenshot()?;
                let new_png = self.workflow.screenshot.get_image_data();
                if let Ok(new_img) = image::load_from_memory(new_png) {
                    const HEADER_HEIGHT: u32 = 150; // Capture full header region from top
                    let header_img = new_img.crop_imm(
                        0,
                        0,
                        new_img.width(),
                        HEADER_HEIGHT.min(new_img.height()),
                    );
                    if let Err(e) = self.workflow.save_header_pattern(&header_img) {
                        log::warn!("Failed to save header pattern: {}", e);
                    }
                }
            }
            AnswerPageType::ExistingQA => {
                // Step 5b: Existing QA page - just append Q&A content (no header)
                info!("Existing QA page found, appending Q&A (no header needed)");

                // Switch to body text mode
                self.workflow.set_body_text_mode()?;
            }
        }

        // Render the Q&A
        let formatted_output = format!("Q: {}\n\nA: {}\n---\n", result.question, result.answer);

        self.workflow.render_text(&formatted_output)?;

        info!("Q&A rendered successfully");
        Ok(())
    }

    /// Navigate back to the original page and verify we arrived
    fn return_to_original_page(&mut self, original_img: &image::DynamicImage) -> Result<()> {
        const MAX_ATTEMPTS: u32 = 3;
        const SAME_PAGE_THRESHOLD: f32 = 0.999;

        info!(
            "Attempting to return to original page (threshold: {:.1}%)",
            SAME_PAGE_THRESHOLD * 100.0
        );

        for attempt in 1..=MAX_ATTEMPTS {
            info!(
                "Return attempt {}/{}: navigating to previous page...",
                attempt, MAX_ATTEMPTS
            );

            self.workflow.navigate_to_previous_page()?;
            std::thread::sleep(std::time::Duration::from_millis(800));

            // Check if we're back on original
            self.workflow.screenshot.take_screenshot()?;
            let current_png = self.workflow.screenshot.get_image_data();
            let current_img = image::load_from_memory(current_png)?;

            let similarity = Workflow::compute_image_similarity_masked(
                original_img,
                &current_img,
                MASK_LEFT_OFFSET,
                MASK_RIGHT_OFFSET,
                MASK_TOP_OFFSET,
                MASK_BOTTOM_OFFSET,
                5, // Default sample rate
            );

            if similarity >= SAME_PAGE_THRESHOLD {
                info!("Confirmed back on original page");
                return Ok(());
            } else {
                info!("Return attempt {}/{}: Failed -> similarity to original = {:.2}% (need >= {:.1}%), retrying...", 
                      attempt, MAX_ATTEMPTS,similarity * 100.0, SAME_PAGE_THRESHOLD * 100.0);
            }
        }

        // If we couldn't get back, log warning but continue
        log::warn!(
            "Could not confirm return to original page after {} attempts - proceeding anyway",
            MAX_ATTEMPTS
        );
        Ok(())
    }

    /// Run the main loop
    pub fn run_loop(&mut self) -> Result<()> {
        info!("Starting Reader Buddy main loop");

        loop {
            match self.run_iteration() {
                Ok(_) => info!("Iteration completed successfully"),
                Err(e) => {
                    error!("Error in iteration: {}", e);
                    // Try to show error to user
                    let _ = self.workflow.set_body_text_mode();
                    let _ = self.workflow.render_text(&format!("Error: {}\n", e));
                }
            }
        }
    }
}
