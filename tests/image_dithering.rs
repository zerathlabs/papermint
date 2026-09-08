#![cfg(feature = "image")]

use image::{ImageBuffer, Rgba};
use papermint::image::dither_dynamic_image;
use papermint::{DitherMode, Encoder, PaperWidth, Receipt};

#[test]
fn test_solid_white_image() {
    let img = image::DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
        16,
        16,
        Rgba([255, 255, 255, 255]),
    ));
    let dithered = dither_dynamic_image(&img, None, DitherMode::FloydSteinberg);

    assert_eq!(dithered.width, 16);
    assert_eq!(dithered.height, 16);
    // 16 / 8 = 2 bytes per row * 16 rows = 32 bytes
    assert_eq!(dithered.pixels.len(), 32);
    // All white = 0 on thermal paper
    assert!(dithered.pixels.iter().all(|&b| b == 0x00));
}

#[test]
fn test_solid_black_image() {
    let img = image::DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
        16,
        8,
        Rgba([0, 0, 0, 255]),
    ));
    let dithered = dither_dynamic_image(&img, None, DitherMode::FloydSteinberg);

    assert_eq!(dithered.width, 16);
    assert_eq!(dithered.height, 8);
    assert_eq!(dithered.pixels.len(), 16);
    // All black = 0xFF on thermal paper
    assert!(dithered.pixels.iter().all(|&b| b == 0xFF));
}

#[test]
fn test_transparent_pixels_treated_as_white_paper() {
    // Transparent pixels (A = 0) must be treated as blank paper (0x00), NOT black
    let img = image::DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
        8,
        8,
        Rgba([0, 0, 0, 0]),
    ));
    let dithered = dither_dynamic_image(&img, None, DitherMode::FloydSteinberg);

    assert_eq!(dithered.width, 8);
    assert_eq!(dithered.height, 8);
    assert!(dithered.pixels.iter().all(|&b| b == 0x00));
}

#[test]
fn test_threshold_cutoff_mode() {
    let mut buf = ImageBuffer::new(8, 2);
    // Row 0: Dark pixels (Luminance = 100 < 128 threshold -> Black)
    for x in 0..8 {
        buf.put_pixel(x, 0, Rgba([100, 100, 100, 255]));
    }
    // Row 1: Light pixels (Luminance = 150 >= 128 threshold -> White)
    for x in 0..8 {
        buf.put_pixel(x, 1, Rgba([150, 150, 150, 255]));
    }

    let img = image::DynamicImage::ImageRgba8(buf);
    let dithered = dither_dynamic_image(&img, None, DitherMode::Threshold(128));

    assert_eq!(dithered.width, 8);
    assert_eq!(dithered.height, 2);
    assert_eq!(dithered.pixels.len(), 2);
    assert_eq!(dithered.pixels[0], 0xFF); // All 8 pixels black
    assert_eq!(dithered.pixels[1], 0x00); // All 8 pixels white
}

#[test]
fn test_floyd_steinberg_halftone_gradient() {
    // 50% gray image with Floyd-Steinberg should create an alternating halftone pattern,
    // neither all-white nor all-black.
    let img = image::DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
        32,
        32,
        Rgba([128, 128, 128, 255]),
    ));
    let dithered = dither_dynamic_image(&img, None, DitherMode::FloydSteinberg);

    let black_dots: u32 = dithered
        .pixels
        .iter()
        .map(|b| b.count_ones())
        .sum();
    let total_pixels = 32 * 32;

    // A 50% gray image should have approximately 50% black dots (±5% tolerance)
    let ratio = (black_dots as f32) / (total_pixels as f32);
    assert!(
        (0.45..=0.55).contains(&ratio),
        "Expected ~50% dither density, got {ratio:.2}"
    );
}

#[test]
fn test_non_multiple_of_8_width_padding() {
    // 11x2 image:
    // row_bytes = ceil(11 / 8) = 2 bytes per row. Total 4 bytes.
    // In row 0: first 11 pixels black, remaining 5 padding bits must be 0.
    let mut buf = ImageBuffer::new(11, 2);
    for y in 0..2 {
        for x in 0..11 {
            buf.put_pixel(x, y, Rgba([0, 0, 0, 255]));
        }
    }

    let img = image::DynamicImage::ImageRgba8(buf);
    let dithered = dither_dynamic_image(&img, None, DitherMode::Threshold(128));

    assert_eq!(dithered.width, 11);
    assert_eq!(dithered.height, 2);
    assert_eq!(dithered.pixels.len(), 4);

    // Row 0:
    // byte 0 has 8 black pixels -> 0b1111_1111 = 0xFF
    // byte 1 has 3 black pixels in MSB (bits 7, 6, 5) and 5 zero padding bits -> 0b1110_0000 = 0xE0
    assert_eq!(dithered.pixels[0], 0xFF);
    assert_eq!(dithered.pixels[1], 0xE0);

    // Row 1 matches Row 0
    assert_eq!(dithered.pixels[2], 0xFF);
    assert_eq!(dithered.pixels[3], 0xE0);
}

#[test]
fn test_aspect_ratio_auto_resizing() {
    let img = image::DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
        800,
        400,
        Rgba([200, 200, 200, 255]),
    ));
    // Request max_width = 400. Width should scale to 400, height to 200.
    let dithered = dither_dynamic_image(&img, Some(400), DitherMode::FloydSteinberg);

    assert_eq!(dithered.width, 400);
    assert_eq!(dithered.height, 200);
    assert_eq!(dithered.pixels.len(), (400 / 8) * 200);
}

#[test]
fn test_receipt_fluent_image_loading_and_encoding() {
    // Generate a synthetic PNG in memory
    let img_buf = ImageBuffer::from_fn(64, 32, |x, y| {
        if (x / 8 + y / 8) % 2 == 0 {
            Rgba([0u8, 0u8, 0u8, 255u8])
        } else {
            Rgba([255u8, 255u8, 255u8, 255u8])
        }
    });

    let mut png_bytes = Vec::new();
    let dynamic_img = image::DynamicImage::ImageRgba8(img_buf);
    dynamic_img
        .write_to(
            &mut std::io::Cursor::new(&mut png_bytes),
            image::ImageFormat::Png,
        )
        .expect("PNG encoding failed");

    // Compose receipt with image_from_bytes
    let receipt = Receipt::new(PaperWidth::Mm80)
        .init()
        .center()
        .image_from_bytes(&png_bytes)
        .expect("image_from_bytes should succeed")
        .feed(2)
        .cut_full();

    // Verify ESC/POS encoding
    let escpos_encoder = Encoder::escpos();
    let escpos_bytes = escpos_encoder
        .encode(receipt.commands())
        .expect("ESC/POS encoding failed");
    // Should contain GS v 0 raster bit image header: 0x1D, 0x76, 0x30, 0x00
    assert!(
        escpos_bytes.windows(4).any(|w| w == [0x1D, 0x76, 0x30, 0x00]),
        "ESC/POS output must contain GS v 0 raster header"
    );

    // Verify Star encoding
    let star_encoder = Encoder::star();
    let star_bytes = star_encoder
        .encode(receipt.commands())
        .expect("Star encoding failed");
    // Should contain Star raster mode command: ESC * r A (0x1B, 0x2A, 0x72, 0x41)
    assert!(
        star_bytes.windows(4).any(|w| w == [0x1B, 0x2A, 0x72, 0x41]),
        "Star output must contain ESC * r A raster mode enter"
    );
}
