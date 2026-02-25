//! Gradient types for backgrounds

use crate::{Color, Point};
use std::fmt;

/// Color stop in a gradient
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorStop {
    /// Offset along the gradient (0.0-1.0)
    pub offset: f64,
    /// Color at this offset
    pub color: Color,
}

impl ColorStop {
    /// Create a new color stop
    #[must_use = "this returns a new value without modifying anything"]
    pub fn new(offset: f64, color: Color) -> Self {
        Self {
            offset: offset.clamp(0.0, 1.0),
            color,
        }
    }
}

/// Gradient type
#[derive(Debug, Clone, PartialEq)]
pub enum Gradient {
    /// Linear gradient from start point to end point
    Linear {
        /// Starting point of the gradient (normalized coordinates 0.0-1.0)
        start_point: Point,
        /// Ending point of the gradient (normalized coordinates 0.0-1.0)
        end_point: Point,
        /// Color stops along the gradient
        color_stops: Vec<ColorStop>,
    },
    /// Radial gradient from center with radius
    Radial {
        /// Center point of the gradient (normalized coordinates 0.0-1.0)
        center: Point,
        /// Radius of the gradient (normalized coordinate 0.0-1.0)
        radius: f64,
        /// Color stops along the gradient
        color_stops: Vec<ColorStop>,
    },
}

impl Gradient {
    /// Create a linear gradient
    #[must_use = "this returns a new value without modifying anything"]
    pub fn linear(start_point: Point, end_point: Point, color_stops: Vec<ColorStop>) -> Self {
        Self::Linear {
            start_point,
            end_point,
            color_stops,
        }
    }

    /// Create a radial gradient
    #[must_use = "this returns a new value without modifying anything"]
    pub fn radial(center: Point, radius: f64, color_stops: Vec<ColorStop>) -> Self {
        Self::Radial {
            center,
            radius: radius.max(0.0),
            color_stops,
        }
    }

    /// Create a linear gradient from an angle in degrees
    ///
    /// Angle is measured clockwise from top (0° = to top, 90° = to right, etc.)
    #[must_use = "this returns a new value without modifying anything"]
    pub fn linear_from_angle(angle_deg: f64, color_stops: Vec<ColorStop>) -> Self {
        use std::f64::consts::PI;

        // Convert angle to radians and adjust for coordinate system
        // CSS angles: 0° = to top, 90° = to right
        // We need to convert to start/end points
        let angle_rad = (angle_deg - 90.0) * PI / 180.0;

        // Calculate start and end points on a unit square
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();

        // Calculate the gradient line endpoints
        let start_x = 0.5 - cos_a * 0.5;
        let start_y = 0.5 - sin_a * 0.5;
        let end_x = 0.5 + cos_a * 0.5;
        let end_y = 0.5 + sin_a * 0.5;

        use crate::Length;
        Self::Linear {
            start_point: Point::new(
                Length::from_pt(start_x * 100.0),
                Length::from_pt(start_y * 100.0),
            ),
            end_point: Point::new(
                Length::from_pt(end_x * 100.0),
                Length::from_pt(end_y * 100.0),
            ),
            color_stops,
        }
    }
}

impl fmt::Display for ColorStop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}%", self.color, self.offset * 100.0)
    }
}

impl fmt::Display for Gradient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Gradient::Linear {
                start_point,
                end_point,
                color_stops,
            } => {
                write!(f, "linear-gradient({} to {}", start_point, end_point)?;
                for stop in color_stops {
                    write!(f, ", {}", stop)?;
                }
                write!(f, ")")
            }
            Gradient::Radial {
                center,
                radius,
                color_stops,
            } => {
                write!(f, "radial-gradient(circle at {}, radius {}", center, radius)?;
                for stop in color_stops {
                    write!(f, ", {}", stop)?;
                }
                write!(f, ")")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Length;

    #[test]
    fn test_color_stop() {
        let stop = ColorStop::new(0.5, Color::RED);
        assert_eq!(stop.offset, 0.5);
        assert_eq!(stop.color, Color::RED);

        // Test clamping
        let stop_clamped = ColorStop::new(1.5, Color::BLUE);
        assert_eq!(stop_clamped.offset, 1.0);
    }

    #[test]
    fn test_linear_gradient() {
        let stops = vec![
            ColorStop::new(0.0, Color::RED),
            ColorStop::new(1.0, Color::BLUE),
        ];

        let gradient = Gradient::linear(
            Point::new(Length::ZERO, Length::ZERO),
            Point::new(Length::from_pt(100.0), Length::from_pt(100.0)),
            stops,
        );

        match gradient {
            Gradient::Linear { color_stops, .. } => {
                assert_eq!(color_stops.len(), 2);
            }
            _ => panic!("Expected linear gradient"),
        }
    }

    #[test]
    fn test_radial_gradient() {
        let stops = vec![
            ColorStop::new(0.0, Color::WHITE),
            ColorStop::new(1.0, Color::BLACK),
        ];

        let gradient = Gradient::radial(
            Point::new(Length::from_pt(50.0), Length::from_pt(50.0)),
            0.5,
            stops,
        );

        match gradient {
            Gradient::Radial {
                radius,
                color_stops,
                ..
            } => {
                assert_eq!(radius, 0.5);
                assert_eq!(color_stops.len(), 2);
            }
            _ => panic!("Expected radial gradient"),
        }
    }

    #[test]
    fn test_linear_from_angle() {
        let stops = vec![
            ColorStop::new(0.0, Color::RED),
            ColorStop::new(1.0, Color::BLUE),
        ];

        // 0 degrees = to top
        let gradient = Gradient::linear_from_angle(0.0, stops.clone());
        assert!(matches!(gradient, Gradient::Linear { .. }));

        // 90 degrees = to right
        let gradient = Gradient::linear_from_angle(90.0, stops);
        assert!(matches!(gradient, Gradient::Linear { .. }));
    }

    #[test]
    fn test_color_stop_display() {
        let stop = ColorStop::new(0.5, Color::RED);
        assert_eq!(format!("{}", stop), "#FF0000 50%");

        let stop_zero = ColorStop::new(0.0, Color::BLACK);
        assert_eq!(format!("{}", stop_zero), "#000000 0%");

        let stop_full = ColorStop::new(1.0, Color::WHITE);
        assert_eq!(format!("{}", stop_full), "#FFFFFF 100%");
    }

    #[test]
    fn test_linear_gradient_display() {
        let stops = vec![
            ColorStop::new(0.0, Color::RED),
            ColorStop::new(1.0, Color::BLUE),
        ];

        let gradient = Gradient::linear(
            Point::new(Length::ZERO, Length::ZERO),
            Point::new(Length::from_pt(100.0), Length::from_pt(100.0)),
            stops,
        );

        let display = format!("{}", gradient);
        assert!(display.starts_with("linear-gradient("));
        assert!(display.contains("#FF0000 0%"));
        assert!(display.contains("#0000FF 100%"));
    }

    #[test]
    fn test_radial_gradient_display() {
        let stops = vec![
            ColorStop::new(0.0, Color::WHITE),
            ColorStop::new(1.0, Color::BLACK),
        ];

        let gradient = Gradient::radial(
            Point::new(Length::from_pt(50.0), Length::from_pt(50.0)),
            0.5,
            stops,
        );

        let display = format!("{}", gradient);
        assert!(display.starts_with("radial-gradient("));
        assert!(display.contains("#FFFFFF 0%"));
        assert!(display.contains("#000000 100%"));
    }
}

#[cfg(test)]
mod gradient_extra_tests {
    use super::*;
    use crate::Length;

    // --- ColorStop ---

    #[test]
    fn test_color_stop_at_zero() {
        let s = ColorStop::new(0.0, Color::BLACK);
        assert_eq!(s.offset, 0.0);
    }

    #[test]
    fn test_color_stop_at_one() {
        let s = ColorStop::new(1.0, Color::WHITE);
        assert_eq!(s.offset, 1.0);
    }

    #[test]
    fn test_color_stop_clamp_above_one() {
        let s = ColorStop::new(1.5, Color::RED);
        assert_eq!(s.offset, 1.0);
    }

    #[test]
    fn test_color_stop_clamp_below_zero() {
        let s = ColorStop::new(-0.5, Color::BLUE);
        assert_eq!(s.offset, 0.0);
    }

    #[test]
    fn test_color_stop_preserves_color() {
        let s = ColorStop::new(0.5, Color::GREEN);
        assert_eq!(s.color, Color::GREEN);
    }

    #[test]
    fn test_color_stop_display_50_pct() {
        let s = ColorStop::new(0.5, Color::RED);
        let display = format!("{}", s);
        assert!(display.contains("50%"));
        assert!(display.contains("#FF0000"));
    }

    #[test]
    fn test_color_stop_display_0_pct() {
        let s = ColorStop::new(0.0, Color::BLACK);
        assert!(format!("{}", s).contains("0%"));
    }

    #[test]
    fn test_color_stop_display_100_pct() {
        let s = ColorStop::new(1.0, Color::WHITE);
        assert!(format!("{}", s).contains("100%"));
    }

    // --- Linear gradient ---

    #[test]
    fn test_linear_gradient_has_correct_stops() {
        let stops = vec![
            ColorStop::new(0.0, Color::RED),
            ColorStop::new(0.5, Color::GREEN),
            ColorStop::new(1.0, Color::BLUE),
        ];
        let g = Gradient::linear(
            Point::new(Length::ZERO, Length::ZERO),
            Point::new(Length::from_pt(100.0), Length::ZERO),
            stops,
        );
        match g {
            Gradient::Linear { color_stops, .. } => {
                assert_eq!(color_stops.len(), 3);
                assert_eq!(color_stops[1].color, Color::GREEN);
            }
            _ => panic!("Expected linear"),
        }
    }

    #[test]
    fn test_linear_gradient_single_stop() {
        let stops = vec![ColorStop::new(0.0, Color::RED)];
        let g = Gradient::linear(
            Point::new(Length::ZERO, Length::ZERO),
            Point::new(Length::from_pt(10.0), Length::ZERO),
            stops,
        );
        match g {
            Gradient::Linear { color_stops, .. } => {
                assert_eq!(color_stops.len(), 1);
            }
            _ => panic!("Expected linear"),
        }
    }

    #[test]
    fn test_linear_gradient_empty_stops() {
        let g = Gradient::linear(
            Point::new(Length::ZERO, Length::ZERO),
            Point::new(Length::from_pt(10.0), Length::ZERO),
            vec![],
        );
        match g {
            Gradient::Linear { color_stops, .. } => {
                assert!(color_stops.is_empty());
            }
            _ => panic!("Expected linear"),
        }
    }

    // --- Radial gradient ---

    #[test]
    fn test_radial_gradient_radius_clamped_non_negative() {
        let g = Gradient::radial(
            Point::new(Length::from_pt(50.0), Length::from_pt(50.0)),
            -1.0, // negative radius should be clamped to 0
            vec![],
        );
        match g {
            Gradient::Radial { radius, .. } => {
                assert!(radius >= 0.0);
            }
            _ => panic!("Expected radial"),
        }
    }

    #[test]
    fn test_radial_gradient_center() {
        let center = Point::new(Length::from_pt(30.0), Length::from_pt(40.0));
        let g = Gradient::radial(center, 0.7, vec![]);
        match g {
            Gradient::Radial { center: c, .. } => {
                assert_eq!(c, center);
            }
            _ => panic!("Expected radial"),
        }
    }

    #[test]
    fn test_radial_gradient_stops_count() {
        let stops = vec![
            ColorStop::new(0.0, Color::WHITE),
            ColorStop::new(0.5, Color::rgb(128, 128, 128)),
            ColorStop::new(1.0, Color::BLACK),
        ];
        let g = Gradient::radial(
            Point::new(Length::from_pt(50.0), Length::from_pt(50.0)),
            0.5,
            stops,
        );
        match g {
            Gradient::Radial { color_stops, .. } => {
                assert_eq!(color_stops.len(), 3);
            }
            _ => panic!("Expected radial"),
        }
    }

    // --- linear_from_angle ---

    #[test]
    fn test_linear_from_angle_0_is_linear() {
        let g = Gradient::linear_from_angle(
            0.0,
            vec![
                ColorStop::new(0.0, Color::RED),
                ColorStop::new(1.0, Color::BLUE),
            ],
        );
        assert!(matches!(g, Gradient::Linear { .. }));
    }

    #[test]
    fn test_linear_from_angle_90_is_linear() {
        let g = Gradient::linear_from_angle(
            90.0,
            vec![
                ColorStop::new(0.0, Color::RED),
                ColorStop::new(1.0, Color::BLUE),
            ],
        );
        assert!(matches!(g, Gradient::Linear { .. }));
    }

    #[test]
    fn test_linear_from_angle_180_is_linear() {
        let g = Gradient::linear_from_angle(180.0, vec![]);
        assert!(matches!(g, Gradient::Linear { .. }));
    }

    #[test]
    fn test_linear_from_angle_360_is_linear() {
        let g = Gradient::linear_from_angle(360.0, vec![]);
        assert!(matches!(g, Gradient::Linear { .. }));
    }

    // --- Display ---

    #[test]
    fn test_linear_gradient_display_contains_linear() {
        let g = Gradient::linear(
            Point::new(Length::ZERO, Length::ZERO),
            Point::new(Length::from_pt(100.0), Length::ZERO),
            vec![
                ColorStop::new(0.0, Color::RED),
                ColorStop::new(1.0, Color::BLUE),
            ],
        );
        let s = format!("{}", g);
        assert!(s.starts_with("linear-gradient("));
    }

    #[test]
    fn test_radial_gradient_display_contains_radial() {
        let g = Gradient::radial(
            Point::new(Length::from_pt(50.0), Length::from_pt(50.0)),
            0.5,
            vec![
                ColorStop::new(0.0, Color::WHITE),
                ColorStop::new(1.0, Color::BLACK),
            ],
        );
        let s = format!("{}", g);
        assert!(s.starts_with("radial-gradient("));
    }

    #[test]
    fn test_gradient_display_contains_stop_colors() {
        let g = Gradient::linear(
            Point::new(Length::ZERO, Length::ZERO),
            Point::new(Length::from_pt(10.0), Length::ZERO),
            vec![
                ColorStop::new(0.0, Color::RED),
                ColorStop::new(1.0, Color::BLUE),
            ],
        );
        let s = format!("{}", g);
        assert!(s.contains("#FF0000"));
        assert!(s.contains("#0000FF"));
    }
}
