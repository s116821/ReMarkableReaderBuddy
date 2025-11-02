use anyhow::Result;
use evdev::EventType as EvdevEventType;
use evdev::{Device, InputEvent};
use log::info;
use std::thread::sleep;
use std::time::Duration;

use super::DeviceModel;

// Output dimensions remain the same for both devices
const VIRTUAL_WIDTH: u32 = 768;
const VIRTUAL_HEIGHT: u32 = 1024;

pub struct Pen {
    device: Option<Device>,
    device_model: DeviceModel,
}

impl Pen {
    pub fn new(no_draw: bool) -> Self {
        let device_model = DeviceModel::detect();
        info!("Pen using device model: {}", device_model.name());

        let pen_input_device = match device_model {
            DeviceModel::Remarkable2 => "/dev/input/event1",
            DeviceModel::RemarkablePaperPro => "/dev/input/event2",
            DeviceModel::Unknown => "/dev/input/event1", // Default to RM2
        };

        let device = if no_draw {
            None
        } else {
            Some(Device::open(pen_input_device).unwrap())
        };

        Self {
            device,
            device_model,
        }
    }

    pub fn draw_line_screen(&mut self, p1: (i32, i32), p2: (i32, i32)) -> Result<()> {
        self.draw_line(self.virtual_to_input(p1), self.virtual_to_input(p2))
    }

    pub fn draw_line(&mut self, (x1, y1): (i32, i32), (x2, y2): (i32, i32)) -> Result<()> {
        let length = ((x2 as f32 - x1 as f32).powf(2.0) + (y2 as f32 - y1 as f32).powf(2.0)).sqrt();
        // 5.0 is the maximum distance between points
        let steps = (length / 5.0).ceil() as i32;
        let dx = (x2 - x1) / steps;
        let dy = (y2 - y1) / steps;

        self.pen_up()?;
        self.goto_xy((x1, y1))?;
        self.pen_down()?;

        for i in 0..steps {
            let x = x1 + dx * i;
            let y = y1 + dy * i;
            self.goto_xy((x, y))?;
        }

        self.pen_up()?;

        Ok(())
    }

    pub fn draw_bitmap(&mut self, bitmap: &[Vec<bool>]) -> Result<()> {
        let mut is_pen_down = false;
        for (y, row) in bitmap.iter().enumerate() {
            for (x, &pixel) in row.iter().enumerate() {
                if pixel {
                    if !is_pen_down {
                        self.goto_xy_virtual((x as i32, y as i32))?;
                        self.pen_down()?;
                        is_pen_down = true;
                        sleep(Duration::from_millis(1));
                    }
                    self.goto_xy_virtual((x as i32, y as i32))?;
                    self.goto_xy_virtual((x as i32 + 1, y as i32))?;
                } else if is_pen_down {
                    self.pen_up()?;
                    is_pen_down = false;
                    sleep(Duration::from_millis(1));
                }
            }
            self.pen_up()?;
            is_pen_down = false;
            sleep(Duration::from_millis(5));
        }
        Ok(())
    }

    pub fn draw_rectangle(
        &mut self,
        top_left: (i32, i32),
        bottom_right: (i32, i32),
        fill: bool,
    ) -> Result<()> {
        let (x1, y1) = top_left;
        let (x2, y2) = bottom_right;

        if fill {
            // Draw horizontal lines to fill the rectangle
            for y in y1..=y2 {
                self.draw_line_screen((x1, y), (x2, y))?;
            }
        } else {
            // Draw outline only
            self.draw_line_screen((x1, y1), (x2, y1))?; // Top
            self.draw_line_screen((x2, y1), (x2, y2))?; // Right
            self.draw_line_screen((x2, y2), (x1, y2))?; // Bottom
            self.draw_line_screen((x1, y2), (x1, y1))?; // Left
        }

        Ok(())
    }

    pub fn pen_down(&mut self) -> Result<()> {
        if let Some(device) = &mut self.device {
            device.send_events(&[
                InputEvent::new(EvdevEventType::KEY.0, 320, 1), // BTN_TOOL_PEN
                InputEvent::new(EvdevEventType::KEY.0, 330, 1), // BTN_TOUCH
                InputEvent::new(EvdevEventType::ABSOLUTE.0, 24, 2630), // ABS_PRESSURE (max pressure)
                InputEvent::new(EvdevEventType::ABSOLUTE.0, 25, 0),    // ABS_DISTANCE
                InputEvent::new(EvdevEventType::SYNCHRONIZATION.0, 0, 0), // SYN_REPORT
            ])?;
        }
        Ok(())
    }

    pub fn pen_up(&mut self) -> Result<()> {
        if let Some(device) = &mut self.device {
            device.send_events(&[
                InputEvent::new(EvdevEventType::ABSOLUTE.0, 24, 0), // ABS_PRESSURE
                InputEvent::new(EvdevEventType::ABSOLUTE.0, 25, 100), // ABS_DISTANCE
                InputEvent::new(EvdevEventType::KEY.0, 330, 0),     // BTN_TOUCH
                InputEvent::new(EvdevEventType::KEY.0, 320, 0),     // BTN_TOOL_PEN
                InputEvent::new(EvdevEventType::SYNCHRONIZATION.0, 0, 0), // SYN_REPORT
            ])?;
        }
        Ok(())
    }

    pub fn goto_xy_virtual(&mut self, point: (i32, i32)) -> Result<()> {
        self.goto_xy(self.virtual_to_input(point))
    }

    pub fn goto_xy(&mut self, (x, y): (i32, i32)) -> Result<()> {
        if let Some(device) = &mut self.device {
            device.send_events(&[
                InputEvent::new(EvdevEventType::ABSOLUTE.0, 0, x), // ABS_X
                InputEvent::new(EvdevEventType::ABSOLUTE.0, 1, y), // ABS_Y
                InputEvent::new(EvdevEventType::SYNCHRONIZATION.0, 0, 0), // SYN_REPORT
            ])?;
        }
        Ok(())
    }

    pub fn max_x_value(&self) -> i32 {
        match self.device_model {
            DeviceModel::Remarkable2 => 15725,
            DeviceModel::RemarkablePaperPro => 11180,
            DeviceModel::Unknown => 15725, // Default to RM2
        }
    }

    pub fn max_y_value(&self) -> i32 {
        match self.device_model {
            DeviceModel::Remarkable2 => 20966,
            DeviceModel::RemarkablePaperPro => 15340,
            DeviceModel::Unknown => 20966, // Default to RM2
        }
    }

    fn virtual_to_input(&self, (x, y): (i32, i32)) -> (i32, i32) {
        // Swap and normalize the coordinates
        let x_normalized = x as f32 / VIRTUAL_WIDTH as f32;
        let y_normalized = y as f32 / VIRTUAL_HEIGHT as f32;

        match self.device_model {
            DeviceModel::RemarkablePaperPro => {
                let x_input = (x_normalized * self.max_x_value() as f32) as i32;
                let y_input = (y_normalized * self.max_y_value() as f32) as i32;
                (x_input, y_input)
            }
            _ => {
                let x_input = ((1.0 - y_normalized) * self.max_y_value() as f32) as i32;
                let y_input = (x_normalized * self.max_x_value() as f32) as i32;
                (x_input, y_input)
            }
        }
    }
}
