pub mod orchestrator;
pub mod page_manager;
pub mod renderer;
pub mod symbol_pool;

use anyhow::Result;
use log::{debug, info};

use crate::analysis::QuestionContext;
use crate::device::{keyboard::Keyboard, pen::Pen, screenshot::Screenshot, touch::Touch};
use crate::llm::LLMEngine;

/// Main workflow coordinator
pub struct Workflow {
    screenshot: Screenshot,
    pen: Pen,
    keyboard: Keyboard,
    touch: Touch,
}

impl Workflow {
    pub fn new(no_draw: bool, trigger_corner: crate::device::touch::TriggerCorner) -> Result<Self> {
        Ok(Self {
            screenshot: Screenshot::new()?,
            pen: Pen::new(no_draw),
            keyboard: Keyboard::new(no_draw, false),
            touch: Touch::new(no_draw, trigger_corner),
        })
    }

    /// Wait for user to trigger the workflow (touch in corner)
    pub fn wait_for_trigger(&mut self) -> Result<()> {
        info!("Waiting for trigger...");
        self.touch.wait_for_trigger()?;
        self.touch.tap_middle_bottom()?;
        Ok(())
    }

    /// Take a screenshot and return the base64-encoded image
    pub fn capture_screenshot(&mut self) -> Result<String> {
        info!("Capturing screenshot...");
        self.screenshot.take_screenshot()?;
        Ok(self.screenshot.base64()?)
    }

    /// Show progress indicator to user
    pub fn show_progress(&mut self, message: &str) -> Result<()> {
        self.keyboard.progress(message)?;
        Ok(())
    }

    /// Clear progress indicator
    pub fn clear_progress(&mut self) -> Result<()> {
        self.keyboard.progress_end()?;
        Ok(())
    }

    /// Erase a region on the screen (draw white rectangle)
    pub fn erase_region(&mut self, region: &crate::analysis::BoundingBox) -> Result<()> {
        info!(
            "Erasing region at ({}, {}) size {}x{}",
            region.x, region.y, region.width, region.height
        );

        // Draw a filled white rectangle to erase the region
        let top_left = (region.x, region.y);
        let bottom_right = (region.x + region.width, region.y + region.height);

        // TODO: Need to implement white pen color or use fill approach
        // For now, this will draw in the default pen color
        self.pen.draw_rectangle(top_left, bottom_right, true)?;

        Ok(())
    }

    /// Draw a reference symbol at a location
    /// TODO: Implement symbol pool with cycling (①②③④⑤⑥⑦⑧⑨⑩)
    /// For now, draws a simple geometric marker
    pub fn draw_symbol(&mut self, x: i32, y: i32, symbol: &str) -> Result<()> {
        info!("Drawing reference symbol '{}' at ({}, {})", symbol, x, y);

        // TODO: Implement proper symbol rendering:
        // - Create a pool of reference markers: ①②③④⑤⑥⑦⑧⑨⑩
        // - Track which symbol was used (persist across triggers)
        // - Cycle through the pool
        // - Render as small, clear glyphs

        // Temporary implementation: Draw a small circle as a marker
        let radius = 10;
        for angle in (0..360).step_by(10) {
            let rad = (angle as f32).to_radians();
            let x1 = x + (radius as f32 * rad.cos()) as i32;
            let y1 = y + (radius as f32 * rad.sin()) as i32;
            let next_angle = angle + 10;
            let next_rad = (next_angle as f32).to_radians();
            let x2 = x + (radius as f32 * next_rad.cos()) as i32;
            let y2 = y + (radius as f32 * next_rad.sin()) as i32;
            self.pen.draw_line_screen((x1, y1), (x2, y2))?;
        }

        Ok(())
    }

    /// Render text on the screen using the keyboard
    pub fn render_text(&mut self, text: &str) -> Result<()> {
        info!("Rendering text: {}", text);
        self.keyboard.key_cmd_body()?;
        self.keyboard.string_to_keypresses(text)?;
        self.keyboard.string_to_keypresses("\n\n")?;
        Ok(())
    }

    /// Get access to the keyboard for direct manipulation
    pub fn get_keyboard_mut(&mut self) -> &mut Keyboard {
        &mut self.keyboard
    }

    /// Get access to the pen for direct manipulation
    pub fn get_pen_mut(&mut self) -> &mut Pen {
        &mut self.pen
    }

    /// Get access to the touch device for direct manipulation
    pub fn get_touch_mut(&mut self) -> &mut Touch {
        &mut self.touch
    }

    /// Create a new page to the right of the current page
    pub fn create_new_page_right(&mut self) -> Result<()> {
        page_manager::PageManager::create_page_right(&mut self.touch)?;
        Ok(())
    }

    /// Navigate back to the previous page
    pub fn navigate_to_previous_page(&mut self) -> Result<()> {
        page_manager::PageManager::previous_page(&mut self.touch)?;
        Ok(())
    }
}
