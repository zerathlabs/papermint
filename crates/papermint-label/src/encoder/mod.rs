//! Dialect Encoders for 2D Thermal Label Printers.

pub mod tspl;
pub mod zpl;

pub use tspl::encode_tspl;
pub use zpl::encode_zpl;
