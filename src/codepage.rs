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

    /// Turkish (CP857).
    Pc857,

    /// Latin 9 with Euro (ISO-8859-15).
    Iso8859_15,

    /// Thai (CP874).
    Pc874,

    /// Custom vendor-specific code page index.
    Custom(u8),
}

impl From<u8> for CodePage {
    fn from(code: u8) -> Self {
        Self::Custom(code)
    }
}

impl CodePage {
    /// Encodes a single Unicode `char` into the 8-bit wire byte for this code page.
    ///
    /// For ASCII characters (`\u{0000}..=\u{007F}`), returns `Some(ch as u8)`.
    /// For non-ASCII characters, translates into the corresponding 8-bit byte (`0x80..=0xFF`).
    /// Returns `None` if the character is not representable in this code page.
    #[must_use]
    pub fn encode_char(&self, ch: char) -> Option<u8> {
        if ch.is_ascii() {
            return Some(ch as u8);
        }

        match self {
            Self::Wpc1252 => Self::encode_wpc1252(ch),
            Self::Pc850 => Self::encode_pc850(ch),
            Self::Pc858 => Self::encode_pc858(ch),
            Self::Pc437 => Self::encode_pc437(ch),
            Self::Pc866 => Self::encode_pc866(ch),
            Self::Wpc1256 => Self::encode_wpc1256(ch),
            Self::Iso8859_15 => Self::encode_iso8859_15(ch),
            _ => None,
        }
    }

    /// Encodes a Unicode string slice into a byte vector for this code page.
    ///
    /// Characters that cannot be represented in the active code page are replaced
    /// with an ASCII question mark (`b'?'`).
    #[must_use]
    pub fn encode_text(&self, text: &str) -> Vec<u8> {
        let mut out = Vec::with_capacity(text.len());
        for ch in text.chars() {
            if let Some(b) = self.encode_char(ch) {
                out.push(b);
            } else {
                out.push(b'?');
            }
        }
        out
    }

    fn encode_wpc1252(ch: char) -> Option<u8> {
        match ch {
            '€' => Some(0x80),
            '‚' => Some(0x82),
            'ƒ' => Some(0x83),
            '„' => Some(0x84),
            '…' => Some(0x85),
            '†' => Some(0x86),
            '‡' => Some(0x87),
            'ˆ' => Some(0x88),
            '‰' => Some(0x89),
            'Š' => Some(0x8A),
            '‹' => Some(0x8B),
            'Œ' => Some(0x8C),
            'Ž' => Some(0x8E),
            '‘' => Some(0x91),
            '’' => Some(0x92),
            '“' => Some(0x93),
            '”' => Some(0x94),
            '•' => Some(0x95),
            '–' => Some(0x96),
            '—' => Some(0x97),
            '˜' => Some(0x98),
            '™' => Some(0x99),
            'š' => Some(0x9A),
            '›' => Some(0x9B),
            'œ' => Some(0x9C),
            'ž' => Some(0x9E),
            'Ÿ' => Some(0x9F),
            c if (c as u32) >= 0x00A0 && (c as u32) <= 0x00FF => Some(c as u8),
            _ => None,
        }
    }

    fn encode_pc858(ch: char) -> Option<u8> {
        if ch == '€' {
            return Some(0xD5);
        }
        Self::encode_pc850(ch)
    }

    fn encode_pc850(ch: char) -> Option<u8> {
        match ch {
            'Ç' => Some(0x80),
            'ü' => Some(0x81),
            'é' => Some(0x82),
            'â' => Some(0x83),
            'ä' => Some(0x84),
            'à' => Some(0x85),
            'å' => Some(0x86),
            'ç' => Some(0x87),
            'ê' => Some(0x88),
            'ë' => Some(0x89),
            'è' => Some(0x8A),
            'ï' => Some(0x8B),
            'î' => Some(0x8C),
            'ì' => Some(0x8D),
            'Ä' => Some(0x8E),
            'Å' => Some(0x8F),
            'É' => Some(0x90),
            'æ' => Some(0x91),
            'Æ' => Some(0x92),
            'ô' => Some(0x93),
            'ö' => Some(0x94),
            'ò' => Some(0x95),
            'û' => Some(0x96),
            'ù' => Some(0x97),
            'ÿ' => Some(0x98),
            'Ö' => Some(0x99),
            'Ü' => Some(0x9A),
            'ø' => Some(0x9B),
            '£' => Some(0x9C),
            'Ø' => Some(0x9D),
            '×' => Some(0x9E),
            'ƒ' => Some(0x9F),
            'á' => Some(0xA0),
            'í' => Some(0xA1),
            'ó' => Some(0xA2),
            'ú' => Some(0xA3),
            'ñ' => Some(0xA4),
            'Ñ' => Some(0xA5),
            'ª' => Some(0xA6),
            'º' => Some(0xA7),
            '¿' => Some(0xA8),
            '®' => Some(0xA9),
            '¬' => Some(0xAA),
            '½' => Some(0xAB),
            '¼' => Some(0xAC),
            '¡' => Some(0xAD),
            '«' => Some(0xAE),
            '»' => Some(0xAF),
            '¥' => Some(0xBE),
            'Á' => Some(0xB5),
            'Â' => Some(0xB6),
            'À' => Some(0xB7),
            'ã' => Some(0xC6),
            'Ã' => Some(0xC7),
            'Ê' => Some(0xD2),
            'Ë' => Some(0xD3),
            'È' => Some(0xD4),
            'Í' => Some(0xD6),
            'Î' => Some(0xD7),
            'Ï' => Some(0xD8),
            'Ì' => Some(0xDE),
            'Ó' => Some(0xE0),
            'ß' => Some(0xE1),
            'Ô' => Some(0xE2),
            'Ò' => Some(0xE3),
            'õ' => Some(0xE4),
            'Õ' => Some(0xE5),
            'µ' => Some(0xE6),
            'Ú' => Some(0xE9),
            'Û' => Some(0xEA),
            'Ù' => Some(0xEB),
            'ý' => Some(0xEC),
            'Ý' => Some(0xED),
            '¯' => Some(0xEE),
            '´' => Some(0xEF),
            '±' => Some(0xF1),
            '¾' => Some(0xF3),
            '¶' => Some(0xF4),
            '§' => Some(0xF5),
            '÷' => Some(0xF6),
            '°' => Some(0xF8),
            '¨' => Some(0xF9),
            '·' => Some(0xFA),
            '¹' => Some(0xFB),
            '³' => Some(0xFC),
            '²' => Some(0xFD),
            _ => None,
        }
    }

    fn encode_pc437(ch: char) -> Option<u8> {
        match ch {
            'Ç' => Some(0x80),
            'ü' => Some(0x81),
            'é' => Some(0x82),
            'â' => Some(0x83),
            'ä' => Some(0x84),
            'à' => Some(0x85),
            'å' => Some(0x86),
            'ç' => Some(0x87),
            'ê' => Some(0x88),
            'ë' => Some(0x89),
            'è' => Some(0x8A),
            'ï' => Some(0x8B),
            'î' => Some(0x8C),
            'ì' => Some(0x8D),
            'Ä' => Some(0x8E),
            'Å' => Some(0x8F),
            'É' => Some(0x90),
            'æ' => Some(0x91),
            'Æ' => Some(0x92),
            'ô' => Some(0x93),
            'ö' => Some(0x94),
            'ò' => Some(0x95),
            'û' => Some(0x96),
            'ù' => Some(0x97),
            'ÿ' => Some(0x98),
            'Ö' => Some(0x99),
            'Ü' => Some(0x9A),
            '¢' => Some(0x9B),
            '£' => Some(0x9C),
            '¥' => Some(0x9D),
            '₧' => Some(0x9E),
            'ƒ' => Some(0x9F),
            'á' => Some(0xA0),
            'í' => Some(0xA1),
            'ó' => Some(0xA2),
            'ú' => Some(0xA3),
            'ñ' => Some(0xA4),
            'Ñ' => Some(0xA5),
            'ª' => Some(0xA6),
            'º' => Some(0xA7),
            '¿' => Some(0xA8),
            '⌐' => Some(0xA9),
            '¬' => Some(0xAA),
            '½' => Some(0xAB),
            '¼' => Some(0xAC),
            '¡' => Some(0xAD),
            '«' => Some(0xAE),
            '»' => Some(0xAF),
            'α' => Some(0xE0),
            'ß' => Some(0xE1),
            'Γ' => Some(0xE2),
            'π' => Some(0xE3),
            'Σ' => Some(0xE4),
            'σ' => Some(0xE5),
            'µ' => Some(0xE6),
            'τ' => Some(0xE7),
            'Φ' => Some(0xE8),
            'Θ' => Some(0xE9),
            'Ω' => Some(0xEA),
            'δ' => Some(0xEB),
            '∞' => Some(0xEC),
            'φ' => Some(0xED),
            'ε' => Some(0xEE),
            '∩' => Some(0xEF),
            '≡' => Some(0xF0),
            '±' => Some(0xF1),
            '≥' => Some(0xF2),
            '≤' => Some(0xF3),
            '⌠' => Some(0xF4),
            '⌡' => Some(0xF5),
            '÷' => Some(0xF6),
            '≈' => Some(0xF7),
            '°' => Some(0xF8),
            '·' => Some(0xFA),
            '√' => Some(0xFB),
            'ⁿ' => Some(0xFC),
            '²' => Some(0xFD),
            _ => None,
        }
    }

    fn encode_pc866(ch: char) -> Option<u8> {
        let code = ch as u32;
        // Russian Capital Letters А..П (U+0410..=U+041F) -> 0x80..=0x8F
        if (0x0410..=0x041F).contains(&code) {
            return Some((0x80 + (code - 0x0410)) as u8);
        }
        // Russian Capital Letters Р..Я (U+0420..=U+042F) -> 0x90..=0x9F
        if (0x0420..=0x042F).contains(&code) {
            return Some((0x90 + (code - 0x0420)) as u8);
        }
        // Russian Small Letters а..п (U+0430..=U+043F) -> 0xA0..=0xAF
        if (0x0430..=0x043F).contains(&code) {
            return Some((0xA0 + (code - 0x0430)) as u8);
        }
        // Russian Small Letters р..я (U+0440..=U+044F) -> 0xE0..=0xEF
        if (0x0440..=0x044F).contains(&code) {
            return Some((0xE0 + (code - 0x0440)) as u8);
        }
        match ch {
            'Ё' => Some(0xF0),
            'ё' => Some(0xF1),
            '°' => Some(0xF8),
            '·' => Some(0xFA),
            _ => None,
        }
    }

    fn encode_wpc1256(ch: char) -> Option<u8> {
        let code = ch as u32;
        // Arabic Letters U+0621..=U+063A -> 0xC1..=0xDA
        if (0x0621..=0x063A).contains(&code) {
            return Some((0xC1 + (code - 0x0621)) as u8);
        }
        // Arabic Letters U+0641..=U+064A -> 0xE1..=0xEA
        if (0x0641..=0x064A).contains(&code) {
            return Some((0xE1 + (code - 0x0641)) as u8);
        }
        // Arabic Tatweel U+0640 -> 0xDC
        if code == 0x0640 {
            return Some(0xDC);
        }
        // Arabic Tashkeel U+064B..=U+0652 -> 0xEB..=0xF2
        if (0x064B..=0x0652).contains(&code) {
            return Some((0xEB + (code - 0x064B)) as u8);
        }
        // Arabic-Indic Digits U+0660..=U+0669 -> Map to ASCII 0..9 for standard thermal heads
        if (0x0660..=0x0669).contains(&code) {
            return Some(b'0' + (code - 0x0660) as u8);
        }
        match ch {
            '€' => Some(0x80),
            '£' => Some(0xA3),
            '¥' => Some(0xA5),
            '«' => Some(0xAB),
            '»' => Some(0xBB),
            '؟' => Some(0xBF), // Arabic question mark
            '،' => Some(0xAC), // Arabic comma
            '؛' => Some(0xBA), // Arabic semicolon
            _ => None,
        }
    }

    fn encode_iso8859_15(ch: char) -> Option<u8> {
        match ch {
            '€' => Some(0xA4),
            'Š' => Some(0xA6),
            'š' => Some(0xA8),
            'Ž' => Some(0xB4),
            'ž' => Some(0xB8),
            'Œ' => Some(0xBC),
            'œ' => Some(0xBD),
            'Ÿ' => Some(0xBE),
            c if (c as u32) >= 0x00A0 && (c as u32) <= 0x00FF => Some(c as u8),
            _ => None,
        }
    }
}
