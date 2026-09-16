//! Memory management primitives for cross-language FFI boundary.

use std::ffi::{CString, c_char};

/// Safely deallocates a byte buffer previously returned by any Papermint FFI encoder.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_bytes_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        unsafe {
            drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len)));
        }
    }
}

/// Frees a string previously returned by SVG or HTML preview renderers.
#[unsafe(no_mangle)]
pub extern "C" fn papermint_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}

pub(crate) fn text_is_empty(ptr: *const c_char) -> bool {
    if ptr.is_null() {
        true
    } else {
        unsafe { *ptr == 0 }
    }
}
