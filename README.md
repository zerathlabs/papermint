# 🌿 papermint

> A fast, multi-vendor, type-safe thermal receipt printing library for Rust.

[![Crates.io](https://img.shields.io/badge/crates.io-papermint-orange)](https://crates.io)
[![Documentation](https://docs.rs/papermint/badge.svg)](https://docs.rs/papermint)

---

## Design Principles

`papermint` is engineered around four core architectural pillars:
- **Decoupled Layout Architecture**: Formatting produces an intermediate command representation, keeping receipt structure completely independent of vendor wire bytes.
- **Rich Error Semantics**: Matchable, typed error variants providing actionable hardware and network diagnostics.
- **Resilient Async Networking**: Built-in connection timeouts, socket reconnect resilience, and non-blocking I/O.
- **Unicode-Aware Layout**: Precise column balancing that properly calculates multi-byte UTF-8 character widths.

Built on a clean **three-layer architecture**:

```text
┌─────────────────────────────────────────────────────────┐
│  Layer 3: Layout Engine (Receipt Builder)               │
│  - Fluent API, multi-column tables, dividers, barcodes  │
└──────────────────────────┬──────────────────────────────┘
                           │ produces Vec<Command> (Command IR)
┌──────────────────────────▼──────────────────────────────┐
│  Layer 2: Dialect Encoders                              │
│  - ESC/POS (Epson, Bixolon, Citizen, Xprinter, Rongta)  │
│  - StarPRNT (Star Micronics TSP100, TSP650, mC-Print)   │
└──────────────────────────┬──────────────────────────────┘
                           │ emits wire bytes (Vec<u8>)
┌──────────────────────────▼──────────────────────────────┐
│  Layer 1: Transports                                    │
│  - Async TCP (IP/Ethernet POS printers)                 │
│  - VecSink (In-memory testing, mock, PDF generation)    │
│  - USB / Serial (future)                                │
└─────────────────────────────────────────────────────────┘
```

The central innovation is the **`Command` Intermediate Representation (IR)**: you build receipts once, test them in-memory, replay them, store them as commands, or encode them for any vendor.

---

## Features

- ⚡ **Zero-cost, low-allocation encoding**: Encodes directly into reusable buffers.
- 📐 **Unicode-aware layout engine**: Precise 80mm (48 cols) and 58mm (32 cols) column balancing.
- 🖨️ **Multi-Vendor Dialect Support**:
  - **Epson ESC/POS**: Full support for TM-T88, TM-m30, TM-T20, Bixolon, Citizen, and compatibles.
  - **StarPRNT / Line Mode**: Native support for TSP100, TSP650II, TSP700, and mC-Print series.
  - **Text Styling**: Bold, Underline, Inverse, Font A/B, Double Width/Height.
  - **Advanced Formatting**: Upside-down 180° rotation (for kitchen order clips), custom line spacing, and code table selection.
  - **Barcodes (1D)**: Code128 (with auto-subset `{B`), EAN13, EAN8, UPC-A, UPC-E, ITF, Codabar, Code39, Code93.
  - **2D QR Codes**: Hardware-accelerated Model 2 generation with configurable cell size and error correction.
  - **Graphics**: 1-bit monochrome raster bitmap printing.
  - **Hardware I/O**: Full & partial paper cutting, RJ12 Pin 2 / Pin 5 cash drawer kicks, and audio buzzer alerts.
- 🌐 **Resilient Async TCP Transport**: Configurable connect and write timeouts with automatic reconnection on stream errors.
- 🧪 **First-Class Testing**: Built-in `VecSink` for zero-hardware unit and integration testing.
- 🛡️ **Safe & reliable**: 100% safe Rust, structured error types, and clean lifetimes.

---

## Quickstart

Add `papermint` to your `Cargo.toml`:

```toml
[dependencies]
papermint = "0.1"
tokio = { version = "1", features = ["full"] }
```

### 1. Print a receipt over TCP network

```rust
use std::net::SocketAddr;
use papermint::{PaperWidth, Printer, Receipt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Build your receipt using the fluent builder
    let receipt = Receipt::new(PaperWidth::Mm80)
        .init()
        .center()
        .bold(true)
        .double_size(true)
        .text_ln("MINT BISTRO")
        .double_size(false)
        .bold(false)
        .text_ln("123 Main Street")
        .divider('-')
        .left()
        .two_column("1x Smash Burger", "$9.50")
        .two_column("1x Truffle Fries", "$4.50")
        .two_column("1x Mint Cooler", "$3.00")
        .divider('=')
        .bold(true)
        .two_column("TOTAL", "$17.00")
        .bold(false)
        .divider('-')
        .feed(1)
        .center()
        .qr("https://pay.example.com/receipt/4820")
        .feed(2)
        .text_ln("Thank you for your visit!")
        .feed(3)
        .cut_full()
        .open_drawer();

    // 2. Connect and print to network thermal printer
    let addr: SocketAddr = "192.168.1.200:9100".parse()?;
    let mut printer = Printer::escpos_tcp(addr);

    printer.print(&receipt).await?;
    println!("Receipt printed successfully!");

    Ok(())
}
```

### 2. Testing without physical printers (`VecSink`)

```rust
use papermint::{PaperWidth, Printer, Receipt};

#[tokio::test]
async fn test_my_receipt() {
    let mut printer = Printer::escpos_mock();

    let receipt = Receipt::new(PaperWidth::Mm80)
        .init()
        .text_ln("Test Order")
        .cut_full();

    printer.print(&receipt).await.unwrap();

    let bytes = printer.transport().bytes();
    assert!(!bytes.is_empty());
    assert!(String::from_utf8_lossy(&bytes).contains("Test Order"));
}
```

---

## Feature Flags

| Feature | Default | Description |
|---|---|---|
| `std` | **Yes** | Standard library support and `std::io::Error` wrapping |
| `escpos` | **Yes** | Epson ESC/POS dialect encoder |
| `star` | **Yes** | StarPRNT / Star Line Mode dialect encoder |
| `async` | **Yes** | Asynchronous traits and `Printer` orchestrator |
| `tcp` | **Yes** | Tokio async TCP transport (`TcpTransport`) |
| `image` | No | PNG/JPEG decoding, aspect-ratio auto-scaling, and Floyd-Steinberg dithering |

### 3. Image Printing & Floyd-Steinberg Dithering

Enable the `image` feature to print logos, graphics, and coupons directly from PNG or JPEG files:

```toml
[dependencies]
papermint = { version = "0.1", features = ["image"] }
```

```rust
use papermint::{DitherMode, PaperWidth, Receipt};

let receipt = Receipt::new(PaperWidth::Mm80)
    .init()
    .center()
    // Auto-scales to fit paper width and applies Floyd-Steinberg error diffusion:
    .image_from_path("assets/logo.png")?
    // Or load from in-memory bytes with custom threshold cutoff for crisp vector art:
    .image_from_bytes_with_options(&logo_bytes, Some(384), DitherMode::Threshold(128))?
    .feed(2)
    .cut_full();
```

---

## Documentation & Guides

- [Hardware & Protocol Specification](docs/HARDWARE_COMMUNICATION.md) — Wire byte sequences, electrical drawer pulses, and QR symbology.
- [Architecture & Internal Design](docs/ARCHITECTURE.md) — 3-layer architecture, atomic state tracking, and Unicode column mathematics.
- [Supported Hardware Guide](docs/SUPPORTED_HARDWARE.md) — Tested printer brands, models, and DIP switch emulation configuration.

