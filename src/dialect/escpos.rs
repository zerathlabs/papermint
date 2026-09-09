//! Standard Epson ESC/POS dialect implementation.
//!
//! Converts abstract [`Command`] variants into standard ESC/POS byte sequences.
//! State like character dimensions (double width/height) is maintained atomically
//! so that [`Dialect::encode`] can remain concurrent and thread-safe.

use std::sync::RwLock;
use std::sync::atomic::{AtomicU8, Ordering};

use crate::codepage::CodePage;
use crate::command::{
    Alignment, BarcodeData, BarcodeSystem, Command, CutMode, DrawerPin, FontFamily, ImageData,
    PrintDensity, QrCorrectionLevel, QrData, UnderlineMode,
};
use crate::dialect::Dialect;
use crate::error::{PapermintError, Result};
use crate::status::{CoverStatus, DrawerStatus, PaperStatus, PrinterStatus};

// Standard ESC/POS ASCII Control Characters
const LF: u8 = 0x0A; // Line Feed (Newline '\n')
const DLE: u8 = 0x10; // Data Link Escape (Real-time status transmission DLE EOT)
const EOT: u8 = 0x04; // End of Transmission (Real-time status transmission DLE EOT)
const ESC: u8 = 0x1B; // Escape (Lead byte for standard printer commands)
const GS: u8 = 0x1D; // Group Separator (Lead byte for advanced POS commands)

/// Standard Epson ESC/POS dialect encoder.
///
/// Converts abstract [`Command`] variants into standard ESC/POS byte sequences.
/// State like character dimensions (double width/height) is maintained atomically
/// so that [`Dialect::encode`] can remain concurrent and thread-safe.
#[derive(Debug)]
pub struct EscPos {
    /// Bitmask for character dimensions via `GS ! n`:
    /// - bits 0–3: height multiplier (0 = 1x, 1 = 2x)
    /// - bits 4–7: width multiplier (0 = 1x, 1 = 2x)
    char_size_mask: AtomicU8,
    /// Active code page for transcoding Unicode characters into 8-bit wire bytes.
    active_codepage: RwLock<CodePage>,
}

impl Default for EscPos {
    fn default() -> Self {
        Self::new()
    }
}

impl EscPos {
    /// Creates a new [`EscPos`] dialect encoder with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            char_size_mask: AtomicU8::new(0),
            active_codepage: RwLock::new(CodePage::Pc437),
        }
    }

    /// Encodes a 1D barcode into ESC/POS Function B commands (`GS h`, `GS w`, `GS k`).
    ///
    /// # Byte Sequences
    /// - Height: `GS h n` (`1D 68 n`) where 1 <= n <= 255
    /// - Width:  `GS w n` (`1D 77 n`) where 2 <= n <= 6
    /// - Data:   `GS k m n d1..dk` (`1D 6B m n d1..dk`) (Function B format)
    fn encode_barcode(&self, data: &BarcodeData, buf: &mut Vec<u8>) -> Result<()> {
        let payload = data.data.as_bytes();

        // Modern ESC/POS Function B barcode system codes:
        // GS k m n d1..dk
        let system_code = match data.system {
            BarcodeSystem::UpcA => 65,
            BarcodeSystem::UpcE => 66,
            BarcodeSystem::Ean13 => 67,
            BarcodeSystem::Ean8 => 68,
            BarcodeSystem::Code39 => 69,
            BarcodeSystem::Itf => 70,
            BarcodeSystem::Codabar => 71,
            BarcodeSystem::Code93 => 72,
            BarcodeSystem::Code128 => 73,
        };

        // ESC/POS Code 128 requires an initial Code Set selector (e.g., {B for Code Set B).
        // If not present in the user payload, auto-prefix {B for standard alphanumeric ASCII.
        let (prefix, payload_bytes) = match data.system {
            BarcodeSystem::Code128
                if !payload.starts_with(b"{A")
                    && !payload.starts_with(b"{B")
                    && !payload.starts_with(b"{C") =>
            {
                (&b"{B"[..], payload)
            }
            _ => (&[][..], payload),
        };

        let total_len = prefix.len() + payload_bytes.len();
        let len = u8::try_from(total_len).map_err(|_| {
            PapermintError::InvalidCommand(format!(
                "barcode data too long: {} bytes (max 255)",
                total_len
            ))
        })?;

        // Barcode height: GS h n (1–255)
        let height = if data.height == 0 { 64 } else { data.height };
        buf.extend_from_slice(&[GS, b'h', height]);

        // Barcode module width: GS w n (2–6)
        let width = data.width.clamp(2, 6);
        buf.extend_from_slice(&[GS, b'w', width]);

        buf.extend_from_slice(&[GS, b'k', system_code, len]);
        buf.extend_from_slice(prefix);
        buf.extend_from_slice(payload_bytes);
        Ok(())
    }

    /// Encodes a 2D QR code using the standard Epson 5-step sequence (`GS ( k ...`).
    ///
    /// # Byte Sequences
    /// 1. Select Model (Function 165): `GS ( k 4 0 49 65 n1 n2` (`1D 28 6B 04 00 31 41 n1 n2`)
    /// 2. Set Module Size (Function 167): `GS ( k 3 0 49 67 n` (`1D 28 6B 03 00 31 43 n`)
    /// 3. Set Error Correction (Function 169): `GS ( k 3 0 49 69 n` (`1D 28 6B 03 00 31 45 n`)
    /// 4. Store Data in Buffer (Function 180): `GS ( k pL pH 49 80 48 d1..dk` (`1D 28 6B pL pH 31 50 30 d1..dk`)
    /// 5. Print Stored Symbol (Function 181): `GS ( k 3 0 49 81 48` (`1D 28 6B 03 00 31 51 30`)
    fn encode_qr(&self, data: &QrData, buf: &mut Vec<u8>) -> Result<()> {
        let payload = data.data.as_bytes();
        let payload_len = payload.len();

        // Length header: payload + 3 header bytes (fn 49, 80, 48)
        let store_len = payload_len.checked_add(3).ok_or_else(|| {
            PapermintError::InvalidCommand("QR code payload too large".to_string())
        })?;

        let (p_l, p_h) = if store_len <= 0xFFFF {
            let p_l = u8::try_from(store_len & 0xFF).unwrap_or(0);
            let p_h = u8::try_from((store_len >> 8) & 0xFF).unwrap_or(0);
            (p_l, p_h)
        } else {
            return Err(PapermintError::InvalidCommand(
                "QR code data exceeds 65535 bytes".to_string(),
            ));
        };

        // 1. Select QR Model (Function 165): GS ( k 4 0 49 65 n1 n2
        let model = if data.model == 1 { 49 } else { 50 }; // 49 = Model 1, 50 = Model 2
        buf.extend_from_slice(&[GS, b'(', b'k', 4, 0, 49, 65, model, 0]);

        // 2. Set Module Size (Function 167): GS ( k 3 0 49 67 n
        let cell_size = data.cell_size.clamp(1, 16);
        buf.extend_from_slice(&[GS, b'(', b'k', 3, 0, 49, 67, cell_size]);

        // 3. Set Error Correction (Function 169): GS ( k 3 0 49 69 n
        let ec_code = match data.correction {
            QrCorrectionLevel::L => 48,
            QrCorrectionLevel::M => 49,
            QrCorrectionLevel::Q => 50,
            QrCorrectionLevel::H => 51,
        };
        buf.extend_from_slice(&[GS, b'(', b'k', 3, 0, 49, 69, ec_code]);

        // 4. Store Data in Symbol Storage Area (Function 180): GS ( k pL pH 49 80 48 <data>
        buf.extend_from_slice(&[GS, b'(', b'k', p_l, p_h, 49, 80, 48]);
        buf.extend_from_slice(payload);

        // 5. Print QR Symbol (Function 181): GS ( k 3 0 49 81 48
        buf.extend_from_slice(&[GS, b'(', b'k', 3, 0, 49, 81, 48]);

        Ok(())
    }

    /// Encodes a monochrome 1-bit-per-pixel raster bitmap image (`GS v 0`).
    ///
    /// # Byte Sequence
    /// - `GS v 0 m xL xH yL yH d1..dk` (`1D 76 30 m xL xH yL yH d1..dk`)
    ///   - `m`: Raster mode (0 = Normal, 1 = Double-width, 2 = Double-height, 3 = Quadruple)
    ///   - `xL, xH`: Number of bytes in horizontal direction (`(width + 7) / 8`)
    ///   - `yL, yH`: Number of dots in vertical direction (`height`)
    ///
    /// # Chunking Rationale
    /// Slices images taller than 960 dots into consecutive `GS v 0` blocks to prevent
    /// buffer overflows in thermal printer volatile RAM (typically 4KB–64KB).
    fn encode_image(&self, img: &ImageData, buf: &mut Vec<u8>) -> Result<()> {
        if img.width == 0 || img.height == 0 {
            return Ok(());
        }

        let width_bytes = img.width.div_ceil(8) as usize;
        let expected_len = width_bytes
            .checked_mul(img.height as usize)
            .ok_or_else(|| {
                PapermintError::InvalidCommand(
                    "image dimensions cause integer overflow".to_string(),
                )
            })?;

        if img.pixels.len() < expected_len {
            return Err(PapermintError::InvalidCommand(format!(
                "image pixel buffer length ({}) is less than expected ({}) for {}x{}",
                img.pixels.len(),
                expected_len,
                img.width,
                img.height
            )));
        }

        // Standard thermal printer safe raster chunk height (dots).
        // Sending images taller than 960 dots in a single GS v 0 command can overflow
        // printer volatile RAM (typically 4KB to 64KB on thermal receipt hardware).
        // Slicing into consecutive 960-dot chunks renders seamlessly on paper
        // with zero vertical gap and eliminates buffer overrun risks.
        const MAX_CHUNK_HEIGHT: usize = 960;

        let total_rows = img.height as usize;
        let x_l = (width_bytes & 0xFF) as u8;
        let x_h = ((width_bytes >> 8) & 0xFF) as u8;

        for chunk_start in (0..total_rows).step_by(MAX_CHUNK_HEIGHT) {
            let chunk_height = (total_rows - chunk_start).min(MAX_CHUNK_HEIGHT);
            let y_l = (chunk_height & 0xFF) as u8;
            let y_h = ((chunk_height >> 8) & 0xFF) as u8;

            let byte_start = chunk_start * width_bytes;
            let byte_end = byte_start + (chunk_height * width_bytes);

            // GS v 0 m xL xH yL yH d1..dk (m: 0 = normal mode)
            buf.extend_from_slice(&[GS, b'v', b'0', 0, x_l, x_h, y_l, y_h]);
            buf.extend_from_slice(&img.pixels[byte_start..byte_end]);
        }

        Ok(())
    }
}

impl Dialect for EscPos {
    fn name(&self) -> &'static str {
        "ESC/POS"
    }

    fn encode(&self, command: &Command, buf: &mut Vec<u8>) -> Result<()> {
        match command {
            Command::Init => {
                // Reset character size mask
                self.char_size_mask.store(0, Ordering::Relaxed);
                if let Ok(mut cp) = self.active_codepage.write() {
                    *cp = CodePage::Pc437;
                }
                // ESC @: Initialize printer
                buf.extend_from_slice(&[ESC, b'@']);
            }

            Command::Text(text) => {
                if text.is_ascii() {
                    buf.extend_from_slice(text.as_bytes());
                } else {
                    let cp = self
                        .active_codepage
                        .read()
                        .map(|c| *c)
                        .unwrap_or(CodePage::Pc437);
                    let encoded = cp.encode_text(text);
                    buf.extend_from_slice(&encoded);
                }
            }

            Command::Feed(lines) => match *lines {
                0 => {}
                1 => buf.push(LF),
                n => buf.extend_from_slice(&[ESC, b'd', n]),
            },

            Command::Cut(mode) => {
                let cut_code = match mode {
                    CutMode::Full => 0x00,
                    CutMode::Partial => 0x01,
                };
                // GS V m
                buf.extend_from_slice(&[GS, b'V', cut_code]);
            }

            Command::Align(alignment) => {
                let code = match alignment {
                    Alignment::Left => 0,
                    Alignment::Center => 1,
                    Alignment::Right => 2,
                };
                // ESC a n
                buf.extend_from_slice(&[ESC, b'a', code]);
            }

            Command::Bold(enable) => {
                // ESC E n
                buf.extend_from_slice(&[ESC, b'E', u8::from(*enable)]);
            }

            Command::Underline(mode) => {
                let code = match mode {
                    UnderlineMode::Off => 0,
                    UnderlineMode::Single => 1,
                    UnderlineMode::Double => 2,
                };
                // ESC - n
                buf.extend_from_slice(&[ESC, b'-', code]);
            }

            Command::Invert(enable) => {
                // GS B n
                buf.extend_from_slice(&[GS, b'B', u8::from(*enable)]);
            }

            Command::DoubleHeight(enable) => {
                let current = self.char_size_mask.load(Ordering::Relaxed);
                let updated = if *enable {
                    current | 0x01 // 2x vertical
                } else {
                    current & !0x01
                };
                self.char_size_mask.store(updated, Ordering::Relaxed);
                // GS ! n
                buf.extend_from_slice(&[GS, b'!', updated]);
            }

            Command::DoubleWidth(enable) => {
                let current = self.char_size_mask.load(Ordering::Relaxed);
                let updated = if *enable {
                    current | 0x10 // 2x horizontal
                } else {
                    current & !0x10
                };
                self.char_size_mask.store(updated, Ordering::Relaxed);
                // GS ! n
                buf.extend_from_slice(&[GS, b'!', updated]);
            }

            Command::Font(family) => {
                let code = match family {
                    FontFamily::A => 0,
                    FontFamily::B => 1,
                };
                // ESC M n
                buf.extend_from_slice(&[ESC, b'M', code]);
            }

            Command::DrawerKick(pin) => {
                let m = match pin {
                    DrawerPin::Pin2 => 0,
                    DrawerPin::Pin5 => 1,
                };
                // ESC p m t1 t2 (pulse on 50ms, pulse off 500ms)
                buf.extend_from_slice(&[ESC, b'p', m, 25, 250]);
            }

            Command::Beep { count, duration } => {
                // ESC B n t
                let n = (*count).clamp(1, 9);
                let t = (*duration).clamp(1, 9);
                buf.extend_from_slice(&[ESC, b'B', n, t]);
            }

            Command::LineSpacing(spacing) => match spacing {
                None => buf.extend_from_slice(&[ESC, b'2']),
                Some(dots) => buf.extend_from_slice(&[ESC, b'3', *dots]),
            },

            Command::UpsideDown(enable) => {
                // ESC { n
                buf.extend_from_slice(&[ESC, b'{', u8::from(*enable)]);
            }

            Command::CodePage(page) => {
                if let Ok(mut cp) = self.active_codepage.write() {
                    *cp = *page;
                }
                let code = match page {
                    CodePage::Pc437 => 0,
                    CodePage::Katakana => 1,
                    CodePage::Pc850 => 2,
                    CodePage::Pc860 => 3,
                    CodePage::Pc863 => 4,
                    CodePage::Pc865 => 5,
                    CodePage::Wpc1252 => 16,
                    CodePage::Pc866 => 17,
                    CodePage::Pc852 => 18,
                    CodePage::Pc858 => 19,
                    CodePage::Pc720 => 32,
                    CodePage::Pc864 => 37,
                    CodePage::Pc737 => 14,
                    CodePage::Wpc1250 => 45,
                    CodePage::Wpc1251 => 46,
                    CodePage::Wpc1253 => 47,
                    CodePage::Wpc1254 => 48,
                    CodePage::Wpc1255 => 49,
                    CodePage::Wpc1256 => 50,
                    CodePage::Wpc1257 => 51,
                    CodePage::Wpc1258 => 52,
                    CodePage::Pc857 => 13,
                    CodePage::Iso8859_15 => 40,
                    CodePage::Pc874 => 20,
                    CodePage::Custom(n) => *n,
                };
                // ESC t n
                buf.extend_from_slice(&[ESC, b't', code]);
            }

            Command::InternationalCharset(charset) => {
                // ESC R n: Select an international character set
                buf.extend_from_slice(&[ESC, b'R', charset.code()]);
            }

            Command::PrintDensity(density) => {
                // GS ( K pL pH fn m: Function 48 (fn = 48 / 0x30, pL = 2, pH = 0)
                let m = match density {
                    PrintDensity::Light => 253,      // -3 (~85%)
                    PrintDensity::Normal => 0,       // 0 (100% standard calibration)
                    PrintDensity::Dark => 3,         // +3 (~115%)
                    PrintDensity::HighContrast => 6, // +6 (~130%)
                    PrintDensity::Custom(val) => *val,
                };
                buf.extend_from_slice(&[GS, b'(', b'K', 0x02, 0x00, 0x30, m]);
            }

            Command::HeatingParameters(params) => {
                // ESC 7 n1 n2 n3: Set thermal heating strobe and timing parameters
                buf.extend_from_slice(&[
                    ESC,
                    b'7',
                    params.max_heating_dots,
                    params.heating_time,
                    params.heating_interval,
                ]);
            }

            Command::Barcode(data) => self.encode_barcode(data, buf)?,

            Command::QrCode(data) => self.encode_qr(data, buf)?,

            Command::Image(img) => self.encode_image(img, buf)?,

            Command::Raw(raw_bytes) => {
                buf.extend_from_slice(raw_bytes);
            }
        }

        Ok(())
    }

    /// Generates real-time status inquiry commands (`DLE EOT n`).
    ///
    /// Transmits a 4-request burst to poll complete hardware state:
    /// - `DLE EOT 1`: Transmit printer status (`10 04 01`) - Drawer kick-out and online status
    /// - `DLE EOT 2`: Transmit offline status (`10 04 02`) - Cover status and paper feed button
    /// - `DLE EOT 3`: Transmit error status (`10 04 03`) - Auto-cutter error and head overheat
    /// - `DLE EOT 4`: Transmit paper roll sensor status (`10 04 04`) - Paper near-end and paper-end
    fn status_query_command(&self) -> Vec<u8> {
        // DLE EOT 1 (Printer) + DLE EOT 2 (Offline) + DLE EOT 3 (Error) + DLE EOT 4 (Paper)
        vec![DLE, EOT, 1, DLE, EOT, 2, DLE, EOT, 3, DLE, EOT, 4]
    }

    fn expected_status_bytes(&self) -> usize {
        4
    }

    fn parse_status_response(&self, bytes: &[u8]) -> Result<PrinterStatus> {
        if bytes.is_empty() {
            return Err(PapermintError::Dialect(
                "empty status response from ESC/POS printer".into(),
            ));
        }

        let mut status = PrinterStatus::default();

        if bytes.len() >= 4 {
            let b1 = bytes[0];
            let b2 = bytes[1];
            let b3 = bytes[2];
            let b4 = bytes[3];

            // DLE EOT 1: Printer status
            status.is_online = (b1 & 0x08) == 0;
            status.drawer = if (b1 & 0x04) != 0 {
                DrawerStatus::Open
            } else {
                DrawerStatus::Closed
            };

            // DLE EOT 2: Offline status
            status.cover = if (b2 & 0x04) != 0 {
                CoverStatus::Open
            } else {
                CoverStatus::Closed
            };

            // DLE EOT 3: Error status
            status.cutter_error = (b3 & 0x08) != 0;
            status.head_overheated = (b3 & 0x40) != 0;

            // DLE EOT 4: Paper roll sensor
            if (b4 & 0x60) != 0 || (b2 & 0x20) != 0 {
                status.paper = PaperStatus::Empty;
            } else if (b4 & 0x0C) != 0 {
                status.paper = PaperStatus::NearEnd;
            } else {
                status.paper = PaperStatus::Adequate;
            }
        } else {
            // Fallback for single-byte responses (GS r 1, ASB byte 1, or single DLE EOT)
            let b = bytes[0];
            status.is_online = (b & 0x08) == 0;
            status.drawer = if (b & 0x04) != 0 {
                DrawerStatus::Open
            } else {
                DrawerStatus::Closed
            };
            status.cover = if (b & 0x20) != 0 {
                CoverStatus::Open
            } else {
                CoverStatus::Closed
            };
            if (b & 0x60) == 0x60 || (b & 0x40) != 0 {
                status.paper = PaperStatus::Empty;
            } else if (b & 0x0C) == 0x0C {
                status.paper = PaperStatus::NearEnd;
            } else {
                status.paper = PaperStatus::Adequate;
            }
        }

        Ok(status)
    }
}
