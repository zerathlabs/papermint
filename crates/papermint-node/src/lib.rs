//! Node.js and TypeScript bindings for the `papermint` thermal printing library.
//!
//! Provides high-level asynchronous access to receipt formatting, 2D label design,
//! transport drivers (TCP, Serial, USB, Mock), and sensor status queries.

pub mod label;
pub mod printer;
pub mod receipt;
pub mod types;

pub use label::JsLabel;
pub use printer::{JsPrinter, available_ports, list_printers};
pub use receipt::{JsReceipt, zatca_qr_base64};
pub use types::*;
