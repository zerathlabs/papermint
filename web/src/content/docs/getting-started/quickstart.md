---
title: Quickstart Guide
description: Get up and printing with papermint in 5 minutes.
---

# 🚀 Quickstart Guide

Get up and printing with `papermint` in minutes across Rust, Node.js, or HTTP.

---

## 1. Using Rust

Add `papermint` and `tokio` to your `Cargo.toml`:

```toml
[dependencies]
papermint = "0.1"
tokio = { version = "1", features = ["full"] }
```

### Print over Network TCP (Port 9100)

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
        .text_ln("123 Market Street, San Francisco")
        .divider('=')
        .left()
        .row(&["1x", "Smash Burger Deluxe", "$12.00"])
        .row(&["1x", "Truffle Fries", "$6.50"])
        .row(&["2x", "Craft Soda", "$7.00"])
        .divider('-')
        .bold(true)
        .row(&["TOTAL", "", "$25.50"])
        .bold(false)
        .divider('=')
        .center()
        .qr("https://mintbistro.com/pay/1042")
        .feed(2)
        .cut();

    // 2. Connect to a thermal printer on your local network
    let target_addr: SocketAddr = "192.168.1.200:9100".parse()?;
    let mut printer = Printer::escpos_tcp(target_addr);

    // 3. Transmit the print job
    printer.print(&receipt).await?;
    println!("✅ Receipt sent successfully to printer!");

    Ok(())
}
```

---

## 2. Using Node.js / TypeScript

Install `papermint`:

```bash
pnpm add papermint
# or
npm install papermint
```

### TypeScript Example

```typescript
import { Receipt, Printer } from 'papermint';

const receipt = new Receipt('80mm')
  .init()
  .center()
  .bold(true)
  .textLn('BLUE CAFE')
  .bold(false)
  .divider('=')
  .left()
  .tableRow(['1x', 'Matcha Latte', '$4.50'])
  .tableRow(['2x', 'Croissant', '$7.00'])
  .divider('-')
  .bold(true)
  .tableRow(['TOTAL', '', '$11.50'])
  .bold(false)
  .center()
  .qr('https://pay.example.com/order/100')
  .cut();

// Print over TCP network:
const printer = Printer.tcp('192.168.1.200:9100', 'escpos');
await printer.print(receipt);

// Or encode directly to a Node Buffer for Bluetooth or USB streaming:
const wireBytes: Buffer = receipt.encode('escpos');
```

---

## 3. Using the HTTP Print Daemon (`papermintd`)

Run the print sidecar microservice:

```bash
cargo run --package papermint-daemon
```

Send a print job via `curl`:

```bash
curl -X POST http://127.0.0.1:8080/api/print \
  -H "Content-Type: application/json" \
  -d '{
    "target": { "type": "mock" },
    "dialect": "escpos",
    "paper_width": "80mm",
    "ticket": {
      "title": "MINT BISTRO",
      "items": [
        { "qty": "1", "description": "Truffle Burger", "price": "$12.00", "total": "$12.00" }
      ],
      "totals": [
        { "label": "TOTAL", "value": "$12.00" }
      ],
      "cut_mode": "full"
    }
  }'
```
