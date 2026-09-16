//! Fluent 2D Label Canvas Builder.

use serde::{Deserialize, Serialize};

use crate::command::{BarcodeType, Direction, LabelCommand, QrErrorCorrection, Rotation, Unit};
use crate::encoder::{encode_tspl, encode_zpl};
use crate::preview::render_svg;

/// A 2D thermal label sticker canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Label {
    /// Width of the label in millimeters.
    pub width_mm: f32,
    /// Height of the label in millimeters.
    pub height_mm: f32,
    /// Distance of the die-cut gap or black mark sensor in millimeters.
    pub gap_mm: f32,
    /// Gap sensor offset in millimeters.
    pub gap_offset_mm: f32,
    /// Resolution of the label printhead (default 203 DPI = 8 dots/mm).
    pub dpi: u32,
    /// Print orientation.
    pub direction: Direction,
    /// Reference origin coordinate X in dots.
    pub reference_x: u32,
    /// Reference origin coordinate Y in dots.
    pub reference_y: u32,
    /// Print speed index.
    pub speed: Option<u8>,
    /// Thermal printhead darkness / density (typically 1 to 15).
    pub density: Option<u8>,
    /// Number of copies to print.
    pub copies: u32,
    /// Sequence of drawing commands on the 2D canvas.
    pub commands: Vec<LabelCommand>,
}

impl Label {
    /// Creates a new label canvas with width and height in millimeters.
    ///
    /// Defaults to 203 DPI (8 dots/mm), 2mm gap sensor, and 1 copy.
    pub fn new(width_mm: f32, height_mm: f32) -> Self {
        Self {
            width_mm,
            height_mm,
            gap_mm: 2.0,
            gap_offset_mm: 0.0,
            dpi: 203,
            direction: Direction::Normal,
            reference_x: 0,
            reference_y: 0,
            speed: None,
            density: None,
            copies: 1,
            commands: Vec::new(),
        }
    }

    /// Creates a label canvas specifying unit of measurement.
    pub fn with_unit(width: f32, height: f32, unit: Unit) -> Self {
        let (w_mm, h_mm) = match unit {
            Unit::Mm => (width, height),
            Unit::Dots => (width / (203.0 / 25.4), height / (203.0 / 25.4)),
            Unit::Inch => (width * 25.4, height * 25.4),
        };
        Self::new(w_mm, h_mm)
    }
}

impl Default for Label {
    fn default() -> Self {
        Self::new(50.0, 30.0)
    }
}

impl Label {
    /// Sets printhead resolution in Dots Per Inch (e.g. 203 or 300 DPI).
    pub fn dpi(mut self, dpi: u32) -> Self {
        self.dpi = dpi;
        self
    }

    /// Sets gap sensor distance and offset in millimeters.
    pub fn gap(mut self, gap_mm: f32, offset_mm: f32) -> Self {
        self.gap_mm = gap_mm;
        self.gap_offset_mm = offset_mm;
        self
    }

    /// Sets print direction (Normal or Inverted 180°).
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Sets home reference origin coordinate (x, y) in dots.
    pub fn reference(mut self, x: u32, y: u32) -> Self {
        self.reference_x = x;
        self.reference_y = y;
        self
    }

    /// Sets printhead speed.
    pub fn speed(mut self, speed: u8) -> Self {
        self.speed = Some(speed);
        self
    }

    /// Sets printhead darkness / density (typically 1 to 15).
    pub fn density(mut self, density: u8) -> Self {
        self.density = Some(density);
        self
    }

    /// Sets number of copies to print.
    pub fn copies(mut self, count: u32) -> Self {
        self.copies = count.max(1);
        self
    }

    /// Appends standard text at (x, y) dot coordinates with size multiplier.
    pub fn text(
        self,
        x: u32,
        y: u32,
        font: impl Into<String>,
        size: u8,
        content: impl Into<String>,
    ) -> Self {
        self.text_ext(x, y, font, Rotation::Deg0, size, size, content)
    }

    /// Appends text with full rotation and independent X/Y scaling.
    #[allow(clippy::too_many_arguments)]
    pub fn text_ext(
        mut self,
        x: u32,
        y: u32,
        font: impl Into<String>,
        rotation: Rotation,
        x_multi: u8,
        y_multi: u8,
        content: impl Into<String>,
    ) -> Self {
        self.commands.push(LabelCommand::Text {
            x,
            y,
            font: font.into(),
            rotation,
            x_multi: x_multi.max(1),
            y_multi: y_multi.max(1),
            content: content.into(),
        });
        self
    }

    /// Appends a 1D barcode with centered human-readable text.
    pub fn barcode(
        self,
        x: u32,
        y: u32,
        barcode_type: BarcodeType,
        height: u32,
        content: impl Into<String>,
    ) -> Self {
        self.barcode_ext(
            x,
            y,
            barcode_type,
            height,
            true,
            Rotation::Deg0,
            2,
            4,
            content,
        )
    }

    /// Appends a 1D barcode with full parameter customization.
    #[allow(clippy::too_many_arguments)]
    pub fn barcode_ext(
        mut self,
        x: u32,
        y: u32,
        barcode_type: BarcodeType,
        height: u32,
        human_readable: bool,
        rotation: Rotation,
        narrow: u8,
        wide: u8,
        content: impl Into<String>,
    ) -> Self {
        self.commands.push(LabelCommand::Barcode {
            x,
            y,
            barcode_type,
            height,
            human_readable,
            rotation,
            narrow,
            wide,
            content: content.into(),
        });
        self
    }

    /// Appends a 2D QR Code.
    pub fn qr(self, x: u32, y: u32, cell_size: u8, content: impl Into<String>) -> Self {
        self.qr_ext(
            x,
            y,
            cell_size,
            QrErrorCorrection::M,
            Rotation::Deg0,
            content,
        )
    }

    /// Appends a 2D QR Code with error correction and rotation.
    pub fn qr_ext(
        mut self,
        x: u32,
        y: u32,
        cell_size: u8,
        ecc: QrErrorCorrection,
        rotation: Rotation,
        content: impl Into<String>,
    ) -> Self {
        self.commands.push(LabelCommand::QrCode {
            x,
            y,
            cell_size: cell_size.max(1),
            ecc,
            rotation,
            content: content.into(),
        });
        self
    }

    /// Appends a bounding rectangle / box outline.
    pub fn box_outline(mut self, x: u32, y: u32, width: u32, height: u32, thickness: u32) -> Self {
        self.commands.push(LabelCommand::Box {
            x,
            y,
            width,
            height,
            thickness: thickness.max(1),
        });
        self
    }

    /// Appends a solid line / filled bar.
    pub fn line(mut self, x: u32, y: u32, width: u32, height: u32) -> Self {
        self.commands.push(LabelCommand::Line {
            x,
            y,
            width,
            height,
        });
        self
    }

    /// Inverts the colors of a rectangular region.
    pub fn reverse(mut self, x: u32, y: u32, width: u32, height: u32) -> Self {
        self.commands.push(LabelCommand::Reverse {
            x,
            y,
            width,
            height,
        });
        self
    }

    /// Appends raw dialect bytes directly.
    pub fn raw(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        self.commands.push(LabelCommand::Raw(bytes.into()));
        self
    }

    /// Calculates printable canvas width in dots.
    pub fn width_dots(&self) -> u32 {
        (self.width_mm * (self.dpi as f32 / 25.4)).round() as u32
    }

    /// Calculates printable canvas height in dots.
    pub fn height_dots(&self) -> u32 {
        (self.height_mm * (self.dpi as f32 / 25.4)).round() as u32
    }

    /// Encodes the label canvas into TSPL-II wire bytes (TSC, Xprinter, Rongta, Munbyn).
    pub fn encode_tspl(&self) -> Vec<u8> {
        encode_tspl(self)
    }

    /// Encodes the label canvas into ZPL II wire bytes (Zebra, Citizen, Godex).
    pub fn encode_zpl(&self) -> Vec<u8> {
        encode_zpl(self)
    }

    /// Renders an exact vector SVG preview string of the physical sticker.
    pub fn render_svg(&self) -> String {
        render_svg(self)
    }
}
