---
title: "Hardware Protocols"
description: "ESC/POS wire commands, thermal dot math, and solenoid pulses"
---

# 🖨️ Thermal Printer Hardware & Protocol Specification

> **Target Audience**: Contributors, driver engineers, and POS system integrators.  
> **Scope**: ESC/POS binary command specification, electrical solenoid drawer pulses, QR symbology, and thermal paper physics.

---

## 1. How Thermal Printers Work

Unlike inkjet or laser printers, a **direct thermal receipt printer** uses no ink, ribbon, or toner.

### The Physical Printing Mechanism:
- **Thermal Print Head (TPH)**: A linear strip of microscopic heating elements (dots) spanning the paper width.
- **Resolution**: Most standard POS printers operate at **203 DPI** (dots per inch), which equals **8 dots per millimeter**.
  - **80mm Paper**: Printable width is ~72mm = **576 dots per line**.
  - **58mm Paper**: Printable width is ~48mm = **384 dots per line**.
- **Thermo-Chromic Paper**: The paper is coated with a colorless chemical dye. When a heating pin reaches ~100°C–150°C, the dye reacts and turns black.
- **Stepper Motor**: Advances the paper roll line-by-line using a rubber roller (platen).

Because thermal heads are simple microcontrollers without a desktop operating system, communication occurs via a **raw binary byte stream** sent over TCP (port 9100), USB, or RS-232 serial.

---

## 2. The ESC/POS Protocol Standard

Invented by Epson in the 1980s, **ESC/POS** (Escape / Point of Sale) is the universal binary standard implemented by Epson, Star, Bixolon, Citizen, Xprinter, Rongta, and Sunmi.

### Control Characters (Command Introducers):

| Constant | Hex | Decimal | ASCII | Purpose |
| :--- | :---: | :---: | :---: | :--- |
| **`ESC`** | `0x1B` | `27` | Escape | Standard command introducer (styling, font, feed) |
| **`GS`** | `0x1D` | `29` | Group Separator | Advanced command introducer (cutter, barcodes, QR, drawer) |
| **`LF`** | `0x0A` | `10` | Line Feed | Advance paper by 1 line & flush line buffer (`\n`) |

When the printer receives raw bytes:
1. Bytes from `0x20` to `0x7E` (printable ASCII) are placed into the line dot buffer.
2. When an **`ESC`** or **`GS`** byte arrives, the printer pauses rendering and parses the following bytes as a hardware instruction.

---

## 3. Core Command Reference

### 3.1 Printer Lifecycle
- **Initialize Printer (`ESC @`)**:
  - Bytes: `[0x1B, 0x40]`
  - Resets all text formatting (bold, invert, sizes), clears line buffers, and restores default font.

### 3.2 Paper Feed & Motor Control
- **Single Line Feed (`LF`)**:
  - Bytes: `[0x0A]`
  - Prints whatever is in the line buffer and advances paper by one line.
- **Feed N Lines (`ESC d n`)**:
  - Bytes: `[0x1B, 0x64, n]`
  - Smooth motor scroll of `n` lines without stutter.

### 3.3 Text Styling & Fonts
- **Alignment (`ESC a n`)**:
  - `[0x1B, 0x61, 0]` ➡️ Left
  - `[0x1B, 0x61, 1]` ➡️ Center
  - `[0x1B, 0x61, 2]` ➡️ Right
- **Bold (`ESC E n`)**:
  - `[0x1B, 0x45, 1]` ➡️ Bold ON
  - `[0x1B, 0x45, 0]` ➡️ Bold OFF
- **Underline (`ESC - n`)**:
  - `[0x1B, 0x2D, 0]` ➡️ OFF
  - `[0x1B, 0x2D, 1]` ➡️ 1-dot thick underline
  - `[0x1B, 0x2D, 2]` ➡️ 2-dot thick underline
- **White/Black Reverse (`GS B n`)**:
  - `[0x1D, 0x42, 1]` ➡️ Invert ON (White text inside solid black background box)
  - `[0x1D, 0x42, 0]` ➡️ Invert OFF
- **Character Dimensions (`GS ! n`)**:
  - Single byte bitmask controlling vertical and horizontal character scaling:
    - **Bits 0–3**: Vertical multiplier (`0x00` = 1x, `0x01` = 2x height)
    - **Bits 4–7**: Horizontal multiplier (`0x00` = 1x, `0x10` = 2x width)
    - Combined Double-Size: `0x11` (2x width AND 2x height)
- **Font Selection (`ESC M n`)**:
  - `[0x1B, 0x4D, 0]` ➡️ Font A (Standard 12×24 dot matrix, 48 cols on 80mm)
  - `[0x1B, 0x4D, 1]` ➡️ Font B (Condensed 9×17 dot matrix, 64 cols on 80mm)

### 3.4 Mechanical Paper Cutter
- **Full Cut (`GS V 0`)**:
  - Bytes: `[0x1D, 0x56, 0x00]`
  - Activates the circular or guillotine blade to slice across the entire roll.
- **Partial Cut (`GS V 1`)**:
  - Bytes: `[0x1D, 0x56, 0x01]`
  - Leaves a small ~1mm perforated paper tab in the center so the receipt doesn't fall to the floor.

### 3.5 Audio Buzzer / Beeper
- **Buzzer Signal (`ESC B n t`)**:
  - Bytes: `[0x1B, 0x42, n, t]`
  - `n`: Number of beeps (`1` to `9`)
  - `t`: Duration multiplier per beep (`1` to `9`)

---

## 4. Cash Drawer Solenoid Control

Most POS cash drawers do **not** have USB or network cables. Instead, they connect directly into the back of the thermal printer via an **RJ11 / RJ12 6-pin phone jack**.

```text
┌─────────────────────────┐          RJ11 / RJ12 Cable          ┌───────────────────┐
│ Thermal Receipt Printer │ ══════════════════════════════════> │ Cash Drawer Box   │
│ (Controller on 24V DC)  │    Pin 2 (Drawer 1) / Pin 5 (Dr 2)  │ (Magnetic Coil)   │
└─────────────────────────┘                                     └───────────────────┘
```

### The Physics:
Inside the cash drawer is a **magnetic solenoid plunger**. When 24 volts of electrical current pulses through the coil, it creates an electromagnet that jerks the latch open, allowing spring-loaded rails to kick the drawer open.

### The ESC/POS Drawer Kick Command:
```text
ESC p m t1 t2
[0x1B, 0x70, m, t1, t2]
```
- **`m` (Pin Selector)**:
  - `0`: Pin 2 (Primary Cash Drawer 1)
  - `1`: Pin 5 (Secondary Cash Drawer 2)
- **`t1` (Pulse ON duration)**:
  - Unit: `2 ms`
  - Value `25` ➡️ `25 × 2ms = 50ms` on-time (standard to energize the coil).
- **`t2` (Pulse OFF duration)**:
  - Unit: `2 ms`
  - Value `250` ➡️ `250 × 2ms = 500ms` cool-down off-time before next firing.

---

## 5. Native 2D QR Code Generation

Modern thermal printers have a built-in hardware QR code rasterizer. Printing a QR code requires a standard 5-step sequence defined in the Epson ESC/POS standard:

```text
Step 1: Set QR Model (Model 2)
        GS ( k 4 0 49 65 50 0
        [0x1D, 0x28, 0x6B, 0x04, 0x00, 0x31, 0x41, 50, 0x00]

Step 2: Set Module (Cell) Size (typically 3 to 8 dots)
        GS ( k 3 0 49 67 <cell_size>
        [0x1D, 0x28, 0x6B, 0x03, 0x00, 0x31, 0x43, cell_size]

Step 3: Set Error Correction Level
        GS ( k 3 0 49 69 <ec_code>
        [0x1D, 0x28, 0x6B, 0x03, 0x00, 0x31, 0x45, ec_code]
        - Level L: 48 (7% recovery)
        - Level M: 49 (15% recovery — Recommended for receipts)
        - Level Q: 50 (25% recovery)
        - Level H: 51 (30% recovery)

Step 4: Store Data in Symbol Storage Area
        GS ( k pL pH 49 80 48 <data...>
        [0x1D, 0x28, 0x6B, pL, pH, 0x31, 0x50, 0x30, ...data...]
        Where length = data.len() + 3
        pL = length & 0xFF
        pH = (length >> 8) & 0xFF

Step 5: Print Stored QR Code
        GS ( k 3 0 49 81 48
        [0x1D, 0x28, 0x6B, 0x03, 0x00, 0x31, 0x51, 0x30]
```

---

## 6. Paper Widths & Column Mathematics

| Paper Roll Size | Total Width | Printable Width | Font A (12×24) | Font B (9×17) |
| :---: | :---: | :---: | :---: | :---: |
| **80mm** (Standard POS) | 80 mm | ~72 mm (576 dots) | **48 Columns** | 64 Columns |
| **58mm** (Mobile / Kitchen) | 58 mm | ~48 mm (384 dots) | **32 Columns** | 42 Columns |

### Column Alignment Golden Rule:
In receipt table rows (e.g. `Item Name` on left and `Price` on right):
```text
Total Paper Columns = 48
"Double Cheeseburger" -> 19 chars
"$9.50"               ->  5 chars
Padding spaces needed = 48 - (19 + 5) = 24 spaces
```

> **Warning for Contributors**: Never use `str.len()` for column calculations! In Rust, `str.len()` returns **byte length**. A multi-byte UTF-8 character (like `€` which is 3 bytes, or Arabic characters) will misalign the entire receipt if byte length is used. Always use **`str.chars().count()`** to calculate visual column positions.

---

## 7. Star Micronics Protocol (StarPRNT / Line Mode)

Star Micronics printers (TSP100, TSP650, TSP700, mC-Print series) use the Star Line Mode command set.

### Epson ESC/POS vs. StarPRNT Comparison Table:

| Function | Epson ESC/POS | StarPRNT / Line Mode | Notes |
| :--- | :--- | :--- | :--- |
| **Initialize** | `ESC @` (`[0x1B, 0x40]`) | `ESC @` (`[0x1B, 0x40]`) | Identical |
| **Bold ON** | `ESC E 1` (`[0x1B, 0x45, 1]`) | `ESC E` (`[0x1B, 0x45]`) | Star uses discrete ON/OFF codes |
| **Bold OFF** | `ESC E 0` (`[0x1B, 0x45, 0]`) | `ESC F` (`[0x1B, 0x46]`) | `ESC F` turns bold off |
| **Invert ON** | `GS B 1` (`[0x1D, 0x42, 1]`) | `ESC 4` (`[0x1B, 0x34]`) | Star uses `ESC 4` |
| **Invert OFF** | `GS B 0` (`[0x1D, 0x42, 0]`) | `ESC 5` (`[0x1B, 0x35]`) | Star uses `ESC 5` |
| **Alignment** | `ESC a n` (`0, 1, 2`) | `ESC GS a n` (`[0x1B, 0x1D, 0x61, n]`) | Star prefixes with `ESC GS` |
| **Full Cut** | `GS V 0` (`[0x1D, 0x56, 0]`) | `ESC d 2` (`[0x1B, 0x64, 2]`) | Auto-feeds and cuts |
| **Partial Cut**| `GS V 1` (`[0x1D, 0x56, 1]`) | `ESC d 3` (`[0x1B, 0x64, 3]`) | Leaves ~1mm paper tab |
| **Text Scaling** | `GS ! n` (packed bitmask) | `ESC i n1 n2` (`[0x1B, 0x69, h, w]`) | Star takes separate height/width |
| **Font A / B** | `ESC M n` (`0x1B 0x4D n`) | `ESC RS F n` (`[0x1B, 0x1E, 0x46, n]`)| Star prefixes with `ESC RS` |
| **Drawer Kick** | `ESC p m t1 t2` (`[0x1B, 0x70, m, 25, 250]`) | `ESC BEL / FS n1 n2` (`[0x1B, 0x07 / 0x1C, 11, 55, 0x07 / 0x1C]`) | Pin 2 / Pin 5 solenoid pulse |
| **Buzzer** | `ESC B n t` (`[0x1B, 0x42, n, t]`) | `ESC BEL` (`[0x1B, 0x07]`) | Hardware sound alert |
| **Line Spacing** | `ESC 2` (default) / `ESC 3 n` | `ESC z 1` (default) / `ESC 3 n` | Vertical line pitch in dots |
| **Upside-Down** | `ESC { 1/0` (`[0x1B, 0x7B, 1/0]`) | `SI` (`0x0F`) / `DC2` (`0x12`) | 180° rotation for kitchen rails |
| **Code Page** | `ESC t n` (`[0x1B, 0x74, n]`) | `ESC GS t n` (`[0x1B, 0x1D, 0x74, n]`) | Select character code table |
| **International Charset** | `ESC R n` (`[0x1B, 0x52, n]`) | `ESC R n` (`[0x1B, 0x52, n]`) | Select country currency and punctuation glyphs |
| **1D Barcode** | `GS k m n d1..dk` (`m=65..73`, auto `{B`) | `ESC b n1 n2 n3 n4 d1..dk RS` (`n1=0..8`) | 0-indexed symbologies in Star |
| **QR Code** | `GS ( k ...` (Function 165–181) | `ESC GS y S ...` | 5-step hardware QR sequence |
| **Raster Image** | `GS v 0 0 xL xH yL yH <data>` | `ESC * r A` / `ESC * r b ...` / `ESC * r B` | Native 1-bit monochrome raster |
| **Status Query** | `DLE EOT 1..4` (`[0x10, 0x04, 1..4]`) | `ENQ` (`0x05`) / ASB | Immediate real-time status inquiry |
| **Print Density** | `GS ( K 2 0 48 m` (`0x1D 0x28 0x4B 0x02 0x00 0x30 m`) | `ESC GS # '0' n LF NUL` (`0x1B 0x1D 0x23 0x30 n 0x0A 0x00`) | Calibration for paper sensitivity & contrast |
| **Heating Parameters** | `ESC 7 n1 n2 n3` (`0x1B 0x37 n1 n2 n3`) | `ESC 7 n1 n2 n3` (`0x1B 0x37 n1 n2 n3`) | Head strobe timing (max dots, heat time, interval) |

---

## 8. Real-Time Hardware Status & Telemetry

### 8.1 Why Real-Time (`DLE EOT`) vs. In-Buffer (`GS r`) Matters

- **`GS r` (In-Buffer Transmit)**: Commands are placed in the printer's receive buffer and evaluated in FIFO sequence. If the printer runs out of paper or the cover is open, buffer processing freezes. As a result, an in-buffer status request will never execute while the printer is in an error condition!
- **`DLE EOT` (Real-Time Transmit)**: Real-time commands bypass the receive buffer and are executed immediately by the printer's interface controller (UART/Ethernet/USB chip) even during active errors, buffer-full conditions, or paper jams.

### 8.2 ESC/POS `DLE EOT n` Specification

Executing `DLE EOT 1..4` (`0x10 0x04 0x01` through `0x10 0x04 0x04`) returns a 4-byte response packet:

| Query | Command Bytes | Bit | Meaning | Value |
| :--- | :--- | :---: | :--- | :--- |
| **DLE EOT 1** | `10 04 01` | Bit 2 | Cash drawer pin 3 switch | `0`: Closed, `1`: Open |
| | | Bit 3 | Online / Offline | `0`: Online, `1`: Offline |
| **DLE EOT 2** | `10 04 02` | Bit 2 | Cover status | `0`: Closed, `1`: Open |
| | | Bit 5 | Printing stopped (paper out) | `0`: Normal, `1`: Out of paper |
| **DLE EOT 3** | `10 04 03` | Bit 3 | Auto-cutter status | `0`: Normal, `1`: Cutter error |
| | | Bit 6 | Thermal print head | `0`: Normal temp, `1`: Overheated |
| **DLE EOT 4** | `10 04 04` | Bits 2, 3 | Roll near-end sensor | `00`: Adequate, `11` (`0x0C`): Near-end |
| | | Bits 5, 6 | Roll end sensor | `00`: Paper present, `11` (`0x60`): Empty |

*(Note: In official Epson specifications, bits 1 and 4 of every DLE EOT response byte are fixed to `1`, yielding an idle base mask of `0x12`).*

### 8.3 StarPRNT Status Specification

- **Real-Time Inquiry (`ENQ`, `0x05`)**: Transmits a 1-byte immediate response:
  - Bit 2: Cash drawer switch (`1`: Open)
  - Bit 3: Offline status (`1`: Offline)
  - Bit 5: Cover open (`1`: Open)
  - Bit 6: Paper empty (`1`: Empty)
- **Auto Status Back (ASB)**: Star printers transmit a 3-to-4 byte frame reporting cover, drawer, near-end detector, cutter jam, and head overheat.

---

## 9. International Character Sets & Code Page Transcoding

### 9.1 Single-Byte Character Architecture
Thermal receipt printers operate strictly as **8-bit character devices**:
- Code points `0x00`–`0x7F` (0–127): Standard 7-bit ASCII.
- Code points `0x80`–`0xFF` (128–255): Mapped to a selectable character code table (Code Page).

> **Common Pitfall**: Modern software uses multi-byte UTF-8. For example, Euro (`€`) is 3 bytes in UTF-8 (`0xE2 0x82 0xAC`). If these 3 bytes are written directly to a thermal printer, the printer interprets each byte as an independent character in its active code page, printing `â,¼` (mojibake).  
> `papermint` includes a zero-dependency **8-bit transcoding engine** that dynamically maps Unicode characters to the physical 8-bit wire byte of the active code page (e.g. `€` ➡️ `0x80` in Windows-1252, or `0xD5` in PC858).

### 9.2 `ESC R n` (Select International Character Set)

Both Epson ESC/POS and StarPRNT Line Mode share the `ESC R n` (`0x1B 0x52 n`) command to replace 12 ASCII punctuation symbols (`#`, `$`, `@`, `[`, `\`, `]`, `^`, `` ` ``, `{`, `|`, `}`, `~`) with localized currency and accented letters:

| `n` | Country Set | Key Symbol Substitutions |
| :---: | :--- | :--- |
| `0` | **USA** | Standard ASCII (`#`, `$`, `@`, `[`, `\`, `]`, `^`, `` ` ``, `{`, `|`, `}`, `~`) |
| `1` | **France** | `$` ➡️ `£`, `@` ➡️ `à`, `[` ➡️ `°`, `\` ➡️ `ç`, `]` ➡️ `§`, `{` ➡️ `é`, `\|` ➡️ `ù`, `}` ➡️ `è` |
| `2` | **Germany** | `@` ➡️ `§`, `[` ➡️ `Ä`, `\` ➡️ `Ö`, `]` ➡️ `Ü`, `{` ➡️ `ä`, `\|` ➡️ `ö`, `}` ➡️ `ü`, `~` ➡️ `ß` |
| `3` | **UK** | `#` ➡️ `£` (Pound sterling) |
| `4` | **Denmark I** | `[` ➡️ `Æ`, `\` ➡️ `Ø`, `]` ➡️ `Å`, `{` ➡️ `æ`, `\|` ➡️ `ø`, `}` ➡️ `å` |
| `5` | **Sweden** | `@` ➡️ `É`, `[` ➡️ `Ä`, `\` ➡️ `Ö`, `]` ➡️ `Å`, `^` ➡️ `Ü`, `{` ➡️ `ä`, `\|` ➡️ `ö`, `}` ➡️ `å`, `~` ➡️ `ü` |
| `6` | **Italy** | `@` ➡️ `§`, `[` ➡️ `°`, `\` ➡️ `ç`, `]` ➡️ `é`, `{` ➡️ `ù`, `\|` ➡️ `à`, `}` ➡️ `ò`, `~` ➡️ `è` |
| `7` | **Spain I** | `$` ➡️ `Pt`, `[` ➡️ `¡`, `\` ➡️ `Ñ`, `]` ➡️ `¿`, `{` ➡️ `¨`, `}` ➡️ `ñ` |
| `8` | **Japan** | `\` ➡️ `¥` (Yen) |
| `17` | **Arabia** | Arabic currency & punctuation set |

### 9.3 Code Page Wire Mappings (`ESC t n` vs `ESC GS t n`)

| Code Page | Description | Epson ESC/POS (`ESC t n`) | StarPRNT (`ESC GS t n`) |
| :--- | :--- | :---: | :---: |
| **`Pc437`** | Standard OEM / USA | `0` | `1` |
| **`Katakana`** | Japanese Katakana | `1` | `2` |
| **`Pc850`** | Multilingual Latin I | `2` | `0` |
| **`Pc860`** | Portuguese | `3` | `6` |
| **`Pc863`** | Canadian-French | `4` | `8` |
| **`Pc865`** | Nordic | `5` | `9` |
| **`Wpc1252`** | Windows Latin I (with Euro `0x80`) | `16` | `32` |
| **`Pc866`** | Cyrillic #2 (Russian) | `17` | `10` |
| **`Pc852`** | Latin II (Slavic / Eastern Europe) | `18` | `5` |
| **`Pc858`** | Multilingual Latin I + Euro (`0xD5`) | `19` | `4` |
| **`Pc720`** | Arabic (DOS) | `32` | `72` |
| **`Pc864`** | Arabic (Simplified) | `37` | `14` |
| **`Wpc1256`** | Windows Arabic | `50` | `72` |
| **`Wpc1250`** | Windows Central European | `45` | `33` |
| **`Wpc1251`** | Windows Cyrillic | `46` | `34` |
| **`Wpc1255`** | Windows Hebrew | `49` | `13` |
| **`Pc857`** | Turkish | `13` | `69` |
| **`Iso8859_15`** | Latin 9 (with Euro `0xA4`) | `40` | `40` |
| **`Pc874`** | Thai | `20` | `22` |

---

## 10. Thermal Printhead Energy & Density Calibration

Thermal printers heat micro-resistors on a linear dot printhead to activate thermal dye embedded in chemical paper. The optimal energy required depends on:
1. **Paper stock sensitivity**: Economical thermal paper typically requires higher heat energy to produce dark black dots, whereas premium high-sensitivity coated labels require less heat to prevent blooming or ghosting.
2. **Ambient temperature**: Colder operating environments (warehouses, outdoor food trucks) require longer pulse times to achieve dot saturation; hot kitchen environments require lower pulse durations to avoid thermal runaway.
3. **Optical contrast requirements**: 2D Barcodes (QR codes, PDF417) and 1-bit raster graphics require sharp dot edges without bleed.

### 10.1 ESC/POS Density Specification (`GS ( K`)

Epson ESC/POS controls print density via Function 48 of the mechanism control sequence:
`GS ( K pL pH fn m` (`0x1D 0x28 0x4B 0x02 0x00 0x30 m`) where:
- `pL = 2`, `pH = 0` (parameter length: 2 bytes)
- `fn = 48` (`0x30`): Function 48 (select print density)
- `m`: Density level
  - `253` (`0xFD`, `-3` in two's complement): Light (~85% energy)
  - `0` (`0x00`): Normal (100% factory default)
  - `3` (`0x03`, `+3`): Dark (~115% energy)
  - `6` (`0x06`, `+6`): High Contrast (~130% energy)

### 10.2 StarPRNT Density Specification (`ESC GS #`)

Star Micronics Line Mode configures print density via the memory switch / density control command:
`ESC GS # '0' n LF NUL` (`0x1B 0x1D 0x23 0x30 n 0x0A 0x00`) where `n` is an ASCII digit:
- `n = '1'` (`0x31`): -2 Light
- `n = '2'` (`0x32`): -1 Medium Light
- `n = '3'` (`0x33`): Standard Normal (100%)
- `n = '4'` (`0x34`): +1 Dark
- `n = '5'` (`0x35`): +2 High Contrast

### 10.3 Heating Strobe Timing Parameters (`ESC 7`)

Direct thermal mechanisms allow fine-grained strobe and pulse timing calibration via `ESC 7 n1 n2 n3` (`0x1B 0x37 n1 n2 n3`):
- `n1` = Maximum heating dots (unit: 8 dots, default `7` = 56 dots). Restricts peak instantaneous current draw across the printhead array.
- `n2` = Heating pulse duration in 10 µs increments (default `80` = 800 µs). Direct micro-resistor activation time.
- `n3` = Heating interval between pulses in 10 µs increments (default `2` = 20 µs). Minimum thermal cooling interval before firing adjacent dots.

---

## 11. Direct Hardware Transports (USB & Serial/RS-232)

### 11.1 Serial / RS-232 Communication

Counter POS terminals and legacy retail devices frequently communicate over serial RS-232 (DB9 / RJ45) or virtual COM port USB-to-UART bridges (FTDI, Prolific, CH340, Silicon Labs CP2102).

#### Baud Rates & Framing:
- Typical thermal printer baud rates: `9600`, `19200` (Epson TM-T88 default), `38400` (Star default), or `115200`.
- Standard framing: 8 data bits, no parity, 1 stop bit (`8N1`).

#### Hardware Flow Control (RTS/CTS):
Thermal printers maintain small internal receive buffers (typically 4 KB). When printing complex graphics, long receipts, or high-density barcodes at high baud rates, host transmitters can easily overrun the printer buffer unless flow control is enabled:
- **Hardware Handshaking (RTS/CTS)**: The printer signals buffer saturation over CTS/RTS hardware lines.
- **Recommendation**: Always enable `.flow_control_hardware()` when connecting over high-speed serial.

### 11.2 USB Printer Class (Class 07) Communication

USB thermal receipt printers implement standard **USB Device Class Definition for Printing Devices (Class 07, SubClass 01)**.

#### Endpoint Topology:
- **Interface**: Class `0x07` (Printer), SubClass `0x01` (Printer), Protocol `0x01` (Unidirectional), `0x02` (Bidirectional), or `0x03` (1284.4).
- **Bulk OUT Endpoint**: (Address typically `0x01` or `0x02`): Primary data pipeline used for transmitting raster graphics, line feeds, and POS commands.
- **Bulk IN Endpoint**: (Address typically `0x81` or `0x82`): Used for receiving real-time status telemetry (`DLE EOT` / `ENQ` responses) and device ID queries.

#### Common Vendor ID (VID) and Product ID (PID) References:
| Manufacturer | Vendor ID (VID) | Common Product ID (PID) | Models |
| :--- | :---: | :---: | :--- |
| **Epson** | `0x04B8` | `0x0202` | TM-T88IV, TM-T88V, TM-T88VI, TM-T20 |
| **Star Micronics** | `0x0519` | `0x0001` / `0x0003` | TSP100, TSP650, TSP700 |
| **Citizen** | `0x1D90` | `0x2060` | CT-S310, CT-S651 |
| **Bixolon** | `0x1504` | `0x0006` | SRP-350, SRP-330 |
| **Xprinter / Generic** | `0x0416` / `0x1FC9` | `0x5011` / `0x2016` | XP-58, XP-80 |





