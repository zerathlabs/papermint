//! Character code page tables for thermal receipt printers.
//!
//! Different thermal printer hardware uses different indexing tables for
//! international character sets. For example, Western European (Windows-1252)
//! is table 16 in Epson ESC/POS, but table 32 in StarPRNT Line Mode.
//!
//! [`CodePage`] provides a vendor-agnostic abstraction so that receipt builders
//! can select a character set once, and the active [`crate::dialect::Dialect`]
//! will translate it to the appropriate wire index.

/// Supported international character code pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodePage {
    /// Standard USA / OEM (CP437).
    #[default]
    Pc437,

    /// Japanese Katakana.
    Katakana,

    /// Multilingual Latin I (CP850 - Western Europe).
    Pc850,

    /// Portuguese (CP860).
    Pc860,

    /// Canadian-French (CP863).
    Pc863,

    /// Nordic (CP865).
    Pc865,

    /// Windows-1252 (WPC1252 - Western European with Euro €).
    Wpc1252,

    /// Cyrillic #2 (CP866 - Russian).
    Pc866,

    /// Latin II (CP852 - Eastern European / Slavic).
    Pc852,

    /// PC858 Euro (PC850 with Euro symbol €).
    Pc858,

    /// Greek (CP737).
    Pc737,

    /// Standard Arabic (CP720).
    Pc720,

    /// Simplified Arabic (CP864).
    Pc864,

    /// Windows-1250 (Central European).
    Wpc1250,

    /// Windows-1251 (Cyrillic).
    Wpc1251,

    /// Windows-1253 (Greek).
    Wpc1253,

    /// Windows-1254 (Turkish).
    Wpc1254,

    /// Windows-1255 (Hebrew).
    Wpc1255,

    /// Windows-1256 (Arabic).
    Wpc1256,

    /// Windows-1257 (Baltic Rim).
    Wpc1257,

    /// Windows-1258 (Vietnamese).
    Wpc1258,

    /// Custom vendor-specific code page index.
    Custom(u8),
}

impl From<u8> for CodePage {
    fn from(code: u8) -> Self {
        Self::Custom(code)
    }
}
