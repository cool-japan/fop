//! Property value types

use fop_types::{Color, EvalContext, Expression, Gradient, Length, Percentage};
use std::borrow::Cow;

/// Relative font-size values (keywords)
///
/// Based on CSS Fonts Module Level 3 specification section 3.5
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RelativeFontSize {
    /// larger - multiply parent font size by 1.2
    Larger,
    /// smaller - divide parent font size by 1.2
    Smaller,
    /// xx-small - absolute keyword (9pt)
    XxSmall,
    /// x-small - absolute keyword (10pt)
    XSmall,
    /// small - absolute keyword (13pt)
    Small,
    /// medium - absolute keyword (16pt, default)
    Medium,
    /// large - absolute keyword (18pt)
    Large,
    /// x-large - absolute keyword (24pt)
    XLarge,
    /// xx-large - absolute keyword (32pt)
    XxLarge,
}

impl RelativeFontSize {
    /// Parse a string into a RelativeFontSize
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "larger" => Some(RelativeFontSize::Larger),
            "smaller" => Some(RelativeFontSize::Smaller),
            "xx-small" => Some(RelativeFontSize::XxSmall),
            "x-small" => Some(RelativeFontSize::XSmall),
            "small" => Some(RelativeFontSize::Small),
            "medium" => Some(RelativeFontSize::Medium),
            "large" => Some(RelativeFontSize::Large),
            "x-large" => Some(RelativeFontSize::XLarge),
            "xx-large" => Some(RelativeFontSize::XxLarge),
            _ => None,
        }
    }

    /// Resolve to an absolute font size
    ///
    /// For larger/smaller, requires parent font size
    /// For absolute keywords, ignores parent and returns fixed size
    pub fn resolve(&self, parent_size: Length) -> Length {
        match self {
            RelativeFontSize::Larger => {
                // Multiply by 1.2
                Length::from_millipoints((parent_size.millipoints() as f64 * 1.2) as i32)
            }
            RelativeFontSize::Smaller => {
                // Divide by 1.2
                Length::from_millipoints((parent_size.millipoints() as f64 / 1.2) as i32)
            }
            // Absolute size keywords
            RelativeFontSize::XxSmall => Length::from_pt(9.0),
            RelativeFontSize::XSmall => Length::from_pt(10.0),
            RelativeFontSize::Small => Length::from_pt(13.0),
            RelativeFontSize::Medium => Length::from_pt(16.0),
            RelativeFontSize::Large => Length::from_pt(18.0),
            RelativeFontSize::XLarge => Length::from_pt(24.0),
            RelativeFontSize::XxLarge => Length::from_pt(32.0),
        }
    }

    /// Check if this is a relative size (larger/smaller)
    pub fn is_relative(&self) -> bool {
        matches!(self, RelativeFontSize::Larger | RelativeFontSize::Smaller)
    }

    /// Check if this is an absolute keyword size
    pub fn is_absolute_keyword(&self) -> bool {
        !self.is_relative()
    }
}

/// A property value that can be stored in a PropertyList
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    /// A length value (e.g., "10pt", "2cm")
    Length(Length),

    /// A color value (e.g., "#FF0000", "red")
    Color(Color),

    /// A gradient value (e.g., "linear-gradient(red, blue)")
    Gradient(Gradient),

    /// An enumerated value (stored as u16 for compactness)
    Enum(u16),

    /// A string value (borrowed or owned)
    String(Cow<'static, str>),

    /// A percentage value (e.g., 50%, 100%)
    Percentage(Percentage),

    /// An integer value
    Integer(i32),

    /// A floating-point value
    Number(f64),

    /// A boolean value
    Boolean(bool),

    /// A list of values
    List(Vec<PropertyValue>),

    /// A space-separated pair of values
    Pair(Box<PropertyValue>, Box<PropertyValue>),

    /// A relative font-size value (larger, smaller, or size keywords)
    RelativeFontSize(RelativeFontSize),

    /// A CSS calc() expression
    Expression(Expression),

    /// The special value "auto"
    Auto,

    /// The special value "none"
    None,

    /// The special value "inherit"
    Inherit,
}

impl PropertyValue {
    /// Check if this is an auto value
    #[inline]
    pub fn is_auto(&self) -> bool {
        matches!(self, PropertyValue::Auto)
    }

    /// Check if this is a none value
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, PropertyValue::None)
    }

    /// Check if this is an inherit value
    #[inline]
    pub fn is_inherit(&self) -> bool {
        matches!(self, PropertyValue::Inherit)
    }

    /// Try to get a length value
    pub fn as_length(&self) -> Option<Length> {
        match self {
            PropertyValue::Length(len) => Some(*len),
            _ => None,
        }
    }

    /// Try to get a relative font size value
    pub fn as_relative_font_size(&self) -> Option<RelativeFontSize> {
        match self {
            PropertyValue::RelativeFontSize(rfs) => Some(*rfs),
            _ => None,
        }
    }

    /// Resolve a font size value to an absolute length
    ///
    /// If the value is a relative font size (larger/smaller/keywords),
    /// it will be resolved using the parent font size.
    pub fn resolve_font_size(&self, parent_size: Length) -> Option<Length> {
        match self {
            PropertyValue::Length(len) => Some(*len),
            PropertyValue::RelativeFontSize(rfs) => Some(rfs.resolve(parent_size)),
            _ => None,
        }
    }

    /// Try to get a color value
    pub fn as_color(&self) -> Option<Color> {
        match self {
            PropertyValue::Color(color) => Some(*color),
            _ => None,
        }
    }

    /// Try to get a gradient value
    pub fn as_gradient(&self) -> Option<&Gradient> {
        match self {
            PropertyValue::Gradient(gradient) => Some(gradient),
            _ => None,
        }
    }

    /// Try to get an enum value
    pub fn as_enum(&self) -> Option<u16> {
        match self {
            PropertyValue::Enum(e) => Some(*e),
            _ => None,
        }
    }

    /// Try to get a string value
    pub fn as_string(&self) -> Option<&str> {
        match self {
            PropertyValue::String(s) => Some(s.as_ref()),
            _ => None,
        }
    }

    /// Try to get an integer value
    pub fn as_integer(&self) -> Option<i32> {
        match self {
            PropertyValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Try to get a number value
    pub fn as_number(&self) -> Option<f64> {
        match self {
            PropertyValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Try to get a boolean value
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            PropertyValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get a percentage value
    pub fn as_percentage(&self) -> Option<Percentage> {
        match self {
            PropertyValue::Percentage(p) => Some(*p),
            _ => None,
        }
    }

    /// Resolve a value to a length, applying percentage if needed
    pub fn resolve_length(&self, base: Length) -> Option<Length> {
        match self {
            PropertyValue::Length(len) => Some(*len),
            PropertyValue::Percentage(pct) => Some(pct.of(base)),
            _ => None,
        }
    }

    /// Try to get an expression value
    pub fn as_expression(&self) -> Option<&Expression> {
        match self {
            PropertyValue::Expression(expr) => Some(expr),
            _ => None,
        }
    }

    /// Resolve a value to a length, evaluating expressions if needed
    pub fn resolve_with_context(&self, context: &EvalContext) -> Option<Length> {
        match self {
            PropertyValue::Length(len) => Some(*len),
            PropertyValue::Percentage(pct) => context.base_width.map(|base| pct.of(base)),
            PropertyValue::Expression(expr) => expr.evaluate(context).ok(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_value() {
        let val = PropertyValue::Length(Length::from_pt(12.0));
        assert!(val.as_length().is_some());
        assert_eq!(
            val.as_length().expect("test: should succeed"),
            Length::from_pt(12.0)
        );
    }

    #[test]
    fn test_color_value() {
        let val = PropertyValue::Color(Color::RED);
        assert!(val.as_color().is_some());
        assert_eq!(val.as_color().expect("test: should succeed"), Color::RED);
    }

    #[test]
    fn test_enum_value() {
        let val = PropertyValue::Enum(42);
        assert_eq!(val.as_enum(), Some(42));
    }

    #[test]
    fn test_string_value() {
        let val = PropertyValue::String(Cow::Borrowed("test"));
        assert_eq!(val.as_string(), Some("test"));
    }

    #[test]
    fn test_special_values() {
        assert!(PropertyValue::Auto.is_auto());
        assert!(PropertyValue::None.is_none());
        assert!(PropertyValue::Inherit.is_inherit());
    }

    #[test]
    fn test_pair_value() {
        let val = PropertyValue::Pair(
            Box::new(PropertyValue::Length(Length::from_pt(10.0))),
            Box::new(PropertyValue::Length(Length::from_pt(20.0))),
        );

        match val {
            PropertyValue::Pair(first, second) => {
                assert_eq!(first.as_length(), Some(Length::from_pt(10.0)));
                assert_eq!(second.as_length(), Some(Length::from_pt(20.0)));
            }
            _ => panic!("Expected Pair"),
        }
    }

    #[test]
    fn test_relative_font_size_parsing() {
        assert_eq!(
            RelativeFontSize::parse("larger"),
            Some(RelativeFontSize::Larger)
        );
        assert_eq!(
            RelativeFontSize::parse("smaller"),
            Some(RelativeFontSize::Smaller)
        );
        assert_eq!(
            RelativeFontSize::parse("xx-small"),
            Some(RelativeFontSize::XxSmall)
        );
        assert_eq!(
            RelativeFontSize::parse("x-small"),
            Some(RelativeFontSize::XSmall)
        );
        assert_eq!(
            RelativeFontSize::parse("small"),
            Some(RelativeFontSize::Small)
        );
        assert_eq!(
            RelativeFontSize::parse("medium"),
            Some(RelativeFontSize::Medium)
        );
        assert_eq!(
            RelativeFontSize::parse("large"),
            Some(RelativeFontSize::Large)
        );
        assert_eq!(
            RelativeFontSize::parse("x-large"),
            Some(RelativeFontSize::XLarge)
        );
        assert_eq!(
            RelativeFontSize::parse("xx-large"),
            Some(RelativeFontSize::XxLarge)
        );
        assert_eq!(RelativeFontSize::parse("invalid"), None);
    }

    #[test]
    fn test_relative_font_size_larger() {
        let parent = Length::from_pt(12.0);
        let larger = RelativeFontSize::Larger;
        let result = larger.resolve(parent);
        // 12 * 1.2 = 14.4
        assert_eq!(result, Length::from_pt(14.4));
    }

    #[test]
    fn test_relative_font_size_smaller() {
        let parent = Length::from_pt(12.0);
        let smaller = RelativeFontSize::Smaller;
        let result = smaller.resolve(parent);
        // 12 / 1.2 = 10
        assert_eq!(result, Length::from_pt(10.0));
    }

    #[test]
    fn test_relative_font_size_absolute_keywords() {
        assert_eq!(
            RelativeFontSize::XxSmall.resolve(Length::from_pt(100.0)),
            Length::from_pt(9.0)
        );
        assert_eq!(
            RelativeFontSize::XSmall.resolve(Length::from_pt(100.0)),
            Length::from_pt(10.0)
        );
        assert_eq!(
            RelativeFontSize::Small.resolve(Length::from_pt(100.0)),
            Length::from_pt(13.0)
        );
        assert_eq!(
            RelativeFontSize::Medium.resolve(Length::from_pt(100.0)),
            Length::from_pt(16.0)
        );
        assert_eq!(
            RelativeFontSize::Large.resolve(Length::from_pt(100.0)),
            Length::from_pt(18.0)
        );
        assert_eq!(
            RelativeFontSize::XLarge.resolve(Length::from_pt(100.0)),
            Length::from_pt(24.0)
        );
        assert_eq!(
            RelativeFontSize::XxLarge.resolve(Length::from_pt(100.0)),
            Length::from_pt(32.0)
        );
    }

    #[test]
    fn test_relative_font_size_checks() {
        assert!(RelativeFontSize::Larger.is_relative());
        assert!(RelativeFontSize::Smaller.is_relative());
        assert!(!RelativeFontSize::Medium.is_relative());
        assert!(RelativeFontSize::Medium.is_absolute_keyword());
        assert!(RelativeFontSize::Large.is_absolute_keyword());
        assert!(!RelativeFontSize::Larger.is_absolute_keyword());
    }

    #[test]
    fn test_property_value_relative_font_size() {
        let val = PropertyValue::RelativeFontSize(RelativeFontSize::Larger);
        assert!(val.as_relative_font_size().is_some());
        assert_eq!(val.as_relative_font_size(), Some(RelativeFontSize::Larger));
    }

    #[test]
    fn test_resolve_font_size_with_length() {
        let val = PropertyValue::Length(Length::from_pt(14.0));
        let parent = Length::from_pt(12.0);
        assert_eq!(val.resolve_font_size(parent), Some(Length::from_pt(14.0)));
    }

    #[test]
    fn test_resolve_font_size_with_larger() {
        let val = PropertyValue::RelativeFontSize(RelativeFontSize::Larger);
        let parent = Length::from_pt(12.0);
        // 12 * 1.2 = 14.4
        assert_eq!(val.resolve_font_size(parent), Some(Length::from_pt(14.4)));
    }

    #[test]
    fn test_resolve_font_size_with_smaller() {
        let val = PropertyValue::RelativeFontSize(RelativeFontSize::Smaller);
        let parent = Length::from_pt(12.0);
        // 12 / 1.2 = 10
        assert_eq!(val.resolve_font_size(parent), Some(Length::from_pt(10.0)));
    }

    #[test]
    fn test_resolve_font_size_with_medium_keyword() {
        let val = PropertyValue::RelativeFontSize(RelativeFontSize::Medium);
        let parent = Length::from_pt(12.0);
        // medium is always 16pt regardless of parent
        assert_eq!(val.resolve_font_size(parent), Some(Length::from_pt(16.0)));
    }

    #[test]
    fn test_resolve_font_size_with_invalid_type() {
        let val = PropertyValue::String(Cow::Borrowed("test"));
        let parent = Length::from_pt(12.0);
        assert_eq!(val.resolve_font_size(parent), None);
    }

    #[test]
    fn test_expression_value() {
        let expr = Expression::parse("calc(100% - 20pt)").expect("test: should succeed");
        let val = PropertyValue::Expression(expr);
        assert!(val.as_expression().is_some());
    }

    #[test]
    fn test_resolve_with_context_expression() {
        let expr = Expression::parse("calc(100% - 20pt)").expect("test: should succeed");
        let val = PropertyValue::Expression(expr);
        let ctx = EvalContext::with_width(Length::from_pt(200.0), Length::from_pt(12.0));
        let result = val
            .resolve_with_context(&ctx)
            .expect("test: should succeed");
        assert_eq!(result, Length::from_pt(180.0));
    }

    #[test]
    fn test_resolve_with_context_length() {
        let val = PropertyValue::Length(Length::from_pt(50.0));
        let ctx = EvalContext::with_width(Length::from_pt(200.0), Length::from_pt(12.0));
        let result = val
            .resolve_with_context(&ctx)
            .expect("test: should succeed");
        assert_eq!(result, Length::from_pt(50.0));
    }

    #[test]
    fn test_resolve_with_context_percentage() {
        let val = PropertyValue::Percentage(Percentage::from_percent(50.0));
        let ctx = EvalContext::with_width(Length::from_pt(200.0), Length::from_pt(12.0));
        let result = val
            .resolve_with_context(&ctx)
            .expect("test: should succeed");
        assert_eq!(result, Length::from_pt(100.0));
    }
}

// ===== ADDITIONAL TESTS =====
#[cfg(test)]
mod additional_tests {
    use super::*;
    use fop_types::{Color, Length, Percentage};

    // ===== PropertyValue type checks =====

    #[test]
    fn test_is_auto() {
        assert!(PropertyValue::Auto.is_auto());
        assert!(!PropertyValue::None.is_auto());
        assert!(!PropertyValue::Inherit.is_auto());
        assert!(!PropertyValue::Length(Length::from_pt(0.0)).is_auto());
    }

    #[test]
    fn test_is_none() {
        assert!(PropertyValue::None.is_none());
        assert!(!PropertyValue::Auto.is_none());
        assert!(!PropertyValue::Inherit.is_none());
        assert!(!PropertyValue::Integer(0).is_none());
    }

    #[test]
    fn test_is_inherit() {
        assert!(PropertyValue::Inherit.is_inherit());
        assert!(!PropertyValue::Auto.is_inherit());
        assert!(!PropertyValue::None.is_inherit());
        assert!(!PropertyValue::Boolean(false).is_inherit());
    }

    #[test]
    fn test_as_length_returns_none_for_auto() {
        assert!(PropertyValue::Auto.as_length().is_none());
    }

    #[test]
    fn test_as_length_returns_none_for_none() {
        assert!(PropertyValue::None.as_length().is_none());
    }

    #[test]
    fn test_as_length_returns_none_for_integer() {
        assert!(PropertyValue::Integer(42).as_length().is_none());
    }

    #[test]
    fn test_as_color_returns_none_for_length() {
        assert!(PropertyValue::Length(Length::from_pt(10.0))
            .as_color()
            .is_none());
    }

    #[test]
    fn test_as_enum_returns_none_for_string() {
        assert!(PropertyValue::String(std::borrow::Cow::Borrowed("foo"))
            .as_enum()
            .is_none());
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_as_integer_returns_none_for_number() {
        assert!(PropertyValue::Number(3.14).as_integer().is_none());
    }

    #[test]
    fn test_as_number_returns_none_for_integer() {
        assert!(PropertyValue::Integer(42).as_number().is_none());
    }

    #[test]
    fn test_as_boolean_returns_none_for_enum() {
        assert!(PropertyValue::Enum(1).as_boolean().is_none());
    }

    #[test]
    fn test_as_percentage_returns_none_for_length() {
        assert!(PropertyValue::Length(Length::from_pt(50.0))
            .as_percentage()
            .is_none());
    }

    #[test]
    fn test_as_string_returns_none_for_color() {
        assert!(PropertyValue::Color(Color::RED).as_string().is_none());
    }

    // ===== PropertyValue construction and retrieval =====

    #[test]
    fn test_integer_value_positive() {
        let v = PropertyValue::Integer(42);
        assert_eq!(v.as_integer(), Some(42));
    }

    #[test]
    fn test_integer_value_negative() {
        let v = PropertyValue::Integer(-7);
        assert_eq!(v.as_integer(), Some(-7));
    }

    #[test]
    fn test_integer_value_zero() {
        let v = PropertyValue::Integer(0);
        assert_eq!(v.as_integer(), Some(0));
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_number_value_positive() {
        let v = PropertyValue::Number(3.14);
        assert_eq!(v.as_number(), Some(3.14));
    }

    #[test]
    fn test_number_value_zero() {
        let v = PropertyValue::Number(0.0);
        assert_eq!(v.as_number(), Some(0.0));
    }

    #[test]
    fn test_boolean_true() {
        let v = PropertyValue::Boolean(true);
        assert_eq!(v.as_boolean(), Some(true));
    }

    #[test]
    fn test_boolean_false() {
        let v = PropertyValue::Boolean(false);
        assert_eq!(v.as_boolean(), Some(false));
    }

    #[test]
    fn test_percentage_value() {
        let v = PropertyValue::Percentage(Percentage::from_percent(75.0));
        assert_eq!(v.as_percentage(), Some(Percentage::from_percent(75.0)));
    }

    #[test]
    fn test_pair_value_access() {
        let v = PropertyValue::Pair(
            Box::new(PropertyValue::Length(Length::from_pt(10.0))),
            Box::new(PropertyValue::Length(Length::from_pt(20.0))),
        );
        match v {
            PropertyValue::Pair(a, b) => {
                assert_eq!(a.as_length(), Some(Length::from_pt(10.0)));
                assert_eq!(b.as_length(), Some(Length::from_pt(20.0)));
            }
            _ => panic!("Expected Pair"),
        }
    }

    #[test]
    fn test_list_value_empty() {
        let v = PropertyValue::List(vec![]);
        match v {
            PropertyValue::List(items) => assert!(items.is_empty()),
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_list_value_with_items() {
        let v = PropertyValue::List(vec![
            PropertyValue::Integer(1),
            PropertyValue::Integer(2),
            PropertyValue::Integer(3),
        ]);
        match v {
            PropertyValue::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].as_integer(), Some(1));
                assert_eq!(items[2].as_integer(), Some(3));
            }
            _ => panic!("Expected List"),
        }
    }

    // ===== resolve_length tests =====

    #[test]
    fn test_resolve_length_with_length() {
        let v = PropertyValue::Length(Length::from_pt(30.0));
        let result = v.resolve_length(Length::from_pt(100.0));
        assert_eq!(result, Some(Length::from_pt(30.0)));
    }

    #[test]
    fn test_resolve_length_with_percentage() {
        let v = PropertyValue::Percentage(Percentage::from_percent(25.0));
        let result = v.resolve_length(Length::from_pt(200.0));
        assert_eq!(result, Some(Length::from_pt(50.0)));
    }

    #[test]
    fn test_resolve_length_with_auto_returns_none() {
        let v = PropertyValue::Auto;
        let result = v.resolve_length(Length::from_pt(100.0));
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_length_with_none_returns_none() {
        let v = PropertyValue::None;
        let result = v.resolve_length(Length::from_pt(100.0));
        assert!(result.is_none());
    }

    // ===== RelativeFontSize edge cases =====

    #[test]
    fn test_relative_font_size_parse_all_keywords() {
        assert!(RelativeFontSize::parse("larger").is_some());
        assert!(RelativeFontSize::parse("smaller").is_some());
        assert!(RelativeFontSize::parse("xx-small").is_some());
        assert!(RelativeFontSize::parse("x-small").is_some());
        assert!(RelativeFontSize::parse("small").is_some());
        assert!(RelativeFontSize::parse("medium").is_some());
        assert!(RelativeFontSize::parse("large").is_some());
        assert!(RelativeFontSize::parse("x-large").is_some());
        assert!(RelativeFontSize::parse("xx-large").is_some());
    }

    #[test]
    fn test_relative_font_size_parse_invalid_returns_none() {
        assert!(RelativeFontSize::parse("").is_none());
        assert!(RelativeFontSize::parse("LARGE").is_none());
        assert!(RelativeFontSize::parse("12pt").is_none());
        assert!(RelativeFontSize::parse("unknown").is_none());
    }

    #[test]
    fn test_relative_font_size_is_relative_for_larger_smaller() {
        assert!(RelativeFontSize::Larger.is_relative());
        assert!(RelativeFontSize::Smaller.is_relative());
        assert!(!RelativeFontSize::Medium.is_relative());
        assert!(!RelativeFontSize::Large.is_relative());
    }

    #[test]
    fn test_relative_font_size_is_absolute_keyword() {
        assert!(RelativeFontSize::XxSmall.is_absolute_keyword());
        assert!(RelativeFontSize::XSmall.is_absolute_keyword());
        assert!(RelativeFontSize::Small.is_absolute_keyword());
        assert!(RelativeFontSize::Medium.is_absolute_keyword());
        assert!(RelativeFontSize::Large.is_absolute_keyword());
        assert!(RelativeFontSize::XLarge.is_absolute_keyword());
        assert!(RelativeFontSize::XxLarge.is_absolute_keyword());
        assert!(!RelativeFontSize::Larger.is_absolute_keyword());
        assert!(!RelativeFontSize::Smaller.is_absolute_keyword());
    }

    #[test]
    fn test_relative_font_size_resolve_xx_small() {
        let result = RelativeFontSize::XxSmall.resolve(Length::from_pt(16.0));
        assert_eq!(result, Length::from_pt(9.0));
    }

    #[test]
    fn test_relative_font_size_resolve_xx_large() {
        let result = RelativeFontSize::XxLarge.resolve(Length::from_pt(16.0));
        assert_eq!(result, Length::from_pt(32.0));
    }

    #[test]
    fn test_relative_font_size_larger_increases() {
        let parent = Length::from_pt(10.0);
        let result = RelativeFontSize::Larger.resolve(parent);
        assert!(result.millipoints() > parent.millipoints());
    }

    #[test]
    fn test_relative_font_size_smaller_decreases() {
        let parent = Length::from_pt(10.0);
        let result = RelativeFontSize::Smaller.resolve(parent);
        assert!(result.millipoints() < parent.millipoints());
    }

    // ===== PropertyValue equality =====

    #[test]
    fn test_property_value_equality_auto() {
        assert_eq!(PropertyValue::Auto, PropertyValue::Auto);
    }

    #[test]
    fn test_property_value_equality_none() {
        assert_eq!(PropertyValue::None, PropertyValue::None);
    }

    #[test]
    fn test_property_value_equality_inherit() {
        assert_eq!(PropertyValue::Inherit, PropertyValue::Inherit);
    }

    #[test]
    fn test_property_value_equality_integer() {
        assert_eq!(PropertyValue::Integer(42), PropertyValue::Integer(42));
        assert_ne!(PropertyValue::Integer(42), PropertyValue::Integer(43));
    }

    #[test]
    fn test_property_value_inequality_different_types() {
        assert_ne!(PropertyValue::Auto, PropertyValue::None);
        assert_ne!(PropertyValue::None, PropertyValue::Inherit);
        assert_ne!(PropertyValue::Integer(0), PropertyValue::Boolean(false));
    }
}
