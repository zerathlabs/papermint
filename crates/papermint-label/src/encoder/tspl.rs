//! TSPL-II (TSC Printer Language) Dialect Encoder.
//!
//! Compatible with Xprinter (XP-365B, XP-420B), TSC, Rongta, Munbyn, Gprinter,
//! and standard desktop thermal sticker printers.

use crate::command::{Direction, LabelCommand, QrErrorCorrection};
use crate::label::Label;

/// Encodes a [`Label`] canvas into raw TSPL wire bytes.
pub fn encode_tspl(label: &Label) -> Vec<u8> {
    let mut out = Vec::new();

    // Dimensions and gap sensor
    out.extend_from_slice(
        format!(
            "SIZE {:.2} mm, {:.2} mm\r\n",
            label.width_mm, label.height_mm
        )
        .as_bytes(),
    );

    out.extend_from_slice(
        format!(
            "GAP {:.2} mm, {:.2} mm\r\n",
            label.gap_mm, label.gap_offset_mm
        )
        .as_bytes(),
    );

    // Print orientation
    let dir_code = match label.direction {
        Direction::Normal => 1,
        Direction::Inverted => 0,
    };
    out.extend_from_slice(format!("DIRECTION {}\r\n", dir_code).as_bytes());

    // Home reference origin
    if label.reference_x > 0 || label.reference_y > 0 {
        out.extend_from_slice(
            format!("REFERENCE {},{}\r\n", label.reference_x, label.reference_y).as_bytes(),
        );
    }

    // Printhead speed and darkness
    if let Some(speed) = label.speed {
        out.extend_from_slice(format!("SPEED {}\r\n", speed).as_bytes());
    }
    if let Some(density) = label.density {
        out.extend_from_slice(format!("DENSITY {}\r\n", density).as_bytes());
    }

    // Clear buffer before issuing drawing commands
    out.extend_from_slice(b"CLS\r\n");

    // Canvas element commands
    for cmd in &label.commands {
        match cmd {
            LabelCommand::Text {
                x,
                y,
                font,
                rotation,
                x_multi,
                y_multi,
                content,
            } => {
                let rot = rotation.tspl_code();
                let escaped = content.replace('"', "\\\"");
                out.extend_from_slice(
                    format!(
                        "TEXT {},{},\"{}\",{},{},{},\"{}\"\r\n",
                        x, y, font, rot, x_multi, y_multi, escaped
                    )
                    .as_bytes(),
                );
            }
            LabelCommand::Barcode {
                x,
                y,
                barcode_type,
                height,
                human_readable,
                rotation,
                narrow,
                wide,
                content,
            } => {
                let readable_code = if *human_readable { 2 } else { 0 }; // 2 = centered below
                let rot = rotation.tspl_code();
                let b_type = barcode_type.tspl_name();
                out.extend_from_slice(
                    format!(
                        "BARCODE {},{},\"{}\",{},{},{},{},{},\"{}\"\r\n",
                        x, y, b_type, height, readable_code, rot, narrow, wide, content
                    )
                    .as_bytes(),
                );
            }
            LabelCommand::QrCode {
                x,
                y,
                cell_size,
                ecc,
                rotation,
                content,
            } => {
                let ecc_char = match ecc {
                    QrErrorCorrection::L => 'L',
                    QrErrorCorrection::M => 'M',
                    QrErrorCorrection::Q => 'Q',
                    QrErrorCorrection::H => 'H',
                };
                let rot = rotation.tspl_code();
                out.extend_from_slice(
                    format!(
                        "QRCODE {},{},{},{},A,{},\"{}\"\r\n",
                        x, y, ecc_char, cell_size, rot, content
                    )
                    .as_bytes(),
                );
            }
            LabelCommand::Box {
                x,
                y,
                width,
                height,
                thickness,
            } => {
                let x_end = x + width;
                let y_end = y + height;
                out.extend_from_slice(
                    format!("BOX {},{},{},{},{}\r\n", x, y, x_end, y_end, thickness).as_bytes(),
                );
            }
            LabelCommand::Line {
                x,
                y,
                width,
                height,
            } => {
                out.extend_from_slice(
                    format!("BAR {},{},{},{}\r\n", x, y, width, height).as_bytes(),
                );
            }
            LabelCommand::Reverse {
                x,
                y,
                width,
                height,
            } => {
                out.extend_from_slice(
                    format!("REVERSE {},{},{},{}\r\n", x, y, width, height).as_bytes(),
                );
            }
            LabelCommand::Raw(bytes) => {
                out.extend_from_slice(bytes);
            }
        }
    }

    // Print execution and copies
    out.extend_from_slice(format!("PRINT {},1\r\n", label.copies.max(1)).as_bytes());

    out
}
