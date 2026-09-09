use papermint::dialect::Dialect;
use papermint::{CodePage, EscPos, InternationalCharset, PaperWidth, Printer, Receipt, Star};

#[test]
fn test_escpos_international_charset_wire_bytes() {
    let escpos = EscPos::new();

    // ESC R 0 -> USA
    let mut buf = Vec::new();
    escpos
        .encode(
            &papermint::Command::InternationalCharset(InternationalCharset::Usa),
            &mut buf,
        )
        .unwrap();
    assert_eq!(buf, vec![0x1B, 0x52, 0x00]);

    // ESC R 1 -> France
    buf.clear();
    escpos
        .encode(
            &papermint::Command::InternationalCharset(InternationalCharset::France),
            &mut buf,
        )
        .unwrap();
    assert_eq!(buf, vec![0x1B, 0x52, 0x01]);

    // ESC R 2 -> Germany
    buf.clear();
    escpos
        .encode(
            &papermint::Command::InternationalCharset(InternationalCharset::Germany),
            &mut buf,
        )
        .unwrap();
    assert_eq!(buf, vec![0x1B, 0x52, 0x02]);

    // ESC R 3 -> UK
    buf.clear();
    escpos
        .encode(
            &papermint::Command::InternationalCharset(InternationalCharset::Uk),
            &mut buf,
        )
        .unwrap();
    assert_eq!(buf, vec![0x1B, 0x52, 0x03]);

    // ESC R 7 -> Spain I
    buf.clear();
    escpos
        .encode(
            &papermint::Command::InternationalCharset(InternationalCharset::SpainI),
            &mut buf,
        )
        .unwrap();
    assert_eq!(buf, vec![0x1B, 0x52, 0x07]);

    // ESC R 8 -> Japan
    buf.clear();
    escpos
        .encode(
            &papermint::Command::InternationalCharset(InternationalCharset::Japan),
            &mut buf,
        )
        .unwrap();
    assert_eq!(buf, vec![0x1B, 0x52, 0x08]);

    // ESC R 17 -> Arabia
    buf.clear();
    escpos
        .encode(
            &papermint::Command::InternationalCharset(InternationalCharset::Arabia),
            &mut buf,
        )
        .unwrap();
    assert_eq!(buf, vec![0x1B, 0x52, 0x11]);
}

#[test]
fn test_star_international_charset_wire_bytes() {
    let star = Star::new();

    // Star also uses ESC R n
    let mut buf = Vec::new();
    star.encode(
        &papermint::Command::InternationalCharset(InternationalCharset::Uk),
        &mut buf,
    )
    .unwrap();
    assert_eq!(buf, vec![0x1B, 0x52, 0x03]);

    buf.clear();
    star.encode(
        &papermint::Command::InternationalCharset(InternationalCharset::Germany),
        &mut buf,
    )
    .unwrap();
    assert_eq!(buf, vec![0x1B, 0x52, 0x02]);
}

#[test]
fn test_escpos_codepage_wire_bytes() {
    let escpos = EscPos::new();

    // ESC t 16 -> WPC1252
    let mut buf = Vec::new();
    escpos
        .encode(&papermint::Command::CodePage(CodePage::Wpc1252), &mut buf)
        .unwrap();
    assert_eq!(buf, vec![0x1B, 0x74, 16]);

    // ESC t 19 -> PC858 (Euro)
    buf.clear();
    escpos
        .encode(&papermint::Command::CodePage(CodePage::Pc858), &mut buf)
        .unwrap();
    assert_eq!(buf, vec![0x1B, 0x74, 19]);

    // ESC t 50 -> WPC1256 (Arabic)
    buf.clear();
    escpos
        .encode(&papermint::Command::CodePage(CodePage::Wpc1256), &mut buf)
        .unwrap();
    assert_eq!(buf, vec![0x1B, 0x74, 50]);

    // ESC t 17 -> PC866 (Cyrillic)
    buf.clear();
    escpos
        .encode(&papermint::Command::CodePage(CodePage::Pc866), &mut buf)
        .unwrap();
    assert_eq!(buf, vec![0x1B, 0x74, 17]);
}

#[test]
fn test_star_codepage_wire_bytes() {
    let star = Star::new();

    // Star uses ESC GS t n (0x1B 0x1D 0x74 n)
    // WPC1252 in Star is table 32
    let mut buf = Vec::new();
    star.encode(&papermint::Command::CodePage(CodePage::Wpc1252), &mut buf)
        .unwrap();
    assert_eq!(buf, vec![0x1B, 0x1D, 0x74, 32]);

    // PC858 in Star is table 4
    buf.clear();
    star.encode(&papermint::Command::CodePage(CodePage::Pc858), &mut buf)
        .unwrap();
    assert_eq!(buf, vec![0x1B, 0x1D, 0x74, 4]);

    // PC850 in Star is table 0
    buf.clear();
    star.encode(&papermint::Command::CodePage(CodePage::Pc850), &mut buf)
        .unwrap();
    assert_eq!(buf, vec![0x1B, 0x1D, 0x74, 0]);
}

#[test]
fn test_wpc1252_single_byte_transcoding() {
    let page = CodePage::Wpc1252;

    // Euro symbol in Windows-1252 is 0x80 (NOT multi-byte UTF-8 0xE2 0x82 0xAC)
    assert_eq!(page.encode_char('€'), Some(0x80));

    // Accented letters:
    assert_eq!(page.encode_char('é'), Some(0xE9));
    assert_eq!(page.encode_char('à'), Some(0xE0));
    assert_eq!(page.encode_char('ñ'), Some(0xF1));
    assert_eq!(page.encode_char('ü'), Some(0xFC));
    assert_eq!(page.encode_char('ß'), Some(0xDF));
    assert_eq!(page.encode_char('£'), Some(0xA3));
    assert_eq!(page.encode_char('¥'), Some(0xA5));

    // String encoding:
    let encoded = page.encode_text("Café 12.50 €");
    assert_eq!(
        encoded,
        vec![
            b'C', b'a', b'f', 0xE9, // Café
            b' ', b'1', b'2', b'.', b'5', b'0', b' ', 0x80 // Euro sign single byte!
        ]
    );
}

#[test]
fn test_pc858_euro_and_accents() {
    let page = CodePage::Pc858;

    // In PC858, Euro is 0xD5 (213)
    assert_eq!(page.encode_char('€'), Some(0xD5));
    // é is 0x82
    assert_eq!(page.encode_char('é'), Some(0x82));

    let encoded = page.encode_text("10 €");
    assert_eq!(encoded, vec![b'1', b'0', b' ', 0xD5]);
}

#[test]
fn test_pc866_cyrillic_transcoding() {
    let page = CodePage::Pc866;

    // 'П' is 0x8F, 'р' is 0xE0, 'и' is 0xA8, 'в' is 0xA2, 'е' is 0xA5, 'т' is 0xE2
    let encoded = page.encode_text("Привет");
    assert_eq!(encoded, vec![0x8F, 0xE0, 0xA8, 0xA2, 0xA5, 0xE2]);
}

#[tokio::test]
async fn test_fluent_receipt_with_international_charset_and_codepage() {
    let mut printer = Printer::escpos_mock();

    let receipt = Receipt::new(PaperWidth::Mm80)
        .init()
        .charset_uk()
        .code_page(CodePage::Wpc1252)
        .text_ln("Fish & Chips: 12.50 £")
        .text_ln("Crêpe: 4.50 €");

    printer.print(&receipt).await.expect("print failed");
    let bytes = printer.transport().bytes();

    // Verify ESC @ (init)
    assert_eq!(&bytes[0..2], &[0x1B, 0x40]);
    // Verify ESC R 3 (UK charset)
    assert_eq!(&bytes[2..5], &[0x1B, 0x52, 0x03]);
    // Verify ESC t 16 (WPC1252)
    assert_eq!(&bytes[5..8], &[0x1B, 0x74, 16]);

    // Verify Euro and Pound are single-byte transcoded:
    // '£' in WPC1252 is 0xA3
    // '€' in WPC1252 is 0x80
    // 'ê' in WPC1252 is 0xEA
    assert!(bytes.contains(&0xA3), "Must contain single-byte £ (0xA3)");
    assert!(bytes.contains(&0x80), "Must contain single-byte € (0x80)");
    assert!(bytes.contains(&0xEA), "Must contain single-byte ê (0xEA)");

    // Ensure NO raw 3-byte UTF-8 Euro bytes [0xE2, 0x82, 0xAC] leaked:
    let has_raw_utf8_euro = bytes.windows(3).any(|w| w == [0xE2, 0x82, 0xAC]);
    assert!(
        !has_raw_utf8_euro,
        "Raw UTF-8 Euro bytes must not leak to printer"
    );
}
