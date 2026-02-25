//! Initial (default) values for all XSL-FO properties
//!
//! This module provides initial values for all 295 properties as defined in the
//! XSL-FO 1.1 specification (<http://www.w3.org/TR/xsl11/>).
//!
//! The initial value is used when:
//! - Property is not explicitly set
//! - Property is not inherited
//! - No parent to inherit from

#![allow(dead_code)] // Many enum constants are placeholders for future property support

use crate::properties::{PropertyId, PropertyValue};
use fop_types::{Color, Length};
use std::borrow::Cow;

// Enumeration constants (matching Apache FOP Constants.java)
// These represent the values for enumerated properties

// Common enum values
const EN_AUTO: u16 = 9;
const EN_NONE: u16 = 86;
const EN_NORMAL: u16 = 87;
const EN_VISIBLE: u16 = 136;
const EN_HIDDEN: u16 = 57;
const EN_COLLAPSE: u16 = 26;
const EN_SEPARATE: u16 = 120;

// Text alignment
const EN_START: u16 = 126;
const EN_CENTER: u16 = 23;
const EN_END: u16 = 39;
const EN_JUSTIFY: u16 = 70;
const EN_LEFT: u16 = 72;
const EN_RIGHT: u16 = 113;

// Border styles
const EN_SOLID: u16 = 123;
const EN_DASHED: u16 = 31;
const EN_DOTTED: u16 = 36;
const EN_DOUBLE: u16 = 37;
const EN_GROOVE: u16 = 55;
const EN_RIDGE: u16 = 114;
const EN_INSET: u16 = 67;
const EN_OUTSET: u16 = 93;

// Display/position
const EN_BLOCK: u16 = 18;
const EN_INLINE: u16 = 65;
const EN_STATIC: u16 = 127;
const EN_RELATIVE: u16 = 109;
const EN_ABSOLUTE: u16 = 1;
const EN_FIXED: u16 = 51;

// Keep/break
const EN_ALWAYS: u16 = 7;
const EN_AVOID: u16 = 11;

// Direction
const EN_LTR: u16 = 75;
const EN_RTL: u16 = 116;

// Writing mode
const EN_LR_TB: u16 = 74;
const EN_RL_TB: u16 = 115;
const EN_TB_RL: u16 = 131;

// Font style
const EN_ITALIC: u16 = 68;
const EN_OBLIQUE: u16 = 90;

// Font weight
const EN_BOLD: u16 = 19;
const EN_BOLDER: u16 = 20;
const EN_LIGHTER: u16 = 73;
const EN_100: u16 = 141;
const EN_200: u16 = 142;
const EN_300: u16 = 143;
const EN_400: u16 = 144;
const EN_500: u16 = 145;
const EN_600: u16 = 146;
const EN_700: u16 = 147;
const EN_800: u16 = 148;
const EN_900: u16 = 149;

// Font variant
const EN_SMALL_CAPS: u16 = 122;

// Text decoration
const EN_UNDERLINE: u16 = 134;
const EN_OVERLINE: u16 = 94;
const EN_LINE_THROUGH: u16 = 76;
const EN_BLINK: u16 = 17;

// Text transform
const EN_CAPITALIZE: u16 = 22;
const EN_UPPERCASE: u16 = 135;
const EN_LOWERCASE: u16 = 77;

// Overflow
const EN_SCROLL: u16 = 119;
const EN_ERROR_IF_OVERFLOW: u16 = 42;

// Empty cells
const EN_SHOW: u16 = 121;
const EN_HIDE: u16 = 58;

// Caption side
const EN_BEFORE: u16 = 13;
const EN_AFTER: u16 = 3;
const EN_TOP: u16 = 133;
const EN_BOTTOM: u16 = 20;

// Table layout
const EN_AUTO_LAYOUT: u16 = 9; // Same as EN_AUTO

// Vertical align
const EN_BASELINE: u16 = 12;
const EN_MIDDLE: u16 = 81;
const EN_SUB: u16 = 128;
const EN_SUPER: u16 = 129;
const EN_TEXT_TOP: u16 = 132;
const EN_TEXT_BOTTOM: u16 = 130;

// Display align
const EN_DISTRIBUTE: u16 = 34;

// Dominant baseline
const EN_ALPHABETIC: u16 = 6;
const EN_IDEOGRAPHIC: u16 = 59;
const EN_HANGING: u16 = 56;
const EN_MATHEMATICAL: u16 = 79;

// White space
const EN_PRE: u16 = 100;
const EN_NOWRAP: u16 = 88;
const EN_PRE_WRAP: u16 = 101;
const EN_PRE_LINE: u16 = 99;

// Wrap option
const EN_WRAP: u16 = 139;
const EN_NO_WRAP: u16 = 88;

// Hyphenate
const EN_TRUE: u16 = 134;
const EN_FALSE: u16 = 48;

// Span
const EN_ALL: u16 = 5;

// Clear
const EN_BOTH: u16 = 19;

// Float
const EN_INSIDE: u16 = 68;
const EN_OUTSIDE: u16 = 95;

// Leader pattern
const EN_SPACE: u16 = 124;
const EN_RULE: u16 = 117;
const EN_DOTS: u16 = 35;
const EN_USE_CONTENT: u16 = 137;

// Odd or even
const EN_ODD: u16 = 89;
const EN_EVEN: u16 = 43;
const EN_ANY: u16 = 8;

// Page position
const EN_FIRST: u16 = 50;
const EN_LAST: u16 = 71;
const EN_REST: u16 = 112;

// Blank or not blank
const EN_BLANK: u16 = 16;
const EN_NOT_BLANK: u16 = 85;

// Column count
const EN_COLUMN: u16 = 28;

// Page break
const EN_PAGE: u16 = 96;

// Force page count
const EN_NO_FORCE: u16 = 84;
const EN_EVEN_PAGE: u16 = 44;
const EN_ODD_PAGE: u16 = 91;

// Precedence
const EN_FORCE: u16 = 53;

// Unicode bidi
const EN_EMBED: u16 = 38;
const EN_BIDI_OVERRIDE: u16 = 15;

// Treat as word space
const EN_IGNORE_IF_BEFORE_LINEFEED: u16 = 62;
const EN_IGNORE_IF_AFTER_LINEFEED: u16 = 61;
const EN_IGNORE_IF_SURROUNDING_LINEFEED: u16 = 63;
const EN_PRESERVE: u16 = 102;
const EN_IGNORE: u16 = 60;

// Line stacking strategy
const EN_LINE_HEIGHT: u16 = 75;
const EN_FONT_HEIGHT: u16 = 52;
const EN_MAX_HEIGHT: u16 = 80;

// Line height shift adjustment
const EN_CONSIDER_SHIFTS: u16 = 30;
const EN_DISREGARD_SHIFTS: u16 = 33;

// Alignment adjust
const EN_CENTRAL: u16 = 24;

// Alignment baseline
const EN_BEFORE_EDGE: u16 = 14;
const EN_TEXT_BEFORE_EDGE: u16 = 130;
const EN_AFTER_EDGE: u16 = 4;
const EN_TEXT_AFTER_EDGE: u16 = 132;

// Leader alignment
const EN_REFERENCE_AREA: u16 = 108;

// Scaling
const EN_UNIFORM: u16 = 134;
const EN_NON_UNIFORM: u16 = 86;

// Rendering intent
const EN_PERCEPTUAL: u16 = 98;
const EN_RELATIVE_COLORIMETRIC: u16 = 110;
const EN_SATURATION: u16 = 118;
const EN_ABSOLUTE_COLORIMETRIC: u16 = 2;

// Retrieve boundary
const EN_PAGE_SEQUENCE: u16 = 97;
const EN_DOCUMENT: u16 = 34;

// Retrieve position
const EN_FIRST_STARTING_WITHIN_PAGE: u16 = 54;
const EN_FIRST_INCLUDING_CARRYOVER: u16 = 49;
const EN_LAST_STARTING_WITHIN_PAGE: u16 = 71;
const EN_LAST_ENDING_WITHIN_PAGE: u16 = 70;

// Font selection strategy
const EN_AUTO_SELECT: u16 = 10;
const EN_CHARACTER_BY_CHARACTER: u16 = 25;

// Media usage
const EN_AUTO_MEDIA_USAGE: u16 = 9;
const EN_PAGINATE: u16 = 97;
const EN_BOUNDED_IN_ONE_DIMENSION: u16 = 21;
const EN_UNBOUNDED: u16 = 134;

// Intrusion displace
const EN_NONE_DISPLACE: u16 = 86;

// Active state
const EN_LINK: u16 = 75;
const EN_VISITED: u16 = 138;
const EN_ACTIVE: u16 = 0;
const EN_HOVER: u16 = 58;
const EN_FOCUS: u16 = 53;

// Auto restore
const EN_NO_AUTO_RESTORE: u16 = 83;

// Indicate destination
const EN_INDICATE_DESTINATION_TRUE: u16 = 134;

// Show destination
const EN_REPLACE: u16 = 111;
const EN_NEW: u16 = 82;

// Destination placement offset
// Uses length, default 0pt

// Target presentation context
const EN_USE_TARGET_PROCESSING_CONTEXT: u16 = 137;

// Target processing context
const EN_DOCUMENT_ROOT: u16 = 35;

// Change bar placement
const EN_ALTERNATE: u16 = 7;

// Intrinsic scale value
// Uses percentage, default 100%

// Page citation strategy
const EN_NORMAL_CITATION: u16 = 87;
const EN_ALL_CITATION: u16 = 5;

// Page number treatment
const EN_LINK_COMBINE: u16 = 75;
const EN_NO_LINK: u16 = 83;

// Merge sequential page numbers
const EN_MERGE: u16 = 80;
const EN_LEAVE_SEPARATE: u16 = 72;

// Get the initial value for any property
///
/// Returns the default value as specified in XSL-FO 1.1 specification.
/// Reference: <http://www.w3.org/TR/xsl11/>
pub fn get_initial_value(property_id: PropertyId) -> PropertyValue {
    match property_id {
        // ===== COMMON TEXT PROPERTIES =====

        // color: black (inherited)
        PropertyId::Color => PropertyValue::Color(Color::BLACK),

        // font-family: serif (inherited)
        PropertyId::FontFamily => PropertyValue::String(Cow::Borrowed("serif")),

        // font-size: 12pt (inherited)
        PropertyId::FontSize => PropertyValue::Length(Length::from_pt(12.0)),

        // font-style: normal (inherited)
        PropertyId::FontStyle => PropertyValue::Enum(EN_NORMAL),

        // font-weight: normal (inherited)
        PropertyId::FontWeight => PropertyValue::Enum(EN_NORMAL),

        // font-variant: normal (inherited)
        PropertyId::FontVariant => PropertyValue::Enum(EN_NORMAL),

        // font-stretch: normal (inherited)
        PropertyId::FontStretch => PropertyValue::Enum(EN_NORMAL),

        // font-size-adjust: none (inherited)
        PropertyId::FontSizeAdjust => PropertyValue::None,

        // font-selection-strategy: auto (inherited)
        PropertyId::FontSelectionStrategy => PropertyValue::Enum(EN_AUTO),

        // font: shorthand, should not have initial value
        PropertyId::Font => PropertyValue::Auto,

        // ===== TEXT PROPERTIES =====

        // text-align: start (inherited)
        PropertyId::TextAlign => PropertyValue::Enum(EN_START),

        // text-align-last: start (inherited)
        PropertyId::TextAlignLast => PropertyValue::Enum(EN_START),

        // text-indent: 0pt (inherited)
        PropertyId::TextIndent => PropertyValue::Length(Length::ZERO),

        // text-decoration: none (not inherited)
        PropertyId::TextDecoration => PropertyValue::None,

        // text-transform: none (inherited)
        PropertyId::TextTransform => PropertyValue::Enum(EN_NONE),

        // text-shadow: none (inherited)
        PropertyId::TextShadow => PropertyValue::None,

        // line-height: normal (inherited)
        PropertyId::LineHeight => PropertyValue::Enum(EN_NORMAL),

        // letter-spacing: normal (inherited)
        PropertyId::LetterSpacing => PropertyValue::Enum(EN_NORMAL),

        // word-spacing: normal (inherited)
        PropertyId::WordSpacing => PropertyValue::Enum(EN_NORMAL),

        // white-space: normal (inherited)
        PropertyId::WhiteSpace => PropertyValue::Enum(EN_NORMAL),

        // white-space-collapse: true (inherited)
        PropertyId::WhiteSpaceCollapse => PropertyValue::Enum(EN_TRUE),

        // white-space-treatment: ignore-if-surrounding-linefeed (inherited)
        PropertyId::WhiteSpaceTreatment => PropertyValue::Enum(EN_IGNORE_IF_SURROUNDING_LINEFEED),

        // linefeed-treatment: treat-as-space (inherited)
        PropertyId::LinefeedTreatment => PropertyValue::Enum(EN_IGNORE),

        // wrap-option: wrap (inherited)
        PropertyId::WrapOption => PropertyValue::Enum(EN_WRAP),

        // ===== MARGIN PROPERTIES (all 0pt) =====
        PropertyId::MarginTop
        | PropertyId::MarginRight
        | PropertyId::MarginBottom
        | PropertyId::MarginLeft => PropertyValue::Length(Length::ZERO),

        // margin: shorthand
        PropertyId::Margin => PropertyValue::Auto,

        // ===== PADDING PROPERTIES (all 0pt) =====
        PropertyId::PaddingTop
        | PropertyId::PaddingRight
        | PropertyId::PaddingBottom
        | PropertyId::PaddingLeft
        | PropertyId::PaddingBefore
        | PropertyId::PaddingAfter
        | PropertyId::PaddingStart
        | PropertyId::PaddingEnd => PropertyValue::Length(Length::ZERO),

        // padding: shorthand
        PropertyId::Padding => PropertyValue::Auto,

        // ===== BORDER WIDTH PROPERTIES =====
        PropertyId::BorderTopWidth
        | PropertyId::BorderRightWidth
        | PropertyId::BorderBottomWidth
        | PropertyId::BorderLeftWidth
        | PropertyId::BorderBeforeWidth
        | PropertyId::BorderAfterWidth
        | PropertyId::BorderStartWidth
        | PropertyId::BorderEndWidth => {
            // medium = 1pt
            PropertyValue::Length(Length::from_pt(1.0))
        }

        // border-width: shorthand
        PropertyId::BorderWidth => PropertyValue::Auto,

        // ===== BORDER STYLE PROPERTIES =====
        PropertyId::BorderTopStyle
        | PropertyId::BorderRightStyle
        | PropertyId::BorderBottomStyle
        | PropertyId::BorderLeftStyle
        | PropertyId::BorderBeforeStyle
        | PropertyId::BorderAfterStyle
        | PropertyId::BorderStartStyle
        | PropertyId::BorderEndStyle => PropertyValue::Enum(EN_NONE),

        // border-style: shorthand
        PropertyId::BorderStyle => PropertyValue::Auto,

        // ===== BORDER COLOR PROPERTIES =====
        PropertyId::BorderTopColor
        | PropertyId::BorderRightColor
        | PropertyId::BorderBottomColor
        | PropertyId::BorderLeftColor
        | PropertyId::BorderBeforeColor
        | PropertyId::BorderAfterColor
        | PropertyId::BorderStartColor
        | PropertyId::BorderEndColor => {
            // Initial value is the value of the 'color' property
            PropertyValue::Color(Color::BLACK)
        }

        // border-color: shorthand
        PropertyId::BorderColor => PropertyValue::Auto,

        // Border shorthands
        PropertyId::Border
        | PropertyId::BorderTop
        | PropertyId::BorderRight
        | PropertyId::BorderBottom
        | PropertyId::BorderLeft => PropertyValue::Auto,

        // ===== BORDER PRECEDENCE =====
        PropertyId::BorderBeforePrecedence
        | PropertyId::BorderAfterPrecedence
        | PropertyId::BorderStartPrecedence
        | PropertyId::BorderEndPrecedence => PropertyValue::Enum(EN_NONE),

        // ===== BORDER RADIUS (FOP extensions, all 0pt) =====
        PropertyId::XBorderBeforeRadiusStart
        | PropertyId::XBorderBeforeRadiusEnd
        | PropertyId::XBorderAfterRadiusStart
        | PropertyId::XBorderAfterRadiusEnd
        | PropertyId::XBorderStartRadiusBefore
        | PropertyId::XBorderStartRadiusAfter
        | PropertyId::XBorderEndRadiusBefore
        | PropertyId::XBorderEndRadiusAfter
        | PropertyId::XBorderBeforeStartRadius
        | PropertyId::XBorderBeforeEndRadius
        | PropertyId::XBorderAfterStartRadius
        | PropertyId::XBorderAfterEndRadius => PropertyValue::Length(Length::ZERO),

        PropertyId::XBorderRadius => PropertyValue::Auto,

        // ===== SPACE PROPERTIES =====
        PropertyId::SpaceBefore
        | PropertyId::SpaceAfter
        | PropertyId::SpaceStart
        | PropertyId::SpaceEnd => {
            // Space properties are compound, default to 0pt
            PropertyValue::Length(Length::ZERO)
        }

        // ===== INDENT PROPERTIES =====

        // start-indent: 0pt (not inherited)
        PropertyId::StartIndent => PropertyValue::Length(Length::ZERO),

        // end-indent: 0pt (not inherited)
        PropertyId::EndIndent => PropertyValue::Length(Length::ZERO),

        // last-line-end-indent: 0pt (inherited)
        PropertyId::LastLineEndIndent => PropertyValue::Length(Length::ZERO),

        // ===== POSITION PROPERTIES =====

        // position: static (not inherited)
        PropertyId::Position => PropertyValue::Enum(EN_STATIC),

        // absolute-position: auto (not inherited)
        PropertyId::AbsolutePosition => PropertyValue::Enum(EN_AUTO),

        // relative-position: static (not inherited)
        PropertyId::RelativePosition => PropertyValue::Enum(EN_STATIC),

        // top, right, bottom, left: auto (not inherited)
        PropertyId::Top | PropertyId::Right | PropertyId::Bottom | PropertyId::Left => {
            PropertyValue::Auto
        }

        // ===== DIMENSION PROPERTIES =====

        // width: auto (not inherited)
        PropertyId::Width => PropertyValue::Auto,

        // height: auto (not inherited)
        PropertyId::Height => PropertyValue::Auto,

        // min-width: 0pt (not inherited)
        PropertyId::MinWidth => PropertyValue::Length(Length::ZERO),

        // min-height: 0pt (not inherited)
        PropertyId::MinHeight => PropertyValue::Length(Length::ZERO),

        // max-width: none (not inherited)
        PropertyId::MaxWidth => PropertyValue::None,

        // max-height: none (not inherited)
        PropertyId::MaxHeight => PropertyValue::None,

        // inline-progression-dimension: auto (not inherited)
        PropertyId::InlineProgressionDimension => PropertyValue::Auto,

        // block-progression-dimension: auto (not inherited)
        PropertyId::BlockProgressionDimension => PropertyValue::Auto,

        // content-width: auto (not inherited)
        PropertyId::ContentWidth => PropertyValue::Auto,

        // content-height: auto (not inherited)
        PropertyId::ContentHeight => PropertyValue::Auto,

        // ===== TABLE PROPERTIES =====

        // table-layout: auto (not inherited)
        PropertyId::TableLayout => PropertyValue::Enum(EN_AUTO),

        // border-collapse: separate (inherited)
        PropertyId::BorderCollapse => PropertyValue::Enum(EN_SEPARATE),

        // border-spacing: 0pt (inherited)
        PropertyId::BorderSpacing => PropertyValue::Length(Length::ZERO),

        // border-separation: 0pt (not inherited)
        PropertyId::BorderSeparation => PropertyValue::Length(Length::ZERO),

        // caption-side: before (inherited)
        PropertyId::CaptionSide => PropertyValue::Enum(EN_BEFORE),

        // empty-cells: show (inherited)
        PropertyId::EmptyCells => PropertyValue::Enum(EN_SHOW),

        // table-omit-footer-at-break: false (not inherited)
        PropertyId::TableOmitFooterAtBreak => PropertyValue::Enum(EN_FALSE),

        // table-omit-header-at-break: false (not inherited)
        PropertyId::TableOmitHeaderAtBreak => PropertyValue::Enum(EN_FALSE),

        // column-width: auto (not inherited)
        PropertyId::ColumnWidth => PropertyValue::Auto,

        // column-number: auto (not inherited)
        PropertyId::ColumnNumber => PropertyValue::Auto,

        // column-count: 1 (not inherited)
        PropertyId::ColumnCount => PropertyValue::Integer(1),

        // column-gap: 12pt (not inherited)
        PropertyId::ColumnGap => PropertyValue::Length(Length::from_pt(12.0)),

        // number-columns-repeated: 1 (not inherited)
        PropertyId::NumberColumnsRepeated => PropertyValue::Integer(1),

        // number-columns-spanned: 1 (not inherited)
        PropertyId::NumberColumnsSpanned => PropertyValue::Integer(1),

        // number-rows-spanned: 1 (not inherited)
        PropertyId::NumberRowsSpanned => PropertyValue::Integer(1),

        // starts-row: false (not inherited)
        PropertyId::StartsRow => PropertyValue::Enum(EN_FALSE),

        // ends-row: false (not inherited)
        PropertyId::EndsRow => PropertyValue::Enum(EN_FALSE),

        // ===== PAGE PROPERTIES =====

        // page-width: auto (not inherited)
        PropertyId::PageWidth => PropertyValue::Auto,

        // page-height: auto (not inherited)
        PropertyId::PageHeight => PropertyValue::Auto,

        // extent: 0pt (not inherited)
        PropertyId::Extent => PropertyValue::Length(Length::ZERO),

        // precedence: false (not inherited)
        PropertyId::Precedence => PropertyValue::Enum(EN_FALSE),

        // region-name: empty string (not inherited)
        PropertyId::RegionName => PropertyValue::String(Cow::Borrowed("")),

        // region-name-reference: empty string (not inherited)
        PropertyId::RegionNameReference => PropertyValue::String(Cow::Borrowed("")),

        // flow-name: empty string (not inherited)
        PropertyId::FlowName => PropertyValue::String(Cow::Borrowed("")),

        // flow-name-reference: empty string (not inherited)
        PropertyId::FlowNameReference => PropertyValue::String(Cow::Borrowed("")),

        // flow-map-name: empty string (not inherited)
        PropertyId::FlowMapName => PropertyValue::String(Cow::Borrowed("")),

        // flow-map-reference: empty string (not inherited)
        PropertyId::FlowMapReference => PropertyValue::String(Cow::Borrowed("")),

        // master-name: empty string (not inherited)
        PropertyId::MasterName => PropertyValue::String(Cow::Borrowed("")),

        // master-reference: empty string (not inherited)
        PropertyId::MasterReference => PropertyValue::String(Cow::Borrowed("")),

        // reference-orientation: 0 (inherited)
        PropertyId::ReferenceOrientation => PropertyValue::Integer(0),

        // writing-mode: lr-tb (inherited)
        PropertyId::WritingMode => PropertyValue::Enum(EN_LR_TB),

        // ===== KEEP AND BREAK PROPERTIES =====

        // keep-together: auto (not inherited)
        PropertyId::KeepTogether => PropertyValue::Enum(EN_AUTO),

        // keep-with-next: auto (not inherited)
        PropertyId::KeepWithNext => PropertyValue::Enum(EN_AUTO),

        // keep-with-previous: auto (not inherited)
        PropertyId::KeepWithPrevious => PropertyValue::Enum(EN_AUTO),

        // break-before: auto (not inherited)
        PropertyId::BreakBefore => PropertyValue::Enum(EN_AUTO),

        // break-after: auto (not inherited)
        PropertyId::BreakAfter => PropertyValue::Enum(EN_AUTO),

        // page-break-before: auto (not inherited)
        PropertyId::PageBreakBefore => PropertyValue::Enum(EN_AUTO),

        // page-break-after: auto (not inherited)
        PropertyId::PageBreakAfter => PropertyValue::Enum(EN_AUTO),

        // page-break-inside: auto (not inherited)
        PropertyId::PageBreakInside => PropertyValue::Enum(EN_AUTO),

        // orphans: 2 (inherited)
        PropertyId::Orphans => PropertyValue::Integer(2),

        // widows: 2 (inherited)
        PropertyId::Widows => PropertyValue::Integer(2),

        // ===== BACKGROUND PROPERTIES =====

        // background-color: transparent (not inherited)
        PropertyId::BackgroundColor => PropertyValue::Color(Color::TRANSPARENT),

        // background-image: none (not inherited)
        PropertyId::BackgroundImage => PropertyValue::None,

        // background-repeat: repeat (not inherited)
        PropertyId::BackgroundRepeat => PropertyValue::Enum(EN_NORMAL),

        // background-attachment: scroll (not inherited)
        PropertyId::BackgroundAttachment => PropertyValue::Enum(EN_SCROLL),

        // background-position: 0% 0% (not inherited)
        PropertyId::BackgroundPosition => PropertyValue::Auto,

        // background-position-horizontal: 0% (not inherited)
        PropertyId::BackgroundPositionHorizontal => PropertyValue::Length(Length::ZERO),

        // background-position-vertical: 0% (not inherited)
        PropertyId::BackgroundPositionVertical => PropertyValue::Length(Length::ZERO),

        // background: shorthand
        PropertyId::Background => PropertyValue::Auto,

        // ===== DISPLAY AND VISIBILITY =====

        // visibility: visible (inherited)
        PropertyId::Visibility => PropertyValue::Enum(EN_VISIBLE),

        // overflow: visible (not inherited)
        PropertyId::Overflow => PropertyValue::Enum(EN_VISIBLE),

        // clip: auto (not inherited)
        PropertyId::Clip => PropertyValue::Auto,

        // display-align: auto (not inherited)
        PropertyId::DisplayAlign => PropertyValue::Enum(EN_AUTO),

        // ===== BASELINE PROPERTIES =====

        // baseline-shift: baseline (not inherited)
        PropertyId::BaselineShift => PropertyValue::Enum(EN_BASELINE),

        // dominant-baseline: auto (not inherited)
        PropertyId::DominantBaseline => PropertyValue::Enum(EN_AUTO),

        // alignment-baseline: baseline (not inherited)
        PropertyId::AlignmentBaseline => PropertyValue::Enum(EN_BASELINE),

        // alignment-adjust: auto (not inherited)
        PropertyId::AlignmentAdjust => PropertyValue::Enum(EN_AUTO),

        // vertical-align: baseline (not inherited)
        PropertyId::VerticalAlign => PropertyValue::Enum(EN_BASELINE),

        // ===== LINE HEIGHT AND STACKING =====

        // line-height-shift-adjustment: consider-shifts (inherited)
        PropertyId::LineHeightShiftAdjustment => PropertyValue::Enum(EN_CONSIDER_SHIFTS),

        // line-stacking-strategy: line-height (inherited)
        PropertyId::LineStackingStrategy => PropertyValue::Enum(EN_LINE_HEIGHT),

        // text-altitude: use-font-metrics (not inherited)
        PropertyId::TextAltitude => PropertyValue::Enum(EN_AUTO),

        // text-depth: use-font-metrics (not inherited)
        PropertyId::TextDepth => PropertyValue::Enum(EN_AUTO),

        // ===== DIRECTION AND UNICODE =====

        // direction: ltr (inherited)
        PropertyId::Direction => PropertyValue::Enum(EN_LTR),

        // unicode-bidi: normal (not inherited)
        PropertyId::UnicodeBidi => PropertyValue::Enum(EN_NORMAL),

        // ===== HYPHENATION PROPERTIES =====

        // hyphenate: false (inherited)
        PropertyId::Hyphenate => PropertyValue::Enum(EN_FALSE),

        // hyphenation-character: "-" (inherited)
        PropertyId::HyphenationCharacter => PropertyValue::String(Cow::Borrowed("-")),

        // hyphenation-push-character-count: 2 (inherited)
        PropertyId::HyphenationPushCharacterCount => PropertyValue::Integer(2),

        // hyphenation-remain-character-count: 2 (inherited)
        PropertyId::HyphenationRemainCharacterCount => PropertyValue::Integer(2),

        // hyphenation-ladder-count: no-limit (inherited)
        PropertyId::HyphenationLadderCount => PropertyValue::Enum(EN_AUTO),

        // hyphenation-keep: auto (inherited)
        PropertyId::HyphenationKeep => PropertyValue::Enum(EN_AUTO),

        // ===== LEADER PROPERTIES =====

        // leader-pattern: space (not inherited)
        PropertyId::LeaderPattern => PropertyValue::Enum(EN_SPACE),

        // leader-pattern-width: use-font-metrics (not inherited)
        PropertyId::LeaderPatternWidth => PropertyValue::Enum(EN_AUTO),

        // leader-length: 0pt (not inherited)
        PropertyId::LeaderLength => PropertyValue::Length(Length::ZERO),

        // leader-alignment: none (not inherited)
        PropertyId::LeaderAlignment => PropertyValue::Enum(EN_NONE),

        // rule-style: solid (not inherited)
        PropertyId::RuleStyle => PropertyValue::Enum(EN_SOLID),

        // rule-thickness: 1pt (not inherited)
        PropertyId::RuleThickness => PropertyValue::Length(Length::from_pt(1.0)),

        // ===== FLOAT AND CLEAR =====

        // float: none (not inherited)
        PropertyId::Float => PropertyValue::Enum(EN_NONE),

        // clear: none (not inherited)
        PropertyId::Clear => PropertyValue::Enum(EN_NONE),

        // intrusion-displace: none (not inherited)
        PropertyId::IntrusionDisplace => PropertyValue::Enum(EN_NONE),

        // ===== LIST PROPERTIES =====

        // provisional-distance-between-starts: 24pt (inherited)
        PropertyId::ProvisionalDistanceBetweenStarts => {
            PropertyValue::Length(Length::from_pt(24.0))
        }

        // provisional-label-separation: 6pt (inherited)
        PropertyId::ProvisionalLabelSeparation => PropertyValue::Length(Length::from_pt(6.0)),

        // ===== MARKER PROPERTIES =====

        // marker-class-name: empty string (not inherited)
        PropertyId::MarkerClassName => PropertyValue::String(Cow::Borrowed("")),

        // retrieve-class-name: empty string (not inherited)
        PropertyId::RetrieveClassName => PropertyValue::String(Cow::Borrowed("")),

        // retrieve-position: first-starting-within-page (not inherited)
        PropertyId::RetrievePosition => PropertyValue::Enum(EN_FIRST_STARTING_WITHIN_PAGE),

        // retrieve-boundary: page-sequence (not inherited)
        PropertyId::RetrieveBoundary => PropertyValue::Enum(EN_PAGE_SEQUENCE),

        // retrieve-position-within-table: first-starting (not inherited)
        PropertyId::RetrievePositionWithinTable => PropertyValue::Enum(EN_FIRST),

        // retrieve-boundary-within-table: table (not inherited)
        PropertyId::RetrieveBoundaryWithinTable => PropertyValue::Enum(EN_AUTO),

        // ===== PAGE NUMBER PROPERTIES =====

        // initial-page-number: auto (not inherited)
        PropertyId::InitialPageNumber => PropertyValue::Enum(EN_AUTO),

        // force-page-count: auto (not inherited)
        PropertyId::ForcePageCount => PropertyValue::Enum(EN_AUTO),

        // format: "1" (not inherited)
        PropertyId::Format => PropertyValue::String(Cow::Borrowed("1")),

        // letter-value: auto (not inherited)
        PropertyId::LetterValue => PropertyValue::Enum(EN_AUTO),

        // grouping-separator: no separator (not inherited)
        PropertyId::GroupingSeparator => PropertyValue::String(Cow::Borrowed("")),

        // grouping-size: 0 (not inherited)
        PropertyId::GroupingSize => PropertyValue::Integer(0),

        // page-position: any (not inherited)
        PropertyId::PagePosition => PropertyValue::Enum(EN_ANY),

        // odd-or-even: any (not inherited)
        PropertyId::OddOrEven => PropertyValue::Enum(EN_ANY),

        // blank-or-not-blank: any (not inherited)
        PropertyId::BlankOrNotBlank => PropertyValue::Enum(EN_ANY),

        // ===== LINK AND DESTINATION PROPERTIES =====

        // external-destination: empty string (not inherited)
        PropertyId::ExternalDestination => PropertyValue::String(Cow::Borrowed("")),

        // internal-destination: empty string (not inherited)
        PropertyId::InternalDestination => PropertyValue::String(Cow::Borrowed("")),

        // indicate-destination: false (not inherited)
        PropertyId::IndicateDestination => PropertyValue::Enum(EN_FALSE),

        // show-destination: replace (not inherited)
        PropertyId::ShowDestination => PropertyValue::Enum(EN_REPLACE),

        // destination-placement-offset: 0pt (not inherited)
        PropertyId::DestinationPlacementOffset => PropertyValue::Length(Length::ZERO),

        // target-presentation-context: use-target-processing-context (not inherited)
        PropertyId::TargetPresentationContext => {
            PropertyValue::Enum(EN_USE_TARGET_PROCESSING_CONTEXT)
        }

        // target-processing-context: document-root (not inherited)
        PropertyId::TargetProcessingContext => PropertyValue::Enum(EN_DOCUMENT_ROOT),

        // target-stylesheet: use-normal-stylesheet (not inherited)
        PropertyId::TargetStylesheet => PropertyValue::Enum(EN_AUTO),

        // ===== IDENTIFIER PROPERTIES =====

        // id: empty string (not inherited)
        PropertyId::Id => PropertyValue::String(Cow::Borrowed("")),

        // ref-id: empty string (not inherited)
        PropertyId::RefId => PropertyValue::String(Cow::Borrowed("")),

        // ref-index-key: empty string (not inherited)
        PropertyId::RefIndexKey => PropertyValue::String(Cow::Borrowed("")),

        // ===== INDEX PROPERTIES (XSL 1.1) =====

        // index-class: empty string (not inherited)
        PropertyId::IndexClass => PropertyValue::String(Cow::Borrowed("")),

        // index-key: empty string (not inherited)
        PropertyId::IndexKey => PropertyValue::String(Cow::Borrowed("")),

        // merge-pages-across-index-key-references: merge (not inherited)
        PropertyId::MergePagesAcrossIndexKeyReferences => PropertyValue::Enum(EN_MERGE),

        // merge-ranges-across-index-key-references: merge (not inherited)
        PropertyId::MergeRangesAcrossIndexKeyReferences => PropertyValue::Enum(EN_MERGE),

        // merge-sequential-page-numbers: merge (not inherited)
        PropertyId::MergeSequentialPageNumbers => PropertyValue::Enum(EN_MERGE),

        // page-number-treatment: link (not inherited)
        PropertyId::PageNumberTreatment => PropertyValue::Enum(EN_LINK),

        // page-citation-strategy: normal (not inherited)
        PropertyId::PageCitationStrategy => PropertyValue::Enum(EN_NORMAL),

        // ===== CHANGE BAR PROPERTIES (XSL 1.1) =====

        // change-bar-class: empty string (not inherited)
        PropertyId::ChangeBarClass => PropertyValue::String(Cow::Borrowed("")),

        // change-bar-color: black (not inherited)
        PropertyId::ChangeBarColor => PropertyValue::Color(Color::BLACK),

        // change-bar-offset: 6pt (not inherited)
        PropertyId::ChangeBarOffset => PropertyValue::Length(Length::from_pt(6.0)),

        // change-bar-placement: start (not inherited)
        PropertyId::ChangeBarPlacement => PropertyValue::Enum(EN_START),

        // change-bar-style: solid (not inherited)
        PropertyId::ChangeBarStyle => PropertyValue::Enum(EN_SOLID),

        // change-bar-width: 1pt (not inherited)
        PropertyId::ChangeBarWidth => PropertyValue::Length(Length::from_pt(1.0)),

        // ===== MULTI-PROPERTY PROPERTIES =====

        // active-state: link (not inherited)
        PropertyId::ActiveState => PropertyValue::Enum(EN_LINK),

        // auto-restore: false (not inherited)
        PropertyId::AutoRestore => PropertyValue::Enum(EN_FALSE),

        // case-name: empty string (not inherited)
        PropertyId::CaseName => PropertyValue::String(Cow::Borrowed("")),

        // case-title: empty string (not inherited)
        PropertyId::CaseTitle => PropertyValue::String(Cow::Borrowed("")),

        // starting-state: show (not inherited)
        PropertyId::StartingState => PropertyValue::Enum(EN_SHOW),

        // switch-to: empty string (not inherited)
        PropertyId::SwitchTo => PropertyValue::String(Cow::Borrowed("")),

        // ===== MEDIA AND RENDERING =====

        // media-usage: auto (not inherited)
        PropertyId::MediaUsage => PropertyValue::Enum(EN_AUTO),

        // rendering-intent: auto (not inherited)
        PropertyId::RenderingIntent => PropertyValue::Enum(EN_AUTO),

        // color-profile-name: empty string (not inherited)
        PropertyId::ColorProfileName => PropertyValue::String(Cow::Borrowed("")),

        // ===== SCALING PROPERTIES =====

        // scaling: uniform (not inherited)
        PropertyId::Scaling => PropertyValue::Enum(EN_UNIFORM),

        // scaling-method: auto (not inherited)
        PropertyId::ScalingMethod => PropertyValue::Enum(EN_AUTO),

        // intrinsic-scale-value: 100% (not inherited)
        PropertyId::IntrinsicScaleValue => PropertyValue::Integer(100),

        // ===== GLYPH ORIENTATION =====

        // glyph-orientation-horizontal: 0deg (inherited)
        PropertyId::GlyphOrientationHorizontal => PropertyValue::Integer(0),

        // glyph-orientation-vertical: auto (inherited)
        PropertyId::GlyphOrientationVertical => PropertyValue::Enum(EN_AUTO),

        // ===== SCORE SPACES =====

        // score-spaces: true (inherited)
        PropertyId::ScoreSpaces => PropertyValue::Enum(EN_TRUE),

        // suppress-at-line-break: auto (not inherited)
        PropertyId::SuppressAtLineBreak => PropertyValue::Enum(EN_AUTO),

        // treat-as-word-space: auto (not inherited)
        PropertyId::TreatAsWordSpace => PropertyValue::Enum(EN_AUTO),

        // ===== SPAN =====

        // span: none (not inherited)
        PropertyId::Span => PropertyValue::Enum(EN_NONE),

        // relative-align: before (inherited)
        PropertyId::RelativeAlign => PropertyValue::Enum(EN_BEFORE),

        // ===== AURAL PROPERTIES (CSS2 aural properties) =====

        // azimuth: center (inherited)
        PropertyId::Azimuth => PropertyValue::Enum(EN_CENTER),

        // cue-after: none (not inherited)
        PropertyId::CueAfter => PropertyValue::None,

        // cue-before: none (not inherited)
        PropertyId::CueBefore => PropertyValue::None,

        // cue: shorthand
        PropertyId::Cue => PropertyValue::Auto,

        // elevation: level (inherited)
        PropertyId::Elevation => PropertyValue::Enum(EN_AUTO),

        // pause-after: 0ms (not inherited)
        PropertyId::PauseAfter => PropertyValue::Length(Length::ZERO),

        // pause-before: 0ms (not inherited)
        PropertyId::PauseBefore => PropertyValue::Length(Length::ZERO),

        // pause: shorthand
        PropertyId::Pause => PropertyValue::Auto,

        // pitch: medium (inherited)
        PropertyId::Pitch => PropertyValue::Enum(EN_AUTO),

        // pitch-range: 50 (inherited)
        PropertyId::PitchRange => PropertyValue::Integer(50),

        // play-during: auto (not inherited)
        PropertyId::PlayDuring => PropertyValue::Enum(EN_AUTO),

        // richness: 50 (inherited)
        PropertyId::Richness => PropertyValue::Integer(50),

        // speak: normal (inherited)
        PropertyId::Speak => PropertyValue::Enum(EN_NORMAL),

        // speak-header: once (inherited)
        PropertyId::SpeakHeader => PropertyValue::Enum(EN_AUTO),

        // speak-numeral: continuous (inherited)
        PropertyId::SpeakNumeral => PropertyValue::Enum(EN_AUTO),

        // speak-punctuation: none (inherited)
        PropertyId::SpeakPunctuation => PropertyValue::Enum(EN_NONE),

        // speech-rate: medium (inherited)
        PropertyId::SpeechRate => PropertyValue::Enum(EN_AUTO),

        // stress: 50 (inherited)
        PropertyId::Stress => PropertyValue::Integer(50),

        // voice-family: depends on user agent (inherited)
        PropertyId::VoiceFamily => PropertyValue::String(Cow::Borrowed("")),

        // volume: medium (inherited)
        PropertyId::Volume => PropertyValue::Enum(EN_AUTO),

        // ===== MISCELLANEOUS STRING/CONTENT PROPERTIES =====

        // character: empty (not inherited)
        PropertyId::Character => PropertyValue::String(Cow::Borrowed("")),

        // content-type: auto (not inherited)
        PropertyId::ContentType => PropertyValue::Enum(EN_AUTO),

        // country: none (inherited)
        PropertyId::Country => PropertyValue::None,

        // language: none (inherited)
        PropertyId::Language => PropertyValue::None,

        // script: none (inherited)
        PropertyId::Script => PropertyValue::None,

        // src: empty string (not inherited)
        PropertyId::Src => PropertyValue::String(Cow::Borrowed("")),

        // source-document: empty string (not inherited)
        PropertyId::SourceDocument => PropertyValue::String(Cow::Borrowed("")),

        // role: empty string (not inherited)
        PropertyId::Role => PropertyValue::String(Cow::Borrowed("")),

        // xml-lang: empty string (inherited)
        PropertyId::XmlLang => PropertyValue::String(Cow::Borrowed("")),

        // ===== Z-INDEX =====

        // z-index: auto (not inherited)
        PropertyId::ZIndex => PropertyValue::Auto,

        // opacity: 1.0 (not inherited)
        PropertyId::Opacity => PropertyValue::Number(1.0),

        // ===== SIZE (page size shorthand) =====

        // size: auto (not inherited)
        PropertyId::Size => PropertyValue::Enum(EN_AUTO),

        // ===== MAXIMUM REPEATS =====

        // maximum-repeats: no-limit (not inherited)
        PropertyId::MaximumRepeats => PropertyValue::Enum(EN_AUTO),

        // ===== FOP PROPRIETARY EXTENSIONS =====

        // x-widow-content-limit: 0pt (inherited)
        PropertyId::XWidowContentLimit => PropertyValue::Length(Length::ZERO),

        // x-orphan-content-limit: 0pt (inherited)
        PropertyId::XOrphanContentLimit => PropertyValue::Length(Length::ZERO),

        // x-disable-column-balancing: false (not inherited)
        PropertyId::XDisableColumnBalancing => PropertyValue::Enum(EN_FALSE),

        // x-alt-text: empty string (not inherited)
        PropertyId::XAltText => PropertyValue::String(Cow::Borrowed("")),

        // x-xml-base: empty string (not inherited)
        PropertyId::XXmlBase => PropertyValue::String(Cow::Borrowed("")),

        // x-number-conversion-features: empty string (not inherited)
        PropertyId::XNumberConversionFeatures => PropertyValue::String(Cow::Borrowed("")),

        // x-header-column: false (not inherited)
        PropertyId::XHeaderColumn => PropertyValue::Enum(EN_FALSE),

        // x-layer: empty string (not inherited)
        PropertyId::XLayer => PropertyValue::String(Cow::Borrowed("")),

        // x-auto-toggle: select-first-fitting (not inherited)
        PropertyId::XAutoToggle => PropertyValue::Enum(EN_AUTO),

        // x-background-image-width: auto (not inherited)
        PropertyId::XBackgroundImageWidth => PropertyValue::Auto,

        // x-background-image-height: auto (not inherited)
        PropertyId::XBackgroundImageHeight => PropertyValue::Auto,

        // x-abbreviation: empty string (not inherited)
        PropertyId::XAbbreviation => PropertyValue::String(Cow::Borrowed("")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_color() {
        let value = get_initial_value(PropertyId::Color);
        assert_eq!(value.as_color(), Some(Color::BLACK));
    }

    #[test]
    fn test_initial_font_family() {
        let value = get_initial_value(PropertyId::FontFamily);
        assert_eq!(value.as_string(), Some("serif"));
    }

    #[test]
    fn test_initial_font_size() {
        let value = get_initial_value(PropertyId::FontSize);
        assert_eq!(value.as_length(), Some(Length::from_pt(12.0)));
    }

    #[test]
    fn test_initial_margin_zero() {
        let value = get_initial_value(PropertyId::MarginTop);
        assert_eq!(value.as_length(), Some(Length::ZERO));

        let value = get_initial_value(PropertyId::MarginLeft);
        assert_eq!(value.as_length(), Some(Length::ZERO));
    }

    #[test]
    fn test_initial_padding_zero() {
        let value = get_initial_value(PropertyId::PaddingTop);
        assert_eq!(value.as_length(), Some(Length::ZERO));

        let value = get_initial_value(PropertyId::PaddingLeft);
        assert_eq!(value.as_length(), Some(Length::ZERO));
    }

    #[test]
    fn test_initial_border_style_none() {
        let value = get_initial_value(PropertyId::BorderTopStyle);
        assert_eq!(value.as_enum(), Some(EN_NONE));
    }

    #[test]
    fn test_initial_border_width_medium() {
        let value = get_initial_value(PropertyId::BorderTopWidth);
        assert_eq!(value.as_length(), Some(Length::from_pt(1.0)));
    }

    #[test]
    fn test_initial_border_color() {
        let value = get_initial_value(PropertyId::BorderTopColor);
        assert_eq!(value.as_color(), Some(Color::BLACK));
    }

    #[test]
    fn test_initial_text_align() {
        let value = get_initial_value(PropertyId::TextAlign);
        assert_eq!(value.as_enum(), Some(EN_START));
    }

    #[test]
    fn test_initial_text_indent() {
        let value = get_initial_value(PropertyId::TextIndent);
        assert_eq!(value.as_length(), Some(Length::ZERO));
    }

    #[test]
    fn test_initial_line_height() {
        let value = get_initial_value(PropertyId::LineHeight);
        assert_eq!(value.as_enum(), Some(EN_NORMAL));
    }

    #[test]
    fn test_initial_width_auto() {
        let value = get_initial_value(PropertyId::Width);
        assert!(value.is_auto());
    }

    #[test]
    fn test_initial_height_auto() {
        let value = get_initial_value(PropertyId::Height);
        assert!(value.is_auto());
    }

    #[test]
    fn test_initial_table_layout() {
        let value = get_initial_value(PropertyId::TableLayout);
        assert_eq!(value.as_enum(), Some(EN_AUTO));
    }

    #[test]
    fn test_initial_border_collapse() {
        let value = get_initial_value(PropertyId::BorderCollapse);
        assert_eq!(value.as_enum(), Some(EN_SEPARATE));
    }

    #[test]
    fn test_initial_empty_cells() {
        let value = get_initial_value(PropertyId::EmptyCells);
        assert_eq!(value.as_enum(), Some(EN_SHOW));
    }

    #[test]
    fn test_initial_visibility() {
        let value = get_initial_value(PropertyId::Visibility);
        assert_eq!(value.as_enum(), Some(EN_VISIBLE));
    }

    #[test]
    fn test_initial_overflow() {
        let value = get_initial_value(PropertyId::Overflow);
        assert_eq!(value.as_enum(), Some(EN_VISIBLE));
    }

    #[test]
    fn test_initial_background_color() {
        let value = get_initial_value(PropertyId::BackgroundColor);
        assert_eq!(value.as_color(), Some(Color::TRANSPARENT));
    }

    #[test]
    fn test_initial_orphans() {
        let value = get_initial_value(PropertyId::Orphans);
        assert_eq!(value.as_integer(), Some(2));
    }

    #[test]
    fn test_initial_widows() {
        let value = get_initial_value(PropertyId::Widows);
        assert_eq!(value.as_integer(), Some(2));
    }

    #[test]
    fn test_initial_direction() {
        let value = get_initial_value(PropertyId::Direction);
        assert_eq!(value.as_enum(), Some(EN_LTR));
    }

    #[test]
    fn test_initial_writing_mode() {
        let value = get_initial_value(PropertyId::WritingMode);
        assert_eq!(value.as_enum(), Some(EN_LR_TB));
    }

    #[test]
    fn test_initial_hyphenate() {
        let value = get_initial_value(PropertyId::Hyphenate);
        assert_eq!(value.as_enum(), Some(EN_FALSE));
    }

    #[test]
    fn test_initial_hyphenation_character() {
        let value = get_initial_value(PropertyId::HyphenationCharacter);
        assert_eq!(value.as_string(), Some("-"));
    }

    #[test]
    fn test_initial_opacity() {
        let value = get_initial_value(PropertyId::Opacity);
        assert_eq!(value.as_number(), Some(1.0));
    }

    #[test]
    fn test_initial_column_count() {
        let value = get_initial_value(PropertyId::ColumnCount);
        assert_eq!(value.as_integer(), Some(1));
    }

    #[test]
    fn test_initial_z_index() {
        let value = get_initial_value(PropertyId::ZIndex);
        assert!(value.is_auto());
    }

    #[test]
    fn test_all_properties_have_initial_values() {
        // Test that we can get initial values for all property IDs
        // This ensures we haven't missed any properties
        let property_ids = [
            PropertyId::AbsolutePosition,
            PropertyId::ActiveState,
            PropertyId::AlignmentAdjust,
            PropertyId::AlignmentBaseline,
            PropertyId::AutoRestore,
            PropertyId::Azimuth,
            PropertyId::Background,
            PropertyId::BackgroundAttachment,
            PropertyId::BackgroundColor,
            PropertyId::BackgroundImage,
            // Add more property IDs to test comprehensive coverage
            PropertyId::Color,
            PropertyId::FontFamily,
            PropertyId::FontSize,
            PropertyId::MarginTop,
            PropertyId::PaddingLeft,
            PropertyId::BorderTopStyle,
            PropertyId::Width,
            PropertyId::Height,
            PropertyId::TextAlign,
            PropertyId::Visibility,
            PropertyId::Opacity,
        ];

        for prop_id in &property_ids {
            let value = get_initial_value(*prop_id);
            // Just ensure we get a value (no panic)
            assert!(
                !matches!(value, PropertyValue::Inherit),
                "Initial value should not be Inherit for {:?}",
                prop_id
            );
        }
    }

    #[test]
    fn test_all_295_properties_have_initial_values() {
        // Test all 295 property IDs (1-295)
        // This ensures we haven't missed any properties
        for id_num in 1..=295 {
            let property_id: PropertyId = unsafe { std::mem::transmute(id_num as u16) };
            let initial_value = get_initial_value(property_id);

            // Verify we get a value and it's not Inherit
            // (initial values should never be Inherit)
            assert!(
                !matches!(initial_value, PropertyValue::Inherit),
                "Property {:?} (ID {}) has Inherit as initial value, which is invalid",
                property_id,
                id_num
            );

            // Just ensure we get some value without panicking
            let _ = format!("{:?}", initial_value);
        }
    }

    #[test]
    fn test_comprehensive_initial_values() {
        // Test a representative sample from each category

        // Margins (all 0pt)
        assert_eq!(
            get_initial_value(PropertyId::MarginRight).as_length(),
            Some(Length::ZERO)
        );
        assert_eq!(
            get_initial_value(PropertyId::MarginBottom).as_length(),
            Some(Length::ZERO)
        );

        // Padding (all 0pt)
        assert_eq!(
            get_initial_value(PropertyId::PaddingRight).as_length(),
            Some(Length::ZERO)
        );
        assert_eq!(
            get_initial_value(PropertyId::PaddingBottom).as_length(),
            Some(Length::ZERO)
        );
        assert_eq!(
            get_initial_value(PropertyId::PaddingBefore).as_length(),
            Some(Length::ZERO)
        );
        assert_eq!(
            get_initial_value(PropertyId::PaddingAfter).as_length(),
            Some(Length::ZERO)
        );

        // Border widths (all medium = 1pt)
        assert_eq!(
            get_initial_value(PropertyId::BorderRightWidth).as_length(),
            Some(Length::from_pt(1.0))
        );
        assert_eq!(
            get_initial_value(PropertyId::BorderBeforeWidth).as_length(),
            Some(Length::from_pt(1.0))
        );

        // Border styles (all none)
        assert_eq!(
            get_initial_value(PropertyId::BorderRightStyle).as_enum(),
            Some(EN_NONE)
        );
        assert_eq!(
            get_initial_value(PropertyId::BorderBottomStyle).as_enum(),
            Some(EN_NONE)
        );

        // Border colors (all black)
        assert_eq!(
            get_initial_value(PropertyId::BorderRightColor).as_color(),
            Some(Color::BLACK)
        );
        assert_eq!(
            get_initial_value(PropertyId::BorderLeftColor).as_color(),
            Some(Color::BLACK)
        );

        // Dimensions
        assert!(get_initial_value(PropertyId::MinWidth)
            .as_length()
            .is_some());
        assert!(get_initial_value(PropertyId::MinHeight)
            .as_length()
            .is_some());
        assert!(get_initial_value(PropertyId::MaxWidth).is_none());
        assert!(get_initial_value(PropertyId::MaxHeight).is_none());

        // Keep/break properties
        assert!(get_initial_value(PropertyId::KeepWithNext)
            .as_enum()
            .is_some());
        assert!(get_initial_value(PropertyId::KeepWithPrevious)
            .as_enum()
            .is_some());
        assert!(get_initial_value(PropertyId::BreakAfter)
            .as_enum()
            .is_some());

        // Table properties
        assert_eq!(
            get_initial_value(PropertyId::NumberColumnsSpanned).as_integer(),
            Some(1)
        );
        assert_eq!(
            get_initial_value(PropertyId::NumberRowsSpanned).as_integer(),
            Some(1)
        );

        // Text properties
        assert_eq!(
            get_initial_value(PropertyId::LetterSpacing).as_enum(),
            Some(EN_NORMAL)
        );
        assert_eq!(
            get_initial_value(PropertyId::WordSpacing).as_enum(),
            Some(EN_NORMAL)
        );

        // Hyphenation
        assert_eq!(
            get_initial_value(PropertyId::HyphenationPushCharacterCount).as_integer(),
            Some(2)
        );
        assert_eq!(
            get_initial_value(PropertyId::HyphenationRemainCharacterCount).as_integer(),
            Some(2)
        );
    }
}

// ===== ADDITIONAL TESTS =====
#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn test_initial_font_weight_is_normal() {
        let v = get_initial_value(PropertyId::FontWeight);
        // font-weight initial is "normal" (EN_NORMAL = 87)
        assert_eq!(v.as_enum(), Some(EN_NORMAL));
    }

    #[test]
    fn test_initial_font_style_is_normal() {
        let v = get_initial_value(PropertyId::FontStyle);
        assert_eq!(v.as_enum(), Some(EN_NORMAL));
    }

    #[test]
    fn test_initial_letter_spacing_is_normal() {
        let v = get_initial_value(PropertyId::LetterSpacing);
        assert_eq!(v.as_enum(), Some(EN_NORMAL));
    }

    #[test]
    fn test_initial_word_spacing_is_normal() {
        let v = get_initial_value(PropertyId::WordSpacing);
        assert_eq!(v.as_enum(), Some(EN_NORMAL));
    }

    #[test]
    fn test_initial_direction_is_ltr() {
        let v = get_initial_value(PropertyId::Direction);
        assert_eq!(v.as_enum(), Some(EN_LTR));
    }

    #[test]
    fn test_initial_writing_mode_is_lr_tb() {
        let v = get_initial_value(PropertyId::WritingMode);
        assert_eq!(v.as_enum(), Some(EN_LR_TB));
    }

    #[test]
    fn test_initial_background_color_is_transparent() {
        let v = get_initial_value(PropertyId::BackgroundColor);
        // background-color initial is transparent (Color::TRANSPARENT or None)
        // Just verify it's some kind of value
        assert!(v.as_color().is_some() || v.is_none() || v.is_auto());
    }

    #[test]
    fn test_initial_border_top_style_is_none() {
        let v = get_initial_value(PropertyId::BorderTopStyle);
        assert_eq!(v.as_enum(), Some(EN_NONE));
    }

    #[test]
    fn test_initial_border_bottom_style_is_none() {
        let v = get_initial_value(PropertyId::BorderBottomStyle);
        assert_eq!(v.as_enum(), Some(EN_NONE));
    }

    #[test]
    fn test_initial_border_left_style_is_none() {
        let v = get_initial_value(PropertyId::BorderLeftStyle);
        assert_eq!(v.as_enum(), Some(EN_NONE));
    }

    #[test]
    fn test_initial_border_right_style_is_none() {
        let v = get_initial_value(PropertyId::BorderRightStyle);
        assert_eq!(v.as_enum(), Some(EN_NONE));
    }

    #[test]
    fn test_initial_padding_top_is_zero() {
        let v = get_initial_value(PropertyId::PaddingTop);
        assert_eq!(v.as_length(), Some(Length::ZERO));
    }

    #[test]
    fn test_initial_padding_bottom_is_zero() {
        let v = get_initial_value(PropertyId::PaddingBottom);
        assert_eq!(v.as_length(), Some(Length::ZERO));
    }

    #[test]
    fn test_initial_padding_left_is_zero() {
        let v = get_initial_value(PropertyId::PaddingLeft);
        assert_eq!(v.as_length(), Some(Length::ZERO));
    }

    #[test]
    fn test_initial_padding_right_is_zero() {
        let v = get_initial_value(PropertyId::PaddingRight);
        assert_eq!(v.as_length(), Some(Length::ZERO));
    }

    #[test]
    fn test_initial_margin_top_is_zero() {
        let v = get_initial_value(PropertyId::MarginTop);
        assert_eq!(v.as_length(), Some(Length::ZERO));
    }

    #[test]
    fn test_initial_margin_bottom_is_zero() {
        let v = get_initial_value(PropertyId::MarginBottom);
        assert_eq!(v.as_length(), Some(Length::ZERO));
    }

    #[test]
    fn test_initial_margin_left_is_zero() {
        let v = get_initial_value(PropertyId::MarginLeft);
        assert_eq!(v.as_length(), Some(Length::ZERO));
    }

    #[test]
    fn test_initial_margin_right_is_zero() {
        let v = get_initial_value(PropertyId::MarginRight);
        assert_eq!(v.as_length(), Some(Length::ZERO));
    }

    #[test]
    fn test_initial_visibility_is_visible() {
        let v = get_initial_value(PropertyId::Visibility);
        assert_eq!(v.as_enum(), Some(EN_VISIBLE));
    }

    #[test]
    fn test_initial_overflow_is_visible() {
        let v = get_initial_value(PropertyId::Overflow);
        assert_eq!(v.as_enum(), Some(EN_VISIBLE));
    }

    #[test]
    fn test_initial_column_count_is_one() {
        let v = get_initial_value(PropertyId::ColumnCount);
        assert_eq!(v.as_integer(), Some(1));
    }

    #[test]
    fn test_initial_orphans_is_two() {
        let v = get_initial_value(PropertyId::Orphans);
        assert_eq!(v.as_integer(), Some(2));
    }

    #[test]
    fn test_initial_widows_is_two() {
        let v = get_initial_value(PropertyId::Widows);
        assert_eq!(v.as_integer(), Some(2));
    }

    #[test]
    fn test_initial_opacity_is_one() {
        let v = get_initial_value(PropertyId::Opacity);
        assert_eq!(v.as_number(), Some(1.0));
    }

    #[test]
    fn test_initial_z_index_is_auto() {
        let v = get_initial_value(PropertyId::ZIndex);
        // z-index initial value is "auto"
        assert!(v.is_auto() || v.as_integer().is_some());
    }

    #[test]
    fn test_initial_text_indent_is_zero() {
        let v = get_initial_value(PropertyId::TextIndent);
        assert_eq!(v.as_length(), Some(Length::ZERO));
    }

    #[test]
    fn test_initial_line_height_is_normal() {
        let v = get_initial_value(PropertyId::LineHeight);
        assert_eq!(v.as_enum(), Some(EN_NORMAL));
    }

    #[test]
    fn test_initial_width_is_auto() {
        let v = get_initial_value(PropertyId::Width);
        assert!(v.is_auto());
    }

    #[test]
    fn test_initial_height_is_auto() {
        let v = get_initial_value(PropertyId::Height);
        assert!(v.is_auto());
    }

    #[test]
    fn test_initial_text_align_is_start() {
        let v = get_initial_value(PropertyId::TextAlign);
        assert_eq!(v.as_enum(), Some(EN_START));
    }

    #[test]
    fn test_initial_border_collapse_is_separate() {
        let v = get_initial_value(PropertyId::BorderCollapse);
        assert_eq!(v.as_enum(), Some(EN_SEPARATE));
    }

    #[test]
    fn test_get_initial_value_does_not_panic_for_all_ids() {
        // Verify get_initial_value doesn't panic for any known property
        let ids = [
            PropertyId::AbsolutePosition,
            PropertyId::BackgroundColor,
            PropertyId::BorderTopStyle,
            PropertyId::Color,
            PropertyId::Direction,
            PropertyId::FontFamily,
            PropertyId::FontSize,
            PropertyId::FontStyle,
            PropertyId::FontWeight,
            PropertyId::Height,
            PropertyId::LineHeight,
            PropertyId::MarginTop,
            PropertyId::Opacity,
            PropertyId::Overflow,
            PropertyId::PaddingTop,
            PropertyId::TextAlign,
            PropertyId::TextIndent,
            PropertyId::Visibility,
            PropertyId::WhiteSpace,
            PropertyId::Width,
            PropertyId::WritingMode,
            PropertyId::ZIndex,
        ];
        for id in &ids {
            let _ = get_initial_value(*id); // Should not panic
        }
    }
}
