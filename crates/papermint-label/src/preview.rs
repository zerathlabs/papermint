//! Virtual SVG Sticker Preview Renderer.
//!
//! Generates vector SVG graphics of physical adhesive labels, complete with
//! die-cut rounded corners, typography, barcode stripes, and QR simulation.

use crate::command::LabelCommand;
use crate::label::Label;

/// Renders a [`Label`] canvas into an SVG string.
pub fn render_svg(label: &Label) -> String {
    let width = label.width_dots();
    let height = label.height_dots();

    let mut svg = String::with_capacity(4096);

    // SVG Header
    svg.push_str(&format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}" style="background:#f1f5f9;padding:12px;border-radius:12px;">"##,
        width, height, width, height
    ));

    // SVG Filters (Drop Shadow)
    svg.push_str(
        r##"<defs><filter id="sticker-shadow" x="-5%" y="-5%" width="110%" height="110%"><feDropShadow dx="0" dy="4" stdDeviation="6" flood-opacity="0.15"/></filter></defs>"##,
    );

    // Die-cut sticker base with rounded corners
    svg.push_str(&format!(
        r##"<rect x="0" y="0" width="{}" height="{}" rx="16" ry="16" fill="#ffffff" stroke="#cbd5e1" stroke-width="1.5" filter="url(#sticker-shadow)"/>"##,
        width, height
    ));

    // Canvas elements
    for cmd in &label.commands {
        match cmd {
            LabelCommand::Text {
                x,
                y,
                font: _,
                rotation,
                x_multi: _,
                y_multi,
                content,
            } => {
                let font_size = 20 * (*y_multi as u32).max(1);
                let transform = if rotation.degrees() > 0 {
                    format!(
                        r##" transform="rotate({}, {}, {})""##,
                        rotation.degrees(),
                        x,
                        y
                    )
                } else {
                    String::new()
                };

                // Approximate baseline offset (y + font_size)
                let baseline_y = y + font_size;

                let escaped = html_escape(content);
                svg.push_str(&format!(
                    r##"<text x="{}" y="{}" font-family="'JetBrains Mono', 'SF Pro Text', Menlo, monospace" font-size="{}" font-weight="600" fill="#0f172a"{}>{}</text>"##,
                    x, baseline_y, font_size, transform, escaped
                ));
            }
            LabelCommand::Barcode {
                x,
                y,
                barcode_type: _,
                height: b_height,
                human_readable,
                rotation,
                narrow: _,
                wide: _,
                content,
            } => {
                let transform = if rotation.degrees() > 0 {
                    format!(
                        r##" transform="rotate({}, {}, {})""##,
                        rotation.degrees(),
                        x,
                        y
                    )
                } else {
                    String::new()
                };

                svg.push_str(&format!(r##"<g{}>"##, transform));

                // Render barcode stripes based on string hash pattern
                let mut bar_x = *x;
                let pattern = generate_barcode_pattern(content);
                for is_black in pattern {
                    let w = 2;
                    if is_black {
                        svg.push_str(&format!(
                            r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#0f172a"/>"##,
                            bar_x, y, w, b_height
                        ));
                    }
                    bar_x += w;
                }

                if *human_readable {
                    let text_y = y + b_height + 14;
                    svg.push_str(&format!(
                        r##"<text x="{}" y="{}" font-family="'JetBrains Mono', monospace" font-size="12" font-weight="500" fill="#0f172a" text-anchor="middle">{}</text>"##,
                        x + (bar_x - x) / 2,
                        text_y,
                        html_escape(content)
                    ));
                }

                svg.push_str("</g>");
            }
            LabelCommand::QrCode {
                x,
                y,
                cell_size,
                ecc: _,
                rotation,
                content: _,
            } => {
                let transform = if rotation.degrees() > 0 {
                    format!(
                        r##" transform="rotate({}, {}, {})""##,
                        rotation.degrees(),
                        x,
                        y
                    )
                } else {
                    String::new()
                };

                let qr_dim = (*cell_size as u32) * 25; // Standard ~25x25 matrix
                svg.push_str(&format!(r##"<g{}>"##, transform));

                // Draw QR finder corners
                draw_qr_finder(&mut svg, *x, *y, *cell_size as u32);
                draw_qr_finder(
                    &mut svg,
                    x + qr_dim - 7 * (*cell_size as u32),
                    *y,
                    *cell_size as u32,
                );
                draw_qr_finder(
                    &mut svg,
                    *x,
                    y + qr_dim - 7 * (*cell_size as u32),
                    *cell_size as u32,
                );

                // Draw decorative QR data grid dots
                for r in 8..17 {
                    for c in 8..17 {
                        if (r + c) % 2 == 0 || (r * c) % 3 == 0 {
                            svg.push_str(&format!(
                                r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#0f172a"/>"##,
                                x + (c * *cell_size as u32),
                                y + (r * *cell_size as u32),
                                *cell_size as u32,
                                *cell_size as u32
                            ));
                        }
                    }
                }

                svg.push_str("</g>");
            }
            LabelCommand::Box {
                x,
                y,
                width: b_width,
                height: b_height,
                thickness,
            } => {
                svg.push_str(&format!(
                    r##"<rect x="{}" y="{}" width="{}" height="{}" fill="none" stroke="#0f172a" stroke-width="{}"/>"##,
                    x, y, b_width, b_height, thickness
                ));
            }
            LabelCommand::Line {
                x,
                y,
                width: l_width,
                height: l_height,
            } => {
                svg.push_str(&format!(
                    r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#0f172a"/>"##,
                    x, y, l_width, l_height
                ));
            }
            LabelCommand::Reverse {
                x,
                y,
                width: r_width,
                height: r_height,
            } => {
                svg.push_str(&format!(
                    r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#0f172a" style="mix-blend-mode:difference;"/>"##,
                    x, y, r_width, r_height
                ));
            }
            LabelCommand::Raw(_) => {}
        }
    }

    svg.push_str("</svg>");
    svg
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn draw_qr_finder(svg: &mut String, x: u32, y: u32, cs: u32) {
    let outer = cs * 7;
    let inner = cs * 3;
    let inner_offset = cs * 2;

    svg.push_str(&format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#0f172a"/>"##,
        x, y, outer, outer
    ));
    svg.push_str(&format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#ffffff"/>"##,
        x + cs,
        y + cs,
        outer - (cs * 2),
        outer - (cs * 2)
    ));
    svg.push_str(&format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#0f172a"/>"##,
        x + inner_offset,
        y + inner_offset,
        inner,
        inner
    ));
}

fn generate_barcode_pattern(content: &str) -> Vec<bool> {
    let mut pattern = vec![true, false, true]; // Start guard
    for b in content.bytes() {
        for bit in 0..6 {
            pattern.push((b >> bit) & 1 == 1);
        }
        pattern.push(false);
    }
    pattern.extend_from_slice(&[true, false, true]); // End guard
    pattern
}
