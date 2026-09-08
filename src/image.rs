//! Image processing and 1-bit monochrome dithering for thermal receipt printers.
//!
//! Thermal receipt printers operate strictly with 1-bit binary pixels (either a heating
//! element is energized or it is not). This module converts multi-channel color or
//! grayscale images (PNG, JPEG, etc.) into 1-bit monochrome raster data suitable for
//! [`crate::command::ImageData`].

use std::path::Path;

use crate::command::ImageData;
use crate::error::Result;

/// Dithering algorithm to convert continuous-tone grayscale images to 1-bit monochrome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DitherMode {
    /// Floyd-Steinberg error diffusion.
    ///
    /// Distributes quantization errors to neighboring unvisited pixels, producing
    /// smooth photographic gradients and eliminating harsh color banding on thermal paper.
    #[default]
    FloydSteinberg,

    /// Hard threshold cutoff (0..=255).
    ///
    /// Pixels with luminance strictly below this threshold become black; all others become white.
    /// Best for crisp monochrome line art, barcodes, and high-contrast vector logos.
    Threshold(u8),
}

/// Dithers a [`image::DynamicImage`] into a 1-bit packed [`ImageData`].
///
/// If `max_width` is provided and the image exceeds this width, it is smoothly downscaled
/// preserving its aspect ratio.
#[must_use]
pub fn dither_dynamic_image(
    img: &image::DynamicImage,
    max_width: Option<u32>,
    mode: DitherMode,
) -> ImageData {
    let resized = if let Some(max_w) = max_width {
        if img.width() > max_w && max_w > 0 {
            let new_h = ((img.height() as u64 * max_w as u64) / img.width() as u64).max(1) as u32;
            img.resize(max_w, new_h, image::imageops::FilterType::Lanczos3)
        } else {
            img.clone()
        }
    } else {
        img.clone()
    };

    let rgba = resized.to_rgba8();
    let (width, height) = rgba.dimensions();

    if width == 0 || height == 0 {
        return ImageData {
            width: 0,
            height: 0,
            pixels: Vec::new(),
        };
    }

    let w = width as usize;
    let h = height as usize;

    // Convert to floating-point luminance buffer using ITU-R BT.709 relative luminance formula.
    // Transparent pixels (alpha < 128) are treated as white paper (255.0).
    let mut lum: Vec<f32> = Vec::with_capacity(w * h);
    for y in 0..height {
        for x in 0..width {
            let pixel = rgba.get_pixel(x, y);
            if pixel[3] < 128 {
                lum.push(255.0);
            } else {
                let y_val = 0.2126 * f32::from(pixel[0])
                    + 0.7152 * f32::from(pixel[1])
                    + 0.0722 * f32::from(pixel[2]);
                lum.push(y_val);
            }
        }
    }

    let row_bytes = w.div_ceil(8);
    let mut pixels = vec![0u8; row_bytes * h];

    match mode {
        DitherMode::FloydSteinberg => {
            for y in 0..h {
                for x in 0..w {
                    let idx = y * w + x;
                    let old_val = lum[idx].clamp(0.0, 255.0);
                    let is_black = old_val < 128.0;
                    let new_val = if is_black { 0.0 } else { 255.0 };
                    let error = old_val - new_val;

                    if is_black {
                        let byte_idx = y * row_bytes + (x / 8);
                        let bit_mask = 0x80 >> (x % 8);
                        pixels[byte_idx] |= bit_mask;
                    }

                    // Diffuse quantization error to neighbors
                    if x + 1 < w {
                        lum[idx + 1] += error * (7.0 / 16.0);
                    }
                    if y + 1 < h {
                        if x > 0 {
                            lum[(y + 1) * w + (x - 1)] += error * (3.0 / 16.0);
                        }
                        lum[(y + 1) * w + x] += error * (5.0 / 16.0);
                        if x + 1 < w {
                            lum[(y + 1) * w + (x + 1)] += error * (1.0 / 16.0);
                        }
                    }
                }
            }
        }

        DitherMode::Threshold(cutoff) => {
            let cutoff_f = f32::from(cutoff);
            for y in 0..h {
                for x in 0..w {
                    let idx = y * w + x;
                    if lum[idx] < cutoff_f {
                        let byte_idx = y * row_bytes + (x / 8);
                        let bit_mask = 0x80 >> (x % 8);
                        pixels[byte_idx] |= bit_mask;
                    }
                }
            }
        }
    }

    ImageData {
        width,
        height,
        pixels,
    }
}

impl ImageData {
    /// Loads an image file from disk and converts it to a 1-bit monochrome [`ImageData`].
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::PapermintError::Image`] if the file cannot be read or decoded.
    pub fn from_path(
        path: impl AsRef<Path>,
        max_width: Option<u32>,
        mode: DitherMode,
    ) -> Result<Self> {
        let img = image::open(path)?;
        Ok(dither_dynamic_image(&img, max_width, mode))
    }

    /// Decodes an image from in-memory bytes (PNG, JPEG, etc.) and converts it to a 1-bit monochrome [`ImageData`].
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::PapermintError::Image`] if the byte slice cannot be decoded.
    pub fn from_bytes(bytes: &[u8], max_width: Option<u32>, mode: DitherMode) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Ok(dither_dynamic_image(&img, max_width, mode))
    }
}
