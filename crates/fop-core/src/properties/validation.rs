//! Property validation with range checks and allowed enum values
//!
//! This module implements comprehensive validation rules for XSL-FO properties
//! based on Section 5 of the XSL-FO 1.1 specification.
//!
//! # Features
//! - Range validation for numeric and length properties
//! - Enum validation with allowed value sets
//! - Required property checks per element type
//! - Mutual exclusion constraints
//! - Detailed error messages with suggestions

use crate::properties::{PropertyId, PropertyValue};
use fop_types::{FopError, Result};
use std::collections::HashMap;

/// Validation error with detailed message and suggestion
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    /// Property that failed validation
    pub property: PropertyId,
    /// Error message describing what went wrong
    pub message: String,
    /// Optional suggestion for how to fix it
    pub suggestion: Option<String>,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(property: PropertyId, message: String, suggestion: Option<String>) -> Self {
        Self {
            property,
            message,
            suggestion,
        }
    }

    /// Convert to FopError
    pub fn to_fop_error(&self, value: &PropertyValue) -> FopError {
        let mut reason = self.message.clone();
        if let Some(ref suggestion) = self.suggestion {
            reason.push_str(" Suggestion: ");
            reason.push_str(suggestion);
        }
        FopError::PropertyValidation {
            property: self.property.name().to_string(),
            value: format!("{:?}", value),
            reason,
        }
    }
}

/// Validation rule for a property
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationRule {
    /// Range validation for numbers (min, max)
    Range { min: f64, max: f64 },
    /// Enum validation (allowed values as u16)
    Enum { allowed: Vec<u16> },
    /// String enum validation (allowed string values)
    StringEnum { allowed: Vec<&'static str> },
    /// Required property (must be present)
    Required,
    /// Conditional validation (when property X has value Y, this must be Z)
    Conditional {
        when: PropertyId,
        has: PropertyValue,
    },
}

/// Property validator with rule-based validation
pub struct PropertyValidator {
    rules: HashMap<PropertyId, Vec<ValidationRule>>,
}

impl Default for PropertyValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyValidator {
    /// Create a new property validator with default rules
    pub fn new() -> Self {
        let mut validator = Self {
            rules: HashMap::new(),
        };
        validator.register_default_rules();
        validator
    }

    /// Register default validation rules
    fn register_default_rules(&mut self) {
        // Opacity: 0.0 - 1.0
        self.add_rule(
            PropertyId::Opacity,
            ValidationRule::Range { min: 0.0, max: 1.0 },
        );

        // Column-count: 1 - 255
        self.add_rule(
            PropertyId::ColumnCount,
            ValidationRule::Range {
                min: 1.0,
                max: 255.0,
            },
        );

        // Orphans: 1 - 999
        self.add_rule(
            PropertyId::Orphans,
            ValidationRule::Range {
                min: 1.0,
                max: 999.0,
            },
        );

        // Widows: 1 - 999
        self.add_rule(
            PropertyId::Widows,
            ValidationRule::Range {
                min: 1.0,
                max: 999.0,
            },
        );

        // Z-index: -999999 - 999999
        self.add_rule(
            PropertyId::ZIndex,
            ValidationRule::Range {
                min: -999999.0,
                max: 999999.0,
            },
        );

        // Text-align enum values
        self.add_rule(
            PropertyId::TextAlign,
            ValidationRule::StringEnum {
                allowed: vec!["start", "end", "center", "justify", "left", "right"],
            },
        );

        // Overflow enum values
        self.add_rule(
            PropertyId::Overflow,
            ValidationRule::StringEnum {
                allowed: vec!["visible", "hidden", "scroll", "auto"],
            },
        );

        // Border styles (applied to all border-style properties)
        let border_style_enum = ValidationRule::StringEnum {
            allowed: vec![
                "none", "solid", "dashed", "dotted", "double", "groove", "ridge", "inset", "outset",
            ],
        };
        self.add_rule(PropertyId::BorderStyle, border_style_enum.clone());
        self.add_rule(PropertyId::BorderTopStyle, border_style_enum.clone());
        self.add_rule(PropertyId::BorderRightStyle, border_style_enum.clone());
        self.add_rule(PropertyId::BorderBottomStyle, border_style_enum.clone());
        self.add_rule(PropertyId::BorderLeftStyle, border_style_enum.clone());
        self.add_rule(PropertyId::BorderBeforeStyle, border_style_enum.clone());
        self.add_rule(PropertyId::BorderAfterStyle, border_style_enum.clone());
        self.add_rule(PropertyId::BorderStartStyle, border_style_enum.clone());
        self.add_rule(PropertyId::BorderEndStyle, border_style_enum);

        // Break-before and break-after enum values
        let break_enum = ValidationRule::StringEnum {
            allowed: vec!["auto", "column", "page", "even-page", "odd-page"],
        };
        self.add_rule(PropertyId::BreakBefore, break_enum.clone());
        self.add_rule(PropertyId::BreakAfter, break_enum);
    }

    /// Add a validation rule for a property
    pub fn add_rule(&mut self, property: PropertyId, rule: ValidationRule) {
        self.rules.entry(property).or_default().push(rule);
    }

    /// Validate a property value according to registered rules (instance method)
    pub fn validate(&self, id: PropertyId, value: &PropertyValue) -> Result<()> {
        // Auto, Inherit, and None are generally always valid
        if matches!(
            value,
            PropertyValue::Auto | PropertyValue::Inherit | PropertyValue::None
        ) {
            return Ok(());
        }

        // Check registered rules
        if let Some(rules) = self.rules.get(&id) {
            for rule in rules {
                self.validate_rule(id, value, rule)?;
            }
        }

        // Additional hardcoded validations
        match id {
            PropertyId::FontSize => Self::validate_positive_length(id, value),
            PropertyId::LineHeight => Self::validate_line_height(value),
            _ => Ok(()),
        }
    }

    /// Validate a property value (static method for backward compatibility)
    ///
    /// This creates a default validator and validates the value.
    /// For better performance with multiple validations, create a PropertyValidator
    /// instance and reuse it.
    pub fn validate_static(id: PropertyId, value: &PropertyValue) -> Result<()> {
        // Auto, Inherit, and None are generally always valid
        if matches!(
            value,
            PropertyValue::Auto | PropertyValue::Inherit | PropertyValue::None
        ) {
            return Ok(());
        }

        // For static validation, we only do basic hardcoded checks
        // to avoid creating a validator instance every time
        match id {
            PropertyId::FontSize => Self::validate_positive_length(id, value),
            PropertyId::ColumnCount => Self::validate_positive_integer(id, value, 1, 255),
            PropertyId::Widows => Self::validate_positive_integer(id, value, 1, 999),
            PropertyId::Orphans => Self::validate_positive_integer(id, value, 1, 999),
            PropertyId::ZIndex => Self::validate_integer_range(id, value, -999999, 999999),
            PropertyId::LineHeight => Self::validate_line_height(value),
            PropertyId::Opacity => Self::validate_range(id, value, 0.0, 1.0),
            _ => Ok(()),
        }
    }

    /// Validate a single rule
    fn validate_rule(
        &self,
        id: PropertyId,
        value: &PropertyValue,
        rule: &ValidationRule,
    ) -> Result<()> {
        match rule {
            ValidationRule::Range { min, max } => Self::validate_range(id, value, *min, *max),
            ValidationRule::Enum { allowed } => Self::validate_enum_values(id, value, allowed),
            ValidationRule::StringEnum { allowed } => {
                Self::validate_string_enum(id, value, allowed)
            }
            ValidationRule::Required => {
                // Required checks are typically done at element level
                Ok(())
            }
            ValidationRule::Conditional { .. } => {
                // Conditional checks require context
                Ok(())
            }
        }
    }

    /// Validate that a value is within the specified range
    fn validate_range(id: PropertyId, value: &PropertyValue, min: f64, max: f64) -> Result<()> {
        let num = match value {
            PropertyValue::Number(n) => *n,
            PropertyValue::Integer(i) => *i as f64,
            PropertyValue::Length(len) => len.to_pt(),
            PropertyValue::Percentage(p) => p.as_fraction(),
            _ => {
                return Err(FopError::PropertyValidation {
                    property: id.name().to_string(),
                    value: format!("{:?}", value),
                    reason: "Expected numeric value for range validation".to_string(),
                });
            }
        };

        if num < min || num > max {
            let error = ValidationError::new(
                id,
                format!("Value {} is out of range (must be {}-{})", num, min, max),
                Some(Self::get_range_suggestion(id, min, max)),
            );
            return Err(error.to_fop_error(value));
        }

        Ok(())
    }

    /// Get a helpful suggestion for range validation
    fn get_range_suggestion(id: PropertyId, min: f64, max: f64) -> String {
        match id {
            PropertyId::Opacity => {
                "Use a value between 0.0 (fully transparent) and 1.0 (fully opaque)".to_string()
            }
            PropertyId::ColumnCount => {
                format!("Column count must be between {} and {} columns", min, max)
            }
            PropertyId::Orphans => format!(
                "Orphans must be between {} and {} lines at the bottom of a page",
                min, max
            ),
            PropertyId::Widows => format!(
                "Widows must be between {} and {} lines at the top of a page",
                min, max
            ),
            PropertyId::ZIndex => {
                format!("Z-index must be between {} and {}", min, max)
            }
            _ => format!("Value must be between {} and {}", min, max),
        }
    }

    /// Validate enum values (u16)
    fn validate_enum_values(id: PropertyId, value: &PropertyValue, allowed: &[u16]) -> Result<()> {
        match value {
            PropertyValue::Enum(e) => {
                if !allowed.contains(e) {
                    return Err(FopError::PropertyValidation {
                        property: id.name().to_string(),
                        value: format!("{}", e),
                        reason: format!("Allowed enum values: {:?}", allowed),
                    });
                }
                Ok(())
            }
            _ => Err(FopError::PropertyValidation {
                property: id.name().to_string(),
                value: format!("{:?}", value),
                reason: "Expected enum value".to_string(),
            }),
        }
    }

    /// Validate string enum values
    fn validate_string_enum(
        id: PropertyId,
        value: &PropertyValue,
        allowed: &[&'static str],
    ) -> Result<()> {
        match value {
            PropertyValue::String(s) => {
                if !allowed.contains(&s.as_ref()) {
                    let error = ValidationError::new(
                        id,
                        format!("'{}' is not a valid value", s),
                        Some(format!("Use one of: {}", allowed.join(", "))),
                    );
                    return Err(error.to_fop_error(value));
                }
                Ok(())
            }
            _ => Err(FopError::PropertyValidation {
                property: id.name().to_string(),
                value: format!("{:?}", value),
                reason: format!("Expected one of: {}", allowed.join(", ")),
            }),
        }
    }

    /// Check for mutual exclusion between properties
    pub fn check_mutual_exclusion(
        &self,
        props: &[(PropertyId, PropertyValue)],
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Check width vs inline-progression-dimension
        let has_width = props.iter().any(|(id, _)| *id == PropertyId::Width);
        let has_ipd = props
            .iter()
            .any(|(id, _)| *id == PropertyId::InlineProgressionDimension);
        if has_width && has_ipd {
            errors.push(ValidationError::new(
                PropertyId::Width,
                "Both 'width' and 'inline-progression-dimension' are set".to_string(),
                Some(
                    "Use 'width' for simple cases or 'inline-progression-dimension' for advanced control, but not both"
                        .to_string(),
                ),
            ));
        }

        // Check height vs block-progression-dimension
        let has_height = props.iter().any(|(id, _)| *id == PropertyId::Height);
        let has_bpd = props
            .iter()
            .any(|(id, _)| *id == PropertyId::BlockProgressionDimension);
        if has_height && has_bpd {
            errors.push(ValidationError::new(
                PropertyId::Height,
                "Both 'height' and 'block-progression-dimension' are set".to_string(),
                Some(
                    "Use 'height' for simple cases or 'block-progression-dimension' for advanced control, but not both"
                        .to_string(),
                ),
            ));
        }

        errors
    }

    /// Validate length range
    pub fn validate_length_range(
        id: PropertyId,
        value: &PropertyValue,
        min: f64,
        max: f64,
    ) -> Result<()> {
        match value {
            PropertyValue::Length(len) => {
                let pts = len.to_pt();
                if pts < min || pts > max {
                    let error = ValidationError::new(
                        id,
                        format!("Length value {} is out of range ({}-{})", pts, min, max),
                        Some(format!("Value must be between {}pt and {}pt", min, max)),
                    );
                    return Err(error.to_fop_error(value));
                }
                Ok(())
            }
            PropertyValue::Auto | PropertyValue::None | PropertyValue::Inherit => Ok(()),
            _ => Err(FopError::PropertyValidation {
                property: id.name().to_string(),
                value: format!("{:?}", value),
                reason: "Expected length value".to_string(),
            }),
        }
    }

    /// Validate number range
    pub fn validate_number_range(
        id: PropertyId,
        value: &PropertyValue,
        min: f64,
        max: f64,
    ) -> Result<()> {
        match value {
            PropertyValue::Number(n) => {
                if *n < min || *n > max {
                    let error = ValidationError::new(
                        id,
                        format!("Number value {} is out of range ({}-{})", n, min, max),
                        Some(format!("Value must be between {} and {}", min, max)),
                    );
                    return Err(error.to_fop_error(value));
                }
                Ok(())
            }
            PropertyValue::Integer(i) => {
                let n = *i as f64;
                if n < min || n > max {
                    let error = ValidationError::new(
                        id,
                        format!("Integer value {} is out of range ({}-{})", i, min, max),
                        Some(format!("Value must be between {} and {}", min, max)),
                    );
                    return Err(error.to_fop_error(value));
                }
                Ok(())
            }
            PropertyValue::Auto | PropertyValue::None | PropertyValue::Inherit => Ok(()),
            _ => Err(FopError::PropertyValidation {
                property: id.name().to_string(),
                value: format!("{:?}", value),
                reason: "Expected numeric value".to_string(),
            }),
        }
    }

    /// Validate percentage range
    pub fn validate_percentage_range(
        id: PropertyId,
        value: &PropertyValue,
        min: f32,
        max: f32,
    ) -> Result<()> {
        match value {
            PropertyValue::Percentage(p) => {
                let fraction = p.as_fraction() as f32;
                if fraction < min || fraction > max {
                    let error = ValidationError::new(
                        id,
                        format!(
                            "Percentage value {}% is out of range ({}%-{}%)",
                            fraction * 100.0,
                            min * 100.0,
                            max * 100.0
                        ),
                        Some(format!(
                            "Value must be between {}% and {}%",
                            min * 100.0,
                            max * 100.0
                        )),
                    );
                    return Err(error.to_fop_error(value));
                }
                Ok(())
            }
            PropertyValue::Auto | PropertyValue::None | PropertyValue::Inherit => Ok(()),
            _ => Err(FopError::PropertyValidation {
                property: id.name().to_string(),
                value: format!("{:?}", value),
                reason: "Expected percentage value".to_string(),
            }),
        }
    }

    /// Validate enum (legacy method for backward compatibility)
    pub fn validate_enum(
        id: PropertyId,
        value: &PropertyValue,
        allowed_values: &[u16],
    ) -> Result<()> {
        Self::validate_enum_values(id, value, allowed_values)
    }

    /// Validate that a length is positive
    fn validate_positive_length(id: PropertyId, value: &PropertyValue) -> Result<()> {
        match value {
            PropertyValue::Length(len) => {
                if len.to_pt() <= 0.0 {
                    let error = ValidationError::new(
                        id,
                        format!("Length value {} must be positive", len.to_pt()),
                        Some("Font size must be greater than 0".to_string()),
                    );
                    return Err(error.to_fop_error(value));
                }
                Ok(())
            }
            // Relative/absolute font-size keywords (xx-small, small, medium, large, larger, etc.)
            PropertyValue::RelativeFontSize(_) => Ok(()),
            // Percentage values (e.g. 70% or em-based values stored as percentage)
            PropertyValue::Percentage(_) => Ok(()),
            // calc() expressions and dynamic values
            PropertyValue::Expression(_) => Ok(()),
            PropertyValue::Auto | PropertyValue::None | PropertyValue::Inherit => Ok(()),
            _ => Err(FopError::PropertyValidation {
                property: id.name().to_string(),
                value: format!("{:?}", value),
                reason: "Expected length value".to_string(),
            }),
        }
    }

    /// Validate that an integer is within range (inclusive)
    fn validate_positive_integer(
        id: PropertyId,
        value: &PropertyValue,
        min: i32,
        max: i32,
    ) -> Result<()> {
        match value {
            PropertyValue::Integer(i) => {
                if *i < min || *i > max {
                    let error = ValidationError::new(
                        id,
                        format!("Integer value {} is out of range ({}-{})", i, min, max),
                        Some(Self::get_range_suggestion(id, min as f64, max as f64)),
                    );
                    return Err(error.to_fop_error(value));
                }
                Ok(())
            }
            PropertyValue::Auto | PropertyValue::None | PropertyValue::Inherit => Ok(()),
            _ => Err(FopError::PropertyValidation {
                property: id.name().to_string(),
                value: format!("{:?}", value),
                reason: "Expected integer value".to_string(),
            }),
        }
    }

    /// Validate that an integer is within range (for z-index)
    fn validate_integer_range(
        id: PropertyId,
        value: &PropertyValue,
        min: i32,
        max: i32,
    ) -> Result<()> {
        match value {
            PropertyValue::Integer(i) => {
                if *i < min || *i > max {
                    let error = ValidationError::new(
                        id,
                        format!("Integer value {} is out of range ({}-{})", i, min, max),
                        Some(Self::get_range_suggestion(id, min as f64, max as f64)),
                    );
                    return Err(error.to_fop_error(value));
                }
                Ok(())
            }
            PropertyValue::Auto | PropertyValue::None | PropertyValue::Inherit => Ok(()),
            _ => Err(FopError::PropertyValidation {
                property: id.name().to_string(),
                value: format!("{:?}", value),
                reason: "Expected integer value".to_string(),
            }),
        }
    }

    /// Validate line-height property
    fn validate_line_height(value: &PropertyValue) -> Result<()> {
        match value {
            PropertyValue::Length(len) => {
                if len.to_pt() <= 0.0 {
                    let error = ValidationError::new(
                        PropertyId::LineHeight,
                        format!("Line height {} must be positive", len.to_pt()),
                        Some("Use a positive length, positive number, or 'normal'".to_string()),
                    );
                    return Err(error.to_fop_error(value));
                }
                Ok(())
            }
            PropertyValue::Number(n) => {
                if *n <= 0.0 {
                    let error = ValidationError::new(
                        PropertyId::LineHeight,
                        format!("Line height multiplier {} must be positive", n),
                        Some("Use a positive number (e.g., 1.5 for 150% of font size)".to_string()),
                    );
                    return Err(error.to_fop_error(value));
                }
                Ok(())
            }
            PropertyValue::String(s) if s.as_ref() == "normal" => Ok(()),
            PropertyValue::Percentage(p) => {
                if p.as_fraction() <= 0.0 {
                    let error = ValidationError::new(
                        PropertyId::LineHeight,
                        format!("Line height percentage {} must be positive", p),
                        Some("Use a positive percentage (e.g., 150%)".to_string()),
                    );
                    return Err(error.to_fop_error(value));
                }
                Ok(())
            }
            PropertyValue::Auto | PropertyValue::None | PropertyValue::Inherit => Ok(()),
            _ => {
                let error = ValidationError::new(
                    PropertyId::LineHeight,
                    "Invalid line-height value".to_string(),
                    Some("Use a positive length, positive number, or 'normal'".to_string()),
                );
                Err(error.to_fop_error(value))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fop_types::{Length, Percentage};

    #[test]
    fn test_opacity_range_validation() {
        let validator = PropertyValidator::new();

        // Valid opacity
        let valid = PropertyValue::Number(0.5);
        assert!(validator.validate(PropertyId::Opacity, &valid).is_ok());

        // Invalid: too high
        let invalid_high = PropertyValue::Number(1.5);
        assert!(validator
            .validate(PropertyId::Opacity, &invalid_high)
            .is_err());

        // Invalid: too low
        let invalid_low = PropertyValue::Number(-0.1);
        assert!(validator
            .validate(PropertyId::Opacity, &invalid_low)
            .is_err());

        // Edge cases
        let zero = PropertyValue::Number(0.0);
        assert!(validator.validate(PropertyId::Opacity, &zero).is_ok());

        let one = PropertyValue::Number(1.0);
        assert!(validator.validate(PropertyId::Opacity, &one).is_ok());
    }

    #[test]
    fn test_column_count_range_validation() {
        let validator = PropertyValidator::new();

        // Valid column count
        let valid = PropertyValue::Integer(3);
        assert!(validator.validate(PropertyId::ColumnCount, &valid).is_ok());

        // Invalid: zero
        let zero = PropertyValue::Integer(0);
        assert!(validator.validate(PropertyId::ColumnCount, &zero).is_err());

        // Invalid: negative
        let negative = PropertyValue::Integer(-1);
        assert!(validator
            .validate(PropertyId::ColumnCount, &negative)
            .is_err());

        // Invalid: too high
        let too_high = PropertyValue::Integer(300);
        assert!(validator
            .validate(PropertyId::ColumnCount, &too_high)
            .is_err());

        // Edge cases
        let min = PropertyValue::Integer(1);
        assert!(validator.validate(PropertyId::ColumnCount, &min).is_ok());

        let max = PropertyValue::Integer(255);
        assert!(validator.validate(PropertyId::ColumnCount, &max).is_ok());
    }

    #[test]
    fn test_orphans_widows_validation() {
        let validator = PropertyValidator::new();

        // Valid values
        let valid = PropertyValue::Integer(2);
        assert!(validator.validate(PropertyId::Orphans, &valid).is_ok());
        assert!(validator.validate(PropertyId::Widows, &valid).is_ok());

        // Invalid: zero
        let zero = PropertyValue::Integer(0);
        assert!(validator.validate(PropertyId::Orphans, &zero).is_err());
        assert!(validator.validate(PropertyId::Widows, &zero).is_err());

        // Invalid: too high
        let too_high = PropertyValue::Integer(1000);
        assert!(validator.validate(PropertyId::Orphans, &too_high).is_err());
        assert!(validator.validate(PropertyId::Widows, &too_high).is_err());

        // Edge cases
        let min = PropertyValue::Integer(1);
        assert!(validator.validate(PropertyId::Orphans, &min).is_ok());
        assert!(validator.validate(PropertyId::Widows, &min).is_ok());

        let max = PropertyValue::Integer(999);
        assert!(validator.validate(PropertyId::Orphans, &max).is_ok());
        assert!(validator.validate(PropertyId::Widows, &max).is_ok());
    }

    #[test]
    fn test_z_index_validation() {
        let validator = PropertyValidator::new();

        // Valid z-index values
        let positive = PropertyValue::Integer(100);
        assert!(validator.validate(PropertyId::ZIndex, &positive).is_ok());

        let negative = PropertyValue::Integer(-50);
        assert!(validator.validate(PropertyId::ZIndex, &negative).is_ok());

        let zero = PropertyValue::Integer(0);
        assert!(validator.validate(PropertyId::ZIndex, &zero).is_ok());

        // Invalid: out of range
        let too_high = PropertyValue::Integer(1_000_000);
        assert!(validator.validate(PropertyId::ZIndex, &too_high).is_err());

        let too_low = PropertyValue::Integer(-1_000_000);
        assert!(validator.validate(PropertyId::ZIndex, &too_low).is_err());

        // Edge cases
        let min = PropertyValue::Integer(-999999);
        assert!(validator.validate(PropertyId::ZIndex, &min).is_ok());

        let max = PropertyValue::Integer(999999);
        assert!(validator.validate(PropertyId::ZIndex, &max).is_ok());
    }

    #[test]
    fn test_text_align_enum_validation() {
        let validator = PropertyValidator::new();

        // Valid values
        let valid_values = vec!["start", "end", "center", "justify", "left", "right"];
        for value_str in valid_values {
            let value = PropertyValue::String(std::borrow::Cow::Borrowed(value_str));
            assert!(validator.validate(PropertyId::TextAlign, &value).is_ok());
        }

        // Invalid value
        let invalid = PropertyValue::String(std::borrow::Cow::Borrowed("invalid"));
        assert!(validator.validate(PropertyId::TextAlign, &invalid).is_err());
    }

    #[test]
    fn test_overflow_enum_validation() {
        let validator = PropertyValidator::new();

        // Valid values
        let valid_values = vec!["visible", "hidden", "scroll", "auto"];
        for value_str in valid_values {
            let value = PropertyValue::String(std::borrow::Cow::Borrowed(value_str));
            assert!(validator.validate(PropertyId::Overflow, &value).is_ok());
        }

        // Invalid value
        let invalid = PropertyValue::String(std::borrow::Cow::Borrowed("clip"));
        assert!(validator.validate(PropertyId::Overflow, &invalid).is_err());
    }

    #[test]
    fn test_border_style_enum_validation() {
        let validator = PropertyValidator::new();

        // Valid values
        let valid_values = vec![
            "none", "solid", "dashed", "dotted", "double", "groove", "ridge", "inset", "outset",
        ];
        for value_str in valid_values {
            let value = PropertyValue::String(std::borrow::Cow::Borrowed(value_str));
            assert!(validator.validate(PropertyId::BorderStyle, &value).is_ok());
            assert!(validator
                .validate(PropertyId::BorderTopStyle, &value)
                .is_ok());
            assert!(validator
                .validate(PropertyId::BorderBottomStyle, &value)
                .is_ok());
        }

        // Invalid value
        let invalid = PropertyValue::String(std::borrow::Cow::Borrowed("wavy"));
        assert!(validator
            .validate(PropertyId::BorderStyle, &invalid)
            .is_err());
    }

    #[test]
    fn test_break_enum_validation() {
        let validator = PropertyValidator::new();

        // Valid values
        let valid_values = vec!["auto", "column", "page", "even-page", "odd-page"];
        for value_str in valid_values {
            let value = PropertyValue::String(std::borrow::Cow::Borrowed(value_str));
            assert!(validator.validate(PropertyId::BreakBefore, &value).is_ok());
            assert!(validator.validate(PropertyId::BreakAfter, &value).is_ok());
        }

        // Invalid value
        let invalid = PropertyValue::String(std::borrow::Cow::Borrowed("always"));
        assert!(validator
            .validate(PropertyId::BreakBefore, &invalid)
            .is_err());
    }

    #[test]
    fn test_font_size_positive_validation() {
        let validator = PropertyValidator::new();

        // Valid font size
        let valid = PropertyValue::Length(Length::from_pt(12.0));
        assert!(validator.validate(PropertyId::FontSize, &valid).is_ok());

        // Invalid: negative
        let invalid = PropertyValue::Length(Length::from_pt(-5.0));
        assert!(validator.validate(PropertyId::FontSize, &invalid).is_err());

        // Invalid: zero
        let zero = PropertyValue::Length(Length::from_pt(0.0));
        assert!(validator.validate(PropertyId::FontSize, &zero).is_err());
    }

    #[test]
    fn test_line_height_validation() {
        let validator = PropertyValidator::new();

        // Valid: positive length
        let valid_length = PropertyValue::Length(Length::from_pt(14.0));
        assert!(validator
            .validate(PropertyId::LineHeight, &valid_length)
            .is_ok());

        // Valid: positive number
        let valid_number = PropertyValue::Number(1.5);
        assert!(validator
            .validate(PropertyId::LineHeight, &valid_number)
            .is_ok());

        // Valid: "normal"
        let normal = PropertyValue::String(std::borrow::Cow::Borrowed("normal"));
        assert!(validator.validate(PropertyId::LineHeight, &normal).is_ok());

        // Invalid: zero length
        let invalid_length = PropertyValue::Length(Length::from_pt(0.0));
        assert!(validator
            .validate(PropertyId::LineHeight, &invalid_length)
            .is_err());

        // Invalid: negative number
        let invalid_number = PropertyValue::Number(-1.0);
        assert!(validator
            .validate(PropertyId::LineHeight, &invalid_number)
            .is_err());
    }

    #[test]
    fn test_auto_inherit_none_always_valid() {
        let validator = PropertyValidator::new();

        // Auto, Inherit, and None should always pass validation
        assert!(validator
            .validate(PropertyId::Opacity, &PropertyValue::Auto)
            .is_ok());
        assert!(validator
            .validate(PropertyId::Opacity, &PropertyValue::Inherit)
            .is_ok());
        assert!(validator
            .validate(PropertyId::Opacity, &PropertyValue::None)
            .is_ok());

        assert!(validator
            .validate(PropertyId::ColumnCount, &PropertyValue::Auto)
            .is_ok());
        assert!(validator
            .validate(PropertyId::FontSize, &PropertyValue::Inherit)
            .is_ok());
    }

    #[test]
    fn test_mutual_exclusion_width_ipd() {
        let validator = PropertyValidator::new();

        let props = vec![
            (
                PropertyId::Width,
                PropertyValue::Length(Length::from_pt(100.0)),
            ),
            (
                PropertyId::InlineProgressionDimension,
                PropertyValue::Length(Length::from_pt(100.0)),
            ),
        ];

        let errors = validator.check_mutual_exclusion(&props);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].property, PropertyId::Width);
    }

    #[test]
    fn test_mutual_exclusion_height_bpd() {
        let validator = PropertyValidator::new();

        let props = vec![
            (
                PropertyId::Height,
                PropertyValue::Length(Length::from_pt(200.0)),
            ),
            (
                PropertyId::BlockProgressionDimension,
                PropertyValue::Length(Length::from_pt(200.0)),
            ),
        ];

        let errors = validator.check_mutual_exclusion(&props);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].property, PropertyId::Height);
    }

    #[test]
    fn test_no_mutual_exclusion() {
        let validator = PropertyValidator::new();

        let props = vec![
            (
                PropertyId::Width,
                PropertyValue::Length(Length::from_pt(100.0)),
            ),
            (
                PropertyId::Height,
                PropertyValue::Length(Length::from_pt(200.0)),
            ),
        ];

        let errors = validator.check_mutual_exclusion(&props);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_validation_error_to_fop_error() {
        let error = ValidationError::new(
            PropertyId::Opacity,
            "Value out of range".to_string(),
            Some("Use 0.0-1.0".to_string()),
        );

        let value = PropertyValue::Number(1.5);
        let fop_error = error.to_fop_error(&value);

        match fop_error {
            FopError::PropertyValidation {
                property,
                value: _,
                reason,
            } => {
                assert_eq!(property, "opacity");
                assert!(reason.contains("Value out of range"));
                assert!(reason.contains("Use 0.0-1.0"));
            }
            _ => panic!("Expected PropertyValidation error"),
        }
    }

    #[test]
    fn test_length_range_validation() {
        let value = PropertyValue::Length(Length::from_pt(50.0));
        assert!(
            PropertyValidator::validate_length_range(PropertyId::FontSize, &value, 0.0, 100.0)
                .is_ok()
        );

        assert!(PropertyValidator::validate_length_range(
            PropertyId::FontSize,
            &value,
            60.0,
            100.0
        )
        .is_err());
    }

    #[test]
    fn test_number_range_validation() {
        let value = PropertyValue::Number(0.5);
        assert!(
            PropertyValidator::validate_number_range(PropertyId::Opacity, &value, 0.0, 1.0).is_ok()
        );

        assert!(
            PropertyValidator::validate_number_range(PropertyId::Opacity, &value, 0.6, 1.0)
                .is_err()
        );
    }

    #[test]
    fn test_percentage_range_validation() {
        let value = PropertyValue::Percentage(Percentage::new(0.5)); // 50%
        assert!(
            PropertyValidator::validate_percentage_range(PropertyId::Width, &value, 0.0, 1.0)
                .is_ok()
        );

        assert!(
            PropertyValidator::validate_percentage_range(PropertyId::Width, &value, 0.6, 1.0)
                .is_err()
        );
    }

    #[test]
    fn test_enum_validation_legacy() {
        let value = PropertyValue::Enum(1);
        let allowed = vec![1, 2, 3];
        assert!(PropertyValidator::validate_enum(PropertyId::FontStyle, &value, &allowed).is_ok());

        let invalid = PropertyValue::Enum(5);
        assert!(
            PropertyValidator::validate_enum(PropertyId::FontStyle, &invalid, &allowed).is_err()
        );
    }

    #[test]
    fn test_validator_instance_vs_static() {
        // Test that both instance and static methods work
        let validator = PropertyValidator::new();

        let valid_opacity = PropertyValue::Number(0.5);
        assert!(validator
            .validate(PropertyId::Opacity, &valid_opacity)
            .is_ok());
        assert!(PropertyValidator::validate_static(PropertyId::Opacity, &valid_opacity).is_ok());

        let invalid_opacity = PropertyValue::Number(1.5);
        assert!(validator
            .validate(PropertyId::Opacity, &invalid_opacity)
            .is_err());
        assert!(PropertyValidator::validate_static(PropertyId::Opacity, &invalid_opacity).is_err());
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;
    use fop_types::{Length, Percentage};

    // -----------------------------------------------------------------------
    // validate_length_range
    // -----------------------------------------------------------------------

    #[test]
    fn test_length_range_exactly_at_min() {
        let v = PropertyValue::Length(Length::from_pt(0.0));
        assert!(
            PropertyValidator::validate_length_range(PropertyId::Width, &v, 0.0, 100.0).is_ok()
        );
    }

    #[test]
    fn test_length_range_exactly_at_max() {
        let v = PropertyValue::Length(Length::from_pt(100.0));
        assert!(
            PropertyValidator::validate_length_range(PropertyId::Width, &v, 0.0, 100.0).is_ok()
        );
    }

    #[test]
    fn test_length_range_below_min() {
        let v = PropertyValue::Length(Length::from_pt(-1.0));
        assert!(
            PropertyValidator::validate_length_range(PropertyId::Width, &v, 0.0, 100.0).is_err()
        );
    }

    #[test]
    fn test_length_range_above_max() {
        let v = PropertyValue::Length(Length::from_pt(101.0));
        assert!(
            PropertyValidator::validate_length_range(PropertyId::Width, &v, 0.0, 100.0).is_err()
        );
    }

    #[test]
    fn test_length_range_auto_passes() {
        assert!(PropertyValidator::validate_length_range(
            PropertyId::Width,
            &PropertyValue::Auto,
            0.0,
            100.0
        )
        .is_ok());
    }

    #[test]
    fn test_length_range_inherit_passes() {
        assert!(PropertyValidator::validate_length_range(
            PropertyId::Width,
            &PropertyValue::Inherit,
            0.0,
            100.0
        )
        .is_ok());
    }

    #[test]
    fn test_length_range_none_passes() {
        assert!(PropertyValidator::validate_length_range(
            PropertyId::Width,
            &PropertyValue::None,
            0.0,
            100.0
        )
        .is_ok());
    }

    #[test]
    fn test_length_range_rejects_non_length() {
        let v = PropertyValue::Number(50.0);
        assert!(
            PropertyValidator::validate_length_range(PropertyId::Width, &v, 0.0, 100.0).is_err()
        );
    }

    // -----------------------------------------------------------------------
    // validate_number_range
    // -----------------------------------------------------------------------

    #[test]
    fn test_number_range_exactly_at_boundaries() {
        assert!(PropertyValidator::validate_number_range(
            PropertyId::Opacity,
            &PropertyValue::Number(0.0),
            0.0,
            1.0
        )
        .is_ok());
        assert!(PropertyValidator::validate_number_range(
            PropertyId::Opacity,
            &PropertyValue::Number(1.0),
            0.0,
            1.0
        )
        .is_ok());
    }

    #[test]
    fn test_number_range_outside_boundaries() {
        assert!(PropertyValidator::validate_number_range(
            PropertyId::Opacity,
            &PropertyValue::Number(-0.001),
            0.0,
            1.0
        )
        .is_err());
        assert!(PropertyValidator::validate_number_range(
            PropertyId::Opacity,
            &PropertyValue::Number(1.001),
            0.0,
            1.0
        )
        .is_err());
    }

    #[test]
    fn test_number_range_integer_value() {
        // Integer values are accepted as numbers
        let v = PropertyValue::Integer(1);
        assert!(
            PropertyValidator::validate_number_range(PropertyId::Opacity, &v, 0.0, 2.0).is_ok()
        );
    }

    #[test]
    fn test_number_range_auto_passes() {
        assert!(PropertyValidator::validate_number_range(
            PropertyId::Opacity,
            &PropertyValue::Auto,
            0.0,
            1.0
        )
        .is_ok());
    }

    #[test]
    fn test_number_range_rejects_string() {
        let v = PropertyValue::String(std::borrow::Cow::Borrowed("0.5"));
        assert!(
            PropertyValidator::validate_number_range(PropertyId::Opacity, &v, 0.0, 1.0).is_err()
        );
    }

    // -----------------------------------------------------------------------
    // validate_percentage_range
    // -----------------------------------------------------------------------

    #[test]
    fn test_percentage_range_valid() {
        let v = PropertyValue::Percentage(Percentage::new(0.5)); // 50%
        assert!(
            PropertyValidator::validate_percentage_range(PropertyId::Width, &v, 0.0, 1.0).is_ok()
        );
    }

    #[test]
    fn test_percentage_range_too_high() {
        let v = PropertyValue::Percentage(Percentage::new(1.5)); // 150%
        assert!(
            PropertyValidator::validate_percentage_range(PropertyId::Width, &v, 0.0, 1.0).is_err()
        );
    }

    #[test]
    fn test_percentage_range_too_low() {
        let v = PropertyValue::Percentage(Percentage::new(-0.1));
        assert!(
            PropertyValidator::validate_percentage_range(PropertyId::Width, &v, 0.0, 1.0).is_err()
        );
    }

    #[test]
    fn test_percentage_range_auto_passes() {
        assert!(PropertyValidator::validate_percentage_range(
            PropertyId::Width,
            &PropertyValue::Auto,
            0.0,
            1.0
        )
        .is_ok());
    }

    // -----------------------------------------------------------------------
    // validate_enum (legacy u16 interface)
    // -----------------------------------------------------------------------

    #[test]
    fn test_enum_valid_value() {
        let v = PropertyValue::Enum(2);
        assert!(PropertyValidator::validate_enum(PropertyId::FontStyle, &v, &[1, 2, 3]).is_ok());
    }

    #[test]
    fn test_enum_invalid_value() {
        let v = PropertyValue::Enum(99);
        assert!(PropertyValidator::validate_enum(PropertyId::FontStyle, &v, &[1, 2, 3]).is_err());
    }

    #[test]
    fn test_enum_wrong_type_rejected() {
        let v = PropertyValue::String(std::borrow::Cow::Borrowed("normal"));
        assert!(PropertyValidator::validate_enum(PropertyId::FontStyle, &v, &[1, 2, 3]).is_err());
    }

    // -----------------------------------------------------------------------
    // validate()/validate_static() for font-size positive-length
    // -----------------------------------------------------------------------

    #[test]
    fn test_font_size_mm_valid() {
        let v = PropertyValue::Length(Length::from_mm(5.0));
        let validator = PropertyValidator::new();
        assert!(validator.validate(PropertyId::FontSize, &v).is_ok());
    }

    #[test]
    fn test_font_size_negative_rejected() {
        let v = PropertyValue::Length(Length::from_pt(-1.0));
        assert!(PropertyValidator::validate_static(PropertyId::FontSize, &v).is_err());
    }

    #[test]
    fn test_font_size_zero_rejected() {
        let v = PropertyValue::Length(Length::from_pt(0.0));
        assert!(PropertyValidator::validate_static(PropertyId::FontSize, &v).is_err());
    }

    #[test]
    fn test_font_size_percentage_passes() {
        let v = PropertyValue::Percentage(Percentage::new(1.2)); // 120%
        let validator = PropertyValidator::new();
        assert!(validator.validate(PropertyId::FontSize, &v).is_ok());
    }

    #[test]
    fn test_font_size_inherit_passes() {
        let validator = PropertyValidator::new();
        assert!(validator
            .validate(PropertyId::FontSize, &PropertyValue::Inherit)
            .is_ok());
    }

    // -----------------------------------------------------------------------
    // line-height validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_line_height_zero_length_rejected() {
        let v = PropertyValue::Length(Length::from_pt(0.0));
        let validator = PropertyValidator::new();
        assert!(validator.validate(PropertyId::LineHeight, &v).is_err());
    }

    #[test]
    fn test_line_height_negative_length_rejected() {
        let v = PropertyValue::Length(Length::from_pt(-5.0));
        let validator = PropertyValidator::new();
        assert!(validator.validate(PropertyId::LineHeight, &v).is_err());
    }

    #[test]
    fn test_line_height_zero_number_rejected() {
        let v = PropertyValue::Number(0.0);
        let validator = PropertyValidator::new();
        assert!(validator.validate(PropertyId::LineHeight, &v).is_err());
    }

    #[test]
    fn test_line_height_negative_number_rejected() {
        let v = PropertyValue::Number(-1.5);
        let validator = PropertyValidator::new();
        assert!(validator.validate(PropertyId::LineHeight, &v).is_err());
    }

    #[test]
    fn test_line_height_percentage_valid() {
        let v = PropertyValue::Percentage(Percentage::new(1.5)); // 150%
        let validator = PropertyValidator::new();
        assert!(validator.validate(PropertyId::LineHeight, &v).is_ok());
    }

    #[test]
    fn test_line_height_invalid_string() {
        let v = PropertyValue::String(std::borrow::Cow::Borrowed("tight"));
        let validator = PropertyValidator::new();
        assert!(validator.validate(PropertyId::LineHeight, &v).is_err());
    }

    // -----------------------------------------------------------------------
    // opacity range via validate_static
    // -----------------------------------------------------------------------

    #[test]
    fn test_static_opacity_edge_values() {
        assert!(PropertyValidator::validate_static(
            PropertyId::Opacity,
            &PropertyValue::Number(0.0)
        )
        .is_ok());
        assert!(PropertyValidator::validate_static(
            PropertyId::Opacity,
            &PropertyValue::Number(1.0)
        )
        .is_ok());
    }

    #[test]
    fn test_static_opacity_out_of_range() {
        assert!(PropertyValidator::validate_static(
            PropertyId::Opacity,
            &PropertyValue::Number(1.01)
        )
        .is_err());
        assert!(PropertyValidator::validate_static(
            PropertyId::Opacity,
            &PropertyValue::Number(-0.01)
        )
        .is_err());
    }

    // -----------------------------------------------------------------------
    // column-count via validate_static
    // -----------------------------------------------------------------------

    #[test]
    fn test_static_column_count_boundaries() {
        assert!(PropertyValidator::validate_static(
            PropertyId::ColumnCount,
            &PropertyValue::Integer(1)
        )
        .is_ok());
        assert!(PropertyValidator::validate_static(
            PropertyId::ColumnCount,
            &PropertyValue::Integer(255)
        )
        .is_ok());
        assert!(PropertyValidator::validate_static(
            PropertyId::ColumnCount,
            &PropertyValue::Integer(0)
        )
        .is_err());
        assert!(PropertyValidator::validate_static(
            PropertyId::ColumnCount,
            &PropertyValue::Integer(256)
        )
        .is_err());
    }

    // -----------------------------------------------------------------------
    // widows / orphans via validate_static
    // -----------------------------------------------------------------------

    #[test]
    fn test_static_widows_orphans_boundaries() {
        assert!(
            PropertyValidator::validate_static(PropertyId::Widows, &PropertyValue::Integer(1))
                .is_ok()
        );
        assert!(PropertyValidator::validate_static(
            PropertyId::Widows,
            &PropertyValue::Integer(999)
        )
        .is_ok());
        assert!(
            PropertyValidator::validate_static(PropertyId::Widows, &PropertyValue::Integer(0))
                .is_err()
        );
        assert!(PropertyValidator::validate_static(
            PropertyId::Orphans,
            &PropertyValue::Integer(1000)
        )
        .is_err());
    }

    // -----------------------------------------------------------------------
    // z-index via validate_static
    // -----------------------------------------------------------------------

    #[test]
    fn test_static_z_index_boundaries() {
        assert!(PropertyValidator::validate_static(
            PropertyId::ZIndex,
            &PropertyValue::Integer(-999999)
        )
        .is_ok());
        assert!(PropertyValidator::validate_static(
            PropertyId::ZIndex,
            &PropertyValue::Integer(999999)
        )
        .is_ok());
        assert!(PropertyValidator::validate_static(
            PropertyId::ZIndex,
            &PropertyValue::Integer(1_000_000)
        )
        .is_err());
        assert!(PropertyValidator::validate_static(
            PropertyId::ZIndex,
            &PropertyValue::Integer(-1_000_000)
        )
        .is_err());
    }

    // -----------------------------------------------------------------------
    // ValidationError struct
    // -----------------------------------------------------------------------

    #[test]
    fn test_validation_error_no_suggestion() {
        let err = ValidationError::new(PropertyId::Opacity, "Bad value".to_string(), None);
        let v = PropertyValue::Number(99.0);
        let fop_err = err.to_fop_error(&v);
        match fop_err {
            FopError::PropertyValidation { reason, .. } => {
                assert!(reason.contains("Bad value"));
                assert!(!reason.contains("Suggestion:"));
            }
            _ => panic!("Expected PropertyValidation error"),
        }
    }

    #[test]
    fn test_validation_error_with_suggestion() {
        let err = ValidationError::new(
            PropertyId::Opacity,
            "Out of range".to_string(),
            Some("Use 0.0 to 1.0".to_string()),
        );
        let v = PropertyValue::Number(99.0);
        let fop_err = err.to_fop_error(&v);
        match fop_err {
            FopError::PropertyValidation { reason, .. } => {
                assert!(reason.contains("Out of range"));
                assert!(reason.contains("Use 0.0 to 1.0"));
            }
            _ => panic!("Expected PropertyValidation error"),
        }
    }

    #[test]
    fn test_validation_error_equality() {
        let e1 = ValidationError::new(PropertyId::Opacity, "msg".to_string(), None);
        let e2 = ValidationError::new(PropertyId::Opacity, "msg".to_string(), None);
        let e3 = ValidationError::new(PropertyId::Width, "msg".to_string(), None);
        assert_eq!(e1, e2);
        assert_ne!(e1, e3);
    }

    // -----------------------------------------------------------------------
    // ValidationRule enum
    // -----------------------------------------------------------------------

    #[test]
    fn test_validation_rule_range_clone() {
        let rule = ValidationRule::Range { min: 0.0, max: 1.0 };
        let cloned = rule.clone();
        assert_eq!(rule, cloned);
    }

    #[test]
    fn test_validation_rule_string_enum_clone() {
        let rule = ValidationRule::StringEnum {
            allowed: vec!["a", "b"],
        };
        let cloned = rule.clone();
        assert_eq!(rule, cloned);
    }

    // -----------------------------------------------------------------------
    // PropertyValidator with custom rules
    // -----------------------------------------------------------------------

    #[test]
    fn test_custom_rule_overrides_for_arbitrary_property() {
        let mut validator = PropertyValidator::new();
        // Add a range rule for Width: 1pt - 500pt
        validator.add_rule(
            PropertyId::Width,
            ValidationRule::Range {
                min: 1.0,
                max: 500.0,
            },
        );
        assert!(validator
            .validate(
                PropertyId::Width,
                &PropertyValue::Length(Length::from_pt(100.0))
            )
            .is_ok());
        assert!(validator
            .validate(
                PropertyId::Width,
                &PropertyValue::Length(Length::from_pt(0.0))
            )
            .is_err());
    }

    #[test]
    fn test_unknown_property_passes_with_any_value() {
        let validator = PropertyValidator::new();
        // A property with no registered rules always passes validation
        let v = PropertyValue::Length(Length::from_pt(42.0));
        // SpaceBefore has no registered range rule
        assert!(validator.validate(PropertyId::SpaceBefore, &v).is_ok());
    }

    // -----------------------------------------------------------------------
    // mutual exclusion: both absent
    // -----------------------------------------------------------------------

    #[test]
    fn test_no_mutual_exclusion_when_neither_present() {
        let validator = PropertyValidator::new();
        let props: Vec<(PropertyId, PropertyValue)> = vec![];
        let errors = validator.check_mutual_exclusion(&props);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_mutual_exclusion_only_width() {
        let validator = PropertyValidator::new();
        let props = vec![(
            PropertyId::Width,
            PropertyValue::Length(Length::from_pt(100.0)),
        )];
        let errors = validator.check_mutual_exclusion(&props);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_mutual_exclusion_only_height() {
        let validator = PropertyValidator::new();
        let props = vec![(
            PropertyId::Height,
            PropertyValue::Length(Length::from_pt(100.0)),
        )];
        let errors = validator.check_mutual_exclusion(&props);
        assert!(errors.is_empty());
    }
}
