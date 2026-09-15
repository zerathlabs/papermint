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
            Self::Wpc1256 | Self::Custom(33) => Self::encode_wpc1256(ch),
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
        // Official Microsoft CP1256 mapping (Unicode Consortium CP1256.TXT).
        // CP1256 interleaves Arabic letters with Latin characters (e.g. French
        // accented letters), so the Arabic range is NOT contiguous.
        match ch {
            // 0x80–0x9F: Symbols, Latin, and Arabic punctuation
            '€' => Some(0x80),
            'پ' => Some(0x81), // U+067E  Arabic Letter Pe (Persian/Urdu)
            '‚' => Some(0x82),
            'ƒ' => Some(0x83),
            '„' => Some(0x84),
            '…' => Some(0x85),
            '†' => Some(0x86),
            '‡' => Some(0x87),
            'ˆ' => Some(0x88),
            '‰' => Some(0x89),
            'ٹ' => Some(0x8A), // U+0679  Arabic Letter Tteh
            '‹' => Some(0x8B),
            'Œ' => Some(0x8C),
            'چ' => Some(0x8D),        // U+0686  Arabic Letter Tcheh
            'ژ' => Some(0x8E),        // U+0698  Arabic Letter Jeh
            'ڈ' => Some(0x8F),        // U+0688  Arabic Letter Ddal
            'گ' => Some(0x90),        // U+06AF  Arabic Letter Gaf
            '\u{2018}' => Some(0x91), // '
            '\u{2019}' => Some(0x92), // '
            '\u{201C}' => Some(0x93), // "
            '\u{201D}' => Some(0x94), // "
            '•' => Some(0x95),
            '–' => Some(0x96),
            '—' => Some(0x97),
            'ک' => Some(0x98), // U+06A9  Arabic Letter Keheh
            '™' => Some(0x99),
            'ڑ' => Some(0x9A), // U+0691  Arabic Letter Rreh
            '›' => Some(0x9B),
            'œ' => Some(0x9C),
            '\u{200C}' => Some(0x9D), // Zero-width non-joiner
            '\u{200D}' => Some(0x9E), // Zero-width joiner
            'ں' => Some(0x9F),        // U+06BA  Arabic Letter Noon Ghunna

            // 0xA0–0xBF: Latin/Symbols + Arabic punctuation
            '\u{00A0}' => Some(0xA0), // Non-breaking space
            '،' => Some(0xA1),        // U+060C  Arabic Comma
            '¢' => Some(0xA2),
            '£' => Some(0xA3),
            '¤' => Some(0xA4),
            '¥' => Some(0xA5),
            '¦' => Some(0xA6),
            '§' => Some(0xA7),
            '¨' => Some(0xA8),
            '©' => Some(0xA9),
            'ھ' => Some(0xAA), // U+06BE  Arabic Letter Heh Doachashmee
            '«' => Some(0xAB),
            '¬' => Some(0xAC),
            '\u{00AD}' => Some(0xAD), // Soft hyphen
            '®' => Some(0xAE),
            '¯' => Some(0xAF),
            '°' => Some(0xB0),
            '±' => Some(0xB1),
            '²' => Some(0xB2),
            '³' => Some(0xB3),
            '´' => Some(0xB4),
            'µ' => Some(0xB5),
            '¶' => Some(0xB6),
            '·' => Some(0xB7),
            '¸' => Some(0xB8),
            '¹' => Some(0xB9),
            '؛' => Some(0xBA), // U+061B  Arabic Semicolon
            '»' => Some(0xBB),
            '¼' => Some(0xBC),
            '½' => Some(0xBD),
            '¾' => Some(0xBE),
            '؟' => Some(0xBF), // U+061F  Arabic Question Mark

            // 0xC0–0xDB: Arabic letters Hamza through Ghain + Tatweel
            'ہ' => Some(0xC0), // U+06C1  Arabic Letter Heh Goal
            'ء' => Some(0xC1), // U+0621  Hamza
            'آ' => Some(0xC2), // U+0622  Alef with Madda
            'أ' => Some(0xC3), // U+0623  Alef with Hamza Above
            'ؤ' => Some(0xC4), // U+0624  Waw with Hamza Above
            'إ' => Some(0xC5), // U+0625  Alef with Hamza Below
            'ئ' => Some(0xC6), // U+0626  Yeh with Hamza Above
            'ا' => Some(0xC7), // U+0627  Alef
            'ب' => Some(0xC8), // U+0628  Beh
            'ة' => Some(0xC9), // U+0629  Teh Marbuta
            'ت' => Some(0xCA), // U+062A  Teh
            'ث' => Some(0xCB), // U+062B  Theh
            'ج' => Some(0xCC), // U+062C  Jeem
            'ح' => Some(0xCD), // U+062D  Hah
            'خ' => Some(0xCE), // U+062E  Khah
            'د' => Some(0xCF), // U+062F  Dal
            'ذ' => Some(0xD0), // U+0630  Thal
            'ر' => Some(0xD1), // U+0631  Reh
            'ز' => Some(0xD2), // U+0632  Zain
            'س' => Some(0xD3), // U+0633  Seen
            'ش' => Some(0xD4), // U+0634  Sheen
            'ص' => Some(0xD5), // U+0635  Sad
            'ض' => Some(0xD6), // U+0636  Dad
            '×' => Some(0xD7), // U+00D7  Multiplication Sign (NOT Arabic)
            'ط' => Some(0xD8), // U+0637  Tah
            'ظ' => Some(0xD9), // U+0638  Zah
            'ع' => Some(0xDA), // U+0639  Ain
            'غ' => Some(0xDB), // U+063A  Ghain

            // 0xDC–0xEA: Tatweel, Fa-Waw (interleaved with Latin accents)
            'ـ' => Some(0xDC), // U+0640  Tatweel
            'ف' => Some(0xDD), // U+0641  Fa
            'ق' => Some(0xDE), // U+0642  Qaf
            'ك' => Some(0xDF), // U+0643  Kaf
            'à' => Some(0xE0), // U+00E0  Latin Small A with Grave
            'ل' => Some(0xE1), // U+0644  Lam
            'â' => Some(0xE2), // U+00E2  Latin Small A with Circumflex
            'م' => Some(0xE3), // U+0645  Mim
            'ن' => Some(0xE4), // U+0646  Nun
            'ه' => Some(0xE5), // U+0647  Ha
            'و' => Some(0xE6), // U+0648  Waw
            'ç' => Some(0xE7), // U+00E7  Latin Small C with Cedilla
            'è' => Some(0xE8), // U+00E8  Latin Small E with Grave
            'é' => Some(0xE9), // U+00E9  Latin Small E with Acute
            'ê' => Some(0xEA), // U+00EA  Latin Small E with Circumflex

            // 0xEB–0xEF: More Arabic + Latin
            'ë' => Some(0xEB), // U+00EB  Latin Small E with Diaeresis
            'ى' => Some(0xEC), // U+0649  Alef Maksura
            'ي' => Some(0xED), // U+064A  Yeh
            'î' => Some(0xEE), // U+00EE  Latin Small I with Circumflex
            'ï' => Some(0xEF), // U+00EF  Latin Small I with Diaeresis

            // 0xF0–0xFF: Tashkeel (diacritics) interleaved with Latin
            'ً' => Some(0xF0),         // U+064B  Fathatan
            'ٌ' => Some(0xF1),         // U+064C  Dammatan
            'ٍ' => Some(0xF2),         // U+064D  Kasratan
            'َ' => Some(0xF3),         // U+064E  Fatha
            'ô' => Some(0xF4),        // U+00F4  Latin Small O with Circumflex
            'ُ' => Some(0xF5),         // U+064F  Damma
            'ِ' => Some(0xF6),         // U+0650  Kasra
            '÷' => Some(0xF7),        // U+00F7  Division Sign
            'ّ' => Some(0xF8),         // U+0651  Shadda
            'ù' => Some(0xF9),        // U+00F9  Latin Small U with Grave
            'ْ' => Some(0xFA),         // U+0652  Sukun
            'û' => Some(0xFB),        // U+00FB  Latin Small U with Circumflex
            'ü' => Some(0xFC),        // U+00FC  Latin Small U with Diaeresis
            '\u{200E}' => Some(0xFD), // Left-to-right mark
            '\u{200F}' => Some(0xFE), // Right-to-left mark
            'ے' => Some(0xFF),        // U+06D2  Arabic Letter Yeh Barree

            // Arabic-Indic Digits U+0660..=U+0669: not in CP1256,
            // fall back to ASCII digits for thermal printer compatibility.
            '٠' => Some(b'0'),
            '١' => Some(b'1'),
            '٢' => Some(b'2'),
            '٣' => Some(b'3'),
            '٤' => Some(b'4'),
            '٥' => Some(b'5'),
            '٦' => Some(b'6'),
            '٧' => Some(b'7'),
            '٨' => Some(b'8'),
            '٩' => Some(b'9'),

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wpc1256_arabic_encoding() {
        // "فاتورة" per official Microsoft CP1256:
        //   ف U+0641 -> 0xDD
        //   ا U+0627 -> 0xC7
        //   ت U+062A -> 0xCA
        //   و U+0648 -> 0xE6
        //   ر U+0631 -> 0xD1
        //   ة U+0629 -> 0xC9
        let text = "فاتورة";
        let expected = vec![0xDD, 0xC7, 0xCA, 0xE6, 0xD1, 0xC9];

        let encoded = CodePage::Wpc1256.encode_text(text);
        assert_eq!(encoded, expected);

        // OEM Custom Table 33 (used by PosBox PB800 / Rongta / Xprinter)
        // routes through the same WPC1256 encoder.
        let encoded_custom33 = CodePage::Custom(33).encode_text(text);
        assert_eq!(encoded_custom33, expected);
    }

    #[test]
    fn test_wpc1256_arabic_punctuation() {
        // Arabic comma U+060C -> 0xA1
        assert_eq!(CodePage::Wpc1256.encode_char('،'), Some(0xA1));
        // Arabic semicolon U+061B -> 0xBA
        assert_eq!(CodePage::Wpc1256.encode_char('؛'), Some(0xBA));
        // Arabic question mark U+061F -> 0xBF
        assert_eq!(CodePage::Wpc1256.encode_char('؟'), Some(0xBF));
    }

    #[test]
    fn test_wpc1256_tashkeel() {
        // Fathatan U+064B -> 0xF0
        assert_eq!(CodePage::Wpc1256.encode_char('\u{064B}'), Some(0xF0));
        // Dammatan U+064C -> 0xF1
        assert_eq!(CodePage::Wpc1256.encode_char('\u{064C}'), Some(0xF1));
        // Kasratan U+064D -> 0xF2
        assert_eq!(CodePage::Wpc1256.encode_char('\u{064D}'), Some(0xF2));
        // Fatha U+064E -> 0xF3
        assert_eq!(CodePage::Wpc1256.encode_char('\u{064E}'), Some(0xF3));
        // Damma U+064F -> 0xF5
        assert_eq!(CodePage::Wpc1256.encode_char('\u{064F}'), Some(0xF5));
        // Kasra U+0650 -> 0xF6
        assert_eq!(CodePage::Wpc1256.encode_char('\u{0650}'), Some(0xF6));
        // Shadda U+0651 -> 0xF8
        assert_eq!(CodePage::Wpc1256.encode_char('\u{0651}'), Some(0xF8));
        // Sukun U+0652 -> 0xFA
        assert_eq!(CodePage::Wpc1256.encode_char('\u{0652}'), Some(0xFA));
    }

    #[test]
    fn test_wpc1256_persian_urdu_letters() {
        // پ U+067E -> 0x81
        assert_eq!(CodePage::Wpc1256.encode_char('پ'), Some(0x81));
        // چ U+0686 -> 0x8D
        assert_eq!(CodePage::Wpc1256.encode_char('چ'), Some(0x8D));
        // ژ U+0698 -> 0x8E
        assert_eq!(CodePage::Wpc1256.encode_char('ژ'), Some(0x8E));
        // گ U+06AF -> 0x90
        assert_eq!(CodePage::Wpc1256.encode_char('گ'), Some(0x90));
        // ک U+06A9 -> 0x98
        assert_eq!(CodePage::Wpc1256.encode_char('ک'), Some(0x98));
    }

    #[test]
    fn test_wpc1256_arabic_indic_digits_fallback() {
        // Arabic-Indic digits are NOT in CP1256, we fall back to ASCII.
        let text = "٠١٢٣٤٥٦٧٨٩";
        let expected = b"0123456789".to_vec();
        let encoded = CodePage::Wpc1256.encode_text(text);
        assert_eq!(encoded, expected);
    }
}
