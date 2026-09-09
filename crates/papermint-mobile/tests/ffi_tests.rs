use std::ffi::{CString, c_char};

use papermint_mobile::{
    papermint_bytes_free, papermint_compile_json, papermint_receipt_align,
    papermint_receipt_barcode, papermint_receipt_beep, papermint_receipt_bold,
    papermint_receipt_create, papermint_receipt_cut, papermint_receipt_divider,
    papermint_receipt_encode, papermint_receipt_feed, papermint_receipt_free,
    papermint_receipt_init, papermint_receipt_open_drawer, papermint_receipt_qr,
    papermint_receipt_table_row, papermint_receipt_text, papermint_receipt_text_ln,
    papermint_receipt_underline,
};

#[test]
fn test_fluent_c_abi_escpos() {
    let receipt = papermint_receipt_create(0); // 80mm
    assert!(!receipt.is_null());

    papermint_receipt_init(receipt);
    papermint_receipt_align(receipt, 1); // Center
    papermint_receipt_bold(receipt, true);
    papermint_receipt_underline(receipt, 1); // Single underline

    let title = CString::new("RUST MOBILE CAFE").unwrap();
    papermint_receipt_text_ln(receipt, title.as_ptr());

    papermint_receipt_underline(receipt, 0); // Off
    papermint_receipt_bold(receipt, false);
    papermint_receipt_align(receipt, 0); // Left

    let prefix = CString::new("Order Note: ").unwrap();
    papermint_receipt_text(receipt, prefix.as_ptr());
    let note = CString::new("Extra hot\n").unwrap();
    papermint_receipt_text(receipt, note.as_ptr());

    let div = CString::new("=").unwrap();
    papermint_receipt_divider(receipt, div.as_ptr());

    let c1 = CString::new("Matcha Latte").unwrap();
    let c2 = CString::new("x1").unwrap();
    let c3 = CString::new("$4.50").unwrap();
    let cols: [*const c_char; 3] = [c1.as_ptr(), c2.as_ptr(), c3.as_ptr()];
    papermint_receipt_table_row(receipt, cols.as_ptr(), 3);

    let barcode_text = CString::new("ORDER1234").unwrap();
    papermint_receipt_barcode(receipt, barcode_text.as_ptr());

    let qr_url = CString::new("https://pay.example.com/order/100").unwrap();
    papermint_receipt_qr(receipt, qr_url.as_ptr());

    papermint_receipt_feed(receipt, 2);
    papermint_receipt_beep(receipt, 1, 2);
    papermint_receipt_open_drawer(receipt);
    papermint_receipt_cut(receipt, false); // Full cut

    let mut out_len: usize = 0;
    let bytes_ptr = papermint_receipt_encode(receipt, 0, &mut out_len); // ESC/POS
    assert!(!bytes_ptr.is_null());
    assert!(out_len > 0);

    let bytes = unsafe { std::slice::from_raw_parts(bytes_ptr, out_len) };
    // Check for standard ESC/POS bytes: ESC @ (1B 40)
    assert!(bytes.windows(2).any(|w| w == [0x1B, 0x40]));
    // Check for cut command: GS V
    assert!(bytes.windows(2).any(|w| w == [0x1D, 0x56]));

    papermint_bytes_free(bytes_ptr, out_len);
    papermint_receipt_free(receipt);
}

#[test]
fn test_fluent_c_abi_star() {
    let receipt = papermint_receipt_create(1); // 58mm
    assert!(!receipt.is_null());

    papermint_receipt_init(receipt);
    let title = CString::new("STAR MOBILE").unwrap();
    papermint_receipt_text_ln(receipt, title.as_ptr());
    papermint_receipt_cut(receipt, true); // Partial cut: ESC d 3 in StarPRNT

    let mut out_len: usize = 0;
    let bytes_ptr = papermint_receipt_encode(receipt, 1, &mut out_len); // StarPRNT
    assert!(!bytes_ptr.is_null());
    assert!(out_len > 0);

    let bytes = unsafe { std::slice::from_raw_parts(bytes_ptr, out_len) };
    // Check for StarPRNT init: ESC @ (1B 40)
    assert!(bytes.windows(2).any(|w| w == [0x1B, 0x40]));
    // Check for Star cut command: ESC d 3 (Feed and partial cut)
    assert!(bytes.windows(3).any(|w| w == [0x1B, 0x64, 0x03]));

    papermint_bytes_free(bytes_ptr, out_len);
    papermint_receipt_free(receipt);
}

#[test]
fn test_json_compiler_escpos_and_star() {
    let json = CString::new(
        r#"{
            "paper_width": "80mm",
            "title": "EXPO SPEED BURGER",
            "subtitle": "Order #5021",
            "address": "456 Market St, San Francisco",
            "metadata": [
                {"label": "Cashier", "value": "Alice"},
                {"label": "Table", "value": "12"}
            ],
            "items": [
                {"qty": "2", "description": "Smashburger Deluxe", "price": "$12.00", "total": "$24.00"},
                {"qty": "1", "description": "Truffle Fries", "price": "$6.50", "total": "$6.50"},
                {"qty": "2", "description": "Craft Soda", "price": "$3.50", "total": "$7.00"}
            ],
            "totals": [
                {"label": "Subtotal", "value": "$37.50"},
                {"label": "Tax (8.5%)", "value": "$3.19"},
                {"label": "TOTAL", "value": "$40.69"}
            ],
            "qr": "https://feedback.expoburger.com",
            "barcode": "50214069",
            "footer": "Thanks for supporting local food!",
            "cut_mode": "full",
            "open_drawer": true,
            "beep": 1
        }"#,
    )
    .unwrap();

    // 1. Test ESC/POS compilation
    let mut escpos_len: usize = 0;
    let escpos_ptr = papermint_compile_json(json.as_ptr(), 0, &mut escpos_len);
    assert!(!escpos_ptr.is_null());
    assert!(escpos_len > 200);

    let escpos_bytes = unsafe { std::slice::from_raw_parts(escpos_ptr, escpos_len) };
    let text = String::from_utf8_lossy(escpos_bytes);
    assert!(text.contains("EXPO SPEED BURGER"));
    assert!(text.contains("Smashburger Deluxe"));
    assert!(text.contains("$40.69"));

    papermint_bytes_free(escpos_ptr, escpos_len);

    // 2. Test StarPRNT compilation
    let mut star_len: usize = 0;
    let star_ptr = papermint_compile_json(json.as_ptr(), 1, &mut star_len);
    assert!(!star_ptr.is_null());
    assert!(star_len > 200);

    let star_bytes = unsafe { std::slice::from_raw_parts(star_ptr, star_len) };
    let star_text = String::from_utf8_lossy(star_bytes);
    assert!(star_text.contains("EXPO SPEED BURGER"));
    assert!(star_text.contains("Truffle Fries"));

    papermint_bytes_free(star_ptr, star_len);
}

#[test]
fn test_null_safety_and_error_handling() {
    // 1. Null handle encode
    let mut out_len: usize = 999;
    let ptr = papermint_receipt_encode(std::ptr::null(), 0, &mut out_len);
    assert!(ptr.is_null());
    assert_eq!(out_len, 0);

    // 2. Null JSON string
    let mut json_len: usize = 999;
    let json_ptr = papermint_compile_json(std::ptr::null(), 0, &mut json_len);
    assert!(json_ptr.is_null());
    assert_eq!(json_len, 0);

    // 3. Invalid JSON string
    let bad_json = CString::new("{ not valid json !!! }").unwrap();
    let bad_ptr = papermint_compile_json(bad_json.as_ptr(), 0, &mut json_len);
    assert!(bad_ptr.is_null());
    assert_eq!(json_len, 0);

    // 4. Null receipt free is safe no-op
    papermint_receipt_free(std::ptr::null_mut());

    // 5. Null bytes free is safe no-op
    papermint_bytes_free(std::ptr::null_mut(), 0);
}
