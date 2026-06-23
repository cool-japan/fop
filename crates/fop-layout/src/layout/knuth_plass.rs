//! Knuth-Plass optimal line breaking algorithm.
//!
//! This is a faithful implementation of the total-fit line breaking algorithm
//! described by Donald Knuth and Michael Plass in *Breaking Paragraphs into
//! Lines* (Software — Practice and Experience, 1981; reprinted in *Digital
//! Typography*). Rather than greedily filling each line, it models a paragraph
//! as a stream of **boxes** (words), **glue** (stretchable/shrinkable inter-word
//! spaces) and **penalties** (break opportunities), then finds — via dynamic
//! programming over an active-node list — the sequence of breakpoints that
//! minimises the *total demerits* of the paragraph.
//!
//! For every candidate line between two breakpoints the algorithm computes the
//! **adjustment ratio** `r` from the available stretch/shrink, derives the
//! line's **badness** `100·|r|³`, and combines it with the breakpoint penalty
//! and fitness-class adjacency into the standard demerit formula. Forced breaks
//! (penalty −∞) and prohibited breaks (penalty +∞) are honoured.
//!
//! ## Glue model and alignment
//!
//! Inter-word glue carries a natural space width plus stretch and shrink. For
//! justified text the shrink is non-zero, so the breaker may set a line tighter
//! than its measure (the spaces are compressed to fill exactly `content_width`).
//! For ragged text (left/right/centre) the shrink is zero — exactly how TeX
//! implements `\raggedright` — which guarantees that no chosen line is wider
//! than `content_width` in its natural setting.

use crate::area::TraitSet;
use crate::layout::properties::measure_text_metrics;
use fop_types::{FontRegistry, Length};

/// Sentinel for an infinitely undesirable (prohibited) break.
const INFINITE_PENALTY: f64 = 1.0e9;

/// Sentinel for a forced break (penalty −∞).
const FORCED_PENALTY: f64 = -1.0e9;

/// `\linepenalty` — a constant added to every line's badness before squaring,
/// biasing the optimiser towards paragraphs with fewer lines.
const LINE_PENALTY: f64 = 10.0;

/// `\adjdemerits` — extra demerits charged when two consecutive lines differ in
/// fitness class by more than one (e.g. a very-loose line beside a tight one).
const ADJ_DEMERITS: f64 = 10_000.0;

/// `\doublehyphendemerits` — charged when two consecutive breakpoints are both
/// flagged. Only the forced end-of-paragraph break is flagged here, so this is
/// effectively reserved for future hyphenation support.
const FLAGGED_DEMERITS: f64 = 100.0;

/// Natural stretch of the finishing glue appended before the forced break, in
/// points. A large finite value approximates the infinite (`fil`) stretch TeX
/// uses so the final line may be arbitrarily short without incurring badness.
const FINISHING_STRETCH_PT: f64 = 10_000.0;

/// A chosen breakpoint in the paragraph.
///
/// This is the public, self-describing view of a break opportunity. The
/// internal optimiser works on a lower-level item/active-node representation,
/// but a [`Breakpoint`] captures everything a caller needs to reason about an
/// individual break: where it is, how wide the line up to it is, the penalty
/// charged for breaking there, and whether the break is forced.
#[derive(Debug, Clone)]
pub struct Breakpoint {
    /// Position in the text (character index).
    pub position: usize,

    /// Width from start of line to this point.
    pub width: Length,

    /// Penalty for breaking at this point (0 = good break, higher = worse).
    pub penalty: i32,

    /// Is this a forced break (e.g., newline)?
    pub forced: bool,
}

/// A box: an unbreakable run of glyphs (one word).
#[derive(Debug, Clone)]
struct TextBox {
    /// Advance width of the word.
    width: Length,

    /// The word's text, retained so lines can be reconstructed.
    text: String,
}

/// Glue: a flexible space that can stretch and shrink.
#[derive(Debug, Clone)]
struct Glue {
    /// Natural width.
    width: Length,

    /// How much the glue may grow beyond its natural width.
    stretch: Length,

    /// How much the glue may shrink below its natural width.
    shrink: Length,
}

/// A penalty: a potential break location with an associated cost.
#[derive(Debug, Clone)]
struct Penalty {
    /// Penalty value (`FORCED_PENALTY` = forced, `INFINITE_PENALTY` = prohibited).
    penalty: f64,

    /// Extra width contributed if the line is broken here (e.g. a hyphen).
    width: Length,

    /// Whether this is a flagged break (used for double-flag demerits).
    flagged: bool,
}

/// An item in the paragraph's horizontal list.
#[derive(Debug, Clone)]
enum Item {
    /// A word.
    Box(TextBox),
    /// An inter-word space.
    Glue(Glue),
    /// A break opportunity.
    Penalty(Penalty),
}

impl Item {
    /// Is this item a box?
    fn is_box(&self) -> bool {
        matches!(self, Item::Box(_))
    }
}

/// A single laid-out line together with its natural (unjustified) advance width.
#[derive(Debug, Clone, PartialEq)]
pub struct LineBox {
    /// The line's text (words joined by single spaces).
    pub text: String,

    /// The natural advance width of [`LineBox::text`] in the resolved font.
    pub natural_width: Length,
}

/// An active node in the dynamic-programming frontier: a feasible breakpoint
/// from which subsequent lines may start.
#[derive(Debug, Clone)]
struct Node {
    /// Index, into the item list, of the breakpoint this node represents.
    position: usize,

    /// The number of the line that *ends* at this breakpoint.
    line: usize,

    /// Fitness class (0=tight, 1=normal, 2=loose, 3=very loose) of that line.
    fitness: u8,

    /// Running total of box+glue width at the start of the *next* line.
    total_width: Length,

    /// Running total of glue stretch at the start of the next line.
    total_stretch: Length,

    /// Running total of glue shrink at the start of the next line.
    total_shrink: Length,

    /// Minimum total demerits to reach this breakpoint.
    demerits: f64,

    /// Index of the predecessor node (None for the paragraph-start node).
    previous: Option<usize>,

    /// Whether the breakpoint at this node is flagged.
    flagged: bool,
}

/// Knuth-Plass optimal line breaker.
pub struct KnuthPlassBreaker {
    /// Available line width (the measure).
    line_width: Length,

    /// Extra indentation removed from the first line's measure.
    first_line_indent: Length,

    /// Whether the paragraph is justified (enables glue shrink).
    justify: bool,

    /// Font registry for text measurement.
    font_registry: FontRegistry,

    /// Tolerance for acceptable lines (0 = strict, 10 = very loose).
    tolerance: u32,
}

impl KnuthPlassBreaker {
    /// Create a new Knuth-Plass breaker for the given line width (measure).
    pub fn new(line_width: Length) -> Self {
        Self {
            line_width,
            first_line_indent: Length::ZERO,
            justify: false,
            font_registry: FontRegistry::new(),
            tolerance: 2, // Default: moderate tolerance
        }
    }

    /// Set tolerance level (0 = very strict, 10 = very loose).
    ///
    /// Higher tolerance widens the band of adjustment ratios that count as
    /// feasible, allowing looser lines in exchange for fewer breaks.
    pub fn with_tolerance(mut self, tolerance: u32) -> Self {
        self.tolerance = tolerance.min(10);
        self
    }

    /// Set the first-line indentation, narrowing the measure of the first line.
    pub fn with_first_line_indent(mut self, indent: Length) -> Self {
        self.first_line_indent = indent;
        self
    }

    /// Enable or disable justification (controls whether inter-word glue may
    /// shrink). Ragged paragraphs use zero shrink so no line is set overfull.
    pub fn with_justify(mut self, justify: bool) -> Self {
        self.justify = justify;
        self
    }

    /// Break `text` into the optimal sequence of lines, returning just the line
    /// strings. Empty or whitespace-only input yields no lines.
    pub fn break_text(&self, text: &str, traits: &TraitSet) -> Vec<String> {
        self.break_into_lines(text, traits)
            .into_iter()
            .map(|line| line.text)
            .collect()
    }

    /// Break `text` into the optimal sequence of lines, returning each line's
    /// text together with its measured natural width.
    ///
    /// The natural widths are measured with the same font metrics the breaker
    /// uses internally, so callers can position and align lines without
    /// re-deriving glyph advances.
    pub fn break_into_lines(&self, text: &str, traits: &TraitSet) -> Vec<LineBox> {
        let items = self.text_to_items(text, traits);
        if items.is_empty() {
            return Vec::new();
        }
        let breaks = self.find_breaks(&items);
        self.items_to_line_boxes(&items, &breaks)
    }

    /// The target width (measure) for the `line`-th line (1-based). The first
    /// line is narrowed by the configured first-line indent.
    fn target_width(&self, line: usize) -> Length {
        if line == 1 && self.first_line_indent > Length::ZERO {
            self.line_width - self.first_line_indent
        } else {
            self.line_width
        }
    }

    /// Convert `text` into the box/glue/penalty item stream.
    ///
    /// Each word becomes a box; each inter-word space becomes glue; the
    /// paragraph is terminated by a finishing glue (with near-infinite stretch
    /// so the last line may be short) followed by a forced-break penalty.
    fn text_to_items(&self, text: &str, traits: &TraitSet) -> Vec<Item> {
        let mut items = Vec::new();

        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return items;
        }

        let space_width = measure_text_metrics(" ", traits, &self.font_registry);
        let space_pt = space_width.to_pt();
        // Reasonable inter-word elasticity. Shrink is only enabled when the
        // paragraph is justified; ragged setting keeps lines from overflowing.
        let stretch = Length::from_pt(space_pt * 0.5);
        let shrink = if self.justify {
            Length::from_pt(space_pt / 3.0)
        } else {
            Length::ZERO
        };

        for (i, word) in words.iter().enumerate() {
            let width = measure_text_metrics(word, traits, &self.font_registry);
            items.push(Item::Box(TextBox {
                width,
                text: (*word).to_string(),
            }));

            if i + 1 < words.len() {
                items.push(Item::Glue(Glue {
                    width: space_width,
                    stretch,
                    shrink,
                }));
            }
        }

        // Finishing glue: lets the final line stop short without badness.
        items.push(Item::Glue(Glue {
            width: Length::ZERO,
            stretch: Length::from_pt(FINISHING_STRETCH_PT),
            shrink: Length::ZERO,
        }));
        // Forced break at the end of the paragraph.
        items.push(Item::Penalty(Penalty {
            penalty: FORCED_PENALTY,
            width: Length::ZERO,
            flagged: true,
        }));

        items
    }

    /// Run the dynamic program and return the chosen breakpoint item indices,
    /// in paragraph order (excluding the implicit start-of-paragraph node).
    fn find_breaks(&self, items: &[Item]) -> Vec<usize> {
        // Feasibility band: tighter tolerance permits less stretch.
        let max_ratio = 2.0 + self.tolerance as f64 * 2.0;

        let mut nodes: Vec<Node> = vec![Node {
            position: 0,
            line: 0,
            fitness: 1,
            total_width: Length::ZERO,
            total_stretch: Length::ZERO,
            total_shrink: Length::ZERO,
            demerits: 0.0,
            previous: None,
            flagged: false,
        }];
        let mut active: Vec<usize> = vec![0];

        let mut sum_width = Length::ZERO;
        let mut sum_stretch = Length::ZERO;
        let mut sum_shrink = Length::ZERO;

        for b in 0..items.len() {
            let legal = match &items[b] {
                Item::Box(_) => false,
                // A glue is a legal breakpoint only when preceded by a box.
                Item::Glue(_) => b > 0 && items[b - 1].is_box(),
                // Any penalty that is not strictly prohibited.
                Item::Penalty(p) => p.penalty < INFINITE_PENALTY,
            };

            if legal {
                self.consider_break(
                    b,
                    items,
                    &mut nodes,
                    &mut active,
                    sum_width,
                    sum_stretch,
                    sum_shrink,
                    max_ratio,
                );
            }

            match &items[b] {
                Item::Box(bx) => sum_width += bx.width,
                Item::Glue(g) => {
                    sum_width += g.width;
                    sum_stretch += g.stretch;
                    sum_shrink += g.shrink;
                }
                Item::Penalty(_) => {}
            }
        }

        // The optimum is the lowest-demerit node on the final frontier (which,
        // thanks to the forced end break, all terminate at the paragraph end).
        let best_final = active.iter().copied().min_by(|&a, &b| {
            nodes[a]
                .demerits
                .partial_cmp(&nodes[b].demerits)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut breaks = Vec::new();
        let mut cursor = best_final;
        while let Some(idx) = cursor {
            // Skip the synthetic start node (the only node with no predecessor).
            if nodes[idx].previous.is_some() {
                breaks.push(nodes[idx].position);
            }
            cursor = nodes[idx].previous;
        }
        breaks.reverse();
        breaks
    }

    /// Consider breaking the paragraph at item `b`, updating the active list.
    #[allow(clippy::too_many_arguments)]
    fn consider_break(
        &self,
        b: usize,
        items: &[Item],
        nodes: &mut Vec<Node>,
        active: &mut Vec<usize>,
        sum_width: Length,
        sum_stretch: Length,
        sum_shrink: Length,
        max_ratio: f64,
    ) {
        let (penalty_b, flagged_b, penalty_width_b) = match &items[b] {
            Item::Penalty(p) => (p.penalty, p.flagged, p.width),
            // Breaking at a glue carries no penalty.
            _ => (0.0, false, Length::ZERO),
        };
        let forced = penalty_b <= FORCED_PENALTY;

        // Best feasible predecessor per fitness class (the DP state), plus an
        // overall least-demerit candidate used as a guaranteed fall-back so the
        // paragraph is never lost (e.g. for a single over-long word).
        let mut best: [Option<(usize, f64)>; 4] = [None, None, None, None];
        let mut fallback: Option<(usize, f64, u8)> = None;

        let mut retained: Vec<usize> = Vec::with_capacity(active.len());

        for &a_idx in active.iter() {
            let a = &nodes[a_idx];
            let line_w = (sum_width - a.total_width) + penalty_width_b;
            let target = self.target_width(a.line + 1);
            let r = adjustment_ratio(
                line_w,
                sum_stretch - a.total_stretch,
                sum_shrink - a.total_shrink,
                target,
            );

            // A node whose line is already overfull (r < −1), or any node when
            // the break is forced, cannot be extended past this point.
            let deactivate = r < -1.0 || forced;

            let badness = if r.is_finite() {
                100.0 * r.abs().powi(3)
            } else {
                f64::INFINITY
            };
            let line_d = line_demerits(badness, penalty_b, flagged_b, a.flagged);
            let fclass = fitness_class(r);
            let adj = if (fclass as i32 - a.fitness as i32).abs() > 1 {
                ADJ_DEMERITS
            } else {
                0.0
            };
            let total_d = a.demerits + line_d + adj;

            if r >= -1.0 && r <= max_ratio {
                let slot = &mut best[fclass as usize];
                if slot.map(|(_, d)| total_d < d).unwrap_or(true) {
                    *slot = Some((a_idx, total_d));
                }
            }

            let fb_d = if total_d.is_finite() {
                total_d
            } else {
                a.demerits + 1.0e18
            };
            if fallback.map(|(_, d, _)| fb_d < d).unwrap_or(true) {
                fallback = Some((a_idx, fb_d, fclass));
            }

            if !deactivate {
                retained.push(a_idx);
            }
        }

        *active = retained;

        let (next_w, next_y, next_z) =
            self.compute_sums_after(b, items, sum_width, sum_stretch, sum_shrink);

        let any_feasible = best.iter().any(|c| c.is_some());
        if any_feasible {
            for (fclass, candidate) in best.iter().enumerate() {
                if let Some((a_idx, total_d)) = *candidate {
                    let new_line = nodes[a_idx].line + 1;
                    nodes.push(Node {
                        position: b,
                        line: new_line,
                        fitness: fclass as u8,
                        total_width: next_w,
                        total_stretch: next_y,
                        total_shrink: next_z,
                        demerits: total_d,
                        previous: Some(a_idx),
                        flagged: flagged_b,
                    });
                    active.push(nodes.len() - 1);
                }
            }
        } else if forced || active.is_empty() {
            // No feasible break exists, but the paragraph must continue: a
            // forced break demands it, or every active node was just removed.
            if let Some((a_idx, total_d, fclass)) = fallback {
                let new_line = nodes[a_idx].line + 1;
                nodes.push(Node {
                    position: b,
                    line: new_line,
                    fitness: fclass,
                    total_width: next_w,
                    total_stretch: next_y,
                    total_shrink: next_z,
                    demerits: total_d,
                    previous: Some(a_idx),
                    flagged: flagged_b,
                });
                active.push(nodes.len() - 1);
            }
        }
    }

    /// Compute the running width/stretch/shrink totals at the start of the line
    /// that *follows* a break at item `b`, discarding the leading glue that is
    /// dropped at a line break.
    fn compute_sums_after(
        &self,
        b: usize,
        items: &[Item],
        sum_width: Length,
        sum_stretch: Length,
        sum_shrink: Length,
    ) -> (Length, Length, Length) {
        let mut w = sum_width;
        let mut y = sum_stretch;
        let mut z = sum_shrink;
        let mut i = b;
        while i < items.len() {
            match &items[i] {
                Item::Box(_) => break,
                Item::Glue(g) => {
                    w += g.width;
                    y += g.stretch;
                    z += g.shrink;
                    i += 1;
                }
                Item::Penalty(p) => {
                    if p.penalty <= FORCED_PENALTY && i > b {
                        break;
                    }
                    i += 1;
                }
            }
        }
        (w, y, z)
    }

    /// Sum the natural widths of boxes and glue in `items[start..end]`.
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

    /// Reconstruct the line strings (and natural widths) from chosen breakpoints.
    fn items_to_line_boxes(&self, items: &[Item], breaks: &[usize]) -> Vec<LineBox> {
        let mut lines = Vec::new();
        let mut start = 0usize;

        for &end in breaks {
            let mut words: Vec<&str> = Vec::new();
            for item in &items[start..end] {
                if let Item::Box(b) = item {
                    words.push(b.text.as_str());
                }
            }

            if !words.is_empty() {
                let text = words.join(" ");
                // The natural width is the sum of the boxes and the interior
                // inter-word glue between this line's breakpoints — exactly what
                // `calculate_width` accumulates over the half-open item range,
                // and identical to re-measuring the joined string.
                let natural_width = self.calculate_width(items, start, end);
                lines.push(LineBox {
                    text,
                    natural_width,
                });
            }

            start = end + 1;
        }

        lines
    }
}

/// Compute the adjustment ratio `r` for a line of natural width `line_w` with
/// the given available `stretch`/`shrink`, set to the `target` measure.
///
/// * `line_w < target` → stretch (`r ≥ 0`); `r = +∞` when no stretch exists.
/// * `line_w > target` → shrink  (`r < 0`);  `r = −∞` when no shrink exists.
/// * `line_w == target` → `r = 0`.
fn adjustment_ratio(line_w: Length, stretch: Length, shrink: Length, target: Length) -> f64 {
    let lw = line_w.to_pt();
    let t = target.to_pt();
    if lw < t {
        let s = stretch.to_pt();
        if s > 0.0 {
            (t - lw) / s
        } else {
            INFINITE_PENALTY
        }
    } else if lw > t {
        let sh = shrink.to_pt();
        if sh > 0.0 {
            (t - lw) / sh
        } else {
            -INFINITE_PENALTY
        }
    } else {
        0.0
    }
}

/// Classify a line by its adjustment ratio into one of four fitness classes:
/// 0 = tight, 1 = normal, 2 = loose, 3 = very loose.
fn fitness_class(r: f64) -> u8 {
    if r < -0.5 {
        0
    } else if r <= 0.5 {
        1
    } else if r <= 1.0 {
        2
    } else {
        3
    }
}

/// The standard Knuth-Plass per-line demerit function.
///
/// `d = (linepenalty + badness)²`, adjusted by the breakpoint penalty
/// (`+p²` for positive penalties, `−p²` for negative non-forced penalties, and
/// nothing for a forced break), plus a flagged-pair surcharge.
fn line_demerits(badness: f64, penalty: f64, flagged: bool, prev_flagged: bool) -> f64 {
    let base = LINE_PENALTY + badness;
    let mut d = base * base;
    if (0.0..INFINITE_PENALTY).contains(&penalty) {
        d += penalty * penalty;
    } else if penalty > FORCED_PENALTY && penalty < 0.0 {
        d -= penalty * penalty;
    }
    if flagged && prev_flagged {
        d += FLAGGED_DEMERITS;
    }
    d
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
        // 300pt comfortably fits "Hello world" on a single line; the key
        // invariant verified here is that every word survives line breaking.
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

    #[test]
    fn test_whitespace_only_text_produces_no_lines() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(200.0));
        let traits = TraitSet::default();
        let lines = breaker.break_text("    \t  ", &traits);
        assert!(lines.is_empty());
    }

    // ── Very long single word (no break point) ───────────────────────────────

    #[test]
    fn test_single_very_long_word_does_not_panic() {
        // A word wider than the measure has no feasible break; the breaker must
        // not panic and (via the fall-back) must still emit the word.
        let breaker = KnuthPlassBreaker::new(Length::from_pt(50.0));
        let traits = TraitSet::default();
        let _lines = breaker.break_text("Supercalifragilisticexpialidocious", &traits);
    }

    #[test]
    fn test_overlong_word_emitted_as_single_line() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(50.0));
        let traits = TraitSet::default();
        let lines = breaker.break_text("Supercalifragilisticexpialidocious", &traits);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "Supercalifragilisticexpialidocious");
    }

    // ── Item construction ────────────────────────────────────────────────────

    #[test]
    fn test_text_to_items_produces_items_for_two_words() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(200.0));
        let traits = TraitSet::default();
        let items = breaker.text_to_items("hello world", &traits);
        // box + glue + box + finishing-glue + forced-penalty = 5
        assert_eq!(items.len(), 5);
    }

    #[test]
    fn test_text_to_items_single_word() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(200.0));
        let traits = TraitSet::default();
        let items = breaker.text_to_items("hello", &traits);
        // box + finishing-glue + forced-penalty = 3
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_text_to_items_three_words() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(200.0));
        let traits = TraitSet::default();
        let items = breaker.text_to_items("one two three", &traits);
        // box,glue,box,glue,box + finishing-glue + forced-penalty = 7
        assert_eq!(items.len(), 7);
    }

    #[test]
    fn test_text_to_items_justify_has_shrink() {
        // Ragged glue carries zero shrink; justified glue carries shrink.
        let ragged = KnuthPlassBreaker::new(Length::from_pt(200.0));
        let justified = KnuthPlassBreaker::new(Length::from_pt(200.0)).with_justify(true);
        let traits = TraitSet::default();

        let ragged_items = ragged.text_to_items("hello world", &traits);
        let justified_items = justified.text_to_items("hello world", &traits);

        // The inter-word glue is item index 1 in both streams.
        let ragged_shrink = match &ragged_items[1] {
            Item::Glue(g) => g.shrink,
            _ => panic!("expected glue at index 1"),
        };
        let justified_shrink = match &justified_items[1] {
            Item::Glue(g) => g.shrink,
            _ => panic!("expected glue at index 1"),
        };
        assert_eq!(ragged_shrink, Length::ZERO);
        assert!(justified_shrink > Length::ZERO);
    }

    // ── calculate_width helper ───────────────────────────────────────────────

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

    // ── Line widths respect the measure ──────────────────────────────────────

    #[test]
    fn test_break_into_lines_widths_positive() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(120.0));
        let traits = TraitSet::default();
        let lines = breaker.break_into_lines("alpha beta gamma delta epsilon", &traits);
        assert!(!lines.is_empty());
        for line in &lines {
            assert!(line.natural_width > Length::ZERO);
        }
    }

    #[test]
    fn test_ragged_lines_each_within_measure() {
        // Ragged (default) glue has zero shrink, so no chosen line may exceed
        // the measure in its natural setting.
        let width = Length::from_pt(100.0);
        let breaker = KnuthPlassBreaker::new(width);
        let traits = TraitSet::default();
        let text = "The quick brown fox jumps over the lazy dog near the river bank today";
        let lines = breaker.break_into_lines(text, &traits);
        assert!(lines.len() >= 2);
        for line in &lines {
            assert!(
                line.natural_width <= width,
                "line {:?} natural width {}pt exceeds measure {}pt",
                line.text,
                line.natural_width.to_pt(),
                width.to_pt()
            );
        }
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

    // ── First-line indent narrows the first line ─────────────────────────────

    #[test]
    fn test_first_line_indent_can_increase_line_count() {
        let traits = TraitSet::default();
        let text = "alpha beta gamma delta epsilon zeta eta theta iota kappa";

        let plain = KnuthPlassBreaker::new(Length::from_pt(120.0));
        let indented = KnuthPlassBreaker::new(Length::from_pt(120.0))
            .with_first_line_indent(Length::from_pt(60.0));

        let plain_lines = plain.break_text(text, &traits);
        let indented_lines = indented.break_text(text, &traits);

        // A large first-line indent removes capacity, never adding capacity, so
        // the indented paragraph needs at least as many lines.
        assert!(indented_lines.len() >= plain_lines.len());
        // Both reconstruct the same words in order.
        let words = |ls: &[String]| -> Vec<String> {
            ls.iter()
                .flat_map(|l| l.split_whitespace().map(|s| s.to_string()))
                .collect()
        };
        assert_eq!(words(&plain_lines), words(&indented_lines));
    }

    // ── break_text round-trip ────────────────────────────────────────────────

    #[test]
    fn test_break_text_reconstructs_all_words() {
        let breaker = KnuthPlassBreaker::new(Length::from_pt(400.0));
        let traits = TraitSet::default();
        let text = "hello world foo bar";
        let lines = breaker.break_text(text, &traits);
        let reconstructed: String = lines.join(" ");
        for word in text.split_whitespace() {
            assert!(reconstructed.contains(word));
        }
    }

    // ── adjustment_ratio / fitness_class / demerits unit checks ──────────────

    #[test]
    fn test_adjustment_ratio_exact_fit_is_zero() {
        let r = adjustment_ratio(
            Length::from_pt(100.0),
            Length::from_pt(10.0),
            Length::from_pt(10.0),
            Length::from_pt(100.0),
        );
        assert_eq!(r, 0.0);
    }

    #[test]
    fn test_adjustment_ratio_needs_stretch_is_positive() {
        // line 90 < target 100, 10pt stretch → r = +1.0
        let r = adjustment_ratio(
            Length::from_pt(90.0),
            Length::from_pt(10.0),
            Length::from_pt(10.0),
            Length::from_pt(100.0),
        );
        assert!((r - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_adjustment_ratio_needs_shrink_is_negative() {
        // line 105 > target 100, 10pt shrink → r = -0.5
        let r = adjustment_ratio(
            Length::from_pt(105.0),
            Length::from_pt(10.0),
            Length::from_pt(10.0),
            Length::from_pt(100.0),
        );
        assert!((r + 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_adjustment_ratio_no_shrink_is_overfull() {
        // line wider than target with zero shrink → strongly negative (overfull)
        let r = adjustment_ratio(
            Length::from_pt(120.0),
            Length::from_pt(10.0),
            Length::ZERO,
            Length::from_pt(100.0),
        );
        assert!(r < -1.0);
    }

    #[test]
    fn test_fitness_classes() {
        assert_eq!(fitness_class(-0.8), 0); // tight
        assert_eq!(fitness_class(0.0), 1); // normal
        assert_eq!(fitness_class(0.8), 2); // loose
        assert_eq!(fitness_class(2.0), 3); // very loose
    }

    #[test]
    fn test_line_demerits_positive_penalty_increases() {
        let without = line_demerits(0.0, 0.0, false, false);
        let with = line_demerits(0.0, 50.0, false, false);
        assert!(with > without);
    }

    #[test]
    fn test_line_demerits_forced_penalty_no_squared_term() {
        // A forced (−∞) penalty must not add or subtract a penalty-squared term.
        let forced = line_demerits(0.0, FORCED_PENALTY, false, false);
        let base = LINE_PENALTY * LINE_PENALTY;
        assert!((forced - base).abs() < 1e-6);
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
