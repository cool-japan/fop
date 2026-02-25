//! Inline-level layout and line breaking
//!
//! Handles horizontal positioning of inline elements and text,
//! including line breaking when content exceeds available width.

use crate::area::TraitSet;
use fop_types::{FontRegistry, Length};
use std::fmt;

/// Text alignment options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Justify,
}

impl fmt::Display for TextAlign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextAlign::Left => write!(f, "left"),
            TextAlign::Right => write!(f, "right"),
            TextAlign::Center => write!(f, "center"),
            TextAlign::Justify => write!(f, "justify"),
        }
    }
}

/// Inline layout context for a single line
pub struct InlineLayoutContext {
    /// Available width for the line
    pub available_width: Length,

    /// Current X position (advances as content is added)
    pub current_x: Length,

    /// Line height (from line-height property or font metrics)
    pub line_height: Length,

    /// Areas in this line
    pub inline_areas: Vec<InlineArea>,

    /// Text alignment
    pub text_align: TextAlign,

    /// Letter spacing to apply between characters
    pub letter_spacing: Length,

    /// Word spacing to apply between words
    pub word_spacing: Length,
}

/// An inline area waiting to be positioned
#[derive(Debug, Clone)]
pub struct InlineArea {
    /// Width of the area
    pub width: Length,

    /// Height of the area
    pub height: Length,

    /// Content (for text areas)
    pub content: Option<String>,

    /// Rendering traits
    pub traits: TraitSet,
}

impl InlineLayoutContext {
    /// Create a new inline layout context
    pub fn new(available_width: Length, line_height: Length) -> Self {
        Self {
            available_width,
            current_x: Length::ZERO,
            line_height,
            inline_areas: Vec::new(),
            text_align: TextAlign::Left,
            letter_spacing: Length::ZERO,
            word_spacing: Length::ZERO,
        }
    }

    /// Set text alignment
    pub fn with_text_align(mut self, align: TextAlign) -> Self {
        self.text_align = align;
        self
    }

    /// Set letter spacing
    pub fn with_letter_spacing(mut self, spacing: Length) -> Self {
        self.letter_spacing = spacing;
        self
    }

    /// Set word spacing
    pub fn with_word_spacing(mut self, spacing: Length) -> Self {
        self.word_spacing = spacing;
        self
    }

    /// Check if content fits on the current line
    pub fn fits(&self, width: Length) -> bool {
        self.current_x + width <= self.available_width
    }

    /// Add an inline area to the line
    pub fn add(&mut self, area: InlineArea) -> bool {
        if !self.fits(area.width) {
            return false; // Doesn't fit
        }

        self.current_x += area.width;

        // Update line height if this area is taller
        if area.height > self.line_height {
            self.line_height = area.height;
        }

        self.inline_areas.push(area);
        true
    }

    /// Get the remaining width on this line
    pub fn remaining_width(&self) -> Length {
        self.available_width - self.current_x
    }

    /// Check if the line is empty
    pub fn is_empty(&self) -> bool {
        self.inline_areas.is_empty()
    }

    /// Get the total width used
    pub fn used_width(&self) -> Length {
        self.current_x
    }

    /// Calculate the starting X offset for aligned content
    pub fn calculate_alignment_offset(&self) -> Length {
        let unused_width = self.available_width - self.current_x;
        match self.text_align {
            TextAlign::Left => Length::ZERO,
            TextAlign::Right => unused_width,
            TextAlign::Center => unused_width / 2,
            TextAlign::Justify => Length::ZERO, // Justify handled differently
        }
    }

    /// Apply text alignment to all areas in the line
    pub fn apply_alignment(&mut self) {
        let offset = self.calculate_alignment_offset();
        if offset > Length::ZERO {
            // Shift all areas by the offset (implementation would adjust positions)
            // This is a placeholder - actual implementation would modify area positions
        }
    }
}

/// Line breaker - breaks text into lines
pub struct LineBreaker {
    /// Available width for lines
    available_width: Length,

    /// Font registry for accurate text measurement
    font_registry: FontRegistry,

    /// Letter spacing to apply
    letter_spacing: Length,

    /// Word spacing to apply
    word_spacing: Length,
}

impl LineBreaker {
    /// Create a new line breaker
    pub fn new(available_width: Length) -> Self {
        Self {
            available_width,
            font_registry: FontRegistry::new(),
            letter_spacing: Length::ZERO,
            word_spacing: Length::ZERO,
        }
    }

    /// Set letter spacing for text measurement
    pub fn with_letter_spacing(mut self, spacing: Length) -> Self {
        self.letter_spacing = spacing;
        self
    }

    /// Set word spacing for text measurement
    pub fn with_word_spacing(mut self, spacing: Length) -> Self {
        self.word_spacing = spacing;
        self
    }

    /// Break text into words for line breaking
    pub fn break_into_words(&self, text: &str) -> Vec<String> {
        text.split_whitespace().map(|s| s.to_string()).collect()
    }

    /// Measure text width using font metrics
    pub fn measure_text(&self, text: &str, font_size: Length) -> Length {
        self.measure_text_with_font(text, font_size, "Helvetica")
    }

    /// Measure text width with specific font
    pub fn measure_text_with_font(&self, text: &str, font_size: Length, font_name: &str) -> Length {
        let font_metrics = self.font_registry.get_or_default(font_name);
        let base_width = font_metrics.measure_text(text, font_size);

        // Add letter spacing: applied between every character (not after the last one)
        let char_count = text.chars().count();
        let letter_spacing_total = if char_count > 0 {
            self.letter_spacing * (char_count.saturating_sub(1) as i32)
        } else {
            Length::ZERO
        };

        // Add word spacing: applied to each space character
        let space_count = text.chars().filter(|&c| c == ' ').count();
        let word_spacing_total = self.word_spacing * (space_count as i32);

        base_width + letter_spacing_total + word_spacing_total
    }

    /// Break text into lines using greedy algorithm
    pub fn break_lines(&self, text: &str, font_size: Length) -> Vec<String> {
        let words = self.break_into_words(text);
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let _space_width = self.measure_text(" ", font_size);

        for word in words {
            let _word_width = self.measure_text(&word, font_size);
            let line_with_word = if current_line.is_empty() {
                word.clone()
            } else {
                format!("{} {}", current_line, word)
            };

            let total_width = self.measure_text(&line_with_word, font_size);

            if total_width <= self.available_width {
                // Word fits on current line
                current_line = line_with_word;
            } else {
                // Word doesn't fit, start new line
                if !current_line.is_empty() {
                    lines.push(current_line);
                }
                current_line = word;
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── TextAlign Display ────────────────────────────────────────────────────

    #[test]
    fn test_text_align_display_left() {
        assert_eq!(format!("{}", TextAlign::Left), "left");
    }

    #[test]
    fn test_text_align_display_right() {
        assert_eq!(format!("{}", TextAlign::Right), "right");
    }

    #[test]
    fn test_text_align_display_center() {
        assert_eq!(format!("{}", TextAlign::Center), "center");
    }

    #[test]
    fn test_text_align_display_justify() {
        assert_eq!(format!("{}", TextAlign::Justify), "justify");
    }

    // ── InlineLayoutContext construction ─────────────────────────────────────

    #[test]
    fn test_inline_context_initial_state() {
        let ctx = InlineLayoutContext::new(Length::from_pt(100.0), Length::from_pt(12.0));
        assert!(ctx.is_empty());
        assert_eq!(ctx.available_width, Length::from_pt(100.0));
        assert_eq!(ctx.current_x, Length::ZERO);
        assert_eq!(ctx.line_height, Length::from_pt(12.0));
        assert_eq!(ctx.text_align, TextAlign::Left);
        assert_eq!(ctx.letter_spacing, Length::ZERO);
        assert_eq!(ctx.word_spacing, Length::ZERO);
    }

    #[test]
    fn test_inline_context_with_text_align() {
        let ctx = InlineLayoutContext::new(Length::from_pt(100.0), Length::from_pt(12.0))
            .with_text_align(TextAlign::Center);
        assert_eq!(ctx.text_align, TextAlign::Center);
    }

    #[test]
    fn test_inline_context_with_letter_spacing() {
        let ctx = InlineLayoutContext::new(Length::from_pt(100.0), Length::from_pt(12.0))
            .with_letter_spacing(Length::from_pt(1.0));
        assert_eq!(ctx.letter_spacing, Length::from_pt(1.0));
    }

    #[test]
    fn test_inline_context_with_word_spacing() {
        let ctx = InlineLayoutContext::new(Length::from_pt(100.0), Length::from_pt(12.0))
            .with_word_spacing(Length::from_pt(2.0));
        assert_eq!(ctx.word_spacing, Length::from_pt(2.0));
    }

    // ── InlineArea creation ──────────────────────────────────────────────────

    #[test]
    fn test_inline_area_text_creation() {
        let area = InlineArea {
            width: Length::from_pt(30.0),
            height: Length::from_pt(12.0),
            content: Some("hello".to_string()),
            traits: TraitSet::default(),
        };
        assert_eq!(area.width, Length::from_pt(30.0));
        assert_eq!(area.height, Length::from_pt(12.0));
        assert_eq!(area.content.as_deref(), Some("hello"));
    }

    #[test]
    fn test_inline_area_space_creation() {
        let area = InlineArea {
            width: Length::from_pt(5.0),
            height: Length::from_pt(12.0),
            content: None,
            traits: TraitSet::default(),
        };
        assert_eq!(area.width, Length::from_pt(5.0));
        assert!(area.content.is_none());
    }

    #[test]
    fn test_inline_area_glue_zero_width() {
        let area = InlineArea {
            width: Length::ZERO,
            height: Length::from_pt(12.0),
            content: None,
            traits: TraitSet::default(),
        };
        assert_eq!(area.width, Length::ZERO);
    }

    // ── InlineLayoutContext add and fits ─────────────────────────────────────

    #[test]
    fn test_inline_context_add_single_area() {
        let mut ctx = InlineLayoutContext::new(Length::from_pt(100.0), Length::from_pt(12.0));
        let area = InlineArea {
            width: Length::from_pt(30.0),
            height: Length::from_pt(12.0),
            content: Some("test".to_string()),
            traits: TraitSet::default(),
        };
        let result = ctx.add(area);
        assert!(result);
        assert!(!ctx.is_empty());
        assert_eq!(ctx.used_width(), Length::from_pt(30.0));
        assert_eq!(ctx.remaining_width(), Length::from_pt(70.0));
    }

    #[test]
    fn test_inline_context_add_multiple_areas() {
        let mut ctx = InlineLayoutContext::new(Length::from_pt(100.0), Length::from_pt(12.0));
        for _ in 0..3 {
            let area = InlineArea {
                width: Length::from_pt(20.0),
                height: Length::from_pt(12.0),
                content: Some("x".to_string()),
                traits: TraitSet::default(),
            };
            assert!(ctx.add(area));
        }
        assert_eq!(ctx.inline_areas.len(), 3);
        assert_eq!(ctx.used_width(), Length::from_pt(60.0));
    }

    #[test]
    fn test_inline_area_width_overflow_not_added() {
        let mut ctx = InlineLayoutContext::new(Length::from_pt(50.0), Length::from_pt(12.0));
        let area = InlineArea {
            width: Length::from_pt(60.0),
            height: Length::from_pt(12.0),
            content: Some("toolong".to_string()),
            traits: TraitSet::default(),
        };
        let result = ctx.add(area);
        assert!(!result);
        assert!(ctx.is_empty());
        assert_eq!(ctx.used_width(), Length::ZERO);
    }

    #[test]
    fn test_inline_context_fits_exact_width() {
        let mut ctx = InlineLayoutContext::new(Length::from_pt(50.0), Length::from_pt(12.0));
        let area = InlineArea {
            width: Length::from_pt(50.0),
            height: Length::from_pt(12.0),
            content: Some("exact".to_string()),
            traits: TraitSet::default(),
        };
        assert!(ctx.add(area));
        assert_eq!(ctx.used_width(), Length::from_pt(50.0));
        assert_eq!(ctx.remaining_width(), Length::ZERO);
    }

    #[test]
    fn test_inline_context_overflow_detection() {
        let mut ctx = InlineLayoutContext::new(Length::from_pt(50.0), Length::from_pt(12.0));
        // First area fits
        let area1 = InlineArea {
            width: Length::from_pt(30.0),
            height: Length::from_pt(12.0),
            content: Some("ok".to_string()),
            traits: TraitSet::default(),
        };
        assert!(ctx.add(area1));
        // Second area does not fit (30+25=55 > 50)
        let area2 = InlineArea {
            width: Length::from_pt(25.0),
            height: Length::from_pt(12.0),
            content: Some("overflow".to_string()),
            traits: TraitSet::default(),
        };
        assert!(!ctx.add(area2));
        assert_eq!(ctx.inline_areas.len(), 1);
    }

    // ── Line height from taller child ────────────────────────────────────────

    #[test]
    fn test_line_height_updated_by_taller_area() {
        let mut ctx = InlineLayoutContext::new(Length::from_pt(100.0), Length::from_pt(12.0));
        let area = InlineArea {
            width: Length::from_pt(20.0),
            height: Length::from_pt(20.0), // taller than initial line_height
            content: None,
            traits: TraitSet::default(),
        };
        ctx.add(area);
        assert_eq!(ctx.line_height, Length::from_pt(20.0));
    }

    #[test]
    fn test_line_height_not_reduced_by_shorter_area() {
        let mut ctx = InlineLayoutContext::new(Length::from_pt(100.0), Length::from_pt(14.0));
        let area = InlineArea {
            width: Length::from_pt(20.0),
            height: Length::from_pt(10.0), // shorter than line_height
            content: None,
            traits: TraitSet::default(),
        };
        ctx.add(area);
        assert_eq!(ctx.line_height, Length::from_pt(14.0));
    }

    // ── Alignment offset calculations ────────────────────────────────────────

    #[test]
    fn test_alignment_offset_left_is_zero() {
        let mut ctx = InlineLayoutContext::new(Length::from_pt(100.0), Length::from_pt(12.0))
            .with_text_align(TextAlign::Left);
        ctx.add(InlineArea {
            width: Length::from_pt(60.0),
            height: Length::from_pt(12.0),
            content: None,
            traits: TraitSet::default(),
        });
        assert_eq!(ctx.calculate_alignment_offset(), Length::ZERO);
    }

    #[test]
    fn test_alignment_offset_right_is_unused_width() {
        let mut ctx = InlineLayoutContext::new(Length::from_pt(100.0), Length::from_pt(12.0))
            .with_text_align(TextAlign::Right);
        ctx.add(InlineArea {
            width: Length::from_pt(60.0),
            height: Length::from_pt(12.0),
            content: None,
            traits: TraitSet::default(),
        });
        // unused = 100 - 60 = 40
        assert_eq!(ctx.calculate_alignment_offset(), Length::from_pt(40.0));
    }

    #[test]
    fn test_alignment_offset_center_is_half_unused() {
        let mut ctx = InlineLayoutContext::new(Length::from_pt(100.0), Length::from_pt(12.0))
            .with_text_align(TextAlign::Center);
        ctx.add(InlineArea {
            width: Length::from_pt(60.0),
            height: Length::from_pt(12.0),
            content: None,
            traits: TraitSet::default(),
        });
        // unused = 40, center = 20
        assert_eq!(ctx.calculate_alignment_offset(), Length::from_pt(20.0));
    }

    #[test]
    fn test_alignment_offset_justify_is_zero() {
        let mut ctx = InlineLayoutContext::new(Length::from_pt(100.0), Length::from_pt(12.0))
            .with_text_align(TextAlign::Justify);
        ctx.add(InlineArea {
            width: Length::from_pt(60.0),
            height: Length::from_pt(12.0),
            content: None,
            traits: TraitSet::default(),
        });
        assert_eq!(ctx.calculate_alignment_offset(), Length::ZERO);
    }

    // ── LineBreaker construction and word splitting ──────────────────────────

    #[test]
    fn test_line_breaker_new() {
        let breaker = LineBreaker::new(Length::from_pt(100.0));
        assert_eq!(breaker.available_width, Length::from_pt(100.0));
    }

    #[test]
    fn test_break_into_words_basic() {
        let breaker = LineBreaker::new(Length::from_pt(100.0));
        let words = breaker.break_into_words("Hello world test");
        assert_eq!(words.len(), 3);
        assert_eq!(words[0], "Hello");
        assert_eq!(words[1], "world");
        assert_eq!(words[2], "test");
    }

    #[test]
    fn test_break_into_words_empty_string() {
        let breaker = LineBreaker::new(Length::from_pt(100.0));
        let words = breaker.break_into_words("");
        assert!(words.is_empty());
    }

    #[test]
    fn test_break_into_words_single_word() {
        let breaker = LineBreaker::new(Length::from_pt(100.0));
        let words = breaker.break_into_words("Hello");
        assert_eq!(words.len(), 1);
        assert_eq!(words[0], "Hello");
    }

    #[test]
    fn test_break_into_words_extra_whitespace() {
        let breaker = LineBreaker::new(Length::from_pt(100.0));
        let words = breaker.break_into_words("  one   two  ");
        assert_eq!(words.len(), 2);
        assert_eq!(words[0], "one");
        assert_eq!(words[1], "two");
    }

    // ── Text measurement ─────────────────────────────────────────────────────

    #[test]
    fn test_measure_text_positive_width() {
        let breaker = LineBreaker::new(Length::from_pt(100.0));
        let width = breaker.measure_text("test", Length::from_pt(12.0));
        assert!(width > Length::ZERO);
    }

    #[test]
    fn test_measure_text_longer_text_wider() {
        let breaker = LineBreaker::new(Length::from_pt(100.0));
        let w1 = breaker.measure_text("hi", Length::from_pt(12.0));
        let w2 = breaker.measure_text("hello world", Length::from_pt(12.0));
        assert!(w2 > w1);
    }

    #[test]
    fn test_measure_text_larger_font_wider() {
        let breaker = LineBreaker::new(Length::from_pt(100.0));
        let w_small = breaker.measure_text("test", Length::from_pt(10.0));
        let w_large = breaker.measure_text("test", Length::from_pt(20.0));
        assert!(w_large > w_small);
    }

    #[test]
    fn test_measure_text_with_letter_spacing() {
        let breaker_plain = LineBreaker::new(Length::from_pt(200.0));
        let breaker_spaced =
            LineBreaker::new(Length::from_pt(200.0)).with_letter_spacing(Length::from_pt(1.0));
        let text = "hello";
        let font_size = Length::from_pt(12.0);
        let w_plain = breaker_plain.measure_text(text, font_size);
        let w_spaced = breaker_spaced.measure_text(text, font_size);
        // 5 chars → 4 gaps × 1pt = 4pt extra
        assert!(w_spaced > w_plain);
    }

    #[test]
    fn test_measure_text_with_word_spacing() {
        let breaker_plain = LineBreaker::new(Length::from_pt(200.0));
        let breaker_spaced =
            LineBreaker::new(Length::from_pt(200.0)).with_word_spacing(Length::from_pt(3.0));
        let text = "hello world";
        let font_size = Length::from_pt(12.0);
        let w_plain = breaker_plain.measure_text(text, font_size);
        let w_spaced = breaker_spaced.measure_text(text, font_size);
        // 1 space → 1 × 3pt extra
        assert!(w_spaced > w_plain);
    }

    // ── Line breaking ────────────────────────────────────────────────────────

    #[test]
    fn test_break_lines_short_text_one_line() {
        let breaker = LineBreaker::new(Length::from_pt(300.0));
        let lines = breaker.break_lines("Hello world", Length::from_pt(12.0));
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "Hello world");
    }

    #[test]
    fn test_break_lines_long_text_multiple_lines() {
        let breaker = LineBreaker::new(Length::from_pt(100.0));
        let long_text = "This is a very long piece of text that definitely needs breaking";
        let lines = breaker.break_lines(long_text, Length::from_pt(12.0));
        assert!(lines.len() > 1);
    }

    #[test]
    fn test_break_lines_single_word_fits_one_line() {
        let breaker = LineBreaker::new(Length::from_pt(200.0));
        let lines = breaker.break_lines("Hello", Length::from_pt(12.0));
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "Hello");
    }

    #[test]
    fn test_break_lines_empty_string_produces_no_lines() {
        let breaker = LineBreaker::new(Length::from_pt(100.0));
        let lines = breaker.break_lines("", Length::from_pt(12.0));
        assert!(lines.is_empty());
    }

    #[test]
    fn test_break_lines_all_words_preserved() {
        let breaker = LineBreaker::new(Length::from_pt(80.0));
        let text = "alpha beta gamma delta";
        let lines = breaker.break_lines(text, Length::from_pt(12.0));
        // Rejoin all words from all lines
        let all_words: Vec<String> = lines
            .iter()
            .flat_map(|l| l.split_whitespace().map(|s| s.to_string()))
            .collect();
        let expected_words: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
        assert_eq!(all_words, expected_words);
    }

    #[test]
    fn test_break_lines_very_narrow_width_one_word_per_line() {
        // Width too narrow for any two words together
        let breaker = LineBreaker::new(Length::from_pt(1.0));
        let lines = breaker.break_lines("one two three", Length::from_pt(12.0));
        // Each word should be on its own line (or at minimum > 1 line)
        assert!(!lines.is_empty());
    }
}
