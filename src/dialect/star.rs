//! Star Micronics (StarPRNT / Line Mode) dialect implementation.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::RwLock;

use crate::codepage::CodePage;
use crate::command::{
    Alignment, BarcodeData, BarcodeSystem, Command, CutMode, DrawerPin, FontFamily,
    ImageData, PrintDensity, QrCorrectionLevel, QrData, UnderlineMode,
};
use crate::dialect::Dialect;
use crate::error::{PapermintError, Result};
use crate::status::{CoverStatus, DrawerStatus, PaperStatus, PrinterStatus};

// Standard Star Control Characters
const ESC: u8 = 0x1B;
const GS: u8 = 0x1D;
const RS: u8 = 0x1E;
const LF: u8 = 0x0A;

/// Star Micronics dialect encoder (StarPRNT / Line Mode).
///
/// Translates abstract [`Command`] variants into Star Micronics command codes
/// supported by the TSP100, TSP650, TSP700, and mC-Print series.
#[derive(Debug)]
pub struct Star {
    /// Character height multiplier (0 = 1x, 1 = 2x).
    char_height: AtomicU8,
    /// Character width multiplier (0 = 1x, 1 = 2x).
    char_width: AtomicU8,
    /// Active code page for transcoding Unicode characters into 8-bit wire bytes.
    active_codepage: RwLock<CodePage>,
}

impl Default for Star {
    fn default() -> Self {
        Self::new()
    }
}

impl Star {
    /// Creates a new [`Star`] dialect encoder with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            char_height: AtomicU8::new(0),
            char_width: AtomicU8::new(0),
            active_codepage: RwLock::new(CodePage::Pc437),
        }
    }

    /// Encodes a 1D barcode using Star command: `ESC b <type> <text_pos> <width> <height> <data> RS`.
    fn encode_barcode(&self, data: &BarcodeData, buf: &mut Vec<u8>) -> Result<()> {
        let payload = data.data.as_bytes();

        // Star Line Mode barcode types (0 <= n1 <= 8):
        // 0: UPC-E, 1: UPC-A, 2: JAN/EAN-8, 3: JAN/EAN-13, 4: CODE 39,
        // 5: ITF, 6: CODE 128, 7: CODE 93, 8: NW-7 (CODABAR)
        let bar_type: u8 = match data.system {
            BarcodeSystem::UpcE => 0,
            BarcodeSystem::UpcA => 1,
            BarcodeSystem::Ean8 => 2,
            BarcodeSystem::Ean13 => 3,
            BarcodeSystem::Code39 => 4,
            BarcodeSystem::Itf => 5,
            BarcodeSystem::Code128 => 6,
            BarcodeSystem::Code93 => 7,
            BarcodeSystem::Codabar => 8,
        };

        // For Code 128, auto-prefix {B for standard alphanumeric ASCII if not already specified.
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
        if total_len > 255 {
            return Err(PapermintError::InvalidCommand(format!(
                "barcode data too long for Star printer: {} bytes (max 255)",
                total_len
            )));
        }

        // Text position under barcode: 2 = below with line feed
        let text_pos: u8 = 2;

        // Barcode module width: 1 = small, 2 = medium, 3 = large
        let width_code: u8 = data.width.clamp(1, 3);

        // Barcode height in dots
        let height: u8 = if data.height == 0 { 64 } else { data.height };

        // ESC b n1 n2 n3 n4 d1...dk RS
        buf.extend_from_slice(&[ESC, b'b', bar_type, text_pos, width_code, height]);
        buf.extend_from_slice(prefix);
        buf.extend_from_slice(payload_bytes);
        buf.push(RS); // End of barcode data

        Ok(())
    }

    /// Encodes a 2D QR code using the Star 5-step sequence: `ESC GS y S ...`.
    fn encode_qr(&self, data: &QrData, buf: &mut Vec<u8>) -> Result<()> {
        let payload = data.data.as_bytes();
        let payload_len = payload.len();

        if payload_len > 0xFFFF {
            return Err(PapermintError::InvalidCommand(
                "QR code data exceeds 65535 bytes".to_string(),
            ));
        }

        let p_l = u8::try_from(payload_len & 0xFF).unwrap_or(0);
        let p_h = u8::try_from((payload_len >> 8) & 0xFF).unwrap_or(0);

        // 1. Set QR Code Model: ESC GS y S 0 n (1 = Model 1, 2 = Model 2)
        let model = if data.model == 1 { 1 } else { 2 };
        buf.extend_from_slice(&[ESC, GS, b'y', b'S', b'0', model]);

        // 2. Set Error Correction Level: ESC GS y S 1 n (0 = L, 1 = M, 2 = Q, 3 = H)
        let ec_code: u8 = match data.correction {
            QrCorrectionLevel::L => 0,
            QrCorrectionLevel::M => 1,
            QrCorrectionLevel::Q => 2,
            QrCorrectionLevel::H => 3,
        };
        buf.extend_from_slice(&[ESC, GS, b'y', b'S', b'1', ec_code]);

        // 3. Set Cell Size: ESC GS y S 2 n (1 to 8 dots)
        let cell_size = data.cell_size.clamp(1, 8);
        buf.extend_from_slice(&[ESC, GS, b'y', b'S', b'2', cell_size]);

        // 4. Store Data in Buffer: ESC GS y D 1 0 pL pH <data>
        buf.extend_from_slice(&[ESC, GS, b'y', b'D', b'1', 0x00, p_l, p_h]);
        buf.extend_from_slice(payload);

        // 5. Print Stored QR Symbol: ESC GS y P
        buf.extend_from_slice(&[ESC, GS, b'y', b'P']);

        Ok(())
    }

    /// Encodes a monochrome 1-bit-per-pixel raster bitmap image using StarPRNT Raster Mode.
    fn encode_image(&self, img: &ImageData, buf: &mut Vec<u8>) -> Result<()> {
        if img.width == 0 || img.height == 0 {
            return Ok(());
        }

        let width_bytes = img.width.div_ceil(8) as usize;
        let expected_len = width_bytes.checked_mul(img.height as usize).ok_or_else(|| {
            PapermintError::InvalidCommand("image dimensions cause integer overflow".to_string())
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

        // StarPRNT Raster Graphics Mode:
        // 1. Enter raster mode: ESC * r A
        // 2. Transfer line data with feed: ESC * r b n1 n2 d1...dk
        // 3. Quit raster mode: ESC * r B
        buf.extend_from_slice(&[ESC, b'*', b'r', b'A']);

        let n1 = (width_bytes & 0xFF) as u8;
        let n2 = ((width_bytes >> 8) & 0xFF) as u8;

        for row in 0..img.height as usize {
            let start = row * width_bytes;
            let end = start + width_bytes;
            buf.extend_from_slice(&[ESC, b'*', b'r', b'b', n1, n2]);
            buf.extend_from_slice(&img.pixels[start..end]);
        }

        buf.extend_from_slice(&[ESC, b'*', b'r', b'B']);

        Ok(())
    }

    fn update_char_size(&self, buf: &mut Vec<u8>) {
        let h = self.char_height.load(Ordering::Relaxed);
        let w = self.char_width.load(Ordering::Relaxed);
        // Star character size command: ESC i n1 n2 (0 = normal, 1 = 2x)
        buf.extend_from_slice(&[ESC, b'i', h, w]);
    }
}

impl Dialect for Star {
    fn name(&self) -> &'static str {
        "StarPRNT"
    }

    fn encode(&self, command: &Command, buf: &mut Vec<u8>) -> Result<()> {
        match command {
            Command::Init => {
                self.char_height.store(0, Ordering::Relaxed);
                self.char_width.store(0, Ordering::Relaxed);
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
                n => {
                    // Star feeds via repeated LF
                    buf.extend(std::iter::repeat_n(LF, n as usize));
                }
            },

            Command::Cut(mode) => {
                let cut_code = match mode {
                    CutMode::Full => 0x02,    // Feed and full cut
                    CutMode::Partial => 0x03, // Feed and partial cut
                };
                // ESC d n
                buf.extend_from_slice(&[ESC, b'd', cut_code]);
            }

            Command::Align(alignment) => {
                let code = match alignment {
                    Alignment::Left => 0x00,
                    Alignment::Center => 0x01,
                    Alignment::Right => 0x02,
                };
                // ESC GS a n
                buf.extend_from_slice(&[ESC, GS, b'a', code]);
            }

            Command::Bold(enable) => {
                if *enable {
                    // ESC E: Bold ON
                    buf.extend_from_slice(&[ESC, b'E']);
                } else {
                    // ESC F: Bold OFF
                    buf.extend_from_slice(&[ESC, b'F']);
                }
            }

            Command::Underline(mode) => {
                let code = match mode {
                    UnderlineMode::Off => 0x00,
                    UnderlineMode::Single => 0x01,
                    UnderlineMode::Double => 0x02,
                };
                // ESC - n
                buf.extend_from_slice(&[ESC, b'-', code]);
            }

            Command::Invert(enable) => {
                if *enable {
                    // ESC 4: Invert ON
                    buf.extend_from_slice(&[ESC, b'4']);
                } else {
                    // ESC 5: Invert OFF
                    buf.extend_from_slice(&[ESC, b'5']);
                }
            }

            Command::DoubleHeight(enable) => {
                self.char_height
                    .store(if *enable { 1 } else { 0 }, Ordering::Relaxed);
                self.update_char_size(buf);
            }

            Command::DoubleWidth(enable) => {
                self.char_width
                    .store(if *enable { 1 } else { 0 }, Ordering::Relaxed);
                self.update_char_size(buf);
            }

            Command::Font(family) => {
                let code = match family {
                    FontFamily::A => 0x00,
                    FontFamily::B => 0x01,
                };
                // ESC RS F n
                buf.extend_from_slice(&[ESC, RS, b'F', code]);
            }

            Command::DrawerKick(pin) => {
                match pin {
                    // Drive circuit 1 (Pin 2): ESC BEL n1 n2 BEL
                    DrawerPin::Pin2 => buf.extend_from_slice(&[ESC, 0x07, 11, 55, 0x07]),
                    // Drive circuit 2 (Pin 5): ESC FS n1 n2 FS
                    DrawerPin::Pin5 => buf.extend_from_slice(&[ESC, 0x1C, 11, 55, 0x1C]),
                }
            }

            Command::Beep { .. } => {
                // ESC BEL (Hardware buzzer trigger)
                buf.extend_from_slice(&[ESC, 0x07]);
            }

            Command::LineSpacing(spacing) => match spacing {
                None => buf.extend_from_slice(&[ESC, b'z', 1]),
                Some(dots) => buf.extend_from_slice(&[ESC, b'3', *dots]),
            },

            Command::UpsideDown(enable) => {
                if *enable {
                    buf.push(0x0F); // SI: Upside-down printing ON
                } else {
                    buf.push(0x12); // DC2: Upside-down printing OFF
                }
            }

            Command::CodePage(page) => {
                if let Ok(mut cp) = self.active_codepage.write() {
                    *cp = *page;
                }
                let code = match page {
                    CodePage::Pc437 => 1,
                    CodePage::Katakana => 2,
                    CodePage::Pc850 => 0,
                    CodePage::Pc860 => 6,
                    CodePage::Pc863 => 8,
                    CodePage::Pc865 => 9,
                    CodePage::Wpc1252 => 32,
                    CodePage::Pc866 => 10,
                    CodePage::Pc852 => 5,
                    CodePage::Pc858 => 4,
                    CodePage::Pc720 => 72,
                    CodePage::Pc864 => 14,
                    CodePage::Pc737 => 15,
                    CodePage::Wpc1250 => 33,
                    CodePage::Wpc1251 => 34,
                    CodePage::Wpc1253 => 18,
                    CodePage::Wpc1254 => 12,
                    CodePage::Wpc1255 => 13,
                    CodePage::Wpc1256 => 72,
                    CodePage::Wpc1257 => 19,
                    CodePage::Wpc1258 => 22,
                    CodePage::Pc857 => 69,
                    CodePage::Iso8859_15 => 40,
                    CodePage::Pc874 => 22,
                    CodePage::Custom(n) => *n,
                };
                // ESC GS t n: Select character code table
                buf.extend_from_slice(&[ESC, GS, b't', code]);
            }

            Command::InternationalCharset(charset) => {
                // ESC R n: Select international character set
                buf.extend_from_slice(&[ESC, b'R', charset.code()]);
            }

            Command::PrintDensity(density) => {
                // Star Line Mode: ESC GS # '0' n LF NUL
                let n = match density {
                    PrintDensity::Light => b'1',
                    PrintDensity::Normal => b'3',
                    PrintDensity::Dark => b'4',
                    PrintDensity::HighContrast => b'5',
                    PrintDensity::Custom(val) => *val,
                };
                buf.extend_from_slice(&[ESC, GS, b'#', b'0', n, 0x0A, 0x00]);
            }

            Command::HeatingParameters(params) => {
                // ESC 7 n1 n2 n3 mechanism strobe
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

    fn status_query_command(&self) -> Vec<u8> {
        // ENQ (0x05): Star real-time status inquiry
        vec![0x05]
    }

    fn expected_status_bytes(&self) -> usize {
        1
    }

    fn parse_status_response(&self, bytes: &[u8]) -> Result<PrinterStatus> {
        if bytes.is_empty() {
            return Err(PapermintError::Dialect("empty status response from Star printer".into()));
        }

        let mut status = PrinterStatus::default();

        if bytes.len() >= 3 {
            // Multi-byte ASB (Auto Status Back) Frame
            let b1 = bytes[0];
            let b2 = bytes[1];
            let b3 = bytes[2];

            status.is_online = (b1 & 0x08) == 0;
            status.cover = if (b1 & 0x20) != 0 {
                CoverStatus::Open
            } else {
                CoverStatus::Closed
            };
            status.drawer = if (b1 & 0x04) != 0 {
                DrawerStatus::Open
            } else {
                DrawerStatus::Closed
            };

            status.cutter_error = (b2 & 0x08) != 0;
            status.head_overheated = (b2 & 0x40) != 0;

            if (b1 & 0x40) != 0 {
                status.paper = PaperStatus::Empty;
            } else if (b3 & 0x0C) != 0 {
                status.paper = PaperStatus::NearEnd;
            } else {
                status.paper = PaperStatus::Adequate;
            }
        } else {
            // Single-byte ENQ response (0x05)
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
            status.paper = if (b & 0x40) != 0 {
                PaperStatus::Empty
            } else {
                PaperStatus::Adequate
            };
        }

        Ok(status)
    }
}

