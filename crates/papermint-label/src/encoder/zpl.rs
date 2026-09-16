//! ZPL II (Zebra Programming Language) Dialect Encoder.
//!
//! Compatible with Zebra desktop & industrial printers (ZD420, GX420d, ZT230, etc.),
//! Citizen, Godex, and global enterprise courier label printers.

use crate::command::{Direction, LabelCommand, QrErrorCorrection};
use crate::label::Label;

/// Encodes a [`Label`] canvas into raw ZPL II wire bytes.
pub fn encode_zpl(label: &Label) -> Vec<u8> {
    let mut out = String::new();

    // Format envelope header
    out.push_str("^XA\n");

    // Convert dimensions from mm to dots based on DPI
    let width_dots = label.width_dots();
    let height_dots = label.height_dots();

    out.push_str(&format!("^PW{}\n", width_dots));
    out.push_str(&format!("^LL{}\n", height_dots));

    // Reference home origin
    out.push_str(&format!("^LH{},{}\n", label.reference_x, label.reference_y));

    // Print Orientation
    match label.direction {
        Direction::Normal => out.push_str("^PON\n"),
        Direction::Inverted => out.push_str("^POI\n"),
    }

    // Speed & Slew darkness
    if let Some(speed) = label.speed {
        out.push_str(&format!("^PR{}\n", speed));
    }
    if let Some(density) = label.density {
        out.push_str(&format!("~SD{:02}\n", density));
    }

    // Canvas element commands
    for cmd in &label.commands {
        match cmd {
            LabelCommand::Text {
                x,
                y,
                font: _,
                rotation,
                x_multi,
                y_multi,
                content,
            } => {
                let rot = rotation.zpl_code();
                // Height and width scaled by multiplier (base ~24 dots for standard 203 DPI)
                let h = 24 * (*y_multi as u32).max(1);
                let w = 24 * (*x_multi as u32).max(1);
                out.push_str(&format!(
                    "^FO{},{}^A0{},{},{}^FD{}^FS\n",
                    x, y, rot, h, w, content
                ));
            }
            LabelCommand::Barcode {
                x,
                y,
                barcode_type,
                height,
                human_readable,
                rotation,
                narrow: _,
                wide: _,
                content,
            } => {
                let rot = rotation.zpl_code();
                let print_line = if *human_readable { "Y" } else { "N" };
                let cmd_prefix = barcode_type.zpl_command();
                out.push_str(&format!(
                    "^FO{},{}^{}{},{},{},N,N^FD{}^FS\n",
                    x, y, cmd_prefix, rot, height, print_line, content
                ));
            }
            LabelCommand::QrCode {
                x,
                y,
                cell_size,
                ecc,
                rotation,
                content,
            } => {
                let rot = rotation.zpl_code();
                let mag = (*cell_size).clamp(1, 10);
                let ecc_char = match ecc {
                    QrErrorCorrection::L => 'L',
                    QrErrorCorrection::M => 'M',
                    QrErrorCorrection::Q => 'Q',
                    QrErrorCorrection::H => 'H',
                };
                // In ZPL: ^BQa,b,c,d^FD<ecc>A,<data>^FS
                out.push_str(&format!(
                    "^FO{},{}^BQ{},2,{},{}^FD{}A,{}^FS\n",
                    x, y, rot, mag, ecc_char, ecc_char, content
                ));
            }
            LabelCommand::Box {
                x,
                y,
                width,
                height,
                thickness,
            } => {
                out.push_str(&format!(
                    "^FO{},{}^GB{},{},{},B,0^FS\n",
                    x, y, width, height, thickness
                ));
            }
            LabelCommand::Line {
                x,
                y,
                width,
                height,
            } => {
                let thickness = (*width).min(*height).max(1);
                out.push_str(&format!(
                    "^FO{},{}^GB{},{},{},B,0^FS\n",
                    x, y, width, height, thickness
                ));
            }
            LabelCommand::Reverse {
                x,
                y,
                width,
                height,
            } => {
                out.push_str(&format!(
                    "^FO{},{}^FR^GB{},{},{},B,0^FS\n",
                    x, y, width, height, height
                ));
            }
            LabelCommand::Raw(bytes) => {
                if let Ok(s) = std::str::from_utf8(bytes) {
                    out.push_str(s);
                }
            }
        }
    }

    // Quantity and envelope footer
    out.push_str(&format!("^PQ{}\n", label.copies.max(1)));
    out.push_str("^XZ\n");

    out.into_bytes()
}
