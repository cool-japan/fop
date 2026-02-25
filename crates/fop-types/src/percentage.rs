//! Percentage type for relative values
//!
//! Percentages are stored as fractions (0.0 to 1.0) internally, but can be
//! created from and converted to percentage notation (0% to 100%).
//!
//! This follows CSS specification for percentage value handling, where
//! percentages can be used for widths, heights, margins, padding, etc.

use crate::{FopError, Length, Result};
use std::fmt;
use std::str::FromStr;

/// A percentage value stored as a fraction
///
/// Internally stores a value from 0.0 to 1.0 (representing 0% to 100%).
/// Values outside this range are allowed for certain use cases (e.g., animations).
///
/// # Examples
///
/// ```
/// use fop_types::Percentage;
///
/// // Create from percentage
/// let half = Percentage::from_percent(50.0);
/// assert_eq!(half.to_percent(), 50.0);
/// assert_eq!(half.as_fraction(), 0.5);
///
/// // Parse from string
/// let pct: Percentage = "75%".parse().unwrap();
/// assert_eq!(pct.to_percent(), 75.0);
/// ```
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct Percentage {
    value: f64,
}

impl Percentage {
    /// Zero percentage (0%)
    pub const ZERO: Self = Self { value: 0.0 };

    /// Full percentage (100%)
    pub const FULL: Self = Self { value: 1.0 };

    /// Half percentage (50%)
    pub const HALF: Self = Self { value: 0.5 };

    /// Create a new percentage from a fraction (0.0 = 0%, 1.0 = 100%)
    ///
    /// # Examples
    ///
    /// ```
    /// use fop_types::Percentage;
    ///
    /// let half = Percentage::new(0.5);
    /// assert_eq!(half.to_percent(), 50.0);
    /// ```
    #[inline]
    #[must_use = "this returns a new value without modifying anything"]
    pub const fn new(fraction: f64) -> Self {
        Self { value: fraction }
    }

    /// Create a percentage from a percent value (0.0 = 0%, 100.0 = 100%)
    ///
    /// # Examples
    ///
    /// ```
    /// use fop_types::Percentage;
    ///
    /// let pct = Percentage::from_percent(75.0);
    /// assert_eq!(pct.as_fraction(), 0.75);
    /// ```
    #[inline]
    #[must_use = "this returns a new value without modifying anything"]
    pub fn from_percent(percent: f64) -> Self {
        Self {
            value: percent / 100.0,
        }
    }

    /// Get the value as a percentage (0.0 to 100.0)
    ///
    /// # Examples
    ///
    /// ```
    /// use fop_types::Percentage;
    ///
    /// let pct = Percentage::new(0.25);
    /// assert_eq!(pct.to_percent(), 25.0);
    /// ```
    #[inline]
    #[must_use = "the result should be used"]
    pub fn to_percent(self) -> f64 {
        self.value * 100.0
    }

    /// Get the value as a fraction (0.0 to 1.0)
    ///
    /// # Examples
    ///
    /// ```
    /// use fop_types::Percentage;
    ///
    /// let pct = Percentage::from_percent(50.0);
    /// assert_eq!(pct.as_fraction(), 0.5);
    /// ```
    #[inline]
    #[must_use = "the result should be used"]
    pub const fn as_fraction(self) -> f64 {
        self.value
    }

    /// Convert this percentage to a Length given a base value
    ///
    /// # Examples
    ///
    /// ```
    /// use fop_types::{Percentage, Length};
    ///
    /// let pct = Percentage::from_percent(50.0);
    /// let base = Length::from_pt(100.0);
    /// let result = pct.of(base);
    /// assert!((result.to_pt() - 50.0).abs() < 0.001);
    /// ```
    #[inline]
    #[must_use = "computed value is not stored automatically"]
    pub fn of(self, base: Length) -> Length {
        Length::from_millipoints((base.millipoints() as f64 * self.value).round() as i32)
    }

    /// Clamp the percentage to a valid range [0.0, 1.0]
    ///
    /// # Examples
    ///
    /// ```
    /// use fop_types::Percentage;
    ///
    /// let pct = Percentage::new(1.5);
    /// let clamped = pct.clamp();
    /// assert_eq!(clamped.as_fraction(), 1.0);
    /// ```
    #[inline]
    #[must_use = "this returns a new value without modifying the original"]
    pub fn clamp(self) -> Self {
        Self {
            value: self.value.clamp(0.0, 1.0),
        }
    }

    /// Check if the percentage is in the valid range [0.0, 1.0]
    #[inline]
    #[must_use = "the result should be used"]
    pub fn is_valid(self) -> bool {
        (0.0..=1.0).contains(&self.value)
    }
}

impl FromStr for Percentage {
    type Err = FopError;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim();

        if let Some(stripped) = s.strip_suffix('%') {
            let percent = stripped
                .trim()
                .parse::<f64>()
                .map_err(|_| FopError::ParseError(format!("Invalid percentage value: {}", s)))?;
            Ok(Self::from_percent(percent))
        } else {
            Err(FopError::ParseError(format!(
                "Percentage must end with '%': {}",
                s
            )))
        }
    }
}

impl fmt::Debug for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Percentage({}%)", self.to_percent())
    }
}

impl fmt::Display for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.to_percent())
    }
}

// Arithmetic operations

impl std::ops::Add for Percentage {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            value: self.value + other.value,
        }
    }
}

impl std::ops::Sub for Percentage {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Self {
            value: self.value - other.value,
        }
    }
}

impl std::ops::Mul<f64> for Percentage {
    type Output = Self;

    #[inline]
    fn mul(self, scalar: f64) -> Self {
        Self {
            value: self.value * scalar,
        }
    }
}

impl std::ops::Mul<Percentage> for f64 {
    type Output = Percentage;

    #[inline]
    fn mul(self, pct: Percentage) -> Percentage {
        Percentage {
            value: self * pct.value,
        }
    }
}

impl std::ops::Div<f64> for Percentage {
    type Output = Self;

    #[inline]
    fn div(self, scalar: f64) -> Self {
        Self {
            value: self.value / scalar,
        }
    }
}

impl std::ops::Neg for Percentage {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self { value: -self.value }
    }
}

impl std::ops::AddAssign for Percentage {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        self.value += other.value;
    }
}

impl std::ops::SubAssign for Percentage {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        self.value -= other.value;
    }
}

impl std::ops::MulAssign<f64> for Percentage {
    #[inline]
    fn mul_assign(&mut self, scalar: f64) {
        self.value *= scalar;
    }
}

impl std::ops::DivAssign<f64> for Percentage {
    #[inline]
    fn div_assign(&mut self, scalar: f64) {
        self.value /= scalar;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(Percentage::ZERO.as_fraction(), 0.0);
        assert_eq!(Percentage::FULL.as_fraction(), 1.0);
        assert_eq!(Percentage::HALF.as_fraction(), 0.5);
    }

    #[test]
    fn test_new() {
        let pct = Percentage::new(0.75);
        assert_eq!(pct.as_fraction(), 0.75);
        assert_eq!(pct.to_percent(), 75.0);
    }

    #[test]
    fn test_from_percent() {
        let pct = Percentage::from_percent(50.0);
        assert_eq!(pct.as_fraction(), 0.5);

        let pct2 = Percentage::from_percent(100.0);
        assert_eq!(pct2.as_fraction(), 1.0);

        let pct3 = Percentage::from_percent(12.5);
        assert_eq!(pct3.as_fraction(), 0.125);
    }

    #[test]
    fn test_to_percent() {
        let pct = Percentage::new(0.25);
        assert_eq!(pct.to_percent(), 25.0);

        let pct2 = Percentage::new(1.0);
        assert_eq!(pct2.to_percent(), 100.0);
    }

    #[test]
    fn test_parse_valid() {
        let pct: Percentage = "50%".parse().expect("test: should succeed");
        assert_eq!(pct.to_percent(), 50.0);

        let pct2: Percentage = "100%".parse().expect("test: should succeed");
        assert_eq!(pct2.as_fraction(), 1.0);

        let pct3: Percentage = "12.5%".parse().expect("test: should succeed");
        assert_eq!(pct3.to_percent(), 12.5);

        let pct4: Percentage = " 75% ".parse().expect("test: should succeed");
        assert_eq!(pct4.to_percent(), 75.0);

        let pct5: Percentage = "0%".parse().expect("test: should succeed");
        assert_eq!(pct5.as_fraction(), 0.0);
    }

    #[test]
    fn test_parse_invalid() {
        assert!("50".parse::<Percentage>().is_err());
        assert!("abc%".parse::<Percentage>().is_err());
        assert!("%50".parse::<Percentage>().is_err());
        assert!("".parse::<Percentage>().is_err());
    }

    #[test]
    fn test_of_length() {
        let pct = Percentage::from_percent(50.0);
        let base = Length::from_pt(100.0);
        let result = pct.of(base);
        assert!((result.to_pt() - 50.0).abs() < 0.001);

        let pct2 = Percentage::from_percent(25.0);
        let base2 = Length::from_pt(200.0);
        let result2 = pct2.of(base2);
        assert!((result2.to_pt() - 50.0).abs() < 0.001);

        let pct3 = Percentage::from_percent(100.0);
        let base3 = Length::from_pt(72.0);
        let result3 = pct3.of(base3);
        assert!((result3.to_pt() - 72.0).abs() < 0.001);
    }

    #[test]
    fn test_clamp() {
        let pct1 = Percentage::new(1.5);
        assert_eq!(pct1.clamp().as_fraction(), 1.0);

        let pct2 = Percentage::new(-0.5);
        assert_eq!(pct2.clamp().as_fraction(), 0.0);

        let pct3 = Percentage::new(0.5);
        assert_eq!(pct3.clamp().as_fraction(), 0.5);
    }

    #[test]
    fn test_is_valid() {
        assert!(Percentage::new(0.5).is_valid());
        assert!(Percentage::new(0.0).is_valid());
        assert!(Percentage::new(1.0).is_valid());
        assert!(!Percentage::new(1.5).is_valid());
        assert!(!Percentage::new(-0.5).is_valid());
    }

    #[test]
    fn test_add() {
        let a = Percentage::from_percent(25.0);
        let b = Percentage::from_percent(25.0);
        let result = a + b;
        assert_eq!(result.to_percent(), 50.0);
    }

    #[test]
    fn test_sub() {
        let a = Percentage::from_percent(75.0);
        let b = Percentage::from_percent(25.0);
        let result = a - b;
        assert_eq!(result.to_percent(), 50.0);
    }

    #[test]
    fn test_mul() {
        let pct = Percentage::from_percent(50.0);
        let result = pct * 2.0;
        assert_eq!(result.to_percent(), 100.0);

        let result2 = 2.0 * pct;
        assert_eq!(result2.to_percent(), 100.0);
    }

    #[test]
    fn test_div() {
        let pct = Percentage::from_percent(100.0);
        let result = pct / 2.0;
        assert_eq!(result.to_percent(), 50.0);
    }

    #[test]
    fn test_neg() {
        let pct = Percentage::from_percent(50.0);
        let result = -pct;
        assert_eq!(result.to_percent(), -50.0);
    }

    #[test]
    fn test_add_assign() {
        let mut pct = Percentage::from_percent(25.0);
        pct += Percentage::from_percent(25.0);
        assert_eq!(pct.to_percent(), 50.0);
    }

    #[test]
    fn test_sub_assign() {
        let mut pct = Percentage::from_percent(75.0);
        pct -= Percentage::from_percent(25.0);
        assert_eq!(pct.to_percent(), 50.0);
    }

    #[test]
    fn test_mul_assign() {
        let mut pct = Percentage::from_percent(50.0);
        pct *= 2.0;
        assert_eq!(pct.to_percent(), 100.0);
    }

    #[test]
    fn test_div_assign() {
        let mut pct = Percentage::from_percent(100.0);
        pct /= 2.0;
        assert_eq!(pct.to_percent(), 50.0);
    }

    #[test]
    fn test_display() {
        let pct = Percentage::from_percent(50.0);
        assert_eq!(format!("{}", pct), "50%");
    }

    #[test]
    fn test_debug() {
        let pct = Percentage::from_percent(75.0);
        assert_eq!(format!("{:?}", pct), "Percentage(75%)");
    }

    #[test]
    fn test_ordering() {
        let a = Percentage::from_percent(25.0);
        let b = Percentage::from_percent(75.0);
        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, a);
    }
}

#[cfg(test)]
mod percentage_extra_tests {
    use super::*;
    use crate::Length;

    // --- boundary values ---

    #[test]
    fn test_zero_percent_of_length() {
        let p = Percentage::ZERO;
        let result = p.of(Length::from_pt(100.0));
        assert_eq!(result, Length::ZERO);
    }

    #[test]
    fn test_full_percent_of_length() {
        let p = Percentage::FULL;
        let base = Length::from_pt(200.0);
        let result = p.of(base);
        assert_eq!(result, base);
    }

    #[test]
    fn test_half_percent_of_length() {
        let p = Percentage::HALF;
        let result = p.of(Length::from_pt(80.0));
        assert!((result.to_pt() - 40.0).abs() < 0.001);
    }

    // --- clamp ---

    #[test]
    fn test_clamp_above_100() {
        let p = Percentage::new(1.5);
        let clamped = p.clamp();
        assert_eq!(clamped.as_fraction(), 1.0);
    }

    #[test]
    fn test_clamp_below_0() {
        let p = Percentage::new(-0.3);
        let clamped = p.clamp();
        assert_eq!(clamped.as_fraction(), 0.0);
    }

    #[test]
    fn test_clamp_in_range() {
        let p = Percentage::from_percent(42.0);
        let clamped = p.clamp();
        assert!((clamped.as_fraction() - 0.42).abs() < 0.001);
    }

    // --- is_valid ---

    #[test]
    fn test_is_valid_exactly_zero() {
        assert!(Percentage::ZERO.is_valid());
    }

    #[test]
    fn test_is_valid_exactly_one() {
        assert!(Percentage::FULL.is_valid());
    }

    #[test]
    fn test_is_not_valid_over_100() {
        assert!(!Percentage::new(1.001).is_valid());
    }

    #[test]
    fn test_is_not_valid_negative() {
        assert!(!Percentage::new(-0.001).is_valid());
    }

    // --- arithmetic produces sensible results ---

    #[test]
    fn test_add_to_over_100() {
        let a = Percentage::from_percent(70.0);
        let b = Percentage::from_percent(50.0);
        let sum = a + b;
        assert!((sum.to_percent() - 120.0).abs() < 0.001);
        // Over 100% is allowed but not valid
        assert!(!sum.is_valid());
    }

    #[test]
    fn test_sub_to_negative() {
        let a = Percentage::from_percent(10.0);
        let b = Percentage::from_percent(30.0);
        let diff = a - b;
        assert!(diff.to_percent() < 0.0);
    }

    #[test]
    fn test_mul_by_zero() {
        let p = Percentage::from_percent(75.0);
        let result = p * 0.0;
        assert_eq!(result.as_fraction(), 0.0);
    }

    #[test]
    fn test_div_shrinks() {
        let p = Percentage::from_percent(80.0);
        let result = p / 4.0;
        assert!((result.to_percent() - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_neg_and_back() {
        let p = Percentage::from_percent(25.0);
        let neg = -p;
        let back = -neg;
        assert!((back.to_percent() - 25.0).abs() < 0.001);
    }

    // --- of with mm-based length ---

    #[test]
    fn test_of_with_mm_length() {
        let p = Percentage::from_percent(10.0);
        let base = Length::from_mm(100.0);
        let result = p.of(base);
        assert!((result.to_mm() - 10.0).abs() < 0.01);
    }

    // --- parse edge cases ---

    #[test]
    fn test_parse_zero_percent() {
        let p: Percentage = "0%".parse().expect("test: should succeed");
        assert_eq!(p.as_fraction(), 0.0);
    }

    #[test]
    fn test_parse_100_percent() {
        let p: Percentage = "100%".parse().expect("test: should succeed");
        assert_eq!(p.as_fraction(), 1.0);
    }

    #[test]
    fn test_parse_decimal_percent() {
        let p: Percentage = "33.33%".parse().expect("test: should succeed");
        assert!((p.to_percent() - 33.33).abs() < 0.001);
    }

    #[test]
    fn test_parse_whitespace_trimmed() {
        let p: Percentage = "  50%  ".parse().expect("test: should succeed");
        assert!((p.to_percent() - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_missing_percent_sign_fails() {
        assert!("50".parse::<Percentage>().is_err());
    }

    #[test]
    fn test_parse_non_numeric_fails() {
        assert!("abc%".parse::<Percentage>().is_err());
    }

    // --- display ---

    #[test]
    fn test_display_zero() {
        assert_eq!(format!("{}", Percentage::ZERO), "0%");
    }

    #[test]
    fn test_display_full() {
        assert_eq!(format!("{}", Percentage::FULL), "100%");
    }

    #[test]
    fn test_display_half() {
        assert_eq!(format!("{}", Percentage::HALF), "50%");
    }

    // --- ordering ---

    #[test]
    fn test_partial_ord_less() {
        let a = Percentage::from_percent(30.0);
        let b = Percentage::from_percent(70.0);
        assert!(a < b);
    }

    #[test]
    fn test_partial_ord_equal() {
        let a = Percentage::from_percent(50.0);
        let b = Percentage::from_percent(50.0);
        assert_eq!(a, b);
    }
}

#[cfg(test)]
mod percentage_ops_tests {
    use super::*;
    use crate::Length;

    // --- Construction ---

    #[test]
    fn test_new_from_fraction() {
        let p = Percentage::new(0.42);
        assert!((p.as_fraction() - 0.42).abs() < 1e-9);
        assert!((p.to_percent() - 42.0).abs() < 1e-6);
    }

    #[test]
    fn test_from_percent_0() {
        let p = Percentage::from_percent(0.0);
        assert_eq!(p.as_fraction(), 0.0);
    }

    #[test]
    fn test_from_percent_100() {
        let p = Percentage::from_percent(100.0);
        assert!((p.as_fraction() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_from_percent_50() {
        let p = Percentage::from_percent(50.0);
        assert!((p.as_fraction() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_constants_zero_full_half() {
        assert_eq!(Percentage::ZERO.as_fraction(), 0.0);
        assert_eq!(Percentage::FULL.as_fraction(), 1.0);
        assert_eq!(Percentage::HALF.as_fraction(), 0.5);
    }

    // --- of(Length) ---

    #[test]
    fn test_zero_pct_of_any_length_is_zero() {
        let result = Percentage::ZERO.of(Length::from_pt(500.0));
        assert_eq!(result, Length::ZERO);
    }

    #[test]
    fn test_full_pct_of_length_equals_base() {
        let base = Length::from_pt(200.0);
        let result = Percentage::FULL.of(base);
        assert_eq!(result, base);
    }

    #[test]
    fn test_half_pct_of_100pt_is_50pt() {
        let result = Percentage::HALF.of(Length::from_pt(100.0));
        assert!((result.to_pt() - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_25pct_of_200pt_is_50pt() {
        let p = Percentage::from_percent(25.0);
        let result = p.of(Length::from_pt(200.0));
        assert!((result.to_pt() - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_of_with_mm_base() {
        let p = Percentage::from_percent(10.0);
        let base = Length::from_mm(100.0);
        let result = p.of(base);
        assert!((result.to_mm() - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_of_with_inch_base() {
        let p = Percentage::from_percent(50.0);
        let base = Length::from_inch(2.0);
        let result = p.of(base);
        assert!((result.to_inch() - 1.0).abs() < 0.001);
    }

    // --- Arithmetic ---

    #[test]
    fn test_add_two_percentages() {
        let a = Percentage::from_percent(30.0);
        let b = Percentage::from_percent(40.0);
        let c = a + b;
        assert!((c.to_percent() - 70.0).abs() < 1e-6);
    }

    #[test]
    fn test_sub_two_percentages() {
        let a = Percentage::from_percent(80.0);
        let b = Percentage::from_percent(30.0);
        let c = a - b;
        assert!((c.to_percent() - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_mul_percentage_by_scalar() {
        let p = Percentage::from_percent(25.0);
        let result = p * 3.0;
        assert!((result.to_percent() - 75.0).abs() < 1e-6);
    }

    #[test]
    fn test_mul_scalar_by_percentage() {
        let p = Percentage::from_percent(40.0);
        let result = 2.5_f64 * p;
        assert!((result.to_percent() - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_div_percentage_by_scalar() {
        let p = Percentage::from_percent(100.0);
        let result = p / 4.0;
        assert!((result.to_percent() - 25.0).abs() < 1e-6);
    }

    #[test]
    fn test_neg_percentage() {
        let p = Percentage::from_percent(50.0);
        let neg = -p;
        assert!((neg.to_percent() - (-50.0)).abs() < 1e-6);
    }

    #[test]
    fn test_add_assign() {
        let mut p = Percentage::from_percent(20.0);
        p += Percentage::from_percent(30.0);
        assert!((p.to_percent() - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_sub_assign() {
        let mut p = Percentage::from_percent(90.0);
        p -= Percentage::from_percent(40.0);
        assert!((p.to_percent() - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_mul_assign() {
        let mut p = Percentage::from_percent(50.0);
        p *= 2.0;
        assert!((p.to_percent() - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_div_assign() {
        let mut p = Percentage::from_percent(80.0);
        p /= 4.0;
        assert!((p.to_percent() - 20.0).abs() < 1e-6);
    }

    // --- clamp ---

    #[test]
    fn test_clamp_150pct_to_100pct() {
        let p = Percentage::from_percent(150.0);
        let clamped = p.clamp();
        assert_eq!(clamped.as_fraction(), 1.0);
    }

    #[test]
    fn test_clamp_neg_50pct_to_0pct() {
        let p = Percentage::from_percent(-50.0);
        let clamped = p.clamp();
        assert_eq!(clamped.as_fraction(), 0.0);
    }

    #[test]
    fn test_clamp_in_range_unchanged() {
        let p = Percentage::from_percent(60.0);
        let clamped = p.clamp();
        assert!((clamped.to_percent() - 60.0).abs() < 1e-6);
    }

    // --- is_valid ---

    #[test]
    fn test_is_valid_in_range() {
        assert!(Percentage::from_percent(50.0).is_valid());
        assert!(Percentage::ZERO.is_valid());
        assert!(Percentage::FULL.is_valid());
    }

    #[test]
    fn test_is_not_valid_above_100() {
        assert!(!Percentage::from_percent(100.001).is_valid());
    }

    #[test]
    fn test_is_not_valid_below_0() {
        assert!(!Percentage::from_percent(-0.001).is_valid());
    }

    // --- Parsing via FromStr ---

    #[test]
    fn test_parse_integer_pct() {
        let p: Percentage = "75%".parse().expect("test: should succeed");
        assert!((p.to_percent() - 75.0).abs() < 1e-6);
    }

    #[test]
    fn test_parse_decimal_pct() {
        let p: Percentage = "12.5%".parse().expect("test: should succeed");
        assert!((p.to_percent() - 12.5).abs() < 1e-6);
    }

    #[test]
    fn test_parse_zero_pct() {
        let p: Percentage = "0%".parse().expect("test: should succeed");
        assert_eq!(p.as_fraction(), 0.0);
    }

    #[test]
    fn test_parse_100_pct() {
        let p: Percentage = "100%".parse().expect("test: should succeed");
        assert!((p.as_fraction() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_parse_with_spaces() {
        let p: Percentage = "  33%  ".parse().expect("test: should succeed");
        assert!((p.to_percent() - 33.0).abs() < 1e-6);
    }

    #[test]
    fn test_parse_missing_percent_fails() {
        assert!("75".parse::<Percentage>().is_err());
    }

    #[test]
    fn test_parse_non_numeric_fails() {
        assert!("abc%".parse::<Percentage>().is_err());
    }

    #[test]
    fn test_parse_empty_string_fails() {
        assert!("".parse::<Percentage>().is_err());
    }

    // --- Display / Debug ---

    #[test]
    fn test_display_zero() {
        assert_eq!(format!("{}", Percentage::ZERO), "0%");
    }

    #[test]
    fn test_display_full() {
        assert_eq!(format!("{}", Percentage::FULL), "100%");
    }

    #[test]
    fn test_display_half() {
        assert_eq!(format!("{}", Percentage::HALF), "50%");
    }

    #[test]
    fn test_debug_format() {
        let p = Percentage::from_percent(33.0);
        let s = format!("{:?}", p);
        assert!(s.contains("33%") || s.contains("Percentage"));
    }

    // --- Ordering ---

    #[test]
    fn test_ordering_less_than() {
        assert!(Percentage::from_percent(10.0) < Percentage::from_percent(90.0));
    }

    #[test]
    fn test_ordering_greater_than() {
        assert!(Percentage::from_percent(80.0) > Percentage::from_percent(20.0));
    }

    #[test]
    fn test_ordering_equal() {
        let a = Percentage::from_percent(50.0);
        let b = Percentage::from_percent(50.0);
        assert_eq!(a, b);
    }
}
