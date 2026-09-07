//! Column layout and paper width calculation utilities.

use crate::command::Alignment;

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
    let left_count = left.chars().count();
    let right_count = right.chars().count();

    if left_count + right_count < total_width {
        let spaces = total_width - (left_count + right_count);
        format!("{left}{}{right}", " ".repeat(spaces))
    } else if right_count >= total_width {
        // Right side is too large to fit anything else
        right.chars().take(total_width).collect()
    } else {
        // Truncate left with an ellipsis
        let max_left = total_width.saturating_sub(right_count + 1);
        let left_truncated: String = if max_left > 1 {
            let take_len = max_left - 1;
            let mut s: String = left.chars().take(take_len).collect();
            s.push('…');
            s
        } else {
            left.chars().take(max_left).collect()
        };
        let spaces = total_width.saturating_sub(left_truncated.chars().count() + right_count);
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
    let left_count = left.chars().count();
    let center_count = center.chars().count();
    let right_count = right.chars().count();

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

/// Aligns a text string within a given column width.
#[must_use]
pub fn align_text(text: &str, width: usize, alignment: Alignment) -> String {
    let count = text.chars().count();
    if count >= width {
        return text.chars().take(width).collect();
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
}
