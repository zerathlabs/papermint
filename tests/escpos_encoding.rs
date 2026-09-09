use papermint::{
    Alignment, BarcodeData, BarcodeSystem, CodePage, Command, CutMode, DrawerPin, Encoder,
    FontFamily, QrCorrectionLevel, QrData, UnderlineMode,
};

#[test]
fn test_escpos_init() {
    let encoder = Encoder::escpos();
    let bytes = encoder.encode(&[Command::Init]).unwrap();
    assert_eq!(bytes, vec![0x1B, 0x40]);
}

#[test]
fn test_escpos_text() {
    let encoder = Encoder::escpos();
    let bytes = encoder
        .encode(&[Command::Text("Hello, POS!\n".to_string())])
        .unwrap();
    assert_eq!(bytes, b"Hello, POS!\n");
}

#[test]
fn test_escpos_feed() {
    let encoder = Encoder::escpos();
    assert_eq!(
        encoder.encode(&[Command::Feed(0)]).unwrap(),
        Vec::<u8>::new()
    );
    assert_eq!(encoder.encode(&[Command::Feed(1)]).unwrap(), vec![0x0A]);
    assert_eq!(
        encoder.encode(&[Command::Feed(3)]).unwrap(),
        vec![0x1B, 0x64, 0x03]
    );
}

#[test]
fn test_escpos_alignment() {
    let encoder = Encoder::escpos();
    assert_eq!(
        encoder.encode(&[Command::Align(Alignment::Left)]).unwrap(),
        vec![0x1B, 0x61, 0]
    );
    assert_eq!(
        encoder
            .encode(&[Command::Align(Alignment::Center)])
            .unwrap(),
        vec![0x1B, 0x61, 1]
    );
    assert_eq!(
        encoder.encode(&[Command::Align(Alignment::Right)]).unwrap(),
        vec![0x1B, 0x61, 2]
    );
}

#[test]
fn test_escpos_bold_and_invert() {
    let encoder = Encoder::escpos();
    assert_eq!(
        encoder.encode(&[Command::Bold(true)]).unwrap(),
        vec![0x1B, 0x45, 1]
    );
    assert_eq!(
        encoder.encode(&[Command::Bold(false)]).unwrap(),
        vec![0x1B, 0x45, 0]
    );
    assert_eq!(
        encoder.encode(&[Command::Invert(true)]).unwrap(),
        vec![0x1D, 0x42, 1]
    );
    assert_eq!(
        encoder.encode(&[Command::Invert(false)]).unwrap(),
        vec![0x1D, 0x42, 0]
    );
}

#[test]
fn test_escpos_underline() {
    let encoder = Encoder::escpos();
    assert_eq!(
        encoder
            .encode(&[Command::Underline(UnderlineMode::Single)])
            .unwrap(),
        vec![0x1B, 0x2D, 1]
    );
    assert_eq!(
        encoder
            .encode(&[Command::Underline(UnderlineMode::Double)])
            .unwrap(),
        vec![0x1B, 0x2D, 2]
    );
    assert_eq!(
        encoder
            .encode(&[Command::Underline(UnderlineMode::Off)])
            .unwrap(),
        vec![0x1B, 0x2D, 0]
    );
}

#[test]
fn test_escpos_font_family() {
    let encoder = Encoder::escpos();
    assert_eq!(
        encoder.encode(&[Command::Font(FontFamily::A)]).unwrap(),
        vec![0x1B, 0x4D, 0]
    );
    assert_eq!(
        encoder.encode(&[Command::Font(FontFamily::B)]).unwrap(),
        vec![0x1B, 0x4D, 1]
    );
}

#[test]
fn test_escpos_cut() {
    let encoder = Encoder::escpos();
    assert_eq!(
        encoder.encode(&[Command::Cut(CutMode::Full)]).unwrap(),
        vec![0x1D, 0x56, 0]
    );
    assert_eq!(
        encoder.encode(&[Command::Cut(CutMode::Partial)]).unwrap(),
        vec![0x1D, 0x56, 1]
    );
}

#[test]
fn test_escpos_drawer_kick() {
    let encoder = Encoder::escpos();
    assert_eq!(
        encoder
            .encode(&[Command::DrawerKick(DrawerPin::Pin2)])
            .unwrap(),
        vec![0x1B, 0x70, 0, 25, 250]
    );
    assert_eq!(
        encoder
            .encode(&[Command::DrawerKick(DrawerPin::Pin5)])
            .unwrap(),
        vec![0x1B, 0x70, 1, 25, 250]
    );
}

#[test]
fn test_escpos_beep() {
    let encoder = Encoder::escpos();
    assert_eq!(
        encoder
            .encode(&[Command::Beep {
                count: 2,
                duration: 3
            }])
            .unwrap(),
        vec![0x1B, 0x42, 2, 3]
    );
}

#[test]
fn test_escpos_barcode_code128() {
    let encoder = Encoder::escpos();
    let bytes = encoder
        .encode(&[Command::Barcode(BarcodeData {
            system: BarcodeSystem::Code128,
            data: "TEST1234".to_string(),
            width: 2,
            height: 64,
        })])
        .unwrap();

    // GS h 64, GS w 2, GS k 73 10 "{BTEST1234" (auto-prefixed with {B for Code Set B)
    let mut expected = vec![
        0x1D, b'h', 64, // height
        0x1D, b'w', 2, // width
        0x1D, b'k', 73, 10, // Code128 system 73, length 10 (2 prefix + 8 data)
    ];
    expected.extend_from_slice(b"{BTEST1234");
    assert_eq!(bytes, expected);
}

#[test]
fn test_escpos_line_spacing_and_modes() {
    let encoder = Encoder::escpos();
    assert_eq!(
        encoder.encode(&[Command::LineSpacing(None)]).unwrap(),
        vec![0x1B, b'2']
    );
    assert_eq!(
        encoder.encode(&[Command::LineSpacing(Some(30))]).unwrap(),
        vec![0x1B, b'3', 30]
    );
    assert_eq!(
        encoder.encode(&[Command::UpsideDown(true)]).unwrap(),
        vec![0x1B, b'{', 1]
    );
    assert_eq!(
        encoder.encode(&[Command::UpsideDown(false)]).unwrap(),
        vec![0x1B, b'{', 0]
    );
    assert_eq!(
        encoder
            .encode(&[Command::CodePage(CodePage::Wpc1252)])
            .unwrap(),
        vec![0x1B, b't', 16]
    );
    assert_eq!(
        encoder
            .encode(&[Command::CodePage(CodePage::Pc437)])
            .unwrap(),
        vec![0x1B, b't', 0]
    );
}

#[test]
fn test_escpos_qr_code() {
    let encoder = Encoder::escpos();
    let bytes = encoder
        .encode(&[Command::QrCode(QrData {
            data: "https://example.com".to_string(),
            model: 2,
            cell_size: 4,
            correction: QrCorrectionLevel::M,
        })])
        .unwrap();

    // Verify presence of QR commands:
    // 1. Model: GS ( k 4 0 49 65 50 0
    assert!(
        bytes
            .windows(9)
            .any(|w| w == [0x1D, b'(', b'k', 4, 0, 49, 65, 50, 0])
    );
    // 2. Cell size: GS ( k 3 0 49 67 4
    assert!(
        bytes
            .windows(8)
            .any(|w| w == [0x1D, b'(', b'k', 3, 0, 49, 67, 4])
    );
    // 3. Correction: GS ( k 3 0 49 69 49 (M = 49)
    assert!(
        bytes
            .windows(8)
            .any(|w| w == [0x1D, b'(', b'k', 3, 0, 49, 69, 49])
    );
    // 4. Print: GS ( k 3 0 49 81 48
    assert!(
        bytes
            .windows(8)
            .any(|w| w == [0x1D, b'(', b'k', 3, 0, 49, 81, 48])
    );
}

#[test]
fn test_escpos_tall_image_chunking() {
    use papermint::ImageData;

    let encoder = Encoder::escpos();
    // Create an image that is 16 pixels wide (2 bytes) and 2000 pixels high.
    // Safe chunk height is 960 dots.
    // 2000 dots should be split into 3 chunks:
    // - Chunk 1: 960 dots (y_l = 0xC0, y_h = 0x03)
    // - Chunk 2: 960 dots (y_l = 0xC0, y_h = 0x03)
    // - Chunk 3: 80 dots  (y_l = 0x50, y_h = 0x00)
    let width = 16u32;
    let height = 2000u32;
    let width_bytes = 2usize;
    let pixels = vec![0xAA; width_bytes * height as usize];

    let img = ImageData {
        width,
        height,
        pixels,
    };

    let bytes = encoder.encode(&[Command::Image(img)]).unwrap();

    let mut headers = Vec::new();
    let mut i = 0;
    while i + 8 <= bytes.len() {
        if bytes[i] == 0x1D && bytes[i + 1] == b'v' && bytes[i + 2] == b'0' && bytes[i + 3] == 0 {
            let x_l = bytes[i + 4];
            let x_h = bytes[i + 5];
            let y_l = bytes[i + 6];
            let y_h = bytes[i + 7];
            let chunk_h = (y_l as usize) | ((y_h as usize) << 8);
            let chunk_bytes = ((x_l as usize) | ((x_h as usize) << 8)) * chunk_h;
            headers.push(chunk_h);
            i += 8 + chunk_bytes;
        } else {
            i += 1;
        }
    }

    assert_eq!(
        headers,
        vec![960, 960, 80],
        "Expected 960, 960, and 80 dot chunk slices"
    );
}
