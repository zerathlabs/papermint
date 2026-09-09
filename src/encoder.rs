//! Command stream encoder.
//!
//! An [`Encoder`] pairs a collection of [`Command`]s with a [`Dialect`]
//! to produce a byte vector ready for transport transmission.

use crate::command::Command;
use crate::dialect::Dialect;
use crate::error::Result;

/// Translates sequences of [`Command`]s into wire-format byte streams.
#[derive(Debug, Clone, Default)]
pub struct Encoder<D> {
    dialect: D,
}

impl<D: Dialect> Encoder<D> {
    /// Creates a new encoder using the provided dialect.
    #[must_use]
    pub const fn new(dialect: D) -> Self {
        Self { dialect }
    }

    /// Returns a reference to the underlying dialect.
    #[must_use]
    pub const fn dialect(&self) -> &D {
        &self.dialect
    }

    /// Consumes the encoder and returns the underlying dialect.
    #[must_use]
    pub fn into_dialect(self) -> D {
        self.dialect
    }

    /// Encodes a single [`Command`] into an existing buffer.
    pub fn encode_one(&self, cmd: &Command, buf: &mut Vec<u8>) -> Result<()> {
        self.dialect.encode(cmd, buf)
    }

    /// Encodes an iterator of [`Command`]s into an existing buffer.
    pub fn encode_into<'a, I>(&self, commands: I, buf: &mut Vec<u8>) -> Result<()>
    where
        I: IntoIterator<Item = &'a Command>,
    {
        for cmd in commands {
            self.dialect.encode(cmd, buf)?;
        }
        Ok(())
    }

    /// Encodes a slice of [`Command`]s into a newly allocated [`Vec<u8>`].
    pub fn encode(&self, commands: &[Command]) -> Result<Vec<u8>> {
        // Pre-allocate a reasonable buffer capacity to reduce reallocations
        let mut buf = Vec::with_capacity(commands.len() * 16);
        self.encode_into(commands, &mut buf)?;
        Ok(buf)
    }
}

#[cfg(feature = "escpos")]
impl Encoder<crate::dialect::escpos::EscPos> {
    /// Creates a new ESC/POS encoder with default configuration.
    #[must_use]
    pub fn escpos() -> Self {
        Self::new(crate::dialect::escpos::EscPos::new())
    }
}

#[cfg(feature = "star")]
impl Encoder<crate::dialect::star::Star> {
    /// Creates a new StarPRNT encoder with default configuration.
    #[must_use]
    pub fn star() -> Self {
        Self::new(crate::dialect::star::Star::new())
    }
}
