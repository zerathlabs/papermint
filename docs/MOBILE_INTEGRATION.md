# Mobile POS Integration Guide: React Native, Expo & Flutter

`papermint-mobile` is a high-performance C-compatible foreign function interface (FFI) and receipt compilation engine. It enables mobile applications (**React Native**, **Expo SDK 56+**, **Flutter**, **iOS Swift**, and **Android Kotlin**) to generate pixel-perfect thermal receipts in **microseconds**, returning zero-copy raw byte buffers (`Uint8Array` / `byte[]`) ready to stream to portable thermal printers.

---

## 1. Why This Architecture Wins for Mobile POS

Portable thermal printers in food trucks, pop-up shops, and mobile retail connect over different hardware transports:
* **Bluetooth Classic (SPP / RFCOMM)**: Used by 80%+ of budget 58mm/80mm printers on Android.
* **Bluetooth Low Energy (BLE)**: Used on iPhones, iPads, and modern portable printers (e.g. Star SM-L200).
* **Wi-Fi / Ethernet**: Used by kitchen and bar countertop printers (RAW Port 9100).
* **USB-C OTG**: Used by fixed Android tablet stands.

Attempting to run a rigid Bluetooth stack inside Rust on mobile leads to severe permission bugs, battery drain, and iOS/Android lifecycle crashes.

Instead, `papermint` follows the **Turbo Engine Pattern**:
1. **Rust (`papermint-mobile`)** does the heavy math: CJK/Unicode column layout, photo dithering, dynamic word wrapping, and ESC/POS or StarPRNT byte generation in **~10 microseconds**.
2. **Mobile OS (React Native / Expo)** handles the hardware: OS Bluetooth permissions, device-picker modals, and direct socket writing.

---

## 2. Compiling the Native Library

Build the static and dynamic libraries for your mobile target architectures:

```bash
# Build desktop / development binaries
cargo build --package papermint-mobile --release

# Android (arm64-v8a, armeabi-v7a, x86_64) via cargo-ndk
cargo ndk -t arm64-v8a -t armeabi-v7a build --package papermint-mobile --release

# iOS (aarch64-apple-ios, aarch64-apple-ios-sim) via cargo-lipo
cargo build --package papermint-mobile --target aarch64-apple-ios --release
```

The output contains:
* **Header**: `crates/papermint-mobile/include/papermint.h`
* **Static Library**: `libpapermint_mobile.a`
* **Dynamic Library**: `libpapermint_mobile.so` (Android) / `libpapermint_mobile.dylib` (iOS/macOS)

---

## 3. Expo SDK 56+ (Inline Modules)

Expo SDK 56 introduced **Inline Modules**, allowing you to write Swift and Kotlin directly in your project without maintaining separate npm packages.

### Step 1: Enable Inline Modules in `app.json`

```json
{
  "expo": {
    "name": "MyMobilePOS",
    "slug": "my-mobile-pos",
    "version": "1.0.0",
    "experiments": {
      "inlineModules": {
        "watchedDirectories": ["modules"]
      }
    }
  }
}
```

### Step 2: Swift Implementation (iOS)

Create `modules/PapermintModule.swift`:

```swift
import ExpoModulesCore

public class PapermintModule: Module {
  public func definition() -> ModuleDefinition {
    Name("Papermint")

    Function("compileTicket") { (jsonString: String, dialect: Int) -> [UInt8] in
      var outLen: Int = 0
      guard let ptr = papermint_compile_json(jsonString, UInt8(dialect), &outLen) else {
        return []
      }
      defer { papermint_bytes_free(ptr, outLen) }

      let buffer = UnsafeBufferPointer(start: ptr, count: outLen)
      return Array(buffer)
    }
  }
}
```

### Step 3: Kotlin Implementation (Android)

Create `modules/PapermintModule.kt`:

```kotlin
package com.mymobilepos.modules

import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition

class PapermintModule : Module() {
  companion object {
    init {
      System.loadLibrary("papermint_mobile")
    }
  }

  private external fun papermintCompileJson(jsonStr: String, dialect: Int): ByteArray

  override fun definition() = ModuleDefinition {
    Name("Papermint")

    Function("compileTicket") { jsonString: String, dialect: Int ->
      papermintCompileJson(jsonString, dialect)
    }
  }
}
```

---

## 4. React Native / TypeScript Usage

In your React Native / Expo application:

```typescript
import { requireNativeModule } from 'expo-modules-core';
const Papermint = requireNativeModule('Papermint');

// 1. Define your receipt ticket
const ticket = {
  paper_width: "80mm",
  title: "BLUE CAFE & ROASTERY",
  subtitle: "Order #4092",
  address: "123 Main St, Austin TX",
  metadata: [
    { label: "Date", value: new Date().toLocaleDateString() },
    { label: "Cashier", value: "Alex" }
  ],
  items: [
    { qty: "1", description: "Iced Oat Latte", price: "$5.50", total: "$5.50" },
    { qty: "2", description: "Avocado Toast", price: "$9.00", total: "$18.00" }
  ],
  divider_style: "=",
  totals: [
    { label: "Subtotal", value: "$23.50" },
    { label: "Tax (8.25%)", value: "$1.94" },
    { label: "TOTAL", value: "$25.44" }
  ],
  qr: "https://pay.bluecafe.com/4092",
  barcode: "40922544",
  footer: "Thank you for supporting local business!",
  cut_mode: "full",
  open_drawer: true,
  beep: 1
};

// 2. Compile into wire bytes in ~10 microseconds!
// dialect: 0 = ESC/POS (Epson/Generic), 1 = StarPRNT (Star Micronics)
const rawBytes: number[] = Papermint.compileTicket(JSON.stringify(ticket), 0);
const wireBuffer = new Uint8Array(rawBytes);
```

---

## 5. Streaming to Mobile Printers

### A. Bluetooth Classic (SPP / Android)

Using `react-native-bluetooth-classic`:

```typescript
import RNBluetoothClassic from 'react-native-bluetooth-classic';

async function printViaBluetoothClassic(address: string, data: Uint8Array) {
  const device = await RNBluetoothClassic.connectToDevice(address);
  // Base64 encode wire buffer for Bluetooth socket
  const base64Data = Buffer.from(data).toString('base64');
  await device.write(base64Data, 'base64');
  console.log("✅ Printed successfully over Bluetooth Classic!");
}
```

### B. Bluetooth Low Energy (BLE / iOS & Android)

Using `react-native-ble-plx`:

```typescript
import { BleManager } from 'react-native-ble-plx';

const ble = new BleManager();

async function printViaBle(
  deviceId: string,
  serviceUuid: string,
  characteristicUuid: string,
  data: Uint8Array
) {
  const device = await ble.connectToDevice(deviceId);
  await device.discoverAllServicesAndCharacteristics();

  // BLE MTU Chunking: Send in 256 or 512 byte packets
  const chunkSize = 256;
  for (let i = 0; i < data.length; i += chunkSize) {
    const chunk = data.subarray(i, i + chunkSize);
    const base64Chunk = Buffer.from(chunk).toString('base64');
    await device.writeCharacteristicWithResponseForService(
      serviceUuid,
      characteristicUuid,
      base64Chunk
    );
  }
  console.log("✅ Printed successfully over BLE!");
}
```

### C. Wi-Fi / Ethernet (RAW TCP Port 9100)

Using `react-native-tcp-socket`:

```typescript
import TcpSocket from 'react-native-tcp-socket';

function printViaNetwork(ip: string, port = 9100, data: Uint8Array) {
  const client = TcpSocket.createConnection({ host: ip, port }, () => {
    client.write(Buffer.from(data));
    client.end();
  });
}
```

---

## 6. Fluent C-ABI Function Reference

If building custom C++ JSI TurboModules or using Flutter `dart:ffi`:

| Function | Description |
| :--- | :--- |
| `papermint_receipt_create(paper_width)` | Creates handle (`0` = 80mm, `1` = 58mm). |
| `papermint_receipt_free(handle)` | Deallocates receipt handle. |
| `papermint_receipt_init(handle)` | Appends hardware reset sequence (`ESC @`). |
| `papermint_receipt_text_ln(handle, text)` | Appends text followed by newline. |
| `papermint_receipt_align(handle, align)` | Sets alignment (`0`=Left, `1`=Center, `2`=Right). |
| `papermint_receipt_bold(handle, enable)` | Enables/disables bold emphasis. |
| `papermint_receipt_underline(handle, mode)`| Sets underline (`0`=Off, `1`=Single, `2`=Double). |
| `papermint_receipt_divider(handle, style)` | Appends full-width divider line. |
| `papermint_receipt_table_row(handle, cols, n)` | Appends synchronized CJK/Unicode multi-column row. |
| `papermint_receipt_qr(handle, content)` | Appends 2D QR Code. |
| `papermint_receipt_barcode(handle, content)` | Appends 1D Code 128 Barcode. |
| `papermint_receipt_cut(handle, partial)` | Appends paper cut command. |
| `papermint_receipt_beep(handle, count, dur)` | Triggers buzzer beep. |
| `papermint_receipt_open_drawer(handle)` | Triggers cash drawer kickout pulse. |
| `papermint_receipt_encode(handle, dialect, out_len)` | Compiles commands into raw byte buffer. |
| `papermint_compile_json(json, dialect, out_len)` | Compiles JSON ticket into raw byte buffer. |
| `papermint_bytes_free(ptr, len)` | Frees buffer returned by encode/compile functions. |
