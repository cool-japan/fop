//! Core trait extraction: extract_traits, font size resolution, text measurement

use crate::area::{
    BorderStyle, Direction, DisplayAlign, FontStretch, FontStyle, FontVariant, Span, TextTransform,
    TraitSet, WritingMode,
};
use crate::layout::TextAlign;
use fop_core::{PropertyId, PropertyList};
use fop_types::{FontRegistry, Length};

use super::misc::{extract_border_radius, extract_opacity, extract_overflow, OverflowBehavior};
use super::spacing::{extract_letter_spacing, extract_line_height, extract_word_spacing};

/// Resolve the computed font-size of the parent element.
///
/// Walks up the property list parent chain, recursively resolving any em
/// (`Percentage`) or relative-keyword values so that nested ems compound
/// correctly (e.g. `1.5em` inside a `1.5em`-of-10pt block yields 22.5 pt).
/// Falls back to 12 pt when no resolvable parent exists.
fn resolve_parent_font_size(properties: &PropertyList) -> Length {
    properties
        .parent()
        .and_then(|parent| {
            parent
                .get(PropertyId::FontSize)
                .ok()
                .and_then(|v| extract_font_size(parent, &v))
        })
        .unwrap_or(Length::from_pt(12.0))
}

/// Extract font-size from a property value, resolving relative sizes.
///
/// Resolution order:
/// 1. `PropertyValue::Length`  — returned directly.
/// 2. `PropertyValue::Percentage` — em values stored by the parser
///    (1 em → `Percentage::new(1.0)`).  Resolved as `pct.of(parent_font_size)`,
///    where `parent_font_size` is itself resolved recursively so nested ems
///    compound: a child element with `font-size="1.5em"` whose parent already
///    resolved to 15 pt gets `1.5 × 15 = 22.5 pt`.
/// 3. `PropertyValue::RelativeFontSize` — keyword sizes (`larger`, `smaller`,
///    `small`, `medium`, `large`, etc.) resolved via `resolve_font_size`.
/// 4. Any other variant returns `None`.
pub(super) fn extract_font_size(
    properties: &PropertyList,
    value: &fop_core::PropertyValue,
) -> Option<Length> {
    // 1. Direct absolute length — no parent lookup needed.
    if let Some(len) = value.as_length() {
        return Some(len);
    }

    // 2. em values: the parser stores `Xem` as `Percentage::new(X)` so that
    //    1 em = 100 % of parent.  Multiplying by the parent's *computed* size
    //    (obtained recursively) makes nested ems compound correctly.
    if let Some(pct) = value.as_percentage() {
        let parent_font_size = resolve_parent_font_size(properties);
        return Some(pct.of(parent_font_size));
    }

    // 3. Relative keyword sizes (larger, smaller, xx-small … xx-large).
    if value.as_relative_font_size().is_some() {
        let parent_font_size = resolve_parent_font_size(properties);
        return value.resolve_font_size(parent_font_size);
    }

    None
}

/// Parse border style from string value
pub(super) fn parse_border_style(s: Option<&str>) -> Option<BorderStyle> {
    match s? {
        "none" => Some(BorderStyle::None),
        "solid" => Some(BorderStyle::Solid),
        "dashed" => Some(BorderStyle::Dashed),
        "dotted" => Some(BorderStyle::Dotted),
        "double" => Some(BorderStyle::Double),
        "groove" => Some(BorderStyle::Groove),
        "ridge" => Some(BorderStyle::Ridge),
        "inset" => Some(BorderStyle::Inset),
        "outset" => Some(BorderStyle::Outset),
        "hidden" => Some(BorderStyle::Hidden),
        _ => None,
    }
}

/// Extract rendering traits from a property list
pub fn extract_traits(properties: &PropertyList) -> TraitSet {
    let mut traits = TraitSet::default();

    // Color
    if let Ok(value) = properties.get(PropertyId::Color) {
        traits.color = value.as_color();
    }

    // Background color
    if let Ok(value) = properties.get(PropertyId::BackgroundColor) {
        traits.background_color = value.as_color();
    }

    // Font family
    if let Ok(value) = properties.get(PropertyId::FontFamily) {
        traits.font_family = value.as_string().map(|s| s.to_string());
    }

    // Font size
    if let Ok(value) = properties.get(PropertyId::FontSize) {
        traits.font_size = extract_font_size(properties, &value);
    }

    // Font weight
    if let Ok(value) = properties.get(PropertyId::FontWeight) {
        if let Some(num) = value.as_integer() {
            traits.font_weight = Some(num as u16);
        }
    }

    // Font style
    if let Ok(value) = properties.get(PropertyId::FontStyle) {
        if let Some(enum_val) = value.as_enum() {
            traits.font_style = match enum_val {
                1 => Some(FontStyle::Italic),
                2 => Some(FontStyle::Oblique),
                _ => Some(FontStyle::Normal),
            };
        }
    }

    // Margins (for spacing calculations - reserved for future use)
    let _margin_top = properties
        .get(PropertyId::MarginTop)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);
    let _margin_right = properties
        .get(PropertyId::MarginRight)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);
    let _margin_bottom = properties
        .get(PropertyId::MarginBottom)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);
    let _margin_left = properties
        .get(PropertyId::MarginLeft)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);

    // Padding
    let padding_top = properties
        .get(PropertyId::PaddingTop)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);
    let padding_right = properties
        .get(PropertyId::PaddingRight)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);
    let padding_bottom = properties
        .get(PropertyId::PaddingBottom)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);
    let padding_left = properties
        .get(PropertyId::PaddingLeft)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);

    traits.padding = Some([padding_top, padding_right, padding_bottom, padding_left]);

    // Border width
    let border_top = properties
        .get(PropertyId::BorderTopWidth)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);
    let border_right = properties
        .get(PropertyId::BorderRightWidth)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);
    let border_bottom = properties
        .get(PropertyId::BorderBottomWidth)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);
    let border_left = properties
        .get(PropertyId::BorderLeftWidth)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);

    traits.border_width = Some([border_top, border_right, border_bottom, border_left]);

    // Border colors
    let border_top_color = properties
        .get(PropertyId::BorderTopColor)
        .ok()
        .and_then(|v| v.as_color())
        .unwrap_or(fop_types::Color::BLACK);
    let border_right_color = properties
        .get(PropertyId::BorderRightColor)
        .ok()
        .and_then(|v| v.as_color())
        .unwrap_or(fop_types::Color::BLACK);
    let border_bottom_color = properties
        .get(PropertyId::BorderBottomColor)
        .ok()
        .and_then(|v| v.as_color())
        .unwrap_or(fop_types::Color::BLACK);
    let border_left_color = properties
        .get(PropertyId::BorderLeftColor)
        .ok()
        .and_then(|v| v.as_color())
        .unwrap_or(fop_types::Color::BLACK);

    traits.border_color = Some([
        border_top_color,
        border_right_color,
        border_bottom_color,
        border_left_color,
    ]);

    // Border styles
    let border_top_style = properties
        .get(PropertyId::BorderTopStyle)
        .ok()
        .and_then(|v| parse_border_style(v.as_string()))
        .unwrap_or(BorderStyle::Solid);
    let border_right_style = properties
        .get(PropertyId::BorderRightStyle)
        .ok()
        .and_then(|v| parse_border_style(v.as_string()))
        .unwrap_or(BorderStyle::Solid);
    let border_bottom_style = properties
        .get(PropertyId::BorderBottomStyle)
        .ok()
        .and_then(|v| parse_border_style(v.as_string()))
        .unwrap_or(BorderStyle::Solid);
    let border_left_style = properties
        .get(PropertyId::BorderLeftStyle)
        .ok()
        .and_then(|v| parse_border_style(v.as_string()))
        .unwrap_or(BorderStyle::Solid);

    traits.border_style = Some([
        border_top_style,
        border_right_style,
        border_bottom_style,
        border_left_style,
    ]);

    // Text alignment
    if let Ok(value) = properties.get(PropertyId::TextAlign) {
        if let Some(s) = value.as_string() {
            traits.text_align = match s {
                "left" => Some(TextAlign::Left),
                "right" => Some(TextAlign::Right),
                "center" => Some(TextAlign::Center),
                "justify" => Some(TextAlign::Justify),
                _ => None,
            };
        }
    }

    // Line height
    traits.line_height = extract_line_height(properties);

    // Letter spacing
    traits.letter_spacing = extract_letter_spacing(properties);

    // Word spacing
    traits.word_spacing = extract_word_spacing(properties);

    // Border radius
    traits.border_radius = extract_border_radius(properties);

    // Overflow
    let overflow = extract_overflow(properties);
    // Only set if not the default (Visible)
    if overflow != OverflowBehavior::Visible {
        traits.overflow = Some(overflow);
    }

    // Opacity
    let opacity = extract_opacity(properties);
    // Only set if not the default (1.0)
    if (opacity - 1.0).abs() > f64::EPSILON {
        traits.opacity = Some(opacity);
    }

    // Text transform
    if let Ok(value) = properties.get(PropertyId::TextTransform) {
        if let Some(s) = value.as_string() {
            traits.text_transform = match s {
                "uppercase" => Some(TextTransform::Uppercase),
                "lowercase" => Some(TextTransform::Lowercase),
                "capitalize" => Some(TextTransform::Capitalize),
                _ => None,
            };
        }
    }

    // Font variant
    if let Ok(value) = properties.get(PropertyId::FontVariant) {
        if let Some(s) = value.as_string() {
            traits.font_variant = match s {
                "small-caps" => Some(FontVariant::SmallCaps),
                _ => None,
            };
        }
    }

    // Display align
    if let Ok(value) = properties.get(PropertyId::DisplayAlign) {
        if let Some(s) = value.as_string() {
            traits.display_align = match s {
                "center" => Some(DisplayAlign::Center),
                "after" => Some(DisplayAlign::After),
                _ => Some(DisplayAlign::Before),
            };
        }
    }

    // Baseline shift (super/sub/length)
    if let Ok(value) = properties.get(PropertyId::BaselineShift) {
        if let Some(s) = value.as_string() {
            traits.baseline_shift = match s {
                "super" => Some(0.5), // 50% of font-size upward
                "sub" => Some(-0.3),  // 30% of font-size downward
                "baseline" | "0" => Some(0.0),
                _ => None,
            };
        } else if let Some(len) = value.as_length() {
            // Store as pt value; will be divided by font-size later
            traits.baseline_shift = Some(len.to_pt() / 12.0); // relative to 12pt default
        }
    }

    // Hyphenation
    if let Ok(value) = properties.get(PropertyId::Hyphenate) {
        if let Some(s) = value.as_string() {
            traits.hyphenate = Some(s == "true");
        } else if let Some(b) = value.as_boolean() {
            traits.hyphenate = Some(b);
        }
    }

    if let Ok(value) = properties.get(PropertyId::HyphenationPushCharacterCount) {
        if let Some(i) = value.as_integer() {
            traits.hyphenation_push_chars = Some(i as u32);
        }
    }

    if let Ok(value) = properties.get(PropertyId::HyphenationRemainCharacterCount) {
        if let Some(i) = value.as_integer() {
            traits.hyphenation_remain_chars = Some(i as u32);
        }
    }

    // Font stretch
    if let Ok(value) = properties.get(PropertyId::FontStretch) {
        if let Some(s) = value.as_string() {
            traits.font_stretch = match s {
                "ultra-condensed" => Some(FontStretch::UltraCondensed),
                "extra-condensed" => Some(FontStretch::ExtraCondensed),
                "condensed" => Some(FontStretch::Condensed),
                "semi-condensed" => Some(FontStretch::SemiCondensed),
                "semi-expanded" => Some(FontStretch::SemiExpanded),
                "expanded" => Some(FontStretch::Expanded),
                "extra-expanded" => Some(FontStretch::ExtraExpanded),
                "ultra-expanded" => Some(FontStretch::UltraExpanded),
                _ => None, // "normal" -> None (default)
            };
        }
    }

    // Text align last
    if let Ok(value) = properties.get(PropertyId::TextAlignLast) {
        if let Some(s) = value.as_string() {
            traits.text_align_last = match s {
                "left" | "start" => Some(TextAlign::Left),
                "right" | "end" => Some(TextAlign::Right),
                "center" => Some(TextAlign::Center),
                "justify" => Some(TextAlign::Justify),
                _ => None,
            };
        }
    }

    // Change bar color
    if let Ok(value) = properties.get(PropertyId::ChangeBarColor) {
        traits.change_bar_color = value.as_color();
    }

    // span property (for multi-column spanning)
    if let Ok(value) = properties.get(PropertyId::Span) {
        if value.as_string() == Some("all") {
            traits.span = Span::All;
        }
    }

    // role attribute for accessibility tagging
    if let Ok(value) = properties.get(PropertyId::Role) {
        traits.role = value.as_string().map(|s| s.to_string());
    }

    // xml:lang for language tagging
    if let Ok(value) = properties.get(PropertyId::XmlLang) {
        traits.xml_lang = value.as_string().map(|s| s.to_string());
    }

    // writing-mode
    if let Ok(value) = properties.get(PropertyId::WritingMode) {
        if let Some(s) = value.as_string() {
            traits.writing_mode = match s {
                "rl-tb" | "rl" => WritingMode::RlTb,
                "tb-rl" | "tb" => WritingMode::TbRl,
                "tb-lr" => WritingMode::TbLr,
                _ => WritingMode::LrTb,
            };
        }
    }

    // direction
    if let Ok(value) = properties.get(PropertyId::Direction) {
        if let Some(s) = value.as_string() {
            traits.direction = match s {
                "rtl" => Direction::Rtl,
                _ => Direction::Ltr,
            };
        }
    }

    traits
}

/// Estimate the pixel width of a text string given font properties.
/// Uses approximate character width metrics — a full implementation would
/// use actual TTF glyph advances via `ttf-parser`.
///
/// Returns estimated width in points.
///
/// # Examples
///
/// ```
/// use fop_layout::measure_text_width;
/// use fop_types::Length;
///
/// let width = measure_text_width("Hello", Length::from_pt(12.0), None);
/// assert!(width.to_pt() > 0.0);
/// ```
pub fn measure_text_width(text: &str, font_size: Length, font_weight: Option<u16>) -> Length {
    if text.is_empty() {
        return Length::ZERO;
    }

    // Average character width approximation:
    // - Normal weight: ~0.5 × font-size per character (for Latin)
    // - Bold: ~0.55 × font-size
    // - CJK characters: ~1.0 × font-size (full-width)
    let weight_factor = match font_weight {
        Some(w) if w >= 600 => 0.55,
        _ => 0.50,
    };

    let mut total_width = 0.0_f64;
    for ch in text.chars() {
        let char_factor = if is_cjk(ch) {
            1.0
        } else if ch == ' ' {
            0.25
        } else if r#"iIl1!|.,;'"#.contains(ch) {
            0.3
        } else if "mMwW".contains(ch) {
            0.7
        } else {
            weight_factor
        };
        total_width += char_factor;
    }

    Length::from_pt(total_width * font_size.to_pt())
}

/// Check if a character is a CJK (Chinese/Japanese/Korean) character
fn is_cjk(ch: char) -> bool {
    matches!(ch,
        '\u{3000}'..='\u{9FFF}'   // CJK Unified Ideographs + common CJK
        | '\u{AC00}'..='\u{D7AF}' // Korean Hangul syllables
        | '\u{F900}'..='\u{FAFF}' // CJK Compatibility Ideographs
        | '\u{FF00}'..='\u{FFEF}' // Halfwidth and Fullwidth Forms
    )
}

/// Resolve a set of resolved text traits to one of the 14 Standard PDF font
/// names whose AFM advance-width tables live in [`FontRegistry`].
///
/// The mapping uses three inputs:
/// * `font-family` — a case-insensitive substring match selects the family
///   (`times`/generic `serif` → Times, `courier`/`mono` → Courier, otherwise
///   Helvetica, which is also the fall-back for unknown families).
/// * `font-weight` — a weight of 600 or greater selects the bold face.
/// * `font-style` — `italic`/`oblique` selects the slanted face (Times uses the
///   "Italic"/"BoldItalic" face names, Helvetica and Courier use the
///   "Oblique"/"BoldOblique" names).
///
/// The returned name is guaranteed to exist in [`FontRegistry::new`], so a
/// subsequent [`FontRegistry::get_or_default`] yields the correct per-variant
/// advance widths rather than silently collapsing every face onto Helvetica.
pub(crate) fn resolve_standard_font_name(traits: &TraitSet) -> &'static str {
    let family = traits
        .font_family
        .as_deref()
        .unwrap_or("Helvetica")
        .to_ascii_lowercase();

    let bold = traits.font_weight.map(|w| w >= 600).unwrap_or(false);
    let italic = matches!(
        traits.font_style,
        Some(FontStyle::Italic) | Some(FontStyle::Oblique)
    );

    // Generic `serif` must not match inside `sans-serif`.
    let is_serif =
        family.contains("times") || (family.contains("serif") && !family.contains("sans"));
    let is_mono = family.contains("courier") || family.contains("mono");

    if is_serif {
        match (bold, italic) {
            (false, false) => "Times-Roman",
            (true, false) => "Times-Bold",
            (false, true) => "Times-Italic",
            (true, true) => "Times-BoldItalic",
        }
    } else if is_mono {
        match (bold, italic) {
            (false, false) => "Courier",
            (true, false) => "Courier-Bold",
            (false, true) => "Courier-Oblique",
            (true, true) => "Courier-BoldOblique",
        }
    } else {
        match (bold, italic) {
            (false, false) => "Helvetica",
            (true, false) => "Helvetica-Bold",
            (false, true) => "Helvetica-Oblique",
            (true, true) => "Helvetica-BoldOblique",
        }
    }
}

/// Measure the advance width of `text` using the **real** per-variant font
/// metrics held by `registry`.
///
/// Unlike [`measure_text_width`] (a coarse average-character-width estimator
/// kept for backwards compatibility), this resolves the concrete Standard-14
/// face from `traits` via [`resolve_standard_font_name`] and sums the exact AFM
/// advance widths for every character at the resolved `font-size` (defaulting to
/// 12 pt when unset).  This is the measurement used by the Knuth-Plass line
/// breaker and by per-line alignment in the block layout engine, so that line
/// breaking and area geometry agree on glyph widths.
pub(crate) fn measure_text_metrics(
    text: &str,
    traits: &TraitSet,
    registry: &FontRegistry,
) -> Length {
    if text.is_empty() {
        return Length::ZERO;
    }
    let font_size = traits.font_size.unwrap_or(Length::from_pt(12.0));
    let name = resolve_standard_font_name(traits);
    registry.get_or_default(name).measure_text(text, font_size)
}

/// Unit tests for em-unit font-size resolution in `extract_font_size`.
#[cfg(test)]
mod em_tests {
    use super::{extract_font_size, resolve_parent_font_size};
    use fop_core::{PropertyId, PropertyList, PropertyValue};
    use fop_types::{Length, Percentage};

    fn pt(v: f64) -> Length {
        Length::from_pt(v)
    }

    fn assert_pt_approx(got: Length, expected_pt: f64) {
        assert!(
            (got.to_pt() - expected_pt).abs() < 0.001,
            "expected {}pt, got {}pt",
            expected_pt,
            got.to_pt()
        );
    }

    // ── basic em resolution ──────────────────────────────────────────────────

    /// `font-size="1.5em"` on a child whose parent has `font-size="10pt"` → 15 pt.
    #[test]
    fn test_em_1_5_under_10pt_parent_yields_15pt() {
        let mut parent_props = PropertyList::new();
        parent_props.set(PropertyId::FontSize, PropertyValue::Length(pt(10.0)));

        let mut child_props = PropertyList::with_parent(&parent_props);
        // 1.5em stored by the parser as Percentage::new(1.5)
        child_props.set(
            PropertyId::FontSize,
            PropertyValue::Percentage(Percentage::new(1.5)),
        );

        let value = child_props
            .get(PropertyId::FontSize)
            .expect("test: get FontSize");
        let size = extract_font_size(&child_props, &value).expect("test: extract_font_size");
        assert_pt_approx(size, 15.0);
    }

    /// `font-size="0.8em"` on a child whose parent has `font-size="10pt"` → 8 pt.
    #[test]
    fn test_em_0_8_under_10pt_parent_yields_8pt() {
        let mut parent_props = PropertyList::new();
        parent_props.set(PropertyId::FontSize, PropertyValue::Length(pt(10.0)));

        let mut child_props = PropertyList::with_parent(&parent_props);
        child_props.set(
            PropertyId::FontSize,
            PropertyValue::Percentage(Percentage::new(0.8)),
        );

        let value = child_props
            .get(PropertyId::FontSize)
            .expect("test: get FontSize");
        let size = extract_font_size(&child_props, &value).expect("test: extract_font_size");
        assert_pt_approx(size, 8.0);
    }

    /// Without an explicit parent the fallback is 12 pt, so `1.5em` → 18 pt.
    #[test]
    fn test_em_without_parent_uses_12pt_default() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::FontSize,
            PropertyValue::Percentage(Percentage::new(1.5)),
        );

        let value = props.get(PropertyId::FontSize).expect("test: get FontSize");
        let size = extract_font_size(&props, &value).expect("test: extract_font_size");
        // 1.5 × 12 pt (default) = 18 pt
        assert_pt_approx(size, 18.0);
    }

    // ── nested em compounding ────────────────────────────────────────────────

    /// Nested em test:
    /// - grandparent: `font-size="10pt"`
    /// - parent:      `font-size="1.5em"` → 15 pt
    /// - child:       `font-size="1.5em"` → 22.5 pt (1.5 × 15)
    ///
    /// The bug report requires: "child of a 1.5em-of-10pt block sees 15 pt as
    /// its parent".  This is verified explicitly in the parent step.
    #[test]
    fn test_nested_em_compounds() {
        let mut grandparent_props = PropertyList::new();
        grandparent_props.set(PropertyId::FontSize, PropertyValue::Length(pt(10.0)));

        let mut parent_props = PropertyList::with_parent(&grandparent_props);
        parent_props.set(
            PropertyId::FontSize,
            PropertyValue::Percentage(Percentage::new(1.5)),
        );

        // Verify the parent step: 1.5em × 10pt = 15pt
        let parent_val = parent_props
            .get(PropertyId::FontSize)
            .expect("test: get parent FontSize");
        let parent_size =
            extract_font_size(&parent_props, &parent_val).expect("test: parent extract");
        assert_pt_approx(parent_size, 15.0);

        // Child step: 1.5em × 15pt = 22.5pt (ems compound)
        let mut child_props = PropertyList::with_parent(&parent_props);
        child_props.set(
            PropertyId::FontSize,
            PropertyValue::Percentage(Percentage::new(1.5)),
        );

        let child_val = child_props
            .get(PropertyId::FontSize)
            .expect("test: get child FontSize");
        let child_size = extract_font_size(&child_props, &child_val).expect("test: child extract");
        assert_pt_approx(child_size, 22.5);
    }

    /// A 0.8em child under a 1.5em-of-10pt parent: 0.8 × 15 = 12 pt.
    #[test]
    fn test_em_child_of_em_parent_compounds() {
        let mut grandparent_props = PropertyList::new();
        grandparent_props.set(PropertyId::FontSize, PropertyValue::Length(pt(10.0)));

        let mut parent_props = PropertyList::with_parent(&grandparent_props);
        parent_props.set(
            PropertyId::FontSize,
            PropertyValue::Percentage(Percentage::new(1.5)),
        );

        let mut child_props = PropertyList::with_parent(&parent_props);
        child_props.set(
            PropertyId::FontSize,
            PropertyValue::Percentage(Percentage::new(0.8)),
        );

        let child_val = child_props
            .get(PropertyId::FontSize)
            .expect("test: get FontSize");
        let child_size =
            extract_font_size(&child_props, &child_val).expect("test: extract_font_size");
        // 0.8 × (1.5 × 10) = 0.8 × 15 = 12 pt
        assert_pt_approx(child_size, 12.0);
    }

    // ── resolve_parent_font_size ─────────────────────────────────────────────

    /// With no parent, resolve_parent_font_size returns the 12 pt default.
    #[test]
    fn test_resolve_parent_font_size_no_parent_returns_default() {
        let props = PropertyList::new();
        let parent_size = resolve_parent_font_size(&props);
        assert_pt_approx(parent_size, 12.0);
    }

    /// With an absolute-length parent, the parent's resolved value is returned.
    #[test]
    fn test_resolve_parent_font_size_absolute_parent() {
        let mut parent_props = PropertyList::new();
        parent_props.set(PropertyId::FontSize, PropertyValue::Length(pt(14.0)));

        let child_props = PropertyList::with_parent(&parent_props);
        let parent_size = resolve_parent_font_size(&child_props);
        assert_pt_approx(parent_size, 14.0);
    }
}
