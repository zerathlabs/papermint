//! Tax and e-invoicing compliance engines.
//!
//! Provides standard generators and validators for electronic fiscal invoices
//! across Middle Eastern and international jurisdictions (ZATCA / Saudi Arabia, FTA / UAE).

pub mod zatca;

pub use zatca::{
    TlvEntry, ZatcaInvoice, base64_decode, base64_encode, decode_tlv, encode_zatca_tlv,
    zatca_qr_base64,
};
