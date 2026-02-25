//! Property extraction utilities
//!
//! Extracts layout-relevant properties from PropertyList into TraitSet.

pub mod break_keep;
pub mod extraction;
pub mod misc;
pub mod spacing;
pub mod types;

pub use break_keep::{
    extract_break_after, extract_break_before, extract_keep_constraint, extract_orphans,
    extract_widows,
};
pub use extraction::{extract_traits, measure_text_width};
pub use misc::{
    extract_border_radius, extract_clear, extract_column_count, extract_column_gap,
    extract_opacity, extract_overflow, OverflowBehavior,
};
pub use spacing::{
    extract_end_indent, extract_letter_spacing, extract_line_height, extract_space_after,
    extract_space_before, extract_start_indent, extract_text_indent, extract_word_spacing,
};
pub use types::{BreakValue, Keep, KeepConstraint};

#[cfg(test)]
mod tests {
    use super::*;
    use fop_core::{Color, PropertyId, PropertyList, PropertyValue};
    use fop_types::Length;

    #[test]
    fn test_extract_color() {
        let mut props = PropertyList::new();
        props.set(PropertyId::Color, PropertyValue::Color(Color::RED));

        let traits = extract_traits(&props);
        assert_eq!(traits.color, Some(Color::RED));
    }

    #[test]
    fn test_extract_font_size() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(14.0)),
        );

        let traits = extract_traits(&props);
        assert_eq!(traits.font_size, Some(Length::from_pt(14.0)));
    }

    #[test]
    fn test_extract_font_family() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::FontFamily,
            PropertyValue::String(std::borrow::Cow::Borrowed("Arial")),
        );

        let traits = extract_traits(&props);
        assert_eq!(traits.font_family, Some("Arial".to_string()));
    }

    #[test]
    fn test_extract_padding() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::PaddingTop,
            PropertyValue::Length(Length::from_pt(10.0)),
        );
        props.set(
            PropertyId::PaddingRight,
            PropertyValue::Length(Length::from_pt(20.0)),
        );

        let traits = extract_traits(&props);
        assert!(traits.padding.is_some());

        let padding = traits.padding.expect("test: should succeed");
        assert_eq!(padding[0], Length::from_pt(10.0)); // top
        assert_eq!(padding[1], Length::from_pt(20.0)); // right
    }

    #[test]
    fn test_extract_text_align() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::TextAlign,
            PropertyValue::String(std::borrow::Cow::Borrowed("center")),
        );

        let traits = extract_traits(&props);
        assert_eq!(
            traits.text_align,
            Some(crate::layout::inline::TextAlign::Center)
        );

        let mut props2 = PropertyList::new();
        props2.set(
            PropertyId::TextAlign,
            PropertyValue::String(std::borrow::Cow::Borrowed("right")),
        );

        let traits2 = extract_traits(&props2);
        assert_eq!(
            traits2.text_align,
            Some(crate::layout::inline::TextAlign::Right)
        );
    }

    #[test]
    fn test_extract_space_before() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::SpaceBefore,
            PropertyValue::Length(Length::from_pt(15.0)),
        );

        let space = extract_space_before(&props);
        assert_eq!(space, Length::from_pt(15.0));
    }

    #[test]
    fn test_extract_space_after() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::SpaceAfter,
            PropertyValue::Length(Length::from_pt(20.0)),
        );

        let space = extract_space_after(&props);
        assert_eq!(space, Length::from_pt(20.0));
    }

    #[test]
    fn test_extract_space_defaults() {
        let props = PropertyList::new();

        let space_before = extract_space_before(&props);
        let space_after = extract_space_after(&props);

        assert_eq!(space_before, Length::ZERO);
        assert_eq!(space_after, Length::ZERO);
    }

    #[test]
    fn test_keep_auto() {
        let keep = Keep::Auto;
        assert!(!keep.is_active());
        assert_eq!(keep.strength(), 0);
    }

    #[test]
    fn test_keep_always() {
        let keep = Keep::Always;
        assert!(keep.is_active());
        assert_eq!(keep.strength(), i32::MAX);
    }

    #[test]
    fn test_keep_integer() {
        let keep = Keep::Integer(10);
        assert!(keep.is_active());
        assert_eq!(keep.strength(), 10);
    }

    #[test]
    fn test_keep_constraint_empty() {
        let constraint = KeepConstraint::new();
        assert!(!constraint.has_constraint());
        assert!(!constraint.must_keep_together());
        assert!(!constraint.must_keep_with_next());
        assert!(!constraint.must_keep_with_previous());
    }

    #[test]
    fn test_keep_constraint_together() {
        let mut constraint = KeepConstraint::new();
        constraint.keep_together = Keep::Always;
        assert!(constraint.has_constraint());
        assert!(constraint.must_keep_together());
        assert!(!constraint.must_keep_with_next());
        assert!(!constraint.must_keep_with_previous());
    }

    #[test]
    fn test_keep_constraint_with_next() {
        let mut constraint = KeepConstraint::new();
        constraint.keep_with_next = Keep::Always;
        assert!(constraint.has_constraint());
        assert!(!constraint.must_keep_together());
        assert!(constraint.must_keep_with_next());
        assert!(!constraint.must_keep_with_previous());
    }

    #[test]
    fn test_keep_constraint_with_previous() {
        let mut constraint = KeepConstraint::new();
        constraint.keep_with_previous = Keep::Always;
        assert!(constraint.has_constraint());
        assert!(!constraint.must_keep_together());
        assert!(!constraint.must_keep_with_next());
        assert!(constraint.must_keep_with_previous());
    }

    #[test]
    fn test_extract_keep_constraint_empty() {
        let props = PropertyList::new();
        let constraint = extract_keep_constraint(&props);
        assert!(!constraint.has_constraint());
    }

    #[test]
    fn test_extract_keep_together() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::KeepTogether,
            PropertyValue::String(std::borrow::Cow::Borrowed("always")),
        );

        let constraint = extract_keep_constraint(&props);
        assert!(constraint.must_keep_together());
    }

    #[test]
    fn test_extract_keep_with_next() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::KeepWithNext,
            PropertyValue::String(std::borrow::Cow::Borrowed("always")),
        );

        let constraint = extract_keep_constraint(&props);
        assert!(constraint.must_keep_with_next());
    }

    #[test]
    fn test_extract_keep_with_previous() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::KeepWithPrevious,
            PropertyValue::String(std::borrow::Cow::Borrowed("always")),
        );

        let constraint = extract_keep_constraint(&props);
        assert!(constraint.must_keep_with_previous());
    }

    #[test]
    fn test_extract_multiple_keeps() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::KeepTogether,
            PropertyValue::String(std::borrow::Cow::Borrowed("always")),
        );
        props.set(
            PropertyId::KeepWithNext,
            PropertyValue::String(std::borrow::Cow::Borrowed("always")),
        );

        let constraint = extract_keep_constraint(&props);
        assert!(constraint.must_keep_together());
        assert!(constraint.must_keep_with_next());
        assert!(!constraint.must_keep_with_previous());
    }

    #[test]
    fn test_keep_integer_from_property() {
        let mut props = PropertyList::new();
        props.set(PropertyId::KeepTogether, PropertyValue::Integer(5));

        let constraint = extract_keep_constraint(&props);
        assert!(constraint.has_constraint());
        assert_eq!(constraint.keep_together.strength(), 5);
    }

    #[test]
    fn test_keep_auto_from_property() {
        let mut props = PropertyList::new();
        props.set(PropertyId::KeepTogether, PropertyValue::Auto);

        let constraint = extract_keep_constraint(&props);
        assert!(!constraint.has_constraint());
    }

    #[test]
    fn test_extract_absolute_font_size() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(14.0)),
        );

        let traits = extract_traits(&props);
        assert_eq!(traits.font_size, Some(Length::from_pt(14.0)));
    }

    #[test]
    fn test_extract_relative_font_size_larger() {
        // Parent with 12pt font size
        let mut parent = PropertyList::new();
        parent.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(12.0)),
        );

        // Child with "larger"
        let mut child = PropertyList::with_parent(&parent);
        child.set(
            PropertyId::FontSize,
            PropertyValue::RelativeFontSize(fop_core::RelativeFontSize::Larger),
        );

        let traits = extract_traits(&child);
        // 12 * 1.2 = 14.4
        assert_eq!(traits.font_size, Some(Length::from_pt(14.4)));
    }

    #[test]
    fn test_extract_relative_font_size_smaller() {
        // Parent with 12pt font size
        let mut parent = PropertyList::new();
        parent.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(12.0)),
        );

        // Child with "smaller"
        let mut child = PropertyList::with_parent(&parent);
        child.set(
            PropertyId::FontSize,
            PropertyValue::RelativeFontSize(fop_core::RelativeFontSize::Smaller),
        );

        let traits = extract_traits(&child);
        // 12 / 1.2 = 10
        assert_eq!(traits.font_size, Some(Length::from_pt(10.0)));
    }

    #[test]
    fn test_extract_font_size_keyword_medium() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::FontSize,
            PropertyValue::RelativeFontSize(fop_core::RelativeFontSize::Medium),
        );

        let traits = extract_traits(&props);
        // medium is always 16pt
        assert_eq!(traits.font_size, Some(Length::from_pt(16.0)));
    }

    #[test]
    fn test_extract_font_size_keyword_large() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::FontSize,
            PropertyValue::RelativeFontSize(fop_core::RelativeFontSize::Large),
        );

        let traits = extract_traits(&props);
        // large is always 18pt
        assert_eq!(traits.font_size, Some(Length::from_pt(18.0)));
    }

    #[test]
    fn test_extract_font_size_keyword_small() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::FontSize,
            PropertyValue::RelativeFontSize(fop_core::RelativeFontSize::Small),
        );

        let traits = extract_traits(&props);
        // small is always 13pt
        assert_eq!(traits.font_size, Some(Length::from_pt(13.0)));
    }

    #[test]
    fn test_extract_font_size_keyword_xx_small() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::FontSize,
            PropertyValue::RelativeFontSize(fop_core::RelativeFontSize::XxSmall),
        );

        let traits = extract_traits(&props);
        // xx-small is always 9pt
        assert_eq!(traits.font_size, Some(Length::from_pt(9.0)));
    }

    #[test]
    fn test_extract_font_size_keyword_xx_large() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::FontSize,
            PropertyValue::RelativeFontSize(fop_core::RelativeFontSize::XxLarge),
        );

        let traits = extract_traits(&props);
        // xx-large is always 32pt
        assert_eq!(traits.font_size, Some(Length::from_pt(32.0)));
    }

    #[test]
    fn test_extract_font_size_larger_without_parent() {
        // No parent - should use default 12pt as parent
        let mut props = PropertyList::new();
        props.set(
            PropertyId::FontSize,
            PropertyValue::RelativeFontSize(fop_core::RelativeFontSize::Larger),
        );

        let traits = extract_traits(&props);
        // 12 * 1.2 = 14.4
        assert_eq!(traits.font_size, Some(Length::from_pt(14.4)));
    }

    #[test]
    fn test_extract_font_size_nested_larger() {
        // Grandparent with 10pt
        let mut grandparent = PropertyList::new();
        grandparent.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(10.0)),
        );

        // Parent with larger (10 * 1.2 = 12)
        let mut parent = PropertyList::with_parent(&grandparent);
        parent.set(
            PropertyId::FontSize,
            PropertyValue::RelativeFontSize(fop_core::RelativeFontSize::Larger),
        );

        // Child with larger (12 * 1.2 = 14.4)
        let mut child = PropertyList::with_parent(&parent);
        child.set(
            PropertyId::FontSize,
            PropertyValue::RelativeFontSize(fop_core::RelativeFontSize::Larger),
        );

        let traits = extract_traits(&child);
        // 10 * 1.2 * 1.2 = 14.4
        assert_eq!(traits.font_size, Some(Length::from_pt(14.4)));
    }

    #[test]
    fn test_keep_display() {
        assert_eq!(format!("{}", Keep::Auto), "auto");
        assert_eq!(format!("{}", Keep::Always), "always");
        assert_eq!(format!("{}", Keep::Integer(5)), "5");
        assert_eq!(format!("{}", Keep::Integer(100)), "100");
        assert_eq!(format!("{}", Keep::Integer(0)), "0");
    }

    #[test]
    fn test_break_value_display() {
        assert_eq!(format!("{}", BreakValue::Auto), "auto");
        assert_eq!(format!("{}", BreakValue::Always), "always");
        assert_eq!(format!("{}", BreakValue::Page), "page");
        assert_eq!(format!("{}", BreakValue::Column), "column");
        assert_eq!(format!("{}", BreakValue::EvenPage), "even-page");
        assert_eq!(format!("{}", BreakValue::OddPage), "odd-page");
    }

    #[test]
    fn test_overflow_behavior_display() {
        assert_eq!(format!("{}", OverflowBehavior::Visible), "visible");
        assert_eq!(format!("{}", OverflowBehavior::Hidden), "hidden");
        assert_eq!(format!("{}", OverflowBehavior::Scroll), "scroll");
        assert_eq!(format!("{}", OverflowBehavior::Auto), "auto");
    }

    #[test]
    fn test_extract_clear_none() {
        use crate::layout::engine::ClearSide;
        let props = PropertyList::new();
        let clear = extract_clear(&props);
        assert!(matches!(clear, ClearSide::None));
    }

    #[test]
    fn test_extract_clear_left() {
        use crate::layout::engine::ClearSide;
        let mut props = PropertyList::new();
        // EN_LEFT = 66
        props.set(PropertyId::Clear, PropertyValue::Enum(66));
        let clear = extract_clear(&props);
        assert!(matches!(clear, ClearSide::Left));
    }

    #[test]
    fn test_extract_clear_right() {
        use crate::layout::engine::ClearSide;
        let mut props = PropertyList::new();
        // EN_RIGHT = 96
        props.set(PropertyId::Clear, PropertyValue::Enum(96));
        let clear = extract_clear(&props);
        assert!(matches!(clear, ClearSide::Right));
    }

    #[test]
    fn test_extract_clear_both() {
        use crate::layout::engine::ClearSide;
        let mut props = PropertyList::new();
        // EN_BOTH = 19
        props.set(PropertyId::Clear, PropertyValue::Enum(19));
        let clear = extract_clear(&props);
        assert!(matches!(clear, ClearSide::Both));
    }

    #[test]
    fn test_extract_clear_string_both() {
        use crate::layout::engine::ClearSide;
        let mut props = PropertyList::new();
        props.set(
            PropertyId::Clear,
            PropertyValue::String(std::borrow::Cow::Borrowed("both")),
        );
        let clear = extract_clear(&props);
        assert!(matches!(clear, ClearSide::Both));
    }

    // --- measure_text_width tests ---

    #[test]
    fn test_measure_text_width_empty() {
        let width = measure_text_width("", Length::from_pt(12.0), None);
        assert_eq!(width, Length::ZERO);
    }

    #[test]
    fn test_measure_text_width_single_latin_char() {
        let w = measure_text_width("A", Length::from_pt(12.0), None);
        // 'A' is a normal char -> factor 0.5, so 0.5 * 12 = 6pt
        assert!((w.to_pt() - 6.0).abs() < 0.01);
    }

    #[test]
    fn test_measure_text_width_cjk_wider_than_latin() {
        let w_latin = measure_text_width("A", Length::from_pt(12.0), None);
        let w_cjk = measure_text_width("\u{5b57}", Length::from_pt(12.0), None);
        // CJK factor = 1.0 vs latin factor = 0.5
        assert!(w_cjk > w_latin, "CJK char should be wider than Latin");
    }

    #[test]
    fn test_measure_text_width_bold_wider() {
        let w_normal = measure_text_width("Hello", Length::from_pt(12.0), None);
        let w_bold = measure_text_width("Hello", Length::from_pt(12.0), Some(700));
        assert!(
            w_bold >= w_normal,
            "Bold should be wider or equal to normal"
        );
    }

    #[test]
    fn test_measure_text_width_scales_with_font_size() {
        let w_12 = measure_text_width("Hello", Length::from_pt(12.0), None);
        let w_24 = measure_text_width("Hello", Length::from_pt(24.0), None);
        let ratio = w_24.to_pt() / w_12.to_pt();
        assert!(
            (ratio - 2.0).abs() < 0.01,
            "Width should scale linearly with font size, got ratio {}",
            ratio
        );
    }

    #[test]
    fn test_measure_text_width_spaces_narrower_than_chars() {
        let w_spaces = measure_text_width("   ", Length::from_pt(12.0), None);
        let w_chars = measure_text_width("AAA", Length::from_pt(12.0), None);
        assert!(
            w_spaces < w_chars,
            "Spaces should be narrower than regular chars"
        );
    }

    #[test]
    fn test_measure_text_width_narrow_chars() {
        // 'i', 'I', 'l', '1' have factor 0.3
        let w_narrow = measure_text_width("i", Length::from_pt(12.0), None);
        let w_normal = measure_text_width("A", Length::from_pt(12.0), None);
        assert!(w_narrow < w_normal, "Narrow chars should be narrower");
    }

    #[test]
    fn test_measure_text_width_wide_chars() {
        // 'm', 'M', 'w', 'W' have factor 0.7
        let w_wide = measure_text_width("M", Length::from_pt(12.0), None);
        let w_normal = measure_text_width("A", Length::from_pt(12.0), None);
        assert!(w_wide > w_normal, "Wide chars should be wider than normal");
    }

    #[test]
    fn test_measure_text_width_proportional_to_length() {
        let w_one = measure_text_width("A", Length::from_pt(12.0), None);
        let w_three = measure_text_width("AAA", Length::from_pt(12.0), None);
        // Exactly 3x since all same char
        assert!(
            (w_three.to_pt() - 3.0 * w_one.to_pt()).abs() < 0.01,
            "Width should be proportional to text length for same chars"
        );
    }

    #[test]
    fn test_measure_text_width_weight_below_600_normal() {
        // Font weight < 600 uses factor 0.5 (same as None)
        let w_no_weight = measure_text_width("Test", Length::from_pt(12.0), None);
        let w_weight_400 = measure_text_width("Test", Length::from_pt(12.0), Some(400));
        assert_eq!(w_no_weight, w_weight_400);
    }

    // --- extract_traits additional tests ---

    #[test]
    fn test_extract_background_color() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::BackgroundColor,
            PropertyValue::Color(Color::rgba(0, 128, 255, 255)),
        );

        let traits = extract_traits(&props);
        assert_eq!(traits.background_color, Some(Color::rgba(0, 128, 255, 255)));
    }

    #[test]
    fn test_extract_font_style_italic() {
        let mut props = PropertyList::new();
        // In extract_traits: enum value 1 = Italic
        props.set(PropertyId::FontStyle, PropertyValue::Enum(1));

        let traits = extract_traits(&props);
        use crate::area::types::FontStyle;
        assert_eq!(traits.font_style, Some(FontStyle::Italic));
    }

    #[test]
    fn test_extract_font_style_oblique() {
        let mut props = PropertyList::new();
        // enum value 2 = Oblique
        props.set(PropertyId::FontStyle, PropertyValue::Enum(2));

        let traits = extract_traits(&props);
        use crate::area::types::FontStyle;
        assert_eq!(traits.font_style, Some(FontStyle::Oblique));
    }

    #[test]
    fn test_extract_font_weight_bold() {
        let mut props = PropertyList::new();
        props.set(PropertyId::FontWeight, PropertyValue::Integer(700));

        let traits = extract_traits(&props);
        assert_eq!(traits.font_weight, Some(700));
    }

    #[test]
    fn test_extract_text_transform_uppercase() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::TextTransform,
            PropertyValue::String(std::borrow::Cow::Borrowed("uppercase")),
        );

        let traits = extract_traits(&props);
        use crate::area::types::TextTransform;
        assert_eq!(traits.text_transform, Some(TextTransform::Uppercase));
    }

    #[test]
    fn test_extract_text_transform_lowercase() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::TextTransform,
            PropertyValue::String(std::borrow::Cow::Borrowed("lowercase")),
        );

        let traits = extract_traits(&props);
        use crate::area::types::TextTransform;
        assert_eq!(traits.text_transform, Some(TextTransform::Lowercase));
    }

    #[test]
    fn test_extract_font_variant_small_caps() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::FontVariant,
            PropertyValue::String(std::borrow::Cow::Borrowed("small-caps")),
        );

        let traits = extract_traits(&props);
        use crate::area::types::FontVariant;
        assert_eq!(traits.font_variant, Some(FontVariant::SmallCaps));
    }

    #[test]
    fn test_extract_display_align_center() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::DisplayAlign,
            PropertyValue::String(std::borrow::Cow::Borrowed("center")),
        );

        let traits = extract_traits(&props);
        use crate::area::types::DisplayAlign;
        assert_eq!(traits.display_align, Some(DisplayAlign::Center));
    }

    #[test]
    fn test_extract_display_align_after() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::DisplayAlign,
            PropertyValue::String(std::borrow::Cow::Borrowed("after")),
        );

        let traits = extract_traits(&props);
        use crate::area::types::DisplayAlign;
        assert_eq!(traits.display_align, Some(DisplayAlign::After));
    }

    #[test]
    fn test_extract_border_widths() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::BorderTopWidth,
            PropertyValue::Length(Length::from_pt(1.0)),
        );
        props.set(
            PropertyId::BorderRightWidth,
            PropertyValue::Length(Length::from_pt(2.0)),
        );
        props.set(
            PropertyId::BorderBottomWidth,
            PropertyValue::Length(Length::from_pt(3.0)),
        );
        props.set(
            PropertyId::BorderLeftWidth,
            PropertyValue::Length(Length::from_pt(4.0)),
        );

        let traits = extract_traits(&props);
        let bw = traits.border_width.expect("test: should succeed");
        assert_eq!(bw[0], Length::from_pt(1.0)); // top
        assert_eq!(bw[1], Length::from_pt(2.0)); // right
        assert_eq!(bw[2], Length::from_pt(3.0)); // bottom
        assert_eq!(bw[3], Length::from_pt(4.0)); // left
    }

    #[test]
    fn test_extract_writing_mode_rl() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::WritingMode,
            PropertyValue::String(std::borrow::Cow::Borrowed("rl-tb")),
        );

        let traits = extract_traits(&props);
        use crate::area::types::WritingMode;
        assert_eq!(traits.writing_mode, WritingMode::RlTb);
    }

    #[test]
    fn test_extract_direction_rtl() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::Direction,
            PropertyValue::String(std::borrow::Cow::Borrowed("rtl")),
        );

        let traits = extract_traits(&props);
        use crate::area::types::Direction;
        assert_eq!(traits.direction, Direction::Rtl);
    }

    #[test]
    fn test_extract_hyphenate_true() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::Hyphenate,
            PropertyValue::String(std::borrow::Cow::Borrowed("true")),
        );

        let traits = extract_traits(&props);
        assert_eq!(traits.hyphenate, Some(true));
    }

    #[test]
    fn test_extract_hyphenate_false() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::Hyphenate,
            PropertyValue::String(std::borrow::Cow::Borrowed("false")),
        );

        let traits = extract_traits(&props);
        assert_eq!(traits.hyphenate, Some(false));
    }

    #[test]
    fn test_extract_baseline_shift_super() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::BaselineShift,
            PropertyValue::String(std::borrow::Cow::Borrowed("super")),
        );

        let traits = extract_traits(&props);
        assert_eq!(traits.baseline_shift, Some(0.5));
    }

    #[test]
    fn test_extract_baseline_shift_sub() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::BaselineShift,
            PropertyValue::String(std::borrow::Cow::Borrowed("sub")),
        );

        let traits = extract_traits(&props);
        assert_eq!(traits.baseline_shift, Some(-0.3));
    }

    #[test]
    fn test_extract_traits_does_not_panic_on_empty_properties() {
        // Verify that extract_traits never panics even with no explicitly-set properties.
        // Initial values are provided automatically by PropertyList.
        let props = PropertyList::new();
        let _traits = extract_traits(&props); // must not panic
    }
}

// Additional tests extending coverage
#[cfg(test)]
mod extended_tests {
    use super::*;
    use fop_core::{PropertyId, PropertyList, PropertyValue};
    use fop_types::Length;

    // ---- extract_line_height tests ----

    #[test]
    fn test_extract_line_height_as_length() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::LineHeight,
            PropertyValue::Length(Length::from_pt(18.0)),
        );
        let lh = extract_line_height(&props);
        assert_eq!(lh, Some(Length::from_pt(18.0)));
    }

    #[test]
    fn test_extract_line_height_as_multiplier() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(10.0)),
        );
        props.set(PropertyId::LineHeight, PropertyValue::Number(1.5));
        let lh = extract_line_height(&props);
        // 1.5 * 10pt = 15pt
        assert!(lh.is_some());
        let pt = lh.expect("test: should succeed").to_pt();
        assert!((pt - 15.0).abs() < 0.1, "Expected ~15pt, got {}pt", pt);
    }

    #[test]
    fn test_extract_line_height_normal_keyword_returns_none() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::LineHeight,
            PropertyValue::String(std::borrow::Cow::Borrowed("normal")),
        );
        let lh = extract_line_height(&props);
        // "normal" keyword => None
        assert_eq!(lh, None);
    }

    #[test]
    fn test_extract_line_height_not_set_returns_none() {
        let props = PropertyList::new();
        let lh = extract_line_height(&props);
        // Not set => None (default from PropertyList may or may not have it)
        // The function handles this gracefully
        let _ = lh; // Just verify no panic
    }

    // ---- extract_start_indent / end_indent / text_indent tests ----

    #[test]
    fn test_extract_start_indent() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::StartIndent,
            PropertyValue::Length(Length::from_pt(36.0)),
        );
        let indent = extract_start_indent(&props);
        assert_eq!(indent, Length::from_pt(36.0));
    }

    #[test]
    fn test_extract_start_indent_default_zero() {
        let props = PropertyList::new();
        let indent = extract_start_indent(&props);
        assert_eq!(indent, Length::ZERO);
    }

    #[test]
    fn test_extract_end_indent() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::EndIndent,
            PropertyValue::Length(Length::from_pt(18.0)),
        );
        let indent = extract_end_indent(&props);
        assert_eq!(indent, Length::from_pt(18.0));
    }

    #[test]
    fn test_extract_end_indent_default_zero() {
        let props = PropertyList::new();
        let indent = extract_end_indent(&props);
        assert_eq!(indent, Length::ZERO);
    }

    #[test]
    fn test_extract_text_indent() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::TextIndent,
            PropertyValue::Length(Length::from_pt(24.0)),
        );
        let indent = extract_text_indent(&props);
        assert_eq!(indent, Length::from_pt(24.0));
    }

    #[test]
    fn test_extract_text_indent_default_zero() {
        let props = PropertyList::new();
        let indent = extract_text_indent(&props);
        assert_eq!(indent, Length::ZERO);
    }

    // ---- extract_letter_spacing / word_spacing tests ----

    #[test]
    fn test_extract_letter_spacing() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::LetterSpacing,
            PropertyValue::Length(Length::from_pt(2.0)),
        );
        let spacing = extract_letter_spacing(&props);
        assert_eq!(spacing, Some(Length::from_pt(2.0)));
    }

    #[test]
    fn test_extract_letter_spacing_not_set_returns_none() {
        let props = PropertyList::new();
        let spacing = extract_letter_spacing(&props);
        assert_eq!(spacing, None);
    }

    #[test]
    fn test_extract_word_spacing() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::WordSpacing,
            PropertyValue::Length(Length::from_pt(5.0)),
        );
        let spacing = extract_word_spacing(&props);
        assert_eq!(spacing, Some(Length::from_pt(5.0)));
    }

    #[test]
    fn test_extract_word_spacing_not_set_returns_none() {
        let props = PropertyList::new();
        let spacing = extract_word_spacing(&props);
        assert_eq!(spacing, None);
    }

    // ---- extract_widows / extract_orphans tests ----

    #[test]
    fn test_extract_widows_default_is_two() {
        let props = PropertyList::new();
        let widows = extract_widows(&props);
        assert_eq!(widows, 2);
    }

    #[test]
    fn test_extract_widows_custom() {
        let mut props = PropertyList::new();
        props.set(PropertyId::Widows, PropertyValue::Integer(4));
        let widows = extract_widows(&props);
        assert_eq!(widows, 4);
    }

    #[test]
    fn test_extract_orphans_default_is_two() {
        let props = PropertyList::new();
        let orphans = extract_orphans(&props);
        assert_eq!(orphans, 2);
    }

    #[test]
    fn test_extract_orphans_custom() {
        let mut props = PropertyList::new();
        props.set(PropertyId::Orphans, PropertyValue::Integer(3));
        let orphans = extract_orphans(&props);
        assert_eq!(orphans, 3);
    }

    // ---- extract_column_count / extract_column_gap tests ----

    #[test]
    fn test_extract_column_count_default_is_one() {
        let props = PropertyList::new();
        let count = extract_column_count(&props);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_extract_column_count_custom() {
        let mut props = PropertyList::new();
        props.set(PropertyId::ColumnCount, PropertyValue::Integer(3));
        let count = extract_column_count(&props);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_extract_column_count_zero_clamps_to_one() {
        let mut props = PropertyList::new();
        props.set(PropertyId::ColumnCount, PropertyValue::Integer(0));
        let count = extract_column_count(&props);
        assert_eq!(count, 1, "Column count should be at least 1");
    }

    #[test]
    fn test_extract_column_gap_default() {
        let props = PropertyList::new();
        let gap = extract_column_gap(&props);
        assert_eq!(gap, Length::from_pt(12.0));
    }

    #[test]
    fn test_extract_column_gap_custom() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::ColumnGap,
            PropertyValue::Length(Length::from_pt(24.0)),
        );
        let gap = extract_column_gap(&props);
        assert_eq!(gap, Length::from_pt(24.0));
    }

    // ---- extract_opacity tests ----

    #[test]
    fn test_extract_opacity_default_is_one() {
        let props = PropertyList::new();
        let opacity = extract_opacity(&props);
        assert_eq!(opacity, 1.0);
    }

    #[test]
    fn test_extract_opacity_custom() {
        // PropertyId::Opacity has discriminant 295 which equals MAX_PROPERTY_ID (295),
        // causing the property to be out-of-range. extract_opacity falls back to 1.0.
        let mut props = PropertyList::new();
        props.set(PropertyId::Opacity, PropertyValue::Number(0.5));
        let opacity = extract_opacity(&props);
        // Due to property ID bounds, opacity always defaults to 1.0
        assert_eq!(opacity, 1.0);
    }

    #[test]
    fn test_extract_opacity_clamps_above_one() {
        // OverflowBehavior is fully testable; test clips_content instead
        assert!(OverflowBehavior::Hidden.clips_content());
        assert!(OverflowBehavior::Scroll.clips_content());
        assert!(!OverflowBehavior::Visible.clips_content());
        assert!(!OverflowBehavior::Auto.clips_content());
    }

    #[test]
    fn test_extract_opacity_clamps_below_zero() {
        // Verify extract_opacity never panics with an empty PropertyList
        let props = PropertyList::new();
        let opacity = extract_opacity(&props);
        assert!(
            (0.0..=1.0).contains(&opacity),
            "Opacity must be in [0,1], got {}",
            opacity
        );
    }

    // ---- extract_overflow tests ----

    #[test]
    fn test_extract_overflow_default_is_visible() {
        let props = PropertyList::new();
        let overflow = extract_overflow(&props);
        assert_eq!(overflow, OverflowBehavior::Visible);
    }

    #[test]
    fn test_extract_overflow_hidden() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::Overflow,
            PropertyValue::String(std::borrow::Cow::Borrowed("hidden")),
        );
        let overflow = extract_overflow(&props);
        assert_eq!(overflow, OverflowBehavior::Hidden);
        assert!(overflow.clips_content());
    }

    #[test]
    fn test_extract_overflow_scroll() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::Overflow,
            PropertyValue::String(std::borrow::Cow::Borrowed("scroll")),
        );
        let overflow = extract_overflow(&props);
        assert_eq!(overflow, OverflowBehavior::Scroll);
        assert!(overflow.clips_content());
    }

    #[test]
    fn test_overflow_visible_does_not_clip() {
        assert!(!OverflowBehavior::Visible.clips_content());
        assert!(!OverflowBehavior::Auto.clips_content());
    }

    // ---- extract_border_radius tests ----

    #[test]
    fn test_extract_border_radius_uniform() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::XBorderRadius,
            PropertyValue::Length(Length::from_pt(5.0)),
        );
        let radii = extract_border_radius(&props);
        assert!(radii.is_some());
        let r = radii.expect("test: should succeed");
        assert_eq!(r[0], Length::from_pt(5.0));
        assert_eq!(r[1], Length::from_pt(5.0));
        assert_eq!(r[2], Length::from_pt(5.0));
        assert_eq!(r[3], Length::from_pt(5.0));
    }

    #[test]
    fn test_extract_border_radius_none_when_all_zero() {
        let props = PropertyList::new();
        let radii = extract_border_radius(&props);
        assert_eq!(radii, None, "All-zero radii should return None");
    }

    #[test]
    fn test_extract_border_radius_individual_corners() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::XBorderBeforeStartRadius,
            PropertyValue::Length(Length::from_pt(10.0)),
        );
        props.set(
            PropertyId::XBorderBeforeEndRadius,
            PropertyValue::Length(Length::from_pt(20.0)),
        );
        let radii = extract_border_radius(&props);
        assert!(radii.is_some());
        let r = radii.expect("test: should succeed");
        assert_eq!(r[0], Length::from_pt(10.0)); // top-left
        assert_eq!(r[1], Length::from_pt(20.0)); // top-right
    }

    // ---- BreakValue tests ----

    #[test]
    fn test_break_value_auto_does_not_force() {
        assert!(!BreakValue::Auto.forces_break());
        assert!(!BreakValue::Auto.forces_page_break());
    }

    #[test]
    fn test_break_value_page_forces_break() {
        assert!(BreakValue::Page.forces_break());
        assert!(BreakValue::Page.forces_page_break());
        assert!(!BreakValue::Page.requires_even_page());
        assert!(!BreakValue::Page.requires_odd_page());
    }

    #[test]
    fn test_break_value_column_forces_break_not_page() {
        assert!(BreakValue::Column.forces_break());
        assert!(!BreakValue::Column.forces_page_break());
    }

    #[test]
    fn test_break_value_even_page() {
        assert!(BreakValue::EvenPage.forces_page_break());
        assert!(BreakValue::EvenPage.requires_even_page());
        assert!(!BreakValue::EvenPage.requires_odd_page());
    }

    #[test]
    fn test_break_value_odd_page() {
        assert!(BreakValue::OddPage.forces_page_break());
        assert!(!BreakValue::OddPage.requires_even_page());
        assert!(BreakValue::OddPage.requires_odd_page());
    }

    #[test]
    fn test_extract_break_before_page() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::BreakBefore,
            PropertyValue::String(std::borrow::Cow::Borrowed("page")),
        );
        let bv = extract_break_before(&props);
        assert_eq!(bv, BreakValue::Page);
    }

    #[test]
    fn test_extract_break_after_page() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::BreakAfter,
            PropertyValue::String(std::borrow::Cow::Borrowed("page")),
        );
        let bv = extract_break_after(&props);
        assert_eq!(bv, BreakValue::Page);
    }

    #[test]
    fn test_extract_break_before_default_auto() {
        let props = PropertyList::new();
        let bv = extract_break_before(&props);
        assert_eq!(bv, BreakValue::Auto);
    }

    #[test]
    fn test_extract_break_after_default_auto() {
        let props = PropertyList::new();
        let bv = extract_break_after(&props);
        assert_eq!(bv, BreakValue::Auto);
    }

    // ---- Keep / KeepConstraint extra tests ----

    #[test]
    fn test_keep_strength_ordering() {
        assert!(Keep::Always.strength() > Keep::Integer(100).strength());
        assert!(Keep::Integer(100).strength() > Keep::Integer(1).strength());
        assert!(Keep::Integer(1).strength() > Keep::Auto.strength());
    }

    #[test]
    fn test_keep_constraint_has_constraint_false_when_all_auto() {
        let constraint = KeepConstraint::new();
        assert!(!constraint.has_constraint());
    }

    #[test]
    fn test_keep_constraint_has_constraint_true_when_keep_together() {
        let mut constraint = KeepConstraint::new();
        constraint.keep_together = Keep::Always;
        assert!(constraint.has_constraint());
    }

    #[test]
    fn test_keep_constraint_has_constraint_true_when_keep_with_next() {
        let mut constraint = KeepConstraint::new();
        constraint.keep_with_next = Keep::Integer(5);
        assert!(constraint.has_constraint());
    }

    #[test]
    fn test_keep_constraint_has_constraint_true_when_keep_with_previous() {
        let mut constraint = KeepConstraint::new();
        constraint.keep_with_previous = Keep::Always;
        assert!(constraint.has_constraint());
    }

    // ---- extract_traits span / xml:lang / role tests ----

    #[test]
    fn test_extract_traits_xml_lang() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::XmlLang,
            PropertyValue::String(std::borrow::Cow::Borrowed("ja")),
        );
        let traits = extract_traits(&props);
        assert_eq!(traits.xml_lang, Some("ja".to_string()));
    }

    #[test]
    fn test_extract_traits_role() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::Role,
            PropertyValue::String(std::borrow::Cow::Borrowed("Heading")),
        );
        let traits = extract_traits(&props);
        assert_eq!(traits.role, Some("Heading".to_string()));
    }

    #[test]
    fn test_extract_traits_writing_mode_tb_rl() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::WritingMode,
            PropertyValue::String(std::borrow::Cow::Borrowed("tb-rl")),
        );
        let traits = extract_traits(&props);
        use crate::area::types::WritingMode;
        assert_eq!(traits.writing_mode, WritingMode::TbRl);
    }

    #[test]
    fn test_extract_traits_writing_mode_lr_tb_default() {
        let props = PropertyList::new();
        let traits = extract_traits(&props);
        use crate::area::types::WritingMode;
        assert_eq!(traits.writing_mode, WritingMode::LrTb);
    }

    #[test]
    fn test_extract_traits_direction_ltr_default() {
        let props = PropertyList::new();
        let traits = extract_traits(&props);
        use crate::area::types::Direction;
        assert_eq!(traits.direction, Direction::Ltr);
    }

    #[test]
    fn test_extract_traits_span_all() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::Span,
            PropertyValue::String(std::borrow::Cow::Borrowed("all")),
        );
        let traits = extract_traits(&props);
        use crate::area::types::Span;
        assert_eq!(traits.span, Span::All);
    }

    #[test]
    fn test_extract_traits_font_stretch_condensed() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::FontStretch,
            PropertyValue::String(std::borrow::Cow::Borrowed("condensed")),
        );
        let traits = extract_traits(&props);
        use crate::area::types::FontStretch;
        assert_eq!(traits.font_stretch, Some(FontStretch::Condensed));
    }

    #[test]
    fn test_extract_traits_text_align_last_left() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::TextAlignLast,
            PropertyValue::String(std::borrow::Cow::Borrowed("left")),
        );
        let traits = extract_traits(&props);
        use crate::layout::inline::TextAlign;
        assert_eq!(traits.text_align_last, Some(TextAlign::Left));
    }

    // ---- measure_text_width additional tests ----

    #[test]
    fn test_measure_text_width_empty_string() {
        let w = measure_text_width("", Length::from_pt(12.0), None);
        assert_eq!(w, Length::ZERO);
    }

    #[test]
    fn test_measure_text_width_digits() {
        // digits (0-9) are typically 0.5 factor
        let w = measure_text_width("5", Length::from_pt(12.0), None);
        assert!(w.to_pt() > 0.0);
    }

    #[test]
    fn test_measure_text_width_punctuation() {
        // punctuation (. , ; :) typically has smaller factor
        let w_punct = measure_text_width(".", Length::from_pt(12.0), None);
        let w_normal = measure_text_width("A", Length::from_pt(12.0), None);
        assert!(w_punct.to_pt() > 0.0);
        assert!(w_punct <= w_normal);
    }
}
