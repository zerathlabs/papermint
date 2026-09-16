//! # 🏷️ papermint-label
//!
//! High-performance, 2D thermal label printing engine supporting **TSPL** (Xprinter/TSC/Rongta)
//! and **ZPL II** (Zebra/Citizen).
//!
//! Designed for adhesive coffee cup stickers, retail barcode tags, courier shipping labels,
//! and warehouse inventory waybills.

pub mod command;
pub mod encoder;
pub mod label;
pub mod preview;

pub use command::{BarcodeType, Direction, LabelCommand, QrErrorCorrection, Rotation, Unit};
pub use encoder::{encode_tspl, encode_zpl};
pub use label::Label;
pub use preview::render_svg;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_dimensions_dots() {
        let label = Label::new(50.0, 30.0); // 203 DPI = 8 dots/mm
        assert_eq!(label.width_dots(), 400);
        assert_eq!(label.height_dots(), 240);
    }

    #[test]
    fn test_tspl_encoding_basic() {
        let label = Label::new(50.0, 30.0)
            .gap(2.0, 0.0)
            .text(20, 20, "3", 1, "ORGANIC COFFEE")
            .barcode(20, 80, BarcodeType::Code128, 50, "ITEM-1042")
            .qr(280, 80, 4, "https://example.com/item/1042")
            .box_outline(10, 10, 380, 220, 2)
            .copies(2);

        let bytes = label.encode_tspl();
        let tspl_str = String::from_utf8_lossy(&bytes);

        assert!(tspl_str.contains("SIZE 50.00 mm, 30.00 mm\r\n"));
        assert!(tspl_str.contains("GAP 2.00 mm, 0.00 mm\r\n"));
        assert!(tspl_str.contains("CLS\r\n"));
        assert!(tspl_str.contains("TEXT 20,20,\"3\",0,1,1,\"ORGANIC COFFEE\"\r\n"));
        assert!(tspl_str.contains("BARCODE 20,80,\"128\",50,2,0,2,4,\"ITEM-1042\"\r\n"));
        assert!(tspl_str.contains("QRCODE 280,80,M,4,A,0,\"https://example.com/item/1042\"\r\n"));
        assert!(tspl_str.contains("BOX 10,10,390,230,2\r\n"));
        assert!(tspl_str.contains("PRINT 2,1\r\n"));
    }

    #[test]
    fn test_zpl_encoding_basic() {
        let label = Label::new(50.0, 30.0)
            .text(20, 20, "0", 1, "ORGANIC COFFEE")
            .barcode(20, 80, BarcodeType::Code128, 50, "ITEM-1042")
            .qr(280, 80, 4, "https://example.com/item/1042")
            .box_outline(10, 10, 380, 220, 2)
            .copies(1);

        let bytes = label.encode_zpl();
        let zpl_str = String::from_utf8_lossy(&bytes);

        assert!(zpl_str.starts_with("^XA\n"));
        assert!(zpl_str.contains("^PW400\n"));
        assert!(zpl_str.contains("^LL240\n"));
        assert!(zpl_str.contains("^FO20,20^A0N,24,24^FDORGANIC COFFEE^FS\n"));
        assert!(zpl_str.contains("^FO20,80^BCN,50,Y,N,N^FDITEM-1042^FS\n"));
        assert!(zpl_str.contains("^FO280,80^BQN,2,4,M^FDMA,https://example.com/item/1042^FS\n"));
        assert!(zpl_str.contains("^FO10,10^GB380,220,2,B,0^FS\n"));
        assert!(zpl_str.contains("^PQ1\n"));
        assert!(zpl_str.ends_with("^XZ\n"));
    }

    #[test]
    fn test_label_svg_preview() {
        let label = Label::new(50.0, 30.0)
            .text(20, 20, "3", 1, "ORGANIC COFFEE")
            .barcode(20, 80, BarcodeType::Code128, 50, "ITEM-1042")
            .qr(280, 80, 4, "https://example.com")
            .box_outline(10, 10, 380, 220, 2);

        let svg = label.render_svg();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains(r#"viewBox="0 0 400 240""#));
        assert!(svg.contains("ORGANIC COFFEE"));
        assert!(svg.contains("ITEM-1042"));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn test_unit_conversions_and_dpi() {
        let label_inch = Label::with_unit(4.0, 6.0, Unit::Inch).dpi(300);
        // 4 inches * 300 DPI = 1200 dots
        // 6 inches * 300 DPI = 1800 dots
        assert_eq!(label_inch.width_dots(), 1200);
        assert_eq!(label_inch.height_dots(), 1800);
    }

    #[test]
    fn test_speed_density_direction() {
        let label = Label::new(40.0, 30.0)
            .direction(Direction::Inverted)
            .speed(4)
            .density(12)
            .line(0, 50, 300, 4)
            .reverse(0, 60, 300, 30);

        let tspl = String::from_utf8(label.encode_tspl()).unwrap();
        assert!(tspl.contains("DIRECTION 0\r\n"));
        assert!(tspl.contains("SPEED 4\r\n"));
        assert!(tspl.contains("DENSITY 12\r\n"));
        assert!(tspl.contains("BAR 0,50,300,4\r\n"));
        assert!(tspl.contains("REVERSE 0,60,300,30\r\n"));

        let zpl = String::from_utf8(label.encode_zpl()).unwrap();
        assert!(zpl.contains("^POI\n"));
        assert!(zpl.contains("^PR4\n"));
        assert!(zpl.contains("~SD12\n"));
        assert!(zpl.contains("^FO0,50^GB300,4,4,B,0^FS\n"));
    }
}
