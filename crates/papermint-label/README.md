# 🏷️ papermint-label

High-performance 2D thermal label and adhesive sticker printing engine supporting **TSPL / TSPL-II** (TSC, Xprinter, Munbyn, Rongta, Gprinter) and **ZPL II** (Zebra Technologies, Citizen, Godex).

`papermint-label` provides a fluent, type-safe canvas builder for coordinate-based label design, 1D barcodes, 2D QR codes, bounding boxes, separator bars, reverse-video highlight badges, and instant virtual SVG vector previews.

---

## ✨ Features

- **Dual Hardware Dialects**:
  - **TSPL / TSPL-II**: Strictly compliant with the standard TSC Auto ID programmer's manual (`SIZE`, `GAP`, `DIRECTION`, `REFERENCE`, `SPEED`, `DENSITY`, `CLS`, `TEXT`, `BARCODE`, `QRCODE`, `BOX`, `BAR`, `REVERSE`, `PRINT`).
  - **ZPL II**: Strictly compliant with the Zebra Technologies ZPL II Programming Guide (`^XA`, `^PW`, `^LL`, `^FO`, `^A0`, `^BC`, `^BQ`, `^GB`, `^FS`, `^XZ`).
- **1D Barcode Formats**: Code 128, Code 39, EAN-13, EAN-8, UPC-A, ITF with configurable height and human-readable text toggles.
- **2D QR Codes**: Configurable cell magnification and error correction levels (`L`, `M`, `Q`, `H`).
- **Precision Typography**: Scalable and bitmap fonts with horizontal and vertical multiplication ratios.
- **Virtual SVG Previewing**: Simulates physical adhesive label stickers with die-cut rounded corners, realistic margins, and vector barcode rendering for checkout/cashier screens.
- **Microsecond Compilation**: Generates raw wire bytes in under 15 microseconds with zero heap reallocations.

---

## 🚀 Quickstart

Add `papermint-label` to your `Cargo.toml`:

```toml
[dependencies]
papermint-label = "0.2"
```

### Designing and Compiling a Label

```rust
use papermint_label::{BarcodeType, Label, QrErrorCorrection};

// 1. Create a 50mm x 30mm label canvas at 203 DPI (8 dots/mm)
let label = Label::new(50.0, 30.0)
    .gap(3.0, 0.0)
    .speed(4)
    .density(10)
    .text(20, 15, "ORGANIC MATCHA LATTE")
    .barcode(20, 50, BarcodeType::Code128, 48, "MATCHA-4092")
    .qr(220, 50, "https://example.com/order/4092")
    .box_outline(10, 10, 380, 220, 2)
    .copies(1);

// 2. Compile to TSPL-II wire bytes (TSC, Xprinter, Munbyn)
let tspl_bytes: Vec<u8> = label.encode_tspl();

// 3. Or compile to ZPL II wire bytes (Zebra)
let zpl_bytes: Vec<u8> = label.encode_zpl();

// 4. Or render an instant virtual SVG preview
let svg_preview: String = label.render_svg();
```

---

## 📜 License

Licensed under the [MIT License](https://github.com/zerathlabs/papermint/blob/main/LICENSE).

