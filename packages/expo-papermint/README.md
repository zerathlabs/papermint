# expo-papermint 🧾⚡

> **The ultra-fast, universal thermal receipt and label compilation engine for Expo and React Native.**  
> Powered by the high-performance Papermint Rust engine. Compiles receipts into ESC/POS and StarPRNT wire bytes (`Uint8Array`) in **~10 microseconds** with zero UI freezing.

---

## Features

- 🚀 **Universal Compatibility**: Works seamlessly with **Expo** (Managed Workflow, Dev Clients, EAS Build) and **Pure / Bare React Native** (CLI projects).
- ⚡ **Microsecond Execution**: Encodes receipts into raw printer wire bytes in **~10 microseconds** via native zero-copy FFI.
- 🖨️ **Dual Dialect**: Native support for Epson **ESC/POS** and Star Micronics **StarPRNT**.
- 📏 **Dynamic Widths**: Supports standard **80mm** (48 chars/line) and compact **58mm** (32 chars/line) thermal paper widths.
- 📊 **Synchronized Table Layout**: Intelligent multi-column tables with automatic word wrapping and CJK/Unicode display width calculations.
- 🇸🇦 **ZATCA Ready**: Built-in TLV Base64 QR encoder for Saudi Arabia e-invoicing Phase 1 & Phase 2.
- 📲 **Plug-and-Play Bluetooth**: Returns `Uint8Array` directly—pipe straight to Bluetooth Classic (`react-native-bluetooth-classic`) or BLE (`react-native-ble-plx`).

---

## Installation

### In an Expo App (Managed / Dev Client)
```bash
npx expo install expo-papermint
# or
pnpm add expo-papermint
```

*For local monorepo development:*
```bash
pnpm add /path/to/papermint/packages/expo-papermint
# or npm install /path/to/papermint/packages/expo-papermint
```

### In a Bare / Pure React Native App
```bash
pnpm add expo-papermint expo-modules-core
# or
npm install expo-papermint expo-modules-core
```
*React Native's community CLI autolinking will discover and link the native modules automatically upon your next `npx pod-install` or `./gradlew assembleDebug`.*


---

## Permissions Setup (`app.json`)

To stream compiled receipt bytes to physical thermal printers via Bluetooth, add the required Bluetooth permissions to your `app.json`:

```json
{
  "expo": {
    "name": "MyPOSApp",
    "slug": "my-pos-app",
    "plugins": [
      [
        "expo-build-properties",
        {
          "android": {
            "extraMavenRepos": []
          }
        }
      ]
    ],
    "android": {
      "permissions": [
        "android.permission.BLUETOOTH",
        "android.permission.BLUETOOTH_ADMIN",
        "android.permission.BLUETOOTH_CONNECT",
        "android.permission.BLUETOOTH_SCAN"
      ]
    },
    "ios": {
      "infoPlist": {
        "NSBluetoothAlwaysUsageDescription": "This app connects to thermal receipt printers via Bluetooth."
      }
    }
  }
}
```

---

## Quickstart

### Method 1: Fluent `Receipt` Builder (Recommended)

```tsx
import React from 'react';
import { Button, Alert } from 'react-native';
import { Receipt } from 'expo-papermint';

export function PrintButton() {
  const handleCompile = () => {
    // 1. Build receipt in TypeScript
    const bytes: Uint8Array = Receipt.create('80mm')
      .title('RUSTIC BAKERY & CAFE')
      .subtitle('Branch #04 - Downtown')
      .address('123 Artisan Way, Seattle, WA')
      .divider('=')
      .meta('Order #', '8492')
      .meta('Date', '2026-09-14 14:30')
      .meta('Server', 'Emily R.')
      .divider('-')
      .item('Almond Croissant', '$4.50', 2, '$9.00')
      .item('Vanilla Oat Latte', '$5.50', 1, '$5.50')
      .item('Sourdough Loaf', '$8.00', 1, '$8.00')
      .divider('-')
      .total('Subtotal', '$22.50')
      .total('Sales Tax (10%)', '$2.25')
      .total('Grand Total', '$24.75')
      .divider('=')
      .qr('https://rusticbakery.com/orders/8492')
      .barcode('849200192834')
      .footer('Thank you for visiting!')
      .openDrawer()
      .beep(1)
      .cut('full')
      .compile('escpos'); // or 'star'

    console.log(`Compiled ${bytes.length} binary bytes in microseconds!`);
    Alert.alert('Success', `Ready to send ${bytes.length} bytes to thermal printer.`);
  };

  return <Button title="Compile & Print Receipt" onPress={handleCompile} />;
}
```

---

### Method 2: One-Shot JSON Specification (`compileTicket`)

Ideal when receipt layouts are received dynamically from a REST / GraphQL backend:

```ts
import { compileTicket, TicketPayload } from 'expo-papermint';

const ticket: TicketPayload = {
  paper_width: '80mm',
  title: 'BURGER BISTRO',
  subtitle: 'Express Lane #02',
  metadata: [
    { label: 'Ticket #', value: 'B-104' },
    { label: 'Time', value: '12:45 PM' }
  ],
  items: [
    { description: 'Smash Burger Double', total: '$12.00', qty: '1' },
    { description: 'Truffle Fries', total: '$6.50', qty: '1' },
    { description: 'Craft Soda', total: '$3.50', qty: '1' }
  ],
  totals: [
    { label: 'Subtotal', value: '$22.00' },
    { label: 'Total', value: '$22.00' }
  ],
  footer: 'Save your receipt for 10% off next visit!',
  cut_mode: 'partial',
  beep: 1
};

// Returns Uint8Array
const wireBytes = compileTicket(ticket, 'escpos');
```

---

## Real-World Bluetooth Streaming Integration

### 1. Bluetooth Classic (SPP / RFCOMM) with `react-native-bluetooth-classic`

Bluetooth Classic is the standard for 95% of battery-powered mobile POS printers (PosBox, Rongta, Xprinter, Zebra, Bixolon, Epson TM-P80).

```bash
npm install react-native-bluetooth-classic
```

```ts
import RNBluetoothClassic, { BluetoothDevice } from 'react-native-bluetooth-classic';
import { Receipt } from 'expo-papermint';
import { Buffer } from 'buffer';

export async function printToBluetoothClassic(deviceAddress: string) {
  // 1. Compile receipt to binary wire bytes (Uint8Array)
  const bytes = Receipt.create('80mm')
    .title('STORE RECEIPT')
    .item('Espresso', '$3.50')
    .total('Total', '$3.50')
    .cut()
    .compile('escpos');

  // 2. Connect to printer
  const device: BluetoothDevice = await RNBluetoothClassic.connectToDevice(deviceAddress);

  // 3. Send binary bytes (Base64 encoded string for the Bluetooth driver)
  const base64Data = Buffer.from(bytes).toString('base64');
  await device.write(base64Data, 'base64');

  console.log('Receipt printed successfully!');
}
```

---

### 2. Bluetooth Low Energy (BLE) with `react-native-ble-plx`

For iOS & Android modern BLE thermal printers:

```bash
npm install react-native-ble-plx
```

```ts
import { BleManager, Device } from 'react-native-ble-plx';
import { Receipt } from 'expo-papermint';
import { Buffer } from 'buffer';

const bleManager = new BleManager();

export async function printOverBLE(
  device: Device,
  serviceUUID: string,
  characteristicUUID: string
) {
  // 1. Compile receipt
  const bytes = Receipt.create('58mm')
    .title('POP-UP BOOTH')
    .item('Latte', '$4.00')
    .total('Total', '$4.00')
    .cut()
    .compile('escpos');

  // 2. BLE MTU Chunking (split into 128-byte chunks to prevent packet drop)
  const CHUNK_SIZE = 128;
  for (let i = 0; i < bytes.length; i += CHUNK_SIZE) {
    const chunk = bytes.slice(i, i + CHUNK_SIZE);
    const base64Chunk = Buffer.from(chunk).toString('base64');

    await device.writeCharacteristicWithResponseForService(
      serviceUUID,
      characteristicUUID,
      base64Chunk
    );
  }

  console.log('BLE thermal job completed.');
}
```

---

## Saudi Arabia ZATCA E-Invoicing (Phase 1 & 2)

`expo-papermint` includes a certified TLV Base64 QR generator compliant with the Saudi ZATCA e-invoicing regulation:

```ts
import { Receipt, generateZatcaQr } from 'expo-papermint';

const zatcaQrCode = generateZatcaQr({
  sellerName: 'Al-Madina Stores LLC',
  vatNumber: '310123456700003',
  timestamp: new Date().toISOString(),
  totalAmount: '115.00',
  taxAmount: '15.00',
});

const bytes = Receipt.create('80mm')
  .title('فاتورة ضريبية مبسطة')
  .subtitle('Simplified Tax Invoice')
  .meta('الرقم الضريبي', '310123456700003')
  .item('حقيبة جلدية', '100.00 SAR')
  .total('ضريبة القيمة المضافة (15%)', '15.00 SAR')
  .total('المجموع الكلي', '115.00 SAR')
  .qr(zatcaQrCode)
  .cut()
  .compile('escpos');
```

---

## Virtual Receipt Previews (SVG & HTML)

Preview your receipts on-screen in your mobile app before sending wire bytes to the physical printer:

```tsx
import { Receipt, renderSvg, renderHtml } from 'expo-papermint';

const receipt = Receipt.create('80mm')
  .center()
  .bold(true)
  .textLn('MINT BISTRO')
  .bold(false)
  .divider('=')
  .twoColumn('1x Truffle Wagyu Burger', '$14.50')
  .twoColumn('1x Mint Cooler', '$4.50')
  .divider('-')
  .twoColumn('TOTAL', '$19.00');

// 1. Render resolution-independent SVG markup (e.g. for react-native-svg)
const svgMarkup: string = receipt.renderSvg();

// 2. Render responsive HTML markup (e.g. for react-native-webview)
const htmlMarkup: string = receipt.renderHtml();
```

---

## Flexible Multi-Column Tables

Create responsive tables with fixed-pitch or fractional column widths:

```ts
const receipt = Receipt.create('80mm')
  .tableHeader(['QTY', 'ITEM', 'PRICE'], [
    { widthFixed: 4, align: 'left' },
    { widthFraction: 0.65, align: 'left' },
    { widthFraction: 0.25, align: 'right' },
  ])
  .row(['2x', 'Wood-Fired Margherita Pizza', '$36.00'])
  .row(['1x', 'San Pellegrino Sparkling', '$3.50'])
  .clearColumns();
```

---

## API Reference

### `Receipt.create(paperWidth?: PaperWidth): Receipt`
Fluent chainable receipt builder with 100% feature parity:
- **Typography**: `.bold()`, `.underline()`, `.invert()`, `.doubleSize()`, `.doubleWidth()`, `.doubleHeight()`, `.text()`, `.textLn()`.
- **Alignment**: `.align()`, `.left()`, `.center()`, `.right()`.
- **Dividers**: `.divider()`, `.dividerDouble()`, `.dividerDotted()`, `.dividerDashed()`, `.dividerPattern()`.
- **Tables & Rows**: `.twoColumn()`, `.threeColumn()`, `.setColumns()`, `.row()`, `.tableHeader()`, `.clearColumns()`.
- **Hardware & Codes**: `.codePage()`, `.qr()`, `.barcode()`, `.image()`, `.raw()`, `.openDrawer()`, `.beep()`, `.cut()`, `.feed()`.
- **Compilation & Previews**: `.compile(dialect?)`, `.renderSvg()`, `.renderHtml()`.

### `renderSvg(ticketOrCommands, paperWidth?): string`
Direct helper to render an SVG receipt preview.

### `renderHtml(ticketOrCommands, paperWidth?): string`
Direct helper to render an HTML receipt preview.

### `compileTicket(ticket: TicketPayload, dialect?: Dialect): Uint8Array`
Compiles a structured ticket payload into binary wire bytes.
- `dialect`: `'escpos'` (default) or `'star'`.

### `generateZatcaQr(params: ZatcaQrParams): string`
Generates compliant ZATCA Phase 1 & 2 TLV Base64 QR strings.

---

## Architecture

```
┌────────────────────────────────────────────────────────┐
│ Expo / React Native App (TypeScript)                  │
│ const bytes = compileTicket(ticket, 'escpos');         │
└──────────────────────────┬─────────────────────────────┘
                           │ JSON payload
┌──────────────────────────▼─────────────────────────────┐
│ Native FFI Bridge (Expo Module: Kotlin / Swift)        │
│ Calls: papermint_compile_json() in Rust                │
└──────────────────────────┬─────────────────────────────┘
                           │ ~10 microseconds execution
┌──────────────────────────▼─────────────────────────────┐
│ Return: Uint8Array ([0x1B, 0x40, 0x1B, 0x74...])       │
│ Stream to Bluetooth / Wi-Fi thermal printer socket     │
└────────────────────────────────────────────────────────┘
```

---

## License

MIT © [Papermint Team](https://github.com/zerathlabs/papermint)
