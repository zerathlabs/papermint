//! High-level printer orchestrator.

use crate::command::Command;
use crate::dialect::Dialect;
use crate::encoder::Encoder;
use crate::error::Result;
use crate::layout::Receipt;
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

