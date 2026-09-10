use papermint::{BarcodeData, BarcodeSystem, PaperWidth, Receipt};

#[test]
fn test_svg_preview_80mm() {
    let receipt = Receipt::new(PaperWidth::Mm80)
        .init()
        .center()
        .bold(true)
        .double_size(true)
        .text_ln("BURGER BAR")
        .double_size(false)
        .bold(false)
        .text_ln("123 Main Boulevard")
        .divider('-')
        .two_column("Double Cheeseburger", "$12.50")
        .two_column("Large Truffle Fries", "$5.50")
        .divider('=')
        .bold(true)
        .two_column("TOTAL", "$18.00")
        .bold(false)
        .feed(1)
        .qr("https://pay.example.com/receipt/12345")
        .barcode(BarcodeData {
            system: BarcodeSystem::Code128,
            data: "TX-998877".to_string(),
            width: 2,
            height: 50,
        })
        .feed(2)
        .cut_full();

    let svg = receipt.render_svg();

    // Check SVG wrapper
    assert!(svg.starts_with("<svg"), "Must start with <svg");
    assert!(svg.ends_with("</svg>\n"), "Must end with </svg>");
    assert!(
        svg.contains(r#"viewBox="0 0 576"#),
        "80mm should have 576 dot width"
    );

    // Check text elements
    assert!(svg.contains("BURGER BAR"), "Must contain header");
    assert!(svg.contains("123 Main Boulevard"), "Must contain address");
    assert!(
        svg.contains("Double Cheeseburger"),
        "Must contain line item"
    );
    assert!(svg.contains("$18.00"), "Must contain total");

    // Check graphics elements
    assert!(
        svg.contains("<!-- QR Code -->"),
        "Must contain QR code block"
    );
    assert!(
        svg.contains("<!-- Barcode -->"),
        "Must contain barcode block"
    );
    assert!(svg.contains("TX-998877"), "Must contain barcode label text");
    assert!(
        svg.contains("<!-- Paper Cut Marker -->"),
        "Must contain cut marker"
    );
    assert!(svg.contains("✂ CUT"), "Must contain cut label");
}

#[test]
fn test_svg_preview_58mm() {
    let receipt = Receipt::new(PaperWidth::Mm58)
        .center()
        .text_ln("MINI 58MM POS")
        .two_column("Tea", "$2.00")
        .cut_partial();

    let svg = receipt.render_svg();
    assert!(
        svg.contains(r#"viewBox="0 0 384"#),
        "58mm should have 384 dot width"
    );
    assert!(svg.contains("MINI 58MM POS"));
    assert!(
        svg.contains("✂ PARTIAL CUT"),
        "Must reflect partial cut mode"
    );
}

#[test]
fn test_html_preview() {
    let receipt = Receipt::new(PaperWidth::Mm80)
        .center()
        .bold(true)
        .text_ln("CAFÉ DELIGHT & CO <SPECIAL>")
        .bold(false)
        .two_column("Cappuccino", "$4.50")
        .feed(1)
        .qr("ORDER-8821")
        .cut_full();

    let html = receipt.render_html();

    assert!(
        html.contains(r#"class="papermint-receipt""#),
        "Must contain receipt container class"
    );
    assert!(
        html.contains("CAFÉ DELIGHT &amp; CO &lt;SPECIAL&gt;"),
        "Must escape HTML characters"
    );
    assert!(html.contains("Cappuccino"), "Must contain item text");
    assert!(
        html.contains("<svg"),
        "Must contain vector graphics inside HTML"
    );
    assert!(html.contains("✂ Cut"), "Must contain cut indicator");
}

#[test]
fn test_html_preview_inverted_styling() {
    let receipt = Receipt::new(PaperWidth::Mm80)
        .invert(true)
        .text_ln("VIP CUSTOMER")
        .invert(false)
        .text_ln("Standard Customer");

    let html = receipt.render_html();
    assert!(
        html.contains("background-color: #111111"),
        "Must apply black background for inverted text"
    );
    assert!(
        html.contains("color: #ffffff"),
        "Must apply white font for inverted text"
    );
    assert!(html.contains("VIP CUSTOMER"));
}
