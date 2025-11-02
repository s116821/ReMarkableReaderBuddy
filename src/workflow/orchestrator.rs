use anyhow::Result;
use log::{info, error};

use crate::analysis::{circle_detector::CircleDetector, question_extractor::QuestionExtractor, QuestionContext};
use crate::llm::{openai::OpenAI, LLMEngine};
use super::Workflow;

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
        self.workflow.show_progress("Processing...")?;

        // Step 2: Capture screenshot
        let screenshot_base64 = self.workflow.capture_screenshot()?;
        self.workflow.show_progress("Analyzing...")?;

        // Step 3: Single LLM call does everything:
        // - Detect outlined region
        // - Extract question text
        // - Generate answer
        let result = self.analyze_and_answer_single_call(&screenshot_base64)?;

        match result {
            None => {
                info!("No outlined regions or questions detected");
                self.workflow.clear_progress()?;
                self.workflow.render_text("No outlined content found. Please draw an outline around content and write a question nearby.")?;
                return Ok(());
            }
            Some((question, answer)) => {
                info!("Got Q&A - Question: {} | Answer: {}", question, answer);
                self.workflow.show_progress("Rendering...")?;
                
                if let Err(e) = self.render_answer(&question, &answer) {
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
    /// Returns None if no outline/question found, or Some((question, answer))
    fn analyze_and_answer_single_call(&mut self, screenshot_base64: &str) -> Result<Option<(String, String)>> {
        info!("Sending single LLM call for analysis + answer");

        self.llm.clear_content();
        self.llm.add_text_content(
            "Look at this reMarkable tablet screenshot. The user is reading and has:\n\
             1. Drawn an outline (circle, rectangle, or any closed shape) around some content\n\
             2. Written a handwritten question nearby about that content\n\n\
             Your task:\n\
             1. Identify what content has been outlined\n\
             2. Read the handwritten question text\n\
             3. Provide a clear, helpful answer to the question based on the outlined content\n\n\
             If you find an outline and question, respond EXACTLY in this format:\n\
             QUESTION: [the extracted question text]\n\
             ---\n\
             ANSWER: [your answer]\n\n\
             If you cannot find a clear outline or question, respond with just:\n\
             NONE\n\n\
             Note: Process only ONE outline-question pair (the most prominent one if multiple exist). \
             Keep the answer concise and focused on what was asked."
        );
        self.llm.add_image_content(screenshot_base64);

        let response = self.llm.execute()?;
        info!("LLM Response: {}", response);

        // Parse the response
        if response.trim().to_uppercase().starts_with("NONE") {
            return Ok(None);
        }

        // TODO: Implement proper parsing to extract QUESTION and ANSWER sections
        // For now, use simple string splitting
        let parts: Vec<&str> = response.split("---").collect();
        if parts.len() >= 2 {
            let question_part = parts[0].trim().strip_prefix("QUESTION:").unwrap_or(parts[0]).trim();
            let answer_part = parts[1].trim().strip_prefix("ANSWER:").unwrap_or(parts[1]).trim();
            Ok(Some((question_part.to_string(), answer_part.to_string())))
        } else {
            // Fallback: treat whole response as answer with generic question
            Ok(Some(("What does this mean?".to_string(), response)))
        }
    }

    /// Render the answer on a new page
    fn render_answer(&mut self, question: &str, answer: &str) -> Result<()> {
        info!("Rendering Q&A");

        // TODO: Create new page after current
        // TODO: Navigate to new page
        
        // Render on current page for now
        self.workflow.clear_progress()?;
        
        let formatted_output = format!(
            "Q: {}\n\nA: {}\n\n---\n\n",
            question,
            answer
        );
        
        self.workflow.render_text(&formatted_output)?;

        // TODO: Erase the original question (need bounding box from LLM)
        // TODO: Draw reference symbol on both pages
        
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

