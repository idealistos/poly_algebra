use log::info;
use serde::{Deserialize, Serialize};

use crate::fint::FInt;
use crate::x_poly::XYPoly;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rectangle {
    pub x0: u32,
    pub y0: u32,
    pub x1: u32,
    pub y1: u32,
}

impl Rectangle {
    pub fn new(x0: u32, y0: u32, x1: u32, y1: u32) -> Self {
        Rectangle { x0, y0, x1, y1 }
    }

    pub fn size(&self) -> u32 {
        (self.y1 - self.y0) * (self.x1 - self.x0)
    }

    pub fn subdivide(&self) -> [Rectangle; 4] {
        let x_mid = (self.x0 + self.x1) / 2;
        let y_mid = (self.y0 + self.y1) / 2;
        [
            Rectangle::new(self.x0, self.y0, x_mid, y_mid), // top-left
            Rectangle::new(x_mid, self.y0, self.x1, y_mid), // top-right
            Rectangle::new(self.x0, y_mid, x_mid, self.y1), // bottom-left
            Rectangle::new(x_mid, y_mid, self.x1, self.y1), // bottom-right
        ]
    }
}

pub struct XYPolyDraw {
    pub xy_poly: XYPoly,
}

impl XYPolyDraw {
    pub fn new(xy_poly: XYPoly) -> Self {
        XYPolyDraw { xy_poly }
    }

    pub fn get_curve_points(
        &self,
        x_interval: FInt,
        y_interval: FInt,
        x_count: u32,
        y_count: u32,
    ) -> Vec<(u32, u32)> {
        let mut points = Vec::new();
        self.inspect_region(
            x_interval,
            y_interval,
            Rectangle::new(0, 0, x_count, y_count),
            &mut points,
            y_count,
        );
        points
    }

    fn inspect_region(
        &self,
        x_interval: FInt,
        y_interval: FInt,
        rect: Rectangle,
        points: &mut Vec<(u32, u32)>,
        y_count: u32,
    ) {
        // Evaluate polynomial at the center of the region
        let value = self.xy_poly.evaluate(x_interval, y_interval);
        if value == FInt::new(0.0) {
            if rect.size() == 1 {
                points.push((rect.x0, y_count - rect.y0 - 1));
            } else {
                // Subdivide the region
                for sub_rect in rect.subdivide() {
                    if sub_rect.size() >= 1 {
                        let (sub_x, sub_y) =
                            FInt::get_subinterval(x_interval, y_interval, rect, sub_rect);
                        self.inspect_region(sub_x, sub_y, sub_rect, points, y_count);
                    }
                }
            }
        }
    }

    pub fn get_curve_points_smoothed(
        &self,
        curve_points: Vec<(u32, u32)>,
        x_count: u32,
        y_count: u32,
    ) -> Vec<(u32, u32, Color)> {
        // Set up intensity map
        let mut intensities: HashMap<(u32, u32), f64> = HashMap::new();
        let white = Color::new(255, 255, 255);
        let red = Color::new(255, 0, 0);

        // Calculate intensities for each point and its neighborhood
        for (x, y) in curve_points {
            for dx in -5..=5 {
                for dy in -5..=5 {
                    let dist_sq = (dx * dx + dy * dy) as f64;
                    if dist_sq <= 25.0 {
                        let nx = (x as i32 + dx) as u32;
                        let ny = (y as i32 + dy) as u32;
                        if nx < x_count && ny < y_count {
                            let intensity = 255.0 * (1.0 - 0.9 * dist_sq.sqrt() / 5.0);
                            intensities
                                .entry((nx, ny))
                                .and_modify(|e| *e = (*e).max(intensity))
                                .or_insert(intensity);
                        }
                    }
                }
            }
        }

        // Sum intensities for coarse grid
        let mut intensity_sums: HashMap<(u32, u32), f64> = HashMap::new();
        for ((x, y), intensity) in intensities {
            let coarse_x = x / 4;
            let coarse_y = y / 4;
            *intensity_sums.entry((coarse_x, coarse_y)).or_insert(0.0) += intensity;
        }

        // Find maximum intensity for normalization
        let max_intensity = intensity_sums
            .values()
            .fold(0.0f64, |max, &val| max.max(val));

        // Convert to colors
        intensity_sums
            .into_iter()
            .map(|((x, y), intensity)| {
                let t = intensity / max_intensity;
                (x, y, Color::interpolate(white, red, t))
            })
            .collect()
    }

    pub fn plot_to_file(
        &self,
        x_interval: FInt,
        y_interval: FInt,
        width: u32,
        height: u32,
        filename: &str,
    ) -> std::io::Result<()> {
        info!("Finding curve points");
        // Get curve points
        let points = self.get_curve_points(x_interval, y_interval, width, height);
        info!("Found {} curve points", points.len());

        // Get smoothed points with colors
        let smoothed_points = self.get_curve_points_smoothed(points, width, height);
        info!("Generated {} smoothed points", smoothed_points.len());

        // Create BMP file
        let mut file = File::create(filename)?;
        let width = width / 4;
        let height = height / 4;

        // BMP header
        let file_size = 54 + 3 * width * height; // 54 bytes header + 3 bytes per pixel
        let header = [
            0x42,
            0x4D,                     // "BM"
            (file_size & 0xFF) as u8, // File size (LSB)
            ((file_size >> 8) & 0xFF) as u8,
            ((file_size >> 16) & 0xFF) as u8,
            ((file_size >> 24) & 0xFF) as u8, // File size (MSB)
            0x00,
            0x00, // Reserved
            0x00,
            0x00, // Reserved
            0x36,
            0x00,
            0x00,
            0x00, // Offset to pixel data
            0x28,
            0x00,
            0x00,
            0x00,                 // DIB header size
            (width & 0xFF) as u8, // Width (LSB)
            ((width >> 8) & 0xFF) as u8,
            ((width >> 16) & 0xFF) as u8,
            ((width >> 24) & 0xFF) as u8, // Width (MSB)
            (height & 0xFF) as u8,        // Height (LSB)
            ((height >> 8) & 0xFF) as u8,
            ((height >> 16) & 0xFF) as u8,
            ((height >> 24) & 0xFF) as u8, // Height (MSB)
            0x01,
            0x00, // Planes
            0x18,
            0x00, // Bits per pixel (24)
            0x00,
            0x00,
            0x00,
            0x00, // Compression
            0x00,
            0x00,
            0x00,
            0x00, // Image size
            0x13,
            0x0B,
            0x00,
            0x00, // X pixels per meter
            0x13,
            0x0B,
            0x00,
            0x00, // Y pixels per meter
            0x00,
            0x00,
            0x00,
            0x00, // Colors in color table
            0x00,
            0x00,
            0x00,
            0x00, // Important color count
        ];
        file.write_all(&header)?;

        // Create a map of colors for each pixel
        let mut colors: HashMap<(u32, u32), Color> = HashMap::new();
        for (x, y, color) in smoothed_points {
            colors.insert((x, y), color);
        }

        // Write pixel data (bottom-up)
        for y in (0..height).rev() {
            for x in 0..width {
                if let Some(color) = colors.get(&(x, y)) {
                    // Use interpolated color
                    file.write_all(&[color.b, color.g, color.r])?;
                } else {
                    // White pixel (BGR format)
                    file.write_all(&[255, 255, 255])?;
                }
            }
        }

        info!(
            "Wrote {} bytes to file {}",
            file.metadata()?.len(),
            filename
        );

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    pub fn interpolate(start: Color, end: Color, t: f64) -> Color {
        Color {
            r: (start.r as f64 + (end.r as f64 - start.r as f64) * t) as u8,
            g: (start.g as f64 + (end.g as f64 - start.g as f64) * t) as u8,
            b: (start.b as f64 + (end.b as f64 - start.b as f64) * t) as u8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::x_poly::XPoly;

    #[test]
    fn test_rectangle_subdivision() {
        let rect = Rectangle::new(0, 0, 4, 4);
        let subregions = rect.subdivide();

        assert_eq!(subregions[0], Rectangle::new(0, 0, 2, 2)); // top-left
        assert_eq!(subregions[1], Rectangle::new(2, 0, 4, 2)); // top-right
        assert_eq!(subregions[2], Rectangle::new(0, 2, 2, 4)); // bottom-left
        assert_eq!(subregions[3], Rectangle::new(2, 2, 4, 4)); // bottom-right
    }

    #[test]
    fn test_curve_points() {
        // Create circle x^2 + y^2 - 1 = 0
        let circle = XYPoly::new(vec![
            XPoly::new(vec![
                FInt::new(-1.0), // -1
                FInt::new(0.0),  // 0y
                FInt::new(1.0),  // y^2
            ]), // constant term in x
            XPoly::new(vec![FInt::new(0.0)]), // 0x
            XPoly::new(vec![FInt::new(1.0)]), // x^2
        ]);

        let drawer = XYPolyDraw::new(circle);
        let points = drawer.get_curve_points(
            FInt::new_with_bounds(-1.0, 1.0),
            FInt::new_with_bounds(-1.0, 1.0),
            4,
            4,
        );

        // Should find 12 points (all except those with x = 1, 2 and y = 1, 2)
        assert_eq!(points.len(), 12);
        for i in 0..4 {
            for j in 0..4 {
                if (i == 1 || i == 2) && (j == 1 || j == 2) {
                    assert!(!points.contains(&(i, j)));
                } else {
                    assert!(points.contains(&(i, j)));
                }
            }
        }
    }
}
