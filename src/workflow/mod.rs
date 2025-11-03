pub mod orchestrator;
pub mod page_manager;
pub mod symbol_pool;

use anyhow::Result;
use log::info;

use crate::device::{keyboard::Keyboard, pen::Pen, screenshot::Screenshot, touch::Touch};

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

    /// Erase a region on the screen using the eraser tool
    pub fn erase_region(&mut self, region: &crate::analysis::BoundingBox) -> Result<()> {
        info!(
            "Erasing region at ({}, {}) size {}x{}",
            region.x, region.y, region.width, region.height
        );

        let top_left = (region.x, region.y);
        let bottom_right = (region.x + region.width, region.y + region.height);

        // Use the eraser tool to erase the rectangle
        self.pen.erase_rectangle(top_left, bottom_right)?;

        Ok(())
    }

    /// Draw a reference symbol at a location using bitmap rendering
    pub fn draw_symbol(&mut self, x: i32, y: i32, symbol: &str) -> Result<()> {
        info!("Drawing reference symbol '{}' at ({}, {})", symbol, x, y);

        // Convert symbol to bitmap
        let size = 20; // Symbol size in pixels
        let bitmap = symbol_pool::SymbolPool::symbol_to_bitmap(symbol, size);

        // Draw the bitmap at the specified location
        // Note: This draws the full bitmap starting at (x, y)
        // For centered placement, we'd offset by -size/2
        let offset_x = x - (size as i32 / 2);
        let offset_y = y - (size as i32 / 2);

        // Create a positioned bitmap by building a temporary full-size bitmap
        // This is not optimal but works for MVP
        let mut positioned_bitmap = vec![vec![false; 768]; 1024];
        for (dy, row) in bitmap.iter().enumerate() {
            for (dx, &pixel) in row.iter().enumerate() {
                let px = offset_x + dx as i32;
                let py = offset_y + dy as i32;
                if px >= 0 && px < 768 && py >= 0 && py < 1024 {
                    positioned_bitmap[py as usize][px as usize] = pixel;
                }
            }
        }

        self.pen.draw_bitmap(&positioned_bitmap)?;

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
