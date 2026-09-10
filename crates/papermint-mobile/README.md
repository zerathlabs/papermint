# 📱 papermint-mobile — C-ABI & Mobile Receipt Engine

`papermint-mobile` compiles the core `papermint` thermal printing engine into a universal C-compatible library (`libpapermint_mobile.so` / `libpapermint_mobile.a`) with a C header for **iOS**, **Android**, **React Native**, **Expo SDK 56+**, and **Flutter**.

---

## Features

- **JSON Receipt Compiler**: `papermint_compile_json()` takes a high-level JSON receipt definition and compiles it into binary wire bytes (ESC/POS or StarPRNT) in **~10 microseconds**.
- **Fluent C Handle API**: Memory-safe C functions for programmatic control (`papermint_receipt_create`, `papermint_receipt_text_ln`, `papermint_receipt_qr`, `papermint_receipt_cut`).
- **Standard C Header**: [`include/papermint.h`](include/papermint.h) ready for Swift bridging headers, Android JNI / NDK CMake, and React Native C++ TurboModules.

---

## Compiling for Mobile

### Android (NDK Cross-Compilation):
```bash
cargo build --package papermint-mobile --release --target aarch64-linux-android
cargo build --package papermint-mobile --release --target armv7-linux-androideabi
cargo build --package papermint-mobile --release --target x86_64-linux-android
```

### iOS (Universal Framework):
```bash
cargo build --package papermint-mobile --release --target aarch64-apple-ios
cargo build --package papermint-mobile --release --target aarch64-apple-ios-sim
```

---

## Mobile Integration Guide

For full step-by-step guides, code snippets for Expo Inline Modules (Swift & Kotlin), and Bluetooth SPP/BLE streaming, see:
👉 **[`docs/MOBILE_INTEGRATION.md`](../../docs/MOBILE_INTEGRATION.md)** or visit **[https://papermint.zerathlabs.com/platforms/mobile/](https://papermint.zerathlabs.com/platforms/mobile/)**
