//! Android JNI bindings for React Native and Expo Modules.

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jbyteArray, jint, jstring};

use crate::json_label::{build_label_from_json, papermint_label_compile_json};
use crate::json_receipt::{build_receipt_from_json, papermint_compile_json};
use crate::memory::papermint_bytes_free;

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

/// JNI bridge for `expo.modules.papermint.ExpoPapermintModule.nativeCompileLabel`.
///
/// Takes a JSON label string and dialect integer (0 = TSPL, 1 = ZPL II),
/// compiles it via [`papermint_label_compile_json`], and returns a `jbyteArray`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_expo_modules_papermint_ExpoPapermintModule_nativeCompileLabel(
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
    let ptr = papermint_label_compile_json(c_json.as_ptr(), dialect as u8, &mut out_len);

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

/// JNI bridge for `expo.modules.papermint.ExpoPapermintModule.nativeRenderLabelSvg`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_expo_modules_papermint_ExpoPapermintModule_nativeRenderLabelSvg(
    mut env: JNIEnv,
    _class: JClass,
    json: JString,
) -> jstring {
    let json_str: String = match env.get_string(&json) {
        Ok(s) => s.into(),
        Err(_) => return std::ptr::null_mut(),
    };

    let label = match build_label_from_json(&json_str) {
        Some(l) => l,
        None => return std::ptr::null_mut(),
    };

    match env.new_string(label.render_svg()) {
        Ok(js) => js.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}
