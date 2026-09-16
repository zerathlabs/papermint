use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use papermint::label::{BarcodeType, Direction, Label, QrErrorCorrection, Rotation, Unit};

/// A 2D label canvas builder for adhesive stickers, tags, and barcodes.
#[napi(js_name = "Label")]
pub struct JsLabel {
    pub(crate) inner: Label,
}

#[napi]
impl JsLabel {
    /// Creates a new label canvas.
    ///
    /// @param width - Physical width (e.g. 50.0).
    /// @param height - Physical height (e.g. 30.0).
    /// @param unit - Unit: "mm" (default), "inch", or "dots".
    #[napi(constructor)]
    pub fn new(width: f64, height: f64, unit: Option<String>) -> Self {
        let u = match unit.as_deref() {
            Some("inch" | "in") => Unit::Inch,
            Some("dots" | "dot") => Unit::Dots,
            _ => Unit::Mm,
        };
        Self {
            inner: Label::with_unit(width as f32, height as f32, u),
        }
    }

    /// Sets printhead resolution in dots per inch (default: 203 DPI).
    #[napi]
    pub fn dpi(&mut self, dpi: u32) -> &Self {
        self.inner = std::mem::take(&mut self.inner).dpi(dpi);
        self
    }

    /// Sets gap distance and optional offset between labels in millimeters.
    #[napi]
    pub fn gap(&mut self, gap_mm: f64, offset_mm: Option<f64>) -> &Self {
        self.inner =
            std::mem::take(&mut self.inner).gap(gap_mm as f32, offset_mm.unwrap_or(0.0) as f32);
        self
    }

    /// Sets print speed in inches per second (e.g. 2 to 6).
    #[napi]
    pub fn speed(&mut self, speed: u32) -> &Self {
        self.inner = std::mem::take(&mut self.inner).speed(speed as u8);
        self
    }

    /// Sets print darkness / density (0 to 15).
    #[napi]
    pub fn density(&mut self, density: u32) -> &Self {
        self.inner = std::mem::take(&mut self.inner).density(density as u8);
        self
    }

    /// Sets print direction: false = Normal, true = Inverted.
    #[napi]
    pub fn direction(&mut self, inverted: Option<bool>) -> &Self {
        let dir = if inverted.unwrap_or(false) {
            Direction::Inverted
        } else {
            Direction::Normal
        };
        self.inner = std::mem::take(&mut self.inner).direction(dir);
        self
    }

    /// Appends text at absolute (x, y) dot coordinates.
    #[napi]
    pub fn text(
        &mut self,
        x: u32,
        y: u32,
        content: String,
        x_mult: Option<u32>,
        y_mult: Option<u32>,
    ) -> &Self {
        let xm = x_mult.unwrap_or(1) as u8;
        let ym = y_mult.unwrap_or(1) as u8;
        self.inner =
            std::mem::take(&mut self.inner).text_ext(x, y, "2", Rotation::Deg0, xm, ym, content);
        self
    }

    /// Appends a 1D barcode at (x, y).
    ///
    /// @param barcodeType - "code128" (default), "code39", "ean13", "ean8", "upca", "itf".
    #[napi]
    pub fn barcode(
        &mut self,
        x: u32,
        y: u32,
        content: String,
        barcode_type: Option<String>,
        height: Option<u32>,
        readable: Option<bool>,
    ) -> &Self {
        let btype = match barcode_type.as_deref() {
            Some("code39") => BarcodeType::Code39,
            Some("ean13") => BarcodeType::Ean13,
            Some("ean8") => BarcodeType::Ean8,
            Some("upca") => BarcodeType::UpcA,
            Some("itf") => BarcodeType::Itf,
            _ => BarcodeType::Code128,
        };
        self.inner = std::mem::take(&mut self.inner).barcode_ext(
            x,
            y,
            btype,
            height.unwrap_or(48),
            readable.unwrap_or(true),
            Rotation::Deg0,
            2,
            4,
            content,
        );
        self
    }

    /// Appends a 2D QR code at (x, y).
    #[napi]
    pub fn qr(
        &mut self,
        x: u32,
        y: u32,
        content: String,
        cell_width: Option<u32>,
        ecc: Option<String>,
    ) -> &Self {
        let ec = match ecc.as_deref() {
            Some("L" | "l") => QrErrorCorrection::L,
            Some("Q" | "q") => QrErrorCorrection::Q,
            Some("H" | "h") => QrErrorCorrection::H,
            _ => QrErrorCorrection::M,
        };
        self.inner = std::mem::take(&mut self.inner).qr_ext(
            x,
            y,
            cell_width.unwrap_or(4) as u8,
            ec,
            Rotation::Deg0,
            content,
        );
        self
    }

    /// Appends a rectangular bounding box at (x, y).
    #[napi]
    pub fn box_rect(
        &mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        thickness: Option<u32>,
    ) -> &Self {
        self.inner = std::mem::take(&mut self.inner).box_outline(
            x,
            y,
            width,
            height,
            thickness.unwrap_or(2),
        );
        self
    }

    /// Appends a solid separator bar/line at (x, y).
    #[napi]
    pub fn line(&mut self, x: u32, y: u32, width: u32, height: u32) -> &Self {
        self.inner = std::mem::take(&mut self.inner).line(x, y, width, height);
        self
    }

    /// Inverts black/white pixels in a rectangular area.
    #[napi]
    pub fn reverse(&mut self, x: u32, y: u32, width: u32, height: u32) -> &Self {
        self.inner = std::mem::take(&mut self.inner).reverse(x, y, width, height);
        self
    }

    /// Sets number of copies to print.
    #[napi]
    pub fn copies(&mut self, count: u32) -> &Self {
        self.inner = std::mem::take(&mut self.inner).copies(count);
        self
    }

    /// Encodes the canvas into TSPL-II wire bytes (TSC, Xprinter, Rongta, Munbyn).
    #[napi]
    pub fn encode_tspl(&self) -> Buffer {
        Buffer::from(self.inner.encode_tspl())
    }

    /// Encodes the canvas into ZPL II wire bytes (Zebra, Citizen, Godex).
    #[napi]
    pub fn encode_zpl(&self) -> Buffer {
        Buffer::from(self.inner.encode_zpl())
    }

    /// Renders a virtual SVG sticker preview with die-cut rounded corners.
    #[napi]
    pub fn render_svg(&self) -> String {
        self.inner.render_svg()
    }
}
