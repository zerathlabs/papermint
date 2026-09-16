//! Mobile JSON IR compiler and preview renderers for 2D adhesive labels.

use std::ffi::{CStr, CString, c_char};

use serde::Deserialize;

use papermint_label::{BarcodeType, Label, QrErrorCorrection, Rotation};

const fn default_true() -> bool {
    true
}
const fn default_mult() -> u32 {
    1
}
const fn default_cell_width() -> u32 {
    4
}
const fn default_thickness() -> u32 {
    2
}

#[derive(Debug, Deserialize)]
struct MobileLabelPayload {
    pub width_mm: Option<f32>,
    pub height_mm: Option<f32>,
    pub dpi: Option<u32>,
    pub gap_mm: Option<f32>,
    pub speed: Option<u32>,
    pub density: Option<u32>,
    pub elements: Option<Vec<MobileLabelElement>>,
    pub print_copies: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum MobileLabelElement {
    Text {
        x: u32,
        y: u32,
        content: String,
        #[serde(default = "default_mult")]
        x_mult: u32,
        #[serde(default = "default_mult")]
        y_mult: u32,
    },
    Barcode {
        x: u32,
        y: u32,
        format: Option<String>,
        height: Option<u32>,
        content: String,
        #[serde(default = "default_true")]
        readable: bool,
    },
    Qr {
        x: u32,
        y: u32,
        content: String,
        #[serde(default = "default_cell_width")]
        cell_width: u32,
        ecc: Option<String>,
    },
    Box {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        #[serde(default = "default_thickness")]
        thickness: u32,
    },
    Line {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    Reverse {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
}

pub(crate) fn build_label_from_json(json_str: &str) -> Option<Label> {
    let payload: MobileLabelPayload = serde_json::from_str(json_str).ok()?;

    let width = payload.width_mm.unwrap_or(50.0);
    let height = payload.height_mm.unwrap_or(30.0);
    let dpi = payload.dpi.unwrap_or(203);

    let mut label = Label::new(width, height).dpi(dpi);

    if let Some(gap) = payload.gap_mm {
        label = label.gap(gap, 0.0);
    }
    if let Some(speed) = payload.speed {
        label = label.speed(speed as u8);
    }
    if let Some(density) = payload.density {
        label = label.density(density as u8);
    }

    if let Some(elements) = payload.elements {
        for el in elements {
            match el {
                MobileLabelElement::Text {
                    x,
                    y,
                    content,
                    x_mult,
                    y_mult,
                } => {
                    label = label.text_ext(
                        x,
                        y,
                        "2",
                        Rotation::Deg0,
                        x_mult as u8,
                        y_mult as u8,
                        content,
                    );
                }
                MobileLabelElement::Barcode {
                    x,
                    y,
                    format,
                    height,
                    content,
                    readable,
                } => {
                    let btype = match format.as_deref() {
                        Some("code39") => BarcodeType::Code39,
                        Some("ean13") => BarcodeType::Ean13,
                        Some("ean8") => BarcodeType::Ean8,
                        Some("upca") => BarcodeType::UpcA,
                        Some("itf") => BarcodeType::Itf,
                        _ => BarcodeType::Code128,
                    };
                    label = label.barcode_ext(
                        x,
                        y,
                        btype,
                        height.unwrap_or(48),
                        readable,
                        Rotation::Deg0,
                        2,
                        4,
                        content,
                    );
                }
                MobileLabelElement::Qr {
                    x,
                    y,
                    content,
                    cell_width,
                    ecc,
                } => {
                    let ec = match ecc.as_deref() {
                        Some("L" | "l") => QrErrorCorrection::L,
                        Some("Q" | "q") => QrErrorCorrection::Q,
                        Some("H" | "h") => QrErrorCorrection::H,
                        _ => QrErrorCorrection::M,
                    };
                    label = label.qr_ext(x, y, cell_width as u8, ec, Rotation::Deg0, content);
                }
                MobileLabelElement::Box {
                    x,
                    y,
                    width,
                    height,
                    thickness,
                } => {
                    label = label.box_outline(x, y, width, height, thickness);
                }
                MobileLabelElement::Line {
                    x,
                    y,
                    width,
                    height,
                } => {
                    label = label.line(x, y, width, height);
                }
                MobileLabelElement::Reverse {
                    x,
                    y,
                    width,
                    height,
                } => {
                    label = label.reverse(x, y, width, height);
                }
            }
        }
    }

    if let Some(copies) = payload.print_copies {
        label = label.copies(copies);
    }

    Some(label)
}

/// Compiles a JSON label representation into TSPL (0) or ZPL (1) bytes.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_label_compile_json(
    json_str: *const c_char,
    dialect: u8,
    out_len: *mut usize,
) -> *mut u8 {
    if json_str.is_null() || out_len.is_null() {
        return std::ptr::null_mut();
    }
    let s = match unsafe { CStr::from_ptr(json_str) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let label = match build_label_from_json(s) {
        Some(l) => l,
        None => return std::ptr::null_mut(),
    };

    let bytes = if dialect == 1 {
        label.encode_zpl()
    } else {
        label.encode_tspl()
    };

    let mut boxed = bytes.into_boxed_slice();
    let len = boxed.len();
    let ptr = boxed.as_mut_ptr();
    std::mem::forget(boxed);
    unsafe {
        *out_len = len;
    }
    ptr
}

/// Renders a JSON label representation into an SVG sticker preview string.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_label_render_svg_json(json_str: *const c_char) -> *mut c_char {
    if json_str.is_null() {
        return std::ptr::null_mut();
    }
    let s = match unsafe { CStr::from_ptr(json_str) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let label = match build_label_from_json(s) {
        Some(l) => l,
        None => return std::ptr::null_mut(),
    };

    let svg = label.render_svg();
    CString::new(svg)
        .map(|c| c.into_raw())
        .unwrap_or(std::ptr::null_mut())
}
