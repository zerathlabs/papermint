use papermint::dialect::Dialect;
use papermint::{
    Command, EscPos, HeatingParameters, PaperWidth, PrintDensity, Printer, Receipt, Star,
};

#[test]
fn test_escpos_print_density_wire_bytes() {
    let escpos = EscPos::new();

    // GS ( K 2 0 48 m
    // Light -> 253 (-3, ~85%)
    let mut buf = Vec::new();
    escpos
        .encode(&Command::PrintDensity(PrintDensity::Light), &mut buf)
        .unwrap();
    assert_eq!(buf, vec![0x1D, 0x28, 0x4B, 0x02, 0x00, 0x30, 253]);

    // Normal -> 0 (100%)
    buf.clear();
    escpos
        .encode(&Command::PrintDensity(PrintDensity::Normal), &mut buf)
        .unwrap();
    assert_eq!(buf, vec![0x1D, 0x28, 0x4B, 0x02, 0x00, 0x30, 0]);

    // Dark -> 3 (+3, ~115%)
    buf.clear();
    escpos
        .encode(&Command::PrintDensity(PrintDensity::Dark), &mut buf)
        .unwrap();
    assert_eq!(buf, vec![0x1D, 0x28, 0x4B, 0x02, 0x00, 0x30, 3]);

    // HighContrast -> 6 (+6, ~130%)
    buf.clear();
    escpos
        .encode(
            &Command::PrintDensity(PrintDensity::HighContrast),
            &mut buf,
        )
        .unwrap();
    assert_eq!(buf, vec![0x1D, 0x28, 0x4B, 0x02, 0x00, 0x30, 6]);

    // Custom(42)
    buf.clear();
    escpos
        .encode(&Command::PrintDensity(PrintDensity::Custom(42)), &mut buf)
        .unwrap();
    assert_eq!(buf, vec![0x1D, 0x28, 0x4B, 0x02, 0x00, 0x30, 42]);
}

#[test]
fn test_escpos_heating_parameters_wire_bytes() {
    let escpos = EscPos::new();

    // ESC 7 n1 n2 n3
    // Default heating params: 7 (56 dots), 80 (800 µs), 2 (20 µs)
    let mut buf = Vec::new();
    escpos
        .encode(
            &Command::HeatingParameters(HeatingParameters::default()),
            &mut buf,
        )
        .unwrap();
    assert_eq!(buf, vec![0x1B, 0x37, 7, 80, 2]);

    // Custom heating params
    buf.clear();
    let custom = HeatingParameters {
        max_heating_dots: 10,
        heating_time: 120,
        heating_interval: 5,
    };
    escpos
        .encode(&Command::HeatingParameters(custom), &mut buf)
        .unwrap();
    assert_eq!(buf, vec![0x1B, 0x37, 10, 120, 5]);
}

#[test]
fn test_star_print_density_wire_bytes() {
    let star = Star::new();

    // ESC GS # '0' n LF NUL
    // Light -> b'1'
    let mut buf = Vec::new();
    star.encode(&Command::PrintDensity(PrintDensity::Light), &mut buf)
        .unwrap();
    assert_eq!(buf, vec![0x1B, 0x1D, 0x23, 0x30, b'1', 0x0A, 0x00]);

    // Normal -> b'3'
    buf.clear();
    star.encode(&Command::PrintDensity(PrintDensity::Normal), &mut buf)
        .unwrap();
    assert_eq!(buf, vec![0x1B, 0x1D, 0x23, 0x30, b'3', 0x0A, 0x00]);

    // Dark -> b'4'
    buf.clear();
    star.encode(&Command::PrintDensity(PrintDensity::Dark), &mut buf)
        .unwrap();
    assert_eq!(buf, vec![0x1B, 0x1D, 0x23, 0x30, b'4', 0x0A, 0x00]);

    // HighContrast -> b'5'
    buf.clear();
    star.encode(
        &Command::PrintDensity(PrintDensity::HighContrast),
        &mut buf,
    )
    .unwrap();
    assert_eq!(buf, vec![0x1B, 0x1D, 0x23, 0x30, b'5', 0x0A, 0x00]);

    // Custom(b'2')
    buf.clear();
    star.encode(&Command::PrintDensity(PrintDensity::Custom(b'2')), &mut buf)
        .unwrap();
    assert_eq!(buf, vec![0x1B, 0x1D, 0x23, 0x30, b'2', 0x0A, 0x00]);
}

#[test]
fn test_star_heating_parameters_wire_bytes() {
    let star = Star::new();

    // Default heating params: ESC 7 7 80 2
    let mut buf = Vec::new();
    star.encode(
        &Command::HeatingParameters(HeatingParameters::default()),
        &mut buf,
    )
    .unwrap();
    assert_eq!(buf, vec![0x1B, 0x37, 7, 80, 2]);
}

#[test]
fn test_receipt_density_builder() {
    let receipt = Receipt::new(PaperWidth::Mm80)
        .density_light()
        .density_normal()
        .density_dark()
        .density_high_contrast()
        .print_density(PrintDensity::Custom(99))
        .heating_parameters(HeatingParameters {
            max_heating_dots: 8,
            heating_time: 90,
            heating_interval: 3,
        });

    let cmds = receipt.commands();
    assert_eq!(cmds.len(), 6);
    assert_eq!(cmds[0], Command::PrintDensity(PrintDensity::Light));
    assert_eq!(cmds[1], Command::PrintDensity(PrintDensity::Normal));
    assert_eq!(cmds[2], Command::PrintDensity(PrintDensity::Dark));
    assert_eq!(cmds[3], Command::PrintDensity(PrintDensity::HighContrast));
    assert_eq!(cmds[4], Command::PrintDensity(PrintDensity::Custom(99)));
    assert_eq!(
        cmds[5],
        Command::HeatingParameters(HeatingParameters {
            max_heating_dots: 8,
            heating_time: 90,
            heating_interval: 3,
        })
    );
}

#[tokio::test]
async fn test_fluent_receipt_print_with_density_escpos() {
    let mut printer = Printer::escpos_mock();

    let receipt = Receipt::new(PaperWidth::Mm80)
        .init()
        .density_dark()
        .heating_parameters(HeatingParameters::default())
        .text_ln("Crisp Dark Receipt Header");

    printer.print(&receipt).await.expect("print failed");
    let bytes = printer.transport().bytes();

    // Verify ESC @ (init)
    assert_eq!(&bytes[0..2], &[0x1B, 0x40]);
    // Verify GS ( K 2 0 48 3 (density dark)
    assert_eq!(&bytes[2..9], &[0x1D, 0x28, 0x4B, 0x02, 0x00, 0x30, 0x03]);
    // Verify ESC 7 7 80 2 (heating strobe parameters)
    assert_eq!(&bytes[9..14], &[0x1B, 0x37, 7, 80, 2]);
}

#[tokio::test]
async fn test_fluent_receipt_print_with_density_star() {
    let mut printer = Printer::star_mock();

    let receipt = Receipt::new(PaperWidth::Mm80)
        .init()
        .density_high_contrast()
        .text_ln("High Contrast Star Printout");

    printer.print(&receipt).await.expect("print failed");
    let bytes = printer.transport().bytes();

    // Verify ESC @ (init)
    assert_eq!(&bytes[0..2], &[0x1B, 0x40]);
    // Verify ESC GS # '0' '5' LF NUL (density high contrast)
    assert_eq!(&bytes[2..9], &[0x1B, 0x1D, 0x23, 0x30, b'5', 0x0A, 0x00]);
}

