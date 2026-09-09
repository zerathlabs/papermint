//! High-level printer orchestrator.

use crate::command::Command;
use crate::dialect::Dialect;
use crate::encoder::Encoder;
use crate::error::{PapermintError, Result};
use crate::layout::Receipt;
use crate::status::PrinterStatus;
use crate::transport::Transport;

/// High-level printer client coordinating dialect encoding and transport delivery.
#[derive(Debug)]
pub struct Printer<D, T> {
    encoder: Encoder<D>,
    transport: T,
}

impl<D: Dialect, T: Transport> Printer<D, T> {
    /// Creates a new printer with the specified dialect and transport.
    #[must_use]
    pub fn new(dialect: D, transport: T) -> Self {
        Self {
            encoder: Encoder::new(dialect),
            transport,
        }
    }

    /// Returns a reference to the underlying transport.
    #[must_use]
    pub const fn transport(&self) -> &T {
        &self.transport
    }

    /// Returns a mutable reference to the underlying transport.
    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    /// Consumes the printer, returning the underlying transport.
    #[must_use]
    pub fn into_transport(self) -> T {
        self.transport
    }

    /// Encodes and sends a single [`Command`] to the printer.
    pub async fn execute(&mut self, cmd: &Command) -> Result<()> {
        let mut buf = Vec::new();
        self.encoder.encode_one(cmd, &mut buf)?;
        self.transport.write_all(&buf).await?;
        self.transport.flush().await?;
        Ok(())
    }

    /// Encodes and transmits a sequence of [`Command`]s.
    pub async fn print_commands(&mut self, commands: &[Command]) -> Result<()> {
        let bytes = self.encoder.encode(commands)?;
        self.transport.write_all(&bytes).await?;
        self.transport.flush().await?;
        Ok(())
    }

    /// Encodes and transmits an entire [`Receipt`].
    pub async fn print(&mut self, receipt: &Receipt) -> Result<()> {
        self.print_commands(receipt.commands()).await
    }

    /// Queries real-time hardware status telemetry from the printer.
    ///
    /// Transmits the dialect status inquiry command, awaits the response from the transport,
    /// and decodes the sensor status into a [`PrinterStatus`].
    ///
    /// # Errors
    ///
    /// Returns [`PapermintError`] if transmission fails, times out, or the response cannot be decoded.
    pub async fn query_status(&mut self) -> Result<PrinterStatus> {
        let cmd = self.encoder.dialect().status_query_command();
        self.transport.write_all(&cmd).await?;
        self.transport.flush().await?;

        let expected = self.encoder.dialect().expected_status_bytes().max(1);
        let mut buf = [0u8; 16];
        let mut total = 0;

        while total < expected {
            let n = self.transport.read(&mut buf[total..expected]).await?;
            if n == 0 {
                break;
            }
            total += n;
        }

        if total == 0 {
            return Err(PapermintError::Transport(
                "empty status response received from printer".into(),
            ));
        }
        self.encoder.dialect().parse_status_response(&buf[..total])
    }
}

#[cfg(all(feature = "escpos", feature = "tcp"))]
impl Printer<crate::dialect::escpos::EscPos, crate::transport::tcp::TcpTransport> {
    /// Creates a new network printer using standard ESC/POS dialect connecting to the specified socket address.
    #[must_use]
    pub fn escpos_tcp(addr: std::net::SocketAddr) -> Self {
        Self::new(
            crate::dialect::escpos::EscPos::new(),
            crate::transport::tcp::TcpTransport::new(addr),
        )
    }
}

#[cfg(all(feature = "escpos", feature = "async"))]
impl Printer<crate::dialect::escpos::EscPos, crate::transport::vec_sink::VecSink> {
    /// Creates a new mock/virtual printer recording ESC/POS bytes into memory.
    #[must_use]
    pub fn escpos_mock() -> Self {
        Self::new(
            crate::dialect::escpos::EscPos::new(),
            crate::transport::vec_sink::VecSink::new(),
        )
    }
}

#[cfg(all(feature = "star", feature = "tcp"))]
impl Printer<crate::dialect::star::Star, crate::transport::tcp::TcpTransport> {
    /// Creates a new network printer using StarPRNT dialect connecting to the specified socket address.
    #[must_use]
    pub fn star_tcp(addr: std::net::SocketAddr) -> Self {
        Self::new(
            crate::dialect::star::Star::new(),
            crate::transport::tcp::TcpTransport::new(addr),
        )
    }
}

#[cfg(all(feature = "star", feature = "async"))]
impl Printer<crate::dialect::star::Star, crate::transport::vec_sink::VecSink> {
    /// Creates a new mock/virtual printer recording StarPRNT bytes into memory.
    #[must_use]
    pub fn star_mock() -> Self {
        Self::new(
            crate::dialect::star::Star::new(),
            crate::transport::vec_sink::VecSink::new(),
        )
    }
}

#[cfg(all(feature = "escpos", feature = "serial"))]
impl Printer<crate::dialect::escpos::EscPos, crate::transport::serial::SerialTransport> {
    /// Creates a new serial printer using standard ESC/POS dialect on the specified port.
    #[must_use]
    pub fn escpos_serial(port: impl Into<String>, baud_rate: u32) -> Self {
        Self::new(
            crate::dialect::escpos::EscPos::new(),
            crate::transport::serial::SerialTransport::new(port, baud_rate),
        )
    }
}

#[cfg(all(feature = "star", feature = "serial"))]
impl Printer<crate::dialect::star::Star, crate::transport::serial::SerialTransport> {
    /// Creates a new serial printer using StarPRNT dialect on the specified port.
    #[must_use]
    pub fn star_serial(port: impl Into<String>, baud_rate: u32) -> Self {
        Self::new(
            crate::dialect::star::Star::new(),
            crate::transport::serial::SerialTransport::new(port, baud_rate),
        )
    }
}

#[cfg(all(feature = "escpos", feature = "usb"))]
impl Printer<crate::dialect::escpos::EscPos, crate::transport::usb::UsbTransport> {
    /// Creates a new USB printer using standard ESC/POS dialect targeting the specified VID and PID.
    #[must_use]
    pub fn escpos_usb(vendor_id: u16, product_id: u16) -> Self {
        Self::new(
            crate::dialect::escpos::EscPos::new(),
            crate::transport::usb::UsbTransport::from_vid_pid(vendor_id, product_id),
        )
    }

    /// Creates a new USB printer using standard ESC/POS dialect auto-discovering the first attached USB printer.
    #[must_use]
    pub fn escpos_usb_auto() -> Self {
        Self::new(
            crate::dialect::escpos::EscPos::new(),
            crate::transport::usb::UsbTransport::find_first_printer(),
        )
    }
}

#[cfg(all(feature = "star", feature = "usb"))]
impl Printer<crate::dialect::star::Star, crate::transport::usb::UsbTransport> {
    /// Creates a new USB printer using StarPRNT dialect targeting the specified VID and PID.
    #[must_use]
    pub fn star_usb(vendor_id: u16, product_id: u16) -> Self {
        Self::new(
            crate::dialect::star::Star::new(),
            crate::transport::usb::UsbTransport::from_vid_pid(vendor_id, product_id),
        )
    }

    /// Creates a new USB printer using StarPRNT dialect auto-discovering the first attached USB printer.
    #[must_use]
    pub fn star_usb_auto() -> Self {
        Self::new(
            crate::dialect::star::Star::new(),
            crate::transport::usb::UsbTransport::find_first_printer(),
        )
    }
}
