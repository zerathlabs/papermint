# 🌿 papermint — Architectural Roadmap & Feature Proposals

This document outlines upcoming architectural enhancements, hardware integrations, and layout engine extensions for `papermint`. Each proposal includes hardware wire specifications, proposed Rust API designs, and implementation steps for progressive rollout.

---

## Table of Contents
1. [Proposal 1: Advanced N-Column Table Engine with Word-Wrapping](#proposal-1-advanced-n-column-table-engine-with-word-wrapping)
2. [Proposal 2: Real-Time Hardware Status & Sensor Telemetry](#proposal-2-real-time-hardware-status--sensor-telemetry)
3. [Proposal 3: International Character Sets & Extended Code Pages](#proposal-3-international-character-sets--extended-code-pages)
4. [Proposal 4: Thermal Printhead Energy & Density Calibration](#proposal-4-thermal-printhead-energy--density-calibration)
5. [Proposal 5: USB & Serial/COM Hardware Transports](#proposal-5-usb--serialcom-hardware-transports)
6. [Proposal 6: POS Monorepo Integration & Native Print Service](#proposal-6-pos-monorepo-integration--native-print-service)

---

## Proposal 1: Advanced N-Column Table Engine with Word-Wrapping

### Motivation
Receipts in restaurant and retail environments frequently require 3 to 5 columns (e.g. `Qty | Item Description | Modifier | Unit | Total`). When an item name exceeds the column boundary, existing basic column formats truncate text. A production layout engine must wrap text cleanly at word boundaries and vertically synchronize multi-line rows.

### Architecture Design
- **Column Definition**:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq)]
  pub enum ColumnWidth {
      /// Fixed character count.
      Fixed(usize),
      /// Percentage of total printable columns (0.0 ..= 1.0).
      Fraction(f32),
  }

  #[derive(Debug, Clone)]
  pub struct Column {
      pub width: ColumnWidth,
      pub alignment: Alignment,
      pub padding_left: usize,
      pub padding_right: usize,
  }
  ```

- **Row Synchronization Algorithm**:
  1. Calculate resolved character widths for each column from total `PaperWidth::columns()`.
  2. For each cell in a row, perform word-wrapping into a `Vec<String>` of line fragments.
  3. Determine maximum row height: `H = max(cell.lines.len())`.
  4. Pad shorter columns with blank lines to height `H`.
  5. Emit `H` formatted text lines to the receipt command stream.

### Proposed Fluent API
```rust
let receipt = Receipt::new(PaperWidth::Mm80)
    .table_header(&[
        Column::new(ColumnWidth::Fixed(4), Alignment::Left),   // Qty
        Column::new(ColumnWidth::Fraction(0.55), Alignment::Left), // Description
        Column::new(ColumnWidth::Fraction(0.20), Alignment::Right), // Unit
        Column::new(ColumnWidth::Fraction(0.25), Alignment::Right), // Total
    ])
    .table_row(&["2x", "Truffle Double Angus Burger with Caramelized Onions", "$12.00", "$24.00"])
    .table_row(&["1x", "Sparkling Mint Lemonade", "$4.50", "$4.50"]);
```

---

## Proposal 2: Real-Time Hardware Status & Sensor Telemetry

### Motivation
Thermal printers are physical electromechanical devices subject to paper depletion, cutter jams, open covers, and drawer switch triggers. Production POS systems require bidirectional telemetry to verify printer readiness before attempting large print jobs or opening cash drawers.

### Hardware Wire Protocols
- **Epson ESC/POS**:
  - Real-Time Status Transmission (`DLE EOT n` where $n \in \{1, 2, 3, 4\}$):
    - `DLE EOT 1` (0x10 0x04 0x01): Printer Status (drawer kickout pin level, online/offline).
    - `DLE EOT 2` (0x10 0x04 0x02): Offline Status (cover open/closed, feed button pressed, paper empty stop).
    - `DLE EOT 3` (0x10 0x04 0x03): Error Status (cutter error, unrecoverable error, auto-recoverable error/head overheat).
    - `DLE EOT 4` (0x10 0x04 0x04): Paper Roll Sensor (near-end sensor, roll-end sensor).
  - Auto Status Back (ASB): `GS a n` enabling unsolicited status broadcast.
- **StarPRNT**:
  - Real-Time Query: `ENQ` (0x05) returning single-byte status.
  - Automatic Status Back (ASB): Multi-byte frame including head temperature, cutter jam, and drawer sensor.

### Proposed Data Structures & API
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaperSensorStatus {
    Adequate,
    NearEnd,
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverStatus {
    Closed,
    Open,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawerSensorStatus {
    Closed,
    Open,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardwareStatus {
    pub is_online: bool,
    pub cover: CoverStatus,
    pub paper: PaperSensorStatus,
    pub drawer: DrawerSensorStatus,
    pub cutter_error: bool,
    pub head_overheated: bool,
}

impl<D: Dialect, T: AsyncRead + AsyncWrite + Unpin> Printer<D, T> {
    /// Queries real-time hardware status over the active transport.
    pub async fn query_status(&mut self) -> Result<HardwareStatus>;
}
```

---

## Proposal 3: International Character Sets & Extended Code Pages

### Motivation
POS installations operating across multilingual regions (Europe, Middle East, Asia) require accurate character table configurations to render accented glyphs, currency symbols (e.g. €, £, ¥, ₹), and localized alphabets without relying exclusively on raster bitmap rendering.

### Technical Specification
1. **International Character Set (`ESC R n`)**:
   - Swaps 12 standard ASCII code points (`#`, `$`, `@`, `[`, `\`, `]`, `^`, `` ` ``, `{`, `|`, `}`, `~`) for localized character glyphs.
   - Enums: `USA (0)`, `France (1)`, `Germany (2)`, `UK (3)`, `DenmarkI (4)`, `Sweden (5)`, `Italy (6)`, `SpainI (7)`, `Japan (8)`, `Norway (9)`, `DenmarkII (10)`, `SpainII (11)`, `LatinAmerica (12)`, `Korea (13)`, `Slovenia (14)`, `China (15)`.

2. **Extended Code Tables (`ESC t n` / `ESC GS t n`)**:
   - Support for Arabic (`CP864`, `Windows-1256`, `CP720`).
   - Support for Cyrillic (`CP866`, `CP855`, `Windows-1251`, `ISO-8859-5`).
   - Support for Greek (`CP737`, `Windows-1253`, `ISO-8859-7`).
   - Support for Hebrew (`CP862`, `Windows-1255`, `ISO-8859-8`).
   - Support for Thai (`CP874`, `Thai2`).

3. **Complex Script Shaping (Future Optional Feature)**:
   - For right-to-left languages (Arabic/Hebrew), integrate Unicode Bidirectional Algorithm (UAX #9) and Arabic presentation forms shaping so connected cursive letters render properly on legacy single-byte hardware.

---

## Proposal 4: Thermal Printhead Energy & Density Calibration

### Motivation
Receipt paper formulations vary (standard thermal, synthetic waterproof, high-sensitivity labels). In addition, barcode readability by optical laser scanners depends heavily on the thermal heating cycle.

### Wire Protocols
- **Thermal Timing Control (`ESC 7 n1 n2 n3`)**:
  - `n1` = Max thermal heating dots ($0 \dots 255$, default $7 \times 8 = 56$ dots).
  - `n2` = Thermal heating time ($3 \dots 255$, default $80 \times 10\mu\text{s} = 800\mu\text{s}$).
  - `n3` = Thermal heating interval ($0 \dots 255$, default $2 \times 10\mu\text{s} = 20\mu\text{s}$).
- **Print Density Adjustment (`GS ( K` / `DC2 # n`)**:
  - Fine adjustment of dot darkness level (-50% to +50%).

### Proposed API
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintDensity {
    Light,
    Normal,
    Dark,
    HighContrast,
    Custom(u8),
}

receipt = receipt.print_density(PrintDensity::HighContrast);
```

---

## Proposal 5: USB & Serial/COM Hardware Transports

### Motivation
While network thermal printers (Ethernet/Wi-Fi) dominate commercial kitchens, front-of-house counter POS setups frequently connect printers via direct physical cables:
- **USB Bulk Transfer** (USB Class 07: Printers).
- **RS-232 / Virtual COM Ports** (RJ45 or DB9 serial interface).

### Proposed Cargo Features
```toml
[features]
serial = ["dep:tokio-serial"]
usb = ["dep:nusb"]
```

### Proposed Transport Interfaces
- `SerialTransport::new(port_name, baudrate)` supporting standard 9600, 19200, 38400, and 115200 baud with hardware flow control (RTS/CTS).
- `UsbTransport::new(vendor_id, product_id)` discovering printer class endpoints (Bulk OUT 0x01, Bulk IN 0x82).

---

## Proposal 6: POS Monorepo Integration & Native Print Service

### Motivation
The current print service in `apps/print-service` handles incoming print jobs for receipts, kitchen order tickets (KOT), and cash drawer kicks. Migrating or bridging this service with `papermint` yields:
1. Instantaneous sub-millisecond receipt generation and network dispatch.
2. Direct raster graphics with Floyd-Steinberg dithering for merchant logos.
3. Resilience against connection drops with zero memory leaks.

### Integration Strategy
- **Phase A (Native CLI / Sidecar Daemon)**:
  Expose a lightweight local HTTP/WebSocket or Unix Domain Socket service running compiled Rust that accepts structured JSON tickets and streams bytes to thermal printers.
- **Phase B (N-API / Node Addon or WASM)**:
  Generate Node.js bindings via `napi-rs` so `apps/print-service` can directly call `papermint` inside the existing TypeScript ecosystem.

---

## Recommended Sequence for Implementation

| Step | Feature | Effort | Impact |
|:---:|:---|:---:|:---:|
| **1** | **Proposal 1: Advanced N-Column Table Engine** | Medium | High (Immediate visual improvement for receipts & KOTs) |
| **2** | **Proposal 2: Real-Time Hardware Status Telemetry** | Medium | High (Crucial for hardware reliability & error detection) |
| **3** | **Proposal 3: International Character Sets & Code Pages** | Low-Medium | Medium (Global POS compliance) |
| **4** | **Proposal 4: Thermal Energy & Density Control** | Low | Medium (Barcode clarity & contrast) |
| **5** | **Proposal 5: USB & Serial Transports** | High | High (Expands connectivity beyond TCP/LAN) |
| **6** | **Proposal 6: POS Monorepo Integration** | Medium | Very High (Unifies repository architecture) |
