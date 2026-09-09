//! Printer dialect abstractions.
//!
//! A [`Dialect`] defines how high-level [`Command`] intermediate
//! representations are translated into vendor-specific byte sequences (such as
//! Epson ESC/POS, StarPRNT, Citizen, etc.).

use crate::command::Command;
use crate::error::Result;
use crate::status::PrinterStatus;

#[cfg(feature = "escpos")]
pub mod escpos;

#[cfg(feature = "star")]
pub mod star;

/// Trait implemented by printer command set encoders.
///
/// Implementors convert high-level [`Command`] IR variants into binary wire bytes,
/// appending them into a provided mutable buffer to minimize heap allocations.
pub trait Dialect: Send + Sync {
    /// Returns the human-readable name of this dialect (e.g., "ESC/POS", "StarPRNT").
    fn name(&self) -> &'static str;

    /// Encodes a single [`Command`] into the output buffer.
    ///
    /// # Errors
    ///
    /// Returns [`PapermintError`](crate::PapermintError) if the command cannot be encoded,
    /// contains invalid parameters, or exceeds dialect-specific size limits.
    fn encode(&self, command: &Command, buf: &mut Vec<u8>) -> Result<()>;

    /// Returns the binary command bytes to request real-time status telemetry from the printer.
    fn status_query_command(&self) -> Vec<u8>;

    /// Returns the expected number of response bytes for the status query command.
    ///
    /// By default, dialects expect 1 byte (e.g., Star `ENQ`). ESC/POS overrides this
    /// to 4 bytes for its 4-stage `DLE EOT 1..4` sequence.
    fn expected_status_bytes(&self) -> usize {
        1
    }

    /// Decodes raw telemetry response bytes received from the printer into a [`PrinterStatus`].
    ///
    /// # Errors
    ///
    /// Returns [`PapermintError::Dialect`](crate::PapermintError::Dialect) if the response byte
    /// sequence cannot be parsed or is malformed.
    fn parse_status_response(&self, bytes: &[u8]) -> Result<PrinterStatus>;
}
