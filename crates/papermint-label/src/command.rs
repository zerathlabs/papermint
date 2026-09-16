//! 2D Label Command definitions and primitives.

use serde::{Deserialize, Serialize};

/// Physical units of measurement for label dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unit {
    Mm,
    Dots,
    Inch,
}

/// Print direction / orientation on label feed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Direction {
    #[default]
    Normal,
    Inverted,
}

/// 90-degree orthogonal rotation angles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Rotation {
    #[default]
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl Rotation {
    /// Returns the integer degrees.
    pub fn degrees(&self) -> u32 {
        match self {
            Rotation::Deg0 => 0,
            Rotation::Deg90 => 90,
            Rotation::Deg180 => 180,
            Rotation::Deg270 => 270,
        }
    }

    /// Returns the TSPL rotation code (0, 90, 180, 270).
    pub fn tspl_code(&self) -> u32 {
        self.degrees()
    }

    /// Returns the ZPL orientation letter (N, R, I, B).
    pub fn zpl_code(&self) -> &'static str {
        match self {
            Rotation::Deg0 => "N",
            Rotation::Deg90 => "R",
            Rotation::Deg180 => "I",
            Rotation::Deg270 => "B",
        }
    }
}

/// 1D Barcode symbologies supported across label thermal printers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BarcodeType {
    Code128,
    Code39,
    Code93,
    Ean13,
    Ean8,
    UpcA,
    UpcE,
    Itf,
    Codabar,
}

impl BarcodeType {
    /// Returns the TSPL barcode name string.
    pub fn tspl_name(&self) -> &'static str {
        match self {
            BarcodeType::Code128 => "128",
            BarcodeType::Code39 => "39",
            BarcodeType::Code93 => "93",
            BarcodeType::Ean13 => "EAN13",
            BarcodeType::Ean8 => "EAN8",
            BarcodeType::UpcA => "UPCA",
            BarcodeType::UpcE => "UPCE",
            BarcodeType::Itf => "ITF",
            BarcodeType::Codabar => "CODA",
        }
    }

    /// Returns the ZPL barcode command prefix.
    pub fn zpl_command(&self) -> &'static str {
        match self {
            BarcodeType::Code128 => "BC",
            BarcodeType::Code39 => "B3",
            BarcodeType::Code93 => "BA",
            BarcodeType::Ean13 => "BE",
            BarcodeType::Ean8 => "B8",
            BarcodeType::UpcA => "BU",
            BarcodeType::UpcE => "B9",
            BarcodeType::Itf => "B2",
            BarcodeType::Codabar => "BK",
        }
    }
}

/// QR Code error correction level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QrErrorCorrection {
    /// Low (7% recovery)
    L,
    /// Medium (15% recovery)
    #[default]
    M,
    /// Quartile (25% recovery)
    Q,
    /// High (30% recovery)
    H,
}

impl QrErrorCorrection {
    pub fn as_char(&self) -> char {
        match self {
            QrErrorCorrection::L => 'L',
            QrErrorCorrection::M => 'M',
            QrErrorCorrection::Q => 'Q',
            QrErrorCorrection::H => 'H',
        }
    }
}

/// An individual 2D drawing instruction on the label canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LabelCommand {
    /// Alphanumeric or international text string at (x, y) dots.
    Text {
        x: u32,
        y: u32,
        font: String,
        rotation: Rotation,
        x_multi: u8,
        y_multi: u8,
        content: String,
    },
    /// 1D Linear Barcode.
    Barcode {
        x: u32,
        y: u32,
        barcode_type: BarcodeType,
        height: u32,
        human_readable: bool,
        rotation: Rotation,
        narrow: u8,
        wide: u8,
        content: String,
    },
    /// 2D QR Code.
    QrCode {
        x: u32,
        y: u32,
        cell_size: u8,
        ecc: QrErrorCorrection,
        rotation: Rotation,
        content: String,
    },
    /// Rectangle / bounding box outline with border thickness.
    Box {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        thickness: u32,
    },
    /// Solid filled horizontal or vertical line/bar.
    Line {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    /// Inverse color rectangular region (black becomes white, white becomes black).
    Reverse {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    /// Raw custom escape command string.
    Raw(Vec<u8>),
}
