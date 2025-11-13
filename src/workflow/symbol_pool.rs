use anyhow::Result;
use log::debug;
use std::fs;
use std::path::Path;

/// Pool of reference symbols for marking question-answer pairs
/// Uses circled numbers: ①②③④⑤⑥⑦⑧⑨⑩
pub struct SymbolPool {
    current_index: usize,
    symbols: Vec<String>,
    state_file: String,
}

impl SymbolPool {
    /// Create a new symbol pool
    pub fn new() -> Self {
        let symbols = vec![
            "①".to_string(),
            "②".to_string(),
            "③".to_string(),
            "④".to_string(),
            "⑤".to_string(),
            "⑥".to_string(),
            "⑦".to_string(),
            "⑧".to_string(),
            "⑨".to_string(),
            "⑩".to_string(),
        ];

        let state_file = "/home/root/.reader-buddy-symbol-state".to_string();

        Self {
            current_index: 0,
            symbols,
            state_file,
        }
    }

    /// Load the symbol pool state from disk
    /// Returns the last used index
    pub fn load(&mut self) -> Result<()> {
        if Path::new(&self.state_file).exists() {
            let content = fs::read_to_string(&self.state_file)?;
            if let Ok(index) = content.trim().parse::<usize>() {
                self.current_index = index % self.symbols.len();
                debug!("Loaded symbol state: index {}", self.current_index);
            }
        }
        Ok(())
    }

    /// Save the current symbol pool state to disk
    fn save(&self) -> Result<()> {
        fs::write(&self.state_file, self.current_index.to_string())?;
        debug!("Saved symbol state: index {}", self.current_index);
        Ok(())
    }

    /// Get the next symbol and advance the pool
    pub fn next_symbol(&mut self) -> Result<String> {
        let symbol = self.symbols[self.current_index].clone();
        debug!("Using symbol: {} (index {})", symbol, self.current_index);

        // Advance to next symbol
        self.current_index = (self.current_index + 1) % self.symbols.len();

        // Save state for persistence across app restarts
        self.save()?;

        Ok(symbol)
    }

    /// Get the current symbol without advancing
    pub fn current_symbol(&self) -> String {
        self.symbols[self.current_index].clone()
    }

    /// Convert symbol to bitmap for rendering using SVG
    pub fn symbol_to_bitmap(symbol: &str, size: u32) -> Vec<Vec<bool>> {
        use crate::util::svg_to_bitmap;
        
        debug!("Converting symbol '{}' to {}x{} bitmap using SVG", symbol, size, size);

        // Create SVG with the circled number symbol
        // Using large font size and centered positioning
        let font_size = (size as f32 * 0.8) as u32;
        let x = size / 2;
        let y = (size as f32 * 0.75) as u32; // Adjust vertical centering for better appearance
        
        let svg_data = format!(
            r#"<svg width='{size}' height='{size}' xmlns='http://www.w3.org/2000/svg'>
                <text x='{x}' y='{y}' font-family='Noto Sans, DejaVu Sans, sans-serif' 
                      font-size='{font_size}' fill='black' text-anchor='middle'>{symbol}</text>
            </svg>"#
        );

        // Try to render the SVG, fall back to simple circle on error
        match svg_to_bitmap(&svg_data, size, size) {
            Ok(bitmap) => bitmap,
            Err(e) => {
                debug!("Failed to render SVG symbol: {}, using fallback circle", e);
                Self::fallback_circle_bitmap(size)
            }
        }
    }

    /// Fallback simple circle bitmap if SVG rendering fails
    fn fallback_circle_bitmap(size: u32) -> Vec<Vec<bool>> {
        let mut bitmap = vec![vec![false; size as usize]; size as usize];
        let center = size as i32 / 2;
        let radius = (size as f32 * 0.4) as i32;

        for y in 0..size as i32 {
            for x in 0..size as i32 {
                let dx = x - center;
                let dy = y - center;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= radius * radius && dist_sq >= (radius - 2) * (radius - 2) {
                    bitmap[y as usize][x as usize] = true;
                }
            }
        }

        bitmap
    }
}

impl Default for SymbolPool {
    fn default() -> Self {
        Self::new()
    }
}
