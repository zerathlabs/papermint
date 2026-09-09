use papermint::{Command, PaperWidth, Printer, Receipt};

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
    printer
        .print(&receipt)
        .await
        .expect("printing should succeed");

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

#[test]
fn test_receipt_table_row_and_word_wrapping() {
    use papermint::{Alignment, Command, TableColumn};

    let columns = [
        TableColumn::fixed(4, Alignment::Left),
        TableColumn::fraction(0.50, Alignment::Left),
        TableColumn::fraction(0.25, Alignment::Right),
        TableColumn::fraction(0.25, Alignment::Right),
    ];

    let receipt = Receipt::new(PaperWidth::Mm80)
        .init()
        .table_row(&["QTY", "ITEM", "PRICE", "TOTAL"], &columns)
        .divider('-')
        .table_row(
            &[
                "2x",
                "Double Truffle Wagyu Smash Burger with Caramelized Onions",
                "$12.00",
                "$24.00",
            ],
            &columns,
        )
        .table_row(&["1x", "Mint Cooler", "$3.50", "$3.50"], &columns);

    let cmds = receipt.commands();
    // Multi-line item wrapping should generate multiple Text commands
    assert!(cmds.len() >= 5);

    let mut all_text = String::new();
    for cmd in cmds {
        if let Command::Text(s) = cmd {
            all_text.push_str(s);
        }
    }

    assert!(all_text.contains("Double Truffle"));
    assert!(all_text.contains("Smash Burger with"));
    assert!(all_text.contains("Caramelized Onions"));
    assert!(all_text.contains("$24.00"));
    assert!(all_text.contains("Mint Cooler"));
}

#[test]
fn test_receipt_stateful_columns_and_row_builder() {
    use papermint::{Alignment, Command, TableColumn};

    let columns = [
        TableColumn::fixed(4, Alignment::Left),
        TableColumn::fraction(0.50, Alignment::Left),
        TableColumn::fraction(0.25, Alignment::Right),
        TableColumn::fraction(0.25, Alignment::Right),
    ];

    let receipt = Receipt::new(PaperWidth::Mm80)
        .init()
        .table_header(&["QTY", "ITEM", "PRICE", "TOTAL"], &columns)
        .divider('-')
        .row(&[
            "2x",
            "Double Truffle Wagyu Smash Burger with Caramelized Onions",
            "$12.00",
            "$24.00",
        ])
        .row(&["1x", "Mint Cooler", "$3.50", "$3.50"]);

    assert_eq!(receipt.active_columns(), Some(&columns[..]));
    let receipt_cleared = receipt.clone().clear_columns();
    assert_eq!(receipt_cleared.active_columns(), None);

    let cmds = receipt.commands();
    // Verify bold commands surround table_header
    assert_eq!(cmds[1], Command::Bold(true));
    assert!(cmds.contains(&Command::Bold(false)));

    let mut all_text = String::new();
    for cmd in cmds {
        if let Command::Text(s) = cmd {
            all_text.push_str(s);
        }
    }

    assert!(all_text.contains("QTY"));
    assert!(all_text.contains("Double Truffle"));
    assert!(all_text.contains("Caramelized Onions"));
    assert!(all_text.contains("$24.00"));
    assert!(all_text.contains("Mint Cooler"));
}

#[test]
fn test_rich_dividers() {
    let receipt_80mm = Receipt::new(PaperWidth::Mm80)
        .divider_double()
        .divider_dotted()
        .divider_dashed()
        .divider_pattern("=-");

    let cols_80 = PaperWidth::Mm80.columns(); // 48
    let cmds_80 = receipt_80mm.commands();

    // Check divider_double
    if let Command::Text(s) = &cmds_80[0] {
        let trimmed = s.trim_end_matches('\n');
        assert_eq!(trimmed.len(), cols_80);
        assert!(trimmed.chars().all(|c| c == '='));
    } else {
        panic!("Expected Text command for divider_double");
    }

    // Check divider_dotted
    if let Command::Text(s) = &cmds_80[1] {
        let trimmed = s.trim_end_matches('\n');
        assert_eq!(trimmed.len(), cols_80);
        assert!(trimmed.chars().all(|c| c == '.'));
    } else {
        panic!("Expected Text command for divider_dotted");
    }

    // Check divider_dashed
    if let Command::Text(s) = &cmds_80[2] {
        let trimmed = s.trim_end_matches('\n');
        assert_eq!(trimmed.len(), cols_80);
        assert!(trimmed.starts_with("- - - "));
    } else {
        panic!("Expected Text command for divider_dashed");
    }

    // Check divider_pattern
    if let Command::Text(s) = &cmds_80[3] {
        let trimmed = s.trim_end_matches('\n');
        assert_eq!(trimmed.len(), cols_80);
        assert!(trimmed.starts_with("=-=-=-"));
    } else {
        panic!("Expected Text command for divider_pattern");
    }

    // Verify 58mm paper width too
    let receipt_58mm = Receipt::new(PaperWidth::Mm58).divider_double();
    let cols_58 = PaperWidth::Mm58.columns(); // 32
    if let Command::Text(s) = &receipt_58mm.commands()[0] {
        let trimmed = s.trim_end_matches('\n');
        assert_eq!(trimmed.len(), cols_58);
        assert!(trimmed.chars().all(|c| c == '='));
    } else {
        panic!("Expected Text command for 58mm divider_double");
    }
}
