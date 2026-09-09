//! papermint-mobile — Mobile C-ABI and zero-copy byte compiler.
//!
//! Provides a high-performance C-compatible foreign function interface (FFI)
//! designed for React Native (C++ JSI / TurboModules), Expo SDK 56+ (Inline Modules),
//! Flutter (dart:ffi), iOS Swift, and Android Kotlin.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{CStr, c_char};
use std::slice;

use serde::Deserialize;

use papermint::{Alignment, CutMode, Encoder, PaperWidth, Receipt, UnderlineMode};

/// Opaque wrapper around [`papermint::Receipt`].
pub struct PapermintReceipt {
    inner: Receipt,
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Fluent C-ABI Handle Functions
// ─────────────────────────────────────────────────────────────────────────────

/// Creates a new receipt builder instance.
///
/// * `paper_width`: `0` for 80mm (default), `1` for 58mm.
///
/// Returns a heap-allocated pointer. Must be freed with [`papermint_receipt_free`].
#[unsafe(no_mangle)]
pub extern "C" fn papermint_receipt_create(paper_width: u8) -> *mut PapermintReceipt {
    let width = if paper_width == 1 {
        PaperWidth::Mm58
    } else {
        PaperWidth::Mm80
    };

    let receipt = Box::new(PapermintReceipt {
        inner: Receipt::new(width),
    });

    Box::into_raw(receipt)
}

/// Frees a receipt builder instance allocated by [`papermint_receipt_create`].
#[unsafe(no_mangle)]
pub extern "C" fn papermint_receipt_free(handle: *mut PapermintReceipt) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

/// Appends a printer hardware reset sequence (clears styles, resets tabs).
#[unsafe(no_mangle)]
pub extern "C" fn papermint_receipt_init(handle: *mut PapermintReceipt) {
    if let Some(r) = unsafe { handle.as_mut() } {
        r.inner = std::mem::take(&mut r.inner).init();
    }
}

/// Appends raw text without a trailing newline.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_receipt_text(handle: *mut PapermintReceipt, text: *const c_char) {
    if text.is_null() {
        return;
    }
    if let (Some(r), Ok(s)) = (
        unsafe { handle.as_mut() },
        unsafe { CStr::from_ptr(text) }.to_str(),
    ) {
        r.inner = std::mem::take(&mut r.inner).text(s);
    }
}

/// Appends text followed by a line feed.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_receipt_text_ln(handle: *mut PapermintReceipt, text: *const c_char) {
    if text.is_null() {
        return;
    }
    if let (Some(r), Ok(s)) = (
        unsafe { handle.as_mut() },
        unsafe { CStr::from_ptr(text) }.to_str(),
    ) {
        r.inner = std::mem::take(&mut r.inner).text_ln(s);
    }
}

/// Sets text alignment for subsequent commands.
///
/// * `align`: `0` = Left, `1` = Center, `2` = Right.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_receipt_align(handle: *mut PapermintReceipt, align: u8) {
    if let Some(r) = unsafe { handle.as_mut() } {
        let a = match align {
            1 => Alignment::Center,
            2 => Alignment::Right,
            _ => Alignment::Left,
        };
        r.inner = std::mem::take(&mut r.inner).align(a);
    }
}

/// Sets bold text emphasis.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_receipt_bold(handle: *mut PapermintReceipt, enable: bool) {
    if let Some(r) = unsafe { handle.as_mut() } {
        r.inner = std::mem::take(&mut r.inner).bold(enable);
    }
}

/// Sets underline mode.
///
/// * `mode`: `0` = None, `1` = Single, `2` = Double.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_receipt_underline(handle: *mut PapermintReceipt, mode: u8) {
    if let Some(r) = unsafe { handle.as_mut() } {
        let u = match mode {
            1 => UnderlineMode::Single,
            2 => UnderlineMode::Double,
            _ => UnderlineMode::Off,
        };
        r.inner = std::mem::take(&mut r.inner).underline(u);
    }
}

/// Feeds `lines` blank lines.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_receipt_feed(handle: *mut PapermintReceipt, lines: u8) {
    if let Some(r) = unsafe { handle.as_mut() } {
        r.inner = std::mem::take(&mut r.inner).feed(lines);
    }
}

/// Appends a full-width repeating divider line (e.g. "-", "=", "*").
#[unsafe(no_mangle)]
pub extern "C" fn papermint_receipt_divider(handle: *mut PapermintReceipt, style: *const c_char) {
    if let Some(r) = unsafe { handle.as_mut() } {
        let s = if !style.is_null() {
            unsafe { CStr::from_ptr(style) }.to_str().unwrap_or("-")
        } else {
            "-"
        };
        let ch = s.chars().next().unwrap_or('-');
        r.inner = std::mem::take(&mut r.inner).divider(ch);
    }
}

/// Appends a synchronized multi-column table row with auto-wrapping and CJK width calculations.
///
/// * `cols`: Array of null-terminated UTF-8 C strings.
/// * `col_count`: Number of elements in `cols`.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_receipt_table_row(
    handle: *mut PapermintReceipt,
    cols: *const *const c_char,
    col_count: usize,
) {
    if handle.is_null() || cols.is_null() || col_count == 0 {
        return;
    }

    if let Some(r) = unsafe { handle.as_mut() } {
        let col_ptrs = unsafe { slice::from_raw_parts(cols, col_count) };
        let mut row_strings: Vec<String> = Vec::with_capacity(col_count);

        for &ptr in col_ptrs {
            if ptr.is_null() {
                row_strings.push(String::new());
            } else if let Ok(s) = unsafe { CStr::from_ptr(ptr) }.to_str() {
                row_strings.push(s.to_string());
            } else {
                row_strings.push(String::new());
            }
        }

        let row_refs: Vec<&str> = row_strings.iter().map(String::as_str).collect();
        r.inner = std::mem::take(&mut r.inner).row(&row_refs);
    }
}

/// Appends a 2D QR Code.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_receipt_qr(handle: *mut PapermintReceipt, content: *const c_char) {
    if text_is_empty(content) {
        return;
    }
    if let (Some(r), Ok(s)) = (
        unsafe { handle.as_mut() },
        unsafe { CStr::from_ptr(content) }.to_str(),
    ) {
        r.inner = std::mem::take(&mut r.inner).qr(s);
    }
}

/// Appends a Code 128 1D Barcode.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_receipt_barcode(handle: *mut PapermintReceipt, content: *const c_char) {
    if text_is_empty(content) {
        return;
    }
    if let (Some(r), Ok(s)) = (
        unsafe { handle.as_mut() },
        unsafe { CStr::from_ptr(content) }.to_str(),
    ) {
        r.inner = std::mem::take(&mut r.inner).barcode_128(s);
    }
}

/// Appends a paper cut command.
///
/// * `partial`: If `true`, performs partial cut (with feed). If `false`, performs full cut.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_receipt_cut(handle: *mut PapermintReceipt, partial: bool) {
    if let Some(r) = unsafe { handle.as_mut() } {
        let mode = if partial {
            CutMode::Partial
        } else {
            CutMode::Full
        };
        r.inner = std::mem::take(&mut r.inner).cut(mode);
    }
}

/// Triggers acoustic buzzer alert.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_receipt_beep(handle: *mut PapermintReceipt, count: u8, duration: u8) {
    if let Some(r) = unsafe { handle.as_mut() } {
        r.inner = std::mem::take(&mut r.inner).beep(count, duration);
    }
}

/// Triggers cash drawer kickout pulse.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_receipt_open_drawer(handle: *mut PapermintReceipt) {
    if let Some(r) = unsafe { handle.as_mut() } {
        r.inner = std::mem::take(&mut r.inner).open_drawer();
    }
}

/// Compiles the accumulated receipt commands into a heap-allocated raw binary byte buffer.
///
/// * `dialect`: `0` = Epson ESC/POS, `1` = Star Micronics (StarPRNT).
/// * `out_len`: Output parameter where the buffer byte length is written.
///
/// Returns a pointer to the buffer. The caller MUST free this buffer using [`papermint_bytes_free`].
/// Returns null and sets `*out_len = 0` on error or null handle.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_receipt_encode(
    handle: *const PapermintReceipt,
    dialect: u8,
    out_len: *mut usize,
) -> *mut u8 {
    if out_len.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { *out_len = 0 };

    let r = match unsafe { handle.as_ref() } {
        Some(val) => val,
        None => return std::ptr::null_mut(),
    };

    let encode_result = if dialect == 1 {
        Encoder::star().encode(r.inner.commands())
    } else {
        Encoder::escpos().encode(r.inner.commands())
    };

    match encode_result {
        Ok(bytes) => {
            let mut boxed = bytes.into_boxed_slice();
            let len = boxed.len();
            let ptr = boxed.as_mut_ptr();
            std::mem::forget(boxed);
            unsafe { *out_len = len };
            ptr
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Safely deallocates a byte buffer previously returned by [`papermint_receipt_encode`] or [`papermint_compile_json`].
#[unsafe(no_mangle)]
pub extern "C" fn papermint_bytes_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        unsafe {
            drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len)));
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. High-Performance One-Shot JSON Ticket Compiler
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct MobileTicketItem {
    pub qty: Option<String>,
    #[serde(alias = "name")]
    pub description: String,
    #[allow(dead_code)]
    pub price: Option<String>,
    pub total: String,
}

#[derive(Debug, Deserialize)]
struct MobileTicketMetadata {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
struct MobileTicketPayload {
    pub paper_width: Option<String>, // "80mm" or "58mm"
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub address: Option<String>,
    pub metadata: Option<Vec<MobileTicketMetadata>>,
    pub items: Option<Vec<MobileTicketItem>>,
    pub totals: Option<Vec<MobileTicketMetadata>>,
    pub qr: Option<String>,
    pub barcode: Option<String>,
    pub footer: Option<String>,
    pub cut_mode: Option<String>, // "full", "partial", "none"
    pub open_drawer: Option<bool>,
    pub beep: Option<u8>,
    pub divider_style: Option<String>,
}

/// Compiles a complete receipt JSON specification into binary wire bytes in ~10 microseconds.
///
/// * `json_str`: Null-terminated UTF-8 JSON string matching the mobile ticket schema.
/// * `dialect`: `0` = Epson ESC/POS, `1` = Star Micronics (StarPRNT).
/// * `out_len`: Output parameter where the buffer byte length is written.
///
/// Returns a pointer to the heap buffer. Must be freed with [`papermint_bytes_free`].
#[unsafe(no_mangle)]
pub extern "C" fn papermint_compile_json(
    json_str: *const c_char,
    dialect: u8,
    out_len: *mut usize,
) -> *mut u8 {
    if out_len.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { *out_len = 0 };

    if json_str.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = match unsafe { CStr::from_ptr(json_str) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let ticket: MobileTicketPayload = match serde_json::from_str(c_str) {
        Ok(t) => t,
        Err(_) => return std::ptr::null_mut(),
    };

    let width = match ticket.paper_width.as_deref() {
        Some("58mm" | "58" | "Mm58") => PaperWidth::Mm58,
        _ => PaperWidth::Mm80,
    };

    let mut r = Receipt::new(width).init();

    // 1. Header (Title, Subtitle, Address)
    if let Some(title) = &ticket.title {
        r = r.center().bold(true).text_ln(title).bold(false);
    }
    if let Some(subtitle) = &ticket.subtitle {
        r = r.center().text_ln(subtitle);
    }
    if let Some(addr) = &ticket.address {
        r = r.center().text_ln(addr);
    }

    let div_char = ticket
        .divider_style
        .as_deref()
        .and_then(|s| s.chars().next())
        .unwrap_or('-');

    if ticket.title.is_some() || ticket.subtitle.is_some() || ticket.address.is_some() {
        r = r.divider(div_char);
    }

    // 2. Metadata Key-Value pairs
    if let Some(meta) = &ticket.metadata {
        for m in meta {
            r = r.left().row(&[&m.label, &m.value]);
        }
        if !meta.is_empty() {
            r = r.divider(div_char);
        }
    }

    // 3. Line Items Table
    if let Some(items) = &ticket.items {
        // Table Header
        r = r.bold(true).row(&["QTY", "ITEM", "TOTAL"]).bold(false);
        r = r.divider('-');

        for item in items {
            let qty = item.qty.as_deref().unwrap_or("1");
            r = r.row(&[qty, &item.description, &item.total]);
        }
        r = r.divider(div_char);
    }

    // 4. Totals (Subtotal, Tax, Final Total)
    if let Some(totals) = &ticket.totals {
        for t in totals {
            if t.label.eq_ignore_ascii_case("total") || t.label.eq_ignore_ascii_case("grand total")
            {
                r = r.bold(true).row(&[&t.label, &t.value]).bold(false);
            } else {
                r = r.row(&[&t.label, &t.value]);
            }
        }
        r = r.divider(div_char);
    }

    // 5. QR Code
    if let Some(qr_url) = &ticket.qr {
        r = r.center().qr(qr_url).feed(1);
    }

    // 6. Barcode
    if let Some(bc) = &ticket.barcode {
        r = r.center().barcode_128(bc).feed(1);
    }

    // 7. Footer
    if let Some(footer) = &ticket.footer {
        r = r.center().text_ln(footer);
    }

    // 8. Cash Drawer Kickout
    if ticket.open_drawer.unwrap_or(false) {
        r = r.open_drawer();
    }

    // 9. Buzzer Beep
    if let Some(b) = ticket.beep.filter(|&b| b > 0) {
        r = r.beep(b, 2);
    }

    // 10. Cut Mode
    match ticket.cut_mode.as_deref() {
        Some("none" | "None") => {}
        Some("partial" | "Partial") => {
            r = r.feed(3).cut(CutMode::Partial);
        }
        _ => {
            r = r.feed(3).cut(CutMode::Full);
        }
    }

    // Encode to Wire Bytes
    let encode_res = if dialect == 1 {
        Encoder::star().encode(r.commands())
    } else {
        Encoder::escpos().encode(r.commands())
    };

    match encode_res {
        Ok(bytes) => {
            let mut boxed = bytes.into_boxed_slice();
            let len = boxed.len();
            let ptr = boxed.as_mut_ptr();
            std::mem::forget(boxed);
            unsafe { *out_len = len };
            ptr
        }
        Err(_) => std::ptr::null_mut(),
    }
}

fn text_is_empty(ptr: *const c_char) -> bool {
    if ptr.is_null() {
        true
    } else {
        unsafe { *ptr == 0 }
    }
}
