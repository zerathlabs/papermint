---
title: "Native iOS, Android & KMP Guide"
description: "High-performance POS receipt printing in native Swift (iOS), Kotlin (Android), and Kotlin Multiplatform (KMP)."
---

# 📱 Native POS Integration Guide: Swift, Kotlin & KMP

`papermint-mobile` exposes a zero-overhead standard **C-ABI (`papermint.h`)** and **Android JNI bridge**, enabling enterprise POS teams to integrate high-performance receipt formatting into **Native iOS (Swift)**, **Native Android (Kotlin/Java)**, and **Kotlin Multiplatform (KMP)** applications.

Receipt compilation occurs in **~10 microseconds** in Rust, returning native byte containers (`Data` in Swift, `ByteArray` in Kotlin) ready for zero-copy streaming to physical thermal printers.

---

## Part 1: Native iOS (Swift)

### 1. Adding the Native Library to Xcode

#### Option A: Pre-built XCFramework (*Recommended*)
1. Download `papermint-mobile-ios-v*.zip` from the latest [GitHub Releases](https://github.com/zerathlabs/papermint/releases).
2. Unzip `PapermintMobile.xcframework` and `papermint.h`.
3. In Xcode, drag `PapermintMobile.xcframework` into your target's **General** $\rightarrow$ **Frameworks, Libraries, and Embedded Content** (set to *Embed & Sign*).
4. Add `#import "papermint.h"` to your project's **Bridging Header** (`YourApp-Bridging-Header.h`).

#### Option B: Build Static Library from Source (*Advanced*)
```bash
# Physical devices (iPhone, iPad)
cargo build --package papermint-mobile --target aarch64-apple-ios --release

# iOS Simulator (Apple Silicon Macs)
cargo build --package papermint-mobile --target aarch64-apple-ios-sim --release
```
In Xcode under **Build Phases** $\rightarrow$ **Link Binary With Libraries**, add `libpapermint_mobile.a`, and add `#import "papermint.h"` to your Bridging Header.

---

### 2. Swift Wrapper (`Papermint.swift`)

Create a Swift wrapper providing clean APIs and automatic memory cleanup:

```swift
import Foundation

public enum PrinterDialect: UInt8 {
    case escpos = 0  // Epson, Rongta, Bixolon, Star ESC/POS emulation
    case star   = 1  // Star Micronics Line Mode
}

public final class Papermint {

    /// Compiles a JSON receipt ticket into raw thermal printer wire bytes.
    /// Execution time is typically 10-20 microseconds.
    public static func compileTicket(jsonString: String, dialect: PrinterDialect = .escpos) -> Data {
        guard let cJson = jsonString.cString(using: .utf8) else {
            return Data()
        }

        var outLen: Int = 0
        guard let ptr = papermint_compile_json(cJson, dialect.rawValue, &outLen), outLen > 0 else {
            return Data()
        }

        // Wrap pointer into Swift Data and ensure native memory is freed
        defer { papermint_bytes_free(ptr, outLen) }
        return Data(bytes: ptr, count: outLen)
    }

    /// Renders a virtual SVG vector graphic preview of the receipt.
    public static func renderSvg(jsonString: String) -> String? {
        guard let cJson = jsonString.cString(using: .utf8),
              let ptr = papermint_compile_json_svg(cJson) else {
            return nil
        }
        defer { papermint_string_free(ptr) }
        return String(cString: ptr)
    }

    /// Renders a responsive HTML preview of the receipt.
    public static func renderHtml(jsonString: String) -> String? {
        guard let cJson = jsonString.cString(using: .utf8),
              let ptr = papermint_compile_json_html(cJson) else {
            return nil
        }
        defer { papermint_string_free(ptr) }
        return String(cString: ptr)
    }
}
```

---

### 3. Streaming to Printers on iOS

#### A. Bluetooth Low Energy (CoreBluetooth)

```swift
import CoreBluetooth

class BleThermalPrinter: NSObject, CBPeripheralDelegate {
    private var peripheral: CBPeripheral?
    private var writeCharacteristic: CBCharacteristic?

    func sendReceipt(data: Data) {
        guard let peripheral = peripheral, let char = writeCharacteristic else { return }

        // CoreBluetooth MTU chunking (default 244 bytes for BLE)
        let chunkSize = 244
        var offset = 0

        while offset < data.count {
            let length = min(chunkSize, data.count - offset)
            let chunk = data.subdata(in: offset..<(offset + length))
            peripheral.writeValue(chunk, for: char, type: .withResponse)
            offset += length
        }
    }
}
```

#### B. Wi-Fi / Ethernet LAN (Network.framework)

```swift
import Network

func printViaNetwork(ip: String, port: UInt16 = 9100, data: Data) {
    let host = NWEndpoint.Host(ip)
    let port = NWEndpoint.Port(rawValue: port)!
    let connection = NWConnection(host: host, port: port, using: .tcp)

    connection.stateUpdateHandler = { state in
        if case .ready = state {
            connection.send(content: data, completion: .contentProcessed({ error in
                if let error = error {
                    print("❌ Print error: \(error)")
                } else {
                    print("✅ Receipt printed successfully over TCP!")
                }
                connection.cancel()
            }))
        }
    }
    connection.start(queue: .global())
}
```

---

## Part 2: Native Android (Kotlin / Java)

### 1. Adding Native Binaries (`jniLibs`)

#### Option A: Pre-built Binaries (*Recommended*)
1. Download `papermint-mobile-android-v*.zip` from the latest [GitHub Releases](https://github.com/zerathlabs/papermint/releases).
2. Extract the `jniLibs` folder directly into your Android app directory:
   ```
   app/src/main/jniLibs/
   ├── arm64-v8a/libpapermint_mobile.so
   ├── armeabi-v7a/libpapermint_mobile.so
   └── x86_64/libpapermint_mobile.so
   ```

#### Option B: Build from Source (*Advanced*)
```bash
# Inside crates/papermint-mobile
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o app/src/main/jniLibs build --release
```

---

### 2. Kotlin Wrapper (`Papermint.kt`)

`crates/papermint-mobile` exports direct C-ABI methods as well as standard JNI bridges:

```kotlin
package com.mypos.printing

class Papermint {
    companion object {
        init {
            System.loadLibrary("papermint_mobile")
        }

        const val DIALECT_ESCPOS = 0
        const val DIALECT_STAR   = 1
    }

    /**
     * Compiles a JSON receipt ticket into raw thermal printer wire bytes.
     * Takes ~10 microseconds in Rust.
     */
    external fun compileTicket(jsonStr: String, dialect: Int = DIALECT_ESCPOS): ByteArray

    /**
     * Generates a virtual SVG string of the receipt.
     */
    external fun renderSvg(jsonStr: String): String

    /**
     * Generates an HTML preview string of the receipt.
     */
    external fun renderHtml(jsonStr: String): String
}
```

---

### 3. Streaming to Printers on Android

#### A. Bluetooth Classic (SPP / RFCOMM)

```kotlin
import android.bluetooth.BluetoothDevice
import android.bluetooth.BluetoothSocket
import java.util.UUID

private val SPP_UUID: UUID = UUID.fromString("00001101-0000-1000-8000-00805F9B34FB")

fun printViaBluetoothClassic(device: BluetoothDevice, wireBytes: ByteArray) {
    var socket: BluetoothSocket? = null
    try {
        socket = device.createRfcommSocketToServiceRecord(SPP_UUID)
        socket.connect()
        socket.outputStream.write(wireBytes)
        socket.outputStream.flush()
        println("✅ Printed successfully over Bluetooth Classic!")
    } finally {
        socket?.close()
    }
}
```

#### B. Wi-Fi / Ethernet LAN (Raw Port 9100)

```kotlin
import java.net.InetSocketAddress
import java.net.Socket

fun printViaNetwork(ipAddress: String, port: Int = 9100, wireBytes: ByteArray) {
    Socket().use { socket ->
        socket.connect(InetSocketAddress(ipAddress, port), 5000)
        socket.outputStream.write(wireBytes)
        socket.outputStream.flush()
        println("✅ Printed successfully via Port 9100!")
    }
}
```

---

## Part 3: Kotlin Multiplatform (KMP)

For teams building shared business logic across Android and iOS, `papermint` integrates seamlessly using KMP's `expect` / `actual` pattern.

### 1. Define the Common Interface (`commonMain`)

```kotlin
// commonMain/kotlin/com/mypos/printing/PapermintEngine.kt
package com.mypos.printing

expect class PapermintEngine() {
    fun compile(ticketJson: String, dialect: Int = 0): ByteArray
    fun renderSvg(ticketJson: String): String?
    fun renderHtml(ticketJson: String): String?
}
```

---

### 2. iOS Implementation via `cinterop` (`iosMain`)

Create a `cinterop` definition file `src/nativeInterop/cinterop/papermint.def`:

```properties
headers = papermint.h
headerFilter = papermint.h
package = com.papermint.native
```

In your `build.gradle.kts`:
```kotlin
kotlin {
    iosTarget("iosArm64") {
        binaries.framework()
        compilations["main"].cinterops {
            create("papermint")
        }
    }
}
```

Implement `PapermintEngine` in `iosMain`:

```kotlin
// iosMain/kotlin/com/mypos/printing/PapermintEngine.kt
package com.mypos.printing

import com.papermint.native.*
import kotlinx.cinterop.*

actual class PapermintEngine {
    actual fun compile(ticketJson: String, dialect: Int): ByteArray = memScoped {
        val outLen = alloc<size_tVar>()
        val ptr = papermint_compile_json(ticketJson, dialect.toUByte(), outLen.ptr) 
            ?: return ByteArray(0)
        val len = outLen.value.toInt()
        val bytes = ptr.readBytes(len)
        papermint_bytes_free(ptr, outLen.value)
        bytes
    }

    actual fun renderSvg(ticketJson: String): String? {
        val ptr = papermint_compile_json_svg(ticketJson) ?: return null
        val str = ptr.toKString()
        papermint_string_free(ptr)
        return str
    }

    actual fun renderHtml(ticketJson: String): String? {
        val ptr = papermint_compile_json_html(ticketJson) ?: return null
        val str = ptr.toKString()
        papermint_string_free(ptr)
        return str
    }
}
```

---

### 3. Android Implementation (`androidMain`)

In `androidMain`, implement `PapermintEngine` by delegating to JNI or loading `libpapermint_mobile.so`:

```kotlin
// androidMain/kotlin/com/mypos/printing/PapermintEngine.kt
package com.mypos.printing

actual class PapermintEngine {
    companion object {
        init {
            System.loadLibrary("papermint_mobile")
        }
    }

    private external fun nativeCompile(ticketJson: String, dialect: Int): ByteArray
    private external fun nativeRenderSvg(ticketJson: String): String
    private external fun nativeRenderHtml(ticketJson: String): String

    actual fun compile(ticketJson: String, dialect: Int): ByteArray {
        return nativeCompile(ticketJson, dialect)
    }

    actual fun renderSvg(ticketJson: String): String? {
        return nativeRenderSvg(ticketJson)
    }

    actual fun renderHtml(ticketJson: String): String? {
        return nativeRenderHtml(ticketJson)
    }
}
```

---

## 4. Feature Parity Matrix

| Feature | iOS Swift | Android Kotlin | Kotlin Multiplatform |
| :--- | :--- | :--- | :--- |
| **Integration** | C-ABI Header (`papermint.h`) | JNI (`libpapermint_mobile.so`) | `cinterop` + JNI |
| **Return Type** | `Data` | `ByteArray` | `ByteArray` |
| **Zero-Copy Memory** | Yes (`papermint_bytes_free`) | Yes (direct JNI buffer) | Yes (`memScoped`) |
| **Microsecond Speed**| ~10 µs | ~10 µs | ~10 µs |
| **Hardware Dialects**| ESC/POS & StarPRNT | ESC/POS & StarPRNT | ESC/POS & StarPRNT |
| **Virtual Previews** | SVG & HTML | SVG & HTML | SVG & HTML |

