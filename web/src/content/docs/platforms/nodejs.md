---
title: "Node.js & TypeScript"
description: "Native N-API bindings for @zerathlabs/papermint"
---

# 📦 Node.js & TypeScript Bindings

`@zerathlabs/papermint` provides native, zero-overhead Node.js and TypeScript bindings for `papermint` compiled using **N-API (`napi-rs`)**.

It runs with native C++ speed directly in Node.js and Electron without requiring Python or native build chains.

---

## Installation

```bash
pnpm add @zerathlabs/papermint
# or
npm install @zerathlabs/papermint
```

---

## Key Features

* **Fluent TypeScript Receipt Builder**: Chain commands with full auto-complete.
* **Direct Wire Encoding**: Outputs raw Node.js `Buffer` objects in microseconds.
* **Device Discovery**: Discover connected USB thermal printers and serial ports.
* **Network & In-Memory Mock Printers**: Print directly over TCP or simulate receipts in memory for testing.

---

## Code Examples

### 1. Formatting a Receipt & Printing via TCP

```typescript
import { Receipt, Printer } from '@zerathlabs/papermint';

const receipt = new Receipt('80mm')
  .init()
  .center()
  .bold(true)
  .textLn('THE MINT BISTRO')
  .bold(false)
  .textLn('Order #4092')
  .divider('=')
  .left()
  .tableRow(['1x', 'Avocado Toast', '$9.00'])
  .tableRow(['2x', 'Cold Brew', '$8.00'])
  .divider('-')
  .bold(true)
  .tableRow(['TOTAL', '', '$17.00'])
  .bold(false)
  .divider('=')
  .center()
  .qr('https://pay.mintbistro.com/4092')
  .feed(2)
  .cut();

// Print over TCP network:
const printer = Printer.tcp('192.168.1.100:9100', 'escpos');
await printer.print(receipt);
```

### 2. Device Auto-Discovery

```typescript
import { availablePorts, listPrinters } from '@zerathlabs/papermint';

// 1. List USB thermal printers:
const usbPrinters = listPrinters();
console.log('Found USB Printers:', usbPrinters);

// 2. List available serial/COM ports:
const serialPorts = availablePorts();
console.log('Available Serial Ports:', serialPorts);
```

### 3. Binary Wire Buffer Generation (for Bluetooth or Custom Sockets)

```typescript
// Encode directly to a binary Node.js Buffer in microseconds
const rawEscposBytes: Buffer = receipt.encode('escpos');
const rawStarBytes: Buffer = receipt.encode('star');

// Stream to your Bluetooth or WebSocket connection
myBluetoothSocket.write(rawEscposBytes);
```
