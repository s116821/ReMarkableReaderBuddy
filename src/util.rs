use anyhow::Result;
use image::GrayImage;
use log::{debug, info};
use resvg::render;
use resvg::tiny_skia::Pixmap;
use resvg::usvg;
use resvg::usvg::{fontdb, Options, Tree};
use std::sync::Arc;

/// Convert SVG string to bitmap representation
pub fn svg_to_bitmap(svg_data: &str, width: u32, height: u32) -> Result<Vec<Vec<bool>>> {
    let mut opt = Options::default();
    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();

    opt.fontdb = Arc::new(fontdb);

    let tree = match Tree::from_str(svg_data, &opt) {
        Ok(tree) => tree,
        Err(e) => {
            info!("Error parsing SVG: {}. Using fallback SVG.", e);
            let fallback_svg = format!(
                r#"<svg width='{width}' height='{height}' xmlns='http://www.w3.org/2000/svg'><circle cx='{}' cy='{}' r='10' fill='black'/></svg>"#,
                width / 2, height / 2
            );
            Tree::from_str(&fallback_svg, &opt)?
        }
    };

    let mut pixmap = Pixmap::new(width, height).unwrap();
    render(&tree, usvg::Transform::default(), &mut pixmap.as_mut());

    let bitmap = pixmap
        .pixels()
        .chunks(width as usize)
        .map(|row| row.iter().map(|p| p.alpha() > 128).collect())
        .collect();

    Ok(bitmap)
}

/// Write bitmap to PNG file for debugging
pub fn write_bitmap_to_file(bitmap: &[Vec<bool>], filename: &str) -> Result<()> {
    let width = bitmap[0].len();
    let height = bitmap.len();
    let mut img = GrayImage::new(width as u32, height as u32);

    for (y, row) in bitmap.iter().enumerate() {
        for (x, &pixel) in row.iter().enumerate() {
            img.put_pixel(x as u32, y as u32, image::Luma([if pixel { 0 } else { 255 }]));
        }
    }

    img.save(filename)?;
    debug!("Bitmap saved to {}", filename);
    Ok(())
}

