// ── error.rs — Structured error types ──
//
// Structured, typed error variants using `thiserror` for comprehensive diagnostics.

/// All errors that can occur in papermint operations.
#[derive(Debug, thiserror::Error)]
pub enum PapermintError {
    /// The transport layer failed to send data to the printer.
    #[error("transport error: {0}")]
    Transport(String),

    /// A connection attempt timed out.
    #[error("connection timeout after {0}ms")]
    Timeout(u64),

    /// The encoder encountered invalid or unsupported command data.
    #[error("encoding error: {0}")]
    Encoding(String),

    /// A command contained invalid parameters.
    #[error("invalid command: {0}")]
    InvalidCommand(String),

    /// The dialect encountered an error parsing or generating commands.
    #[error("dialect error: {0}")]
    Dialect(String),

    /// A protocol error occurred during communication.
    #[error("protocol error: {0}")]
    Protocol(String),

    /// An underlying I/O error (only available with `std` feature).
    #[cfg(feature = "std")]
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// An image processing error (only available with `image` feature).
    #[cfg(feature = "image")]
    #[error("image error: {0}")]
    Image(#[from] ::image::ImageError),

    /// A requested USB or serial printer device was not found.
    #[error("device not found: {0}")]
    DeviceNotFound(String),

    /// A serial port communication error.
    #[error("serial error: {0}")]
    Serial(String),

    /// A USB communication error.
    #[error("usb error: {0}")]
    Usb(String),
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, PapermintError>;
