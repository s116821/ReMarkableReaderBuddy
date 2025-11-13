use anyhow::Result;
use log::{debug, info, warn};
use std::thread::sleep;
use std::time::Duration;

use crate::device::touch::Touch;
use super::xochitl_integration::{XochitlIntegration, NavigationDirection};

/// Manages page navigation and creation on the reMarkable using xochitl integration
pub struct PageManager;

impl PageManager {
    /// Create a new page to the right of the current page
    /// Uses xochitl's native menu system to properly insert a new page
    pub fn create_page_right(touch: &mut Touch) -> Result<()> {
        info!("Creating new page via xochitl menu system");

        // Use xochitl integration to create page via native UI
        match XochitlIntegration::create_page_after_current(touch) {
            Ok(_) => {
                info!("Page created successfully via xochitl menu");
                // Navigate to the newly created page
                sleep(Duration::from_millis(300));
                XochitlIntegration::navigate_to_page(touch, NavigationDirection::Next)?;
                Ok(())
            }
            Err(e) => {
                warn!("Failed to create page via xochitl menu: {}", e);
                Err(e)
            }
        }
    }

    /// Navigate to the next page (swipe left)
    pub fn next_page(touch: &mut Touch) -> Result<()> {
        info!("Navigating to next page");
        XochitlIntegration::navigate_to_page(touch, NavigationDirection::Next)?;
        Ok(())
    }

    /// Navigate to the previous page (swipe right)
    pub fn previous_page(touch: &mut Touch) -> Result<()> {
        info!("Navigating to previous page");
        XochitlIntegration::navigate_to_page(touch, NavigationDirection::Previous)?;
        Ok(())
    }

    /// Simulate a left swipe (next page)
    /// Swipes from right edge to left
    fn swipe_left(touch: &mut Touch) -> Result<()> {
        debug!("Simulating left swipe");

        // Start from right edge, middle height
        let start_x = 700;
        let start_y = 512;

        // End at left side, same height
        let end_x = 100;
        let _end_y = 512;

        // Perform swipe with multiple touch points for smooth gesture
        touch.touch_start((start_x, start_y))?;
        sleep(Duration::from_millis(50));

        // Interpolate between start and end
        let steps = 10;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let x = start_x - ((start_x - end_x) as f32 * t) as i32;
            let y = start_y;
            touch.goto_xy((x, y))?;
            sleep(Duration::from_millis(10));
        }

        touch.touch_stop()?;
        Ok(())
    }

    /// Simulate a right swipe (previous page)
    /// Swipes from left edge to right
    fn swipe_right(touch: &mut Touch) -> Result<()> {
        debug!("Simulating right swipe");

        // Start from left edge, middle height
        let start_x = 100;
        let start_y = 512;

        // End at right side, same height
        let end_x = 700;
        let _end_y = 512;

        // Perform swipe with multiple touch points for smooth gesture
        touch.touch_start((start_x, start_y))?;
        sleep(Duration::from_millis(50));

        // Interpolate between start and end
        let steps = 10;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let x = start_x + ((end_x - start_x) as f32 * t) as i32;
            let y = start_y;
            touch.goto_xy((x, y))?;
            sleep(Duration::from_millis(10));
        }

        touch.touch_stop()?;
        Ok(())
    }
}
