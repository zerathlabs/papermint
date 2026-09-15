//! papermint-mobile — Mobile C-ABI and zero-copy byte compiler.
//!
//! Provides a high-performance C-compatible foreign function interface (FFI)
//! designed for React Native (C++ JSI / TurboModules), Expo SDK 56+ (Inline Modules),
//! Flutter (dart:ffi), iOS Swift, and Android Kotlin.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{CStr, CString, c_char};
use std::slice;

use serde::Deserialize;

use papermint::{
    Alignment, CodePage, ColumnWidth, CutMode, Encoder, PaperWidth, Receipt, TableColumn,
    UnderlineMode, base64_decode,
};

/// Opaque wrapper around [`papermint::Receipt`].
pub struct PapermintReceipt {
    inner: Receipt,
}

// Fluent C-ABI handle functions

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

// High-performance JSON ticket compiler & Command IR engine

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
struct MobileTableColumn {
    pub width_fixed: Option<usize>,
    pub width: Option<usize>,
    pub width_fraction: Option<f32>,
    pub align: Option<String>,
    pub alignment: Option<String>,
}

const fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum MobileCommand {
    #[serde(rename = "text")]
    Text {
        text: String,
        align: Option<String>,
        bold: Option<bool>,
        underline: Option<bool>,
        invert: Option<bool>,
        double_width: Option<bool>,
        double_height: Option<bool>,
        double_size: Option<bool>,
    },
    #[serde(rename = "text_ln")]
    TextLn {
        text: String,
        align: Option<String>,
        bold: Option<bool>,
        underline: Option<bool>,
        invert: Option<bool>,
        double_width: Option<bool>,
        double_height: Option<bool>,
        double_size: Option<bool>,
    },
    #[serde(rename = "feed")]
    Feed { lines: Option<u8> },
    #[serde(rename = "align")]
    Align {
        #[serde(alias = "alignment")]
        align: String,
    },
    #[serde(rename = "bold")]
    Bold {
        #[serde(alias = "enabled", default = "default_true")]
        enable: bool,
    },
    #[serde(rename = "underline")]
    Underline {
        #[serde(alias = "enabled", default = "default_true")]
        enable: bool,
        mode: Option<String>,
    },
    #[serde(rename = "invert")]
    Invert {
        #[serde(alias = "enabled", default = "default_true")]
        enable: bool,
    },
    #[serde(rename = "double_size")]
    DoubleSize {
        #[serde(alias = "enabled", default = "default_true")]
        enable: bool,
    },
    #[serde(rename = "double_width")]
    DoubleWidth {
        #[serde(alias = "enabled", default = "default_true")]
        enable: bool,
    },
    #[serde(rename = "double_height")]
    DoubleHeight {
        #[serde(alias = "enabled", default = "default_true")]
        enable: bool,
    },
    #[serde(rename = "divider")]
    Divider {
        style: Option<String>,
        variant: Option<String>,
        pattern: Option<String>,
    },
    #[serde(rename = "divider_double")]
    DividerDouble,
    #[serde(rename = "divider_dotted")]
    DividerDotted,
    #[serde(rename = "divider_dashed")]
    DividerDashed,
    #[serde(rename = "two_column")]
    TwoColumn { left: String, right: String },
    #[serde(rename = "three_column")]
    ThreeColumn {
        left: String,
        center: String,
        right: String,
    },
    #[serde(rename = "table_header")]
    TableHeader {
        #[serde(default)]
        headers: Vec<String>,
        #[serde(default)]
        columns: Vec<MobileTableColumn>,
    },
    #[serde(rename = "set_columns")]
    SetColumns { columns: Vec<MobileTableColumn> },
    #[serde(rename = "row")]
    Row { cells: Vec<String> },
    #[serde(rename = "clear_columns")]
    ClearColumns,
    #[serde(rename = "qr")]
    Qr { content: String },
    #[serde(rename = "barcode")]
    Barcode { content: String },
    #[serde(rename = "code_page")]
    CodePage { page: serde_json::Value },
    #[serde(rename = "image")]
    Image {
        data: String,
        max_width: Option<u32>,
    },
    #[serde(rename = "raw")]
    Raw {
        bytes: Option<Vec<u8>>,
        base64: Option<String>,
    },
    #[serde(rename = "open_drawer")]
    OpenDrawer,
    #[serde(rename = "beep")]
    Beep {
        count: Option<u8>,
        duration: Option<u8>,
    },
    #[serde(rename = "cut")]
    Cut {
        #[serde(alias = "cut_mode")]
        mode: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
struct MobileTicketPayload {
    pub paper_width: Option<String>, // "80mm" or "58mm"
    pub code_page: Option<serde_json::Value>,
    pub commands: Option<Vec<MobileCommand>>,
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

fn parse_alignment(s: &str) -> Alignment {
    match s.to_ascii_lowercase().as_str() {
        "center" => Alignment::Center,
        "right" => Alignment::Right,
        _ => Alignment::Left,
    }
}

fn parse_code_page_value(val: &serde_json::Value) -> CodePage {
    match val {
        serde_json::Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                CodePage::Custom(u as u8)
            } else {
                CodePage::Pc437
            }
        }
        serde_json::Value::String(s) => match s.to_ascii_lowercase().as_str() {
            "wpc1256" | "cp1256" | "windows-1256" => CodePage::Wpc1256,
            "pc437" | "cp437" => CodePage::Pc437,
            "katakana" => CodePage::Katakana,
            "pc850" | "cp850" => CodePage::Pc850,
            "pc860" | "cp860" => CodePage::Pc860,
            "pc863" | "cp863" => CodePage::Pc863,
            "pc865" | "cp865" => CodePage::Pc865,
            "wpc1252" | "cp1252" | "windows-1252" => CodePage::Wpc1252,
            "pc866" | "cp866" => CodePage::Pc866,
            "pc852" | "cp852" => CodePage::Pc852,
            "pc858" | "cp858" => CodePage::Pc858,
            "pc720" | "cp720" => CodePage::Pc720,
            "pc864" | "cp864" => CodePage::Pc864,
            "pc737" | "cp737" => CodePage::Pc737,
            "wpc1250" | "cp1250" => CodePage::Wpc1250,
            "wpc1251" | "cp1251" => CodePage::Wpc1251,
            "wpc1253" | "cp1253" => CodePage::Wpc1253,
            "wpc1254" | "cp1254" => CodePage::Wpc1254,
            "wpc1255" | "cp1255" => CodePage::Wpc1255,
            "wpc1257" | "cp1257" => CodePage::Wpc1257,
            "wpc1258" | "cp1258" => CodePage::Wpc1258,
            "pc857" | "cp857" => CodePage::Pc857,
            "iso8859_15" | "iso8859-15" => CodePage::Iso8859_15,
            "pc874" | "cp874" => CodePage::Pc874,
            other => {
                if let Ok(n) = other.parse::<u8>() {
                    CodePage::Custom(n)
                } else {
                    CodePage::Pc437
                }
            }
        },
        _ => CodePage::Pc437,
    }
}

fn parse_mobile_columns(cols: &[MobileTableColumn]) -> Vec<TableColumn> {
    cols.iter()
        .map(|c| {
            let width = if let Some(f) = c.width_fixed.or(c.width) {
                ColumnWidth::Fixed(f)
            } else if let Some(frac) = c.width_fraction {
                ColumnWidth::Fraction(frac)
            } else {
                ColumnWidth::Fraction(1.0)
            };
            let align_str = c
                .align
                .as_deref()
                .or(c.alignment.as_deref())
                .unwrap_or("left");
            TableColumn::new(width, parse_alignment(align_str))
        })
        .collect()
}

fn build_receipt_from_json(json_str: &str) -> Option<Receipt> {
    let ticket: MobileTicketPayload = serde_json::from_str(json_str).ok()?;

    let width = match ticket.paper_width.as_deref() {
        Some("58mm" | "58" | "Mm58") => PaperWidth::Mm58,
        _ => PaperWidth::Mm80,
    };

    let mut r = Receipt::new(width).init();

    if let Some(cp_val) = &ticket.code_page {
        r = r.code_page(parse_code_page_value(cp_val));
    }

    if let Some(commands) = ticket.commands {
        for cmd in commands {
            match cmd {
                MobileCommand::Text {
                    text,
                    align,
                    bold,
                    underline,
                    invert,
                    double_width,
                    double_height,
                    double_size,
                } => {
                    if let Some(a) = align {
                        r = r.align(parse_alignment(&a));
                    }
                    if let Some(b) = bold {
                        r = r.bold(b);
                    }
                    if let Some(u) = underline {
                        r = r.underline(if u {
                            UnderlineMode::Single
                        } else {
                            UnderlineMode::Off
                        });
                    }
                    if let Some(i) = invert {
                        r = r.invert(i);
                    }
                    if let Some(dw) = double_width {
                        r = r.double_width(dw);
                    }
                    if let Some(dh) = double_height {
                        r = r.double_height(dh);
                    }
                    if let Some(ds) = double_size {
                        r = r.double_size(ds);
                    }
                    r = r.text(&text);
                }
                MobileCommand::TextLn {
                    text,
                    align,
                    bold,
                    underline,
                    invert,
                    double_width,
                    double_height,
                    double_size,
                } => {
                    if let Some(a) = align {
                        r = r.align(parse_alignment(&a));
                    }
                    if let Some(b) = bold {
                        r = r.bold(b);
                    }
                    if let Some(u) = underline {
                        r = r.underline(if u {
                            UnderlineMode::Single
                        } else {
                            UnderlineMode::Off
                        });
                    }
                    if let Some(i) = invert {
                        r = r.invert(i);
                    }
                    if let Some(dw) = double_width {
                        r = r.double_width(dw);
                    }
                    if let Some(dh) = double_height {
                        r = r.double_height(dh);
                    }
                    if let Some(ds) = double_size {
                        r = r.double_size(ds);
                    }
                    r = r.text_ln(&text);
                }
                MobileCommand::Feed { lines } => {
                    r = r.feed(lines.unwrap_or(1));
                }
                MobileCommand::Align { align } => {
                    r = r.align(parse_alignment(&align));
                }
                MobileCommand::Bold { enable } => {
                    r = r.bold(enable);
                }
                MobileCommand::Underline { enable, mode } => {
                    let u = if !enable {
                        UnderlineMode::Off
                    } else if mode.as_deref() == Some("double") {
                        UnderlineMode::Double
                    } else {
                        UnderlineMode::Single
                    };
                    r = r.underline(u);
                }
                MobileCommand::Invert { enable } => {
                    r = r.invert(enable);
                }
                MobileCommand::DoubleSize { enable } => {
                    r = r.double_size(enable);
                }
                MobileCommand::DoubleWidth { enable } => {
                    r = r.double_width(enable);
                }
                MobileCommand::DoubleHeight { enable } => {
                    r = r.double_height(enable);
                }
                MobileCommand::Divider {
                    style,
                    variant,
                    pattern,
                } => {
                    if let Some(pat) = pattern {
                        r = r.divider_pattern(&pat);
                    } else {
                        match variant.as_deref() {
                            Some("double") => {
                                r = r.divider_double();
                            }
                            Some("dotted") => {
                                r = r.divider_dotted();
                            }
                            Some("dashed") => {
                                r = r.divider_dashed();
                            }
                            _ => {
                                let ch = style
                                    .as_deref()
                                    .and_then(|s| s.chars().next())
                                    .unwrap_or('-');
                                r = r.divider(ch);
                            }
                        }
                    }
                }
                MobileCommand::DividerDouble => {
                    r = r.divider_double();
                }
                MobileCommand::DividerDotted => {
                    r = r.divider_dotted();
                }
                MobileCommand::DividerDashed => {
                    r = r.divider_dashed();
                }
                MobileCommand::TwoColumn { left, right } => {
                    r = r.two_column(&left, &right);
                }
                MobileCommand::ThreeColumn {
                    left,
                    center,
                    right,
                } => {
                    r = r.three_column(&left, &center, &right);
                }
                MobileCommand::TableHeader { headers, columns } => {
                    let cols = parse_mobile_columns(&columns);
                    if headers.is_empty() {
                        r = r.set_columns(&cols);
                    } else {
                        let header_refs: Vec<&str> = headers.iter().map(String::as_str).collect();
                        r = r.table_header(&header_refs, &cols);
                    }
                }
                MobileCommand::SetColumns { columns } => {
                    let cols = parse_mobile_columns(&columns);
                    r = r.set_columns(&cols);
                }
                MobileCommand::Row { cells } => {
                    let cell_refs: Vec<&str> = cells.iter().map(String::as_str).collect();
                    r = r.row(&cell_refs);
                }
                MobileCommand::ClearColumns => {
                    r = r.clear_columns();
                }
                MobileCommand::Qr { content } => {
                    r = r.qr(&content);
                }
                MobileCommand::Barcode { content } => {
                    r = r.barcode_128(&content);
                }
                MobileCommand::CodePage { page } => {
                    r = r.code_page(parse_code_page_value(&page));
                }
                MobileCommand::Image { data, max_width } => {
                    let clean_b64 = if let Some(pos) = data.find(";base64,") {
                        &data[pos + 8..]
                    } else {
                        &data
                    };
                    if let Some(raw_bytes) = base64_decode(clean_b64) {
                        let w = max_width.unwrap_or(width.dots());
                        if let Ok(new_r) = r.clone().image_from_bytes_with_options(
                            &raw_bytes,
                            Some(w),
                            papermint::DitherMode::FloydSteinberg,
                        ) {
                            r = new_r;
                        }
                    }
                }
                MobileCommand::Raw { bytes, base64 } => {
                    if let Some(b) = bytes {
                        r = r.raw(b);
                    } else if let Some(decoded) = base64.as_deref().and_then(base64_decode) {
                        r = r.raw(decoded);
                    }
                }
                MobileCommand::OpenDrawer => {
                    r = r.open_drawer();
                }
                MobileCommand::Beep { count, duration } => {
                    r = r.beep(count.unwrap_or(1), duration.unwrap_or(2));
                }
                MobileCommand::Cut { mode } => match mode.as_deref() {
                    Some("none") => {}
                    Some("partial") => {
                        r = r.cut(CutMode::Partial);
                    }
                    _ => {
                        r = r.cut(CutMode::Full);
                    }
                },
            }
        }
    }

    // Process high-level opinionated fields if provided
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

    if let Some(meta) = &ticket.metadata {
        for m in meta {
            r = r.two_column(&m.label, &m.value);
        }
        if !meta.is_empty() {
            r = r.divider(div_char);
        }
    }

    if let Some(items) = &ticket.items {
        let item_cols = [
            TableColumn::fixed(4, Alignment::Left),
            TableColumn::fraction(0.65, Alignment::Left),
            TableColumn::fraction(0.25, Alignment::Right),
        ];
        r = r.set_columns(&item_cols);
        r = r.bold(true).row(&["QTY", "ITEM", "TOTAL"]).bold(false);
        r = r.divider('-');

        for item in items {
            let qty = item.qty.as_deref().unwrap_or("1");
            r = r.row(&[qty, &item.description, &item.total]);
        }
        r = r.clear_columns();
        r = r.divider(div_char);
    }

    if let Some(totals) = &ticket.totals {
        for t in totals {
            if t.label.eq_ignore_ascii_case("total") || t.label.eq_ignore_ascii_case("grand total")
            {
                r = r.bold(true).two_column(&t.label, &t.value).bold(false);
            } else {
                r = r.two_column(&t.label, &t.value);
            }
        }
        r = r.divider(div_char);
    }

    if let Some(qr_url) = &ticket.qr {
        r = r.center().qr(qr_url).feed(1);
    }

    if let Some(bc) = &ticket.barcode {
        r = r.center().barcode_128(bc).feed(1);
    }

    if let Some(footer) = &ticket.footer {
        r = r.center().text_ln(footer);
    }

    if ticket.open_drawer.unwrap_or(false) {
        r = r.open_drawer();
    }

    if let Some(b) = ticket.beep.filter(|&b| b > 0) {
        r = r.beep(b, 2);
    }

    if let Some(cut_mode) = &ticket.cut_mode {
        match cut_mode.to_ascii_lowercase().as_str() {
            "partial" => r = r.feed(3).cut_partial(),
            "none" => {}
            _ => r = r.feed(3).cut_full(),
        }
    }

    Some(r)
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

    let r = match build_receipt_from_json(c_str) {
        Some(r) => r,
        None => return std::ptr::null_mut(),
    };

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

/// Renders a JSON ticket payload into a virtual SVG vector graphic string.
///
/// Returns a pointer to a heap-allocated C string. Must be freed with [`papermint_string_free`].
#[unsafe(no_mangle)]
pub extern "C" fn papermint_compile_json_svg(json_str: *const c_char) -> *mut c_char {
    if json_str.is_null() {
        return std::ptr::null_mut();
    }
    let c_str = match unsafe { CStr::from_ptr(json_str) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let r = match build_receipt_from_json(c_str) {
        Some(r) => r,
        None => return std::ptr::null_mut(),
    };
    let svg = r.render_svg();
    match CString::new(svg) {
        Ok(cs) => cs.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Renders a JSON ticket payload into a responsive HTML preview string.
///
/// Returns a pointer to a heap-allocated C string. Must be freed with [`papermint_string_free`].
#[unsafe(no_mangle)]
pub extern "C" fn papermint_compile_json_html(json_str: *const c_char) -> *mut c_char {
    if json_str.is_null() {
        return std::ptr::null_mut();
    }
    let c_str = match unsafe { CStr::from_ptr(json_str) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let r = match build_receipt_from_json(c_str) {
        Some(r) => r,
        None => return std::ptr::null_mut(),
    };
    let html = r.render_html();
    match CString::new(html) {
        Ok(cs) => cs.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Frees a string previously returned by [`papermint_compile_json_svg`] or [`papermint_compile_json_html`].
#[unsafe(no_mangle)]
pub extern "C" fn papermint_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}

fn text_is_empty(ptr: *const c_char) -> bool {
    if ptr.is_null() {
        true
    } else {
        unsafe { *ptr == 0 }
    }
}

// Android JNI bindings (Expo & React Native)

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jbyteArray, jint, jstring};

/// JNI bridge for `expo.modules.papermint.ExpoPapermintModule.nativeCompileTicket`.
///
/// Takes a JSON receipt string and dialect integer (0 = ESC/POS, 1 = StarPRNT),
/// compiles it via [`papermint_compile_json`], and returns a `jbyteArray`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_expo_modules_papermint_ExpoPapermintModule_nativeCompileTicket(
    mut env: JNIEnv,
    _class: JClass,
    json: JString,
    dialect: jint,
) -> jbyteArray {
    let json_str: String = match env.get_string(&json) {
        Ok(s) => s.into(),
        Err(_) => return std::ptr::null_mut(),
    };

    let c_json = match std::ffi::CString::new(json_str) {
        Ok(c) => c,
        Err(_) => return std::ptr::null_mut(),
    };

    let mut out_len: usize = 0;
    let ptr = papermint_compile_json(c_json.as_ptr(), dialect as u8, &mut out_len);

    if ptr.is_null() || out_len == 0 {
        return std::ptr::null_mut();
    }

    let slice = unsafe { std::slice::from_raw_parts(ptr, out_len) };
    let byte_array = match env.byte_array_from_slice(slice) {
        Ok(ba) => ba,
        Err(_) => {
            papermint_bytes_free(ptr, out_len);
            return std::ptr::null_mut();
        }
    };

    papermint_bytes_free(ptr, out_len);
    byte_array.into_raw()
}

/// JNI bridge for `expo.modules.papermint.ExpoPapermintModule.nativeRenderSvg`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_expo_modules_papermint_ExpoPapermintModule_nativeRenderSvg(
    mut env: JNIEnv,
    _class: JClass,
    json: JString,
) -> jstring {
    let json_str: String = match env.get_string(&json) {
        Ok(s) => s.into(),
        Err(_) => return std::ptr::null_mut(),
    };

    let r = match build_receipt_from_json(&json_str) {
        Some(r) => r,
        None => return std::ptr::null_mut(),
    };

    match env.new_string(r.render_svg()) {
        Ok(js) => js.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// JNI bridge for `expo.modules.papermint.ExpoPapermintModule.nativeRenderHtml`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_expo_modules_papermint_ExpoPapermintModule_nativeRenderHtml(
    mut env: JNIEnv,
    _class: JClass,
    json: JString,
) -> jstring {
    let json_str: String = match env.get_string(&json) {
        Ok(s) => s.into(),
        Err(_) => return std::ptr::null_mut(),
    };

    let r = match build_receipt_from_json(&json_str) {
        Some(r) => r,
        None => return std::ptr::null_mut(),
    };

    match env.new_string(r.render_html()) {
        Ok(js) => js.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_legacy_json() {
        let json = r#"{
            "paper_width": "80mm",
            "title": "COFFEE SHOP",
            "items": [
                { "qty": "2x", "description": "Latte", "total": "$9.00" }
            ],
            "totals": [
                { "label": "TOTAL", "value": "$9.00" }
            ],
            "cut_mode": "full"
        }"#;

        let c_json = CString::new(json).unwrap();
        let mut out_len = 0;
        let ptr = papermint_compile_json(c_json.as_ptr(), 0, &mut out_len);

        assert!(!ptr.is_null());
        assert!(out_len > 0);

        papermint_bytes_free(ptr, out_len);
    }

    #[test]
    fn test_compile_command_ir_json() {
        let json = r#"{
            "paper_width": "80mm",
            "code_page": "wpc1256",
            "commands": [
                { "type": "align", "align": "center" },
                { "type": "bold", "enable": true },
                { "type": "double_size", "enable": true },
                { "type": "text_ln", "text": "MINT BISTRO" },
                { "type": "bold", "enable": false },
                { "type": "double_size", "enable": false },
                { "type": "divider", "variant": "double" },
                { "type": "table_header", "headers": ["QTY", "ITEM", "TOTAL"], "columns": [
                    { "width_fixed": 4, "align": "left" },
                    { "width_fraction": 0.6, "align": "left" },
                    { "width_fraction": 0.3, "align": "right" }
                ]},
                { "type": "row", "cells": ["1x", "Avocado Toast", "$12.00"] },
                { "type": "clear_columns" },
                { "type": "divider", "variant": "dashed" },
                { "type": "two_column", "left": "TOTAL DUE:", "right": "$12.00" },
                { "type": "qr", "content": "https://example.com/order/1" },
                { "type": "cut", "mode": "partial" }
            ]
        }"#;

        let c_json = CString::new(json).unwrap();
        let mut out_len = 0;
        let ptr = papermint_compile_json(c_json.as_ptr(), 0, &mut out_len);

        assert!(!ptr.is_null());
        assert!(out_len > 0);
        papermint_bytes_free(ptr, out_len);

        // Test SVG rendering
        let svg_ptr = papermint_compile_json_svg(c_json.as_ptr());
        assert!(!svg_ptr.is_null());
        let svg_str = unsafe { CStr::from_ptr(svg_ptr) }.to_str().unwrap();
        assert!(svg_str.contains("<svg"));
        assert!(svg_str.contains("MINT BISTRO"));
        papermint_string_free(svg_ptr);

        // Test HTML rendering
        let html_ptr = papermint_compile_json_html(c_json.as_ptr());
        assert!(!html_ptr.is_null());
        let html_str = unsafe { CStr::from_ptr(html_ptr) }.to_str().unwrap();
        assert!(html_str.contains("<div"));
        assert!(html_str.contains("MINT BISTRO"));
        papermint_string_free(html_ptr);
    }
}
