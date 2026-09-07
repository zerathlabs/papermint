//! Layout and formatting engine for receipts.
//!
//! Provides the fluent [`Receipt`] builder and column alignment
//! utilities for creating structured, multi-column POS receipts.

pub mod column;
pub mod receipt;

pub use column::PaperWidth;
pub use receipt::Receipt;
