use anyhow::Result;
use log::debug;
use std::thread::sleep;
use std::time::Duration;

use crate::device::touch::Touch;

/// Integration with xochitl's navigation features
pub struct XochitlIntegration;

impl XochitlIntegration {
    /// Navigate to a specific page by swiping
    pub fn navigate_to_page(touch: &mut Touch, direction: NavigationDirection) -> Result<()> {
        match direction {
            NavigationDirection::Next => Self::swipe_left(touch)?,
            NavigationDirection::Previous => Self::swipe_right(touch)?,
        }
        sleep(Duration::from_millis(500)); // Wait for page transition
        Ok(())
    }

    /// Swipe left (go to next page)
    fn swipe_left(touch: &mut Touch) -> Result<()> {
        debug!("Swiping left to next page");
        let start_x = 700;
        let start_y = 512;
        let end_x = 100;

        touch.touch_start((start_x, start_y))?;
        sleep(Duration::from_millis(50));

        // Interpolate between start and end
        let steps = 15; // More steps for smoother gesture
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let x = start_x - ((start_x - end_x) as f32 * t) as i32;
            touch.goto_xy((x, start_y))?;
            sleep(Duration::from_millis(10));
        }

        touch.touch_stop()?;
        Ok(())
    }

    /// Swipe right (go to previous page)
    fn swipe_right(touch: &mut Touch) -> Result<()> {
        debug!("Swiping right to previous page");
        let start_x = 100;
        let start_y = 512;
        let end_x = 700;

        touch.touch_start((start_x, start_y))?;
        sleep(Duration::from_millis(50));

        // Interpolate between start and end
        let steps = 15;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let x = start_x + ((end_x - start_x) as f32 * t) as i32;
            touch.goto_xy((x, start_y))?;
            sleep(Duration::from_millis(10));
        }

        touch.touch_stop()?;
        Ok(())
    }
}

/// Direction for page navigation
pub enum NavigationDirection {
    Next,
    Previous,
}
