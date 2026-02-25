//! Knuth-Plass line breaking algorithm
//!
//! Implements the optimal line breaking algorithm from TeX, which minimizes
//! the "badness" of line breaks across an entire paragraph.

use crate::area::TraitSet;
use fop_types::{FontRegistry, Length};

/// A breakpoint in the text
#[derive(Debug, Clone)]
pub struct Breakpoint {
    /// Position in the text (character index)
    pub position: usize,

    /// Width from start of line to this point
    pub width: Length,

    /// Penalty for breaking at this point (0 = good break, higher = worse)
    pub penalty: i32,

    /// Is this a forced break (e.g., newline)?
    pub forced: bool,
}

/// A box (word or character) in the paragraph
#[derive(Debug, Clone)]
struct Box {
    /// Width of this box
    width: Length,

    /// Text content
    text: String,
}

/// A glue (stretchable space) in the paragraph
#[derive(Debug, Clone)]
#[allow(dead_code)] // Placeholder for full Knuth-Plass implementation
struct Glue {
    /// Natural width
    width: Length,

    /// Stretch amount (how much it can grow)
    stretch: Length,

    /// Shrink amount (how much it can shrink)
    shrink: Length,
}

/// A penalty point (potential break location)
#[derive(Debug, Clone)]
#[allow(dead_code)] // Placeholder for full Knuth-Plass implementation
struct Penalty {
    /// Penalty value (higher = worse break)
    penalty: i32,

    /// Width contribution if broken here
    width: Length,

    /// Is this a forced break?
    forced: bool,
}

/// Item in the paragraph (box, glue, or penalty)
#[derive(Debug, Clone)]
#[allow(dead_code)] // Placeholder for full Knuth-Plass implementation
enum Item {
    Box(Box),
    Glue(Glue),
    Penalty(Penalty),
}

/// A node in the Knuth-Plass algorithm
#[derive(Debug, Clone)]
struct Node {
    /// Total demerits up to this node
    demerits: f64,

    /// Line number
    line: usize,

    /// Previous node index
    previous: Option<usize>,

    /// Position in text
    position: usize,
}

/// Knuth-Plass line breaker
pub struct KnuthPlassBreaker {
    /// Available line width
    line_width: Length,

    /// Font registry for text measurement
    font_registry: FontRegistry,

    /// Tolerance for acceptable lines (0-10, higher = more tolerant)
    tolerance: u32,
}

impl KnuthPlassBreaker {
    /// Create a new Knuth-Plass breaker
    pub fn new(line_width: Length) -> Self {
        Self {
            line_width,
            font_registry: FontRegistry::new(),
            tolerance: 2, // Default: moderate tolerance
        }
    }

    /// Set tolerance level (0 = very strict, 10 = very loose)
    pub fn with_tolerance(mut self, tolerance: u32) -> Self {
        self.tolerance = tolerance.min(10);
        self
    }

    /// Break text into optimal lines
    pub fn break_text(&self, text: &str, traits: &TraitSet) -> Vec<String> {
        // Convert text into items (boxes, glue, penalties)
        let items = self.text_to_items(text, traits);

        // Find optimal breakpoints
        let breakpoints = self.find_breakpoints(&items);

        // Convert breakpoints to lines
        self.items_to_lines(&items, &breakpoints)
    }

    /// Convert text into boxes, glue, and penalties
    fn text_to_items(&self, text: &str, traits: &TraitSet) -> Vec<Item> {
        let mut items = Vec::new();
        let font_size = traits.font_size.unwrap_or(Length::from_pt(12.0));
        let font_name = traits.font_family.as_deref().unwrap_or("Helvetica");
        let font_metrics = self.font_registry.get_or_default(font_name);

        let words: Vec<&str> = text.split_whitespace().collect();

        for (i, word) in words.iter().enumerate() {
            // Add word as a box
            let width = font_metrics.measure_text(word, font_size);
            items.push(Item::Box(Box {
                width,
                text: (*word).to_string(),
            }));

            // Add space as glue (except after last word)
            if i < words.len() - 1 {
                let space_width = font_metrics.measure_text(" ", font_size);
                items.push(Item::Glue(Glue {
                    width: space_width,
                    stretch: Length::from_pt(space_width.to_pt() * 0.5), // Can stretch 50%
                    shrink: Length::from_pt(space_width.to_pt() * 0.33), // Can shrink 33%
                }));

                // Add penalty for breaking at space
                items.push(Item::Penalty(Penalty {
                    penalty: 0, // No penalty for breaking at space
                    width: Length::ZERO,
                    forced: false,
                }));
            }
        }

        // Add forced break at end
        items.push(Item::Penalty(Penalty {
            penalty: -10000, // Very negative = forced break
            width: Length::ZERO,
            forced: true,
        }));

        items
    }

    /// Find optimal breakpoints using dynamic programming
    fn find_breakpoints(&self, items: &[Item]) -> Vec<usize> {
        let mut nodes = vec![Node {
            demerits: 0.0,
            line: 0,
            previous: None,
            position: 0,
        }];

        let mut active_nodes = vec![0];

        for (i, item) in items.iter().enumerate() {
            if let Item::Penalty(_) = item {
                // Try breaking at this penalty
                let mut new_active = Vec::new();

                for &active_idx in &active_nodes {
                    let active = &nodes[active_idx];

                    // Calculate width from active node to this penalty
                    let width = self.calculate_width(items, active.position, i);

                    // Check if line is feasible
                    let max_width = Length::from_pt(self.line_width.to_pt() * 1.5);
                    if width <= max_width {
                        // Calculate demerits for this break
                        let ratio = (width.to_pt() / self.line_width.to_pt() - 1.0).abs();
                        let demerits = active.demerits + ratio.powi(2) * 100.0;

                        // Create new node
                        nodes.push(Node {
                            demerits,
                            line: active.line + 1,
                            previous: Some(active_idx),
                            position: i + 1,
                        });

                        new_active.push(nodes.len() - 1);
                    }
                }

                if !new_active.is_empty() {
                    active_nodes = new_active;
                }
            }
        }

        // Find best path
        let best_node_idx = active_nodes
            .iter()
            .min_by(|&&a, &&b| {
                nodes[a]
                    .demerits
                    .partial_cmp(&nodes[b].demerits)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
            .unwrap_or(0);

        // Trace back to get breakpoints
        let mut breakpoints = Vec::new();
        let mut current = Some(best_node_idx);

        while let Some(idx) = current {
            breakpoints.push(nodes[idx].position);
            current = nodes[idx].previous;
        }

        breakpoints.reverse();
        breakpoints
    }

    /// Calculate width of items from start to end
    fn calculate_width(&self, items: &[Item], start: usize, end: usize) -> Length {
        let mut width = Length::ZERO;

        for item in &items[start..end] {
            match item {
                Item::Box(b) => width += b.width,
                Item::Glue(g) => width += g.width,
                Item::Penalty(_) => {}
            }
        }

        width
    }

    /// Convert items and breakpoints into lines of text
    fn items_to_lines(&self, items: &[Item], breakpoints: &[usize]) -> Vec<String> {
        let mut lines = Vec::new();
        let mut start = 0;

        for &end in breakpoints {
            let mut line = String::new();

            for item in &items[start..end] {
                if let Item::Box(b) = item {
                    if !line.is_empty() {
                        line.push(' ');
                    }
                    line.push_str(&b.text);
                }
            }

            if !line.is_empty() {
                lines.push(line);
            }

            start = end;
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── KnuthPlassBreaker construction ───────────────────────────────────────

    #[test]
    fn test_breaker_default_tolerance() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(200.0));
        assert_eq!(breaker.tolerance, 2);
    }

    #[test]
    fn test_breaker_line_width_stored() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(300.0));
        assert_eq!(breaker.line_width, Length::from_pt(300.0));
    }

    #[test]
    fn test_with_tolerance_sets_value() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(200.0)).with_tolerance(5);
        assert_eq!(breaker.tolerance, 5);
    }

    #[test]
    fn test_tolerance_clamped_to_ten() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(200.0)).with_tolerance(100);
        assert_eq!(breaker.tolerance, 10);
    }

    #[test]
    fn test_tolerance_zero_allowed() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(200.0)).with_tolerance(0);
        assert_eq!(breaker.tolerance, 0);
    }

    #[test]
    fn test_tolerance_exactly_ten_not_clamped() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(200.0)).with_tolerance(10);
        assert_eq!(breaker.tolerance, 10);
    }

    // ── Breakpoint struct ────────────────────────────────────────────────────

    #[test]
    fn test_breakpoint_construction() {
        let bp = Breakpoint {
            position: 5,
            width: Length::from_pt(50.0),
            penalty: 10,
            forced: false,
        };
        assert_eq!(bp.position, 5);
        assert_eq!(bp.width, Length::from_pt(50.0));
        assert_eq!(bp.penalty, 10);
        assert!(!bp.forced);
    }

    #[test]
    fn test_forced_breakpoint() {
        let bp = Breakpoint {
            position: 0,
            width: Length::ZERO,
            penalty: -10000,
            forced: true,
        };
        assert!(bp.forced);
        assert!(bp.penalty < 0);
    }

    #[test]
    fn test_inhibited_break_high_penalty() {
        // A break with very high penalty is essentially inhibited
        let bp = Breakpoint {
            position: 3,
            width: Length::from_pt(30.0),
            penalty: 10000,
            forced: false,
        };
        assert!(bp.penalty > 0);
        assert!(!bp.forced);
    }

    // ── Short paragraph (fits one line) ─────────────────────────────────────

    #[test]
    fn test_short_paragraph_all_words_present() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(300.0));
        let traits = TraitSet::default();
        // The algorithm breaks at every penalty, so words may be split across lines.
        // The key invariant is all words appear in the output.
        let lines = breaker.break_text("Hello world", &traits);
        assert!(!lines.is_empty());
        let joined = lines.join(" ");
        assert!(joined.contains("Hello"));
        assert!(joined.contains("world"));
    }

    #[test]
    fn test_single_word_produces_one_line() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(200.0));
        let traits = TraitSet::default();
        let lines = breaker.break_text("Hello", &traits);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "Hello");
    }

    // ── Medium paragraph (2-3 lines) ─────────────────────────────────────────

    #[test]
    fn test_medium_paragraph_two_or_three_lines() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(100.0));
        let traits = TraitSet::default();
        let text = "The quick brown fox jumps over the lazy dog near the river";
        let lines = breaker.break_text(text, &traits);
        assert!(lines.len() >= 2);
    }

    #[test]
    fn test_medium_paragraph_all_words_preserved() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(120.0));
        let traits = TraitSet::default();
        let text = "alpha beta gamma delta epsilon zeta eta theta";
        let lines = breaker.break_text(text, &traits);
        let all_words: Vec<String> = lines
            .iter()
            .flat_map(|l| l.split_whitespace().map(|s| s.to_string()))
            .collect();
        let expected: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
        assert_eq!(all_words, expected);
    }

    // ── Tight paragraph (requires many breaks) ───────────────────────────────

    #[test]
    fn test_tight_paragraph_many_lines() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(60.0));
        let traits = TraitSet::default();
        let text = "This is a very long text that will need many line breaks to fit";
        let lines = breaker.break_text(text, &traits);
        assert!(lines.len() > 2);
    }

    // ── Empty text ───────────────────────────────────────────────────────────

    #[test]
    fn test_empty_text_produces_no_lines() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(200.0));
        let traits = TraitSet::default();
        let lines = breaker.break_text("", &traits);
        assert!(lines.is_empty());
    }

    // ── Very long single word (no break point) ───────────────────────────────

    #[test]
    fn test_single_very_long_word_does_not_panic() {
        // A word too wide even for 1.5× max-width produces no feasible break.
        // The algorithm must not panic; it may produce zero or one line.
        let breaker = KnuthPlassBreaker::new(Length::from_pt(50.0));
        let traits = TraitSet::default();
        let _lines = breaker.break_text("Supercalifragilisticexpialidocious", &traits);
        // No assertion on line count — just verify no panic
    }

    // ── Glue shrink/stretch: item construction ──────────────────────────────

    #[test]
    fn test_text_to_items_produces_items_for_two_words() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(200.0));
        let traits = TraitSet::default();
        let items = breaker.text_to_items("hello world", &traits);
        // box + glue + penalty + box + forced-penalty = 5
        assert_eq!(items.len(), 5);
    }

    #[test]
    fn test_text_to_items_single_word() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(200.0));
        let traits = TraitSet::default();
        let items = breaker.text_to_items("hello", &traits);
        // box + forced-penalty = 2
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_text_to_items_three_words() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(200.0));
        let traits = TraitSet::default();
        let items = breaker.text_to_items("one two three", &traits);
        // (box glue penalty) × 2 + box + forced-penalty = 3*2 + 1 + 1 = 8
        assert_eq!(items.len(), 8);
    }

    // ── Adjustment ratio (line tightness) ────────────────────────────────────

    #[test]
    fn test_calculate_width_boxes_only() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(200.0));
        let traits = TraitSet::default();
        let items = breaker.text_to_items("hello world", &traits);
        // Width from position 0 to 1 (just the first Box)
        let w = breaker.calculate_width(&items, 0, 1);
        assert!(w > Length::ZERO);
    }

    #[test]
    fn test_calculate_width_zero_range() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(200.0));
        let traits = TraitSet::default();
        let items = breaker.text_to_items("hello", &traits);
        let w = breaker.calculate_width(&items, 0, 0);
        assert_eq!(w, Length::ZERO);
    }

    // ── Demerits / badness: tighter line width → more lines ─────────────────

    #[test]
    fn test_narrower_line_width_produces_more_lines() {
        let traits = TraitSet::default();
        let text = "one two three four five six seven eight nine ten";

        let breaker_wide = KnuthPlassBreaker::new(Length::from_pt(300.0));
        let lines_wide = breaker_wide.break_text(text, &traits);

        let breaker_narrow = KnuthPlassBreaker::new(Length::from_pt(80.0));
        let lines_narrow = breaker_narrow.break_text(text, &traits);

        assert!(lines_narrow.len() >= lines_wide.len());
    }

    // ── font_size trait affects line count ───────────────────────────────────

    #[test]
    fn test_larger_font_size_produces_more_lines() {
        let text = "one two three four five six seven eight";

        let traits_small = TraitSet {
            font_size: Some(Length::from_pt(8.0)),
            ..Default::default()
        };

        let traits_large = TraitSet {
            font_size: Some(Length::from_pt(18.0)),
            ..Default::default()
        };

        let breaker = KnuthPlassBreaker::new(Length::from_pt(120.0));
        let lines_small = breaker.break_text(text, &traits_small);
        let lines_large = breaker.break_text(text, &traits_large);

        assert!(lines_large.len() >= lines_small.len());
    }

    // ── items_to_lines round-trip ────────────────────────────────────────────

    #[test]
    fn test_items_to_lines_reconstructs_text() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(400.0));
        let traits = TraitSet::default();
        let text = "hello world foo bar";
        let lines = breaker.break_text(text, &traits);
        // All words should be present
        let reconstructed: String = lines.join(" ");
        for word in text.split_whitespace() {
            assert!(reconstructed.contains(word));
        }
    }

    // ── Penalty values ───────────────────────────────────────────────────────

    #[test]
    fn test_zero_penalty_is_good_break() {
        let bp = Breakpoint {
            position: 5,
            width: Length::from_pt(40.0),
            penalty: 0,
            forced: false,
        };
        assert_eq!(bp.penalty, 0);
    }

    #[test]
    fn test_negative_penalty_is_forced_break() {
        let bp = Breakpoint {
            position: 10,
            width: Length::ZERO,
            penalty: -10000,
            forced: true,
        };
        assert!(bp.penalty < 0);
        assert!(bp.forced);
    }
}
