---
title: "Supported Hardware"
description: "Tested printer models, driverless USB udev rules, RS-232, and cash drawer pins"
---

# 🖨️ Supported Hardware & Emulation Guide

`papermint` is designed to be vendor-agnostic and work with any POS thermal receipt printer supporting either standard **ESC/POS** or **StarPRNT / Star Line Mode**.

---

## 1. Verified Printer Compatibility

### Epson & ESC/POS Compatible
The `EscPos` dialect implementation works with standard ESC/POS printers:

| Manufacturer | Series / Models | Default Port | Notes |
| :--- | :--- | :---: | :--- |
| **Epson** | TM-T88 (III, IV, V, VI, VII), TM-T20 (II, III), TM-m30 (I, II, III), TM-U220 | 9100 | Industry standard reference. |
| **Bixolon** | SRP-350 (plus/plusII/plusIII), SRP-330, SRP-Q300 | 9100 | Fully compatible with standard ESC/POS. |
| **Citizen** | CT-S310II, CT-S651, CT-S801 | 9100 | Fully compatible. |
| **Star Micronics** *(ESC/POS mode)* | TSP100 (in ESC/POS mode), TSP650II | 9100 | When DIP switch or Memory Switch is set to ESC/POS emulation. |
| **Xprinter / Rongta / POS-58** | XP-N160I, XP-Q90EC, RPP-02N, 58mm/80mm budget models | 9100 | Standard ESC/POS commands supported. |
| **Sunmi / iMin** | Android POS terminals with integrated thermal heads | Direct / 9100 | Uses ESC/POS command parser. |

### Star Micronics (StarPRNT / Line Mode)
The `Star` dialect implementation works with native Star Micronics printers:

| Series | Verified Models | Native Emulation | Notes |
| :--- | :--- | :--- | :--- |
| **TSP100 Series** | TSP143III (LAN/WLAN/USB), TSP100futurePRNT | Star Line Mode | Built-in cutter, StarPRNT raster graphics. |
| **TSP650 Series** | TSP654II (LAN/AirPrint/CloudPRNT) | Star Line Mode | High speed (300mm/s), native Star QR. |
| **TSP700 Series** | TSP743II | Star Line Mode | Heavy-duty receipt/label printer. |
| **mC-Print Series** | mC-Print2 (58mm), mC-Print3 (80mm) | StarPRNT / Star Line Mode | Compact kitchen and front-of-house printer. |

---

## 2. Hardware Interface Standards

### Network Printing (RAW / JetDirect TCP)
Standard network-enabled thermal printers listen on **TCP Port 9100** (RAW socket printing):
- **Communication Flow**:
  1. `TcpTransport` connects to `<printer_ip>:9100`.
  2. Byte stream is sent directly to the printer controller.
  3. Printer flushes buffer and executes cut.
  4. Connection closes or stays persistent depending on configuration.

### Raw Driverless USB (USB Class 0x07 — Printers)
Direct USB thermal printers implement the standard USB Device Class 7 (Printers). `papermint` communicates directly with bulk endpoints using `nusb` without requiring vendor drivers:
- **Common Vendor IDs (VID)**:
  - **Epson**: `0x04B8`
  - **Star Micronics**: `0x0518`
  - **Bixolon**: `0x1504`
  - **Citizen**: `0x1D90`
  - **Zebra**: `0x0A5F`
  - **Xprinter / Generic**: `0x0483`, `0x0416`, `0x1FC9`, `0x20D1`
- **Linux Non-Root Permissions (`udev` Rule)**:
  To allow unprivileged POS applications to communicate with USB printers without `sudo`, create `/etc/udev/rules.d/99-thermal-printers.rules`:
  ```bash
  # Standard USB Class 7 (Printers)
  SUBSYSTEM=="usb", ATTR{bInterfaceClass}=="07", MODE="0666", GROUP="plugdev"
  ```
  Then reload rules: `sudo udevadm control --reload-rules && sudo udevadm trigger`

### RS-232 Serial & Bluetooth Classic (SPP)
Thermal printers equipped with serial cables or paired over Bluetooth Classic (Serial Port Profile) operate via standard UART streams:
- **Ports**:
  - Linux: `/dev/ttyS0`, `/dev/ttyUSB0` (FTDI/Prolific adapter), `/dev/rfcomm0` (Bluetooth SPP)
  - Windows: `COM1`, `COM3`, `COM4` (Bluetooth outgoing serial port)
- **Standard Serial Settings**:
  - Baud rate: 9600, 19200, 38400, or 115200 (check printer self-test sheet)
  - Data bits: 8, Parity: None, Stop bits: 1, Flow control: DTR/DSR or RTS/CTS

### Cash Drawer Kick Port (RJ11 / RJ12)
Cash drawers connect directly into the back of the thermal printer via a 6-pin modular connector:
- **Pin 2 (Drive Circuit 1)**: Primary cash drawer trigger.
  - Epson: `ESC p 0 25 250`
  - Star: `ESC BEL 11 55 BEL` (`1B 07 0B 37 07`)
- **Pin 5 (Drive Circuit 2)**: Secondary cash drawer trigger.
  - Epson: `ESC p 1 25 250`
  - Star: `ESC FS 11 55 FS` (`1B 1C 0B 37 1C`)

---

## 3. Emulation DIP Switch & Configuration

Some printers (notably Star Micronics and Citizen) support multiple emulation modes selectable via physical DIP switches under the printer base or software memory switches:

1. **Star TSP650 / TSP700**:
   - **Star Line Mode**: DIP Switch 1-1 = OFF (default for Star drivers).
   - **ESC/POS Mode**: DIP Switch 1-1 = ON (printer parses standard ESC/POS).
2. **Epson TM-T88**:
   - Standard ESC/POS out of the box.

`papermint` supports both modes seamlessly:
```rust
// Use Epson ESC/POS dialect:
let mut printer = Printer::escpos_tcp(addr);

// Or use StarPRNT dialect:
let mut printer = Printer::star_tcp(addr);
```
Both accept the exact same `Receipt` layout!
