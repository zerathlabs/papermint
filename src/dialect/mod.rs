//! Printer dialect abstractions.
//!
//! A [`Dialect`] defines how high-level [`Command`] intermediate
//! representations are translated into vendor-specific byte sequences.

use crate::command::Command;
use crate::error::Result;

#[cfg(feature = "escpos")]
pub mod escpos;

/// Trait implemented by printer command set encoders.
///
/// Implementors convert high-level [`Command`] IR variants into binary wire bytes,
/// appending them into a provided mutable buffer to minimize heap allocations.
pub trait Dialect: Send + Sync {
    /// Returns the human-readable name of this dialect (e.g., "ESC/POS").
    fn name(&self) -> &'static str;

    /// Encodes a single [`Command`] into the output buffer.
    ///
    /// # Errors
    ///
    /// Returns [`PapermintError`](crate::PapermintError) if the command cannot be encoded,
    /// contains invalid parameters, or exceeds dialect-specific size limits.
    fn encode(&self, command: &Command, buf: &mut Vec<u8>) -> Result<()>;
}
