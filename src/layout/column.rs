//! Column layout and paper width calculation utilities.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::command::Alignment;

/// Returns the printable monospace display width of a string in accordance with
/// Unicode Standard Annex #11 (East Asian Width).
///
/// Fullwidth CJK ideographs, Hangul, Kana, and fullwidth symbols return width 2,
/// standard ASCII/Latin characters return width 1, and zero-width/combining characters return width 0.
#[must_use]
pub fn str_display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Truncates a string slice so its monospace display width does not exceed `max_width`.
#[must_use]
pub fn truncate_to_display_width(s: &str, max_width: usize) -> &str {
    let mut current_width = 0;
    for (byte_idx, ch) in s.char_indices() {
        let ch_w = UnicodeWidthChar::width(ch).unwrap_or(0);
        if current_width + ch_w > max_width {
            return &s[..byte_idx];
        }
        current_width += ch_w;
    }
    s
}

/// Thermal printer paper width and character capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaperWidth {
    /// Standard 80mm receipt paper (48 columns in Font A).
    #[default]
    Mm80,
    /// Compact 58mm receipt paper (32 columns in Font A).
    Mm58,
    /// Custom paper character width.
    Custom(usize),
}

impl PaperWidth {
    /// Returns the number of printable character columns for this paper width.
    #[must_use]
    pub const fn columns(&self) -> usize {
        match self {
            Self::Mm80 => 48,
            Self::Mm58 => 32,
            Self::Custom(cols) => *cols,
        }
    }

    /// Returns the standard printable dot width (at 203 DPI).
    ///
    /// - 80mm paper: 576 printable dots.
    /// - 58mm paper: 384 printable dots.
    /// - Custom: `columns * 12` dots (based on Font A 12-dot character cell width).
    #[must_use]
    pub const fn dots(&self) -> u32 {
        match self {
            Self::Mm80 => 576,
            Self::Mm58 => 384,
            Self::Custom(cols) => (*cols as u32) * 12,
        }
    }
}

/// Formats a two-column row (e.g. Item Name on left, Price on right).
///
/// If the text exceeds `total_width`, the right text is preserved and the left
/// text is truncated with an ellipsis or wrapped.
#[must_use]
pub fn format_two_column(left: &str, right: &str, total_width: usize) -> String {
    let left_count = str_display_width(left);
    let right_count = str_display_width(right);

    if left_count + right_count < total_width {
        let spaces = total_width - (left_count + right_count);
        format!("{left}{}{right}", " ".repeat(spaces))
    } else if right_count >= total_width {
        // Right side is too large to fit anything else
        truncate_to_display_width(right, total_width).to_string()
    } else {
        // Truncate left with an ellipsis
        let max_left = total_width.saturating_sub(right_count + 1);
        let left_truncated: String = if max_left > 1 {
            let take_w = max_left - 1;
            let mut s = truncate_to_display_width(left, take_w).to_string();
            s.push('…');
            s
        } else {
            truncate_to_display_width(left, max_left).to_string()
        };
        let spaces = total_width.saturating_sub(str_display_width(&left_truncated) + right_count);
        format!("{left_truncated}{}{right}", " ".repeat(spaces))
    }
}

/// Formats a three-column row (e.g. Qty on left, Item in center, Price on right).
#[must_use]
pub fn format_three_column(
    left: &str,
    center: &str,
    right: &str,
    total_width: usize,
) -> String {
    let left_count = str_display_width(left);
    let center_count = str_display_width(center);
    let right_count = str_display_width(right);

    let text_len = left_count + center_count + right_count;
    if text_len + 2 <= total_width {
        let remaining = total_width - text_len;
        let left_spaces = remaining / 2;
        let right_spaces = remaining - left_spaces;
        format!(
            "{left}{}{center}{}{right}",
            " ".repeat(left_spaces),
            " ".repeat(right_spaces)
        )
    } else {
        // Fallback: 2-column left + right
        format_two_column(left, right, total_width)
    }
}

/// Specification of a column's width within a tabular receipt layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnWidth {
    /// Fixed character cell width.
    Fixed(usize),
    /// Proportional fraction of total printable paper columns (0.0 ..= 1.0).
    Fraction(f32),
}

/// Definition of a single column within an N-column receipt table.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TableColumn {
    /// Target width specification.
    pub width: ColumnWidth,
    /// Text alignment inside this column cell.
    pub alignment: Alignment,
}

impl TableColumn {
    /// Creates a new [`TableColumn`].
    #[must_use]
    pub const fn new(width: ColumnWidth, alignment: Alignment) -> Self {
        Self { width, alignment }
    }

    /// Convenience constructor for a fixed-width column.
    #[must_use]
    pub const fn fixed(width: usize, alignment: Alignment) -> Self {
        Self::new(ColumnWidth::Fixed(width), alignment)
    }

    /// Convenience constructor for a proportional fractional column.
    #[must_use]
    pub const fn fraction(fraction: f32, alignment: Alignment) -> Self {
        Self::new(ColumnWidth::Fraction(fraction), alignment)
    }

    /// Convenience constructor for a left-aligned column.
    #[must_use]
    pub const fn left(width: ColumnWidth) -> Self {
        Self::new(width, Alignment::Left)
    }

    /// Convenience constructor for a right-aligned column.
    #[must_use]
    pub const fn right(width: ColumnWidth) -> Self {
        Self::new(width, Alignment::Right)
    }

    /// Convenience constructor for a center-aligned column.
    #[must_use]
    pub const fn center(width: ColumnWidth) -> Self {
        Self::new(width, Alignment::Center)
    }
}

/// Resolves a slice of [`TableColumn`] specifications into concrete integer character widths.
///
/// If mixed fixed and fractional columns are provided, fixed widths are allocated first,
/// and fractional columns divide the remaining available paper width.
/// Ensures the sum of all column widths matches or stays strictly within `total_width`.
#[must_use]
pub fn resolve_column_widths(columns: &[TableColumn], total_width: usize) -> Vec<usize> {
    if columns.is_empty() || total_width == 0 {
        return Vec::new();
    }

    let mut widths = vec![0; columns.len()];
    let mut fixed_sum = 0;
    let mut total_frac = 0.0f32;

    for (i, col) in columns.iter().enumerate() {
        match col.width {
            ColumnWidth::Fixed(w) => {
                widths[i] = w;
                fixed_sum += w;
            }
            ColumnWidth::Fraction(f) => {
                total_frac += f.max(0.0);
            }
        }
    }

    // If fixed columns already exceed or equal total_width
    if fixed_sum >= total_width {
        let scale = (total_width as f32) / (fixed_sum as f32);
        let mut new_sum = 0;
        for (i, col) in columns.iter().enumerate() {
            if matches!(col.width, ColumnWidth::Fixed(_)) {
                widths[i] = ((widths[i] as f32) * scale).floor() as usize;
                new_sum += widths[i];
            } else {
                widths[i] = 0;
            }
        }
        if new_sum < total_width {
            let remainder = total_width - new_sum;
            if let Some(max_idx) = widths.iter().enumerate().max_by_key(|&(_, w)| w).map(|(i, _)| i) {
                widths[max_idx] += remainder;
            }
        }
        return widths;
    }

    // Allocate remaining space to fractional columns
    let remaining = total_width - fixed_sum;
    let mut allocated_frac = 0;

    if total_frac > 0.0 {
        for (i, col) in columns.iter().enumerate() {
            if let ColumnWidth::Fraction(f) = col.width {
                let norm_f = f / total_frac;
                let w = ((remaining as f32) * norm_f).round() as usize;
                widths[i] = w;
                allocated_frac += w;
            }
        }
    }

    let total_allocated = fixed_sum + allocated_frac;
    if total_allocated < total_width {
        let remainder = total_width - total_allocated;
        // Distribute remainder to the widest fractional column (or widest column overall)
        if let Some(target_idx) = columns
            .iter()
            .enumerate()
            .filter(|(_, c)| matches!(c.width, ColumnWidth::Fraction(_)))
            .max_by_key(|(i, _)| widths[*i])
            .map(|(i, _)| i)
            .or_else(|| widths.iter().enumerate().max_by_key(|&(_, w)| w).map(|(i, _)| i))
        {
            widths[target_idx] += remainder;
        }
    } else if total_allocated > total_width {
        let excess = total_allocated - total_width;
        if let Some(target_idx) = widths.iter().enumerate().max_by_key(|&(_, w)| w).map(|(i, _)| i) {
            widths[target_idx] = widths[target_idx].saturating_sub(excess);
        }
    }

    widths
}

/// Wraps text into multiple lines respecting word boundaries, display width, and explicit line breaks (`\n`).
///
/// If a single unbroken word exceeds `max_width`, it will be broken cleanly across lines.
#[must_use]
pub fn wrap_text_to_width(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![String::new()];
    }

    let trimmed = text.trim();
    if trimmed.is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();

    // Preserve explicit newline breaks (\n) within cell text
    for raw_line in text.split('\n') {
        let line_trimmed = raw_line.trim();
        if line_trimmed.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current_line = String::new();
        let mut current_len = 0;

        for word in line_trimmed.split_whitespace() {
            let word_len = str_display_width(word);

            // If a single word exceeds max_width, break it into display-width chunks
            if word_len > max_width {
                if !current_line.is_empty() {
                    lines.push(std::mem::take(&mut current_line));
                    current_len = 0;
                }

                let mut remaining_word = word;
                while !remaining_word.is_empty() {
                    let chunk = truncate_to_display_width(remaining_word, max_width);
                    let chunk_len = str_display_width(chunk);
                    if chunk_len == 0 {
                        break;
                    }
                    if chunk_len == max_width {
                        lines.push(chunk.to_string());
                    } else {
                        current_line = chunk.to_string();
                        current_len = chunk_len;
                    }
                    remaining_word = &remaining_word[chunk.len()..];
                }
                continue;
            }

            if current_line.is_empty() {
                current_line.push_str(word);
                current_len = word_len;
            } else if current_len + 1 + word_len <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
                current_len += 1 + word_len;
            } else {
                lines.push(std::mem::take(&mut current_line));
                current_line.push_str(word);
                current_len = word_len;
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// Formats an N-column row with word-wrapping and synchronized vertical row height.
///
/// Returns a sequence of physical formatted lines to be emitted to the printer.
#[must_use]
pub fn format_table_row(cells: &[&str], columns: &[TableColumn], total_width: usize) -> Vec<String> {
    let widths = resolve_column_widths(columns, total_width);
    if widths.is_empty() {
        return Vec::new();
    }

    let mut col_lines: Vec<Vec<String>> = Vec::with_capacity(columns.len());
    let mut max_height = 1;

    for (i, &col_width) in widths.iter().enumerate() {
        let cell_text = cells.get(i).copied().unwrap_or("");
        let wrapped = wrap_text_to_width(cell_text, col_width);
        max_height = max_height.max(wrapped.len());
        col_lines.push(wrapped);
    }

    let mut result_lines = Vec::with_capacity(max_height);
    for h in 0..max_height {
        let mut line_str = String::with_capacity(total_width);
        for (i, col) in columns.iter().enumerate() {
            let width = widths[i];
            let text = col_lines[i].get(h).map(String::as_str).unwrap_or("");
            let padded = align_text(text, width, col.alignment);
            line_str.push_str(&padded);
        }
        result_lines.push(line_str);
    }

    result_lines
}

/// Aligns a text string within a given column width using monospace display width.
#[must_use]
pub fn align_text(text: &str, width: usize, alignment: Alignment) -> String {
    let count = str_display_width(text);
    if count >= width {
        return truncate_to_display_width(text, width).to_string();
    }

    let diff = width - count;
    match alignment {
        Alignment::Left => format!("{text}{}", " ".repeat(diff)),
        Alignment::Right => format!("{}{text}", " ".repeat(diff)),
        Alignment::Center => {
            let left_pad = diff / 2;
            let right_pad = diff - left_pad;
            format!("{}{text}{}", " ".repeat(left_pad), " ".repeat(right_pad))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_two_column_standard() {
        let row = format_two_column("Coffee", "$3.50", 20);
        assert_eq!(row.chars().count(), 20);
        assert_eq!(row, "Coffee         $3.50");
    }

    #[test]
    fn test_format_two_column_overflow() {
        let row = format_two_column("Very Long Artisan Special Roast Coffee", "$3.50", 20);
        assert_eq!(row.chars().count(), 20);
        assert!(row.ends_with("$3.50"));
    }

    #[test]
    fn test_align_text() {
        assert_eq!(align_text("HI", 6, Alignment::Center), "  HI  ");
        assert_eq!(align_text("HI", 6, Alignment::Left), "HI    ");
        assert_eq!(align_text("HI", 6, Alignment::Right), "    HI");
    }

    #[test]
    fn test_wrap_text_to_width() {
        let text = "Double Truffle Wagyu Smash Burger";
        let lines = wrap_text_to_width(text, 15);
        assert_eq!(lines, vec!["Double Truffle", "Wagyu Smash", "Burger"]);

        // Single long unbroken word
        let long_word = "Supercalifragilistic";
        let lines = wrap_text_to_width(long_word, 8);
        assert_eq!(lines, vec!["Supercal", "ifragili", "stic"]);
    }

    #[test]
    fn test_resolve_column_widths() {
        let cols = [
            TableColumn::fixed(4, Alignment::Left),
            TableColumn::fraction(0.5, Alignment::Left),
            TableColumn::fraction(0.25, Alignment::Right),
            TableColumn::fraction(0.25, Alignment::Right),
        ];
        let widths = resolve_column_widths(&cols, 48);
        assert_eq!(widths.len(), 4);
        assert_eq!(widths[0], 4);
        let sum: usize = widths.iter().sum();
        assert_eq!(sum, 48);
    }

    #[test]
    fn test_format_table_row_multiline_sync() {
        let cols = [
            TableColumn::fixed(4, Alignment::Left),
            TableColumn::fixed(16, Alignment::Left),
            TableColumn::fixed(8, Alignment::Right),
        ];
        let cells = ["2x", "Double Truffle Wagyu Burger", "$24.00"];
        let lines = format_table_row(&cells, &cols, 28);

        // Column 1 is 16 chars: "Double Truffle" (14), "Wagyu Burger" (12) -> 2 lines
        assert_eq!(lines.len(), 2);
        // Line 0: "2x  " (4) + "Double Truffle  " (16) + "  $24.00" (8) = 28 chars
        assert_eq!(lines[0], "2x  Double Truffle    $24.00");
        // Line 1: "    " (4) + "Wagyu Burger    " (16) + "        " (8) = 28 chars
        assert_eq!(lines[1], "    Wagyu Burger            ");

        for line in &lines {
            assert_eq!(str_display_width(line), 28);
        }
    }

    #[test]
    fn test_resolve_proportional_fractions_sum_under_one() {
        // Two columns with 0.3 fraction each should share remaining space 50/50 (24 and 24 on 48 cols)
        let cols = [
            TableColumn::fraction(0.3, Alignment::Left),
            TableColumn::fraction(0.3, Alignment::Right),
        ];
        let widths = resolve_column_widths(&cols, 48);
        assert_eq!(widths, vec![24, 24]);
    }

    #[test]
    fn test_wrap_text_with_explicit_newlines() {
        let text = "Double Burger\n- Rare\n- Extra Pickles";
        let lines = wrap_text_to_width(text, 20);
        assert_eq!(lines, vec!["Double Burger", "- Rare", "- Extra Pickles"]);
    }

    #[test]
    fn test_cjk_fullwidth_character_display_width() {
        // Japanese: "ラーメン" has 4 chars, but 8 display width units
        assert_eq!(str_display_width("ラーメン"), 8);

        // Alignment with Japanese fullwidth characters
        let aligned = align_text("牛丼", 8, Alignment::Left);
        assert_eq!(str_display_width(&aligned), 8);
        assert_eq!(aligned, "牛丼    ");

        // Two column with CJK
        let row = format_two_column("1x 抹茶ラテ", "¥450", 20);
        assert_eq!(str_display_width(&row), 20);
    }

    #[test]
    fn test_arabic_text_display_width() {
        // Standard Arabic letters (each is 1 monospace width column)
        // "قهوة" (Coffee): 4 base letters -> display width 4
        assert_eq!(str_display_width("قهوة"), 4);

        // Arabic text with diacritics / tashkeel: "شُكْرًا" (Thank you)
        // Base letters: ش, ك, ر, ا (4 letters)
        // Tashkeel combining marks: ُ (dammah), ْ (sukun), ً (tanwin fath)
        // unicode-width correctly treats non-spacing combining marks as 0 width (total width 4),
        // whereas raw .chars().count() would incorrectly count 7!
        assert_eq!("شُكْرًا".chars().count(), 7);
        assert_eq!(str_display_width("شُكْرًا"), 4);

        // Two-column receipt row with Arabic item and price
        let row = format_two_column("قهوة عربية", "1.500 KWD", 24);
        assert_eq!(str_display_width(&row), 24);

        // Word wrapping Arabic text
        let arabic_menu = "شاورما دجاج مع ثوم ومخلل وبطاطا";
        let wrapped = wrap_text_to_width(arabic_menu, 16);
        for line in &wrapped {
            assert!(str_display_width(line) <= 16);
        }
    }
}


