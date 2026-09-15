---
title: "Flutter Integration Guide"
description: "High-performance thermal receipt printing in Flutter using dart:ffi, zero-copy native bytes, and Bluetooth/BLE/TCP streaming."
---

# 🎯 Flutter POS Integration Guide (`dart:ffi`)

`papermint-mobile` provides high-performance C foreign function interface (FFI) bindings for **Flutter** apps on Android and iOS.

Instead of relying on fragile pure-Dart receipt packages that break on complex table layouts, Arabic shaping, or StarPRNT dialects, Papermint runs a compiled Rust engine that formats receipts and compiles ESC/POS or StarPRNT wire bytes in **~10 microseconds**, returning a zero-copy byte buffer (`Uint8List`) ready for Bluetooth, BLE, or TCP transmission.

---

## 1. Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│ Flutter App (Dart UI / State)                               │
│  • Defines receipt ticket (Map<String, dynamic>)            │
│  • Calls Papermint.compile(ticket)                          │
└──────────────────────────────┬──────────────────────────────┘
                               │ dart:ffi (Pointer / C-ABI)
                               ▼
┌─────────────────────────────────────────────────────────────┐
│ libpapermint_mobile (Rust Engine)                           │
│  • UAX #11 CJK & Arabic WPC1256/CP864 shaping               │
│  • Table column math & line wraps                           │
│  • ESC/POS & StarPRNT dialect byte encoding (~10 µs)        │
└──────────────────────────────┬──────────────────────────────┘
                               │ Returns Uint8List raw bytes
                               ▼
┌─────────────────────────────────────────────────────────────┐
│ Hardware Transports (Flutter Plugins / Sockets)             │
│  • Bluetooth Classic (SPP): print_bluetooth_thermal         │
│  • Bluetooth Low Energy:    flutter_blue_plus               │
│  • Wi-Fi / Ethernet:        dart:io Socket (Port 9100)      │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Step 1: Getting Native Binaries

You can integrate the native libraries using pre-compiled binaries (no Rust setup required) or by compiling from source.

### Option A: Pre-built Binaries (*Recommended*)

Pre-compiled production binaries are attached to every [GitHub Release](https://github.com/zerathlabs/papermint/releases). You do **not** need Rust or the Android NDK installed.

#### 1. Android
1. Download `papermint-mobile-android-v*.zip` from the latest release.
2. Extract the `jniLibs` directory directly into your Flutter Android folder:
   ```
   android/
   └── app/
       └── src/
           └── main/
               └── jniLibs/
                   ├── arm64-v8a/libpapermint_mobile.so
                   ├── armeabi-v7a/libpapermint_mobile.so
                   └── x86_64/libpapermint_mobile.so
   ```

#### 2. iOS
1. Download `papermint-mobile-ios-v*.zip` from the latest release.
2. Unzip `PapermintMobile.xcframework` and `papermint.h`.
3. Open `ios/Runner.xcworkspace` in Xcode, and drag `PapermintMobile.xcframework` into **Runner** $\rightarrow$ **General** $\rightarrow$ **Frameworks, Libraries, and Embedded Content** (set to *Embed & Sign*).
4. Copy `papermint.h` into `ios/Runner/` and add `#import "papermint.h"` to your `Runner-Bridging-Header.h`.

---

### Option B: Build from Source (*Advanced*)

If you are modifying the Rust receipt engine or building custom architectures:

#### 1. Android Build via `cargo-ndk`:
```bash
# Inside crates/papermint-mobile
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o ../../android/app/src/main/jniLibs build --release
```

#### 2. iOS Build via `cargo`:
```bash
# Compile static libraries for device and simulator
cargo build --package papermint-mobile --target aarch64-apple-ios --release
cargo build --package papermint-mobile --target aarch64-apple-ios-sim --release
```
In Xcode under **Runner** $\rightarrow$ **Build Phases** $\rightarrow$ **Link Binary With Libraries**, add `libpapermint_mobile.a`.

---

## 3. Step 2: Add Dependencies

In your Flutter app's `pubspec.yaml`, add `ffi`:

```bash
flutter pub add ffi
```

---

## 4. Step 3: The Complete `Papermint` Dart Service

Create `lib/services/papermint.dart` in your Flutter project. This single file provides type-safe Dart bindings, memory management, ticket compilation, and virtual SVG/HTML previews:

```dart
import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:typed_data';
import 'package:ffi/ffi.dart';

/// Dialect selector for thermal printers.
enum PrinterDialect {
  escpos, // 0: Epson, Rongta, Bixolon, Xprinter, POS-58/80
  star,   // 1: Star Micronics (TSP100, TSP650II, mC-Print)
}

// C-ABI Function Signatures
typedef _NativeCompileJson = Pointer<Uint8> Function(
    Pointer<Utf8> json, Uint8 dialect, Pointer<Size> outLen);
typedef _DartCompileJson = Pointer<Uint8> Function(
    Pointer<Utf8> json, int dialect, Pointer<Size> outLen);

typedef _NativeBytesFree = Void Function(Pointer<Uint8> ptr, Size len);
typedef _DartBytesFree = void Function(Pointer<Uint8> ptr, int len);

typedef _NativeCompileJsonStr = Pointer<Utf8> Function(Pointer<Utf8> json);
typedef _DartCompileJsonStr = Pointer<Utf8> Function(Pointer<Utf8> json);

typedef _NativeStringFree = Void Function(Pointer<Utf8> ptr);
typedef _DartStringFree = void Function(Pointer<Utf8> ptr);

/// High-performance Papermint printing service for Flutter.
class Papermint {
  static final DynamicLibrary _lib = () {
    if (Platform.isAndroid) {
      return DynamicLibrary.open('libpapermint_mobile.so');
    } else if (Platform.isIOS || Platform.isMacOS) {
      return DynamicLibrary.process(); // Statically linked into Runner
    } else if (Platform.isWindows) {
      return DynamicLibrary.open('papermint_mobile.dll');
    } else if (Platform.isLinux) {
      return DynamicLibrary.open('libpapermint_mobile.so');
    }
    throw UnsupportedError('Unsupported platform for Papermint');
  }();

  static final _compileJson =
      _lib.lookupFunction<_NativeCompileJson, _DartCompileJson>('papermint_compile_json');
  static final _bytesFree =
      _lib.lookupFunction<_NativeBytesFree, _DartBytesFree>('papermint_bytes_free');

  static final _compileSvg =
      _lib.lookupFunction<_NativeCompileJsonStr, _DartCompileJsonStr>('papermint_compile_json_svg');
  static final _compileHtml =
      _lib.lookupFunction<_NativeCompileJsonStr, _DartCompileJsonStr>('papermint_compile_json_html');
  static final _stringFree =
      _lib.lookupFunction<_NativeStringFree, _DartStringFree>('papermint_string_free');

  /// Compiles a JSON ticket map into raw thermal printer wire bytes.
  /// Execution time is typically 10-20 microseconds.
  static Uint8List compileTicket(
    Map<String, dynamic> ticket, {
    PrinterDialect dialect = PrinterDialect.escpos,
  }) {
    final jsonStr = jsonEncode(ticket);
    final jsonUtf8 = jsonStr.toNativeUtf8();
    final outLenPtr = calloc<Size>();

    try {
      final dialectCode = dialect == PrinterDialect.star ? 1 : 0;
      final bytesPtr = _compileJson(jsonUtf8, dialectCode, outLenPtr);
      final len = outLenPtr.value;

      if (bytesPtr.address == 0 || len == 0) {
        return Uint8List(0);
      }

      // Copy native bytes to Dart Uint8List
      final result = Uint8List.fromList(bytesPtr.asTypedList(len));

      // Safely release native heap allocation
      _bytesFree(bytesPtr, len);
      return result;
    } finally {
      calloc.free(jsonUtf8);
      calloc.free(outLenPtr);
    }
  }

  /// Generates a pixel-perfect SVG vector string for virtual receipt preview.
  static String renderSvg(Map<String, dynamic> ticket) {
    final jsonStr = jsonEncode(ticket);
    final jsonUtf8 = jsonStr.toNativeUtf8();

    try {
      final strPtr = _compileSvg(jsonUtf8);
      if (strPtr.address == 0) return '';
      final result = strPtr.toDartString();
      _stringFree(strPtr);
      return result;
    } finally {
      calloc.free(jsonUtf8);
    }
  }

  /// Generates an HTML preview string of the receipt.
  static String renderHtml(Map<String, dynamic> ticket) {
    final jsonStr = jsonEncode(ticket);
    final jsonUtf8 = jsonStr.toNativeUtf8();

    try {
      final strPtr = _compileHtml(jsonUtf8);
      if (strPtr.address == 0) return '';
      final result = strPtr.toDartString();
      _stringFree(strPtr);
      return result;
    } finally {
      calloc.free(jsonUtf8);
    }
  }
}
```

---

## 5. Step 4: Constructing a Receipt Payload

You can format receipts using standard Dart maps:

```dart
final ticket = {
  "paper_width": "80mm", // or "58mm"
  "title": "AL-NAKHEEL LUXURY CAFE",
  "subtitle": "Order #4092",
  "address": "King Fahd Rd, Riyadh 12214",
  "metadata": [
    {"label": "Date", "value": "2026-09-15 14:30"},
    {"label": "Cashier", "value": "Fahad Al-Otaibi"}
  ],
  "items": [
    {"qty": "1x", "description": "Arabic Cardamom Coffee", "price": "18.00", "total": "18.00"},
    {"qty": "2x", "description": "Saffron Milk Cake", "price": "24.00", "total": "48.00"},
    {"qty": "1x", "description": "Sparkling Water", "price": "6.00", "total": "6.00"}
  ],
  "divider_style": "=",
  "totals": [
    {"label": "Subtotal", "value": "72.00 SAR"},
    {"label": "VAT (15%)", "value": "10.80 SAR"},
    {"label": "TOTAL", "value": "82.80 SAR"}
  ],
  "qr": "https://zatca.gov.sa/einvoice/verify?id=4092",
  "barcode": "40928280",
  "footer": "Thank you for visiting! / شكراً لزيارتكم",
  "cut_mode": "full",
  "open_drawer": true,
  "beep": 1
};

// Generate binary ESC/POS wire bytes in ~10 microseconds
final Uint8List wireBytes = Papermint.compileTicket(ticket, dialect: PrinterDialect.escpos);
```

---

## 6. Step 5: Streaming Wire Bytes to Thermal Printers

### A. Wi-Fi / Ethernet LAN (Native Dart `Socket` — Port 9100)

No third-party packages required! Dart's built-in `dart:io` sends raw bytes directly to network thermal printers:

```dart
import 'dart:io';
import 'dart:typed_data';

Future<void> printViaNetwork({
  required String ipAddress,
  int port = 9100,
  required Uint8List bytes,
}) async {
  final socket = await Socket.connect(ipAddress, port, timeout: const Duration(seconds: 5));
  try {
    socket.add(bytes);
    await socket.flush();
  } finally {
    await socket.close();
  }
  print("✅ Receipt printed successfully via LAN Port 9100!");
}
```

---

### B. Bluetooth Classic (SPP / Android)

For Android mobile POS devices and budget thermal printers (e.g. POS-58, POS-80, PT-210), use `print_bluetooth_thermal`:

```bash
flutter pub add print_bluetooth_thermal
```

```dart
import 'package:print_bluetooth_thermal/print_bluetooth_thermal.dart';

Future<void> printViaBluetoothClassic({
  required String macAddress,
  required Uint8List bytes,
}) async {
  final isConnected = await PrintBluetoothThermal.connect(macPrinterAddress: macAddress);
  if (!isConnected) {
    throw Exception("Failed to connect to Bluetooth printer");
  }

  // Stream raw bytes directly to the RFCOMM socket
  final result = await PrintBluetoothThermal.writeBytes(bytes.toList());
  print("✅ Printed over Bluetooth Classic: $result");
}
```

---

### C. Bluetooth Low Energy (BLE / iOS & Android)

For portable printers operating over BLE (e.g. Star SM-L200, Zebra, Rongta BLE), use `flutter_blue_plus`:

```bash
flutter pub add flutter_blue_plus
```

```dart
import 'dart:typed_data';
import 'package:flutter_blue_plus/flutter_blue_plus.dart';

Future<void> printViaBle({
  required BluetoothDevice device,
  required Guid serviceUuid,
  required Guid characteristicUuid,
  required Uint8List bytes,
}) async {
  await device.connect();

  // Discover GATT services and target write characteristic
  final services = await device.discoverServices();
  final service = services.firstWhere((s) => s.uuid == serviceUuid);
  final char = service.characteristics.firstWhere((c) => c.uuid == characteristicUuid);

  // Negotiate higher MTU if supported (up to 512 bytes)
  if (Platform.isAndroid) {
    await device.requestMtu(512);
  }

  // Chunk bytes to respect BLE packet size (default 244 or MTU - 3)
  const chunkSize = 200;
  for (var i = 0; i < bytes.length; i += chunkSize) {
    final end = (i + chunkSize < bytes.length) ? i + chunkSize : bytes.length;
    final chunk = bytes.sublist(i, end);
    await char.write(chunk, withoutResponse: false);
  }

  print("✅ Printed successfully over BLE!");
}
```

---

## 7. Step 6: In-App Virtual Preview in Flutter

Before printing to physical paper, render the exact receipt output on screen using `flutter_svg`:

```bash
flutter pub add flutter_svg
```

```dart
import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'services/papermint.dart';

class ReceiptPreviewScreen extends StatelessWidget {
  final Map<String, dynamic> ticket;

  const ReceiptPreviewScreen({super.key, required this.ticket});

  @override
  Widget build(BuildContext context) {
    // Generate virtual vector preview in microseconds
    final String svgString = Papermint.renderSvg(ticket);

    return Scaffold(
      appBar: AppBar(title: const Text('Receipt Preview')),
      body: Center(
        child: SingleChildScrollView(
          padding: const Duration(milliseconds: 16) != null
              ? const EdgeInsets.all(16.0)
              : EdgeInsets.zero,
          child: Container(
            decoration: BoxDecoration(
              color: Colors.white,
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withOpacity(0.15),
                  blurRadius: 10,
                  offset: const Offset(0, 4),
                )
              ],
            ),
            child: SvgPicture.string(svgString),
          ),
        ),
      ),
    );
  }
}
```

---

## 8. Summary & Performance Comparison

| Feature | Legacy Dart Receipt Packages | **Papermint + dart:ffi** |
| :--- | :--- | :--- |
| **Compilation Speed** | 50–200 ms (GC-heavy) | **~10–20 µs (zero-copy Rust)** |
| **Dialect Parity** | ESC/POS only (often partial) | **Epson ESC/POS + StarPRNT** |
| **Arabic & CJK Shaping** | Broken / missing glyph shaper | **WPC1256 & CP864 hardware native** |
| **Virtual SVG/HTML Preview**| Not supported | **Full vector renderer included** |
| **Memory Footprint** | Heap allocations per command | **Single contiguous native buffer** |

