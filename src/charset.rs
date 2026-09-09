//! International character set selection for thermal receipt printers.
//!
//! International character sets modify 12 specific ASCII punctuation code points
//! (`#`, `$`, `@`, `[`, `\`, `]`, `^`, `` ` ``, `{`, `|`, `}`, `~`) to provide
//! localized national currency symbols and accented characters (e.g. `£`, `¥`, `é`, `ñ`, `§`).
//!
//! Both Epson ESC/POS and StarPRNT Line Mode use the `ESC R n` (`0x1B 0x52 n`) command.

/// Standard international character set selections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InternationalCharset {
    /// United States (standard ASCII).
    #[default]
    Usa,
    /// France (replaces currency with `£`, brackets with `°`, `ç`, `§`, etc.).
    France,
    /// Germany (replaces brackets and braces with umlauts `Ä`, `Ö`, `Ü`, `ä`, `ö`, `ü`, `ß`).
    Germany,
    /// United Kingdom (replaces `#` with Pound `£`).
    Uk,
    /// Denmark I.
    DenmarkI,
    /// Sweden.
    Sweden,
    /// Italy.
    Italy,
    /// Spain I (replaces `$` with `Pt`, adds `¡`, `¿`, `ñ`, `Ñ`).
    SpainI,
    /// Japan (replaces `\` with Yen `¥`).
    Japan,
    /// Norway.
    Norway,
    /// Denmark II.
    DenmarkII,
    /// Spain II.
    SpainII,
    /// Latin America.
    LatinAmerica,
    /// Korea.
    Korea,
    /// Slovenia / Croatia.
    Slovenia,
    /// China.
    China,
    /// Vietnam.
    Vietnam,
    /// Arabia.
    Arabia,
    /// Custom vendor-specific international character set code.
    Custom(u8),
}

impl InternationalCharset {
    /// Returns the wire byte index `n` for `ESC R n`.
    #[must_use]
    pub const fn code(&self) -> u8 {
        match self {
            Self::Usa => 0,
            Self::France => 1,
            Self::Germany => 2,
            Self::Uk => 3,
            Self::DenmarkI => 4,
            Self::Sweden => 5,
            Self::Italy => 6,
            Self::SpainI => 7,
            Self::Japan => 8,
            Self::Norway => 9,
            Self::DenmarkII => 10,
            Self::SpainII => 11,
            Self::LatinAmerica => 12,
            Self::Korea => 13,
            Self::Slovenia => 14,
            Self::China => 15,
            Self::Vietnam => 16,
            Self::Arabia => 17,
            Self::Custom(n) => *n,
        }
    }
}

impl From<u8> for InternationalCharset {
    fn from(n: u8) -> Self {
        match n {
            0 => Self::Usa,
            1 => Self::France,
            2 => Self::Germany,
            3 => Self::Uk,
            4 => Self::DenmarkI,
            5 => Self::Sweden,
            6 => Self::Italy,
            7 => Self::SpainI,
            8 => Self::Japan,
            9 => Self::Norway,
            10 => Self::DenmarkII,
            11 => Self::SpainII,
            12 => Self::LatinAmerica,
            13 => Self::Korea,
            14 => Self::Slovenia,
            15 => Self::China,
            16 => Self::Vietnam,
            17 => Self::Arabia,
            other => Self::Custom(other),
        }
    }
}
