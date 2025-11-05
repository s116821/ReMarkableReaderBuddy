use anyhow::Result;
use log::{debug, info};
use std::thread::sleep;
use std::time::Duration;

use crate::device::touch::Touch;

/// Manages page navigation and creation on the reMarkable using touch gestures
pub struct PageManager;

impl PageManager {
    /// Create a new page to the right of the current page
    /// Uses swipe gesture simulation to navigate and create pages
    pub fn create_page_right(touch: &mut Touch) -> Result<()> {
        info!("Creating new page to the right via swipe gesture");

        // Strategy: Swipe left to go to next page
        // If we're at the last page, xochitl will create a new blank page

        Self::swipe_left(touch)?;
        sleep(Duration::from_millis(500)); // Wait for page transition

        Ok(())
    }

    /// Navigate to the next page (swipe left)
    pub fn next_page(touch: &mut Touch) -> Result<()> {
        info!("Navigating to next page");
        Self::swipe_left(touch)?;
        sleep(Duration::from_millis(300));
        Ok(())
    }

    /// Navigate to the previous page (swipe right)
    pub fn previous_page(touch: &mut Touch) -> Result<()> {
        info!("Navigating to previous page");
        Self::swipe_right(touch)?;
        sleep(Duration::from_millis(300));
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
