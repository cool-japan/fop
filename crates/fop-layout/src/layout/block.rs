//! Block-level layout algorithms
//!
//! Handles vertical stacking of block-level areas.

use fop_types::{Length, Point, Rect, Result, Size};

/// Block layout context
pub struct BlockLayoutContext {
    /// Available width for blocks
    pub available_width: Length,

    /// Current Y position
    pub current_y: Length,

    /// Maximum width seen
    pub max_width: Length,
}

impl BlockLayoutContext {
    /// Create a new block layout context
    pub fn new(available_width: Length) -> Self {
        Self {
            available_width,
            current_y: Length::ZERO,
            max_width: Length::ZERO,
        }
    }

    /// Allocate space for a block with optional space-before/space-after
    pub fn allocate(&mut self, width: Length, height: Length) -> Rect {
        let rect = Rect::from_point_size(
            Point::new(Length::ZERO, self.current_y),
            Size::new(width, height),
        );

        self.current_y += height;
        if width > self.max_width {
            self.max_width = width;
        }

        rect
    }

    /// Allocate space with space-before and space-after
    pub fn allocate_with_spacing(
        &mut self,
        width: Length,
        height: Length,
        space_before: Length,
        space_after: Length,
    ) -> Rect {
        // Add space before
        self.current_y += space_before;

        let rect = Rect::from_point_size(
            Point::new(Length::ZERO, self.current_y),
            Size::new(width, height),
        );

        self.current_y += height;
        self.current_y += space_after;

        if width > self.max_width {
            self.max_width = width;
        }

        rect
    }

    /// Add vertical space (for explicit spacing)
    pub fn add_space(&mut self, space: Length) {
        self.current_y += space;
    }

    /// Get the total height allocated
    pub fn total_height(&self) -> Length {
        self.current_y
    }
}

/// Stack blocks vertically
pub fn stack_blocks(
    blocks: Vec<(Length, Length)>, // (width, height) pairs
    available_width: Length,
) -> Result<Vec<Rect>> {
    let mut context = BlockLayoutContext::new(available_width);
    let mut rects = Vec::new();

    for (width, height) in blocks {
        let rect = context.allocate(width, height);
        rects.push(rect);
    }

    Ok(rects)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── BlockLayoutContext construction ──────────────────────────────────────

    #[test]
    fn test_block_context_new_initial_state() {
        let ctx = BlockLayoutContext::new(Length::from_pt(200.0));
        assert_eq!(ctx.available_width, Length::from_pt(200.0));
        assert_eq!(ctx.current_y, Length::ZERO);
        assert_eq!(ctx.max_width, Length::ZERO);
    }

    #[test]
    fn test_block_context_zero_width() {
        let ctx = BlockLayoutContext::new(Length::ZERO);
        assert_eq!(ctx.available_width, Length::ZERO);
        assert_eq!(ctx.current_y, Length::ZERO);
        assert_eq!(ctx.max_width, Length::ZERO);
    }

    // ── allocate: position and size ─────────────────────────────────────────

    #[test]
    fn test_allocate_first_block_starts_at_origin() {
        let mut ctx = BlockLayoutContext::new(Length::from_pt(100.0));
        let rect = ctx.allocate(Length::from_pt(80.0), Length::from_pt(25.0));
        assert_eq!(rect.x, Length::ZERO);
        assert_eq!(rect.y, Length::ZERO);
        assert_eq!(rect.width, Length::from_pt(80.0));
        assert_eq!(rect.height, Length::from_pt(25.0));
    }

    #[test]
    fn test_allocate_advances_current_y() {
        let mut ctx = BlockLayoutContext::new(Length::from_pt(100.0));
        ctx.allocate(Length::from_pt(100.0), Length::from_pt(20.0));
        assert_eq!(ctx.current_y, Length::from_pt(20.0));
    }

    #[test]
    fn test_allocate_second_block_stacks_below_first() {
        let mut ctx = BlockLayoutContext::new(Length::from_pt(100.0));
        let _r1 = ctx.allocate(Length::from_pt(100.0), Length::from_pt(20.0));
        let r2 = ctx.allocate(Length::from_pt(100.0), Length::from_pt(30.0));
        assert_eq!(r2.y, Length::from_pt(20.0));
        assert_eq!(r2.height, Length::from_pt(30.0));
    }

    #[test]
    fn test_allocate_total_height_accumulates() {
        let mut ctx = BlockLayoutContext::new(Length::from_pt(100.0));
        ctx.allocate(Length::from_pt(100.0), Length::from_pt(10.0));
        ctx.allocate(Length::from_pt(100.0), Length::from_pt(20.0));
        ctx.allocate(Length::from_pt(100.0), Length::from_pt(30.0));
        assert_eq!(ctx.total_height(), Length::from_pt(60.0));
    }

    // ── max_width tracking ───────────────────────────────────────────────────

    #[test]
    fn test_max_width_updated_on_wider_block() {
        let mut ctx = BlockLayoutContext::new(Length::from_pt(200.0));
        ctx.allocate(Length::from_pt(80.0), Length::from_pt(10.0));
        assert_eq!(ctx.max_width, Length::from_pt(80.0));
        ctx.allocate(Length::from_pt(120.0), Length::from_pt(10.0));
        assert_eq!(ctx.max_width, Length::from_pt(120.0));
    }

    #[test]
    fn test_max_width_not_reduced_by_narrower_block() {
        let mut ctx = BlockLayoutContext::new(Length::from_pt(200.0));
        ctx.allocate(Length::from_pt(120.0), Length::from_pt(10.0));
        ctx.allocate(Length::from_pt(60.0), Length::from_pt(10.0));
        assert_eq!(ctx.max_width, Length::from_pt(120.0));
    }

    #[test]
    fn test_max_width_equal_block_does_not_change() {
        let mut ctx = BlockLayoutContext::new(Length::from_pt(200.0));
        ctx.allocate(Length::from_pt(100.0), Length::from_pt(10.0));
        ctx.allocate(Length::from_pt(100.0), Length::from_pt(10.0));
        assert_eq!(ctx.max_width, Length::from_pt(100.0));
    }

    // ── allocate_with_spacing: space-before / space-after ───────────────────

    #[test]
    fn test_spacing_space_before_offsets_rect_y() {
        let mut ctx = BlockLayoutContext::new(Length::from_pt(100.0));
        let rect = ctx.allocate_with_spacing(
            Length::from_pt(100.0),
            Length::from_pt(20.0),
            Length::from_pt(10.0), // space-before
            Length::ZERO,
        );
        assert_eq!(rect.y, Length::from_pt(10.0));
    }

    #[test]
    fn test_spacing_space_after_advances_current_y() {
        let mut ctx = BlockLayoutContext::new(Length::from_pt(100.0));
        ctx.allocate_with_spacing(
            Length::from_pt(100.0),
            Length::from_pt(20.0),
            Length::ZERO,
            Length::from_pt(5.0), // space-after
        );
        // 0 (space-before) + 20 (height) + 5 (space-after) = 25
        assert_eq!(ctx.current_y, Length::from_pt(25.0));
    }

    #[test]
    fn test_spacing_combined_space_before_and_after() {
        let mut ctx = BlockLayoutContext::new(Length::from_pt(100.0));
        let rect = ctx.allocate_with_spacing(
            Length::from_pt(100.0),
            Length::from_pt(20.0),
            Length::from_pt(10.0),
            Length::from_pt(5.0),
        );
        // rect starts at space-before = 10
        assert_eq!(rect.y, Length::from_pt(10.0));
        assert_eq!(rect.height, Length::from_pt(20.0));
        // current_y = 10 + 20 + 5 = 35
        assert_eq!(ctx.current_y, Length::from_pt(35.0));
        assert_eq!(ctx.total_height(), Length::from_pt(35.0));
    }

    #[test]
    fn test_spacing_second_block_stacks_correctly() {
        let mut ctx = BlockLayoutContext::new(Length::from_pt(100.0));
        ctx.allocate_with_spacing(
            Length::from_pt(100.0),
            Length::from_pt(20.0),
            Length::from_pt(10.0),
            Length::from_pt(5.0),
        );
        // current_y = 35 after first block
        let rect2 = ctx.allocate_with_spacing(
            Length::from_pt(100.0),
            Length::from_pt(15.0),
            Length::from_pt(8.0),
            Length::from_pt(12.0),
        );
        // rect2.y = 35 + 8 = 43
        assert_eq!(rect2.y, Length::from_pt(43.0));
        // current_y = 43 + 15 + 12 = 70
        assert_eq!(ctx.total_height(), Length::from_pt(70.0));
    }

    #[test]
    fn test_spacing_zero_behaves_like_plain_allocate() {
        let mut ctx_plain = BlockLayoutContext::new(Length::from_pt(100.0));
        let mut ctx_spaced = BlockLayoutContext::new(Length::from_pt(100.0));

        let r1 = ctx_plain.allocate(Length::from_pt(80.0), Length::from_pt(20.0));
        let r2 = ctx_spaced.allocate_with_spacing(
            Length::from_pt(80.0),
            Length::from_pt(20.0),
            Length::ZERO,
            Length::ZERO,
        );

        assert_eq!(r1.x, r2.x);
        assert_eq!(r1.y, r2.y);
        assert_eq!(r1.width, r2.width);
        assert_eq!(r1.height, r2.height);
        assert_eq!(ctx_plain.total_height(), ctx_spaced.total_height());
    }

    // ── add_space: explicit spacing ──────────────────────────────────────────

    #[test]
    fn test_add_space_increases_current_y() {
        let mut ctx = BlockLayoutContext::new(Length::from_pt(100.0));
        ctx.add_space(Length::from_pt(15.0));
        assert_eq!(ctx.current_y, Length::from_pt(15.0));
        assert_eq!(ctx.total_height(), Length::from_pt(15.0));
    }

    #[test]
    fn test_add_space_cumulative() {
        let mut ctx = BlockLayoutContext::new(Length::from_pt(100.0));
        ctx.add_space(Length::from_pt(10.0));
        ctx.add_space(Length::from_pt(5.0));
        assert_eq!(ctx.total_height(), Length::from_pt(15.0));
    }

    #[test]
    fn test_add_space_after_block() {
        let mut ctx = BlockLayoutContext::new(Length::from_pt(100.0));
        ctx.allocate(Length::from_pt(100.0), Length::from_pt(20.0));
        ctx.add_space(Length::from_pt(10.0));
        assert_eq!(ctx.total_height(), Length::from_pt(30.0));
    }

    // ── overflow detection ───────────────────────────────────────────────────

    #[test]
    fn test_overflow_detected_when_content_exceeds_page_height() {
        let page_height = Length::from_pt(100.0);
        let mut ctx = BlockLayoutContext::new(Length::from_pt(100.0));
        ctx.allocate(Length::from_pt(100.0), Length::from_pt(60.0));
        ctx.allocate(Length::from_pt(100.0), Length::from_pt(50.0));
        // Total = 110 > 100 (page height)
        assert!(ctx.total_height() > page_height);
    }

    #[test]
    fn test_no_overflow_when_content_fits_exactly() {
        let page_height = Length::from_pt(100.0);
        let mut ctx = BlockLayoutContext::new(Length::from_pt(100.0));
        ctx.allocate(Length::from_pt(100.0), Length::from_pt(40.0));
        ctx.allocate(Length::from_pt(100.0), Length::from_pt(60.0));
        // Total = 100 exactly fits
        assert!(ctx.total_height() <= page_height);
    }

    // ── stack_blocks convenience function ───────────────────────────────────

    #[test]
    fn test_stack_blocks_empty_input() {
        let rects = stack_blocks(vec![], Length::from_pt(100.0)).expect("test: should succeed");
        assert!(rects.is_empty());
    }

    #[test]
    fn test_stack_blocks_single_block() {
        let blocks = vec![(Length::from_pt(80.0), Length::from_pt(30.0))];
        let rects = stack_blocks(blocks, Length::from_pt(100.0)).expect("test: should succeed");
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].x, Length::ZERO);
        assert_eq!(rects[0].y, Length::ZERO);
        assert_eq!(rects[0].width, Length::from_pt(80.0));
        assert_eq!(rects[0].height, Length::from_pt(30.0));
    }

    #[test]
    fn test_stack_blocks_three_blocks() {
        let blocks = vec![
            (Length::from_pt(100.0), Length::from_pt(20.0)),
            (Length::from_pt(100.0), Length::from_pt(30.0)),
            (Length::from_pt(100.0), Length::from_pt(25.0)),
        ];
        let rects = stack_blocks(blocks, Length::from_pt(100.0)).expect("test: should succeed");
        assert_eq!(rects.len(), 3);
        assert_eq!(rects[0].y, Length::ZERO);
        assert_eq!(rects[1].y, Length::from_pt(20.0));
        assert_eq!(rects[2].y, Length::from_pt(50.0));
    }

    #[test]
    fn test_stack_blocks_x_always_zero() {
        let blocks = vec![
            (Length::from_pt(50.0), Length::from_pt(10.0)),
            (Length::from_pt(70.0), Length::from_pt(10.0)),
        ];
        let rects = stack_blocks(blocks, Length::from_pt(100.0)).expect("test: should succeed");
        for rect in &rects {
            assert_eq!(rect.x, Length::ZERO);
        }
    }

    // ── padding/margin box vs content box ───────────────────────────────────

    #[test]
    fn test_padding_box_wider_than_content_box() {
        // content width = 80, padding left = 10, padding right = 10 -> padding box = 100
        let content_width = Length::from_pt(80.0);
        let padding_left = Length::from_pt(10.0);
        let padding_right = Length::from_pt(10.0);
        let padding_box_width = content_width + padding_left + padding_right;
        assert_eq!(padding_box_width, Length::from_pt(100.0));

        let mut ctx = BlockLayoutContext::new(Length::from_pt(100.0));
        let rect = ctx.allocate(padding_box_width, Length::from_pt(20.0));
        assert_eq!(rect.width, Length::from_pt(100.0));
    }

    #[test]
    fn test_block_height_from_border_and_content() {
        // content height = 20, border-top = 2, border-bottom = 2
        let border_top = Length::from_pt(2.0);
        let border_bottom = Length::from_pt(2.0);
        let content_height = Length::from_pt(20.0);
        let margin_top = Length::from_pt(5.0);
        let margin_bottom = Length::from_pt(5.0);

        let mut ctx = BlockLayoutContext::new(Length::from_pt(100.0));
        let rect = ctx.allocate_with_spacing(
            Length::from_pt(100.0),
            border_top + content_height + border_bottom,
            margin_top,
            margin_bottom,
        );
        // rect.y = margin_top = 5
        assert_eq!(rect.y, Length::from_pt(5.0));
        // rect.height = 2 + 20 + 2 = 24
        assert_eq!(rect.height, Length::from_pt(24.0));
        // total = 5 + 24 + 5 = 34
        assert_eq!(ctx.total_height(), Length::from_pt(34.0));
    }

    // ── background bounds ────────────────────────────────────────────────────

    #[test]
    fn test_background_area_bounds_match_allocated_rect() {
        // Background covers the padding box (content + padding, not margins)
        let padding_w = Length::from_pt(100.0);
        let padding_h = Length::from_pt(30.0);
        let mut ctx = BlockLayoutContext::new(Length::from_pt(120.0));
        let rect = ctx.allocate(padding_w, padding_h);
        // The background rect should match the allocated rect (padding box)
        assert_eq!(rect.width, padding_w);
        assert_eq!(rect.height, padding_h);
        assert_eq!(rect.x, Length::ZERO);
        assert_eq!(rect.y, Length::ZERO);
    }

    // ── keep-together / keep-with-next logic ────────────────────────────────

    #[test]
    fn test_keep_constraint_keep_together_active() {
        use crate::layout::{Keep, KeepConstraint};
        let mut kc = KeepConstraint::new();
        kc.keep_together = Keep::Always;
        assert!(kc.must_keep_together());
        assert!(kc.has_constraint());
    }

    #[test]
    fn test_keep_constraint_keep_with_next_active() {
        use crate::layout::{Keep, KeepConstraint};
        let mut kc = KeepConstraint::new();
        kc.keep_with_next = Keep::Always;
        assert!(kc.must_keep_with_next());
        assert!(kc.has_constraint());
    }

    #[test]
    fn test_keep_constraint_default_all_inactive() {
        use crate::layout::KeepConstraint;
        let kc = KeepConstraint::new();
        assert!(!kc.has_constraint());
        assert!(!kc.must_keep_together());
        assert!(!kc.must_keep_with_next());
        assert!(!kc.must_keep_with_previous());
    }

    #[test]
    fn test_keep_constraint_keep_with_previous_active() {
        use crate::layout::{Keep, KeepConstraint};
        let mut kc = KeepConstraint::new();
        kc.keep_with_previous = Keep::Always;
        assert!(kc.must_keep_with_previous());
        assert!(kc.has_constraint());
    }

    #[test]
    fn test_keep_integer_strength_ordering() {
        use crate::layout::Keep;
        let weak = Keep::Integer(3);
        let strong = Keep::Integer(7);
        assert!(weak.is_active());
        assert!(strong.is_active());
        assert!(strong.strength() > weak.strength());
    }

    // ── break handling ───────────────────────────────────────────────────────

    #[test]
    fn test_break_value_page_is_page_break() {
        use crate::layout::BreakValue;
        assert!(BreakValue::Page.forces_page_break());
        assert!(BreakValue::Page.forces_break());
    }

    #[test]
    fn test_break_value_auto_is_not_active() {
        use crate::layout::BreakValue;
        assert!(!BreakValue::Auto.forces_break());
        assert!(!BreakValue::Auto.forces_page_break());
    }

    #[test]
    fn test_break_value_column_active_not_page() {
        use crate::layout::BreakValue;
        assert!(BreakValue::Column.forces_break());
        assert!(!BreakValue::Column.forces_page_break());
    }

    #[test]
    fn test_break_value_even_page_is_page_break() {
        use crate::layout::BreakValue;
        assert!(BreakValue::EvenPage.forces_page_break());
        assert!(BreakValue::EvenPage.requires_even_page());
    }

    #[test]
    fn test_break_value_odd_page_is_page_break() {
        use crate::layout::BreakValue;
        assert!(BreakValue::OddPage.forces_page_break());
        assert!(BreakValue::OddPage.requires_odd_page());
    }
}
