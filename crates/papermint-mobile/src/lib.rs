//! papermint-mobile — Mobile C-ABI and zero-copy byte compiler.
//!
//! Provides a high-performance C-compatible foreign function interface (FFI)
//! designed for React Native (C++ JSI / TurboModules), Expo SDK 56+ (Inline Modules),
//! Flutter (dart:ffi), iOS Swift, and Android Kotlin.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod jni;
pub mod json_label;
pub mod json_receipt;
pub mod label_ffi;
pub mod memory;
pub mod receipt_ffi;

pub use jni::*;
pub use json_label::*;
pub use json_receipt::*;
pub use label_ffi::*;
pub use memory::*;
pub use receipt_ffi::*;

#[cfg(test)]
mod tests {
    use std::ffi::{CStr, CString};

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

    #[test]
    fn test_mobile_label_c_abi() {
        let handle = papermint_label_create(50.0, 30.0, 203);
        assert!(!handle.is_null());

        let text = CString::new("ICED LATTE").unwrap();
        papermint_label_text(handle, 16, 16, text.as_ptr(), 1, 1);

        let code = CString::new("ORD-5001").unwrap();
        papermint_label_barcode(handle, 16, 60, 0, 48, code.as_ptr(), true);

        let qr_data = CString::new("https://example.com").unwrap();
        papermint_label_qr(handle, 200, 60, qr_data.as_ptr(), 4, 0);

        papermint_label_print(handle, 1);

        let mut tspl_len = 0;
        let tspl_ptr = papermint_label_encode_tspl(handle, &mut tspl_len);
        assert!(!tspl_ptr.is_null());
        assert!(tspl_len > 0);
        let tspl_slice = unsafe { std::slice::from_raw_parts(tspl_ptr, tspl_len) };
        let tspl_str = String::from_utf8_lossy(tspl_slice);
        assert!(tspl_str.contains("SIZE 50.00 mm, 30.00 mm"));
        assert!(tspl_str.contains("ICED LATTE"));
        papermint_bytes_free(tspl_ptr, tspl_len);

        let mut zpl_len = 0;
        let zpl_ptr = papermint_label_encode_zpl(handle, &mut zpl_len);
        assert!(!zpl_ptr.is_null());
        assert!(zpl_len > 0);
        let zpl_slice = unsafe { std::slice::from_raw_parts(zpl_ptr, zpl_len) };
        let zpl_str = String::from_utf8_lossy(zpl_slice);
        assert!(zpl_str.contains("^XA"));
        assert!(zpl_str.contains("ICED LATTE"));
        papermint_bytes_free(zpl_ptr, zpl_len);

        let svg_ptr = papermint_label_render_svg(handle);
        assert!(!svg_ptr.is_null());
        let svg_str = unsafe { CStr::from_ptr(svg_ptr) }.to_str().unwrap();
        assert!(svg_str.contains("<svg"));
        assert!(svg_str.contains("ICED LATTE"));
        papermint_string_free(svg_ptr);

        papermint_label_free(handle);
    }

    #[test]
    fn test_mobile_label_json_compiler() {
        let json = r#"{
            "width_mm": 50.0,
            "height_mm": 30.0,
            "dpi": 203,
            "elements": [
                { "type": "text", "x": 16, "y": 16, "content": "MATCHA LATTE" },
                { "type": "barcode", "x": 16, "y": 60, "format": "code128", "height": 48, "content": "MTC-99" },
                { "type": "qr", "x": 220, "y": 60, "content": "https://mint.com" },
                { "type": "reverse", "x": 250, "y": 16, "width": 60, "height": 30 }
            ],
            "print_copies": 1
        }"#;

        let c_json = CString::new(json).unwrap();
        let mut tspl_len = 0;
        let tspl_ptr = papermint_label_compile_json(c_json.as_ptr(), 0, &mut tspl_len);
        assert!(!tspl_ptr.is_null());
        assert!(tspl_len > 0);
        papermint_bytes_free(tspl_ptr, tspl_len);

        let mut zpl_len = 0;
        let zpl_ptr = papermint_label_compile_json(c_json.as_ptr(), 1, &mut zpl_len);
        assert!(!zpl_ptr.is_null());
        assert!(zpl_len > 0);
        papermint_bytes_free(zpl_ptr, zpl_len);

        let svg_ptr = papermint_label_render_svg_json(c_json.as_ptr());
        assert!(!svg_ptr.is_null());
        let svg_str = unsafe { CStr::from_ptr(svg_ptr) }.to_str().unwrap();
        assert!(svg_str.contains("<svg"));
        assert!(svg_str.contains("MATCHA LATTE"));
        papermint_string_free(svg_ptr);
    }
}
