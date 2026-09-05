use papermint::{
    Alignment, BarcodeData, BarcodeSystem, CodePage, Command, CutMode, DrawerPin, Encoder,
    ImageData, QrCorrectionLevel, QrData,
};

#[test]
fn test_star_init() {
    let encoder = Encoder::star();
    let bytes = encoder.encode(&[Command::Init]).unwrap();
    assert_eq!(bytes, vec![0x1B, 0x40]);
}

#[test]
fn test_star_text_and_feed() {
    let encoder = Encoder::star();
    let bytes = encoder
        .encode(&[
            Command::Text("Star Line\n".to_string()),
            Command::Feed(2),
        ])
        .unwrap();
    assert_eq!(bytes, b"Star Line\n\n\n");
}

#[test]
fn test_star_bold_and_invert() {
    let encoder = Encoder::star();
    assert_eq!(encoder.encode(&[Command::Bold(true)]).unwrap(), vec![0x1B, b'E']);
    assert_eq!(encoder.encode(&[Command::Bold(false)]).unwrap(), vec![0x1B, b'F']);
    assert_eq!(encoder.encode(&[Command::Invert(true)]).unwrap(), vec![0x1B, b'4']);
    assert_eq!(encoder.encode(&[Command::Invert(false)]).unwrap(), vec![0x1B, b'5']);
}

#[test]
fn test_star_alignment() {
    let encoder = Encoder::star();
    // Star alignment is ESC GS a n (0 = left, 1 = center, 2 = right)
    assert_eq!(encoder.encode(&[Command::Align(Alignment::Left)]).unwrap(), vec![0x1B, 0x1D, b'a', 0]);
    assert_eq!(encoder.encode(&[Command::Align(Alignment::Center)]).unwrap(), vec![0x1B, 0x1D, b'a', 1]);
    assert_eq!(encoder.encode(&[Command::Align(Alignment::Right)]).unwrap(), vec![0x1B, 0x1D, b'a', 2]);
}

#[test]
fn test_star_cut() {
    let encoder = Encoder::star();
    // Star cut is ESC d 2 (full) and ESC d 3 (partial)
    assert_eq!(encoder.encode(&[Command::Cut(CutMode::Full)]).unwrap(), vec![0x1B, b'd', 2]);
    assert_eq!(encoder.encode(&[Command::Cut(CutMode::Partial)]).unwrap(), vec![0x1B, b'd', 3]);
}

#[test]
fn test_star_drawer_and_beep() {
    let encoder = Encoder::star();
    assert_eq!(
        encoder.encode(&[Command::DrawerKick(DrawerPin::Pin2)]).unwrap(),
        vec![0x1B, 0x07, 11, 55, 0x07] // ESC BEL n1 n2 BEL
    );
    assert_eq!(
        encoder.encode(&[Command::DrawerKick(DrawerPin::Pin5)]).unwrap(),
        vec![0x1B, 0x1C, 11, 55, 0x1C] // ESC FS n1 n2 FS
    );
    assert_eq!(
        encoder.encode(&[Command::Beep { count: 1, duration: 1 }]).unwrap(),
        vec![0x1B, 0x07] // ESC BEL
    );
}

#[test]
fn test_star_barcode_and_image() {
    let encoder = Encoder::star();

    // Barcode: Code 128 (system 6 in Star Line Mode)
    let barcode_bytes = encoder
        .encode(&[Command::Barcode(BarcodeData {
            system: BarcodeSystem::Code128,
            data: "12345".to_string(),
            width: 2,
            height: 64,
        })])
        .unwrap();
    // ESC b 6 2 2 64 "{B12345" RS (0x1E)
    let mut expected_bc = vec![0x1B, b'b', 6, 2, 2, 64];
    expected_bc.extend_from_slice(b"{B12345");
    expected_bc.push(0x1E);
    assert_eq!(barcode_bytes, expected_bc);

    // Raster Image: 16x2 pixels (2 bytes wide, 2 rows)
    let img_bytes = encoder
        .encode(&[Command::Image(ImageData {
            width: 16,
            height: 2,
            pixels: vec![0xAA, 0x55, 0xFF, 0x00],
        })])
        .unwrap();
    let expected_img = vec![
        0x1B, b'*', b'r', b'A',           // Enter raster mode
        0x1B, b'*', b'r', b'b', 2, 0, 0xAA, 0x55, // Row 0
        0x1B, b'*', b'r', b'b', 2, 0, 0xFF, 0x00, // Row 1
        0x1B, b'*', b'r', b'B',           // Exit raster mode
    ];
    assert_eq!(img_bytes, expected_img);
}

#[test]
fn test_star_line_spacing_and_modes() {
    let encoder = Encoder::star();
    assert_eq!(encoder.encode(&[Command::LineSpacing(None)]).unwrap(), vec![0x1B, b'z', 1]);
    assert_eq!(encoder.encode(&[Command::LineSpacing(Some(24))]).unwrap(), vec![0x1B, b'3', 24]);
    assert_eq!(encoder.encode(&[Command::UpsideDown(true)]).unwrap(), vec![0x0F]);
    assert_eq!(encoder.encode(&[Command::UpsideDown(false)]).unwrap(), vec![0x12]);
    assert_eq!(encoder.encode(&[Command::CodePage(CodePage::Wpc1252)]).unwrap(), vec![0x1B, 0x1D, b't', 32]);
    assert_eq!(encoder.encode(&[Command::CodePage(CodePage::Pc437)]).unwrap(), vec![0x1B, 0x1D, b't', 1]);
    assert_eq!(encoder.encode(&[Command::CodePage(CodePage::Pc850)]).unwrap(), vec![0x1B, 0x1D, b't', 0]);
}

#[test]
fn test_star_qr_code() {
    let encoder = Encoder::star();
    let bytes = encoder
        .encode(&[Command::QrCode(QrData {
            data: "https://example.com".to_string(),
            model: 2,
            cell_size: 4,
            correction: QrCorrectionLevel::M,
        })])
        .unwrap();

    // Verify presence of Star QR commands:
    // 1. Model: ESC GS y S 0 2
    assert!(bytes.windows(6).any(|w| w == [0x1B, 0x1D, b'y', b'S', b'0', 2]));
    // 2. Correction: ESC GS y S 1 1 (M = 1)
    assert!(bytes.windows(6).any(|w| w == [0x1B, 0x1D, b'y', b'S', b'1', 1]));
    // 3. Cell size: ESC GS y S 2 4
    assert!(bytes.windows(6).any(|w| w == [0x1B, 0x1D, b'y', b'S', b'2', 4]));
    // 4. Print: ESC GS y P
    assert!(bytes.windows(4).any(|w| w == [0x1B, 0x1D, b'y', b'P']));
}
