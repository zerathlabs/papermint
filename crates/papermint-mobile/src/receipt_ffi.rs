//! Fluent C-ABI handle functions for thermal receipt printing.

use std::ffi::{CStr, c_char};
use std::slice;

use papermint::{Alignment, CutMode, Encoder, PaperWidth, Receipt, UnderlineMode};

use crate::memory::text_is_empty;

/// Opaque wrapper around [`papermint::Receipt`].
pub struct PapermintReceipt {
    pub(crate) inner: Receipt,
}

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
