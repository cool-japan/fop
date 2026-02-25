//! Color type for RGBA color values

use std::fmt;

/// RGBA color value
///
/// Each component is stored as a u8 (0-255).
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Color {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
    /// Alpha component (0-255, 255 = fully opaque)
    pub a: u8,
}

impl Color {
    /// Black color
    pub const BLACK: Self = Self::rgb(0, 0, 0);

    /// White color
    pub const WHITE: Self = Self::rgb(255, 255, 255);

    /// Red color
    pub const RED: Self = Self::rgb(255, 0, 0);

    /// Green color
    pub const GREEN: Self = Self::rgb(0, 255, 0);

    /// Blue color
    pub const BLUE: Self = Self::rgb(0, 0, 255);

    /// Transparent color
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);

    /// Create an RGB color (fully opaque)
    #[inline]
    #[must_use = "this returns a new value without modifying anything"]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create an RGBA color
    #[inline]
    #[must_use = "this returns a new value without modifying anything"]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from a hex string (e.g., "#FF0000" or "#FF0000FF")
    #[must_use = "this returns a new value without modifying anything"]
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);

        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::rgba(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Convert to hex string (e.g., "#FF0000")
    #[must_use = "the result should be used"]
    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }

    /// Get the red component as a float (0.0-1.0)
    #[inline]
    #[must_use = "the result should be used"]
    pub fn r_f32(&self) -> f32 {
        self.r as f32 / 255.0
    }

    /// Get the green component as a float (0.0-1.0)
    #[inline]
    #[must_use = "the result should be used"]
    pub fn g_f32(&self) -> f32 {
        self.g as f32 / 255.0
    }

    /// Get the blue component as a float (0.0-1.0)
    #[inline]
    #[must_use = "the result should be used"]
    pub fn b_f32(&self) -> f32 {
        self.b as f32 / 255.0
    }

    /// Get the alpha component as a float (0.0-1.0)
    #[inline]
    #[must_use = "the result should be used"]
    pub fn a_f32(&self) -> f32 {
        self.a as f32 / 255.0
    }

    /// Check if the color is fully opaque
    #[inline]
    #[must_use = "the result should be used"]
    pub const fn is_opaque(&self) -> bool {
        self.a == 255
    }

    /// Check if the color is fully transparent
    #[inline]
    #[must_use = "the result should be used"]
    pub const fn is_transparent(&self) -> bool {
        self.a == 0
    }

    /// Parse a color from a CSS/SVG color name (case-insensitive)
    ///
    /// Supports all 140+ CSS3/SVG color keywords.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fop_types::Color;
    /// assert_eq!(Color::from_name("red"), Some(Color::rgb(255, 0, 0)));
    /// assert_eq!(Color::from_name("Blue"), Some(Color::rgb(0, 0, 255)));
    /// assert_eq!(Color::from_name("LIME"), Some(Color::rgb(0, 255, 0)));
    /// assert_eq!(Color::from_name("aliceblue"), Some(Color::rgb(240, 248, 255)));
    /// ```
    #[must_use = "this returns a new value without modifying anything"]
    pub fn from_name(name: &str) -> Option<Self> {
        color_name_to_rgb(name)
    }

    /// Parse a color from a string (hex, or CSS/SVG color name)
    ///
    /// # Examples
    ///
    /// ```
    /// # use fop_types::Color;
    /// assert_eq!(Color::parse("#FF0000"), Some(Color::rgb(255, 0, 0)));
    /// assert_eq!(Color::parse("red"), Some(Color::rgb(255, 0, 0)));
    /// assert_eq!(Color::parse("Blue"), Some(Color::rgb(0, 0, 255)));
    /// ```
    #[must_use = "this returns a new value without modifying anything"]
    pub fn parse(input: &str) -> Option<Self> {
        // Try hex format first
        if let Some(color) = Self::from_hex(input) {
            return Some(color);
        }

        // Try color name
        Self::from_name(input)
    }
}

/// Convert a CSS/SVG color name to RGB values (case-insensitive)
///
/// Supports all 140+ CSS3/SVG color keywords from the W3C specification.
fn color_name_to_rgb(name: &str) -> Option<Color> {
    let name_lower = name.to_lowercase();

    match name_lower.as_str() {
        // Pink colors
        "pink" => Some(Color::rgb(255, 192, 203)),
        "lightpink" => Some(Color::rgb(255, 182, 193)),
        "hotpink" => Some(Color::rgb(255, 105, 180)),
        "deeppink" => Some(Color::rgb(255, 20, 147)),
        "palevioletred" => Some(Color::rgb(219, 112, 147)),
        "mediumvioletred" => Some(Color::rgb(199, 21, 133)),

        // Red colors
        "lightsalmon" => Some(Color::rgb(255, 160, 122)),
        "salmon" => Some(Color::rgb(250, 128, 114)),
        "darksalmon" => Some(Color::rgb(233, 150, 122)),
        "lightcoral" => Some(Color::rgb(240, 128, 128)),
        "indianred" => Some(Color::rgb(205, 92, 92)),
        "crimson" => Some(Color::rgb(220, 20, 60)),
        "firebrick" => Some(Color::rgb(178, 34, 34)),
        "darkred" => Some(Color::rgb(139, 0, 0)),
        "red" => Some(Color::rgb(255, 0, 0)),

        // Orange colors
        "orangered" => Some(Color::rgb(255, 69, 0)),
        "tomato" => Some(Color::rgb(255, 99, 71)),
        "coral" => Some(Color::rgb(255, 127, 80)),
        "darkorange" => Some(Color::rgb(255, 140, 0)),
        "orange" => Some(Color::rgb(255, 165, 0)),

        // Yellow colors
        "yellow" => Some(Color::rgb(255, 255, 0)),
        "lightyellow" => Some(Color::rgb(255, 255, 224)),
        "lemonchiffon" => Some(Color::rgb(255, 250, 205)),
        "lightgoldenrodyellow" => Some(Color::rgb(250, 250, 210)),
        "papayawhip" => Some(Color::rgb(255, 239, 213)),
        "moccasin" => Some(Color::rgb(255, 228, 181)),
        "peachpuff" => Some(Color::rgb(255, 218, 185)),
        "palegoldenrod" => Some(Color::rgb(238, 232, 170)),
        "khaki" => Some(Color::rgb(240, 230, 140)),
        "darkkhaki" => Some(Color::rgb(189, 183, 107)),
        "gold" => Some(Color::rgb(255, 215, 0)),

        // Brown colors
        "cornsilk" => Some(Color::rgb(255, 248, 220)),
        "blanchedalmond" => Some(Color::rgb(255, 235, 205)),
        "bisque" => Some(Color::rgb(255, 228, 196)),
        "navajowhite" => Some(Color::rgb(255, 222, 173)),
        "wheat" => Some(Color::rgb(245, 222, 179)),
        "burlywood" => Some(Color::rgb(222, 184, 135)),
        "tan" => Some(Color::rgb(210, 180, 140)),
        "rosybrown" => Some(Color::rgb(188, 143, 143)),
        "sandybrown" => Some(Color::rgb(244, 164, 96)),
        "goldenrod" => Some(Color::rgb(218, 165, 32)),
        "darkgoldenrod" => Some(Color::rgb(184, 134, 11)),
        "peru" => Some(Color::rgb(205, 133, 63)),
        "chocolate" => Some(Color::rgb(210, 105, 30)),
        "saddlebrown" => Some(Color::rgb(139, 69, 19)),
        "sienna" => Some(Color::rgb(160, 82, 45)),
        "brown" => Some(Color::rgb(165, 42, 42)),
        "maroon" => Some(Color::rgb(128, 0, 0)),

        // Green colors
        "darkolivegreen" => Some(Color::rgb(85, 107, 47)),
        "olive" => Some(Color::rgb(128, 128, 0)),
        "olivedrab" => Some(Color::rgb(107, 142, 35)),
        "yellowgreen" => Some(Color::rgb(154, 205, 50)),
        "limegreen" => Some(Color::rgb(50, 205, 50)),
        "lime" => Some(Color::rgb(0, 255, 0)),
        "lawngreen" => Some(Color::rgb(124, 252, 0)),
        "chartreuse" => Some(Color::rgb(127, 255, 0)),
        "greenyellow" => Some(Color::rgb(173, 255, 47)),
        "springgreen" => Some(Color::rgb(0, 255, 127)),
        "mediumspringgreen" => Some(Color::rgb(0, 250, 154)),
        "lightgreen" => Some(Color::rgb(144, 238, 144)),
        "palegreen" => Some(Color::rgb(152, 251, 152)),
        "darkseagreen" => Some(Color::rgb(143, 188, 143)),
        "mediumseagreen" => Some(Color::rgb(60, 179, 113)),
        "seagreen" => Some(Color::rgb(46, 139, 87)),
        "forestgreen" => Some(Color::rgb(34, 139, 34)),
        "green" => Some(Color::rgb(0, 128, 0)),
        "darkgreen" => Some(Color::rgb(0, 100, 0)),

        // Cyan colors
        "mediumaquamarine" => Some(Color::rgb(102, 205, 170)),
        "aqua" => Some(Color::rgb(0, 255, 255)),
        "cyan" => Some(Color::rgb(0, 255, 255)),
        "lightcyan" => Some(Color::rgb(224, 255, 255)),
        "paleturquoise" => Some(Color::rgb(175, 238, 238)),
        "aquamarine" => Some(Color::rgb(127, 255, 212)),
        "turquoise" => Some(Color::rgb(64, 224, 208)),
        "mediumturquoise" => Some(Color::rgb(72, 209, 204)),
        "darkturquoise" => Some(Color::rgb(0, 206, 209)),
        "lightseagreen" => Some(Color::rgb(32, 178, 170)),
        "cadetblue" => Some(Color::rgb(95, 158, 160)),
        "darkcyan" => Some(Color::rgb(0, 139, 139)),
        "teal" => Some(Color::rgb(0, 128, 128)),

        // Blue colors
        "lightsteelblue" => Some(Color::rgb(176, 196, 222)),
        "powderblue" => Some(Color::rgb(176, 224, 230)),
        "lightblue" => Some(Color::rgb(173, 216, 230)),
        "skyblue" => Some(Color::rgb(135, 206, 235)),
        "lightskyblue" => Some(Color::rgb(135, 206, 250)),
        "deepskyblue" => Some(Color::rgb(0, 191, 255)),
        "dodgerblue" => Some(Color::rgb(30, 144, 255)),
        "cornflowerblue" => Some(Color::rgb(100, 149, 237)),
        "steelblue" => Some(Color::rgb(70, 130, 180)),
        "royalblue" => Some(Color::rgb(65, 105, 225)),
        "blue" => Some(Color::rgb(0, 0, 255)),
        "mediumblue" => Some(Color::rgb(0, 0, 205)),
        "darkblue" => Some(Color::rgb(0, 0, 139)),
        "navy" => Some(Color::rgb(0, 0, 128)),
        "midnightblue" => Some(Color::rgb(25, 25, 112)),

        // Purple/Violet/Magenta colors
        "lavender" => Some(Color::rgb(230, 230, 250)),
        "thistle" => Some(Color::rgb(216, 191, 216)),
        "plum" => Some(Color::rgb(221, 160, 221)),
        "violet" => Some(Color::rgb(238, 130, 238)),
        "orchid" => Some(Color::rgb(218, 112, 214)),
        "fuchsia" => Some(Color::rgb(255, 0, 255)),
        "magenta" => Some(Color::rgb(255, 0, 255)),
        "mediumorchid" => Some(Color::rgb(186, 85, 211)),
        "mediumpurple" => Some(Color::rgb(147, 112, 219)),
        "blueviolet" => Some(Color::rgb(138, 43, 226)),
        "darkviolet" => Some(Color::rgb(148, 0, 211)),
        "darkorchid" => Some(Color::rgb(153, 50, 204)),
        "darkmagenta" => Some(Color::rgb(139, 0, 139)),
        "purple" => Some(Color::rgb(128, 0, 128)),
        "indigo" => Some(Color::rgb(75, 0, 130)),
        "darkslateblue" => Some(Color::rgb(72, 61, 139)),
        "slateblue" => Some(Color::rgb(106, 90, 205)),
        "mediumslateblue" => Some(Color::rgb(123, 104, 238)),

        // White colors
        "white" => Some(Color::rgb(255, 255, 255)),
        "snow" => Some(Color::rgb(255, 250, 250)),
        "honeydew" => Some(Color::rgb(240, 255, 240)),
        "mintcream" => Some(Color::rgb(245, 255, 250)),
        "azure" => Some(Color::rgb(240, 255, 255)),
        "aliceblue" => Some(Color::rgb(240, 248, 255)),
        "ghostwhite" => Some(Color::rgb(248, 248, 255)),
        "whitesmoke" => Some(Color::rgb(245, 245, 245)),
        "seashell" => Some(Color::rgb(255, 245, 238)),
        "beige" => Some(Color::rgb(245, 245, 220)),
        "oldlace" => Some(Color::rgb(253, 245, 230)),
        "floralwhite" => Some(Color::rgb(255, 250, 240)),
        "ivory" => Some(Color::rgb(255, 255, 240)),
        "antiquewhite" => Some(Color::rgb(250, 235, 215)),
        "linen" => Some(Color::rgb(250, 240, 230)),
        "lavenderblush" => Some(Color::rgb(255, 240, 245)),
        "mistyrose" => Some(Color::rgb(255, 228, 225)),

        // Gray/Black colors
        "gainsboro" => Some(Color::rgb(220, 220, 220)),
        "lightgray" | "lightgrey" => Some(Color::rgb(211, 211, 211)),
        "silver" => Some(Color::rgb(192, 192, 192)),
        "darkgray" | "darkgrey" => Some(Color::rgb(169, 169, 169)),
        "gray" | "grey" => Some(Color::rgb(128, 128, 128)),
        "dimgray" | "dimgrey" => Some(Color::rgb(105, 105, 105)),
        "lightslategray" | "lightslategrey" => Some(Color::rgb(119, 136, 153)),
        "slategray" | "slategrey" => Some(Color::rgb(112, 128, 144)),
        "darkslategray" | "darkslategrey" => Some(Color::rgb(47, 79, 79)),
        "black" => Some(Color::rgb(0, 0, 0)),

        // Special
        "transparent" => Some(Color::rgba(0, 0, 0, 0)),

        _ => None,
    }
}

impl fmt::Debug for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Color({})", self.to_hex())
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(Color::BLACK, Color::rgb(0, 0, 0));
        assert_eq!(Color::WHITE, Color::rgb(255, 255, 255));
        assert_eq!(Color::RED, Color::rgb(255, 0, 0));
        assert_eq!(Color::GREEN, Color::rgb(0, 255, 0));
        assert_eq!(Color::BLUE, Color::rgb(0, 0, 255));
    }

    #[test]
    fn test_rgb_creation() {
        let color = Color::rgb(128, 64, 32);
        assert_eq!(color.r, 128);
        assert_eq!(color.g, 64);
        assert_eq!(color.b, 32);
        assert_eq!(color.a, 255);
    }

    #[test]
    fn test_rgba_creation() {
        let color = Color::rgba(128, 64, 32, 128);
        assert_eq!(color.r, 128);
        assert_eq!(color.g, 64);
        assert_eq!(color.b, 32);
        assert_eq!(color.a, 128);
    }

    #[test]
    fn test_hex_parsing() {
        assert_eq!(Color::from_hex("#FF0000"), Some(Color::rgb(255, 0, 0)));
        assert_eq!(Color::from_hex("00FF00"), Some(Color::rgb(0, 255, 0)));
        assert_eq!(
            Color::from_hex("#0000FFAA"),
            Some(Color::rgba(0, 0, 255, 170))
        );
        assert_eq!(Color::from_hex("invalid"), None);
    }

    #[test]
    fn test_hex_output() {
        assert_eq!(Color::rgb(255, 0, 0).to_hex(), "#FF0000");
        assert_eq!(Color::rgba(0, 255, 0, 128).to_hex(), "#00FF0080");
    }

    #[test]
    fn test_float_conversion() {
        let color = Color::rgb(255, 128, 0);
        assert!((color.r_f32() - 1.0).abs() < 0.01);
        assert!((color.g_f32() - 0.502).abs() < 0.01);
        assert!((color.b_f32() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_transparency() {
        assert!(Color::rgb(255, 0, 0).is_opaque());
        assert!(!Color::rgb(255, 0, 0).is_transparent());
        assert!(Color::rgba(0, 0, 0, 0).is_transparent());
        assert!(!Color::rgba(0, 0, 0, 0).is_opaque());
    }

    #[test]
    fn test_display() {
        let red = Color::rgb(255, 0, 0);
        assert_eq!(format!("{}", red), "#FF0000");

        let green_transparent = Color::rgba(0, 255, 0, 128);
        assert_eq!(format!("{}", green_transparent), "#00FF0080");

        let black = Color::BLACK;
        assert_eq!(format!("{}", black), "#000000");

        let white = Color::WHITE;
        assert_eq!(format!("{}", white), "#FFFFFF");

        let transparent = Color::TRANSPARENT;
        assert_eq!(format!("{}", transparent), "#00000000");
    }

    #[test]
    fn test_display_edge_cases() {
        // Test all zeros
        let color = Color::rgb(0, 0, 0);
        assert_eq!(format!("{}", color), "#000000");

        // Test all max
        let color = Color::rgb(255, 255, 255);
        assert_eq!(format!("{}", color), "#FFFFFF");

        // Test partial transparency
        let color = Color::rgba(128, 64, 32, 200);
        assert_eq!(format!("{}", color), "#804020C8");

        // Test single digit hex values
        let color = Color::rgb(15, 15, 15);
        assert_eq!(format!("{}", color), "#0F0F0F");
    }
}

#[cfg(test)]
mod color_extra_tests {
    use super::*;

    // --- from_hex ---

    #[test]
    fn test_hex_lowercase() {
        let c = Color::from_hex("ff0000").expect("test: should succeed");
        assert_eq!(c, Color::RED);
    }

    #[test]
    fn test_hex_mixed_case() {
        let c = Color::from_hex("Ff0000").expect("test: should succeed");
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn test_hex_with_hash_blue() {
        let c = Color::from_hex("#0000ff").expect("test: should succeed");
        assert_eq!(c, Color::BLUE);
    }

    #[test]
    fn test_hex_8digit_full_alpha() {
        let c = Color::from_hex("#FF0000FF").expect("test: should succeed");
        assert_eq!(c.r, 255);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_hex_8digit_half_alpha() {
        let c = Color::from_hex("#00FF0080").expect("test: should succeed");
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 255);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 128);
    }

    #[test]
    fn test_hex_8digit_transparent() {
        let c = Color::from_hex("#00000000").expect("test: should succeed");
        assert!(c.is_transparent());
    }

    #[test]
    fn test_hex_invalid_short() {
        assert!(Color::from_hex("fff").is_none());
    }

    #[test]
    fn test_hex_invalid_non_hex_chars() {
        assert!(Color::from_hex("GGGGGG").is_none());
    }

    #[test]
    fn test_hex_empty_string() {
        assert!(Color::from_hex("").is_none());
    }

    #[test]
    fn test_hex_wrong_length_5() {
        assert!(Color::from_hex("FFFFF").is_none());
    }

    #[test]
    fn test_hex_wrong_length_7() {
        assert!(Color::from_hex("FFFFFFF").is_none());
    }

    // --- from_name ---

    #[test]
    fn test_name_red_lowercase() {
        assert_eq!(Color::from_name("red"), Some(Color::rgb(255, 0, 0)));
    }

    #[test]
    fn test_name_red_uppercase() {
        assert_eq!(Color::from_name("RED"), Some(Color::rgb(255, 0, 0)));
    }

    #[test]
    fn test_name_red_mixed_case() {
        assert_eq!(Color::from_name("Red"), Some(Color::rgb(255, 0, 0)));
    }

    #[test]
    fn test_name_white() {
        assert_eq!(Color::from_name("white"), Some(Color::rgb(255, 255, 255)));
    }

    #[test]
    fn test_name_black() {
        assert_eq!(Color::from_name("black"), Some(Color::rgb(0, 0, 0)));
    }

    #[test]
    fn test_name_blue() {
        assert_eq!(Color::from_name("blue"), Some(Color::rgb(0, 0, 255)));
    }

    #[test]
    fn test_name_green() {
        assert_eq!(Color::from_name("green"), Some(Color::rgb(0, 128, 0)));
    }

    #[test]
    fn test_name_lime() {
        // CSS "lime" = rgb(0, 255, 0), not "green"
        assert_eq!(Color::from_name("lime"), Some(Color::rgb(0, 255, 0)));
    }

    #[test]
    fn test_name_transparent() {
        let c = Color::from_name("transparent").expect("test: should succeed");
        assert!(c.is_transparent());
    }

    #[test]
    fn test_name_aliceblue() {
        assert_eq!(
            Color::from_name("aliceblue"),
            Some(Color::rgb(240, 248, 255))
        );
    }

    #[test]
    fn test_name_orange() {
        assert_eq!(Color::from_name("orange"), Some(Color::rgb(255, 165, 0)));
    }

    #[test]
    fn test_name_yellow() {
        assert_eq!(Color::from_name("yellow"), Some(Color::rgb(255, 255, 0)));
    }

    #[test]
    fn test_name_cyan() {
        assert_eq!(Color::from_name("cyan"), Some(Color::rgb(0, 255, 255)));
    }

    #[test]
    fn test_name_aqua_equals_cyan() {
        // "aqua" and "cyan" are aliases
        assert_eq!(Color::from_name("aqua"), Color::from_name("cyan"));
    }

    #[test]
    fn test_name_fuchsia_equals_magenta() {
        assert_eq!(Color::from_name("fuchsia"), Color::from_name("magenta"));
    }

    #[test]
    fn test_name_gray_aliases() {
        assert_eq!(Color::from_name("gray"), Color::from_name("grey"));
        assert_eq!(Color::from_name("darkgray"), Color::from_name("darkgrey"));
        assert_eq!(Color::from_name("lightgray"), Color::from_name("lightgrey"));
    }

    #[test]
    fn test_name_unknown_returns_none() {
        assert!(Color::from_name("notacolor").is_none());
        assert!(Color::from_name("").is_none());
        assert!(Color::from_name("12345").is_none());
    }

    #[test]
    fn test_name_silver() {
        assert_eq!(Color::from_name("silver"), Some(Color::rgb(192, 192, 192)));
    }

    #[test]
    fn test_name_gold() {
        assert_eq!(Color::from_name("gold"), Some(Color::rgb(255, 215, 0)));
    }

    #[test]
    fn test_name_navy() {
        assert_eq!(Color::from_name("navy"), Some(Color::rgb(0, 0, 128)));
    }

    #[test]
    fn test_name_maroon() {
        assert_eq!(Color::from_name("maroon"), Some(Color::rgb(128, 0, 0)));
    }

    #[test]
    fn test_name_purple() {
        assert_eq!(Color::from_name("purple"), Some(Color::rgb(128, 0, 128)));
    }

    #[test]
    fn test_name_teal() {
        assert_eq!(Color::from_name("teal"), Some(Color::rgb(0, 128, 128)));
    }

    // --- parse ---

    #[test]
    fn test_parse_hex_string() {
        assert_eq!(Color::parse("#FF0000"), Some(Color::RED));
    }

    #[test]
    fn test_parse_name_string() {
        assert_eq!(Color::parse("red"), Some(Color::RED));
    }

    #[test]
    fn test_parse_unknown_returns_none() {
        assert!(Color::parse("notacolor").is_none());
    }

    #[test]
    fn test_parse_hex_no_hash() {
        assert_eq!(Color::parse("0000FF"), Some(Color::BLUE));
    }

    // --- to_hex ---

    #[test]
    fn test_to_hex_opaque() {
        let c = Color::rgb(0, 128, 255);
        assert_eq!(c.to_hex(), "#0080FF");
    }

    #[test]
    fn test_to_hex_with_alpha() {
        let c = Color::rgba(255, 0, 0, 128);
        assert_eq!(c.to_hex(), "#FF000080");
    }

    #[test]
    fn test_to_hex_fully_transparent() {
        let c = Color::rgba(0, 0, 0, 0);
        assert_eq!(c.to_hex(), "#00000000");
    }

    // --- float conversions ---

    #[test]
    fn test_r_f32_max() {
        let c = Color::rgb(255, 0, 0);
        assert!((c.r_f32() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_g_f32_mid() {
        let c = Color::rgb(0, 128, 0);
        assert!((c.g_f32() - 0.502).abs() < 0.01);
    }

    #[test]
    fn test_b_f32_zero() {
        let c = Color::rgb(100, 100, 0);
        assert!((c.b_f32() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_a_f32_full() {
        let c = Color::rgb(0, 0, 0);
        assert!((c.a_f32() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_a_f32_zero() {
        let c = Color::rgba(0, 0, 0, 0);
        assert!((c.a_f32() - 0.0).abs() < 0.01);
    }

    // --- is_opaque / is_transparent ---

    #[test]
    fn test_is_opaque_full_alpha() {
        assert!(Color::rgb(100, 100, 100).is_opaque());
    }

    #[test]
    fn test_is_not_opaque_partial_alpha() {
        assert!(!Color::rgba(100, 100, 100, 200).is_opaque());
    }

    #[test]
    fn test_is_transparent_zero_alpha() {
        assert!(Color::TRANSPARENT.is_transparent());
    }

    #[test]
    fn test_is_not_transparent_full_alpha() {
        assert!(!Color::BLACK.is_transparent());
    }

    // --- constants ---

    #[test]
    fn test_constant_black() {
        assert_eq!(Color::BLACK.r, 0);
        assert_eq!(Color::BLACK.g, 0);
        assert_eq!(Color::BLACK.b, 0);
        assert_eq!(Color::BLACK.a, 255);
    }

    #[test]
    fn test_constant_white() {
        assert_eq!(Color::WHITE.r, 255);
        assert_eq!(Color::WHITE.g, 255);
        assert_eq!(Color::WHITE.b, 255);
        assert_eq!(Color::WHITE.a, 255);
    }

    #[test]
    fn test_constant_transparent() {
        assert_eq!(Color::TRANSPARENT.a, 0);
    }

    // --- equality ---

    #[test]
    fn test_equality_same_rgb() {
        assert_eq!(Color::rgb(10, 20, 30), Color::rgb(10, 20, 30));
    }

    #[test]
    fn test_inequality_different_rgb() {
        assert_ne!(Color::rgb(10, 20, 30), Color::rgb(10, 20, 31));
    }

    #[test]
    fn test_rgba_alpha_distinguishes() {
        assert_ne!(Color::rgba(255, 0, 0, 255), Color::rgba(255, 0, 0, 0));
    }
}

#[cfg(test)]
mod color_parsing_tests {
    use super::*;

    // --- Hex parsing: 6-digit ---

    #[test]
    fn test_hex_6digit_red() {
        let c = Color::from_hex("#FF0000").expect("test: should succeed");
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_hex_6digit_green() {
        let c = Color::from_hex("#00FF00").expect("test: should succeed");
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 255);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn test_hex_6digit_blue() {
        let c = Color::from_hex("#0000FF").expect("test: should succeed");
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 255);
    }

    #[test]
    fn test_hex_6digit_orange() {
        // #ff8800 = 255, 136, 0
        let c = Color::from_hex("#ff8800").expect("test: should succeed");
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 136);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn test_hex_6digit_no_hash() {
        let c = Color::from_hex("FF0000").expect("test: should succeed");
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn test_hex_6digit_lowercase() {
        let c = Color::from_hex("ff8800").expect("test: should succeed");
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 136);
        assert_eq!(c.b, 0);
    }

    // --- Hex parsing: 8-digit (with alpha) ---

    #[test]
    fn test_hex_8digit_red_opaque() {
        let c = Color::from_hex("#FF0000FF").expect("test: should succeed");
        assert_eq!(c.r, 255);
        assert_eq!(c.a, 255);
        assert!(c.is_opaque());
    }

    #[test]
    fn test_hex_8digit_half_transparent() {
        let c = Color::from_hex("#00000080").expect("test: should succeed");
        assert_eq!(c.r, 0);
        assert_eq!(c.a, 128);
        assert!(!c.is_opaque());
        assert!(!c.is_transparent());
    }

    #[test]
    fn test_hex_8digit_fully_transparent() {
        let c = Color::from_hex("#00000000").expect("test: should succeed");
        assert!(c.is_transparent());
    }

    // --- Hex parsing: invalid formats ---

    #[test]
    fn test_hex_3digit_returns_none() {
        // 3-digit shorthand (#rgb) is NOT supported by from_hex
        assert!(Color::from_hex("#f00").is_none());
    }

    #[test]
    fn test_hex_invalid_chars_returns_none() {
        assert!(Color::from_hex("#GGGGGG").is_none());
    }

    #[test]
    fn test_hex_empty_returns_none() {
        assert!(Color::from_hex("").is_none());
    }

    #[test]
    fn test_hex_too_short_returns_none() {
        assert!(Color::from_hex("#FFF").is_none());
    }

    #[test]
    fn test_hex_too_long_returns_none() {
        assert!(Color::from_hex("#FFFFFFFFF").is_none());
    }

    // --- Named color parsing ---

    #[test]
    fn test_named_black() {
        let c = Color::from_name("black").expect("test: should succeed");
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
        assert!(c.is_opaque());
    }

    #[test]
    fn test_named_white() {
        let c = Color::from_name("white").expect("test: should succeed");
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 255);
        assert_eq!(c.b, 255);
    }

    #[test]
    fn test_named_red() {
        let c = Color::from_name("red").expect("test: should succeed");
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn test_named_green_is_128_not_255() {
        // CSS "green" = #008000, NOT lime (#00FF00)
        let c = Color::from_name("green").expect("test: should succeed");
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn test_named_lime_is_255() {
        let c = Color::from_name("lime").expect("test: should succeed");
        assert_eq!(c.g, 255);
    }

    #[test]
    fn test_named_blue() {
        let c = Color::from_name("blue").expect("test: should succeed");
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 255);
    }

    #[test]
    fn test_named_case_insensitive_upper() {
        let c = Color::from_name("RED").expect("test: should succeed");
        assert_eq!(c.r, 255);
    }

    #[test]
    fn test_named_case_insensitive_mixed() {
        let c = Color::from_name("BlUe").expect("test: should succeed");
        assert_eq!(c.b, 255);
    }

    #[test]
    fn test_named_transparent() {
        let c = Color::from_name("transparent").expect("test: should succeed");
        assert!(c.is_transparent());
    }

    #[test]
    fn test_named_unknown_returns_none() {
        assert!(Color::from_name("not-a-color").is_none());
        assert!(Color::from_name("rgb(255, 0, 0)").is_none());
        assert!(Color::from_name("#FF0000").is_none());
    }

    #[test]
    fn test_named_orange() {
        let c = Color::from_name("orange").expect("test: should succeed");
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 165);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn test_named_navy() {
        let c = Color::from_name("navy").expect("test: should succeed");
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 128);
    }

    #[test]
    fn test_named_silver() {
        let c = Color::from_name("silver").expect("test: should succeed");
        assert_eq!(c.r, 192);
        assert_eq!(c.g, 192);
        assert_eq!(c.b, 192);
    }

    #[test]
    fn test_named_gray_and_grey_aliases() {
        assert_eq!(Color::from_name("gray"), Color::from_name("grey"));
    }

    #[test]
    fn test_named_aqua_and_cyan_aliases() {
        assert_eq!(Color::from_name("aqua"), Color::from_name("cyan"));
        let c = Color::from_name("cyan").expect("test: should succeed");
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 255);
        assert_eq!(c.b, 255);
    }

    #[test]
    fn test_named_fuchsia_and_magenta_aliases() {
        assert_eq!(Color::from_name("fuchsia"), Color::from_name("magenta"));
    }

    // --- parse() convenience method ---

    #[test]
    fn test_parse_hex_string() {
        let c = Color::parse("#FF0000").expect("test: should succeed");
        assert_eq!(c.r, 255);
    }

    #[test]
    fn test_parse_named_string() {
        let c = Color::parse("blue").expect("test: should succeed");
        assert_eq!(c.b, 255);
    }

    #[test]
    fn test_parse_unknown_returns_none() {
        assert!(Color::parse("not-a-color").is_none());
    }

    // --- float accessors ---

    #[test]
    fn test_r_f32_full() {
        let c = Color::rgb(255, 0, 0);
        assert!((c.r_f32() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_g_f32_zero() {
        let c = Color::rgb(0, 0, 0);
        assert!((c.g_f32() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_b_f32_mid() {
        // 128 / 255 ≈ 0.502
        let c = Color::rgb(0, 0, 128);
        assert!((c.b_f32() - 0.502).abs() < 0.01);
    }

    #[test]
    fn test_a_f32_half() {
        let c = Color::rgba(0, 0, 0, 128);
        assert!((c.a_f32() - 0.502).abs() < 0.01);
    }

    // --- to_hex ---

    #[test]
    fn test_to_hex_uppercase() {
        let c = Color::rgb(255, 0, 0);
        assert_eq!(c.to_hex(), "#FF0000");
    }

    #[test]
    fn test_to_hex_with_alpha_channel() {
        let c = Color::rgba(0, 255, 0, 128);
        let hex = c.to_hex();
        assert_eq!(hex, "#00FF0080");
    }

    #[test]
    fn test_to_hex_transparent() {
        let hex = Color::TRANSPARENT.to_hex();
        assert_eq!(hex, "#00000000");
    }

    // --- is_opaque / is_transparent ---

    #[test]
    fn test_rgb_is_opaque() {
        assert!(Color::rgb(100, 100, 100).is_opaque());
    }

    #[test]
    fn test_rgba_partial_alpha_not_opaque_not_transparent() {
        let c = Color::rgba(128, 128, 128, 127);
        assert!(!c.is_opaque());
        assert!(!c.is_transparent());
    }

    #[test]
    fn test_transparent_constant_is_transparent() {
        assert!(Color::TRANSPARENT.is_transparent());
        assert!(!Color::TRANSPARENT.is_opaque());
    }
}
