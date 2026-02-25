//! Layout algorithms
//!
//! Transforms the FO tree into an area tree through layout algorithms.

pub mod block;
pub mod engine;
pub mod inline;
pub mod knuth_plass;
pub mod list;
pub mod page_break;
pub mod page_number_resolver;
pub mod properties;
pub mod streaming;
pub mod table;

pub use block::BlockLayoutContext;
pub use engine::{LayoutEngine, MultiColumnLayout};
pub use inline::{InlineArea, InlineLayoutContext, LineBreaker, TextAlign};
pub use knuth_plass::KnuthPlassBreaker;
pub use list::{ListLayout, ListMarkerStyle};
pub use page_break::PageBreaker;
pub use page_number_resolver::PageNumberResolver;
pub use properties::{
    extract_break_after, extract_break_before, extract_clear, extract_column_count,
    extract_column_gap, extract_end_indent, extract_keep_constraint, extract_letter_spacing,
    extract_line_height, extract_opacity, extract_orphans, extract_overflow, extract_space_after,
    extract_space_before, extract_start_indent, extract_text_indent, extract_traits,
    extract_widows, extract_word_spacing, BreakValue, Keep, KeepConstraint, OverflowBehavior,
};
pub use streaming::{StreamingConfig, StreamingLayoutEngine, StreamingLayoutIterator};
pub use table::{
    BorderCollapse, CellCollapsedBorders, CollapsedBorder, ColumnInfo, ColumnWidth, TableLayout,
    TableLayoutMode,
};

#[cfg(test)]
mod tests {
    use super::*;
    use fop_types::{Length, Point, Rect, Size};

    fn make_rect(w: f64, h: f64) -> Rect {
        Rect::from_point_size(
            Point::ZERO,
            Size::new(Length::from_pt(w), Length::from_pt(h)),
        )
    }

    // ---- BlockLayoutContext tests ----

    #[test]
    fn test_block_layout_context_new() {
        let ctx = BlockLayoutContext::new(Length::from_pt(400.0));
        assert_eq!(ctx.total_height(), Length::ZERO);
    }

    #[test]
    fn test_block_layout_context_allocate() {
        let mut ctx = BlockLayoutContext::new(Length::from_pt(400.0));
        let rect = ctx.allocate(Length::from_pt(400.0), Length::from_pt(20.0));
        assert_eq!(rect.width, Length::from_pt(400.0));
        assert_eq!(rect.height, Length::from_pt(20.0));
        assert_eq!(ctx.total_height(), Length::from_pt(20.0));
    }

    #[test]
    fn test_block_layout_context_allocate_multiple() {
        let mut ctx = BlockLayoutContext::new(Length::from_pt(400.0));
        ctx.allocate(Length::from_pt(400.0), Length::from_pt(20.0));
        ctx.allocate(Length::from_pt(400.0), Length::from_pt(30.0));
        assert_eq!(ctx.total_height(), Length::from_pt(50.0));
    }

    #[test]
    fn test_block_layout_context_add_space() {
        let mut ctx = BlockLayoutContext::new(Length::from_pt(400.0));
        ctx.add_space(Length::from_pt(10.0));
        assert_eq!(ctx.total_height(), Length::from_pt(10.0));
    }

    #[test]
    fn test_block_layout_context_allocate_with_spacing() {
        let mut ctx = BlockLayoutContext::new(Length::from_pt(400.0));
        let rect = ctx.allocate_with_spacing(
            Length::from_pt(400.0),
            Length::from_pt(12.0),
            Length::from_pt(6.0),
            Length::from_pt(6.0),
        );
        // Width should match available width
        assert_eq!(rect.width, Length::from_pt(400.0));
    }

    // ---- TextAlign tests ----

    #[test]
    fn test_text_align_display_left() {
        let align = TextAlign::Left;
        let s = format!("{}", align);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_text_align_display_center() {
        let align = TextAlign::Center;
        let s = format!("{}", align);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_text_align_display_right() {
        let align = TextAlign::Right;
        let s = format!("{}", align);
        assert!(!s.is_empty());
    }

    // ---- InlineLayoutContext tests ----

    #[test]
    fn test_inline_layout_context_new() {
        let ctx = InlineLayoutContext::new(Length::from_pt(400.0), Length::from_pt(12.0));
        assert!(ctx.is_empty());
        assert_eq!(ctx.used_width(), Length::ZERO);
    }

    #[test]
    fn test_inline_layout_context_fits() {
        let ctx = InlineLayoutContext::new(Length::from_pt(400.0), Length::from_pt(12.0));
        assert!(ctx.fits(Length::from_pt(100.0)));
        assert!(!ctx.fits(Length::from_pt(500.0)));
    }

    #[test]
    fn test_inline_layout_context_remaining_width() {
        let ctx = InlineLayoutContext::new(Length::from_pt(400.0), Length::from_pt(12.0));
        assert_eq!(ctx.remaining_width(), Length::from_pt(400.0));
    }

    #[test]
    fn test_inline_layout_context_with_text_align() {
        let ctx = InlineLayoutContext::new(Length::from_pt(400.0), Length::from_pt(12.0))
            .with_text_align(TextAlign::Center);
        // Should not panic and should be constructable
        let _ = ctx;
    }

    // ---- LineBreaker tests ----

    #[test]
    fn test_line_breaker_new() {
        let breaker = LineBreaker::new(Length::from_pt(400.0));
        let _ = breaker;
    }

    #[test]
    fn test_line_breaker_break_into_words_empty() {
        let breaker = LineBreaker::new(Length::from_pt(400.0));
        let words = breaker.break_into_words("");
        assert!(words.is_empty());
    }

    #[test]
    fn test_line_breaker_break_into_words_single() {
        let breaker = LineBreaker::new(Length::from_pt(400.0));
        let words = breaker.break_into_words("hello");
        assert_eq!(words.len(), 1);
        assert_eq!(words[0], "hello");
    }

    #[test]
    fn test_line_breaker_break_into_words_multiple() {
        let breaker = LineBreaker::new(Length::from_pt(400.0));
        let words = breaker.break_into_words("hello world foo");
        assert_eq!(words.len(), 3);
    }

    #[test]
    fn test_line_breaker_measure_text() {
        let breaker = LineBreaker::new(Length::from_pt(400.0));
        let width = breaker.measure_text("hello", Length::from_pt(12.0));
        assert!(width > Length::ZERO);
    }

    #[test]
    fn test_line_breaker_break_lines() {
        let breaker = LineBreaker::new(Length::from_pt(400.0));
        let lines = breaker.break_lines("short text", Length::from_pt(12.0));
        assert!(!lines.is_empty());
    }

    // ---- PageNumberResolver tests ----

    #[test]
    fn test_page_number_resolver_new() {
        let resolver = PageNumberResolver::new();
        assert_eq!(resolver.current_page(), 1);
    }

    #[test]
    fn test_page_number_resolver_set_current_page() {
        let mut resolver = PageNumberResolver::new();
        resolver.set_current_page(5);
        assert_eq!(resolver.current_page(), 5);
    }

    #[test]
    fn test_page_number_resolver_register_element() {
        use crate::area::{Area, AreaTree, AreaType};
        let mut resolver = PageNumberResolver::new();
        let mut tree = AreaTree::new();
        let id = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));
        resolver.register_element("my-id".to_string(), id);
        // Should not panic and element should be registered
        let format = resolver.get_format_for_id("my-id");
        // format may or may not be set depending on implementation
        let _ = format;
    }

    #[test]
    fn test_page_number_resolver_format() {
        let mut resolver = PageNumberResolver::new();
        resolver.set_current_format("1".to_string());
        assert_eq!(resolver.current_format(), "1");
    }

    // ---- BreakValue tests ----

    #[test]
    fn test_break_value_forces_break_auto() {
        let bv = BreakValue::Auto;
        assert!(!bv.forces_break());
    }

    #[test]
    fn test_break_value_forces_break_page() {
        let bv = BreakValue::Page;
        assert!(bv.forces_break());
    }

    #[test]
    fn test_break_value_forces_page_break_column() {
        let bv = BreakValue::Column;
        assert!(!bv.forces_page_break());
    }

    // ---- StreamingLayoutEngine re-export tests ----

    #[test]
    fn test_streaming_config_re_exported() {
        let config = StreamingConfig::default();
        assert_eq!(config.max_memory_pages, 10);
    }

    #[test]
    fn test_streaming_layout_engine_re_exported() {
        let engine = StreamingLayoutEngine::new();
        let _ = engine;
    }
}
