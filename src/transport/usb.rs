//! USB Printer Class (Class 07) direct bulk transfer transport.

use std::time::Duration;

use async_trait::async_trait;
use nusb::transfer::{Buffer, Bulk, In, Out};
use nusb::MaybeFuture;
use tokio::time::timeout;

use crate::error::{PapermintError, Result};
use crate::transport::Transport;

/// Standard USB Printer Device Class code (`0x07`).
pub const USB_CLASS_PRINTER: u8 = 0x07;

/// Asynchronous user-space USB transport for thermal receipt printers.
///
/// Thermal printers commonly implement the USB Device Class Definition for Printing
/// Devices (Class 07, SubClass 01). Communication takes place over Bulk OUT (printing commands)
/// and optional Bulk IN (real-time telemetry and status) endpoints.
///
/// `UsbTransport` uses pure-Rust user-space USB access via `nusb`, avoiding OS print spooler
/// lag and driver installation hurdles on Linux, Windows, and macOS.
pub struct UsbTransport {
    vendor_id: Option<u16>,
    product_id: Option<u16>,
    serial_number: Option<String>,
    auto_discover_printer: bool,
    interface_number: Option<u8>,
    endpoint_out_address: Option<u8>,
    endpoint_in_address: Option<u8>,
    interface: Option<nusb::Interface>,
    endpoint_out: Option<nusb::Endpoint<Bulk, Out>>,
    endpoint_in: Option<nusb::Endpoint<Bulk, In>>,
    write_timeout: Duration,
    read_timeout: Duration,
}

impl std::fmt::Debug for UsbTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UsbTransport")
            .field("vendor_id", &self.vendor_id.map(|v| format!("0x{v:04X}")))
            .field("product_id", &self.product_id.map(|p| format!("0x{p:04X}")))
            .field("serial_number", &self.serial_number)
            .field("auto_discover_printer", &self.auto_discover_printer)
            .field("interface_number", &self.interface_number)
            .field(
                "endpoint_out_address",
                &self.endpoint_out_address.map(|a| format!("0x{a:02X}")),
            )
            .field(
                "endpoint_in_address",
                &self.endpoint_in_address.map(|a| format!("0x{a:02X}")),
            )
            .field("is_connected", &self.interface.is_some())
            .field("write_timeout", &self.write_timeout)
            .field("read_timeout", &self.read_timeout)
            .finish()
    }
}

/// Discovered USB printer metadata from system enumeration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbPrinterInfo {
    /// USB Vendor ID.
    pub vendor_id: u16,
    /// USB Product ID.
    pub product_id: u16,
    /// Device serial number string, if provided by the device.
    pub serial_number: Option<String>,
    /// Manufacturer name string, if provided.
    pub manufacturer: Option<String>,
    /// Product name string, if provided.
    pub product_name: Option<String>,
    /// Discovered interface number for printing.
    pub interface_number: u8,
}

impl UsbTransport {
    /// Enumerates and returns all attached USB devices matching the USB Printer Class (`0x07`).
    ///
    /// # Errors
    ///
    /// Returns [`PapermintError::Usb`] if USB device enumeration fails.
    pub fn list_printers() -> Result<Vec<UsbPrinterInfo>> {
        let devices = nusb::list_devices()
            .wait()
            .map_err(|e| PapermintError::Usb(format!("failed to enumerate USB devices: {e}")))?;

        let mut printers = Vec::new();
        for dev in devices {
            for iface in dev.interfaces() {
                if iface.class() == USB_CLASS_PRINTER {
                    printers.push(UsbPrinterInfo {
                        vendor_id: dev.vendor_id(),
                        product_id: dev.product_id(),
                        serial_number: dev.serial_number().map(String::from),
                        manufacturer: dev.manufacturer_string().map(String::from),
                        product_name: dev.product_string().map(String::from),
                        interface_number: iface.interface_number(),
                    });
                    break;
                }
            }
        }
        Ok(printers)
    }

    /// Creates a new USB transport targeting the given Vendor ID (VID) and Product ID (PID).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use papermint::transport::usb::UsbTransport;
    ///
    /// // Epson TM-T88VI (VID 0x04B8, PID 0x0202)
    /// let transport = UsbTransport::from_vid_pid(0x04B8, 0x0202);
    /// ```
    #[must_use]
    pub fn from_vid_pid(vendor_id: u16, product_id: u16) -> Self {
        Self {
            vendor_id: Some(vendor_id),
            product_id: Some(product_id),
            serial_number: None,
            auto_discover_printer: false,
            interface_number: None,
            endpoint_out_address: None,
            endpoint_in_address: None,
            interface: None,
            endpoint_out: None,
            endpoint_in: None,
            write_timeout: Duration::from_secs(5),
            read_timeout: Duration::from_secs(5),
        }
    }

    /// Creates a new USB transport configured to auto-discover the first attached USB Printer Class device.
    #[must_use]
    pub fn find_first_printer() -> Self {
        Self {
            vendor_id: None,
            product_id: None,
            serial_number: None,
            auto_discover_printer: true,
            interface_number: None,
            endpoint_out_address: None,
            endpoint_in_address: None,
            interface: None,
            endpoint_out: None,
            endpoint_in: None,
            write_timeout: Duration::from_secs(5),
            read_timeout: Duration::from_secs(5),
        }
    }

    /// Filters by device serial number string.
    #[must_use]
    pub fn with_serial_number(mut self, serial: impl Into<String>) -> Self {
        self.serial_number = Some(serial.into());
        self
    }

    /// Configures an explicit USB interface number instead of automatic discovery.
    #[must_use]
    pub const fn with_interface(mut self, interface_number: u8) -> Self {
        self.interface_number = Some(interface_number);
        self
    }

    /// Configures explicit endpoint addresses for bulk OUT and bulk IN transfers.
    #[must_use]
    pub const fn with_endpoints(mut self, out_address: u8, in_address: u8) -> Self {
        self.endpoint_out_address = Some(out_address);
        self.endpoint_in_address = Some(in_address);
        self
    }

    /// Configures the bulk OUT write timeout.
    #[must_use]
    pub const fn with_write_timeout(mut self, timeout: Duration) -> Self {
        self.write_timeout = timeout;
        self
    }

    /// Configures the bulk IN read timeout.
    #[must_use]
    pub const fn with_read_timeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = timeout;
        self
    }

    /// Returns the target Vendor ID, if specified.
    #[must_use]
    pub const fn vendor_id(&self) -> Option<u16> {
        self.vendor_id
    }

    /// Returns the target Product ID, if specified.
    #[must_use]
    pub const fn product_id(&self) -> Option<u16> {
        self.product_id
    }

    /// Connects to the USB printer device and claims the printing interface.
    pub async fn connect(&mut self) -> Result<()> {
        if self.interface.is_some() {
            return Ok(());
        }

        let devices = nusb::list_devices()
            .wait()
            .map_err(|e| PapermintError::Usb(format!("failed to enumerate USB devices: {e}")))?;

        let device_info = devices
            .into_iter()
            .find(|dev| {
                if let (Some(vid), Some(pid)) = (self.vendor_id, self.product_id) {
                    if dev.vendor_id() == vid && dev.product_id() == pid {
                        if self
                            .serial_number
                            .as_deref()
                            .is_some_and(|s| dev.serial_number() != Some(s))
                        {
                            return false;
                        }
                        return true;
                    }
                    false
                } else if self.auto_discover_printer {
                    for iface in dev.interfaces() {
                        if iface.class() == USB_CLASS_PRINTER {
                            return true;
                        }
                    }
                    false
                } else {
                    false
                }
            })
            .ok_or_else(|| {
                let desc = match (self.vendor_id, self.product_id) {
                    (Some(vid), Some(pid)) => format!("VID: 0x{vid:04X}, PID: 0x{pid:04X}"),
                    _ => "USB Printer Class device (Class 07)".into(),
                };
                PapermintError::DeviceNotFound(format!("USB printer not found ({desc})"))
            })?;

        let device = device_info
            .open()
            .wait()
            .map_err(|e| PapermintError::Usb(format!("failed to open USB printer device: {e}")))?;

        let mut iface_num = self.interface_number.unwrap_or(0);
        let mut ep_out_addr = self.endpoint_out_address;
        let mut ep_in_addr = self.endpoint_in_address;

        // Auto-detect printer interface and endpoints from active configuration if not explicitly provided
        if (ep_out_addr.is_none() || ep_in_addr.is_none())
            && let Ok(config) = device.active_configuration()
        {
            for iface in config.interfaces() {
                let matches_target = self.interface_number.is_none_or(|n| n == iface.interface_number());
                if matches_target {
                    for alt in iface.alt_settings() {
                        if alt.class() == USB_CLASS_PRINTER || self.interface_number.is_some() {
                            iface_num = iface.interface_number();
                            for ep in alt.endpoints() {
                                let addr = ep.address();
                                let is_in = (addr & 0x80) != 0;
                                if !is_in && ep_out_addr.is_none() {
                                    ep_out_addr = Some(addr);
                                } else if is_in && ep_in_addr.is_none() {
                                    ep_in_addr = Some(addr);
                                }
                            }
                        }
                    }
                }
            }
        }

        let out_addr = ep_out_addr.unwrap_or(0x01);
        let in_addr = ep_in_addr.unwrap_or(0x81);

        let interface = device
            .claim_interface(iface_num)
            .wait()
            .map_err(|e| {
                PapermintError::Usb(format!(
                    "failed to claim USB printer interface {iface_num}: {e}"
                ))
            })?;

        let ep_out = interface
            .endpoint::<Bulk, Out>(out_addr)
            .map_err(|e| {
                PapermintError::Usb(format!(
                    "failed to claim bulk OUT endpoint 0x{out_addr:02X}: {e}"
                ))
            })?;

        let ep_in = interface.endpoint::<Bulk, In>(in_addr).ok();

        self.interface = Some(interface);
        self.endpoint_out = Some(ep_out);
        self.endpoint_in = ep_in;
        self.endpoint_out_address = Some(out_addr);
        self.endpoint_in_address = Some(in_addr);
        self.interface_number = Some(iface_num);

        Ok(())
    }

    /// Disconnects and releases the USB interface.
    pub async fn disconnect(&mut self) -> Result<()> {
        self.endpoint_out = None;
        self.endpoint_in = None;
        self.interface = None;
        Ok(())
    }
}

#[async_trait]
impl Transport for UsbTransport {
    async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        if self.endpoint_out.is_none() {
            self.connect().await?;
        }

        let ep_out = self
            .endpoint_out
            .as_mut()
            .expect("bulk OUT endpoint must be initialized");

        // Transmit in 16KB bulk chunks
        for chunk in buf.chunks(16384) {
            ep_out.submit(chunk.to_vec().into());
            match timeout(self.write_timeout, ep_out.next_complete()).await {
                Ok(completion) => {
                    completion.into_result().map_err(|e| {
                        PapermintError::Usb(format!("USB bulk transfer failed: {e}"))
                    })?;
                }
                Err(_) => {
                    return Err(PapermintError::Timeout(
                        self.write_timeout.as_millis() as u64,
                    ));
                }
            }
        }

        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        if let Some(ep_out) = self.endpoint_out.as_mut() {
            while ep_out.pending() > 0 {
                match timeout(self.write_timeout, ep_out.next_complete()).await {
                    Ok(completion) => {
                        completion.into_result().map_err(|e| {
                            PapermintError::Usb(format!("USB bulk flush failed: {e}"))
                        })?;
                    }
                    Err(_) => {
                        return Err(PapermintError::Timeout(
                            self.write_timeout.as_millis() as u64,
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.endpoint_in.is_none() {
            self.connect().await?;
        }

        let ep_in = self.endpoint_in.as_mut().ok_or_else(|| {
            PapermintError::Usb("USB printer does not provide a bulk IN telemetry endpoint".into())
        })?;

        let request_len = buf.len().max(64);
        ep_in.submit(Buffer::new(request_len));

        match timeout(self.read_timeout, ep_in.next_complete()).await {
            Ok(completion) => {
                let response = completion.into_result().map_err(|e| {
                    PapermintError::Usb(format!("USB bulk IN read failed: {e}"))
                })?;
                let n = response.len().min(buf.len());
                buf[..n].copy_from_slice(&response[..n]);
                Ok(n)
            }
            Err(_) => Err(PapermintError::Timeout(
                self.read_timeout.as_millis() as u64,
            )),
        }
    }
}
