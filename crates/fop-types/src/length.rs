//! Length type for dimensional measurements
//!
//! All lengths are stored internally as millipoints (1/1000 of a point).
//! This provides sufficient precision while using integer arithmetic.
//!
//! Font-relative units (em, ex) are stored as enum variants and require
//! a FontContext to resolve to absolute lengths.

use std::fmt;

/// Unit type for length measurements
///
/// Supports both absolute units (stored as millipoints) and font-relative
/// units (em, ex) that require a FontContext for resolution.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LengthUnit {
    /// Absolute length in millipoints
    Absolute(i32),
    /// Font-size relative unit (1em = font-size)
    Em(f64),
    /// X-height relative unit (1ex ≈ 0.5em for Latin fonts)
    Ex(f64),
}

/// Context for resolving font-relative units
///
/// Contains the current font size and x-height needed to resolve
/// em and ex units to absolute lengths.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FontContext {
    /// Current font size
    pub font_size: Length,
    /// X-height of the current font (typically ~0.5em)
    pub x_height: Length,
}

impl FontContext {
    /// Create a new font context
    ///
    /// The x_height will be calculated as 0.5 times the font_size
    /// if not explicitly provided.
    #[must_use = "this returns a new value without modifying anything"]
    pub fn new(font_size: Length) -> Self {
        Self {
            font_size,
            x_height: font_size / 2,
        }
    }

    /// Create a new font context with explicit x-height
    #[must_use = "this returns a new value without modifying anything"]
    pub fn with_x_height(font_size: Length, x_height: Length) -> Self {
        Self {
            font_size,
            x_height,
        }
    }
}

/// Length measurement stored as millipoints (1/1000 point)
///
/// Conversion factors:
/// - 1 point = 1/72 inch
/// - 1 inch = 72 points = 72000 millipoints
/// - 1 mm = 2.834645669 points
/// - 1 cm = 28.34645669 points
///
/// Font-relative units:
/// - 1em = current font-size
/// - 1ex = x-height of current font (typically ~0.5em)
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Length {
    millipoints: i32,
}

impl Length {
    /// Zero length constant
    pub const ZERO: Self = Self { millipoints: 0 };

    /// Conversion factor: points to millipoints
    const PT_TO_MILLI: f64 = 1000.0;

    /// Conversion factor: inches to millipoints (72 * 1000)
    const IN_TO_MILLI: f64 = 72000.0;

    /// Conversion factor: millimeters to millipoints
    const MM_TO_MILLI: f64 = 2834.645669;

    /// Conversion factor: centimeters to millipoints
    const CM_TO_MILLI: f64 = 28346.45669;

    /// Create a length from points
    #[inline]
    #[must_use = "this returns a new value without modifying anything"]
    pub fn from_pt(pt: f64) -> Self {
        Self {
            millipoints: (pt * Self::PT_TO_MILLI).round() as i32,
        }
    }

    /// Create a length from inches
    #[inline]
    #[must_use = "this returns a new value without modifying anything"]
    pub fn from_inch(inch: f64) -> Self {
        Self {
            millipoints: (inch * Self::IN_TO_MILLI).round() as i32,
        }
    }

    /// Create a length from millimeters
    #[inline]
    #[must_use = "this returns a new value without modifying anything"]
    pub fn from_mm(mm: f64) -> Self {
        Self {
            millipoints: (mm * Self::MM_TO_MILLI).round() as i32,
        }
    }

    /// Create a length from centimeters
    #[inline]
    #[must_use = "this returns a new value without modifying anything"]
    pub fn from_cm(cm: f64) -> Self {
        Self {
            millipoints: (cm * Self::CM_TO_MILLI).round() as i32,
        }
    }

    /// Create a length from millipoints directly
    #[inline]
    #[must_use = "this returns a new value without modifying anything"]
    pub const fn from_millipoints(millipoints: i32) -> Self {
        Self { millipoints }
    }

    /// Create a font-relative length in em units
    ///
    /// 1em equals the current font-size. The returned LengthUnit
    /// must be resolved with a FontContext to get an absolute length.
    ///
    /// # Examples
    ///
    /// ```
    /// use fop_types::{Length, FontContext, LengthUnit};
    ///
    /// let em_unit = LengthUnit::Em(1.5);
    /// let context = FontContext::new(Length::from_pt(12.0));
    /// let resolved = Length::resolve_unit(&em_unit, &context);
    /// assert_eq!(resolved, Length::from_pt(18.0)); // 1.5 * 12pt
    /// ```
    #[inline]
    #[must_use = "this returns a new value without modifying anything"]
    pub fn from_em(em: f64) -> LengthUnit {
        LengthUnit::Em(em)
    }

    /// Create a font-relative length in ex units
    ///
    /// 1ex equals the x-height of the current font (typically ~0.5em).
    /// The returned LengthUnit must be resolved with a FontContext
    /// to get an absolute length.
    ///
    /// # Examples
    ///
    /// ```
    /// use fop_types::{Length, FontContext, LengthUnit};
    ///
    /// let ex_unit = LengthUnit::Ex(2.0);
    /// let context = FontContext::new(Length::from_pt(12.0));
    /// let resolved = Length::resolve_unit(&ex_unit, &context);
    /// // 2.0 * 6pt (x-height is 0.5 * font-size)
    /// assert_eq!(resolved, Length::from_pt(12.0));
    /// ```
    #[inline]
    #[must_use = "this returns a new value without modifying anything"]
    pub fn from_ex(ex: f64) -> LengthUnit {
        LengthUnit::Ex(ex)
    }

    /// Resolve a LengthUnit to an absolute Length using a FontContext
    ///
    /// For absolute units, returns the length directly.
    /// For em units, multiplies by the font size.
    /// For ex units, multiplies by the x-height.
    ///
    /// # Examples
    ///
    /// ```
    /// use fop_types::{Length, FontContext, LengthUnit};
    ///
    /// let context = FontContext::new(Length::from_pt(12.0));
    ///
    /// // Absolute unit
    /// let abs = LengthUnit::Absolute(12000);
    /// assert_eq!(Length::resolve_unit(&abs, &context), Length::from_pt(12.0));
    ///
    /// // Em unit
    /// let em = LengthUnit::Em(1.5);
    /// assert_eq!(Length::resolve_unit(&em, &context), Length::from_pt(18.0));
    ///
    /// // Ex unit
    /// let ex = LengthUnit::Ex(2.0);
    /// assert_eq!(Length::resolve_unit(&ex, &context), Length::from_pt(12.0));
    /// ```
    #[must_use = "computed value is not stored automatically"]
    pub fn resolve_unit(unit: &LengthUnit, context: &FontContext) -> Self {
        match unit {
            LengthUnit::Absolute(millipoints) => Self::from_millipoints(*millipoints),
            LengthUnit::Em(em) => {
                let font_size_milli = context.font_size.millipoints() as f64;
                Self::from_millipoints((em * font_size_milli).round() as i32)
            }
            LengthUnit::Ex(ex) => {
                let x_height_milli = context.x_height.millipoints() as f64;
                Self::from_millipoints((ex * x_height_milli).round() as i32)
            }
        }
    }

    /// Get the value in points
    #[inline]
    #[must_use = "the result should be used"]
    pub fn to_pt(self) -> f64 {
        self.millipoints as f64 / Self::PT_TO_MILLI
    }

    /// Get the value in inches
    #[inline]
    #[must_use = "the result should be used"]
    pub fn to_inch(self) -> f64 {
        self.millipoints as f64 / Self::IN_TO_MILLI
    }

    /// Get the value in millimeters
    #[inline]
    #[must_use = "the result should be used"]
    pub fn to_mm(self) -> f64 {
        self.millipoints as f64 / Self::MM_TO_MILLI
    }

    /// Get the value in centimeters
    #[inline]
    #[must_use = "the result should be used"]
    pub fn to_cm(self) -> f64 {
        self.millipoints as f64 / Self::CM_TO_MILLI
    }

    /// Get the raw millipoint value
    #[inline]
    #[must_use = "the result should be used"]
    pub const fn millipoints(self) -> i32 {
        self.millipoints
    }

    /// Get the absolute value
    #[inline]
    #[must_use = "this returns a new value without modifying the original"]
    pub fn abs(self) -> Self {
        Self {
            millipoints: self.millipoints.abs(),
        }
    }
}

impl std::ops::Neg for Length {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self {
            millipoints: -self.millipoints,
        }
    }
}

impl fmt::Debug for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Length({}pt)", self.to_pt())
    }
}

impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}pt", self.to_pt())
    }
}

impl std::ops::Add for Length {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            millipoints: self.millipoints + other.millipoints,
        }
    }
}

impl std::ops::Sub for Length {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Self {
            millipoints: self.millipoints - other.millipoints,
        }
    }
}

impl std::ops::Mul<i32> for Length {
    type Output = Self;

    #[inline]
    fn mul(self, scalar: i32) -> Self {
        Self {
            millipoints: self.millipoints * scalar,
        }
    }
}

impl std::ops::Div<i32> for Length {
    type Output = Self;

    #[inline]
    fn div(self, scalar: i32) -> Self {
        Self {
            millipoints: self.millipoints / scalar,
        }
    }
}

impl std::ops::AddAssign for Length {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        self.millipoints += other.millipoints;
    }
}

impl std::ops::SubAssign for Length {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        self.millipoints -= other.millipoints;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero() {
        assert_eq!(Length::ZERO.millipoints(), 0);
        assert_eq!(Length::ZERO.to_pt(), 0.0);
    }

    #[test]
    fn test_point_conversion() {
        let len = Length::from_pt(72.0);
        assert_eq!(len.millipoints(), 72000);
        assert!((len.to_pt() - 72.0).abs() < 0.001);
    }

    #[test]
    fn test_inch_conversion() {
        let len = Length::from_inch(1.0);
        assert_eq!(len.millipoints(), 72000);
        assert!((len.to_inch() - 1.0).abs() < 0.001);
        assert!((len.to_pt() - 72.0).abs() < 0.001);
    }

    #[test]
    fn test_mm_conversion() {
        let len = Length::from_mm(25.4);
        // 25.4 mm = 1 inch = 72 points
        assert!((len.to_inch() - 1.0).abs() < 0.001);
        assert!((len.to_pt() - 72.0).abs() < 0.1);
    }

    #[test]
    fn test_cm_conversion() {
        let len = Length::from_cm(2.54);
        // 2.54 cm = 1 inch = 72 points
        assert!((len.to_inch() - 1.0).abs() < 0.001);
        assert!((len.to_pt() - 72.0).abs() < 0.1);
    }

    #[test]
    fn test_arithmetic() {
        let a = Length::from_pt(10.0);
        let b = Length::from_pt(5.0);

        let sum = a + b;
        assert!((sum.to_pt() - 15.0).abs() < 0.001);

        let diff = a - b;
        assert!((diff.to_pt() - 5.0).abs() < 0.001);

        let prod = a * 2;
        assert!((prod.to_pt() - 20.0).abs() < 0.001);

        let quot = a / 2;
        assert!((quot.to_pt() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_abs_neg() {
        let len = Length::from_pt(-10.0);
        assert_eq!(len.abs(), Length::from_pt(10.0));
        assert_eq!(-len, Length::from_pt(10.0));
    }

    #[test]
    fn test_ordering() {
        let a = Length::from_pt(10.0);
        let b = Length::from_pt(20.0);
        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, a);
    }

    #[test]
    fn test_display() {
        let len = Length::from_pt(12.0);
        assert_eq!(format!("{}", len), "12pt");

        let zero = Length::ZERO;
        assert_eq!(format!("{}", zero), "0pt");

        let negative = Length::from_pt(-5.5);
        assert_eq!(format!("{}", negative), "-5.5pt");

        let fractional = Length::from_pt(12.345);
        assert_eq!(format!("{}", fractional), "12.345pt");
    }

    #[test]
    fn test_display_units() {
        // Display always shows points
        let from_inch = Length::from_inch(1.0);
        assert_eq!(format!("{}", from_inch), "72pt");

        let from_mm = Length::from_mm(25.4);
        // 25.4mm ≈ 72pt (1 inch)
        assert!(format!("{}", from_mm).contains("pt"));

        let from_cm = Length::from_cm(2.54);
        // 2.54cm ≈ 72pt (1 inch)
        assert!(format!("{}", from_cm).contains("pt"));
    }

    // Font-relative unit tests
    #[test]
    fn test_font_context_new() {
        let context = FontContext::new(Length::from_pt(12.0));
        assert_eq!(context.font_size, Length::from_pt(12.0));
        // x-height should be 0.5 * font-size = 6pt
        assert_eq!(context.x_height, Length::from_pt(6.0));
    }

    #[test]
    fn test_font_context_with_x_height() {
        let context = FontContext::with_x_height(Length::from_pt(12.0), Length::from_pt(5.0));
        assert_eq!(context.font_size, Length::from_pt(12.0));
        assert_eq!(context.x_height, Length::from_pt(5.0));
    }

    #[test]
    fn test_em_unit_creation() {
        let em = Length::from_em(1.5);
        assert_eq!(em, LengthUnit::Em(1.5));
    }

    #[test]
    fn test_ex_unit_creation() {
        let ex = Length::from_ex(2.0);
        assert_eq!(ex, LengthUnit::Ex(2.0));
    }

    #[test]
    fn test_resolve_absolute_unit() {
        let context = FontContext::new(Length::from_pt(12.0));
        let unit = LengthUnit::Absolute(12000);
        let resolved = Length::resolve_unit(&unit, &context);
        assert_eq!(resolved, Length::from_pt(12.0));
    }

    #[test]
    fn test_resolve_em_unit_one_em() {
        let context = FontContext::new(Length::from_pt(12.0));
        let unit = Length::from_em(1.0);
        let resolved = Length::resolve_unit(&unit, &context);
        assert_eq!(resolved, Length::from_pt(12.0));
    }

    #[test]
    fn test_resolve_em_unit_one_and_half_em() {
        let context = FontContext::new(Length::from_pt(12.0));
        let unit = Length::from_em(1.5);
        let resolved = Length::resolve_unit(&unit, &context);
        assert_eq!(resolved, Length::from_pt(18.0));
    }

    #[test]
    fn test_resolve_em_unit_fractional() {
        let context = FontContext::new(Length::from_pt(10.0));
        let unit = Length::from_em(0.8);
        let resolved = Length::resolve_unit(&unit, &context);
        assert_eq!(resolved, Length::from_pt(8.0));
    }

    #[test]
    fn test_resolve_ex_unit_one_ex() {
        let context = FontContext::new(Length::from_pt(12.0));
        let unit = Length::from_ex(1.0);
        let resolved = Length::resolve_unit(&unit, &context);
        // x-height is 6pt (0.5 * 12pt)
        assert_eq!(resolved, Length::from_pt(6.0));
    }

    #[test]
    fn test_resolve_ex_unit_two_ex() {
        let context = FontContext::new(Length::from_pt(12.0));
        let unit = Length::from_ex(2.0);
        let resolved = Length::resolve_unit(&unit, &context);
        // 2 * 6pt = 12pt
        assert_eq!(resolved, Length::from_pt(12.0));
    }

    #[test]
    fn test_resolve_ex_unit_with_custom_x_height() {
        let context = FontContext::with_x_height(Length::from_pt(12.0), Length::from_pt(5.0));
        let unit = Length::from_ex(2.0);
        let resolved = Length::resolve_unit(&unit, &context);
        assert_eq!(resolved, Length::from_pt(10.0));
    }

    #[test]
    fn test_em_with_different_font_sizes() {
        // Test with 16pt font
        let context16 = FontContext::new(Length::from_pt(16.0));
        let unit = Length::from_em(1.0);
        assert_eq!(
            Length::resolve_unit(&unit, &context16),
            Length::from_pt(16.0)
        );

        // Test with 24pt font
        let context24 = FontContext::new(Length::from_pt(24.0));
        assert_eq!(
            Length::resolve_unit(&unit, &context24),
            Length::from_pt(24.0)
        );
    }

    #[test]
    fn test_ex_with_different_font_sizes() {
        // Test with 16pt font (x-height = 8pt)
        let context16 = FontContext::new(Length::from_pt(16.0));
        let unit = Length::from_ex(1.0);
        assert_eq!(
            Length::resolve_unit(&unit, &context16),
            Length::from_pt(8.0)
        );

        // Test with 20pt font (x-height = 10pt)
        let context20 = FontContext::new(Length::from_pt(20.0));
        assert_eq!(
            Length::resolve_unit(&unit, &context20),
            Length::from_pt(10.0)
        );
    }

    #[test]
    fn test_em_negative_values() {
        let context = FontContext::new(Length::from_pt(12.0));
        let unit = Length::from_em(-1.0);
        let resolved = Length::resolve_unit(&unit, &context);
        assert_eq!(resolved, Length::from_pt(-12.0));
    }

    #[test]
    fn test_ex_negative_values() {
        let context = FontContext::new(Length::from_pt(12.0));
        let unit = Length::from_ex(-2.0);
        let resolved = Length::resolve_unit(&unit, &context);
        assert_eq!(resolved, Length::from_pt(-12.0));
    }

    #[test]
    fn test_em_zero() {
        let context = FontContext::new(Length::from_pt(12.0));
        let unit = Length::from_em(0.0);
        let resolved = Length::resolve_unit(&unit, &context);
        assert_eq!(resolved, Length::ZERO);
    }

    #[test]
    fn test_ex_zero() {
        let context = FontContext::new(Length::from_pt(12.0));
        let unit = Length::from_ex(0.0);
        let resolved = Length::resolve_unit(&unit, &context);
        assert_eq!(resolved, Length::ZERO);
    }

    #[test]
    fn test_length_unit_equality() {
        assert_eq!(LengthUnit::Em(1.5), LengthUnit::Em(1.5));
        assert_eq!(LengthUnit::Ex(2.0), LengthUnit::Ex(2.0));
        assert_eq!(LengthUnit::Absolute(12000), LengthUnit::Absolute(12000));

        assert_ne!(LengthUnit::Em(1.5), LengthUnit::Em(2.0));
        assert_ne!(LengthUnit::Ex(1.0), LengthUnit::Ex(2.0));
        assert_ne!(LengthUnit::Em(1.0), LengthUnit::Ex(1.0));
    }
}

#[cfg(test)]
mod length_extra_tests {
    use super::*;

    const TOLERANCE: f64 = 0.01;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < TOLERANCE
    }

    // --- Unit conversion round-trips ---

    #[test]
    fn test_mm_roundtrip() {
        let mm = Length::from_mm(42.0);
        assert!(approx(mm.to_mm(), 42.0));
    }

    #[test]
    fn test_cm_roundtrip() {
        let cm = Length::from_cm(5.5);
        assert!(approx(cm.to_cm(), 5.5));
    }

    #[test]
    fn test_inch_roundtrip() {
        let inch = Length::from_inch(3.0);
        assert!(approx(inch.to_inch(), 3.0));
    }

    #[test]
    fn test_pt_roundtrip() {
        let pt = Length::from_pt(144.0);
        assert!(approx(pt.to_pt(), 144.0));
    }

    // --- Cross-unit equivalence ---

    #[test]
    fn test_25_4mm_equals_1inch() {
        // 25.4 mm = 1 inch exactly
        let mm = Length::from_mm(25.4);
        let inch = Length::from_inch(1.0);
        assert!(approx(mm.to_pt(), inch.to_pt()));
    }

    #[test]
    fn test_2_54cm_equals_1inch() {
        let cm = Length::from_cm(2.54);
        let inch = Length::from_inch(1.0);
        assert!(approx(cm.to_pt(), inch.to_pt()));
    }

    #[test]
    fn test_1cm_equals_10mm() {
        let cm = Length::from_cm(1.0);
        let mm = Length::from_mm(10.0);
        assert!(approx(cm.to_pt(), mm.to_pt()));
    }

    #[test]
    fn test_1inch_equals_72pt() {
        let inch = Length::from_inch(1.0);
        assert!(approx(inch.to_pt(), 72.0));
    }

    #[test]
    fn test_a4_width_mm_to_pt() {
        // A4 width: 210mm ≈ 595.276 pt
        let w = Length::from_mm(210.0);
        assert!(approx(w.to_pt(), 595.276));
    }

    #[test]
    fn test_a4_height_mm_to_pt() {
        // A4 height: 297mm ≈ 841.890 pt
        let h = Length::from_mm(297.0);
        assert!(approx(h.to_pt(), 841.890));
    }

    #[test]
    fn test_letter_width_in_to_pt() {
        // US Letter: 8.5in × 11in
        let w = Length::from_inch(8.5);
        assert!(approx(w.to_pt(), 612.0));
    }

    #[test]
    fn test_letter_height_in_to_pt() {
        let h = Length::from_inch(11.0);
        assert!(approx(h.to_pt(), 792.0));
    }

    // --- Comparison via Ord ---

    #[test]
    fn test_max_via_ord() {
        let a = Length::from_mm(5.0);
        let b = Length::from_mm(10.0);
        assert_eq!(a.max(b), b);
        assert_eq!(b.max(a), b);
    }

    #[test]
    fn test_min_via_ord() {
        let a = Length::from_mm(5.0);
        let b = Length::from_mm(10.0);
        assert_eq!(a.min(b), a);
        assert_eq!(b.min(a), a);
    }

    #[test]
    fn test_clamp_via_ord() {
        let val = Length::from_mm(15.0);
        let lo = Length::from_mm(0.0);
        let hi = Length::from_mm(10.0);
        assert_eq!(val.clamp(lo, hi), hi);

        let val2 = Length::from_mm(-5.0);
        assert_eq!(val2.clamp(lo, hi), lo);

        let val3 = Length::from_mm(7.0);
        assert_eq!(val3.clamp(lo, hi), val3);
    }

    // --- Arithmetic ---

    #[test]
    fn test_add_assign() {
        let mut a = Length::from_pt(10.0);
        a += Length::from_pt(5.0);
        assert!(approx(a.to_pt(), 15.0));
    }

    #[test]
    fn test_sub_assign() {
        let mut a = Length::from_pt(10.0);
        a -= Length::from_pt(3.0);
        assert!(approx(a.to_pt(), 7.0));
    }

    #[test]
    fn test_mul_i32() {
        let a = Length::from_pt(7.0);
        assert!(approx((a * 3).to_pt(), 21.0));
    }

    #[test]
    fn test_div_i32() {
        let a = Length::from_pt(15.0);
        assert!(approx((a / 5).to_pt(), 3.0));
    }

    #[test]
    fn test_neg_operator() {
        let a = Length::from_pt(8.0);
        assert!(approx((-a).to_pt(), -8.0));
    }

    #[test]
    fn test_abs_positive() {
        let a = Length::from_pt(5.0);
        assert_eq!(a.abs(), a);
    }

    #[test]
    fn test_abs_negative() {
        let a = Length::from_pt(-5.0);
        assert!(approx(a.abs().to_pt(), 5.0));
    }

    // --- Millipoints ---

    #[test]
    fn test_from_millipoints_and_back() {
        let mp = 72000_i32;
        let len = Length::from_millipoints(mp);
        assert_eq!(len.millipoints(), mp);
        assert!(approx(len.to_pt(), 72.0));
    }

    #[test]
    fn test_zero_millipoints() {
        assert_eq!(Length::ZERO.millipoints(), 0);
    }

    #[test]
    fn test_negative_millipoints() {
        let neg = Length::from_pt(-10.0);
        assert!(neg.millipoints() < 0);
    }

    // --- FontContext ---

    #[test]
    fn test_font_context_x_height_is_half_font_size() {
        let ctx = FontContext::new(Length::from_pt(20.0));
        assert!(approx(ctx.x_height.to_pt(), 10.0));
    }

    #[test]
    fn test_font_context_with_x_height_explicit() {
        let ctx = FontContext::with_x_height(Length::from_pt(16.0), Length::from_pt(7.0));
        assert!(approx(ctx.x_height.to_pt(), 7.0));
        assert!(approx(ctx.font_size.to_pt(), 16.0));
    }

    #[test]
    fn test_em_resolution_large_font() {
        // 2.5em with 24pt font = 60pt
        let ctx = FontContext::new(Length::from_pt(24.0));
        let unit = Length::from_em(2.5);
        let resolved = Length::resolve_unit(&unit, &ctx);
        assert!(approx(resolved.to_pt(), 60.0));
    }

    #[test]
    fn test_ex_resolution_custom_x_height() {
        // 3ex with x-height=4pt = 12pt
        let ctx = FontContext::with_x_height(Length::from_pt(12.0), Length::from_pt(4.0));
        let unit = Length::from_ex(3.0);
        let resolved = Length::resolve_unit(&unit, &ctx);
        assert!(approx(resolved.to_pt(), 12.0));
    }

    #[test]
    fn test_resolve_absolute_unit_large() {
        let ctx = FontContext::new(Length::from_pt(12.0));
        let unit = LengthUnit::Absolute(595276); // A4 width in millipoints approx
        let resolved = Length::resolve_unit(&unit, &ctx);
        assert_eq!(resolved.millipoints(), 595276);
    }

    // --- Display/Debug ---

    #[test]
    fn test_debug_format() {
        let len = Length::from_pt(36.0);
        let s = format!("{:?}", len);
        assert!(s.contains("36pt"));
    }

    #[test]
    fn test_display_zero() {
        assert_eq!(format!("{}", Length::ZERO), "0pt");
    }

    #[test]
    fn test_display_fractional() {
        let len = Length::from_pt(1.5);
        assert_eq!(format!("{}", len), "1.5pt");
    }

    // --- LengthUnit Debug ---

    #[test]
    fn test_length_unit_debug_em() {
        let unit = LengthUnit::Em(1.5);
        let s = format!("{:?}", unit);
        assert!(s.contains("Em"));
        assert!(s.contains("1.5"));
    }

    #[test]
    fn test_length_unit_debug_ex() {
        let unit = LengthUnit::Ex(2.0);
        let s = format!("{:?}", unit);
        assert!(s.contains("Ex"));
    }

    #[test]
    fn test_length_unit_debug_absolute() {
        let unit = LengthUnit::Absolute(12000);
        let s = format!("{:?}", unit);
        assert!(s.contains("Absolute"));
    }
}

#[cfg(test)]
mod length_unit_conversion_tests {
    use super::*;

    const TOL: f64 = 0.001;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < TOL
    }

    // --- 1mm = 2.8346...pt ---

    #[test]
    fn test_1mm_to_pt_approx_2_8346() {
        let l = Length::from_mm(1.0);
        let pts = l.to_pt();
        assert!(
            (pts - 2.8346).abs() < 0.01,
            "1mm should be ~2.83pt, got {}",
            pts
        );
    }

    // --- 1cm = 10mm ---

    #[test]
    fn test_1cm_equals_10mm_pt_value() {
        let cm = Length::from_cm(1.0);
        let mm = Length::from_mm(10.0);
        assert!(approx_eq(cm.to_pt(), mm.to_pt()), "1cm != 10mm in pt");
    }

    // --- 1in = 72pt ---

    #[test]
    fn test_1in_equals_72pt() {
        let inch = Length::from_inch(1.0);
        assert!(
            approx_eq(inch.to_pt(), 72.0),
            "1in should be 72pt, got {}",
            inch.to_pt()
        );
    }

    // --- pc unit: 1pc = 12pt ---

    #[test]
    fn test_pica_to_pt() {
        // 1 pica = 12 pt (1/6 inch)
        // Build via 12pt directly since we have no from_pc
        let one_pica_via_pt = Length::from_pt(12.0);
        let one_pica_via_inch = Length::from_inch(1.0 / 6.0);
        assert!(approx_eq(
            one_pica_via_pt.to_pt(),
            one_pica_via_inch.to_pt()
        ));
    }

    // --- px equivalence (CSS px = 1/96 inch) ---

    #[test]
    fn test_css_px_to_pt() {
        // 1 CSS px = 1/96 inch = 0.75 pt
        let one_px = Length::from_inch(1.0 / 96.0);
        assert!(
            (one_px.to_pt() - 0.75).abs() < 0.01,
            "1px should be 0.75pt, got {}",
            one_px.to_pt()
        );
    }

    #[test]
    fn test_96px_equals_72pt() {
        // 96px = 1in = 72pt
        let ninety_six_px = Length::from_inch(1.0);
        assert!(
            approx_eq(ninety_six_px.to_pt(), 72.0),
            "96px (=1in) should be 72pt"
        );
    }

    // --- millipoint precision ---

    #[test]
    fn test_millipoint_precision_1pt() {
        let l = Length::from_pt(1.0);
        assert_eq!(l.millipoints(), 1000);
    }

    #[test]
    fn test_millipoint_precision_1inch() {
        let l = Length::from_inch(1.0);
        assert_eq!(l.millipoints(), 72_000);
    }

    // --- round trip: pt -> millipoints -> pt ---

    #[test]
    fn test_pt_millipoints_roundtrip_fractional() {
        let l = Length::from_pt(3.5);
        assert!(approx_eq(l.to_pt(), 3.5));
    }

    // --- zero conversions ---

    #[test]
    fn test_zero_mm_to_pt() {
        let l = Length::from_mm(0.0);
        assert_eq!(l.to_pt(), 0.0);
    }

    #[test]
    fn test_zero_cm_to_pt() {
        assert_eq!(Length::from_cm(0.0).to_pt(), 0.0);
    }

    #[test]
    fn test_zero_inch_to_pt() {
        assert_eq!(Length::from_inch(0.0).to_pt(), 0.0);
    }

    // --- negative values ---

    #[test]
    fn test_negative_mm() {
        let l = Length::from_mm(-5.0);
        assert!(l.millipoints() < 0);
        assert!(l < Length::ZERO);
    }

    #[test]
    fn test_negative_inch() {
        let l = Length::from_inch(-1.0);
        assert!((l.to_pt() - (-72.0)).abs() < 0.1, "got {}pt", l.to_pt());
    }

    // --- multiplication by f64 via millipoints ---

    #[test]
    fn test_mul_by_i32_scalar() {
        let l = Length::from_pt(12.0);
        let result = l * 3;
        assert!(approx_eq(result.to_pt(), 36.0));
    }

    #[test]
    fn test_div_by_i32_scalar() {
        let l = Length::from_pt(30.0);
        let result = l / 5;
        assert!(approx_eq(result.to_pt(), 6.0));
    }

    // --- max/min (inherited from Ord) ---

    #[test]
    fn test_max_of_two_lengths() {
        let a = Length::from_mm(5.0);
        let b = Length::from_mm(10.0);
        assert_eq!(a.max(b), b);
    }

    #[test]
    fn test_min_of_two_lengths() {
        let a = Length::from_mm(5.0);
        let b = Length::from_mm(10.0);
        assert_eq!(a.min(b), a);
    }

    // --- ordering ---

    #[test]
    fn test_ordering_less_than() {
        let a = Length::from_mm(5.0);
        let b = Length::from_mm(10.0);
        assert!(a < b);
    }

    #[test]
    fn test_ordering_greater_than() {
        let a = Length::from_mm(10.0);
        let b = Length::from_mm(5.0);
        assert!(a > b);
    }

    #[test]
    fn test_ordering_equal() {
        let a = Length::from_pt(12.0);
        let b = Length::from_pt(12.0);
        assert!(a <= b && b <= a);
    }

    // --- addition and subtraction across units ---

    #[test]
    fn test_add_mm_and_pt() {
        // 25.4mm + 0pt = 1in = 72pt
        let mm = Length::from_mm(25.4);
        let pt = Length::from_pt(0.0);
        let result = mm + pt;
        assert!((result.to_pt() - 72.0).abs() < 0.1);
    }

    #[test]
    fn test_sub_pt_from_inch() {
        // 1in - 36pt = 0.5in = 36pt
        let inch = Length::from_inch(1.0);
        let half = Length::from_pt(36.0);
        let result = inch - half;
        assert!((result.to_pt() - 36.0).abs() < 0.1);
    }

    // --- abs ---

    #[test]
    fn test_abs_of_negative_mm() {
        let l = Length::from_mm(-7.5);
        let abs = l.abs();
        assert!(approx_eq(abs.to_mm(), 7.5));
    }

    // --- neg operator ---

    #[test]
    fn test_neg_of_positive_length() {
        let l = Length::from_pt(10.0);
        let neg = -l;
        assert!(approx_eq(neg.to_pt(), -10.0));
    }

    #[test]
    fn test_neg_of_negative_length() {
        let l = Length::from_pt(-5.0);
        let pos = -l;
        assert!(approx_eq(pos.to_pt(), 5.0));
    }

    // --- clamp ---

    #[test]
    fn test_clamp_length() {
        let val = Length::from_pt(150.0);
        let lo = Length::from_pt(0.0);
        let hi = Length::from_pt(100.0);
        assert_eq!(val.clamp(lo, hi), hi);
    }

    // --- add_assign / sub_assign ---

    #[test]
    fn test_add_assign_mm() {
        let mut a = Length::from_mm(5.0);
        a += Length::from_mm(3.0);
        assert!(approx_eq(a.to_mm(), 8.0));
    }

    #[test]
    fn test_sub_assign_pt() {
        let mut a = Length::from_pt(20.0);
        a -= Length::from_pt(8.0);
        assert!(approx_eq(a.to_pt(), 12.0));
    }
}
