# Changelog

## [0.2.3] - 2026-09-15

### Added
- **Complete Feature Parity for Mobile (`expo-papermint` & `papermint-mobile`)**:
  - Reached 100% feature parity with `papermint-node` and Rust core.
  - Granular Command IR: Mobile apps can now use `.bold()`, `.underline()`, `.invert()`, `.doubleSize()`, `.doubleWidth()`, `.doubleHeight()`, `.left()`, `.center()`, `.right()`, `.align()`.
  - Multi-Column Flexible Tables: Added `.setColumns()`, `.row()`, `.tableHeader()`, `.clearColumns()` with fixed-pitch and fractional column widths.
  - Rich Dividers: Added `.dividerDouble()`, `.dividerDotted()`, `.dividerDashed()`, and `.dividerPattern()`.
  - Hardware Controls: Added `.openDrawer()`, `.beep()`, `.cut()`, `.feed()`.
  - Custom CodePage Switching: Added `.codePage()` for dynamic code table selection on mobile.
  - Image Printing: Portable `.image(base64PngOrJpeg)` with zero native image library dependencies.
  - Raw Injection: Added `.raw()` for custom ESC/POS or StarPRNT byte sequences.
- **On-Device Virtual Receipt Previewers (SVG & HTML)**:
  - Added `.renderSvg()` and `.renderHtml()` to mobile `Receipt` builder and standalone helpers (`renderSvg()`, `renderHtml()`).
  - Added C-ABI exports `papermint_compile_json_svg`, `papermint_compile_json_html`, and `papermint_string_free` in `crates/papermint-mobile`.
  - Added Android JNI bridges (`nativeRenderSvg`, `nativeRenderHtml`) and iOS Swift bindings with automatic memory deallocation.
  - Enables instant, pixel-accurate vector and HTML receipt previews on mobile screens before physical printing.

### Fixed
- **Unicode-Compliant Windows-1256 (WPC1256) Arabic Transcoding Engine**:
  - Replaced legacy contiguous arithmetic with the official Microsoft / Unicode Consortium mapping (`CP1256.TXT`).
  - Fixed Arabic letters (`ف`, `و`, `ك`, `ط`, etc.) and punctuation (`،`, `؛`, `؟`) which previously collided with Latin French accents (`è`) or mathematical signs (`×`).
  - Added full Arabic Tashkeel diacritics (Fathatan, Dammatan, Kasratan, Fatha, Damma, Kasra, Shadda, Sukun).
  - Added Persian & Urdu character support (`پ`, `ٹ`, `چ`, `ژ`, `ڈ`, `گ`, `ک`, `ڑ`, `ں`, `ہ`).
  - Standardized OEM Table 33 for Chinese/OEM thermal POS printers (PosBox PB800, Rongta, Xprinter).
  - Thermal head protection: Arabic-Indic digits (`٠..٩`) automatically fall back to ASCII (`0..9`) to prevent thermal print head mojibake.

## [0.2.2] - 2026-09-14

### Added
- **Universal Expo & React Native Module (`expo-papermint`)**:
  - Added official Expo Module package in `packages/expo-papermint` adhering to Expo Modules API v2 and `create-expo-module`.
  - Zero-copy native marshaling from `ByteArray` (Android) and `Data` (iOS) directly into JavaScript `Uint8Array`.
  - Added Android JNI bridge `Java_expo_modules_papermint_ExpoPapermintModule_nativeCompileTicket` in `crates/papermint-mobile`.
  - Added fluent `Receipt` builder, `compileTicket()`, and `generateZatcaQr()` for mobile POS applications.
  - Added complete Bluetooth Classic (`react-native-bluetooth-classic`) and BLE (`react-native-ble-plx`) printing guides.
- **Automated Mobile Release & CI Pipelines**:
  - Configured GitHub Actions release workflow to cross-compile Android `.so` libraries (`arm64-v8a`, `armeabi-v7a`, `x86_64`) via `cargo-ndk`.
  - Configured macOS runner to build device + simulator iOS static libraries and package into `PapermintMobile.xcframework`.
  - Automated dual npm publishing for `papermint` (desktop/server) and `expo-papermint` (mobile).
  - Unified workspace bumping target in `Makefile` (`make bump V=x.y.z`) covering Cargo crates, npm packages, iOS Podspec, and Android Gradle.
- **Node.js N-API Extended Methods (`papermint-node`)**:
  - Added `.codePage(page)` to switch character code tables by name (`"wpc1256"`, `"pc850"`, `"pc437"`, etc.) or numeric hardware table ID (`33`, `22`, etc.).
  - Added `.raw(bytes)` to inject arbitrary raw printer escape sequences into the command stream.
  - Added `.image(buffer, maxWidth?)` to print Floyd-Steinberg dithered raster images from Node.js buffers (canvas graphics, logos).
- **OEM WPC1256 Arabic Table 33 Mapping**:
  - Mapped `CodePage::Custom(33)` to `encode_wpc1256` in Rust core for Chinese/OEM thermal printers (PosBox PB800, Rongta, Xprinter).
- **Arabic Thermal Receipt Example**:
  - Added `examples/print_arabic_bill.mjs` demonstrating bilingual formatting, SAR currency, ZATCA e-invoicing QR code, and hardware codepage diagnostics.


## [0.2.1] - 2026-09-11

### Fixed
- **Multi-OS Universal npm Binaries**:
  - Configured GitHub Actions release pipeline to build and bundle native `.node` modules for Windows (`win32-x64-msvc`), Linux (`linux-x64-gnu`), and macOS (`darwin-arm64`) directly into the published npm package.
  - Fixes `Cannot find module 'papermint-win32-x64-msvc'` error on Windows when installing via `npm install papermint`.

## [0.2.0] - 2026-09-11

### Added
- **Middle East E-Invoicing Engine (ZATCA & UAE FTA)**:
  - Binary Tag-Length-Value (TLV) Phase 1 & Phase 2 encoder supporting Tags 1 through 9 (`ZatcaInvoice`, `encode_zatca_tlv`).
  - Native `.zatca_qr(&invoice)` on fluent `Receipt` builder.
  - Cross-platform Base64 QR payload generator (`zatca_qr_base64`).
  - Node.js & TypeScript bindings: `zatcaQrBase64()` and `receipt.zatcaQr()`.
  - Comprehensive documentation guide for Saudi Arabia and UAE tax compliance.
- **Virtual Receipt Previewer (SVG & HTML)**:
  - Resolution-independent vector SVG renderer (`render_svg()`) simulating physical monospace character cells, paper drop shadows, and jagged tear-off edge.
  - Standalone responsive HTML component previewer (`render_html()`) for web POS checkouts and cashier screens.
  - Node.js & TypeScript bindings: `receipt.renderSvg()` and `receipt.renderHtml()`.
- **WebUSB Direct Browser Printing**:
  - Driverless in-browser USB printing guide using the W3C WebUSB API.
  - Production-ready React hook (`useWebUsbPrinter`) with auto-claiming Class 07 printer endpoints.
- **Developer Experience (DX) Aliases in `papermint-node`**:
  - `TableColumnOptions` now accepts flexible aliases: `width` (shorthand for `widthFixed`) and `alignment` (alias for `align`).
- **Real-World Hardware Validation**:
  - Physical hardware verification on 80mm high-speed POS hardware (PosBox PB800 at 230 mm/s) verifying synchronized multiline tables, ZATCA QR optical contrast, and auto-cutter actuation.

### Changed
- Modernized internal Base64 encoding in `src/tax/zatca.rs` using `as_chunks::<3>()` and `as_chunks::<4>()` for Rust 1.98 compatibility.

## [0.1.1] - 2026-09-10

### Added
- Dedicated README documentation for `papermint-node`, `papermint-daemon`, and `papermint-mobile`.
- Search keywords, repository, and homepage metadata for the `papermint` npm package.
- Automated workspace version bumping target (`make bump V=...`) in Makefile.

## [0.1.0] - 2026-09-10

### Added
- **Core Fluent Receipt Builder**:
  - Flexible typography commands (`bold`, `underline`, `invert`, `doubleSize`, `font`).
  - Text alignment (`left`, `center`, `right`) and line feeds (`feed`, `feedLines`).
  - Dividers (`divider`, `dividerDouble`, `dividerDashed`, `dividerDotted`, `dividerPattern`).
  - Two-column justified key-value row formatter (`twoColumn`).
  - Hardware triggers: paper cut (`cut`, `cutPartial`, `cutFull`), cash drawer kick (`drawerKick`), and acoustic buzzer (`beep`).
  - Density and thermal print head calibration (`density`, `heatingParameters`).
- **Monospace Table Engine**:
  - Auto-balanced column layouts with fixed, percentage/fractional, and flex width distribution.
  - Synchronized multiline cell wrapping across adjacent columns.
  - Unicode Standard Annex #11 (East Asian Width) calculation for correct monospace alignment with CJK, Arabic, and emoji characters.
- **Dialect Wire Encoders**:
  - **Epson ESC/POS**: Full binary protocol encoder supporting standard POS hardware (Epson, Bixolon, Rongta, Xprinter, Citizen).
  - **StarPRNT**: Line-mode binary protocol encoder supporting Star Micronics hardware (TSP100, TSP650II, TSP700, mC-Print).
  - 1D Barcodes: Code 128, Code 39, EAN-13, UPC-A, ITF.
  - 2D QR Codes: Multi-step model, error correction (L/M/Q/H), module size, and binary rendering envelopes.
  - Raster Bitmaps: Single-bit packed raster graphics with Floyd-Steinberg and Atkinson error-diffusion halftoning algorithms.
- **Hardware Transports**:
  - **Driverless USB (`nusb`)**: Direct OS-independent bulk endpoint communication without vendor driver installation or elevated permissions.
  - **Async RAW TCP**: Network POS printer communication on Port 9100 via Tokio with connection and read/write timeouts.
  - **Serial / RS-232 & Bluetooth SPP**: Asynchronous serial communications via `tokio-serial`.
  - **In-Memory Mock Transport**: `Printer::mock()` test sink for recording, verifying, and unit testing wire byte streams.
- **Real-Time Hardware Telemetry**:
  - Real-time ASB (Auto Status Back) status queries (`query_status`).
  - Typed diagnostics: paper empty, paper near-end, cover open, cutter error, and print head overheat.
- **Node.js & TypeScript Bindings (`papermint`)**:
  - Native zero-overhead N-API (`napi-rs`) bindings.
  - Published to npm as `papermint`.
  - TypeScript definitions (`index.d.ts`) with full type safety and autocomplete.
- **Mobile C-ABI Engine (`papermint-mobile`)**:
  - C-compatible dynamic and static library (`libpapermint_mobile`).
  - JSON receipt intermediate compiler (`papermint_compile_json`).
  - C header file (`include/papermint.h`) for iOS, Android, and Expo SDK 56+ Inline Native Modules.
- **HTTP Print Daemon (`papermintd`)**:
  - High-concurrency local REST server powered by Axum and Tokio.
  - Endpoints: `GET /health`, `GET /devices`, `POST /print`, `POST /status`.
- **Documentation & CI/CD**:
  - Astro Starlight documentation site deployed at [papermint.zerathlabs.com](https://papermint.zerathlabs.com).
  - Automated release pipeline for multi-platform binaries, crates.io, and npm.
