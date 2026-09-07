use papermint::{PaperWidth, Printer, Receipt};

#[tokio::test]
async fn test_receipt_builder_and_mock_printer() {
    let mut printer = Printer::escpos_mock();

    let receipt = Receipt::new(PaperWidth::Mm80)
        .init()
        .center()
        .bold(true)
        .double_size(true)
        .text_ln("PAPERTOWN RESTAURANT")
        .double_size(false)
        .bold(false)
        .text_ln("Order # 1042")
        .left()
        .divider('-')
        .two_column("1x Smash Burger", "KWD 3.500")
        .two_column("1x Truffle Fries", "KWD 1.750")
        .two_column("1x Vanilla Shake", "KWD 1.250")
        .divider('=')
        .bold(true)
        .two_column("SUBTOTAL", "KWD 6.500")
        .two_column("TOTAL (VAT incl.)", "KWD 6.500")
        .bold(false)
        .divider('-')
        .center()
        .qr("https://pay.papertown.app/receipt/1042")
        .feed(2)
        .text_ln("Thank you for dining with us!")
        .feed(3)
        .cut_full()
        .open_drawer();

    let cmd_count = receipt.commands().len();
    assert!(cmd_count > 15);

    // Print receipt through mock printer
    printer.print(&receipt).await.expect("printing should succeed");

    let bytes = printer.transport().bytes();
    assert!(!bytes.is_empty());

    // Verify initial reset bytes ESC @
    assert_eq!(&bytes[0..2], &[0x1B, 0x40]);

    // Verify restaurant name appears in bytes
    let output_str = String::from_utf8_lossy(&bytes);
    assert!(output_str.contains("PAPERTOWN RESTAURANT"));
    assert!(output_str.contains("1x Smash Burger"));
    assert!(output_str.contains("KWD 3.500"));
    assert!(output_str.contains("SUBTOTAL"));

    // Verify cut full (0x1D, 0x56, 0x00) is present
    assert!(bytes.windows(3).any(|w| w == [0x1D, 0x56, 0x00]));

    // Verify drawer kick (0x1B, 0x70, 0x00, 25, 250) is present
    assert!(bytes.windows(5).any(|w| w == [0x1B, 0x70, 0x00, 25, 250]));
}

#[test]
fn test_paper_widths() {
    let r80 = Receipt::new(PaperWidth::Mm80);
    assert_eq!(r80.columns(), 48);

    let r58 = Receipt::new(PaperWidth::Mm58);
    assert_eq!(r58.columns(), 32);

    let r_custom = Receipt::new(PaperWidth::Custom(42));
    assert_eq!(r_custom.columns(), 42);
}
