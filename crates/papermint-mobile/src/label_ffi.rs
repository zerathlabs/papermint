//! Fluent C-ABI handle functions for 2D label printing (TSPL & ZPL).

use std::ffi::{CStr, CString, c_char};

use papermint_label::{BarcodeType, Direction, Label, QrErrorCorrection, Rotation};

/// Opaque wrapper around [`papermint_label::Label`].
pub struct PapermintLabel {
    pub(crate) inner: Label,
}

/// Creates a new label canvas instance.
///
/// * `width_mm`: Label physical width in millimeters (e.g. 50.0).
/// * `height_mm`: Label physical height in millimeters (e.g. 30.0).
/// * `dpi`: Printhead resolution in dots per inch (default 203 if 0).
#[unsafe(no_mangle)]
pub extern "C" fn papermint_label_create(
    width_mm: f32,
    height_mm: f32,
    dpi: u32,
) -> *mut PapermintLabel {
    let d = if dpi == 0 { 203 } else { dpi };
    let label = Box::new(PapermintLabel {
        inner: Label::new(width_mm, height_mm).dpi(d),
    });
    Box::into_raw(label)
}

/// Frees a label canvas instance allocated by [`papermint_label_create`].
#[unsafe(no_mangle)]
pub extern "C" fn papermint_label_free(handle: *mut PapermintLabel) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

/// Sets media gap between labels in millimeters.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_label_gap(handle: *mut PapermintLabel, gap_mm: f32, offset_mm: f32) {
    if let Some(l) = unsafe { handle.as_mut() } {
        l.inner = std::mem::take(&mut l.inner).gap(gap_mm, offset_mm);
    }
}

/// Sets print speed in inches per second (e.g. 2 to 6).
#[unsafe(no_mangle)]
pub extern "C" fn papermint_label_speed(handle: *mut PapermintLabel, speed: u32) {
    if let Some(l) = unsafe { handle.as_mut() } {
        l.inner = std::mem::take(&mut l.inner).speed(speed as u8);
    }
}

/// Sets print density / darkness (0 to 15).
#[unsafe(no_mangle)]
pub extern "C" fn papermint_label_density(handle: *mut PapermintLabel, density: u32) {
    if let Some(l) = unsafe { handle.as_mut() } {
        l.inner = std::mem::take(&mut l.inner).density(density as u8);
    }
}

/// Sets print feed direction (0 = Forward, 1 = Backward).
#[unsafe(no_mangle)]
pub extern "C" fn papermint_label_direction(handle: *mut PapermintLabel, direction: u8) {
    if let Some(l) = unsafe { handle.as_mut() } {
        let dir = if direction == 1 {
            Direction::Inverted
        } else {
            Direction::Normal
        };
        l.inner = std::mem::take(&mut l.inner).direction(dir);
    }
}

/// Appends text at absolute (x, y) coordinates with optional font multipliers.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_label_text(
    handle: *mut PapermintLabel,
    x: u32,
    y: u32,
    text: *const c_char,
    x_mult: u32,
    y_mult: u32,
) {
    if text.is_null() {
        return;
    }
    if let (Some(l), Ok(s)) = (
        unsafe { handle.as_mut() },
        unsafe { CStr::from_ptr(text) }.to_str(),
    ) {
        l.inner = std::mem::take(&mut l.inner).text_ext(
            x,
            y,
            "2",
            Rotation::Deg0,
            x_mult as u8,
            y_mult as u8,
            s,
        );
    }
}

/// Appends a 1D barcode at (x, y).
///
/// * `code_type`: 0 = Code128, 1 = Code39, 2 = Ean13, 3 = Ean8, 4 = UpcA, 5 = Itf.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_label_barcode(
    handle: *mut PapermintLabel,
    x: u32,
    y: u32,
    code_type: u8,
    height: u32,
    content: *const c_char,
    readable: bool,
) {
    if content.is_null() {
        return;
    }
    if let (Some(l), Ok(s)) = (
        unsafe { handle.as_mut() },
        unsafe { CStr::from_ptr(content) }.to_str(),
    ) {
        let btype = match code_type {
            1 => BarcodeType::Code39,
            2 => BarcodeType::Ean13,
            3 => BarcodeType::Ean8,
            4 => BarcodeType::UpcA,
            5 => BarcodeType::Itf,
            _ => BarcodeType::Code128,
        };
        l.inner = std::mem::take(&mut l.inner).barcode_ext(
            x,
            y,
            btype,
            height,
            readable,
            Rotation::Deg0,
            2,
            4,
            s,
        );
    }
}

/// Appends a 2D QR Code at (x, y).
///
/// * `ecc`: 0 = M (15%), 1 = L (7%), 2 = Q (25%), 3 = H (30%).
#[unsafe(no_mangle)]
pub extern "C" fn papermint_label_qr(
    handle: *mut PapermintLabel,
    x: u32,
    y: u32,
    content: *const c_char,
    cell_width: u32,
    ecc: u8,
) {
    if content.is_null() {
        return;
    }
    if let (Some(l), Ok(s)) = (
        unsafe { handle.as_mut() },
        unsafe { CStr::from_ptr(content) }.to_str(),
    ) {
        let ec = match ecc {
            1 => QrErrorCorrection::L,
            2 => QrErrorCorrection::Q,
            3 => QrErrorCorrection::H,
            _ => QrErrorCorrection::M,
        };
        l.inner =
            std::mem::take(&mut l.inner).qr_ext(x, y, cell_width as u8, ec, Rotation::Deg0, s);
    }
}

/// Appends a rectangular outline box at (x, y) with specified dimensions and thickness.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_label_box(
    handle: *mut PapermintLabel,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    thickness: u32,
) {
    if let Some(l) = unsafe { handle.as_mut() } {
        l.inner = std::mem::take(&mut l.inner).box_outline(x, y, w, h, thickness);
    }
}

/// Appends a solid separator bar/line at (x, y).
#[unsafe(no_mangle)]
pub extern "C" fn papermint_label_line(
    handle: *mut PapermintLabel,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) {
    if let Some(l) = unsafe { handle.as_mut() } {
        l.inner = std::mem::take(&mut l.inner).line(x, y, w, h);
    }
}

/// Inverts white/black pixels within a rectangular area (highlight badge).
#[unsafe(no_mangle)]
pub extern "C" fn papermint_label_reverse(
    handle: *mut PapermintLabel,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) {
    if let Some(l) = unsafe { handle.as_mut() } {
        l.inner = std::mem::take(&mut l.inner).reverse(x, y, w, h);
    }
}

/// Sets number of print copies.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_label_print(handle: *mut PapermintLabel, copies: u32) {
    if let Some(l) = unsafe { handle.as_mut() } {
        l.inner = std::mem::take(&mut l.inner).copies(copies);
    }
}

/// Encodes the label canvas into TSPL-II wire bytes (TSC, Xprinter, Rongta, Munbyn).
#[unsafe(no_mangle)]
pub extern "C" fn papermint_label_encode_tspl(
    handle: *mut PapermintLabel,
    out_len: *mut usize,
) -> *mut u8 {
    if handle.is_null() || out_len.is_null() {
        return std::ptr::null_mut();
    }
    let l = unsafe { &*handle };
    let bytes = l.inner.encode_tspl();
    let mut boxed = bytes.into_boxed_slice();
    let len = boxed.len();
    let ptr = boxed.as_mut_ptr();
    std::mem::forget(boxed);
    unsafe {
        *out_len = len;
    }
    ptr
}

/// Encodes the label canvas into ZPL II wire bytes (Zebra, Citizen, Godex).
#[unsafe(no_mangle)]
pub extern "C" fn papermint_label_encode_zpl(
    handle: *mut PapermintLabel,
    out_len: *mut usize,
) -> *mut u8 {
    if handle.is_null() || out_len.is_null() {
        return std::ptr::null_mut();
    }
    let l = unsafe { &*handle };
    let bytes = l.inner.encode_zpl();
    let mut boxed = bytes.into_boxed_slice();
    let len = boxed.len();
    let ptr = boxed.as_mut_ptr();
    std::mem::forget(boxed);
    unsafe {
        *out_len = len;
    }
    ptr
}

/// Renders a virtual SVG sticker preview.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_label_render_svg(handle: *mut PapermintLabel) -> *mut c_char {
    if handle.is_null() {
        return std::ptr::null_mut();
    }
    let l = unsafe { &*handle };
    let svg = l.inner.render_svg();
    CString::new(svg)
        .map(|c| c.into_raw())
        .unwrap_or(std::ptr::null_mut())
}
