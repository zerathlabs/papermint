# Changelog

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
