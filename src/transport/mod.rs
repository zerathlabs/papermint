//! Printer communication transports.
//!
//! A [`Transport`] is responsible for delivering raw byte buffers to the
//! physical or virtual printer destination (TCP network, USB, Serial, Bluetooth, memory buffer, etc.).

use crate::error::Result;

#[cfg(feature = "async")]
use async_trait::async_trait;

#[cfg(feature = "async")]
pub mod vec_sink;

#[cfg(feature = "tcp")]
pub mod tcp;

/// Asynchronous transport interface for printer communication.
#[cfg(feature = "async")]
#[async_trait]
pub trait Transport: Send + Sync {
    /// Transmits the entire byte slice to the printer.
    async fn write_all(&mut self, buf: &[u8]) -> Result<()>;

    /// Flushes any pending data in the transport buffer.
    async fn flush(&mut self) -> Result<()>;

    /// Reads incoming bytes from the printer into `buf`, returning the number of bytes read.
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
}

