use napi_derive::napi;
use papermint::{Alignment, ColumnWidth, CoverStatus, DrawerStatus, PaperStatus, TableColumn};

#[cfg(feature = "usb")]
use papermint::UsbPrinterInfo;

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

pub(crate) fn parse_columns(cols: &[TableColumnOptions]) -> Vec<TableColumn> {
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
