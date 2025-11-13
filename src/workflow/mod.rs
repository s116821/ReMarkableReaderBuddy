pub mod orchestrator;
pub mod page_manager;
pub mod symbol_pool;
pub mod xochitl_integration;

use anyhow::Result;
use log::info;

use crate::device::{keyboard::Keyboard, pen::Pen, screenshot::Screenshot, touch::Touch};

/// Main workflow coordinator
pub struct Workflow {
    screenshot: Screenshot,
    pen: Pen,
    keyboard: Keyboard,
    touch: Touch,
    debug_dump: bool,
    iteration_count: u32,
}

impl Workflow {
    pub fn new(no_draw: bool, trigger_corner: crate::device::touch::TriggerCorner, debug_dump: bool) -> Result<Self> {
        Ok(Self {
            screenshot: Screenshot::new()?,
            pen: Pen::new(no_draw),
            keyboard: Keyboard::new(no_draw, false),
            touch: Touch::new(no_draw, trigger_corner),
            debug_dump,
            iteration_count: 0,
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
        self.screenshot.base64()
    }

    /// Take a screenshot and return both base64 and raw PNG data
    pub fn capture_screenshot_with_data(&mut self) -> Result<(String, Vec<u8>)> {
        info!("Capturing screenshot...");
        self.screenshot.take_screenshot()?;
        let base64 = self.screenshot.base64()?;
        let png_data = self.screenshot.get_image_data().to_vec();
        
        // Debug dump if enabled
        if self.debug_dump {
            self.iteration_count += 1;
            let filename = format!("/tmp/reader-buddy-screenshot-{:03}.png", self.iteration_count);
            if let Err(e) = self.screenshot.save_image(&filename) {
                log::warn!("Failed to save debug screenshot: {}", e);
            } else {
                log::debug!("Saved debug screenshot to {}", filename);
            }
        }
        
        Ok((base64, png_data))
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

    /// Smart erase that only erases detected ink pixels within the region
    pub fn erase_region_smart(&mut self, region: &crate::analysis::BoundingBox, screenshot_data: &[u8]) -> Result<()> {
        use image::{GenericImageView, Rgba, RgbaImage};
        
        info!(
            "Smart erasing region at ({}, {}) size {}x{}",
            region.x, region.y, region.width, region.height
        );

        // Load the screenshot
        let img = image::load_from_memory(screenshot_data)?;
        let gray_img = img.to_luma8();

        // Define ink detection threshold (darker pixels are ink)
        const INK_THRESHOLD: u8 = 200; // Pixels darker than this are considered ink
        const MARGIN: i32 = 2; // Add margin around detected ink

        // Scan the region and identify rows with ink
        let mut rows_with_ink = Vec::new();
        for y in region.y..(region.y + region.height).min(1024) {
            if y < 0 || y >= gray_img.height() as i32 {
                continue;
            }
            
            let mut has_ink = false;
            for x in region.x..(region.x + region.width).min(768) {
                if x < 0 || x >= gray_img.width() as i32 {
                    continue;
                }
                
                let pixel = gray_img.get_pixel(x as u32, y as u32);
                if pixel[0] < INK_THRESHOLD {
                    has_ink = true;
                    break;
                }
            }
            
            if has_ink {
                rows_with_ink.push(y);
            }
        }

        debug!("Found {} rows with ink out of {} total rows", rows_with_ink.len(), region.height);

        // Debug dump if enabled - show erase mask overlay
        if self.debug_dump {
            let mut debug_img = img.to_rgba8();
            // Draw red box around the region
            for x in region.x.max(0)..((region.x + region.width).min(768)) {
                if x >= 0 && x < debug_img.width() as i32 {
                    if region.y >= 0 && region.y < debug_img.height() as i32 {
                        debug_img.put_pixel(x as u32, region.y as u32, Rgba([255, 0, 0, 255]));
                    }
                    let bottom_y = (region.y + region.height - 1).min(debug_img.height() as i32 - 1);
                    if bottom_y >= 0 && bottom_y < debug_img.height() as i32 {
                        debug_img.put_pixel(x as u32, bottom_y as u32, Rgba([255, 0, 0, 255]));
                    }
                }
            }
            // Highlight rows to be erased in yellow
            for &y in &rows_with_ink {
                for x in region.x.max(0)..((region.x + region.width).min(768)) {
                    if x >= 0 && x < debug_img.width() as i32 && y >= 0 && y < debug_img.height() as i32 {
                        debug_img.put_pixel(x as u32, y as u32, Rgba([255, 255, 0, 128]));
                    }
                }
            }
            let filename = format!("/tmp/reader-buddy-erase-mask-{:03}.png", self.iteration_count);
            if let Err(e) = debug_img.save(&filename) {
                log::warn!("Failed to save debug erase mask: {}", e);
            } else {
                log::debug!("Saved debug erase mask to {}", filename);
            }
        }

        // Erase rows with ink (with margin)
        for &y in &rows_with_ink {
            let erase_y_start = (y - MARGIN).max(region.y).max(0);
            let erase_y_end = (y + MARGIN + 1).min(region.y + region.height).min(1024);
            
            for erase_y in erase_y_start..erase_y_end {
                let top_left = (region.x, erase_y);
                let bottom_right = ((region.x + region.width).min(768), erase_y + 1);
                self.pen.erase_rectangle(top_left, bottom_right)?;
            }
        }

        Ok(())
    }

    /// Draw a reference symbol at a location using bitmap rendering
    pub fn draw_symbol(&mut self, x: i32, y: i32, symbol: &str) -> Result<()> {
        info!("Drawing reference symbol '{}' at ({}, {})", symbol, x, y);

        // Convert symbol to bitmap - larger size for better visibility
        let size = 40; // Symbol size in pixels (increased from 20)
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
                if (0..768).contains(&px) && (0..1024).contains(&py) {
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

    /// Navigate to the next page
    pub fn navigate_to_next_page(&mut self) -> Result<()> {
        page_manager::PageManager::next_page(&mut self.touch)?;
        Ok(())
    }

    /// Navigate back to the previous page
    pub fn navigate_to_previous_page(&mut self) -> Result<()> {
        page_manager::PageManager::previous_page(&mut self.touch)?;
        Ok(())
    }

    /// Check if the next page is a Reader Buddy answer page
    /// Does this by navigating to the next page, taking a screenshot, and checking for the marker text
    pub fn check_if_next_page_is_answer_page(&mut self) -> Result<bool> {
        use image::GenericImageView;
        
        info!("Checking if next page is an answer page");
        
        // Navigate to next page
        self.navigate_to_next_page()?;
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        // Take screenshot
        self.screenshot.take_screenshot()?;
        let png_data = self.screenshot.get_image_data();
        
        // Load image and check for dark pixels in the header region where "Reader Buddy Answers" would be
        let img = match image::load_from_memory(png_data) {
            Ok(img) => img,
            Err(e) => {
                log::warn!("Failed to load screenshot for answer page check: {}", e);
                // Navigate back on error
                self.navigate_to_previous_page()?;
                return Ok(false);
            }
        };
        
        let gray_img = img.to_luma8();
        
        // Check the top 100 pixels of the page for dark content (text)
        // If there's significant dark content in the header area, it's likely our answer page
        const HEADER_HEIGHT: u32 = 100;
        const INK_THRESHOLD: u8 = 200;
        const MIN_INK_PIXELS: u32 = 50; // Minimum number of dark pixels to consider it has text
        
        let mut ink_pixel_count = 0;
        for y in 0..HEADER_HEIGHT.min(gray_img.height()) {
            for x in 0..gray_img.width() {
                let pixel = gray_img.get_pixel(x, y);
                if pixel[0] < INK_THRESHOLD {
                    ink_pixel_count += 1;
                    if ink_pixel_count >= MIN_INK_PIXELS {
                        // Found enough ink, likely an answer page
                        log::debug!("Detected answer page marker (found {} ink pixels)", ink_pixel_count);
                        // Navigate back to original page
                        self.navigate_to_previous_page()?;
                        return Ok(true);
                    }
                }
            }
        }
        
        log::debug!("No answer page marker found (only {} ink pixels in header)", ink_pixel_count);
        // Navigate back to original page
        self.navigate_to_previous_page()?;
        Ok(false)
    }
}
