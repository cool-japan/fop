//! Expression type for CSS calc() expressions in property values
//!
//! This module implements mathematical expressions that can be used in CSS calc()
//! functions, supporting mixed units, nested expressions, and basic arithmetic.

use crate::{FopError, Length, Percentage, Result};
use std::fmt;

/// Mathematical expression for CSS calc() values
///
/// Supports:
/// - Literals (absolute lengths)
/// - Percentages (relative to base value)
/// - Binary operations: +, -, *, /
/// - Nested expressions
///
/// # Examples
///
/// ```
/// use fop_types::{Expression, Length, Percentage, expression::EvalContext};
///
/// // Parse calc(100% - 20pt)
/// let expr = Expression::parse("calc(100% - 20pt)").unwrap();
///
/// // Evaluate with context
/// let ctx = EvalContext {
///     base_width: Some(Length::from_pt(200.0)),
///     base_height: None,
///     font_size: Length::from_pt(12.0),
/// };
/// let result = expr.evaluate(&ctx).unwrap();
/// assert_eq!(result, Length::from_pt(180.0));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// A literal length value
    Literal(Length),

    /// A percentage value (needs base to resolve)
    Percentage(Percentage),

    /// Addition of two expressions
    Add(Box<Expression>, Box<Expression>),

    /// Subtraction of two expressions
    Sub(Box<Expression>, Box<Expression>),

    /// Multiplication of expression by scalar
    Mul(Box<Expression>, f64),

    /// Division of expression by scalar
    Div(Box<Expression>, f64),
}

/// Context for evaluating expressions
///
/// Provides base values needed to resolve percentages and other
/// context-dependent expressions.
#[derive(Debug, Clone, Copy)]
pub struct EvalContext {
    /// Base width for percentage resolution
    pub base_width: Option<Length>,

    /// Base height for percentage resolution
    pub base_height: Option<Length>,

    /// Current font size
    pub font_size: Length,
}

impl EvalContext {
    /// Create a new evaluation context
    #[inline]
    #[must_use = "this returns a new value without modifying anything"]
    pub const fn new(
        base_width: Option<Length>,
        base_height: Option<Length>,
        font_size: Length,
    ) -> Self {
        Self {
            base_width,
            base_height,
            font_size,
        }
    }

    /// Create a context with only width
    #[inline]
    #[must_use = "this returns a new value without modifying anything"]
    pub const fn with_width(base_width: Length, font_size: Length) -> Self {
        Self {
            base_width: Some(base_width),
            base_height: None,
            font_size,
        }
    }

    /// Create a context with only height
    #[inline]
    #[must_use = "this returns a new value without modifying anything"]
    pub const fn with_height(base_height: Length, font_size: Length) -> Self {
        Self {
            base_width: None,
            base_height: Some(base_height),
            font_size,
        }
    }
}

impl Expression {
    /// Parse a calc() expression from a string
    ///
    /// # Examples
    ///
    /// ```
    /// use fop_types::Expression;
    ///
    /// let expr = Expression::parse("calc(100% - 20pt)").unwrap();
    /// let expr2 = Expression::parse("calc(50% + 10mm)").unwrap();
    /// let expr3 = Expression::parse("calc((100% - 40pt) / 2)").unwrap();
    /// ```
    pub fn parse(input: &str) -> Result<Self> {
        let input = input.trim();

        // Check for calc() wrapper
        if let Some(stripped) = input.strip_prefix("calc(") {
            if let Some(content) = stripped.strip_suffix(')') {
                return Self::parse_expression(content.trim());
            }
        }

        Err(FopError::ParseError(format!(
            "Expression must be wrapped in calc(): {}",
            input
        )))
    }

    /// Parse an expression (internal)
    fn parse_expression(input: &str) -> Result<Self> {
        // Try to parse as addition/subtraction first (lowest precedence)
        if let Some(pos) = Self::find_operator(input, &['+', '-']) {
            let op = input.chars().nth(pos).ok_or_else(|| {
                FopError::ParseError(format!(
                    "Operator not found at position {} in: {}",
                    pos, input
                ))
            })?;
            let left = Self::parse_expression(input[..pos].trim())?;
            let right = Self::parse_expression(input[pos + 1..].trim())?;

            return match op {
                '+' => Ok(Expression::Add(Box::new(left), Box::new(right))),
                '-' => Ok(Expression::Sub(Box::new(left), Box::new(right))),
                _ => unreachable!(),
            };
        }

        // Try to parse as multiplication/division (higher precedence)
        if let Some(pos) = Self::find_operator(input, &['*', '/']) {
            let op = input.chars().nth(pos).ok_or_else(|| {
                FopError::ParseError(format!(
                    "Operator not found at position {} in: {}",
                    pos, input
                ))
            })?;
            let left = Self::parse_expression(input[..pos].trim())?;
            let right_str = input[pos + 1..].trim();

            // Right side of * or / must be a number
            let scalar = right_str.parse::<f64>().map_err(|_| {
                FopError::ParseError(format!(
                    "Right side of {} must be a number: {}",
                    op, right_str
                ))
            })?;

            return match op {
                '*' => Ok(Expression::Mul(Box::new(left), scalar)),
                '/' => {
                    if scalar == 0.0 {
                        Err(FopError::ParseError("Division by zero".to_string()))
                    } else {
                        Ok(Expression::Div(Box::new(left), scalar))
                    }
                }
                _ => unreachable!(),
            };
        }

        // Try to parse parenthesized expression
        if let Some(stripped) = input.strip_prefix('(') {
            if let Some(content) = stripped.strip_suffix(')') {
                return Self::parse_expression(content.trim());
            }
        }

        // Try to parse as percentage
        if input.ends_with('%') {
            let pct_str = input.trim_end_matches('%').trim();
            let pct_value = pct_str
                .parse::<f64>()
                .map_err(|_| FopError::ParseError(format!("Invalid percentage: {}", input)))?;
            return Ok(Expression::Percentage(Percentage::from_percent(pct_value)));
        }

        // Try to parse as length
        Self::parse_length(input)
    }

    /// Find the position of an operator at the top level (not in parentheses)
    fn find_operator(input: &str, operators: &[char]) -> Option<usize> {
        let mut depth = 0;
        let mut last_op_pos = None;

        for (i, c) in input.chars().enumerate() {
            match c {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ if depth == 0 && operators.contains(&c) => {
                    last_op_pos = Some(i);
                }
                _ => {}
            }
        }

        last_op_pos
    }

    /// Parse a length value with units
    fn parse_length(input: &str) -> Result<Self> {
        let input = input.trim();

        // Find where the number ends and the unit begins
        let mut split_pos = 0;
        for (i, c) in input.chars().enumerate() {
            if !c.is_numeric() && c != '.' && c != '-' && c != '+' {
                split_pos = i;
                break;
            }
        }

        if split_pos == 0 {
            return Err(FopError::ParseError(format!("Invalid length: {}", input)));
        }

        let value_str = &input[..split_pos];
        let unit = &input[split_pos..];

        let value = value_str
            .parse::<f64>()
            .map_err(|_| FopError::ParseError(format!("Invalid number: {}", value_str)))?;

        let length = match unit {
            "pt" => Length::from_pt(value),
            "mm" => Length::from_mm(value),
            "cm" => Length::from_cm(value),
            "in" => Length::from_inch(value),
            _ => return Err(FopError::ParseError(format!("Unknown unit: {}", unit))),
        };

        Ok(Expression::Literal(length))
    }

    /// Evaluate the expression to a concrete length
    ///
    /// # Examples
    ///
    /// ```
    /// use fop_types::{Expression, Length, expression::EvalContext};
    ///
    /// let expr = Expression::parse("calc(100% - 20pt)").unwrap();
    /// let ctx = EvalContext::with_width(Length::from_pt(200.0), Length::from_pt(12.0));
    /// let result = expr.evaluate(&ctx).unwrap();
    /// assert_eq!(result, Length::from_pt(180.0));
    /// ```
    pub fn evaluate(&self, context: &EvalContext) -> Result<Length> {
        match self {
            Expression::Literal(len) => Ok(*len),

            Expression::Percentage(pct) => {
                // Use base_width as the default base for percentage resolution
                let base = context.base_width.ok_or_else(|| {
                    FopError::Generic("No base value available for percentage".to_string())
                })?;
                Ok(pct.of(base))
            }

            Expression::Add(left, right) => {
                let left_val = left.evaluate(context)?;
                let right_val = right.evaluate(context)?;
                Ok(left_val + right_val)
            }

            Expression::Sub(left, right) => {
                let left_val = left.evaluate(context)?;
                let right_val = right.evaluate(context)?;
                Ok(left_val - right_val)
            }

            Expression::Mul(expr, scalar) => {
                let val = expr.evaluate(context)?;
                let millipoints = (val.millipoints() as f64 * scalar).round() as i32;
                Ok(Length::from_millipoints(millipoints))
            }

            Expression::Div(expr, scalar) => {
                let val = expr.evaluate(context)?;
                let millipoints = (val.millipoints() as f64 / scalar).round() as i32;
                Ok(Length::from_millipoints(millipoints))
            }
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Literal(len) => write!(f, "{}", len),
            Expression::Percentage(pct) => write!(f, "{}", pct),
            Expression::Add(left, right) => write!(f, "({} + {})", left, right),
            Expression::Sub(left, right) => write!(f, "({} - {})", left, right),
            Expression::Mul(expr, scalar) => write!(f, "({} * {})", expr, scalar),
            Expression::Div(expr, scalar) => write!(f, "({} / {})", expr, scalar),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_literal() {
        let expr = Expression::parse("calc(20pt)").expect("test: should succeed");
        assert_eq!(expr, Expression::Literal(Length::from_pt(20.0)));
    }

    #[test]
    fn test_parse_percentage() {
        let expr = Expression::parse("calc(50%)").expect("test: should succeed");
        assert_eq!(expr, Expression::Percentage(Percentage::from_percent(50.0)));
    }

    #[test]
    fn test_parse_addition() {
        let expr = Expression::parse("calc(50% + 10pt)").expect("test: should succeed");
        match expr {
            Expression::Add(left, right) => {
                assert_eq!(
                    *left,
                    Expression::Percentage(Percentage::from_percent(50.0))
                );
                assert_eq!(*right, Expression::Literal(Length::from_pt(10.0)));
            }
            _ => panic!("Expected Add expression"),
        }
    }

    #[test]
    fn test_parse_subtraction() {
        let expr = Expression::parse("calc(100% - 20pt)").expect("test: should succeed");
        match expr {
            Expression::Sub(left, right) => {
                assert_eq!(
                    *left,
                    Expression::Percentage(Percentage::from_percent(100.0))
                );
                assert_eq!(*right, Expression::Literal(Length::from_pt(20.0)));
            }
            _ => panic!("Expected Sub expression"),
        }
    }

    #[test]
    fn test_parse_multiplication() {
        let expr = Expression::parse("calc(50% * 2)").expect("test: should succeed");
        match expr {
            Expression::Mul(inner, scalar) => {
                assert_eq!(
                    *inner,
                    Expression::Percentage(Percentage::from_percent(50.0))
                );
                assert!((scalar - 2.0).abs() < 0.001);
            }
            _ => panic!("Expected Mul expression"),
        }
    }

    #[test]
    fn test_parse_division() {
        let expr = Expression::parse("calc(100% / 3)").expect("test: should succeed");
        match expr {
            Expression::Div(inner, scalar) => {
                assert_eq!(
                    *inner,
                    Expression::Percentage(Percentage::from_percent(100.0))
                );
                assert!((scalar - 3.0).abs() < 0.001);
            }
            _ => panic!("Expected Div expression"),
        }
    }

    #[test]
    fn test_parse_nested_expression() {
        let expr = Expression::parse("calc((100% - 40pt) / 2)").expect("test: should succeed");
        match expr {
            Expression::Div(inner, scalar) => {
                assert!((scalar - 2.0).abs() < 0.001);
                match *inner {
                    Expression::Sub(left, right) => {
                        assert_eq!(
                            *left,
                            Expression::Percentage(Percentage::from_percent(100.0))
                        );
                        assert_eq!(*right, Expression::Literal(Length::from_pt(40.0)));
                    }
                    _ => panic!("Expected Sub expression inside Div"),
                }
            }
            _ => panic!("Expected Div expression"),
        }
    }

    #[test]
    fn test_parse_mixed_units() {
        let expr = Expression::parse("calc(100% - 10mm)").expect("test: should succeed");
        match expr {
            Expression::Sub(left, right) => {
                assert_eq!(
                    *left,
                    Expression::Percentage(Percentage::from_percent(100.0))
                );
                assert_eq!(*right, Expression::Literal(Length::from_mm(10.0)));
            }
            _ => panic!("Expected Sub expression"),
        }
    }

    #[test]
    fn test_parse_invalid_no_calc() {
        let result = Expression::parse("50%");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_division_by_zero() {
        let result = Expression::parse("calc(100% / 0)");
        assert!(result.is_err());
    }

    #[test]
    fn test_evaluate_literal() {
        let expr = Expression::Literal(Length::from_pt(20.0));
        let ctx = EvalContext::with_width(Length::from_pt(100.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        assert_eq!(result, Length::from_pt(20.0));
    }

    #[test]
    fn test_evaluate_percentage() {
        let expr = Expression::Percentage(Percentage::from_percent(50.0));
        let ctx = EvalContext::with_width(Length::from_pt(100.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        assert_eq!(result, Length::from_pt(50.0));
    }

    #[test]
    fn test_evaluate_addition() {
        let expr = Expression::parse("calc(50% + 10pt)").expect("test: should succeed");
        let ctx = EvalContext::with_width(Length::from_pt(100.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        // 50% of 100pt + 10pt = 50pt + 10pt = 60pt
        assert_eq!(result, Length::from_pt(60.0));
    }

    #[test]
    fn test_evaluate_subtraction() {
        let expr = Expression::parse("calc(100% - 20pt)").expect("test: should succeed");
        let ctx = EvalContext::with_width(Length::from_pt(200.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        // 100% of 200pt - 20pt = 200pt - 20pt = 180pt
        assert_eq!(result, Length::from_pt(180.0));
    }

    #[test]
    fn test_evaluate_multiplication() {
        let expr = Expression::parse("calc(50pt * 2)").expect("test: should succeed");
        let ctx = EvalContext::with_width(Length::from_pt(100.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        assert_eq!(result, Length::from_pt(100.0));
    }

    #[test]
    fn test_evaluate_division() {
        let expr = Expression::parse("calc(100pt / 3)").expect("test: should succeed");
        let ctx = EvalContext::with_width(Length::from_pt(200.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        assert!((result.to_pt() - 33.333).abs() < 0.01);
    }

    #[test]
    fn test_evaluate_nested() {
        let expr = Expression::parse("calc((100% - 40pt) / 2)").expect("test: should succeed");
        let ctx = EvalContext::with_width(Length::from_pt(200.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        // (100% of 200pt - 40pt) / 2 = (200pt - 40pt) / 2 = 160pt / 2 = 80pt
        assert_eq!(result, Length::from_pt(80.0));
    }

    #[test]
    fn test_evaluate_complex_nested() {
        let expr = Expression::parse("calc(50% + (20pt * 2))").expect("test: should succeed");
        let ctx = EvalContext::with_width(Length::from_pt(100.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        // 50% of 100pt + (20pt * 2) = 50pt + 40pt = 90pt
        assert_eq!(result, Length::from_pt(90.0));
    }

    #[test]
    fn test_evaluate_percentage_no_base() {
        let expr = Expression::Percentage(Percentage::from_percent(50.0));
        let ctx = EvalContext::new(None, None, Length::from_pt(12.0));
        let result = expr.evaluate(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_display() {
        let expr = Expression::parse("calc(100% - 20pt)").expect("test: should succeed");
        let display = format!("{}", expr);
        assert!(display.contains("100%"));
        assert!(display.contains("20pt"));
    }

    #[test]
    fn test_parse_millimeters() {
        let expr = Expression::parse("calc(10mm)").expect("test: should succeed");
        assert_eq!(expr, Expression::Literal(Length::from_mm(10.0)));
    }

    #[test]
    fn test_parse_centimeters() {
        let expr = Expression::parse("calc(2.5cm)").expect("test: should succeed");
        assert_eq!(expr, Expression::Literal(Length::from_cm(2.5)));
    }

    #[test]
    fn test_parse_inches() {
        let expr = Expression::parse("calc(1in)").expect("test: should succeed");
        assert_eq!(expr, Expression::Literal(Length::from_inch(1.0)));
    }
}

#[cfg(test)]
mod expression_extra_tests {
    use super::*;

    // --- EvalContext constructors ---

    #[test]
    fn test_eval_context_new() {
        let ctx = EvalContext::new(
            Some(Length::from_pt(100.0)),
            Some(Length::from_pt(200.0)),
            Length::from_pt(12.0),
        );
        assert_eq!(ctx.base_width, Some(Length::from_pt(100.0)));
        assert_eq!(ctx.base_height, Some(Length::from_pt(200.0)));
        assert_eq!(ctx.font_size, Length::from_pt(12.0));
    }

    #[test]
    fn test_eval_context_with_width() {
        let ctx = EvalContext::with_width(Length::from_pt(400.0), Length::from_pt(10.0));
        assert_eq!(ctx.base_width, Some(Length::from_pt(400.0)));
        assert!(ctx.base_height.is_none());
    }

    #[test]
    fn test_eval_context_with_height() {
        let ctx = EvalContext::with_height(Length::from_pt(300.0), Length::from_pt(10.0));
        assert!(ctx.base_width.is_none());
        assert_eq!(ctx.base_height, Some(Length::from_pt(300.0)));
    }

    #[test]
    fn test_eval_context_no_base() {
        let ctx = EvalContext::new(None, None, Length::from_pt(12.0));
        assert!(ctx.base_width.is_none());
        assert!(ctx.base_height.is_none());
    }

    // --- Expression variants ---

    #[test]
    fn test_literal_expression_evaluate() {
        let expr = Expression::Literal(Length::from_mm(10.0));
        let ctx = EvalContext::with_width(Length::from_pt(100.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        assert!((result.to_mm() - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_percentage_expression_25_percent() {
        let expr = Expression::Percentage(Percentage::from_percent(25.0));
        let ctx = EvalContext::with_width(Length::from_pt(200.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        assert!((result.to_pt() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_percentage_no_base_errors() {
        let expr = Expression::Percentage(Percentage::from_percent(50.0));
        let ctx = EvalContext::new(None, None, Length::from_pt(12.0));
        assert!(expr.evaluate(&ctx).is_err());
    }

    #[test]
    fn test_add_two_literals() {
        let expr = Expression::Add(
            Box::new(Expression::Literal(Length::from_pt(30.0))),
            Box::new(Expression::Literal(Length::from_pt(20.0))),
        );
        let ctx = EvalContext::with_width(Length::from_pt(100.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        assert!((result.to_pt() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_sub_two_literals() {
        let expr = Expression::Sub(
            Box::new(Expression::Literal(Length::from_pt(30.0))),
            Box::new(Expression::Literal(Length::from_pt(12.0))),
        );
        let ctx = EvalContext::with_width(Length::from_pt(100.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        assert!((result.to_pt() - 18.0).abs() < 0.01);
    }

    #[test]
    fn test_mul_literal_by_scalar() {
        let expr = Expression::Mul(Box::new(Expression::Literal(Length::from_pt(15.0))), 4.0);
        let ctx = EvalContext::with_width(Length::from_pt(100.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        assert!((result.to_pt() - 60.0).abs() < 0.01);
    }

    #[test]
    fn test_div_literal_by_scalar() {
        let expr = Expression::Div(Box::new(Expression::Literal(Length::from_pt(90.0))), 3.0);
        let ctx = EvalContext::with_width(Length::from_pt(100.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        assert!((result.to_pt() - 30.0).abs() < 0.01);
    }

    // --- Parsing ---

    #[test]
    fn test_parse_literal_mm() {
        let expr = Expression::parse("calc(25mm)").expect("test: should succeed");
        assert_eq!(expr, Expression::Literal(Length::from_mm(25.0)));
    }

    #[test]
    fn test_parse_literal_cm() {
        let expr = Expression::parse("calc(5cm)").expect("test: should succeed");
        assert_eq!(expr, Expression::Literal(Length::from_cm(5.0)));
    }

    #[test]
    fn test_parse_literal_in() {
        let expr = Expression::parse("calc(2in)").expect("test: should succeed");
        assert_eq!(expr, Expression::Literal(Length::from_inch(2.0)));
    }

    #[test]
    fn test_parse_literal_pt() {
        let expr = Expression::parse("calc(72pt)").expect("test: should succeed");
        assert_eq!(expr, Expression::Literal(Length::from_pt(72.0)));
    }

    #[test]
    fn test_parse_100_percent() {
        let expr = Expression::parse("calc(100%)").expect("test: should succeed");
        assert_eq!(
            expr,
            Expression::Percentage(Percentage::from_percent(100.0))
        );
    }

    #[test]
    fn test_parse_0_percent() {
        let expr = Expression::parse("calc(0%)").expect("test: should succeed");
        assert_eq!(expr, Expression::Percentage(Percentage::ZERO));
    }

    #[test]
    fn test_parse_requires_calc_wrapper() {
        assert!(Expression::parse("100%").is_err());
        assert!(Expression::parse("10pt").is_err());
        assert!(Expression::parse("50% + 10pt").is_err());
    }

    #[test]
    fn test_parse_div_by_zero_errors() {
        assert!(Expression::parse("calc(100% / 0)").is_err());
        assert!(Expression::parse("calc(100pt / 0)").is_err());
    }

    #[test]
    fn test_parse_add_mm_plus_pt() {
        let expr = Expression::parse("calc(10mm + 20pt)").expect("test: should succeed");
        match expr {
            Expression::Add(left, right) => {
                assert_eq!(*left, Expression::Literal(Length::from_mm(10.0)));
                assert_eq!(*right, Expression::Literal(Length::from_pt(20.0)));
            }
            _ => panic!("Expected Add"),
        }
    }

    // --- Evaluate parsed expressions ---

    #[test]
    fn test_eval_100pct_minus_margin_a4_width() {
        // Simulate: page content width = 100% - 2*margin(20pt)
        let expr = Expression::parse("calc(100% - 40pt)").expect("test: should succeed");
        let ctx = EvalContext::with_width(Length::from_pt(595.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        assert!((result.to_pt() - 555.0).abs() < 0.1);
    }

    #[test]
    fn test_eval_mul_gives_larger_result() {
        let expr = Expression::parse("calc(10pt * 5)").expect("test: should succeed");
        let ctx = EvalContext::with_width(Length::from_pt(100.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        assert!((result.to_pt() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_eval_nested_add_in_div() {
        // calc((10pt + 20pt) / 3) = 30pt / 3 = 10pt
        let expr = Expression::parse("calc((10pt + 20pt) / 3)").expect("test: should succeed");
        let ctx = EvalContext::with_width(Length::from_pt(100.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        assert!((result.to_pt() - 10.0).abs() < 0.1);
    }

    // --- Display ---

    #[test]
    fn test_display_literal() {
        let expr = Expression::Literal(Length::from_pt(10.0));
        let s = format!("{}", expr);
        assert!(s.contains("10pt"));
    }

    #[test]
    fn test_display_percentage() {
        let expr = Expression::Percentage(Percentage::from_percent(50.0));
        let s = format!("{}", expr);
        assert!(s.contains("50%"));
    }

    #[test]
    fn test_display_add() {
        let expr = Expression::Add(
            Box::new(Expression::Literal(Length::from_pt(10.0))),
            Box::new(Expression::Literal(Length::from_pt(20.0))),
        );
        let s = format!("{}", expr);
        assert!(s.contains('+'));
    }

    #[test]
    fn test_display_sub() {
        let expr = Expression::Sub(
            Box::new(Expression::Literal(Length::from_pt(10.0))),
            Box::new(Expression::Literal(Length::from_pt(5.0))),
        );
        let s = format!("{}", expr);
        assert!(s.contains('-'));
    }

    #[test]
    fn test_display_mul() {
        let expr = Expression::Mul(Box::new(Expression::Literal(Length::from_pt(10.0))), 3.0);
        let s = format!("{}", expr);
        assert!(s.contains('*'));
    }

    #[test]
    fn test_display_div() {
        let expr = Expression::Div(Box::new(Expression::Literal(Length::from_pt(10.0))), 2.0);
        let s = format!("{}", expr);
        assert!(s.contains('/'));
    }

    // --- Clone / PartialEq ---

    #[test]
    fn test_expression_clone() {
        let expr = Expression::Literal(Length::from_pt(12.0));
        let cloned = expr.clone();
        assert_eq!(expr, cloned);
    }

    #[test]
    fn test_expression_inequality() {
        let a = Expression::Literal(Length::from_pt(10.0));
        let b = Expression::Literal(Length::from_pt(20.0));
        assert_ne!(a, b);
    }
}

#[cfg(test)]
mod expression_eval_tests {
    use super::*;

    // --- Parsing various unit types ---

    #[test]
    fn test_parse_pt_literal() {
        let expr = Expression::parse("calc(12pt)").expect("test: should succeed");
        assert_eq!(expr, Expression::Literal(Length::from_pt(12.0)));
    }

    #[test]
    fn test_parse_mm_literal() {
        let expr = Expression::parse("calc(10mm)").expect("test: should succeed");
        assert_eq!(expr, Expression::Literal(Length::from_mm(10.0)));
    }

    #[test]
    fn test_parse_cm_literal() {
        let expr = Expression::parse("calc(2cm)").expect("test: should succeed");
        assert_eq!(expr, Expression::Literal(Length::from_cm(2.0)));
    }

    #[test]
    fn test_parse_in_literal() {
        let expr = Expression::parse("calc(1in)").expect("test: should succeed");
        assert_eq!(expr, Expression::Literal(Length::from_inch(1.0)));
    }

    #[test]
    fn test_parse_zero_percent() {
        let expr = Expression::parse("calc(0%)").expect("test: should succeed");
        assert_eq!(expr, Expression::Percentage(Percentage::ZERO));
    }

    #[test]
    fn test_parse_100_percent() {
        let expr = Expression::parse("calc(100%)").expect("test: should succeed");
        assert_eq!(
            expr,
            Expression::Percentage(Percentage::from_percent(100.0))
        );
    }

    #[test]
    fn test_parse_fractional_percent() {
        let expr = Expression::parse("calc(33%)").expect("test: should succeed");
        match expr {
            Expression::Percentage(p) => {
                assert!((p.to_percent() - 33.0).abs() < 0.001);
            }
            _ => panic!("Expected Percentage"),
        }
    }

    // --- Addition ---

    #[test]
    fn test_parse_add_pt_plus_pt() {
        let expr = Expression::parse("calc(10pt + 20pt)").expect("test: should succeed");
        let ctx = EvalContext::with_width(Length::from_pt(100.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        assert!((result.to_pt() - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_add_pct_plus_pt() {
        let expr = Expression::parse("calc(50% + 10pt)").expect("test: should succeed");
        let ctx = EvalContext::with_width(Length::from_pt(200.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        // 50% of 200 = 100 + 10 = 110
        assert!((result.to_pt() - 110.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_add_mm_plus_pt() {
        let expr = Expression::parse("calc(10mm + 20pt)").expect("test: should succeed");
        let ctx = EvalContext::with_width(Length::from_pt(100.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        let expected = Length::from_mm(10.0) + Length::from_pt(20.0);
        assert!((result.to_pt() - expected.to_pt()).abs() < 0.1);
    }

    // --- Subtraction ---

    #[test]
    fn test_parse_subtract_pt_from_pct() {
        let expr = Expression::parse("calc(100% - 20pt)").expect("test: should succeed");
        let ctx = EvalContext::with_width(Length::from_pt(100.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        assert!((result.to_pt() - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_subtract_larger_from_smaller_gives_negative() {
        let expr = Expression::parse("calc(10pt - 30pt)").expect("test: should succeed");
        let ctx = EvalContext::with_width(Length::from_pt(100.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        assert!(result.to_pt() < 0.0);
        assert!((result.to_pt() - (-20.0)).abs() < 0.01);
    }

    // --- Multiplication ---

    #[test]
    fn test_parse_mul_pt_by_scalar() {
        let expr = Expression::parse("calc(10pt * 5)").expect("test: should succeed");
        let ctx = EvalContext::with_width(Length::from_pt(100.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        assert!((result.to_pt() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_mul_pct_by_scalar() {
        let expr = Expression::parse("calc(50% * 2)").expect("test: should succeed");
        let ctx = EvalContext::with_width(Length::from_pt(100.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        // 50% of 100 * 2 via mul: first eval pct (=50pt) then * 2 = 100pt
        // However the expression structure is Mul(Pct(50%), 2) so result = 50 * 2 = 100pt
        assert!((result.to_pt() - 100.0).abs() < 0.01);
    }

    // --- Division ---

    #[test]
    fn test_parse_div_pt_by_scalar() {
        let expr = Expression::parse("calc(60pt / 4)").expect("test: should succeed");
        let ctx = EvalContext::with_width(Length::from_pt(100.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        assert!((result.to_pt() - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_div_by_zero_fails() {
        assert!(Expression::parse("calc(100% / 0)").is_err());
        assert!(Expression::parse("calc(100pt / 0)").is_err());
    }

    // --- Nested expressions ---

    #[test]
    fn test_nested_paren_sub_then_div() {
        // calc((100% - 40pt) / 2)
        let expr = Expression::parse("calc((100% - 40pt) / 2)").expect("test: should succeed");
        let ctx = EvalContext::with_width(Length::from_pt(200.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        // (200 - 40) / 2 = 80pt
        assert!((result.to_pt() - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_nested_paren_add_in_mul() {
        // calc((10pt + 20pt) * 3)
        let expr = Expression::parse("calc((10pt + 20pt) * 3)").expect("test: should succeed");
        let ctx = EvalContext::with_width(Length::from_pt(100.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        // (10 + 20) * 3 = 90pt
        assert!((result.to_pt() - 90.0).abs() < 0.01);
    }

    #[test]
    fn test_a4_content_width_calc() {
        // A4 = 595pt wide; typical 2cm margins = 2 * 56.7pt ≈ 113.4pt
        // Content = 100% - 2*margin
        let expr = Expression::parse("calc(100% - 40pt)").expect("test: should succeed");
        let ctx = EvalContext::with_width(Length::from_pt(595.0), Length::from_pt(12.0));
        let result = expr.evaluate(&ctx).expect("test: should succeed");
        assert!((result.to_pt() - 555.0).abs() < 0.1);
    }

    // --- No base value for percentage ---

    #[test]
    fn test_eval_percentage_no_base_returns_error() {
        let expr = Expression::parse("calc(50%)").expect("test: should succeed");
        let ctx = EvalContext::new(None, None, Length::from_pt(12.0));
        assert!(expr.evaluate(&ctx).is_err());
    }

    // --- Error: missing calc() wrapper ---

    #[test]
    fn test_parse_no_calc_wrapper_fails() {
        assert!(Expression::parse("100%").is_err());
        assert!(Expression::parse("10pt + 5pt").is_err());
        assert!(Expression::parse("50mm").is_err());
    }

    // --- Display ---

    #[test]
    fn test_display_contains_operands() {
        let expr = Expression::parse("calc(100% - 20pt)").expect("test: should succeed");
        let s = format!("{}", expr);
        assert!(s.contains("100%"));
        assert!(s.contains("20pt"));
        assert!(s.contains('-'));
    }

    #[test]
    fn test_display_literal_pt() {
        let expr = Expression::Literal(Length::from_pt(36.0));
        assert_eq!(format!("{}", expr), "36pt");
    }

    #[test]
    fn test_display_percentage() {
        let expr = Expression::Percentage(Percentage::from_percent(75.0));
        let s = format!("{}", expr);
        assert!(s.contains("75%"));
    }

    // --- Clone and PartialEq ---

    #[test]
    fn test_clone_and_eq() {
        let expr = Expression::parse("calc(50% + 10pt)").expect("test: should succeed");
        let cloned = expr.clone();
        assert_eq!(expr, cloned);
    }

    #[test]
    fn test_ne_different_expressions() {
        let a = Expression::Literal(Length::from_pt(10.0));
        let b = Expression::Literal(Length::from_pt(20.0));
        assert_ne!(a, b);
    }

    // --- EvalContext helpers ---

    #[test]
    fn test_eval_context_with_width_has_no_height() {
        let ctx = EvalContext::with_width(Length::from_pt(200.0), Length::from_pt(12.0));
        assert!(ctx.base_height.is_none());
        assert_eq!(ctx.base_width, Some(Length::from_pt(200.0)));
    }

    #[test]
    fn test_eval_context_with_height_has_no_width() {
        let ctx = EvalContext::with_height(Length::from_pt(300.0), Length::from_pt(12.0));
        assert!(ctx.base_width.is_none());
        assert_eq!(ctx.base_height, Some(Length::from_pt(300.0)));
    }
}
