//! Virtual receipt preview engine.
//!
//! Transforms abstract receipt commands into realistic visual representations
//! (SVG vector graphics and self-contained HTML snippets) for display on web dashboards,
//! POS customer-facing screens, mobile apps, or PDF generation.

use crate::command::{Alignment, BarcodeData, Command, CutMode, FontFamily, ImageData, QrData};
use crate::layout::receipt::Receipt;

/// Internal representation of styled text spans in a receipt line.
#[derive(Debug, Clone, PartialEq, Eq)]
struct StyledSpan {
    text: String,
    bold: bool,
    underline: bool,
    invert: bool,
    double_height: bool,
    double_width: bool,
    font: FontFamily,
}

/// Abstract visual elements extracted from the receipt command stream.
#[derive(Debug, Clone)]
enum PreviewElement {
    Line {
        align: Alignment,
        spans: Vec<StyledSpan>,
    },
    Feed(u8),
    Cut(CutMode),
    QrCode {
        data: String,
        align: Alignment,
    },
    Barcode {
        data: String,
        align: Alignment,
        height: u8,
    },
    Image {
        width: u32,
        height: u32,
        pixels: Vec<u8>,
        align: Alignment,
    },
}

/// Parses receipt commands into high-level preview elements.
fn parse_commands(receipt: &Receipt) -> Vec<PreviewElement> {
    let mut elements = Vec::new();

    let mut current_align = Alignment::Left;
    let mut current_bold = false;
    let mut current_underline = false;
    let mut current_invert = false;
    let mut current_double_height = false;
    let mut current_double_width = false;
    let mut current_font = FontFamily::A;

    let mut current_spans: Vec<StyledSpan> = Vec::new();

    let flush_line = |elements: &mut Vec<PreviewElement>,
                      current_spans: &mut Vec<StyledSpan>,
                      align: Alignment| {
        if !current_spans.is_empty() {
            elements.push(PreviewElement::Line {
                align,
                spans: std::mem::take(current_spans),
            });
        }
    };

    for cmd in receipt.commands() {
        match cmd {
            Command::Init => {
                flush_line(&mut elements, &mut current_spans, current_align);
                current_align = Alignment::Left;
                current_bold = false;
                current_underline = false;
                current_invert = false;
                current_double_height = false;
                current_double_width = false;
                current_font = FontFamily::A;
            }
            Command::Align(align) => {
                if !current_spans.is_empty() {
                    flush_line(&mut elements, &mut current_spans, current_align);
                }
                current_align = *align;
            }
            Command::Bold(bold) => {
                current_bold = *bold;
            }
            Command::Underline(mode) => {
                current_underline = *mode != crate::command::UnderlineMode::Off;
            }
            Command::Invert(inv) => {
                current_invert = *inv;
            }
            Command::DoubleHeight(dh) => {
                current_double_height = *dh;
            }
            Command::DoubleWidth(dw) => {
                current_double_width = *dw;
            }
            Command::Font(f) => {
                current_font = *f;
            }
            Command::Text(text) => {
                let lines: Vec<&str> = text.split('\n').collect();
                for (i, segment) in lines.iter().enumerate() {
                    if !segment.is_empty() {
                        current_spans.push(StyledSpan {
                            text: (*segment).to_string(),
                            bold: current_bold,
                            underline: current_underline,
                            invert: current_invert,
                            double_height: current_double_height,
                            double_width: current_double_width,
                            font: current_font,
                        });
                    }

                    if i + 1 < lines.len() {
                        flush_line(&mut elements, &mut current_spans, current_align);
                    }
                }
            }
            Command::Feed(n) => {
                flush_line(&mut elements, &mut current_spans, current_align);
                elements.push(PreviewElement::Feed(*n));
            }
            Command::Cut(mode) => {
                flush_line(&mut elements, &mut current_spans, current_align);
                elements.push(PreviewElement::Cut(*mode));
            }
            Command::QrCode(QrData { data, .. }) => {
                flush_line(&mut elements, &mut current_spans, current_align);
                elements.push(PreviewElement::QrCode {
                    data: data.clone(),
                    align: current_align,
                });
            }
            Command::Barcode(BarcodeData { data, height, .. }) => {
                flush_line(&mut elements, &mut current_spans, current_align);
                elements.push(PreviewElement::Barcode {
                    data: data.clone(),
                    align: current_align,
                    height: *height,
                });
            }
            Command::Image(ImageData {
                width,
                height,
                pixels,
            }) => {
                flush_line(&mut elements, &mut current_spans, current_align);
                elements.push(PreviewElement::Image {
                    width: *width,
                    height: *height,
                    pixels: pixels.clone(),
                    align: current_align,
                });
            }
            _ => {
                // Non-visual commands ignored
            }
        }
    }

    flush_line(&mut elements, &mut current_spans, current_align);
    elements
}

fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// Generates a deterministic, realistic vector QR code SVG path.
#[allow(clippy::needless_range_loop)]
fn generate_qr_svg_path(data: &str, size: f32) -> String {
    const GRID: usize = 25;
    let mut matrix = vec![vec![false; GRID]; GRID];

    let place_finder = |matrix: &mut Vec<Vec<bool>>, r: usize, c: usize| {
        for dr in 0..7 {
            for dc in 0..7 {
                let is_outer = dr == 0 || dr == 6 || dc == 0 || dc == 6;
                let is_inner = (2..=4).contains(&dr) && (2..=4).contains(&dc);
                matrix[r + dr][c + dc] = is_outer || is_inner;
            }
        }
    };

    // 3 Finder Patterns
    place_finder(&mut matrix, 0, 0);
    place_finder(&mut matrix, 0, GRID - 7);
    place_finder(&mut matrix, GRID - 7, 0);

    // Timing patterns
    for i in 8..(GRID - 8) {
        matrix[6][i] = i % 2 == 0;
        matrix[i][6] = i % 2 == 0;
    }

    // Alignment pattern (center at 18, 18 for 25x25)
    let ar = 16;
    let ac = 16;
    for dr in 0..5 {
        for dc in 0..5 {
            let is_outer = dr == 0 || dr == 4 || dc == 0 || dc == 4;
            let is_center = dr == 2 && dc == 2;
            matrix[ar + dr][ac + dc] = is_outer || is_center;
        }
    }

    // Fill data pattern deterministically using FNV-1a hash of input data
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in data.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }

    let mut state = hash;
    for r in 0..GRID {
        for c in 0..GRID {
            let in_finder1 = r < 8 && c < 8;
            let in_finder2 = r < 8 && c >= (GRID - 8);
            let in_finder3 = r >= (GRID - 8) && c < 8;
            let in_align = (ar..ar + 5).contains(&r) && (ac..ac + 5).contains(&c);
            let in_timing = (r == 6 && c < GRID - 8) || (c == 6 && r < GRID - 8);

            if !in_finder1 && !in_finder2 && !in_finder3 && !in_align && !in_timing {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                matrix[r][c] = (state >> 32) % 100 < 52;
            }
        }
    }

    // Build SVG path
    let cell = size / GRID as f32;
    let mut path = String::with_capacity(4096);
    for r in 0..GRID {
        for c in 0..GRID {
            if matrix[r][c] {
                let x = c as f32 * cell;
                let y = r as f32 * cell;
                path.push_str(&format!("M{x:.1},{y:.1}h{cell:.1}v{cell:.1}h-{cell:.1}Z "));
            }
        }
    }

    path
}

/// Generates a realistic vector barcode SVG pattern.
fn generate_barcode_svg(data: &str, width: f32, height: f32) -> String {
    let mut path = String::new();
    let num_bars = 48.min((data.len() * 4).max(36));
    let bar_width = width / (num_bars as f32 * 1.5);

    let mut hash: u64 = 0xcbf29ce484222325;
    for b in data.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }

    let mut x = 0.0f32;
    for i in 0..num_bars {
        let is_bar = (hash >> (i % 64)) & 1 == 1 || i % 2 == 0;
        let w = if (hash >> ((i + 3) % 64)) & 1 == 1 {
            bar_width * 2.0
        } else {
            bar_width
        };
        if is_bar {
            path.push_str(&format!("M{x:.1},0h{w:.1}v{height:.1}h-{w:.1}Z "));
        }
        x += w + bar_width * 0.5;
        hash = hash.rotate_left(1);
    }

    path
}

/// Renders a [`Receipt`] into an SVG vector graphic.
///
/// Simulates realistic physical thermal receipt paper with accurate character grid
/// dimensions, monospace font styling, vector barcode and QR code blocks, and
/// a serrated bottom tear edge.
///
/// # Example
///
/// ```rust
/// use papermint::{Receipt, PaperWidth};
///
/// let receipt = Receipt::new(PaperWidth::Mm80)
///     .center()
///     .bold(true)
///     .text_ln("MINT BISTRO")
///     .divider('-')
///     .two_column("Coffee", "$3.50")
///     .cut_full();
///
/// let svg = receipt.render_svg();
/// assert!(svg.starts_with("<svg"));
/// assert!(svg.contains("MINT BISTRO"));
/// ```
#[must_use]
pub fn render_svg(receipt: &Receipt) -> String {
    let paper_dots = receipt.paper_width().dots() as f32;
    let elements = parse_commands(receipt);

    let pad_x = 24.0f32;
    let pad_top = 28.0f32;
    let pad_bottom = 36.0f32;

    let normal_line_h = 24.0f32;
    let double_line_h = 44.0f32;

    // First pass: calculate total height
    let mut total_h = pad_top;
    for el in &elements {
        match el {
            PreviewElement::Line { spans, .. } => {
                let has_double = spans.iter().any(|s| s.double_height);
                total_h += if has_double {
                    double_line_h
                } else {
                    normal_line_h
                };
            }
            PreviewElement::Feed(n) => {
                total_h += *n as f32 * normal_line_h;
            }
            PreviewElement::Cut(_) => {
                total_h += 28.0;
            }
            PreviewElement::QrCode { .. } => {
                total_h += 140.0;
            }
            PreviewElement::Barcode { height, .. } => {
                total_h += (*height as f32).max(48.0) + 26.0;
            }
            PreviewElement::Image { height, .. } => {
                total_h += *height as f32 + 16.0;
            }
        }
    }
    total_h += pad_bottom;

    let mut svg = String::with_capacity(8192);
    svg.push_str(&format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {paper_dots:.0} {total_h:.0}" width="{paper_dots:.0}" height="{total_h:.0}" style="font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Courier New', monospace; background-color: transparent;">
  <defs>
    <filter id="receipt-shadow" x="-5%" y="-2%" width="110%" height="106%">
      <feDropShadow dx="0" dy="4" stdDeviation="6" flood-color="#000000" flood-opacity="0.14" />
    </filter>
  </defs>
  <!-- Thermal Paper Body with Drop Shadow -->
  <g filter="url(#receipt-shadow)">
    <path d="M 0 4 Q 0 0 4 0 L {w_minus_4:.0} 0 Q {paper_dots:.0} 0 {paper_dots:.0} 4 L {paper_dots:.0} {h_minus_12:.0} "##,
        w_minus_4 = paper_dots - 4.0,
        h_minus_12 = total_h - 12.0
    ));

    // Jagged Serrated Cut Edge along the bottom
    let tooth_w = 12.0f32;
    let tooth_h = 6.0f32;
    let mut cx = paper_dots;
    let mut step = 0;
    while cx > 0.0 {
        let next_x = (cx - tooth_w).max(0.0);
        let y = if step % 2 == 0 {
            total_h
        } else {
            total_h - tooth_h
        };
        svg.push_str(&format!("L {next_x:.1} {y:.1} "));
        cx = next_x;
        step += 1;
    }
    svg.push_str(
        r##"Z" fill="#ffffff" stroke="#e0e0e0" stroke-width="1" />
  </g>
  <!-- Receipt Content Stream -->
"##,
    );

    // Second pass: render elements
    let mut cur_y = pad_top;
    for el in &elements {
        match el {
            PreviewElement::Line { align, spans } => {
                let has_double = spans.iter().any(|s| s.double_height);
                let line_h = if has_double {
                    double_line_h
                } else {
                    normal_line_h
                };
                let baseline_y = cur_y + line_h * 0.75;

                let (text_x, text_anchor) = match align {
                    Alignment::Left => (pad_x, "start"),
                    Alignment::Center => (paper_dots / 2.0, "middle"),
                    Alignment::Right => (paper_dots - pad_x, "end"),
                };

                svg.push_str(&format!(
                    r##"  <text x="{text_x:.1}" y="{baseline_y:.1}" text-anchor="{text_anchor}""##
                ));

                // If uniform styling across spans, apply to <text>
                if spans.len() == 1 {
                    let s = &spans[0];
                    let font_size = if s.double_height || s.double_width {
                        26
                    } else {
                        16
                    };
                    let font_weight = if s.bold { "bold" } else { "normal" };
                    let decor = if s.underline {
                        "text-decoration=\"underline\" "
                    } else {
                        ""
                    };
                    let fill = if s.invert { "#ffffff" } else { "#111111" };

                    if s.invert {
                        // Invert background rectangle
                        let rect_w = paper_dots - pad_x * 2.0;
                        svg.push_str(&format!(
                            r##"><tspan fill="{fill}" font-size="{font_size}px" font-weight="{font_weight}" {decor}>{text}</tspan></text>
  <rect x="{pad_x:.1}" y="{cur_y:.1}" width="{rect_w:.1}" height="{line_h:.1}" fill="#111111" style="mix-blend-mode: difference;" />
"##,
                            text = escape_xml(&s.text)
                        ));
                        cur_y += line_h;
                        continue;
                    }

                    svg.push_str(&format!(
                        r##" font-size="{font_size}px" font-weight="{font_weight}" fill="{fill}" {decor}>{text}</text>
"##,
                        text = escape_xml(&s.text)
                    ));
                } else {
                    svg.push_str(">\n");
                    for s in spans {
                        let font_size = if s.double_height || s.double_width {
                            26
                        } else {
                            16
                        };
                        let font_weight = if s.bold { "bold" } else { "normal" };
                        let decor = if s.underline {
                            "text-decoration=\"underline\" "
                        } else {
                            ""
                        };
                        let fill = if s.invert { "#ffffff" } else { "#111111" };

                        svg.push_str(&format!(
                            r##"    <tspan font-size="{font_size}px" font-weight="{font_weight}" fill="{fill}" {decor}>{text}</tspan>
"##,
                            text = escape_xml(&s.text)
                        ));
                    }
                    svg.push_str("  </text>\n");
                }

                cur_y += line_h;
            }
            PreviewElement::Feed(n) => {
                cur_y += *n as f32 * normal_line_h;
            }
            PreviewElement::Cut(mode) => {
                let y = cur_y + 14.0;
                let label = match mode {
                    CutMode::Partial => "✂ PARTIAL CUT",
                    CutMode::Full => "✂ CUT",
                };
                svg.push_str(&format!(
                    r##"  <!-- Paper Cut Marker -->
  <line x1="{pad_x:.1}" y1="{y:.1}" x2="{x2:.1}" y2="{y:.1}" stroke="#999999" stroke-width="1.5" stroke-dasharray="6,4" />
  <text x="{mid_x:.1}" y="{text_y:.1}" text-anchor="middle" font-size="11px" fill="#888888">{label}</text>
"##,
                    x2 = paper_dots - pad_x,
                    mid_x = paper_dots / 2.0,
                    text_y = y - 4.0
                ));
                cur_y += 28.0;
            }
            PreviewElement::QrCode { data, align } => {
                let qr_size = 110.0f32;
                let qr_x = match align {
                    Alignment::Left => pad_x,
                    Alignment::Center => (paper_dots - qr_size) / 2.0,
                    Alignment::Right => paper_dots - pad_x - qr_size,
                };

                let path_d = generate_qr_svg_path(data, qr_size);
                svg.push_str(&format!(
                    r##"  <!-- QR Code -->
  <g transform="translate({qr_x:.1}, {cur_y:.1})">
    <rect width="{qr_size:.1}" height="{qr_size:.1}" fill="#ffffff" />
    <path d="{path_d}" fill="#000000" />
  </g>
"##
                ));
                cur_y += 140.0;
            }
            PreviewElement::Barcode {
                data,
                align,
                height,
            } => {
                let bc_h = (*height as f32).max(48.0);
                let bc_w = 240.0f32;
                let bc_x = match align {
                    Alignment::Left => pad_x,
                    Alignment::Center => (paper_dots - bc_w) / 2.0,
                    Alignment::Right => paper_dots - pad_x - bc_w,
                };

                let path_d = generate_barcode_svg(data, bc_w, bc_h);
                let label_y = cur_y + bc_h + 16.0;
                let label_x = bc_x + bc_w / 2.0;

                svg.push_str(&format!(
                    r##"  <!-- Barcode -->
  <g transform="translate({bc_x:.1}, {cur_y:.1})">
    <path d="{path_d}" fill="#000000" />
  </g>
  <text x="{label_x:.1}" y="{label_y:.1}" text-anchor="middle" font-size="13px" fill="#222222">{esc_data}</text>
"##,
                    esc_data = escape_xml(data)
                ));
                cur_y += bc_h + 26.0;
            }
            PreviewElement::Image {
                width,
                height,
                pixels,
                align,
            } => {
                let img_w = *width as f32;
                let img_h = *height as f32;
                let img_x = match align {
                    Alignment::Left => pad_x,
                    Alignment::Center => (paper_dots - img_w) / 2.0,
                    Alignment::Right => paper_dots - pad_x - img_w,
                };

                // Render 1bpp bitmap as SVG pixel rects
                let mut path = String::new();
                let bytes_per_row = (*width as usize).div_ceil(8);
                for y in 0..*height as usize {
                    for x in 0..*width as usize {
                        let byte_idx = y * bytes_per_row + (x / 8);
                        if byte_idx < pixels.len() {
                            let bit = (pixels[byte_idx] >> (7 - (x % 8))) & 1;
                            if bit == 1 {
                                path.push_str(&format!("M{x},{y}h1v1h-1Z "));
                            }
                        }
                    }
                }

                svg.push_str(&format!(
                    r##"  <!-- Raster Image -->
  <g transform="translate({img_x:.1}, {cur_y:.1})">
    <path d="{path}" fill="#000000" />
  </g>
"##
                ));
                cur_y += img_h + 16.0;
            }
        }
    }

    svg.push_str("</svg>\n");
    svg
}

/// Renders a [`Receipt`] into a responsive, styled HTML component snippet.
///
/// Can be embedded directly into React, Next.js, Vue, or vanilla web pages
/// using standard HTML injection (`dangerouslySetInnerHTML`).
///
/// # Example
///
/// ```rust
/// use papermint::{Receipt, PaperWidth};
///
/// let receipt = Receipt::new(PaperWidth::Mm80)
///     .center()
///     .bold(true)
///     .text_ln("MINT BISTRO")
///     .two_column("Item", "$5.00");
///
/// let html = receipt.render_html();
/// assert!(html.contains("<div class=\"papermint-receipt\""));
/// assert!(html.contains("MINT BISTRO"));
/// ```
#[must_use]
pub fn render_html(receipt: &Receipt) -> String {
    let elements = parse_commands(receipt);
    let max_w = match receipt.paper_width() {
        crate::layout::column::PaperWidth::Mm80 => 400,
        crate::layout::column::PaperWidth::Mm58 => 300,
        crate::layout::column::PaperWidth::Custom(cols) => (cols * 9).clamp(260, 600),
    };

    let mut html = String::with_capacity(8192);
    html.push_str(&format!(
        r##"<div class="papermint-receipt" style="max-width: {max_w}px; margin: 0 auto; background-color: #ffffff; color: #111111; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Courier New', monospace; font-size: 13px; line-height: 1.45; padding: 24px 20px 32px; box-shadow: 0 4px 20px rgba(0,0,0,0.12); border-radius: 4px; box-sizing: border-box; position: relative;">
"##
    ));

    for el in &elements {
        match el {
            PreviewElement::Line { align, spans } => {
                let text_align = match align {
                    Alignment::Left => "left",
                    Alignment::Center => "center",
                    Alignment::Right => "right",
                };

                html.push_str(&format!(
                    r##"  <div style="text-align: {text_align}; min-height: 1.25em; white-space: pre-wrap; word-break: break-word;">"##
                ));

                for s in spans {
                    let mut styles = Vec::new();
                    if s.bold {
                        styles.push("font-weight: bold");
                    }
                    if s.underline {
                        styles.push("text-decoration: underline");
                    }
                    if s.double_height && s.double_width {
                        styles.push("font-size: 1.6em; line-height: 1.2; letter-spacing: 0.05em");
                    } else if s.double_height {
                        styles.push("font-size: 1.5em; line-height: 1.2");
                    } else if s.double_width {
                        styles.push("font-size: 1.2em; letter-spacing: 0.1em");
                    }
                    if s.invert {
                        styles.push("background-color: #111111; color: #ffffff; padding: 1px 4px; border-radius: 2px");
                    }

                    let style_attr = if styles.is_empty() {
                        String::new()
                    } else {
                        format!(r#" style="{}""#, styles.join("; "))
                    };

                    let esc_text = escape_xml(&s.text);
                    if style_attr.is_empty() {
                        html.push_str(&esc_text);
                    } else {
                        html.push_str(&format!(r#"<span{style_attr}>{esc_text}</span>"#));
                    }
                }

                html.push_str("</div>\n");
            }
            PreviewElement::Feed(n) => {
                let h = (*n as usize) * 16;
                html.push_str(&format!(r#"  <div style="height: {h}px;"></div>"#));
                html.push('\n');
            }
            PreviewElement::Cut(mode) => {
                let label = match mode {
                    CutMode::Partial => "✂ Partial Cut",
                    CutMode::Full => "✂ Cut",
                };
                html.push_str(&format!(
                    r##"  <div style="border-top: 1.5px dashed #888888; margin: 16px 0; text-align: center; position: relative;">
    <span style="position: absolute; top: -9px; background: #ffffff; padding: 0 8px; color: #888888; font-size: 10px; text-transform: uppercase;">{label}</span>
  </div>
"##
                ));
            }
            PreviewElement::QrCode { data, align } => {
                let justify = match align {
                    Alignment::Left => "flex-start",
                    Alignment::Center => "center",
                    Alignment::Right => "flex-end",
                };

                let path_d = generate_qr_svg_path(data, 120.0);
                html.push_str(&format!(
                    r##"  <div style="display: flex; justify-content: {justify}; margin: 12px 0;">
    <svg viewBox="0 0 120 120" width="120" height="120" style="background: #ffffff;">
      <path d="{path_d}" fill="#000000" />
    </svg>
  </div>
"##
                ));
            }
            PreviewElement::Barcode {
                data,
                align,
                height,
            } => {
                let justify = match align {
                    Alignment::Left => "flex-start",
                    Alignment::Center => "center",
                    Alignment::Right => "flex-end",
                };
                let bc_h = (*height as f32).max(44.0);
                let path_d = generate_barcode_svg(data, 220.0, bc_h);

                html.push_str(&format!(
                    r##"  <div style="display: flex; flex-direction: column; align-items: {justify}; margin: 12px 0;">
    <svg viewBox="0 0 220 {bc_h:.0}" width="220" height="{bc_h:.0}">
      <path d="{path_d}" fill="#000000" />
    </svg>
    <div style="font-size: 12px; color: #333333; margin-top: 4px; text-align: center;">{esc_data}</div>
  </div>
"##,
                    esc_data = escape_xml(data)
                ));
            }
            PreviewElement::Image {
                width,
                height,
                pixels,
                align,
            } => {
                let justify = match align {
                    Alignment::Left => "flex-start",
                    Alignment::Center => "center",
                    Alignment::Right => "flex-end",
                };

                let mut path = String::new();
                let bytes_per_row = (*width as usize).div_ceil(8);
                for y in 0..*height as usize {
                    for x in 0..*width as usize {
                        let byte_idx = y * bytes_per_row + (x / 8);
                        if byte_idx < pixels.len() {
                            let bit = (pixels[byte_idx] >> (7 - (x % 8))) & 1;
                            if bit == 1 {
                                path.push_str(&format!("M{x},{y}h1v1h-1Z "));
                            }
                        }
                    }
                }

                html.push_str(&format!(
                    r##"  <div style="display: flex; justify-content: {justify}; margin: 10px 0;">
    <svg viewBox="0 0 {width} {height}" width="{width}" height="{height}">
      <path d="{path}" fill="#000000" />
    </svg>
  </div>
"##
                ));
            }
        }
    }

    html.push_str("</div>\n");
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::column::PaperWidth;

    #[test]
    fn test_render_svg_basic() {
        let receipt = Receipt::new(PaperWidth::Mm80)
            .center()
            .bold(true)
            .text_ln("MINT BISTRO")
            .bold(false)
            .text_ln("123 Main Street")
            .divider('-')
            .two_column("Burger", "$12.00")
            .cut_full();

        let svg = receipt.render_svg();
        assert!(svg.starts_with("<svg"), "Must start with <svg");
        assert!(svg.ends_with("</svg>\n"), "Must end with </svg>");
        assert!(svg.contains("MINT BISTRO"), "Must contain header text");
        assert!(svg.contains("Burger"), "Must contain row item");
        assert!(svg.contains("CUT"), "Must contain cut marker");
    }

    #[test]
    fn test_render_html_basic() {
        let receipt = Receipt::new(PaperWidth::Mm58)
            .center()
            .text_ln("COMPACT 58MM")
            .divider('=')
            .two_column("Total", "$42.00");

        let html = receipt.render_html();
        assert!(
            html.contains("papermint-receipt"),
            "Must contain receipt container class"
        );
        assert!(html.contains("COMPACT 58MM"));
        assert!(html.contains("$42.00"));
    }
}
