use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use napi::Result as NapiResult;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use tokio::sync::Mutex;

use papermint::{Encoder, EscPos, Printer, Star};

#[cfg(feature = "serial")]
use papermint::SerialTransport;
#[cfg(feature = "usb")]
use papermint::UsbTransport;

use crate::label::JsLabel;
use crate::receipt::JsReceipt;
use crate::types::JsPrinterStatus;
#[cfg(feature = "usb")]
use crate::types::JsUsbPrinterInfo;

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
#[allow(unused_variables)]
pub fn list_printers() -> NapiResult<Vec<crate::types::JsUsbPrinterInfo>> {
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
                    let _ = (port, baud);
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
                    let _ = (vid, pid);
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
                    let _ = (port, baud);
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
                    let _ = (vid, pid);
                    Err(napi::Error::from_reason(
                        "USB feature not compiled into papermint-node",
                    ))
                }
            }
        }
    }

    /// Sends raw pre-encoded binary bytes (e.g. TSPL, ZPL, ESC/POS macros) directly to the printer.
    #[napi]
    pub async fn print_raw(&self, data: Buffer) -> NapiResult<()> {
        let target = self.target.lock().await;
        let bytes: &[u8] = &data;
        match &*target {
            Target::Mock(sink) => {
                use papermint::transport::Transport;
                let mut s = sink.clone();
                s.write_all(bytes)
                    .await
                    .map_err(|e| napi::Error::from_reason(e.to_string()))?;
                Ok(())
            }
            Target::Tcp(addr) => {
                use papermint::transport::Transport;
                let sock: SocketAddr = addr.parse().map_err(|e| {
                    napi::Error::from_reason(format!("invalid socket address {addr}: {e}"))
                })?;
                let mut transport = papermint::TcpTransport::new(sock)
                    .with_connect_timeout(Duration::from_secs(3))
                    .with_write_timeout(Duration::from_secs(5));
                transport
                    .write_all(bytes)
                    .await
                    .map_err(|e| napi::Error::from_reason(e.to_string()))?;
                Ok(())
            }
            Target::Serial { port, baud } => {
                #[cfg(feature = "serial")]
                {
                    use papermint::transport::Transport;
                    let mut transport = SerialTransport::new(port, *baud);
                    transport
                        .write_all(bytes)
                        .await
                        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
                    Ok(())
                }
                #[cfg(not(feature = "serial"))]
                {
                    let _ = (port, baud);
                    Err(napi::Error::from_reason(
                        "Serial feature not compiled into papermint-node",
                    ))
                }
            }
            Target::Usb { vid, pid } => {
                #[cfg(feature = "usb")]
                {
                    use papermint::transport::Transport;
                    let mut transport = UsbTransport::from_vid_pid(*vid, *pid);
                    transport
                        .write_all(bytes)
                        .await
                        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
                    Ok(())
                }
                #[cfg(not(feature = "usb"))]
                {
                    let _ = (vid, pid);
                    Err(napi::Error::from_reason(
                        "USB feature not compiled into papermint-node",
                    ))
                }
            }
        }
    }

    /// Encodes and prints a 2D label canvas (TSPL by default, or ZPL).
    #[napi]
    pub async fn print_label(&self, label: &JsLabel, dialect: Option<String>) -> NapiResult<()> {
        let bytes = match dialect.as_deref() {
            Some("zpl" | "Zpl" | "ZPL") => label.inner.encode_zpl(),
            _ => label.inner.encode_tspl(),
        };
        self.print_raw(Buffer::from(bytes)).await
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
