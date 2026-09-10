//! Layout and formatting engine for receipts.
//!
//! Provides the fluent [`Receipt`] builder and column alignment
//! utilities for creating structured, multi-column POS receipts.

pub mod column;
pub mod preview;
pub mod receipt;

pub use column::PaperWidth;
pub use preview::{render_html, render_svg};
pub use receipt::Receipt;
