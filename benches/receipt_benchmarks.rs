//! Performance benchmarks for `papermint` thermal printing library.
//!
//! Measures:
//! - High-level receipt builder throughput
//! - N-column table layout with word-wrapping across 50 items
//! - Wire dialect serialization (ESC/POS vs StarPRNT)
//! - Floyd-Steinberg error-diffusion image dithering for thermal printheads
//!
//! Run with:
//!   `cargo bench`
//! Or fast sanity check with:
//!   `cargo bench --bench receipt_benchmarks -- --test`

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use papermint::{Alignment, Encoder, PaperWidth, Receipt, TableColumn};

fn bench_receipt_builder(c: &mut Criterion) {
    let columns = [
        TableColumn::fixed(4, Alignment::Left),
        TableColumn::fraction(0.50, Alignment::Left),
        TableColumn::fraction(0.22, Alignment::Right),
        TableColumn::fraction(0.24, Alignment::Right),
    ];

    c.bench_function("receipt_builder_restaurant_bill", |b| {
        b.iter(|| {
            let receipt = Receipt::new(PaperWidth::Mm80)
                .init()
                .center()
                .bold(true)
                .double_size(true)
                .text_ln(black_box("THE MINT BISTRO"))
                .double_size(false)
                .bold(false)
                .divider('=')
                .table_header(&["QTY", "DESCRIPTION", "PRICE", "TOTAL"], &columns)
                .row(&[
                    "2x",
                    "Truffle Wagyu Burger with Caramelized Onions",
                    "$14.50",
                    "$29.00",
                ])
                .row(&["1x", "Wood-Fired Margherita Pizza", "$18.00", "$18.00"])
                .row(&["2x", "San Pellegrino Mint Cooler", "$4.50", "$9.00"])
                .divider('-')
                .two_column("TOTAL:", "$56.00")
                .feed(2)
                .cut_full();

            black_box(receipt.commands().len())
        })
    });
}

fn bench_table_layout_50_items(c: &mut Criterion) {
    let columns = [
        TableColumn::fixed(4, Alignment::Left),
        TableColumn::fraction(0.50, Alignment::Left),
        TableColumn::fraction(0.22, Alignment::Right),
        TableColumn::fraction(0.24, Alignment::Right),
    ];

    c.bench_function("table_layout_50_items", |b| {
        b.iter(|| {
            let mut receipt = Receipt::new(PaperWidth::Mm80)
                .init()
                .table_header(&["QTY", "ITEM", "UNIT", "TOTAL"], &columns);

            for _ in 0..50 {
                receipt = receipt.row(&[
                    "1x",
                    "Artisanal Organic Farm-Fresh Product with Long Descriptive Ingredients",
                    "$12.99",
                    "$12.99",
                ]);
            }

            black_box(receipt.commands().len())
        })
    });
}

fn bench_dialects_encoding(c: &mut Criterion) {
    let columns = [
        TableColumn::fixed(4, Alignment::Left),
        TableColumn::fraction(0.50, Alignment::Left),
        TableColumn::fraction(0.22, Alignment::Right),
        TableColumn::fraction(0.24, Alignment::Right),
    ];

    let mut receipt = Receipt::new(PaperWidth::Mm80)
        .init()
        .table_header(&["QTY", "ITEM", "UNIT", "TOTAL"], &columns);

    for _ in 0..50 {
        receipt = receipt.row(&[
            "1x",
            "Artisanal Organic Farm-Fresh Product with Long Descriptive Ingredients",
            "$12.99",
            "$12.99",
        ]);
    }
    let commands = receipt.commands();

    #[cfg(feature = "escpos")]
    {
        let encoder = Encoder::escpos();
        c.bench_function("escpos_encode_50_items", |b| {
            b.iter(|| {
                let bytes = encoder.encode(black_box(commands)).unwrap();
                black_box(bytes.len())
            })
        });
    }

    #[cfg(feature = "star")]
    {
        let encoder = Encoder::star();
        c.bench_function("star_encode_50_items", |b| {
            b.iter(|| {
                let bytes = encoder.encode(black_box(commands)).unwrap();
                black_box(bytes.len())
            })
        });
    }
}

#[cfg(feature = "image")]
fn bench_image_dithering(c: &mut Criterion) {
    use papermint::image::{DitherMode, dither_dynamic_image};

    // Create a 384x200 continuous horizontal grayscale gradient
    let mut gray = ::image::GrayImage::new(384, 200);
    for (x, _y, pixel) in gray.enumerate_pixels_mut() {
        pixel[0] = ((x as f32 / 384.0) * 255.0) as u8;
    }
    let img = ::image::DynamicImage::ImageLuma8(gray);

    c.bench_function("floyd_steinberg_dither_384x200", |b| {
        b.iter(|| {
            let dithered =
                dither_dynamic_image(black_box(&img), Some(384), DitherMode::FloydSteinberg);
            black_box(dithered.pixels.len())
        })
    });
}

#[cfg(feature = "image")]
criterion_group!(
    benches,
    bench_receipt_builder,
    bench_table_layout_50_items,
    bench_dialects_encoding,
    bench_image_dithering
);

#[cfg(not(feature = "image"))]
criterion_group!(
    benches,
    bench_receipt_builder,
    bench_table_layout_50_items,
    bench_dialects_encoding
);

criterion_main!(benches);
