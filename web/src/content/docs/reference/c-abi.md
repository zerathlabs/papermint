---
title: "C-ABI Header Reference"
description: "papermint.h C-compatible foreign function interface reference"
---

# 📑 C-ABI Header Reference (`papermint.h`)

`papermint-mobile` exports a clean, standard C foreign function interface (FFI) defined in [`include/papermint.h`](https://github.com/zerathlabs/papermint/blob/main/crates/papermint-mobile/include/papermint.h).

This header can be included directly in:
* **Expo SDK 56+ Inline Modules** (Swift on iOS, Kotlin on Android)
* **React Native C++ JSI / TurboModules**
* **Flutter** (`dart:ffi`)
* **Objective-C & Swift bridging headers** in Xcode

---

## Header Overview

```c
#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

/* Paper width constants */
#define PAPERMINT_WIDTH_80MM 0
#define PAPERMINT_WIDTH_58MM 1

/* Alignment constants */
#define PAPERMINT_ALIGN_LEFT   0
#define PAPERMINT_ALIGN_CENTER 1
#define PAPERMINT_ALIGN_RIGHT  2

/* Dialect constants */
#define PAPERMINT_DIALECT_ESCPOS 0
#define PAPERMINT_DIALECT_STAR   1
```

---

## 1. High-Performance One-Shot JSON Compiler

```c
/**
 * Compiles a JSON ticket payload into binary ESC/POS or StarPRNT wire bytes in ~10 microseconds.
 *
 * @param json_str Null-terminated UTF-8 JSON ticket string.
 * @param dialect  0 for Epson ESC/POS, 1 for StarPRNT.
 * @param out_len  Output pointer where buffer length is written.
 * @return Pointer to heap byte buffer. MUST be freed with papermint_bytes_free.
 */
uint8_t* papermint_compile_json(const char* json_str, uint8_t dialect, size_t* out_len);
```

---

## 2. Fluent Handle API

```c
/* Opaque handle */
typedef struct PapermintReceipt PapermintReceipt;

PapermintReceipt* papermint_receipt_create(uint8_t paper_width);
void papermint_receipt_free(PapermintReceipt* handle);
void papermint_receipt_init(PapermintReceipt* handle);
void papermint_receipt_text(PapermintReceipt* handle, const char* text);
void papermint_receipt_text_ln(PapermintReceipt* handle, const char* text);
void papermint_receipt_align(PapermintReceipt* handle, uint8_t align);
void papermint_receipt_bold(PapermintReceipt* handle, bool enable);
void papermint_receipt_underline(PapermintReceipt* handle, uint8_t mode);
void papermint_receipt_feed(PapermintReceipt* handle, uint8_t lines);
void papermint_receipt_divider(PapermintReceipt* handle, const char* style);
void papermint_receipt_table_row(PapermintReceipt* handle, const char* const* cols, size_t col_count);
void papermint_receipt_qr(PapermintReceipt* handle, const char* content);
void papermint_receipt_barcode(PapermintReceipt* handle, const char* content);
void papermint_receipt_cut(PapermintReceipt* handle, bool partial);
void papermint_receipt_beep(PapermintReceipt* handle, uint8_t count, uint8_t duration);
void papermint_receipt_open_drawer(PapermintReceipt* handle);
uint8_t* papermint_receipt_encode(const PapermintReceipt* handle, uint8_t dialect, size_t* out_len);
```

---

## 3. Memory Deallocation

```c
/**
 * Safely deallocates a byte buffer returned by papermint_receipt_encode or papermint_compile_json.
 */
void papermint_bytes_free(uint8_t* ptr, size_t len);
```
