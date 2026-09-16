//! Data transfer models and configuration types for papermintd.

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Daemon application state.
pub struct AppState {
    pub start_time: Instant,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
}

#[derive(Debug, Serialize)]
pub struct DevicesResponse {
    pub serial_ports: Vec<String>,
    pub usb_printers: Vec<UsbDeviceInfo>,
}

#[derive(Debug, Serialize)]
pub struct UsbDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub product_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TargetConfig {
    Tcp { address: String },
    Serial { port: String, baud: Option<u32> },
    Usb { vendor_id: u16, product_id: u16 },
    Mock,
}

pub fn parse_target_str(s: &str) -> Result<TargetConfig, String> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("mock") {
        return Ok(TargetConfig::Mock);
    }
    if let Some(rest) = s.strip_prefix("tcp://") {
        return Ok(TargetConfig::Tcp {
            address: rest.to_string(),
        });
    }
    if let Some(rest) = s.strip_prefix("usb:") {
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.len() == 2 {
            let vid = u16::from_str_radix(parts[0].trim_start_matches("0x"), 16)
                .map_err(|e| format!("invalid USB vendor ID '{}': {e}", parts[0]))?;
            let pid = u16::from_str_radix(parts[1].trim_start_matches("0x"), 16)
                .map_err(|e| format!("invalid USB product ID '{}': {e}", parts[1]))?;
            return Ok(TargetConfig::Usb {
                vendor_id: vid,
                product_id: pid,
            });
        }
    }
    if let Some(rest) = s.strip_prefix("serial:") {
        let parts: Vec<&str> = rest.split(':').collect();
        let port = parts[0].to_string();
        let baud = if parts.len() > 1 {
            parts[1].parse::<u32>().ok()
        } else {
            None
        };
        return Ok(TargetConfig::Serial { port, baud });
    }
    if s.contains(':') && !s.contains('/') {
        return Ok(TargetConfig::Tcp {
            address: s.to_string(),
        });
    }
    Err(format!(
        "Unrecognized printer target format: '{s}'. Expected 'mock', 'tcp://host:port', 'usb:vid:pid', or 'serial:/dev/port[:baud]'"
    ))
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DialectChoice {
    #[default]
    EscPos,
    Star,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketItem {
    pub qty: String,
    #[serde(alias = "name")]
    pub description: String,
    pub price: Option<String>,
    pub total: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketMetadata {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TicketPayload {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub address: Option<String>,
    pub metadata: Option<Vec<TicketMetadata>>,
    pub items: Option<Vec<TicketItem>>,
    pub totals: Option<Vec<TicketMetadata>>,
    pub qr: Option<String>,
    pub barcode: Option<String>,
    pub footer: Option<String>,
    pub cut_mode: Option<String>, // "full", "partial", "none"
    pub open_drawer: Option<bool>,
    pub beep: Option<u8>,
    pub divider_style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintRequest {
    pub target: TargetConfig,
    #[serde(default)]
    pub dialect: DialectChoice,
    #[serde(default = "default_paper_width")]
    pub paper_width: String,
    #[serde(default)]
    pub ticket: Option<TicketPayload>,
    #[serde(default)]
    pub raw_bytes: Option<Vec<u8>>,
}

pub fn default_paper_width() -> String {
    "80mm".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintResponse {
    pub success: bool,
    pub bytes_sent: usize,
    pub mock_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusRequest {
    pub target: TargetConfig,
    #[serde(default)]
    pub dialect: DialectChoice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub is_online: bool,
    pub is_ready: bool,
    pub paper: String,
    pub cover: String,
    pub drawer: String,
    pub cutter_error: bool,
    pub head_overheated: bool,
}
