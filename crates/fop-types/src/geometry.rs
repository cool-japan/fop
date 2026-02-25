//! Geometry types for positions, sizes, and rectangles

use crate::Length;
use std::fmt;

/// A 2D point with x and y coordinates
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: Length,
    pub y: Length,
}

impl Point {
    /// Origin point (0, 0)
    pub const ZERO: Self = Self {
        x: Length::ZERO,
        y: Length::ZERO,
    };

    /// Create a new point
    #[inline]
    #[must_use = "this returns a new value without modifying anything"]
    pub const fn new(x: Length, y: Length) -> Self {
        Self { x, y }
    }

    /// Translate the point by an offset
    #[inline]
    #[must_use = "this returns a new value without modifying the original"]
    pub fn translate(self, dx: Length, dy: Length) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Point({}, {})", self.x, self.y)
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// A 2D size with width and height
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Size {
    pub width: Length,
    pub height: Length,
}

impl Size {
    /// Zero size
    pub const ZERO: Self = Self {
        width: Length::ZERO,
        height: Length::ZERO,
    };

    /// Create a new size
    #[inline]
    #[must_use = "this returns a new value without modifying anything"]
    pub const fn new(width: Length, height: Length) -> Self {
        Self { width, height }
    }

    /// Check if the size is empty (width or height is zero)
    #[inline]
    #[must_use = "the result should be used"]
    pub const fn is_empty(self) -> bool {
        self.width.millipoints() <= 0 || self.height.millipoints() <= 0
    }

    /// Get the area (width * height)
    #[inline]
    #[must_use = "the result should be used"]
    pub fn area(self) -> i64 {
        self.width.millipoints() as i64 * self.height.millipoints() as i64
    }
}

impl fmt::Debug for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Size({}×{})", self.width, self.height)
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}×{}", self.width, self.height)
    }
}

/// A rectangle defined by position and size
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Rect {
    pub x: Length,
    pub y: Length,
    pub width: Length,
    pub height: Length,
}

impl Rect {
    /// Zero rectangle
    pub const ZERO: Self = Self {
        x: Length::ZERO,
        y: Length::ZERO,
        width: Length::ZERO,
        height: Length::ZERO,
    };

    /// Create a new rectangle from position and size
    #[inline]
    #[must_use = "this returns a new value without modifying anything"]
    pub const fn new(x: Length, y: Length, width: Length, height: Length) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create a rectangle from a point and size
    #[inline]
    #[must_use = "this returns a new value without modifying anything"]
    pub const fn from_point_size(origin: Point, size: Size) -> Self {
        Self {
            x: origin.x,
            y: origin.y,
            width: size.width,
            height: size.height,
        }
    }

    /// Get the origin point (top-left corner)
    #[inline]
    #[must_use = "the result should be used"]
    pub const fn origin(self) -> Point {
        Point::new(self.x, self.y)
    }

    /// Get the size
    #[inline]
    #[must_use = "the result should be used"]
    pub const fn size(self) -> Size {
        Size::new(self.width, self.height)
    }

    /// Get the right edge x-coordinate
    #[inline]
    #[must_use = "the result should be used"]
    pub fn right(self) -> Length {
        self.x + self.width
    }

    /// Get the bottom edge y-coordinate
    #[inline]
    #[must_use = "the result should be used"]
    pub fn bottom(self) -> Length {
        self.y + self.height
    }

    /// Check if the rectangle is empty
    #[inline]
    #[must_use = "the result should be used"]
    pub const fn is_empty(self) -> bool {
        self.width.millipoints() <= 0 || self.height.millipoints() <= 0
    }

    /// Check if the rectangle contains a point
    #[inline]
    #[must_use = "the result should be used"]
    pub fn contains(self, point: Point) -> bool {
        point.x.millipoints() >= self.x.millipoints()
            && point.x.millipoints() < self.right().millipoints()
            && point.y.millipoints() >= self.y.millipoints()
            && point.y.millipoints() < self.bottom().millipoints()
    }

    /// Check if this rectangle intersects another
    #[must_use = "the result should be used"]
    pub fn intersects(self, other: Rect) -> bool {
        self.x.millipoints() < other.right().millipoints()
            && self.right().millipoints() > other.x.millipoints()
            && self.y.millipoints() < other.bottom().millipoints()
            && self.bottom().millipoints() > other.y.millipoints()
    }

    /// Translate the rectangle by an offset
    #[inline]
    #[must_use = "this returns a new value without modifying the original"]
    pub fn translate(self, dx: Length, dy: Length) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            width: self.width,
            height: self.height,
        }
    }
}

impl fmt::Debug for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Rect(x={}, y={}, w={}, h={})",
            self.x, self.y, self.width, self.height
        )
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{},{}×{})", self.x, self.y, self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point() {
        let p = Point::new(Length::from_pt(10.0), Length::from_pt(20.0));
        assert_eq!(p.x, Length::from_pt(10.0));
        assert_eq!(p.y, Length::from_pt(20.0));

        let translated = p.translate(Length::from_pt(5.0), Length::from_pt(3.0));
        assert_eq!(translated.x, Length::from_pt(15.0));
        assert_eq!(translated.y, Length::from_pt(23.0));
    }

    #[test]
    fn test_size() {
        let s = Size::new(Length::from_pt(100.0), Length::from_pt(200.0));
        assert_eq!(s.width, Length::from_pt(100.0));
        assert_eq!(s.height, Length::from_pt(200.0));
        assert!(!s.is_empty());

        let empty = Size::new(Length::ZERO, Length::from_pt(100.0));
        assert!(empty.is_empty());
    }

    #[test]
    fn test_rect() {
        let r = Rect::new(
            Length::from_pt(10.0),
            Length::from_pt(20.0),
            Length::from_pt(100.0),
            Length::from_pt(50.0),
        );

        assert_eq!(r.right(), Length::from_pt(110.0));
        assert_eq!(r.bottom(), Length::from_pt(70.0));

        let p_inside = Point::new(Length::from_pt(50.0), Length::from_pt(40.0));
        assert!(r.contains(p_inside));

        let p_outside = Point::new(Length::from_pt(5.0), Length::from_pt(40.0));
        assert!(!r.contains(p_outside));
    }

    #[test]
    fn test_rect_intersection() {
        let r1 = Rect::new(
            Length::from_pt(0.0),
            Length::from_pt(0.0),
            Length::from_pt(100.0),
            Length::from_pt(100.0),
        );

        let r2 = Rect::new(
            Length::from_pt(50.0),
            Length::from_pt(50.0),
            Length::from_pt(100.0),
            Length::from_pt(100.0),
        );

        assert!(r1.intersects(r2));
        assert!(r2.intersects(r1));

        let r3 = Rect::new(
            Length::from_pt(200.0),
            Length::from_pt(200.0),
            Length::from_pt(100.0),
            Length::from_pt(100.0),
        );

        assert!(!r1.intersects(r3));
    }

    #[test]
    fn test_point_display() {
        let p = Point::new(Length::from_pt(10.0), Length::from_pt(20.0));
        assert_eq!(format!("{}", p), "(10pt, 20pt)");

        let p_zero = Point::ZERO;
        assert_eq!(format!("{}", p_zero), "(0pt, 0pt)");

        let p_neg = Point::new(Length::from_pt(-5.5), Length::from_pt(12.25));
        assert_eq!(format!("{}", p_neg), "(-5.5pt, 12.25pt)");
    }

    #[test]
    fn test_size_display() {
        let s = Size::new(Length::from_pt(100.0), Length::from_pt(200.0));
        assert_eq!(format!("{}", s), "100pt×200pt");

        let s_zero = Size::ZERO;
        assert_eq!(format!("{}", s_zero), "0pt×0pt");

        let s_fractional = Size::new(Length::from_pt(12.5), Length::from_pt(7.75));
        assert_eq!(format!("{}", s_fractional), "12.5pt×7.75pt");
    }

    #[test]
    fn test_rect_display() {
        let r = Rect::new(
            Length::from_pt(10.0),
            Length::from_pt(20.0),
            Length::from_pt(100.0),
            Length::from_pt(50.0),
        );
        assert_eq!(format!("{}", r), "(10pt,20pt,100pt×50pt)");

        let r_zero = Rect::ZERO;
        assert_eq!(format!("{}", r_zero), "(0pt,0pt,0pt×0pt)");

        let r_neg = Rect::new(
            Length::from_pt(-10.0),
            Length::from_pt(-20.0),
            Length::from_pt(30.0),
            Length::from_pt(40.0),
        );
        assert_eq!(format!("{}", r_neg), "(-10pt,-20pt,30pt×40pt)");
    }
}

#[cfg(test)]
mod geometry_extra_tests {
    use super::*;

    // --- Point ---

    #[test]
    fn test_point_zero() {
        assert_eq!(Point::ZERO.x, Length::ZERO);
        assert_eq!(Point::ZERO.y, Length::ZERO);
    }

    #[test]
    fn test_point_translate_positive() {
        let p = Point::new(Length::from_pt(5.0), Length::from_pt(10.0));
        let t = p.translate(Length::from_pt(3.0), Length::from_pt(7.0));
        assert_eq!(t.x, Length::from_pt(8.0));
        assert_eq!(t.y, Length::from_pt(17.0));
    }

    #[test]
    fn test_point_translate_negative() {
        let p = Point::new(Length::from_pt(20.0), Length::from_pt(30.0));
        let t = p.translate(Length::from_pt(-5.0), Length::from_pt(-10.0));
        assert_eq!(t.x, Length::from_pt(15.0));
        assert_eq!(t.y, Length::from_pt(20.0));
    }

    #[test]
    fn test_point_translate_zero() {
        let p = Point::new(Length::from_pt(5.0), Length::from_pt(5.0));
        let t = p.translate(Length::ZERO, Length::ZERO);
        assert_eq!(t, p);
    }

    #[test]
    fn test_point_equality() {
        let a = Point::new(Length::from_pt(3.0), Length::from_pt(4.0));
        let b = Point::new(Length::from_pt(3.0), Length::from_pt(4.0));
        assert_eq!(a, b);
    }

    #[test]
    fn test_point_inequality() {
        let a = Point::new(Length::from_pt(3.0), Length::from_pt(4.0));
        let b = Point::new(Length::from_pt(3.0), Length::from_pt(5.0));
        assert_ne!(a, b);
    }

    #[test]
    fn test_point_debug_format() {
        let p = Point::new(Length::from_pt(10.0), Length::from_pt(20.0));
        let s = format!("{:?}", p);
        assert!(s.contains("10pt"));
        assert!(s.contains("20pt"));
    }

    // --- Size ---

    #[test]
    fn test_size_zero() {
        assert!(Size::ZERO.is_empty());
    }

    #[test]
    fn test_size_non_empty() {
        let s = Size::new(Length::from_pt(10.0), Length::from_pt(20.0));
        assert!(!s.is_empty());
    }

    #[test]
    fn test_size_zero_width_is_empty() {
        let s = Size::new(Length::ZERO, Length::from_pt(100.0));
        assert!(s.is_empty());
    }

    #[test]
    fn test_size_zero_height_is_empty() {
        let s = Size::new(Length::from_pt(100.0), Length::ZERO);
        assert!(s.is_empty());
    }

    #[test]
    fn test_size_area() {
        // 100pt × 200pt → millipoints: 100000 × 200000 = 20_000_000_000
        let s = Size::new(Length::from_pt(100.0), Length::from_pt(200.0));
        let expected: i64 = 100_000_i64 * 200_000_i64;
        assert_eq!(s.area(), expected);
    }

    #[test]
    fn test_size_area_zero() {
        assert_eq!(Size::ZERO.area(), 0);
    }

    #[test]
    fn test_size_equality() {
        let a = Size::new(Length::from_pt(50.0), Length::from_pt(80.0));
        let b = Size::new(Length::from_pt(50.0), Length::from_pt(80.0));
        assert_eq!(a, b);
    }

    #[test]
    fn test_size_debug_format() {
        let s = Size::new(Length::from_pt(100.0), Length::from_pt(200.0));
        let dbg = format!("{:?}", s);
        assert!(dbg.contains("100pt"));
        assert!(dbg.contains("200pt"));
    }

    // --- Rect ---

    #[test]
    fn test_rect_zero() {
        let r = Rect::ZERO;
        assert_eq!(r.x, Length::ZERO);
        assert_eq!(r.width, Length::ZERO);
        assert!(r.is_empty());
    }

    #[test]
    fn test_rect_from_point_size() {
        let origin = Point::new(Length::from_pt(5.0), Length::from_pt(10.0));
        let size = Size::new(Length::from_pt(100.0), Length::from_pt(50.0));
        let r = Rect::from_point_size(origin, size);
        assert_eq!(r.x, Length::from_pt(5.0));
        assert_eq!(r.y, Length::from_pt(10.0));
        assert_eq!(r.width, Length::from_pt(100.0));
        assert_eq!(r.height, Length::from_pt(50.0));
    }

    #[test]
    fn test_rect_origin() {
        let r = Rect::new(
            Length::from_pt(3.0),
            Length::from_pt(7.0),
            Length::from_pt(20.0),
            Length::from_pt(15.0),
        );
        let origin = r.origin();
        assert_eq!(
            origin,
            Point::new(Length::from_pt(3.0), Length::from_pt(7.0))
        );
    }

    #[test]
    fn test_rect_size() {
        let r = Rect::new(
            Length::from_pt(0.0),
            Length::from_pt(0.0),
            Length::from_pt(100.0),
            Length::from_pt(200.0),
        );
        let size = r.size();
        assert_eq!(
            size,
            Size::new(Length::from_pt(100.0), Length::from_pt(200.0))
        );
    }

    #[test]
    fn test_rect_right() {
        let r = Rect::new(
            Length::from_pt(10.0),
            Length::from_pt(0.0),
            Length::from_pt(50.0),
            Length::from_pt(0.0),
        );
        assert_eq!(r.right(), Length::from_pt(60.0));
    }

    #[test]
    fn test_rect_bottom() {
        let r = Rect::new(
            Length::from_pt(0.0),
            Length::from_pt(20.0),
            Length::from_pt(0.0),
            Length::from_pt(30.0),
        );
        assert_eq!(r.bottom(), Length::from_pt(50.0));
    }

    #[test]
    fn test_rect_contains_boundary_left() {
        // contains uses >= for left, < for right
        let r = Rect::new(
            Length::from_pt(10.0),
            Length::from_pt(10.0),
            Length::from_pt(100.0),
            Length::from_pt(100.0),
        );
        let p_left_edge = Point::new(Length::from_pt(10.0), Length::from_pt(50.0));
        assert!(r.contains(p_left_edge));
    }

    #[test]
    fn test_rect_does_not_contain_right_edge() {
        let r = Rect::new(
            Length::from_pt(10.0),
            Length::from_pt(10.0),
            Length::from_pt(100.0),
            Length::from_pt(100.0),
        );
        let p_right_edge = Point::new(Length::from_pt(110.0), Length::from_pt(50.0));
        assert!(!r.contains(p_right_edge));
    }

    #[test]
    fn test_rect_contains_top_edge() {
        let r = Rect::new(
            Length::from_pt(0.0),
            Length::from_pt(0.0),
            Length::from_pt(100.0),
            Length::from_pt(100.0),
        );
        let p = Point::new(Length::from_pt(50.0), Length::from_pt(0.0));
        assert!(r.contains(p));
    }

    #[test]
    fn test_rect_does_not_contain_bottom_edge() {
        let r = Rect::new(
            Length::from_pt(0.0),
            Length::from_pt(0.0),
            Length::from_pt(100.0),
            Length::from_pt(100.0),
        );
        let p = Point::new(Length::from_pt(50.0), Length::from_pt(100.0));
        assert!(!r.contains(p));
    }

    #[test]
    fn test_rect_not_contains_outside() {
        let r = Rect::new(
            Length::from_pt(0.0),
            Length::from_pt(0.0),
            Length::from_pt(50.0),
            Length::from_pt(50.0),
        );
        let p = Point::new(Length::from_pt(100.0), Length::from_pt(100.0));
        assert!(!r.contains(p));
    }

    #[test]
    fn test_rect_intersects_self() {
        let r = Rect::new(
            Length::from_pt(10.0),
            Length::from_pt(10.0),
            Length::from_pt(100.0),
            Length::from_pt(100.0),
        );
        assert!(r.intersects(r));
    }

    #[test]
    fn test_rect_intersects_adjacent_no_overlap() {
        let r1 = Rect::new(
            Length::from_pt(0.0),
            Length::from_pt(0.0),
            Length::from_pt(100.0),
            Length::from_pt(100.0),
        );
        // r2 starts exactly at r1's right edge — no intersection
        let r2 = Rect::new(
            Length::from_pt(100.0),
            Length::from_pt(0.0),
            Length::from_pt(100.0),
            Length::from_pt(100.0),
        );
        assert!(!r1.intersects(r2));
    }

    #[test]
    fn test_rect_translate() {
        let r = Rect::new(
            Length::from_pt(10.0),
            Length::from_pt(20.0),
            Length::from_pt(50.0),
            Length::from_pt(30.0),
        );
        let t = r.translate(Length::from_pt(5.0), Length::from_pt(-5.0));
        assert_eq!(t.x, Length::from_pt(15.0));
        assert_eq!(t.y, Length::from_pt(15.0));
        // Size unchanged
        assert_eq!(t.width, Length::from_pt(50.0));
        assert_eq!(t.height, Length::from_pt(30.0));
    }

    #[test]
    fn test_rect_is_empty_zero_width() {
        let r = Rect::new(
            Length::from_pt(0.0),
            Length::from_pt(0.0),
            Length::ZERO,
            Length::from_pt(100.0),
        );
        assert!(r.is_empty());
    }

    #[test]
    fn test_rect_is_empty_zero_height() {
        let r = Rect::new(
            Length::from_pt(0.0),
            Length::from_pt(0.0),
            Length::from_pt(100.0),
            Length::ZERO,
        );
        assert!(r.is_empty());
    }

    #[test]
    fn test_rect_debug_format() {
        let r = Rect::ZERO;
        let s = format!("{:?}", r);
        assert!(s.contains("Rect"));
    }
}
