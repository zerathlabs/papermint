# 🌿 papermint

[![npm](https://img.shields.io/npm/v/papermint.svg)](https://www.npmjs.com/package/papermint)
[![Documentation](https://img.shields.io/badge/docs-papermint.zerathlabs.com-388E3C)](https://papermint.zerathlabs.com)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/zerathlabs/papermint)

> **High-performance Node.js & TypeScript bindings for thermal receipt printing.**  
> Powered by the native `papermint` Rust engine compiled with **N-API (`napi-rs`)**.

---

## Features

- ⚡ **Zero-Overhead Native Speed**: Formats receipts and generates binary wire buffers in microseconds.
- 📐 **Typographic Monospace Mathematics**: Auto-balanced multi-column tables with synchronized multiline wrapping and full **Unicode East Asian Width (UAX #11)** calculation (CJK, Arabic, emoji).
- 🖨️ **Multi-Vendor Dialects**: Full native binary encoding for **Epson ESC/POS** and **StarPRNT** (Star Micronics).
- 🔌 **Device Discovery**: Discover connected USB thermal printers and serial / COM ports directly from Node.js.
- 🌐 **Async TCP & In-Memory Mocking**: Transmit directly to network printers on Port 9100 or test headlessly with `Printer.mock()`.
- 🛡️ **Type-Safe**: Complete TypeScript type definitions (`index.d.ts`) included out of the box.

---

## Installation

```bash
pnpm add papermint
# or
npm install papermint
```

---

## Quickstart

### 1. Build a Receipt & Print via TCP

```typescript
import { Receipt, Printer } from 'papermint';

const receipt = new Receipt('80mm')
  .init()
  .center()
  .bold(true)
  .doubleSize(true)
  .textLn('MINT BISTRO')
  .doubleSize(false)
  .bold(false)
  .textLn('123 Main Street')
  .dividerDouble()
  .tableHeader(
    ['QTY', 'DESCRIPTION', 'PRICE', 'TOTAL'],
    [
      { widthFixed: 4, align: 'left' },
      { widthFraction: 0.50, align: 'left' },
      { widthFraction: 0.22, align: 'right' },
      { widthFraction: 0.24, align: 'right' },
    ]
  )
  .row(['2x', 'Truffle Wagyu Burger with Onions', '$14.50', '$29.00'])
  .row(['1x', 'Wood-Fired Margherita Pizza', '$18.00', '$18.00'])
  .dividerDashed()
  .twoColumn('BALANCE DUE:', '$47.00')
  .dividerDotted()
  .qr('https://pay.mintbistro.com/1042')
  .feed(2)
  .cutFull();

// Print over TCP network:
const printer = Printer.tcp('192.168.1.100:9100', 'escpos');
await printer.print(receipt);
```

### 2. Direct Wire Encoding to Node.js Buffers

If you are transmitting over Bluetooth, WebSockets, or a custom socket:

```typescript
import { Receipt } from 'papermint';

const receipt = new Receipt('80mm').init().textLn('Hello World').cut();

// Returns raw Node.js Buffer with ESC/POS binary opcodes:
const escposBytes: Buffer = receipt.encode('escpos');

// Returns raw Node.js Buffer with StarPRNT binary opcodes:
const starBytes: Buffer = receipt.encode('star');
```

### 3. Hardware Device Discovery

```typescript
import { availablePorts, listPrinters } from 'papermint';

// List connected USB thermal printers:
const usbPrinters = listPrinters();
console.log('Detected USB Printers:', usbPrinters);

// List available serial / COM ports:
const serialPorts = availablePorts();
console.log('Available Serial Ports:', serialPorts);
```

### 4. Unit Testing with In-Memory Mock Printer

```typescript
import { Receipt, Printer } from 'papermint';

const mock = Printer.mock('escpos');
const receipt = new Receipt('80mm').init().textLn('Test').cut();

await mock.print(receipt);

const recorded: Buffer = await mock.getRecordedBytes();
console.log(`Recorded ${recorded.length} wire bytes in memory.`);
```

---

## Documentation

Comprehensive guides, API specifications, and multi-platform tutorials are available at:  
👉 **[https://papermint.zerathlabs.com](https://papermint.zerathlabs.com)**

---

## License

Dual-licensed under [MIT OR Apache-2.0](https://github.com/zerathlabs/papermint).
