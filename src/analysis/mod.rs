use serde::{Deserialize, Serialize};

/// Represents a region of interest on the screen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
