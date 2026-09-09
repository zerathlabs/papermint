//! Fluent builder for composing printable receipts.

use crate::charset::InternationalCharset;
use crate::codepage::CodePage;
use crate::command::{
    Alignment, BarcodeData, BarcodeSystem, Command, CutMode, DrawerPin, FontFamily,
    HeatingParameters, ImageData, PrintDensity, QrCorrectionLevel, QrData, UnderlineMode,
};
use crate::layout::column::{
    PaperWidth, TableColumn, format_table_row, format_three_column, format_two_column,
};

/// A fluent receipt document builder that generates a [`Vec<Command>`].
///
/// # Example
///
/// ```rust
/// use papermint::{Receipt, PaperWidth, Alignment};
///
/// let receipt = Receipt::new(PaperWidth::Mm80)
///     .init()
///     .center()
///     .bold(true)
///     .text_ln("MINT BISTRO")
///     .bold(false)
///     .text_ln("123 Main Street")
///     .divider('-')
///     .two_column("Double Cheeseburger", "$12.50")
///     .two_column("Large Truffle Fries", "$5.50")
///     .divider('=')
///     .bold(true)
///     .two_column("TOTAL", "$18.00")
///     .bold(false)
///     .feed(2)
///     .center()
///     .qr("https://example.com/receipt/1042")
///     .feed(3)
///     .cut_full();
///
/// assert!(!receipt.commands().is_empty());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Receipt {
    paper_width: PaperWidth,
    commands: Vec<Command>,
    active_columns: Option<Vec<TableColumn>>,
}

impl Default for Receipt {
    fn default() -> Self {
        Self::new(PaperWidth::Mm80)
    }
}

impl Receipt {
    /// Creates a new empty [`Receipt`] configured with the specified [`PaperWidth`].
    #[must_use]
    pub fn new(paper_width: PaperWidth) -> Self {
        Self {
            paper_width,
            commands: Vec::with_capacity(32),
            active_columns: None,
        }
    }

    /// Returns the configured [`PaperWidth`].
    #[must_use]
    pub const fn paper_width(&self) -> PaperWidth {
        self.paper_width
    }

    /// Returns the character column width for this receipt.
    #[must_use]
    pub const fn columns(&self) -> usize {
        self.paper_width.columns()
    }

    /// Returns a reference to the sequence of accumulated [`Command`]s.
    #[must_use]
    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    /// Consumes the builder and returns the underlying [`Vec<Command>`].
    #[must_use]
    pub fn into_commands(self) -> Vec<Command> {
        self.commands
    }

    /// Resets the printer hardware to default state.
    #[must_use]
    pub fn init(mut self) -> Self {
        self.commands.push(Command::Init);
        self
    }

    /// Appends raw text without a trailing newline.
    #[must_use]
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.commands.push(Command::Text(text.into()));
        self
    }

    /// Appends text followed by a newline (`\n`).
    #[must_use]
    pub fn text_ln(mut self, text: impl Into<String>) -> Self {
        let mut s = text.into();
        s.push('\n');
        self.commands.push(Command::Text(s));
        self
    }

    /// Feeds paper by the specified number of lines.
    #[must_use]
    pub fn feed(mut self, lines: u8) -> Self {
        self.commands.push(Command::Feed(lines));
        self
    }

    /// Feeds paper by a single line.
    #[must_use]
    pub fn feed_ln(self) -> Self {
        self.feed(1)
    }

    /// Sets the text alignment.
    #[must_use]
    pub fn align(mut self, alignment: Alignment) -> Self {
        self.commands.push(Command::Align(alignment));
        self
    }

    /// Convenience for `.align(Alignment::Left)`.
    #[must_use]
    pub fn left(self) -> Self {
        self.align(Alignment::Left)
    }

    /// Convenience for `.align(Alignment::Center)`.
    #[must_use]
    pub fn center(self) -> Self {
        self.align(Alignment::Center)
    }

    /// Convenience for `.align(Alignment::Right)`.
    #[must_use]
    pub fn right(self) -> Self {
        self.align(Alignment::Right)
    }

    /// Enables or disables bold text emphasis.
    #[must_use]
    pub fn bold(mut self, enable: bool) -> Self {
        self.commands.push(Command::Bold(enable));
        self
    }

    /// Sets the underline style.
    #[must_use]
    pub fn underline(mut self, mode: UnderlineMode) -> Self {
        self.commands.push(Command::Underline(mode));
        self
    }

    /// Enables single-line underline.
    #[must_use]
    pub fn underline_on(self) -> Self {
        self.underline(UnderlineMode::Single)
    }

    /// Disables underline.
    #[must_use]
    pub fn underline_off(self) -> Self {
        self.underline(UnderlineMode::Off)
    }

    /// Enables or disables inverse (white-on-black) printing.
    #[must_use]
    pub fn invert(mut self, enable: bool) -> Self {
        self.commands.push(Command::Invert(enable));
        self
    }

    /// Enables or disables double-height characters.
    #[must_use]
    pub fn double_height(mut self, enable: bool) -> Self {
        self.commands.push(Command::DoubleHeight(enable));
        self
    }

    /// Enables or disables double-width characters.
    #[must_use]
    pub fn double_width(mut self, enable: bool) -> Self {
        self.commands.push(Command::DoubleWidth(enable));
        self
    }

    /// Convenience to set both double height and double width.
    #[must_use]
    pub fn double_size(self, enable: bool) -> Self {
        self.double_height(enable).double_width(enable)
    }

    /// Selects the font family (e.g. Font A or Font B).
    #[must_use]
    pub fn font(mut self, font: FontFamily) -> Self {
        self.commands.push(Command::Font(font));
        self
    }

    /// Prints a full horizontal divider rule spanning the entire paper width.
    #[must_use]
    pub fn divider(self, ch: char) -> Self {
        let cols = self.columns();
        let rule: String = std::iter::repeat_n(ch, cols).collect();
        self.text_ln(rule)
    }

    /// Prints a 2-column row with left text aligned left and right text aligned right.
    #[must_use]
    pub fn two_column(self, left: impl AsRef<str>, right: impl AsRef<str>) -> Self {
        let cols = self.columns();
        let formatted = format_two_column(left.as_ref(), right.as_ref(), cols);
        self.text_ln(formatted)
    }

    /// Prints a 3-column row (left, center, right).
    #[must_use]
    pub fn three_column(
        self,
        left: impl AsRef<str>,
        center: impl AsRef<str>,
        right: impl AsRef<str>,
    ) -> Self {
        let cols = self.columns();
        let formatted = format_three_column(left.as_ref(), center.as_ref(), right.as_ref(), cols);
        self.text_ln(formatted)
    }

    /// Prints an N-column table row with automatic word-wrapping and synchronized vertical row height.
    ///
    /// Long cell text wraps within its allocated column width while preserving
    /// strict columnar alignment across adjacent columns on the receipt.
    #[must_use]
    pub fn table_row<S: AsRef<str>>(mut self, cells: &[S], columns: &[TableColumn]) -> Self {
        let str_cells: Vec<&str> = cells.iter().map(|s| s.as_ref()).collect();
        let lines = format_table_row(&str_cells, columns, self.columns());
        for line in lines {
            self = self.text_ln(line);
        }
        self
    }

    /// Prints multiple N-column table rows.
    #[must_use]
    pub fn table<S: AsRef<str>>(mut self, columns: &[TableColumn], rows: &[&[S]]) -> Self {
        for row in rows {
            self = self.table_row(row, columns);
        }
        self
    }

    /// Configures the active table columns for subsequent calls to [`.row(...)`](Self::row)
    /// or [`.header(...)`](Self::header).
    #[must_use]
    pub fn set_columns(mut self, columns: &[TableColumn]) -> Self {
        self.active_columns = Some(columns.to_vec());
        self
    }

    /// Configures the active table columns and prints a bold table header row.
    ///
    /// Subsequent calls to [`.row(...)`](Self::row) will automatically format against
    /// these columns without needing to re-specify the column slice on every row.
    #[must_use]
    pub fn table_header<S: AsRef<str>>(mut self, headers: &[S], columns: &[TableColumn]) -> Self {
        self.active_columns = Some(columns.to_vec());
        self = self.bold(true).table_row(headers, columns).bold(false);
        self
    }

    /// Prints a bold header row using the currently configured active columns.
    #[must_use]
    pub fn header<S: AsRef<str>>(mut self, headers: &[S]) -> Self {
        self = self.bold(true).row(headers).bold(false);
        self
    }

    /// Returns a slice of the active table columns, if configured.
    #[must_use]
    pub fn active_columns(&self) -> Option<&[TableColumn]> {
        self.active_columns.as_deref()
    }

    /// Clears any active table columns configured on this receipt builder.
    #[must_use]
    pub fn clear_columns(mut self) -> Self {
        self.active_columns = None;
        self
    }

    /// Prints an N-column table row using the active table columns configured via
    /// [`.table_header(...)`](Self::table_header) or [`.set_columns(...)`](Self::set_columns).
    ///
    /// Long cell text wraps automatically at word boundaries while synchronizing
    /// multi-line heights across adjacent columns.
    #[must_use]
    pub fn row<S: AsRef<str>>(mut self, cells: &[S]) -> Self {
        if let Some(ref cols) = self.active_columns {
            let str_cells: Vec<&str> = cells.iter().map(|s| s.as_ref()).collect();
            let lines = format_table_row(&str_cells, cols, self.columns());
            for line in lines {
                self = self.text_ln(line);
            }
            self
        } else {
            let joined = cells
                .iter()
                .map(|s| s.as_ref())
                .collect::<Vec<_>>()
                .join("  ");
            self.text_ln(joined)
        }
    }

    /// Prints a barcode with full custom parameters.
    #[must_use]
    pub fn barcode(mut self, barcode: BarcodeData) -> Self {
        self.commands.push(Command::Barcode(barcode));
        self
    }

    /// Convenience to print a Code128 barcode with default dimensions (width 2, height 64).
    #[must_use]
    pub fn barcode_128(self, data: impl Into<String>) -> Self {
        self.barcode(BarcodeData {
            system: BarcodeSystem::Code128,
            data: data.into(),
            width: 2,
            height: 64,
        })
    }

    /// Prints a QR code with full custom parameters.
    #[must_use]
    pub fn qr_code(mut self, qr: QrData) -> Self {
        self.commands.push(Command::QrCode(qr));
        self
    }

    /// Convenience to print a QR code with sensible defaults (Model 2, cell size 4, M correction).
    #[must_use]
    pub fn qr(self, data: impl Into<String>) -> Self {
        self.qr_code(QrData {
            data: data.into(),
            model: 2,
            cell_size: 4,
            correction: QrCorrectionLevel::M,
        })
    }

    /// Prints a monochrome raster image.
    #[must_use]
    pub fn image(mut self, image: ImageData) -> Self {
        self.commands.push(Command::Image(image));
        self
    }

    /// Loads an image file from disk, dithers it with Floyd-Steinberg error diffusion,
    /// auto-downscales to fit the paper width if necessary, and appends it to the receipt.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::PapermintError::Image`] if the file cannot be read or decoded.
    #[cfg(feature = "image")]
    pub fn image_from_path(self, path: impl AsRef<std::path::Path>) -> crate::error::Result<Self> {
        let max_w = self.paper_width.dots();
        self.image_from_path_with_options(
            path,
            Some(max_w),
            crate::image::DitherMode::FloydSteinberg,
        )
    }

    /// Loads an image file from disk with custom `max_width` and [`crate::image::DitherMode`].
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::PapermintError::Image`] if the file cannot be read or decoded.
    #[cfg(feature = "image")]
    pub fn image_from_path_with_options(
        self,
        path: impl AsRef<std::path::Path>,
        max_width: Option<u32>,
        mode: crate::image::DitherMode,
    ) -> crate::error::Result<Self> {
        let img = ImageData::from_path(path, max_width, mode)?;
        Ok(self.image(img))
    }

    /// Decodes an image from in-memory bytes (PNG, JPEG, etc.), dithers it with Floyd-Steinberg error diffusion,
    /// auto-downscales to fit the paper width if necessary, and appends it to the receipt.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::PapermintError::Image`] if decoding fails.
    #[cfg(feature = "image")]
    pub fn image_from_bytes(self, bytes: &[u8]) -> crate::error::Result<Self> {
        let max_w = self.paper_width.dots();
        self.image_from_bytes_with_options(
            bytes,
            Some(max_w),
            crate::image::DitherMode::FloydSteinberg,
        )
    }

    /// Decodes an image from in-memory bytes with custom `max_width` and [`crate::image::DitherMode`].
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::PapermintError::Image`] if decoding fails.
    #[cfg(feature = "image")]
    pub fn image_from_bytes_with_options(
        self,
        bytes: &[u8],
        max_width: Option<u32>,
        mode: crate::image::DitherMode,
    ) -> crate::error::Result<Self> {
        let img = ImageData::from_bytes(bytes, max_width, mode)?;
        Ok(self.image(img))
    }

    /// Cuts the paper with the specified [`CutMode`].
    #[must_use]
    pub fn cut(mut self, mode: CutMode) -> Self {
        self.commands.push(Command::Cut(mode));
        self
    }

    /// Full paper cut.
    #[must_use]
    pub fn cut_full(self) -> Self {
        self.cut(CutMode::Full)
    }

    /// Partial paper cut (leaves a perforated tag).
    #[must_use]
    pub fn cut_partial(self) -> Self {
        self.cut(CutMode::Partial)
    }

    /// Triggers the cash drawer kick on the specified pin.
    #[must_use]
    pub fn drawer_kick(mut self, pin: DrawerPin) -> Self {
        self.commands.push(Command::DrawerKick(pin));
        self
    }

    /// Opens the cash drawer on Pin 2 (standard RJ12 cash drawer port).
    #[must_use]
    pub fn open_drawer(self) -> Self {
        self.drawer_kick(DrawerPin::Pin2)
    }

    /// Sounds the printer internal buzzer.
    #[must_use]
    pub fn beep(mut self, count: u8, duration: u8) -> Self {
        self.commands.push(Command::Beep { count, duration });
        self
    }

    /// Sets line spacing in vertical motion dots, or resets to default if `None`.
    #[must_use]
    pub fn line_spacing(mut self, spacing: Option<u8>) -> Self {
        self.commands.push(Command::LineSpacing(spacing));
        self
    }

    /// Resets line spacing to the printer hardware default (typically 1/6 inch).
    #[must_use]
    pub fn default_line_spacing(self) -> Self {
        self.line_spacing(None)
    }

    /// Enables or disables upside-down (180° rotated) printing.
    #[must_use]
    pub fn upside_down(mut self, enable: bool) -> Self {
        self.commands.push(Command::UpsideDown(enable));
        self
    }

    /// Selects character code page table (e.g. [`CodePage::Wpc1252`], [`CodePage::Pc850`]).
    /// Also accepts raw `u8` index via `Into<CodePage>` for custom vendor code pages.
    #[must_use]
    pub fn code_page(mut self, page: impl Into<CodePage>) -> Self {
        self.commands.push(Command::CodePage(page.into()));
        self
    }

    /// Selects an international character set (e.g. [`InternationalCharset::Uk`], [`InternationalCharset::France`]).
    ///
    /// Modifies 12 standard ASCII punctuation positions (`#`, `$`, `@`, `[`, `\`, `]`, `^`, `` ` ``, `{`, `|`, `}`, `~`)
    /// to provide localized national currency and accented character symbols.
    #[must_use]
    pub fn international_charset(mut self, charset: impl Into<InternationalCharset>) -> Self {
        self.commands
            .push(Command::InternationalCharset(charset.into()));
        self
    }

    /// Convenience for `.international_charset(InternationalCharset::Usa)`.
    #[must_use]
    pub fn charset_usa(self) -> Self {
        self.international_charset(InternationalCharset::Usa)
    }

    /// Convenience for `.international_charset(InternationalCharset::Uk)`.
    #[must_use]
    pub fn charset_uk(self) -> Self {
        self.international_charset(InternationalCharset::Uk)
    }

    /// Convenience for `.international_charset(InternationalCharset::France)`.
    #[must_use]
    pub fn charset_france(self) -> Self {
        self.international_charset(InternationalCharset::France)
    }

    /// Convenience for `.international_charset(InternationalCharset::Germany)`.
    #[must_use]
    pub fn charset_germany(self) -> Self {
        self.international_charset(InternationalCharset::Germany)
    }

    /// Convenience for `.international_charset(InternationalCharset::SpainI)`.
    #[must_use]
    pub fn charset_spain(self) -> Self {
        self.international_charset(InternationalCharset::SpainI)
    }

    /// Convenience for `.international_charset(InternationalCharset::Italy)`.
    #[must_use]
    pub fn charset_italy(self) -> Self {
        self.international_charset(InternationalCharset::Italy)
    }

    /// Convenience for `.international_charset(InternationalCharset::Japan)`.
    #[must_use]
    pub fn charset_japan(self) -> Self {
        self.international_charset(InternationalCharset::Japan)
    }

    /// Convenience for `.international_charset(InternationalCharset::Arabia)`.
    #[must_use]
    pub fn charset_arabia(self) -> Self {
        self.international_charset(InternationalCharset::Arabia)
    }

    /// Sets the thermal printhead dot density.
    #[must_use]
    pub fn print_density(mut self, density: PrintDensity) -> Self {
        self.commands.push(Command::PrintDensity(density));
        self
    }

    /// Convenience for setting density to [`PrintDensity::Light`].
    #[must_use]
    pub fn density_light(self) -> Self {
        self.print_density(PrintDensity::Light)
    }

    /// Convenience for setting density to [`PrintDensity::Normal`].
    #[must_use]
    pub fn density_normal(self) -> Self {
        self.print_density(PrintDensity::Normal)
    }

    /// Convenience for setting density to [`PrintDensity::Dark`].
    #[must_use]
    pub fn density_dark(self) -> Self {
        self.print_density(PrintDensity::Dark)
    }

    /// Convenience for setting density to [`PrintDensity::HighContrast`].
    #[must_use]
    pub fn density_high_contrast(self) -> Self {
        self.print_density(PrintDensity::HighContrast)
    }

    /// Sets the thermal printhead heating strobe and interval parameters.
    #[must_use]
    pub fn heating_parameters(mut self, params: HeatingParameters) -> Self {
        self.commands.push(Command::HeatingParameters(params));
        self
    }

    /// Appends raw bytes directly to the printer command stream.
    #[must_use]
    pub fn raw(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        self.commands.push(Command::Raw(bytes.into()));
        self
    }
}
