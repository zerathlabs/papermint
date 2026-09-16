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

/* Fluent C-ABI handle functions */


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

/* High-performance one-shot JSON ticket compiler */


/**
 * Compiles a JSON ticket payload into binary ESC/POS or StarPRNT wire bytes.
 *
 * Returns pointer to heap byte buffer. Must be freed with papermint_bytes_free.
 */
uint8_t* papermint_compile_json(const char* json_str, uint8_t dialect, size_t* out_len);

/**
 * Renders a JSON ticket payload into a virtual SVG vector graphic string.
 *
 * Returns a heap-allocated UTF-8 C string. Must be freed with papermint_string_free.
 */
char* papermint_compile_json_svg(const char* json_str);

/**
 * Renders a JSON ticket payload into a responsive HTML preview string.
 *
 * Returns a heap-allocated UTF-8 C string. Must be freed with papermint_string_free.
 */
char* papermint_compile_json_html(const char* json_str);

/**
 * Frees a string allocated by papermint_compile_json_svg or papermint_compile_json_html.
 */
void papermint_string_free(char* ptr);

/* Label Printing (TSPL & ZPL II) */

/* Dialect constants for labels */
#define PAPERMINT_LABEL_DIALECT_TSPL 0
#define PAPERMINT_LABEL_DIALECT_ZPL  1

/* Barcode type constants for labels */
#define PAPERMINT_BARCODE_CODE128 0
#define PAPERMINT_BARCODE_CODE39  1
#define PAPERMINT_BARCODE_EAN13   2
#define PAPERMINT_BARCODE_EAN8    3
#define PAPERMINT_BARCODE_UPCA    4
#define PAPERMINT_BARCODE_ITF     5

/* QR ECC constants for labels */
#define PAPERMINT_QR_ECC_M 0
#define PAPERMINT_QR_ECC_L 1
#define PAPERMINT_QR_ECC_Q 2
#define PAPERMINT_QR_ECC_H 3

/* Opaque label handle */
typedef struct PapermintLabel PapermintLabel;

/** Creates a 2D label canvas instance (width and height in mm, dpi: default 203 if 0). */
PapermintLabel* papermint_label_create(float width_mm, float height_mm, uint32_t dpi);

/** Frees a label canvas instance. */
void papermint_label_free(PapermintLabel* handle);

/** Sets label gap sensor dimensions in mm. */
void papermint_label_gap(PapermintLabel* handle, float gap_mm, float offset_mm);

/** Sets print speed in inches per second. */
void papermint_label_speed(PapermintLabel* handle, uint32_t speed);

/** Sets print density/darkness (0 to 15). */
void papermint_label_density(PapermintLabel* handle, uint32_t density);

/** Sets feed direction (0 = Forward/Normal, 1 = Inverted). */
void papermint_label_direction(PapermintLabel* handle, uint8_t direction);

/** Appends 2D text at (x, y) coordinates with multipliers. */
void papermint_label_text(PapermintLabel* handle, uint32_t x, uint32_t y, const char* text, uint32_t x_mult, uint32_t y_mult);

/** Appends a 1D barcode at (x, y). */
void papermint_label_barcode(PapermintLabel* handle, uint32_t x, uint32_t y, uint8_t code_type, uint32_t height, const char* content, bool readable);

/** Appends a 2D QR Code at (x, y). */
void papermint_label_qr(PapermintLabel* handle, uint32_t x, uint32_t y, const char* content, uint32_t cell_width, uint8_t ecc);

/** Appends a rectangular outline box at (x, y). */
void papermint_label_box(PapermintLabel* handle, uint32_t x, uint32_t y, uint32_t w, uint32_t h, uint32_t thickness);

/** Appends a solid separator bar at (x, y). */
void papermint_label_line(PapermintLabel* handle, uint32_t x, uint32_t y, uint32_t w, uint32_t h);

/** Inverts pixels in a rectangular area (highlight badge). */
void papermint_label_reverse(PapermintLabel* handle, uint32_t x, uint32_t y, uint32_t w, uint32_t h);

/** Sets number of print copies. */
void papermint_label_print(PapermintLabel* handle, uint32_t copies);

/** Encodes label canvas into TSPL-II wire bytes. Must be freed with papermint_bytes_free. */
uint8_t* papermint_label_encode_tspl(PapermintLabel* handle, size_t* out_len);

/** Encodes label canvas into ZPL II wire bytes. Must be freed with papermint_bytes_free. */
uint8_t* papermint_label_encode_zpl(PapermintLabel* handle, size_t* out_len);

/** Renders virtual SVG sticker preview. Must be freed with papermint_string_free. */
char* papermint_label_render_svg(PapermintLabel* handle);

/** Compiles JSON label specification into TSPL (0) or ZPL (1) bytes. Must be freed with papermint_bytes_free. */
uint8_t* papermint_label_compile_json(const char* json_str, uint8_t dialect, size_t* out_len);

/** Renders JSON label specification into SVG string. Must be freed with papermint_string_free. */
char* papermint_label_render_svg_json(const char* json_str);

#ifdef __cplusplus
}
#endif

#endif /* PAPERMINT_H */
