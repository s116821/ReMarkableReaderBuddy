use anyhow::Result;
use log::{debug, error, info};

use super::{symbol_pool::SymbolPool, Workflow};
use crate::analysis::BoundingBox;
use crate::llm::{openai::OpenAI, LLMEngine};

/// Result from LLM analysis containing question, answer, and bounding boxes
struct AnalysisResult {
    question: String,
    answer: String,
    question_box: Option<BoundingBox>,
    _outline_box: Option<BoundingBox>,
    screenshot_data: Vec<u8>, // PNG data for downstream processing
}

/// High-level orchestrator for the complete workflow
pub struct Orchestrator {
    workflow: Workflow,
    llm: OpenAI,
    symbol_pool: SymbolPool,
}

impl Orchestrator {
    pub fn new(workflow: Workflow, llm: OpenAI) -> Self {
        let mut symbol_pool = SymbolPool::new();
        // Load previous state (if any)
        let _ = symbol_pool.load();

        Self {
            workflow,
            llm,
            symbol_pool,
        }
    }

    /// Run one complete iteration of the reader buddy workflow
    /// NOTE: v0.1 processes ONE outline-question pair per trigger
    pub fn run_iteration(&mut self) -> Result<()> {
        info!("=== Starting Reader Buddy Iteration ===");

        // Step 1: Wait for trigger
        self.workflow.wait_for_trigger()?;
        self.workflow.show_progress("Processing...")?;

        // Step 2: Capture screenshot
        let (screenshot_base64, screenshot_png_data) = self.workflow.capture_screenshot_with_data()?;
        self.workflow.show_progress("Analyzing...")?;

        // Step 3: Single LLM call does everything:
        // - Detect outlined region
        // - Extract question text
        // - Generate answer
        let result = self.analyze_and_answer_single_call(&screenshot_base64, screenshot_png_data)?;

        match result {
            None => {
                info!("No outlined regions or questions detected");
                self.workflow.clear_progress()?;
                self.workflow.render_text("No outlined content found. Please draw an outline around content and write a question nearby.")?;
                return Ok(());
            }
            Some(result) => {
                info!(
                    "Got Q&A - Question: {} | Answer: {}",
                    result.question, result.answer
                );
                self.workflow.show_progress("Rendering...")?;

                if let Err(e) = self.render_answer(&result) {
                    error!("Error rendering answer: {}", e);
                    self.workflow.clear_progress()?;
                    self.workflow.render_text(&format!("Error: {}", e))?;
                }
            }
        }

        self.workflow.clear_progress()?;
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
                question_box: None,
                _outline_box: None,
                screenshot_data: screenshot_png_data,
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
            question_box,
            _outline_box: outline_box,
            screenshot_data: screenshot_png_data,
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

    /// Render the answer on a new page with proper cleanup
    fn render_answer(&mut self, result: &AnalysisResult) -> Result<()> {
        info!("Rendering Q&A on new page");

        // Get the next symbol from the pool
        let symbol = self.symbol_pool.next_symbol()?;
        info!("Using reference symbol: {}", symbol);

        // Step 1: Erase question text if we have its location
        // IMPORTANT: Only erase question, preserve outline
        if let Some(question_box) = &result.question_box {
            // Clamp coordinates to virtual workspace (768x1024)
            let clamped_x = question_box.x.max(0).min(768 - question_box.width.max(1));
            let clamped_y = question_box.y.max(0).min(1024 - question_box.height.max(1));
            let clamped_width = question_box.width.max(1).min(768 - clamped_x);
            let clamped_height = question_box.height.max(1).min(1024 - clamped_y);
            
            debug!(
                "Original question box: ({}, {}) size {}x{} -> Clamped: ({}, {}) size {}x{}",
                question_box.x, question_box.y, question_box.width, question_box.height,
                clamped_x, clamped_y, clamped_width, clamped_height
            );
            
            info!(
                "Erasing question at ({}, {}) size {}x{}",
                clamped_x, clamped_y, clamped_width, clamped_height
            );
            
            let clamped_box = BoundingBox {
                x: clamped_x,
                y: clamped_y,
                width: clamped_width,
                height: clamped_height,
            };
            
            self.workflow.show_progress("Erasing question...")?;
            self.workflow.erase_region_smart(&clamped_box, &result.screenshot_data)?;
        } else {
            debug!("No question bounding box provided, skipping erasure");
        }

        // Step 2: Draw symbol on current page (where question was)
        self.workflow.show_progress("Marking original...")?;
        let symbol_x = if let Some(qbox) = &result.question_box {
            (qbox.x + qbox.width / 2).max(0).min(767)
        } else {
            50 // Default location if no box
        };
        let symbol_y = if let Some(qbox) = &result.question_box {
            (qbox.y + qbox.height / 2).max(0).min(1023)
        } else {
            950 // Default location if no box
        };
        debug!("Symbol placement at virtual coordinates: ({}, {})", symbol_x, symbol_y);
        self.draw_symbol_on_page(&symbol, symbol_x, symbol_y)?;
        
        // Allow e-ink display to settle before navigation
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Step 3: Check if answer page already exists, or create new one
        self.workflow.show_progress("Checking for answer page...")?;
        let needs_new_page = !self.workflow.check_if_next_page_is_answer_page()?;
        
        if needs_new_page {
            info!("No answer page found, creating new one");
            self.workflow.show_progress("Creating page...")?;
            self.workflow.create_new_page_right()?;
            
            // Wait for page to be fully created and active
            std::thread::sleep(std::time::Duration::from_millis(1000));
            
            // Add header to mark this as an answer page
            self.workflow.clear_progress()?;
            self.workflow.get_keyboard_mut().key_cmd_body()?;
            std::thread::sleep(std::time::Duration::from_millis(200));
            self.workflow.render_text("=== Reader Buddy Answers ===\n\n")?;
        } else {
            info!("Reusing existing answer page");
            self.workflow.show_progress("Using existing page...")?;
            self.workflow.navigate_to_next_page()?;
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        
        // Step 4: Render Q&A on answer page with matching symbol
        self.workflow.clear_progress()?;
        
        // Ensure keyboard is in body text mode before typing
        self.workflow.get_keyboard_mut().key_cmd_body()?;
        std::thread::sleep(std::time::Duration::from_millis(200));

        let formatted_output = format!(
            "{} Q: {}\n\nA: {}\n\n---\n\n",
            symbol, result.question, result.answer
        );

        self.workflow.render_text(&formatted_output)?;

        // Step 5: Navigate back to original page to preserve reading context
        self.workflow.navigate_to_previous_page()?;

        info!("Q&A rendered successfully with symbol {}", symbol);
        Ok(())
    }

    /// Draw a symbol on the current page
    fn draw_symbol_on_page(&mut self, symbol: &str, x: i32, y: i32) -> Result<()> {
        info!("Drawing symbol {} at ({}, {})", symbol, x, y);

        // Use the workflow's draw_symbol method which converts to bitmap and draws
        self.workflow.draw_symbol(x, y, symbol)?;

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
                    let _ = self.workflow.render_text(&format!("Error: {}", e));
                }
            }
        }
    }
}
