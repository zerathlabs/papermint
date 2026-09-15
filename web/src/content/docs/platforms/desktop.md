---
title: "Desktop POS (Electron & Tauri)"
description: "Building cross-platform desktop POS kiosks and cashier stations with Electron and Tauri using papermint."
---

# 🖥️ Desktop POS Integration Guide: Electron & Tauri

Desktop Point-of-Sale (POS) stations, self-checkout kiosks, and kitchen ticket display monitors run across **Windows**, **macOS**, and **Linux** (e.g. Ubuntu POS). 

Whether you choose **Electron** or **Tauri**, `papermint` provides native, zero-friction printing:

* **Electron**: Uses our pre-compiled **Node-API (N-API)** package (`npm install papermint`). **No `electron-rebuild` or C++ toolchains required**.
* **Tauri (v1 & v2)**: Uses the core Rust crate directly (`cargo add papermint`). **Zero bridges or FFI overhead**, delivering instantaneous print times and tiny binary sizes (~15MB).

---

## Part 1: Electron POS Integration

Electron separates execution into a **Main Process** (Node.js runtime with operating system access) and a **Renderer Process** (Chromium webview for your POS UI).

### 1. Installation

In your Electron project root:

```bash
npm install papermint
# or
pnpm add papermint
```

> [!TIP] No `electron-rebuild` Required!
> `papermint` is compiled with **Node-API (N-API)** ABI stability. Unlike legacy native modules that fail or crash when Electron updates Chromium/Node versions, `papermint` works out of the box without recompilation.

---

### 2. Main Process Implementation (`main.js` / `main.ts`)

Hardware operations (USB, TCP, and Serial) belong in the **Main Process**:

```javascript
// electron/main.js
const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const { Receipt, Printer, listPrinters } = require('papermint');

let mainWindow;

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1024,
    height: 768,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });

  mainWindow.loadFile('index.html');
}

app.whenReady().then(createWindow);

// IPC Handler: Discover connected USB printers
ipcMain.handle('pos:list-printers', async () => {
  return listPrinters();
});

// IPC Handler: Print Receipt
ipcMain.handle('pos:print', async (event, ticket) => {
  try {
    // 1. Build receipt
    const receipt = new Receipt(ticket.paperWidth || '80mm')
      .init()
      .center()
      .bold(true)
      .textLn(ticket.title)
      .bold(false)
      .textLn(ticket.subtitle || '')
      .divider('=')
      .left();

    for (const item of ticket.items) {
      receipt.tableRow([item.qty, item.name, item.price]);
    }

    receipt
      .divider('-')
      .bold(true)
      .tableRow(['TOTAL', '', ticket.total])
      .bold(false)
      .divider('=')
      .center()
      .qr(ticket.qrData || 'https://papermint.zerathlabs.com')
      .feed(2)
      .cut();

    // 2. Connect to printer (TCP or USB)
    if (ticket.printerIp) {
      const printer = Printer.tcp(`${ticket.printerIp}:9100`, 'escpos');
      await printer.print(receipt);
    } else {
      // Driverless USB (0x04b8 Epson, 0x0519 Star, etc.)
      const printer = Printer.usb(ticket.vendorId || 0x04b8, ticket.productId || 0x0202, 'escpos');
      await printer.print(receipt);
    }

    return { success: true };
  } catch (err) {
    console.error('Print failed:', err);
    return { success: false, error: err.message };
  }
});

// IPC Handler: Generate In-App SVG Preview
ipcMain.handle('pos:preview', async (event, ticket) => {
  const receipt = new Receipt(ticket.paperWidth || '80mm')
    .init()
    .center()
    .bold(true)
    .textLn(ticket.title)
    .left();

  for (const item of ticket.items) {
    receipt.tableRow([item.qty, item.name, item.price]);
  }
  receipt.divider('=').bold(true).tableRow(['TOTAL', '', ticket.total]);

  // Generates crisp SVG string
  return receipt.renderSvg();
});
```

---

### 3. Preload Script (`preload.js`)

Safely expose POS functions to the renderer via `contextBridge`:

```javascript
// electron/preload.js
const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('posApi', {
  listPrinters: () => ipcRenderer.invoke('pos:list-printers'),
  printReceipt: (ticket) => ipcRenderer.invoke('pos:print', ticket),
  previewReceipt: (ticket) => ipcRenderer.invoke('pos:preview', ticket),
});
```

---

### 4. Renderer UI (React / Vue / HTML)

In your cashier frontend:

```javascript
// In your React / Vue cashier component:
async function handleCheckout() {
  const ticket = {
    paperWidth: '80mm',
    title: 'THE MINT BISTRO',
    subtitle: 'Order #4092',
    items: [
      { qty: '1x', name: 'Avocado Toast', price: '$9.00' },
      { qty: '2x', name: 'Cold Brew Coffee', price: '$8.00' }
    ],
    total: '$17.00',
    printerIp: '192.168.1.200'
  };

  // 1. Optional: Render live preview in UI
  const svgPreview = await window.posApi.previewReceipt(ticket);
  document.getElementById('preview-container').innerHTML = svgPreview;

  // 2. Transmit to physical printer
  const result = await window.posApi.printReceipt(ticket);
  if (result.success) {
    alert('Receipt printed!');
  } else {
    alert('Print error: ' + result.error);
  }
}
```

---

## Part 2: Tauri POS Integration (v1 & v2)

Tauri pairs a lightweight web frontend with a **pure Rust backend**. Because `papermint` is a Rust crate, Tauri apps achieve maximum speed, zero FFI overhead, and a tiny memory footprint.

### 1. Add `papermint` to `src-tauri/Cargo.toml`

```toml
[dependencies]
papermint = "0.2"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

---

### 2. Implement Tauri Commands (`src-tauri/src/main.rs`)

Define backend commands callable directly from your frontend:

```rust
// src-tauri/src/main.rs (or lib.rs in Tauri v2)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use papermint::{PaperWidth, Printer, Receipt};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Deserialize)]
struct TicketItem {
    qty: String,
    name: String,
    price: String,
}

#[derive(Deserialize)]
struct TicketPayload {
    store_name: String,
    items: Vec<TicketItem>,
    total: String,
    printer_ip: Option<String>,
}

#[tauri::command]
async fn print_receipt(payload: TicketPayload) -> Result<String, String> {
    // 1. Build receipt
    let mut receipt = Receipt::new(PaperWidth::Mm80)
        .init()
        .center()
        .bold(true)
        .double_size(true)
        .text_ln(&payload.store_name)
        .double_size(false)
        .bold(false)
        .divider('=')
        .left();

    for item in &payload.items {
        receipt = receipt.row(&[&item.qty, &item.name, &item.price]);
    }

    receipt = receipt
        .divider('-')
        .bold(true)
        .row(&["TOTAL", "", &payload.total])
        .bold(false)
        .divider('=')
        .center()
        .feed(2)
        .cut();

    // 2. Transmit over TCP Port 9100 or USB
    if let Some(ip) = payload.printer_ip {
        let addr: SocketAddr = format!("{}:9100", ip)
            .parse()
            .map_err(|e| format!("Invalid IP: {}", e))?;
        let mut printer = Printer::escpos_tcp(addr);
        printer.print(&receipt).await.map_err(|e| e.to_string())?;
    } else {
        // Driverless USB (Epson / Generic 0x04b8:0x0202)
        let mut printer = Printer::escpos_usb(0x04b8, 0x0202)
            .map_err(|e| format!("USB Printer not found: {}", e))?;
        printer.print(&receipt).await.map_err(|e| e.to_string())?;
    }

    Ok("Receipt printed successfully!".into())
}

#[tauri::command]
fn preview_receipt(payload: TicketPayload) -> Result<String, String> {
    let mut receipt = Receipt::new(PaperWidth::Mm80)
        .init()
        .center()
        .bold(true)
        .text_ln(&payload.store_name)
        .divider('=')
        .left();

    for item in &payload.items {
        receipt = receipt.row(&[&item.qty, &item.name, &item.price]);
    }

    receipt = receipt.divider('=').bold(true).row(&["TOTAL", "", &payload.total]);

    // Zero-overhead SVG rendering
    Ok(receipt.render_svg())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![print_receipt, preview_receipt])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

### 3. Frontend Invocation (React / Vue / Svelte / TypeScript)

In your Tauri UI application:

```typescript
import { invoke } from '@tauri-apps/api/core'; // Tauri v2 (or '@tauri-apps/api/tauri' for v1)

const payload = {
  store_name: "AL-NAKHEEL LUXURY ROASTERY",
  items: [
    { qty: "1x", name: "Cardamom Qahwa", price: "18.00 SAR" },
    { qty: "2x", name: "Saffron Date Cake", price: "48.00 SAR" }
  ],
  total: "66.00 SAR",
  printer_ip: "192.168.1.200"
};

// 1. Live Vector SVG Preview
async function showPreview() {
  const svgString = await invoke<string>('preview_receipt', { payload });
  document.getElementById('receipt-preview')!.innerHTML = svgString;
}

// 2. Physical Thermal Print Job
async function printOrder() {
  try {
    const status = await invoke<string>('print_receipt', { payload });
    console.log(status);
  } catch (err) {
    console.error("Print failed:", err);
  }
}
```

---

## Comparison: Electron vs. Tauri for Desktop POS

| Dimension | Electron POS | Tauri POS |
| :--- | :--- | :--- |
| **Language Binding** | Node.js N-API (`papermint`) | Pure Rust (`papermint` crate) |
| **App Bundle Size** | ~120–180 MB | **~10–25 MB** |
| **Memory Footprint** | ~150–350 MB RAM | **~30–60 MB RAM** |
| **Driverless USB (`nusb`)** | Fully Supported | Fully Supported |
| **Network TCP (Port 9100)** | Fully Supported | Fully Supported |
| **Native CJK & Arabic Shaping**| Built-in | Built-in |
| **In-App SVG/HTML Preview** | Yes (`renderSvg()`) | Yes (`render_svg()`) |
| **Setup Complexity** | Zero-rebuild (`npm install`) | Standard Cargo (`cargo add`) |

