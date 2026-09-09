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
│  - Async TCP (RAW network POS printers on Port 9100)    │
│  - Driverless USB (`nusb` OS-independent Class 7)       │
│  - Serial / RS-232 & Bluetooth SPP (`tokio-serial`)     │
│  - VecSink (In-memory testing, mock execution)          │
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
| `serial` | No | Asynchronous serial RS-232 / COM port transport (`SerialTransport`) |
| `usb` | No | Pure-Rust async USB Printer Class 07 transport (`UsbTransport`) |
| `image` | No | PNG/JPEG decoding, aspect-ratio auto-scaling, and Floyd-Steinberg dithering |

### 3. Multi-Column Tables with Word-Wrapping

Print tabular receipts where long item names automatically wrap at word boundaries without breaking columnar alignment:

```rust
use papermint::{Alignment, PaperWidth, Receipt, TableColumn};

let columns = [
    TableColumn::fixed(4, Alignment::Left),          // Qty
    TableColumn::fraction(0.50, Alignment::Left),   // Description
    TableColumn::fraction(0.20, Alignment::Right),  // Unit Price
    TableColumn::fraction(0.25, Alignment::Right),  // Total Price
];

let receipt = Receipt::new(PaperWidth::Mm80)
    .init()
    .table_header(&["QTY", "ITEM", "PRICE", "TOTAL"], &columns)
    .divider('-')
    .row(&[
        "2x",
        "Double Truffle Wagyu Smash Burger with Caramelized Onions",
        "$12.50",
        "$25.00",
    ])
    .row(&["1x", "Large Truffle Fries", "$5.50", "$5.50"])
    .divider('=')
    .two_column("TOTAL", "$30.50")
    .cut_full();
```

### 4. Image Printing & Floyd-Steinberg Dithering

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

### 5. Real-Time Hardware Status & Sensor Telemetry

Query printer readiness, paper low warnings, cover open, cutter jams, and cash drawer state in real time:

```rust
use papermint::{Printer, PaperStatus, CoverStatus, DrawerStatus};

// Query real-time telemetry (ESC/POS or StarPRNT)
let status = printer.query_status().await?;

if !status.is_ready() {
    if status.cover == CoverStatus::Open {
        eprintln!("Warning: Printer cover is open!");
    }
    if status.paper == PaperStatus::Empty {
        eprintln!("Error: Out of paper roll!");
    }
    if status.cutter_error {
        eprintln!("Alert: Auto-cutter jammed!");
    }
}

if status.paper == PaperStatus::NearEnd {
    println!("Notice: Paper roll is running low, replace soon.");
}

if status.drawer == DrawerStatus::Open {
    println!("Cash drawer is currently open.");
}
```

### 6. International Character Sets & Extended Code Pages

Thermal receipt printers are 8-bit devices that cannot natively parse multi-byte UTF-8 without garbled output. `papermint` provides automatic single-byte transcoding and standard `ESC R n` national character set switching:

```rust
use papermint::{Receipt, PaperWidth, CodePage, InternationalCharset};

let receipt = Receipt::new(PaperWidth::Mm80)
    .init()
    // Select national currency/symbol set (e.g. UK replaces '#' with '£'):
    .charset_uk()
    // Select 8-bit extended code table (e.g. Windows-1252 for Western Europe):
    .code_page(CodePage::Wpc1252)
    .text_ln("Fish & Chips: £12.50")
    // Euro (€) is automatically transcoded to 0x80 (no UTF-8 mojibake!):
    .two_column("Crêpe Nutella", "4.50 €")
    .feed(2)
    .cut_full();
```

### 7. Thermal Printhead Energy & Density Calibration

Calibrate printhead strobe pulses and burn darkness to match specific thermal paper sensitivities or cold/hot operating environments:

```rust
use papermint::{Receipt, PaperWidth, PrintDensity, HeatingParameters};

let receipt = Receipt::new(PaperWidth::Mm80)
    .init()
    // Increase density for high optical contrast on 2D barcodes or low-sensitivity paper:
    .density_dark() // or .density_light(), .density_normal(), .density_high_contrast()
    // Configure low-level thermal strobe timings: max dots, heat time (x10µs), interval (x10µs)
    .heating_parameters(HeatingParameters {
        max_heating_dots: 8,    // 64 dots simultaneously
        heating_time: 90,       // 900 µs pulse duration
        heating_interval: 3,    // 30 µs cooling interval
    })
    .text_ln("Crisp, High-Contrast Receipt")
    .feed(2)
    .cut_full();
```

### 8. Direct USB & Serial Hardware Transports

Communicate with physical thermal receipt printers connected via USB or Serial RS-232 / Virtual COM ports:

#### Serial / RS-232 Counter POS
```rust
use papermint::{Printer, Receipt, PaperWidth};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect over serial with automatic RTS/CTS hardware flow control:
    let mut printer = Printer::escpos_serial("/dev/ttyUSB0", 19200);

    let receipt = Receipt::new(PaperWidth::Mm80)
        .init()
        .text_ln("Serial Print Job")
        .cut_full();

    printer.print(&receipt).await?;
    Ok(())
}
```

#### USB Printer Class (Class 07)
```rust
use papermint::{Printer, Receipt, PaperWidth};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Target by Vendor ID & Product ID (e.g. Epson TM-T88VI):
    let mut printer = Printer::escpos_usb(0x04B8, 0x0202);

    // Or auto-discover the first attached USB printer:
    // let mut printer = Printer::escpos_usb_auto();

    let receipt = Receipt::new(PaperWidth::Mm80)
        .init()
        .text_ln("Direct USB Bulk Print Job")
        .cut_full();

    printer.print(&receipt).await?;
    Ok(())
}
```

#### Hardware Device Discovery & Port Enumeration
```rust
use papermint::{SerialTransport, UsbTransport};

// Discover available serial COM ports (e.g. /dev/ttyUSB0, COM3):
let serial_ports = SerialTransport::available_ports()?;
for port in serial_ports {
    println!("Found serial port: {port}");
}

// Discover attached USB printers (Vendor ID, Product ID, Serial, Manufacturer):
let usb_printers = UsbTransport::list_printers()?;
for printer in usb_printers {
    println!(
        "Found USB printer: VID={:#06x} PID={:#06x} ({:?})",
        printer.vendor_id, printer.product_id, printer.product_name
    );
}
```

---

### 9. Node.js & TypeScript Bindings (`@zerathlabs/papermint`)

High-performance native N-API bindings allow your TypeScript POS and backend services to format receipts and stream to hardware with zero runtime serialization lag:

```bash
pnpm add @zerathlabs/papermint
```

```typescript
import { Receipt, Printer } from '@zerathlabs/papermint';

const receipt = new Receipt('80mm')
  .init()
  .center()
  .bold(true)
  .textLn('MINT BISTRO')
  .bold(false)
  .divider('=')
  .tableHeader(
    ['QTY', 'DESCRIPTION', 'PRICE', 'TOTAL'],
    [
      { widthFixed: 4, align: 'left' },
      { widthFraction: 0.50, align: 'left' },
      { widthFraction: 0.22, align: 'right' },
      { widthFraction: 0.24, align: 'right' },
    ]
  )
  .row(['2x', 'Truffle Wagyu Burger with Caramelized Onions', '$14.50', '$29.00'])
  .row(['1x', 'Wood-Fired Margherita Pizza', '$18.00', '$18.00'])
  .divider('-')
  .twoColumn('TOTAL DUE:', '$47.00')
  .qr('https://pay.mintbistro.com/bill/1042')
  .cutFull();

// Print over TCP network to thermal printer:
const printer = Printer.tcp('192.168.1.100:9100', 'escpos');
await printer.print(receipt);

// Or encode directly to a Node Buffer:
const rawBuffer = receipt.encode('escpos');
```

---

### 10. Native Print Daemon (`papermintd`)

`papermintd` is a standalone, high-concurrency HTTP sidecar service powered by Axum and Tokio for local POS ticket streaming:

```bash
cargo run -p papermint-daemon -- --port 8080
```

Send print jobs via simple HTTP POST:
```bash
curl -X POST http://127.0.0.1:8080/api/print \
  -H "Content-Type: application/json" \
  -d '{
    "target": { "type": "tcp", "address": "192.168.1.100:9100" },
    "dialect": "escpos",
    "ticket": {
      "title": "MINT BISTRO",
      "items": [
        { "qty": "1x", "description": "Truffle Smash Burger", "total": "$14.50" }
      ],
      "totals": [
        { "label": "TOTAL:", "value": "$14.50" }
      ],
      "cut_mode": "full"
    }
  }'
```

---

## Documentation & Guides

- [Architecture & Multi-Crate Workspace](docs/ARCHITECTURE.md) — 3-layer architecture, atomic state tracking, and Unicode column mathematics.
- [Mobile POS & React Native Guide](docs/MOBILE_INTEGRATION.md) — Expo SDK 56+ Inline Modules, C-ABI FFI, and Bluetooth Classic/BLE streaming.
- [Print Daemon HTTP REST API](docs/DAEMON_API.md) — `papermintd` endpoints, JSON ticket schemas, and hardware sensor telemetry.
- [Hardware & Protocol Specification](docs/HARDWARE_COMMUNICATION.md) — Wire byte sequences, electrical drawer pulses, and QR symbology.
- [Supported Hardware & Connection Guide](docs/SUPPORTED_HARDWARE.md) — Tested printer models, driverless USB (`udev` rules), RS-232, and cash drawer pins.

