---
title: 2D Label & Barcode Printing (TSPL & ZPL)
description: Generate high-speed TSPL-II and ZPL II wire commands for barcode sticker labels, shipping labels, coffee cup tags, and shelf tags.
---

While traditional receipt printers feed continuous thermal rolls vertically using ESC/POS and Star Line Mode, industrial and retail **label printers** require precise **2D coordinate positioning `(x, y)`** on die-cut adhesive stickers, shelf tags, and shipping labels.

The `papermint-label` crate brings native, type-safe 2D label creation to the `papermint` ecosystem with zero external layout overhead.

---

## Supported Label Protocols

| Protocol | Hardware Manufacturers | Use Cases |
| :--- | :--- | :--- |
| **TSPL / TSPL-II** | **TSC**, **Xprinter**, **Rongta**, **Gprinter**, **Munbyn**, **Brother** (TD Series) | Coffee cup tags, retail price stickers, warehouse inventory labels |
| **ZPL II** | **Zebra** (ZD410, ZD420, ZD621, ZT411), **Citizen**, **Godex**, **Datamax** | 4×6 logistics shipping labels (FedEx/UPS/DHL), medical specimen tubes |

Both protocols compile from a unified fluent API and generate wire bytes in under **5 microseconds**.

---

## Adding `papermint-label` to Cargo.toml

```toml
[dependencies]
papermint = "0.2.3"
papermint-label = "0.2.3"
```

---

## Creating a Label Canvas

Labels are declared with physical dimensions in millimeters, inches, or printer dots:

```rust
use papermint_label::{Label, Unit, BarcodeType, QrErrorCorrection, Rotation};

let label = Label::new(50.0, 30.0, Unit::Millimeters) // 50mm x 30mm sticker
    .dpi(203) // 8 dots/mm (default)
    .gap(3.0, 0.0) // 3mm gap between die-cut labels
    .speed(4) // 4 inches per second
    .density(8) // print darkness (0-15)
    // Header text
    .text(16, 16, "MINT SPECIALTY COFFEE", 1, 1)
    // Product details
    .text(16, 44, "Oat Milk Latte (16oz)", 1, 1)
    .text(16, 68, "Extra Shot • Sugar Free", 1, 1)
    // Inverted order badge (white text inside black box)
    .reverse(310, 16, 70, 36)
    .text(318, 22, "#104", 1, 1)
    // 1D Barcode with human-readable label
    .barcode(16, 110, BarcodeType::Code128, 48, "ORD-9021", true)
    // 2D QR Code for customer pickup loyalty
    .qr(280, 100, "https://mintcafe.com/order/9021", 4, QrErrorCorrection::M)
    // Print 1 copy
    .print(1);
```

---

## Encoding Wire Commands

Convert the label canvas into hardware wire bytes for your target printer:

### 1. TSPL-II (Xprinter, TSC, Rongta, Munbyn)

```rust
let wire_bytes: Vec<u8> = label.encode_tspl();
```

Output wire stream:
```text
SIZE 50 mm, 30 mm
GAP 3 mm, 0 mm
SPEED 4
DENSITY 8
DIRECTION 1
CLS
TEXT 16,16,"2",0,1,1,"MINT SPECIALTY COFFEE"
TEXT 16,44,"2",0,1,1,"Oat Milk Latte (16oz)"
TEXT 16,68,"2",0,1,1,"Extra Shot • Sugar Free"
REVERSE 310,16,70,36
TEXT 318,22,"2",0,1,1,"#104"
BARCODE 16,110,"128",48,1,0,2,2,"ORD-9021"
QRCODE 280,100,M,4,A,0,"https://mintcafe.com/order/9021"
PRINT 1,1
```

### 2. ZPL II (Zebra, Citizen)

```rust
let wire_bytes: Vec<u8> = label.encode_zpl();
```

Output wire stream:
```text
^XA
^PW400
^LL240
^LH0,0
^PO0
^PR4
~SD19
^FO16,16^A0N,18,18^FDMINT SPECIALTY COFFEE^FS
^FO16,44^A0N,18,18^FDOat Milk Latte (16oz)^FS
^FO16,68^A0N,18,18^FDExtra Shot • Sugar Free^FS
^FO310,16^GB70,36,36,B^FS
^FO318,22^A0N,18,18^FD#104^FS
^FO16,110^BCN,48,Y,N,N^FDORD-9021^FS
^FO280,100^BQN,2,4,M^FDMA,https://mintcafe.com/order/9021^FS
^PQ1,0,1,Y
^XZ
```

---

## Virtual SVG Sticker Preview

Before transmitting to a physical roll or wasting adhesive label media, render an instant, pixel-accurate SVG vector preview:

```rust
let svg_string = label.render_svg();
```

The generated SVG features:
- **Rounded die-cut sticker corners** (`rx="8" ry="8"`) matching physical sticker rolls.
- **Drop-shadow elevation** for realistic UI display in admin portals.
- **Simulated barcode stripes** and **vector QR code matrices**.
- Inverted reverse blocks and high-contrast typography.

You can display this SVG directly inside React, Svelte, Vue, or web dashboards.

---

## Streaming to Network or USB Printers

Send the generated wire bytes to any thermal label printer using `papermint` transports:

```rust
use papermint::TcpTransport;
use papermint::transport::Transport;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wire_bytes = label.encode_tspl();

    // Stream directly over raw TCP (Port 9100)
    let addr: SocketAddr = "192.168.1.150:9100".parse()?;
    let mut transport = TcpTransport::new(addr);
    transport.write_all(&wire_bytes).await?;

    println!("✅ Label dispatched to Xprinter!");
    Ok(())
}
```

---

## Coordinate Cheat Sheet

| Parameter | Unit | Description |
| :--- | :--- | :--- |
| `x`, `y` | Dots | Top-left coordinate relative to the label peel edge |
| `font` | Integer | Standard bitmap/scalable font family index |
| `x_mult`, `y_mult` | Integer | Width and height multipliers (1x to 8x) |
| `rotation` | Enum | `Rotation::D0`, `Rotation::D90`, `Rotation::D180`, `Rotation::D270` |
| `dpi` | Integer | 203 DPI = 8 dots/mm; 300 DPI = 11.8 dots/mm |

