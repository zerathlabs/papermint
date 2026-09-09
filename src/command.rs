// ── command.rs — The Intermediate Representation ──
//
// This is the key differentiator of papermint. Instead of coupling formatting
// directly to vendor-specific bytes, we define an abstract Command enum.
//
//   Layout Engine → Vec<Command> → Dialect Encoder → Vec<u8> → Transport
//
// A Vec<Command> can be serialized, stored, replayed, or re-encoded for a
// different printer vendor without touching the original receipt logic.

use crate::charset::InternationalCharset;
use crate::codepage::CodePage;

/// Text alignment on the receipt paper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

/// Paper cut mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CutMode {
    Full,
    Partial,
}

/// Underline weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnderlineMode {
    Off,
    Single,
    Double,
}

/// Font selection (most printers support at least A and B).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontFamily {
    A,
    B,
}

/// Cash drawer pin selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawerPin {
    Pin2,
    Pin5,
}

/// Barcode symbology.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarcodeSystem {
    UpcA,
    UpcE,
    Ean13,
    Ean8,
    Code39,
    Code128,
    Itf,
    Codabar,
    Code93,
}

/// Barcode data with configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarcodeData {
    pub system: BarcodeSystem,
    pub data: String,
    pub width: u8,
    pub height: u8,
}

/// QR code error correction level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QrCorrectionLevel {
    L,
    M,
    Q,
    H,
}

/// QR code data with configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrData {
    pub data: String,
    pub model: u8,
    pub cell_size: u8,
    pub correction: QrCorrectionLevel,
}

/// Raster image data for bitmap printing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    /// Raw monochrome pixel data, packed as 1 bit per pixel, MSB first.
    pub pixels: Vec<u8>,
}

/// Printhead density / dot darkness level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrintDensity {
    /// Light density (~85% energy). Best for high-sensitivity paper.
    Light,
    /// Standard density (100% default factory calibration).
    #[default]
    Normal,
    /// Dark density (~115% energy). Enhances optical contrast for barcodes and QR codes.
    Dark,
    /// High-contrast dark (~130% energy). Best for synthetic or low-sensitivity paper.
    HighContrast,
    /// Custom vendor-specific density value.
    Custom(u8),
}

/// Thermal printhead heating strobe and pulse timing parameters (`ESC 7 n1 n2 n3`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeatingParameters {
    /// Maximum dots heated simultaneously (unit: 8 dots, e.g. 7 = 56 dots).
    pub max_heating_dots: u8,
    /// Heating pulse duration in 10µs units (e.g. 80 = 800µs).
    pub heating_time: u8,
    /// Interval between heating pulses in 10µs units (e.g. 2 = 20µs).
    pub heating_interval: u8,
}

impl Default for HeatingParameters {
    fn default() -> Self {
        Self {
            max_heating_dots: 7, // 56 dots
            heating_time: 80,    // 800 µs
            heating_interval: 2, // 20 µs
        }
    }
}

/// The intermediate representation between layout and encoding.
///
/// Each variant maps to a single printer instruction. The layout engine
/// produces `Vec<Command>`, and the dialect encoder translates each
/// `Command` into vendor-specific bytes.
///
/// # Design
///
/// Commands are deliberately simple and flat. Compound operations like
/// "print a bold, centered header" are expressed as a sequence:
///
/// ```text
/// [Align(Center), Bold(true), Text("HELLO"), Bold(false), Align(Left)]
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Initialize/reset the printer to default state.
    Init,

    /// Append text content to the print buffer.
    Text(String),

    /// Feed paper by the given number of lines.
    Feed(u8),

    /// Cut the paper.
    Cut(CutMode),

    /// Set text alignment.
    Align(Alignment),

    /// Enable or disable bold emphasis.
    Bold(bool),

    /// Set underline mode.
    Underline(UnderlineMode),

    /// Enable or disable inverse (white-on-black) printing.
    Invert(bool),

    /// Enable or disable double-height characters.
    DoubleHeight(bool),

    /// Enable or disable double-width characters.
    DoubleWidth(bool),

    /// Select the font family.
    Font(FontFamily),

    /// Print a barcode.
    Barcode(BarcodeData),

    /// Print a QR code.
    QrCode(QrData),

    /// Print a raster image.
    Image(ImageData),

    /// Open the cash drawer.
    DrawerKick(DrawerPin),

    /// Sound the printer buzzer.
    Beep {
        /// Number of beep pulses (1–9).
        count: u8,
        /// Duration of each beep (1–9, printer-defined unit).
        duration: u8,
    },

    /// Set line spacing. `None` restores the hardware default (typically 1/6 inch).
    /// `Some(dots)` specifies vertical line pitch in dots.
    LineSpacing(Option<u8>),

    /// Enable or disable upside-down (180° rotated) printing.
    UpsideDown(bool),

    /// Select character code page table (e.g., [`CodePage::Wpc1252`], [`CodePage::Pc850`]).
    CodePage(CodePage),

    /// Select international character set (e.g., [`InternationalCharset::Uk`], [`InternationalCharset::France`]).
    InternationalCharset(InternationalCharset),

    /// Set printhead dot darkness density.
    PrintDensity(PrintDensity),

    /// Configure thermal heating pulse and strobe timing parameters (`ESC 7 n1 n2 n3`).
    HeatingParameters(HeatingParameters),

    /// Inject raw bytes directly (escape hatch for vendor-specific commands).
    Raw(Vec<u8>),
}
