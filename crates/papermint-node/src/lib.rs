use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use napi::Result as NapiResult;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use tokio::sync::Mutex;

use papermint::{
    Alignment, ColumnWidth, CoverStatus, DrawerStatus, Encoder, EscPos, PaperStatus, PaperWidth,
    Printer, Receipt, Star, TableColumn, ZatcaInvoice,
};

#[cfg(feature = "serial")]
use papermint::SerialTransport;
#[cfg(feature = "usb")]
use papermint::{UsbPrinterInfo, UsbTransport};

/// Configuration options for table columns.
#[napi(object)]
pub struct TableColumnOptions {
    /// Fixed character width.
    pub width_fixed: Option<u32>,
    /// Simple fixed character width shorthand.
    pub width: Option<u32>,
    /// Proportional fraction of total columns (0.0 .. 1.0).
    pub width_fraction: Option<f64>,
    /// Text alignment: "left", "center", or "right".
    pub align: Option<String>,
    /// Text alignment alias: "left", "center", or "right".
    pub alignment: Option<String>,
}

/// ZATCA (Saudi Arabia) & FTA (UAE) compliant invoice metadata for E-Invoicing QR codes.
#[napi(object)]
pub struct JsZatcaInvoice {
    /// Seller's legal or commercial trading name (Tag 1).
    pub seller_name: String,
    /// Tax Registration Number (TRN / VAT number, typically 15 digits) (Tag 2).
    pub vat_number: String,
    /// Invoice timestamp in ISO 8601 format (Tag 3, e.g. "2026-09-10T14:30:00Z").
    pub timestamp: String,
    /// Invoice total amount including VAT (Tag 4, e.g. "115.00").
    pub total_amount: String,
    /// Total VAT amount (Tag 5, e.g. "15.00").
    pub vat_amount: String,
}

/// A receipt builder for thermal printers.
#[napi(js_name = "Receipt")]
pub struct JsReceipt {
    inner: Receipt,
}

#[napi]
impl JsReceipt {
    /// Creates a new receipt builder.
    ///
    /// @param paperWidth - Paper width: "80mm" (default) or "58mm".
    #[napi(constructor)]
    pub fn new(paper_width: Option<String>) -> Self {
        let width = match paper_width.as_deref() {
            Some("58mm" | "58" | "Mm58") => PaperWidth::Mm58,
            _ => PaperWidth::Mm80,
        };
        Self {
            inner: Receipt::new(width),
        }
    }

    /// Initializes printer hardware state (clears formatting, resets modes).
    #[napi]
    pub fn init(&mut self) -> &Self {
        self.inner = std::mem::take(&mut self.inner).init();
        self
    }

    /// Prints text without trailing newline.
    #[napi]
    pub fn text(&mut self, text: String) -> &Self {
        self.inner = std::mem::take(&mut self.inner).text(text);
        self
    }

    /// Prints text followed by a line feed.
    #[napi]
    pub fn text_ln(&mut self, text: String) -> &Self {
        self.inner = std::mem::take(&mut self.inner).text_ln(text);
        self
    }

    /// Aligns subsequent output to the left.
    #[napi]
    pub fn left(&mut self) -> &Self {
        self.inner = std::mem::take(&mut self.inner).left();
        self
    }

    /// Centers subsequent output.
    #[napi]
    pub fn center(&mut self) -> &Self {
        self.inner = std::mem::take(&mut self.inner).center();
        self
    }

    /// Aligns subsequent output to the right.
    #[napi]
    pub fn right(&mut self) -> &Self {
        self.inner = std::mem::take(&mut self.inner).right();
        self
    }

    /// Enables or disables bold text emphasis.
    #[napi]
    pub fn bold(&mut self, enable: bool) -> &Self {
        self.inner = std::mem::take(&mut self.inner).bold(enable);
        self
    }

    /// Enables or disables single underline.
    #[napi]
    pub fn underline(&mut self, enable: bool) -> &Self {
        self.inner = if enable {
            std::mem::take(&mut self.inner).underline_on()
        } else {
            std::mem::take(&mut self.inner).underline_off()
        };
        self
    }

    /// Enables or disables inverted (white-on-black) printing.
    #[napi]
    pub fn invert(&mut self, enable: bool) -> &Self {
        self.inner = std::mem::take(&mut self.inner).invert(enable);
        self
    }

    /// Enables or disables double-height characters.
    #[napi]
    pub fn double_height(&mut self, enable: bool) -> &Self {
        self.inner = std::mem::take(&mut self.inner).double_height(enable);
        self
    }

    /// Enables or disables double-width characters.
    #[napi]
    pub fn double_width(&mut self, enable: bool) -> &Self {
        self.inner = std::mem::take(&mut self.inner).double_width(enable);
        self
    }

    /// Enables or disables both double-height and double-width characters.
    #[napi]
    pub fn double_size(&mut self, enable: bool) -> &Self {
        self.inner = std::mem::take(&mut self.inner).double_size(enable);
        self
    }

    /// Feeds paper by the specified number of lines.
    #[napi]
    pub fn feed(&mut self, lines: Option<u32>) -> &Self {
        let n = lines.unwrap_or(1) as u8;
        self.inner = std::mem::take(&mut self.inner).feed(n);
        self
    }

    /// Performs a full paper cut.
    #[napi]
    pub fn cut_full(&mut self) -> &Self {
        self.inner = std::mem::take(&mut self.inner).cut_full();
        self
    }

    /// Performs a partial paper cut.
    #[napi]
    pub fn cut_partial(&mut self) -> &Self {
        self.inner = std::mem::take(&mut self.inner).cut_partial();
        self
    }

    /// Prints a full horizontal divider line spanning paper width.
    #[napi]
    pub fn divider(&mut self, ch: Option<String>) -> &Self {
        let char_val = ch.as_deref().and_then(|s| s.chars().next()).unwrap_or('-');
        self.inner = std::mem::take(&mut self.inner).divider(char_val);
        self
    }

    /// Prints a double-line horizontal divider rule (`================`).
    #[napi]
    pub fn divider_double(&mut self) -> &Self {
        self.inner = std::mem::take(&mut self.inner).divider_double();
        self
    }

    /// Prints a dotted horizontal divider rule (`................`).
    #[napi]
    pub fn divider_dotted(&mut self) -> &Self {
        self.inner = std::mem::take(&mut self.inner).divider_dotted();
        self
    }

    /// Prints a dashed horizontal divider rule (`- - - - - - - - `).
    #[napi]
    pub fn divider_dashed(&mut self) -> &Self {
        self.inner = std::mem::take(&mut self.inner).divider_dashed();
        self
    }

    /// Prints a repeating pattern horizontal divider rule across paper width.
    #[napi]
    pub fn divider_pattern(&mut self, pattern: String) -> &Self {
        self.inner = std::mem::take(&mut self.inner).divider_pattern(&pattern);
        self
    }

    /// Prints a 2-column row (left aligned left, right aligned right).
    #[napi]
    pub fn two_column(&mut self, left: String, right: String) -> &Self {
        self.inner = std::mem::take(&mut self.inner).two_column(left, right);
        self
    }

    /// Prints a 3-column row (left, center, right).
    #[napi]
    pub fn three_column(&mut self, left: String, center: String, right: String) -> &Self {
        self.inner = std::mem::take(&mut self.inner).three_column(left, center, right);
        self
    }

    /// Configures active table columns and prints a bold header row.
    #[napi]
    pub fn table_header(
        &mut self,
        headers: Vec<String>,
        columns: Vec<TableColumnOptions>,
    ) -> &Self {
        let cols = parse_columns(&columns);
        self.inner = std::mem::take(&mut self.inner).table_header(&headers, &cols);
        self
    }

    /// Sets active table columns without printing a header row.
    #[napi]
    pub fn set_columns(&mut self, columns: Vec<TableColumnOptions>) -> &Self {
        let cols = parse_columns(&columns);
        self.inner = std::mem::take(&mut self.inner).set_columns(&cols);
        self
    }

    /// Prints a table row formatted against the configured table columns.
    #[napi]
    pub fn row(&mut self, cells: Vec<String>) -> &Self {
        self.inner = std::mem::take(&mut self.inner).row(&cells);
        self
    }

    /// Clears any configured table columns.
    #[napi]
    pub fn clear_columns(&mut self) -> &Self {
        self.inner = std::mem::take(&mut self.inner).clear_columns();
        self
    }

    /// Prints a QR code with the given content.
    #[napi]
    pub fn qr(&mut self, content: String) -> &Self {
        self.inner = std::mem::take(&mut self.inner).qr(content);
        self
    }

    /// Appends a ZATCA (Saudi Arabia) & FTA (UAE) compliant E-Invoicing QR code.
    #[napi]
    pub fn zatca_qr(&mut self, invoice: JsZatcaInvoice) -> &Self {
        let inv = ZatcaInvoice::new(
            invoice.seller_name,
            invoice.vat_number,
            invoice.timestamp,
            invoice.total_amount,
            invoice.vat_amount,
        );
        self.inner = std::mem::take(&mut self.inner).zatca_qr(&inv);
        self
    }

    /// Prints a Code128 barcode.
    #[napi]
    pub fn barcode_128(&mut self, content: String) -> &Self {
        self.inner = std::mem::take(&mut self.inner).barcode_128(content);
        self
    }

    /// Triggers acoustic buzzer alert.
    #[napi]
    pub fn beep(&mut self, count: Option<u32>, duration: Option<u32>) -> &Self {
        let c = count.unwrap_or(1) as u8;
        let d = duration.unwrap_or(2) as u8;
        self.inner = std::mem::take(&mut self.inner).beep(c, d);
        self
    }

    /// Triggers cash drawer kickout pin.
    #[napi]
    pub fn open_drawer(&mut self) -> &Self {
        self.inner = std::mem::take(&mut self.inner).open_drawer();
        self
    }

    /// Renders a virtual SVG vector graphic preview of the receipt.
    #[napi]
    pub fn render_svg(&self) -> String {
        self.inner.render_svg()
    }

    /// Renders a responsive, styled HTML component snippet preview of the receipt.
    #[napi]
    pub fn render_html(&self) -> String {
        self.inner.render_html()
    }

    /// Encodes receipt commands into raw binary wire bytes for the given dialect.
    ///
    /// @param dialect - "escpos" (default) or "star".
    #[napi]
    pub fn encode(&self, dialect: Option<String>) -> NapiResult<Buffer> {
        let d = dialect.as_deref().unwrap_or("escpos");
        let bytes = match d.to_ascii_lowercase().as_str() {
            "star" | "starprnt" => Encoder::star()
                .encode(self.inner.commands())
                .map_err(|e| napi::Error::from_reason(e.to_string()))?,
            _ => Encoder::escpos()
                .encode(self.inner.commands())
                .map_err(|e| napi::Error::from_reason(e.to_string()))?,
        };
        Ok(Buffer::from(bytes))
    }
}

/// Generates a ZATCA (Saudi Arabia) & FTA (UAE) compliant Base64 QR payload string from invoice metadata.
#[napi]
pub fn zatca_qr_base64(
    seller_name: String,
    vat_number: String,
    timestamp: String,
    total_amount: String,
    vat_amount: String,
) -> String {
    let inv = ZatcaInvoice::new(seller_name, vat_number, timestamp, total_amount, vat_amount);
    inv.to_qr_base64()
}

fn parse_columns(cols: &[TableColumnOptions]) -> Vec<TableColumn> {
    let mut result = Vec::with_capacity(cols.len());
    for c in cols {
        let width = if let Some(fixed) = c.width_fixed.or(c.width) {
            ColumnWidth::Fixed(fixed as usize)
        } else if let Some(frac) = c.width_fraction {
            ColumnWidth::Fraction(frac as f32)
        } else {
            ColumnWidth::Fraction(1.0)
        };

        let align_str = c.align.as_deref().or(c.alignment.as_deref());
        let align = match align_str.map(str::to_ascii_lowercase).as_deref() {
            Some("right") => Alignment::Right,
            Some("center") => Alignment::Center,
            _ => Alignment::Left,
        };

        result.push(TableColumn::new(width, align));
    }
    result
}

/// Real-time printer sensor telemetry and mechanical status.
#[napi(object)]
pub struct JsPrinterStatus {
    pub is_online: bool,
    pub is_ready: bool,
    pub paper: String,
    pub cover: String,
    pub drawer: String,
    pub cutter_error: bool,
    pub head_overheated: bool,
}

impl From<papermint::PrinterStatus> for JsPrinterStatus {
    fn from(s: papermint::PrinterStatus) -> Self {
        let paper = match s.paper {
            PaperStatus::Adequate => "Adequate",
            PaperStatus::NearEnd => "NearEnd",
            PaperStatus::Empty => "Empty",
        };
        let cover = match s.cover {
            CoverStatus::Closed => "Closed",
            CoverStatus::Open => "Open",
        };
        let drawer = match s.drawer {
            DrawerStatus::Closed => "Closed",
            DrawerStatus::Open => "Open",
        };

        Self {
            is_online: s.is_online,
            is_ready: s.is_ready(),
            paper: paper.to_string(),
            cover: cover.to_string(),
            drawer: drawer.to_string(),
            cutter_error: s.cutter_error,
            head_overheated: s.head_overheated,
        }
    }
}

/// Metadata for an enumerated USB Printer Class device.
#[napi(object)]
pub struct JsUsbPrinterInfo {
    pub vendor_id: u32,
    pub product_id: u32,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub product_name: Option<String>,
}

#[cfg(feature = "usb")]
impl From<UsbPrinterInfo> for JsUsbPrinterInfo {
    fn from(info: UsbPrinterInfo) -> Self {
        Self {
            vendor_id: info.vendor_id as u32,
            product_id: info.product_id as u32,
            serial_number: info.serial_number,
            manufacturer: info.manufacturer,
            product_name: info.product_name,
        }
    }
}

/// Enumerates available serial / COM ports on the host system.
#[napi]
pub fn available_ports() -> NapiResult<Vec<String>> {
    #[cfg(feature = "serial")]
    {
        SerialTransport::available_ports().map_err(|e| napi::Error::from_reason(e.to_string()))
    }
    #[cfg(not(feature = "serial"))]
    {
        Ok(Vec::new())
    }
}

/// Enumerates connected USB Printer Class (0x07) devices.
#[napi]
pub fn list_printers() -> NapiResult<Vec<JsUsbPrinterInfo>> {
    #[cfg(feature = "usb")]
    {
        match UsbTransport::list_printers() {
            Ok(list) => Ok(list.into_iter().map(JsUsbPrinterInfo::from).collect()),
            Err(papermint::PapermintError::Usb(msg)) if msg.contains("not found") => Ok(Vec::new()),
            Err(papermint::PapermintError::DeviceNotFound(_)) => Ok(Vec::new()),
            Err(e) => Err(napi::Error::from_reason(e.to_string())),
        }
    }
    #[cfg(not(feature = "usb"))]
    {
        Ok(Vec::new())
    }
}

enum Target {
    Tcp(String),
    Serial { port: String, baud: u32 },
    Usb { vid: u16, pid: u16 },
    Mock(papermint::VecSink),
}

/// High-level client for communicating with thermal receipt printers.
#[napi(js_name = "Printer")]
pub struct JsPrinter {
    dialect: String,
    target: Arc<Mutex<Target>>,
}

#[napi]
impl JsPrinter {
    /// Creates a printer connecting over TCP / Ethernet / Wi-Fi.
    #[napi(factory)]
    pub fn tcp(address: String, dialect: Option<String>) -> Self {
        let d = dialect.unwrap_or_else(|| "escpos".to_string());
        Self {
            dialect: d,
            target: Arc::new(Mutex::new(Target::Tcp(address))),
        }
    }

    /// Creates a printer connecting over Serial / RS-232 / Virtual COM.
    #[napi(factory)]
    pub fn serial(port: String, baud_rate: Option<u32>, dialect: Option<String>) -> Self {
        let d = dialect.unwrap_or_else(|| "escpos".to_string());
        let baud = baud_rate.unwrap_or(19200);
        Self {
            dialect: d,
            target: Arc::new(Mutex::new(Target::Serial { port, baud })),
        }
    }

    /// Creates a printer connecting over direct USB (Printer Class 07).
    #[napi(factory)]
    pub fn usb(vendor_id: u32, product_id: u32, dialect: Option<String>) -> Self {
        let d = dialect.unwrap_or_else(|| "escpos".to_string());
        Self {
            dialect: d,
            target: Arc::new(Mutex::new(Target::Usb {
                vid: vendor_id as u16,
                pid: product_id as u16,
            })),
        }
    }

    /// Creates an in-memory mock printer for testing without physical hardware.
    #[napi(factory)]
    pub fn mock(dialect: Option<String>) -> Self {
        let d = dialect.unwrap_or_else(|| "escpos".to_string());
        Self {
            dialect: d,
            target: Arc::new(Mutex::new(Target::Mock(papermint::VecSink::new()))),
        }
    }

    /// Transmits receipt commands to the target printer.
    #[napi]
    pub async fn print(&self, receipt: &JsReceipt) -> NapiResult<()> {
        let target = self.target.lock().await;
        let is_star = self.dialect.eq_ignore_ascii_case("star");

        match &*target {
            Target::Mock(sink) => {
                let bytes = if is_star {
                    Encoder::star()
                        .encode(receipt.inner.commands())
                        .map_err(|e| napi::Error::from_reason(e.to_string()))?
                } else {
                    Encoder::escpos()
                        .encode(receipt.inner.commands())
                        .map_err(|e| napi::Error::from_reason(e.to_string()))?
                };
                use papermint::transport::Transport;
                let mut s = sink.clone();
                s.write_all(&bytes)
                    .await
                    .map_err(|e| napi::Error::from_reason(e.to_string()))?;
                Ok(())
            }
            Target::Tcp(addr) => {
                let sock: SocketAddr = addr.parse().map_err(|e| {
                    napi::Error::from_reason(format!("invalid socket address {addr}: {e}"))
                })?;
                let transport = papermint::TcpTransport::new(sock)
                    .with_connect_timeout(Duration::from_secs(3))
                    .with_write_timeout(Duration::from_secs(5));

                if is_star {
                    let mut p = Printer::new(Star::new(), transport);
                    p.print(&receipt.inner)
                        .await
                        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
                } else {
                    let mut p = Printer::new(EscPos::new(), transport);
                    p.print(&receipt.inner)
                        .await
                        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
                }
                Ok(())
            }
            Target::Serial { port, baud } => {
                #[cfg(feature = "serial")]
                {
                    let transport = SerialTransport::new(port, *baud);
                    if is_star {
                        let mut p = Printer::new(Star::new(), transport);
                        p.print(&receipt.inner)
                            .await
                            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
                    } else {
                        let mut p = Printer::new(EscPos::new(), transport);
                        p.print(&receipt.inner)
                            .await
                            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
                    }
                    Ok(())
                }
                #[cfg(not(feature = "serial"))]
                {
                    Err(napi::Error::from_reason(
                        "Serial feature not compiled into papermint-node",
                    ))
                }
            }
            Target::Usb { vid, pid } => {
                #[cfg(feature = "usb")]
                {
                    let transport = UsbTransport::from_vid_pid(*vid, *pid);
                    if is_star {
                        let mut p = Printer::new(Star::new(), transport);
                        p.print(&receipt.inner)
                            .await
                            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
                    } else {
                        let mut p = Printer::new(EscPos::new(), transport);
                        p.print(&receipt.inner)
                            .await
                            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
                    }
                    Ok(())
                }
                #[cfg(not(feature = "usb"))]
                {
                    Err(napi::Error::from_reason(
                        "USB feature not compiled into papermint-node",
                    ))
                }
            }
        }
    }

    /// Queries real-time hardware status and sensor telemetry.
    #[napi]
    pub async fn query_status(&self) -> NapiResult<JsPrinterStatus> {
        let target = self.target.lock().await;
        let is_star = self.dialect.eq_ignore_ascii_case("star");

        match &*target {
            Target::Mock(sink) => {
                if is_star {
                    let mut p = Printer::new(Star::new(), sink.clone());
                    let status = p
                        .query_status()
                        .await
                        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
                    Ok(JsPrinterStatus::from(status))
                } else {
                    let mut p = Printer::new(EscPos::new(), sink.clone());
                    let status = p
                        .query_status()
                        .await
                        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
                    Ok(JsPrinterStatus::from(status))
                }
            }
            Target::Tcp(addr) => {
                let sock: SocketAddr = addr.parse().map_err(|e| {
                    napi::Error::from_reason(format!("invalid socket address {addr}: {e}"))
                })?;
                let transport = papermint::TcpTransport::new(sock)
                    .with_connect_timeout(Duration::from_secs(3))
                    .with_read_timeout(Duration::from_secs(3));

                if is_star {
                    let mut p = Printer::new(Star::new(), transport);
                    let status = p
                        .query_status()
                        .await
                        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
                    Ok(JsPrinterStatus::from(status))
                } else {
                    let mut p = Printer::new(EscPos::new(), transport);
                    let status = p
                        .query_status()
                        .await
                        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
                    Ok(JsPrinterStatus::from(status))
                }
            }
            Target::Serial { port, baud } => {
                #[cfg(feature = "serial")]
                {
                    let transport = SerialTransport::new(port, *baud);
                    if is_star {
                        let mut p = Printer::new(Star::new(), transport);
                        let status = p
                            .query_status()
                            .await
                            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
                        Ok(JsPrinterStatus::from(status))
                    } else {
                        let mut p = Printer::new(EscPos::new(), transport);
                        let status = p
                            .query_status()
                            .await
                            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
                        Ok(JsPrinterStatus::from(status))
                    }
                }
                #[cfg(not(feature = "serial"))]
                {
                    Err(napi::Error::from_reason(
                        "Serial feature not compiled into papermint-node",
                    ))
                }
            }
            Target::Usb { vid, pid } => {
                #[cfg(feature = "usb")]
                {
                    let transport = UsbTransport::from_vid_pid(*vid, *pid);
                    if is_star {
                        let mut p = Printer::new(Star::new(), transport);
                        let status = p
                            .query_status()
                            .await
                            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
                        Ok(JsPrinterStatus::from(status))
                    } else {
                        let mut p = Printer::new(EscPos::new(), transport);
                        let status = p
                            .query_status()
                            .await
                            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
                        Ok(JsPrinterStatus::from(status))
                    }
                }
                #[cfg(not(feature = "usb"))]
                {
                    Err(napi::Error::from_reason(
                        "USB feature not compiled into papermint-node",
                    ))
                }
            }
        }
    }

    /// Returns the accumulated binary wire bytes written to an in-memory mock printer.
    #[napi]
    pub async fn get_recorded_bytes(&self) -> NapiResult<Option<Buffer>> {
        let target = self.target.lock().await;
        if let Target::Mock(sink) = &*target {
            Ok(Some(Buffer::from(sink.bytes())))
        } else {
            Ok(None)
        }
    }
}
