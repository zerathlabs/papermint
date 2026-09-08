// papermint — The GOAT Thermal Printing Library
//
// Three-layer architecture:
//   Layer 3 (Layout)    → Receipt builder produces Vec<Command>
//   Layer 2 (Encoder)   → Dialect trait translates Command → raw bytes
//   Layer 1 (Transport) → Sends bytes to hardware (TCP, USB, Serial, etc.)

pub mod charset;
pub mod codepage;
pub mod command;
pub mod encoder;
pub mod error;
pub mod status;

pub mod dialect;

pub mod layout;

#[cfg(feature = "async")]
pub mod transport;

#[cfg(feature = "async")]
pub mod printer;

// ── Public re-exports for ergonomic top-level imports ──

pub use charset::InternationalCharset;
pub use codepage::CodePage;
pub use command::*;
pub use encoder::Encoder;
pub use error::PapermintError;
pub use status::{CoverStatus, DrawerStatus, PaperStatus, PrinterStatus};

#[cfg(feature = "escpos")]
pub use dialect::escpos::EscPos;

#[cfg(feature = "star")]
pub use dialect::star::Star;

pub use layout::receipt::Receipt;
pub use layout::column::{ColumnWidth, PaperWidth, TableColumn};

#[cfg(feature = "async")]
pub use printer::Printer;

#[cfg(feature = "tcp")]
pub use transport::tcp::TcpTransport;

#[cfg(feature = "async")]
pub use transport::vec_sink::VecSink;

#[cfg(feature = "image")]
pub mod image;

#[cfg(feature = "image")]
pub use self::image::DitherMode;
