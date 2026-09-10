---
title: Introduction
description: What is papermint and why was it built?
---

# 🌿 Introduction to papermint

**papermint** is a fast, multi-vendor, type-safe thermal receipt printing engine designed from first principles in Rust.

---

## The Problem with Legacy Thermal Printing

For decades, developers building Point-of-Sale (POS) systems, kitchen ticket routers, self-checkout kiosks, and mobile receipts relied on fragile legacy libraries in Python or JavaScript (such as `python-escpos` or `node-thermal-printer`).

These legacy libraries share fundamental architectural flaws:
* **Ad-hoc byte concatenation**: Functions directly append raw hex escape bytes into an array buffer. If you need to switch printer vendors (e.g. from Epson to Star Micronics), you have to rewrite your formatting logic.
* **Jagged, broken tables**: They calculate character padding using `str.length`. When printing international characters (Japanese, Chinese, Arabic, emojis, or accents), table columns distort and misalign because they fail to account for monospace terminal cell widths.
* **Kernel driver crashes**: USB communication in legacy libraries often requires `pyusb` and kernel driver detachment (`detach_kernel_driver(0)`), which destabilizes operating system USB hubs and requires root (`sudo`) privileges.
* **Printer buffer overflows**: Sending large images without chunking exceeds the 4KB–64KB volatile RAM limits of thermal printers, causing printers to freeze, spit out random characters, or crash.

---

## The Papermint Solution

`papermint` redesigns thermal printing around an **Intermediate Representation (IR)**:

```
┌─────────────────────────────────────────────────────────────┐
│ 1. Layout Engine (Receipt Builder)                          │
│    Produces an abstract stream of instructions              │
└──────────────────────────────┬──────────────────────────────┘
                               │ Vec<Command>
                               ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. Dialect Encoders (EscPos / StarPRNT)                     │
│    Translates commands into vendor wire bytes on demand     │
└──────────────────────────────┬──────────────────────────────┘
                               │ Wire Bytes (Vec<u8>)
                               ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. Driverless Transports                                    │
│    TCP Port 9100 | Raw USB (nusb) | RS-232 | In-Memory Mock │
└─────────────────────────────────────────────────────────────┘
```

### Core Highlights
1. **Universal Portability**: Write your receipt logic once. Render it on Epson ESC/POS, StarPRNT, or an in-memory mock sink for testing.
2. **Typographic Monospace Precision**: Monospace column mathematics powered by **Unicode Standard Annex #11 (East Asian Width)** with synchronized multiline row wrapping.
3. **Driverless USB**: Communicates directly with operating system USB bulk endpoints via `nusb` without vendor drivers or `sudo`.
4. **Universal Ecosystem**: Use `papermint` in pure Rust, in Node.js/TypeScript via N-API (`papermint`), over HTTP via `papermintd`, or in mobile apps via Expo SDK 56+ Inline Modules.
