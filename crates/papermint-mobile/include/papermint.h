/**
 * papermint.h — C-ABI Header for Papermint Thermal Printing Engine
 *
 * High-performance receipt formatting and ESC/POS & StarPRNT byte generation
 * for React Native, Expo SDK 56+ Inline Modules, Flutter, iOS, and Android.
 */

#ifndef PAPERMINT_H
#define PAPERMINT_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Paper width constants */
#define PAPERMINT_WIDTH_80MM 0
#define PAPERMINT_WIDTH_58MM 1

/* Alignment constants */
#define PAPERMINT_ALIGN_LEFT   0
#define PAPERMINT_ALIGN_CENTER 1
#define PAPERMINT_ALIGN_RIGHT  2

/* Underline constants */
#define PAPERMINT_UNDERLINE_NONE   0
#define PAPERMINT_UNDERLINE_SINGLE 1
#define PAPERMINT_UNDERLINE_DOUBLE 2

/* Dialect constants */
#define PAPERMINT_DIALECT_ESCPOS 0
#define PAPERMINT_DIALECT_STAR   1

/* Opaque receipt handle */
typedef struct PapermintReceipt PapermintReceipt;

/* ─────────────────────────────────────────────────────────────────────────────
 * 1. Fluent C-ABI Handle Functions
 * ───────────────────────────────────────────────────────────────────────────── */

/** Creates a new receipt builder instance. Must be freed with papermint_receipt_free. */
PapermintReceipt* papermint_receipt_create(uint8_t paper_width);

/** Frees a receipt builder handle. */
void papermint_receipt_free(PapermintReceipt* handle);

/** Clears formatting and resets printer state. */
void papermint_receipt_init(PapermintReceipt* handle);

/** Appends raw text without trailing newline. */
void papermint_receipt_text(PapermintReceipt* handle, const char* text);

/** Appends text followed by line feed. */
void papermint_receipt_text_ln(PapermintReceipt* handle, const char* text);

/** Sets text alignment (0=Left, 1=Center, 2=Right). */
void papermint_receipt_align(PapermintReceipt* handle, uint8_t align);

/** Enables or disables bold text emphasis. */
void papermint_receipt_bold(PapermintReceipt* handle, bool enable);

/** Sets underline mode (0=None, 1=Single, 2=Double). */
void papermint_receipt_underline(PapermintReceipt* handle, uint8_t mode);

/** Feeds blank lines. */
void papermint_receipt_feed(PapermintReceipt* handle, uint8_t lines);

/** Appends a full-width repeating divider (e.g. "-", "=", "*"). */
void papermint_receipt_divider(PapermintReceipt* handle, const char* style);

/** Appends a synchronized multi-column row with CJK/Unicode width calculation. */
void papermint_receipt_table_row(PapermintReceipt* handle, const char* const* cols, size_t col_count);

/** Appends a 2D QR Code. */
void papermint_receipt_qr(PapermintReceipt* handle, const char* content);

/** Appends a Code 128 1D Barcode. */
void papermint_receipt_barcode(PapermintReceipt* handle, const char* content);

/** Appends a paper cut command (partial or full). */
void papermint_receipt_cut(PapermintReceipt* handle, bool partial);

/** Triggers acoustic buzzer alert. */
void papermint_receipt_beep(PapermintReceipt* handle, uint8_t count, uint8_t duration);

/** Triggers cash drawer kickout pulse. */
void papermint_receipt_open_drawer(PapermintReceipt* handle);

/** Compiles commands into raw binary wire bytes. Must be freed with papermint_bytes_free. */
uint8_t* papermint_receipt_encode(const PapermintReceipt* handle, uint8_t dialect, size_t* out_len);

/** Frees a byte buffer allocated by papermint_receipt_encode or papermint_compile_json. */
void papermint_bytes_free(uint8_t* ptr, size_t len);

/* ─────────────────────────────────────────────────────────────────────────────
 * 2. High-Performance One-Shot JSON Ticket Compiler
 * ───────────────────────────────────────────────────────────────────────────── */

/**
 * Compiles a JSON ticket payload into binary ESC/POS or StarPRNT wire bytes.
 *
 * Returns pointer to heap byte buffer. Must be freed with papermint_bytes_free.
 */
uint8_t* papermint_compile_json(const char* json_str, uint8_t dialect, size_t* out_len);

#ifdef __cplusplus
}
#endif

#endif /* PAPERMINT_H */
