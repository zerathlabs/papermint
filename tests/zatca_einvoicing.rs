use papermint::tax::{
    ZatcaInvoice, base64_decode, base64_encode, decode_tlv, encode_zatca_tlv, zatca_qr_base64,
};
use papermint::{PaperWidth, Receipt};

#[test]
fn test_zatca_phase_1_tlv_packing() {
    let invoice = ZatcaInvoice::new(
        "Bob's Fashions",
        "310122393500003",
        "2022-04-25T15:30:00Z",
        "1000.00",
        "150.00",
    );

    let tlv = encode_zatca_tlv(&invoice);
    let entries = decode_tlv(&tlv);

    assert_eq!(entries.len(), 5);

    // Tag 1: Seller's name
    assert_eq!(entries[0].tag, 1);
    assert_eq!(
        std::str::from_utf8(&entries[0].value).unwrap(),
        "Bob's Fashions"
    );

    // Tag 2: VAT Registration Number
    assert_eq!(entries[1].tag, 2);
    assert_eq!(
        std::str::from_utf8(&entries[1].value).unwrap(),
        "310122393500003"
    );

    // Tag 3: Timestamp
    assert_eq!(entries[2].tag, 3);
    assert_eq!(
        std::str::from_utf8(&entries[2].value).unwrap(),
        "2022-04-25T15:30:00Z"
    );

    // Tag 4: Invoice Total
    assert_eq!(entries[3].tag, 4);
    assert_eq!(std::str::from_utf8(&entries[3].value).unwrap(), "1000.00");

    // Tag 5: Total VAT
    assert_eq!(entries[4].tag, 5);
    assert_eq!(std::str::from_utf8(&entries[4].value).unwrap(), "150.00");
}

#[test]
fn test_zatca_base64_qr_payload() {
    let invoice = ZatcaInvoice::new(
        "Bob's Fashions",
        "310122393500003",
        "2022-04-25T15:30:00Z",
        "1000.00",
        "150.00",
    );

    let b64 = zatca_qr_base64(&invoice);
    assert!(!b64.is_empty());

    let decoded = base64_decode(&b64).expect("Base64 decoding must succeed");
    let entries = decode_tlv(&decoded);
    assert_eq!(entries.len(), 5);
    assert_eq!(
        std::str::from_utf8(&entries[0].value).unwrap(),
        "Bob's Fashions"
    );
}

#[test]
fn test_zatca_phase_2_fields() {
    let hash = vec![0x12, 0x34, 0x56, 0x78];
    let signature = vec![0xAA, 0xBB, 0xCC, 0xDD];
    let public_key = vec![0x04, 0x11, 0x22, 0x33];

    let invoice = ZatcaInvoice::new(
        "Al-Madina Hypermarket",
        "310999999900003",
        "2026-09-10T11:00:00Z",
        "245.50",
        "32.02",
    )
    .with_invoice_hash(hash.clone())
    .with_signature(signature.clone())
    .with_public_key(public_key.clone());

    let tlv = invoice.to_tlv_bytes();
    let entries = decode_tlv(&tlv);

    assert_eq!(entries.len(), 8);
    assert_eq!(entries[5].tag, 6);
    assert_eq!(entries[5].value, hash);

    assert_eq!(entries[6].tag, 7);
    assert_eq!(entries[6].value, signature);

    assert_eq!(entries[7].tag, 8);
    assert_eq!(entries[7].value, public_key);
}

#[test]
fn test_receipt_zatca_qr_integration() {
    let invoice = ZatcaInvoice::new(
        "Dubai Bistro LLC",
        "100234567800003",
        "2026-09-10T18:00:00Z",
        "85.00",
        "4.25",
    );

    let receipt = Receipt::new(PaperWidth::Mm80)
        .center()
        .bold(true)
        .text_ln("DUBAI BISTRO")
        .bold(false)
        .divider('-')
        .two_column("Shawarma Platter", "AED 80.76")
        .two_column("VAT (5%)", "AED 4.24")
        .divider('=')
        .two_column("TOTAL", "AED 85.00")
        .feed(1)
        .zatca_qr(&invoice)
        .feed(2)
        .cut_full();

    // Verify commands contain QrCode with Base64 payload
    let qr_cmd = receipt
        .commands()
        .iter()
        .find(|cmd| matches!(cmd, papermint::Command::QrCode(_)));

    assert!(qr_cmd.is_some(), "Receipt must contain QR code command");
    if let Some(papermint::Command::QrCode(qr)) = qr_cmd {
        assert_eq!(qr.data, invoice.to_qr_base64());
    }
}

#[test]
fn test_base64_encode_known_vectors() {
    assert_eq!(base64_encode(b""), "");
    assert_eq!(base64_encode(b"a"), "YQ==");
    assert_eq!(base64_encode(b"ab"), "YWI=");
    assert_eq!(base64_encode(b"abc"), "YWJj");
    assert_eq!(base64_encode(b"abcd"), "YWJjZA==");
}
