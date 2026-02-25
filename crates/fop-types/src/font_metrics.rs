//! Font metrics for text measurement
//!
//! Provides character width information for the 14 standard PDF fonts:
//! - Helvetica family: Helvetica, Helvetica-Bold, Helvetica-Oblique, Helvetica-BoldOblique
//! - Times family: Times-Roman, Times-Bold, Times-Italic, Times-BoldItalic
//! - Courier family: Courier, Courier-Bold, Courier-Oblique, Courier-BoldOblique
//! - Symbol (Greek letters and mathematical symbols)
//! - ZapfDingbats (decorative characters and symbols)

use crate::Length;
use std::collections::HashMap;
use std::fmt;

/// Font metrics for a single font
#[derive(Debug, Clone)]
pub struct FontMetrics {
    /// Font name (e.g., "Helvetica", "Times-Roman")
    pub name: String,

    /// Character widths in font units (1/1000 of em-square)
    pub char_widths: HashMap<char, u16>,

    /// Default character width (for missing characters)
    pub default_width: u16,

    /// Cap height (height of capital letters)
    pub cap_height: u16,

    /// X-height (height of lowercase 'x')
    pub x_height: u16,

    /// Ascender (height above baseline)
    pub ascender: i16,

    /// Descender (depth below baseline, typically negative)
    pub descender: i16,
}

impl FontMetrics {
    /// Get the width of a character in font units
    pub fn char_width(&self, c: char) -> u16 {
        *self.char_widths.get(&c).unwrap_or(&self.default_width)
    }

    /// Measure the width of text at a given font size
    pub fn measure_text(&self, text: &str, font_size: Length) -> Length {
        let mut total_width = 0;
        for c in text.chars() {
            total_width += self.char_width(c) as i32;
        }

        // Convert from font units to points: (total_width / 1000) * font_size
        let width_pt = (total_width as f64 / 1000.0) * font_size.to_pt();
        Length::from_pt(width_pt)
    }

    /// Create Helvetica font metrics (simplified - common characters only)
    pub fn helvetica() -> Self {
        let mut char_widths = HashMap::new();

        // ASCII letters and common punctuation (approximate widths)
        // These are simplified values; real AFM files have exact measurements
        let width_data = [
            (' ', 278),
            ('!', 278),
            ('"', 355),
            ('#', 556),
            ('$', 556),
            ('%', 889),
            ('&', 667),
            ('\'', 191),
            ('(', 333),
            (')', 333),
            ('*', 389),
            ('+', 584),
            (',', 278),
            ('-', 333),
            ('.', 278),
            ('/', 278),
            ('0', 556),
            ('1', 556),
            ('2', 556),
            ('3', 556),
            ('4', 556),
            ('5', 556),
            ('6', 556),
            ('7', 556),
            ('8', 556),
            ('9', 556),
            (':', 278),
            (';', 278),
            ('<', 584),
            ('=', 584),
            ('>', 584),
            ('?', 556),
            ('@', 1015),
            ('A', 667),
            ('B', 667),
            ('C', 722),
            ('D', 722),
            ('E', 667),
            ('F', 611),
            ('G', 778),
            ('H', 722),
            ('I', 278),
            ('J', 500),
            ('K', 667),
            ('L', 556),
            ('M', 833),
            ('N', 722),
            ('O', 778),
            ('P', 667),
            ('Q', 778),
            ('R', 722),
            ('S', 667),
            ('T', 611),
            ('U', 722),
            ('V', 667),
            ('W', 944),
            ('X', 667),
            ('Y', 667),
            ('Z', 611),
            ('[', 278),
            ('\\', 278),
            (']', 278),
            ('^', 469),
            ('_', 556),
            ('`', 333),
            ('a', 556),
            ('b', 556),
            ('c', 500),
            ('d', 556),
            ('e', 556),
            ('f', 278),
            ('g', 556),
            ('h', 556),
            ('i', 222),
            ('j', 222),
            ('k', 500),
            ('l', 222),
            ('m', 833),
            ('n', 556),
            ('o', 556),
            ('p', 556),
            ('q', 556),
            ('r', 333),
            ('s', 500),
            ('t', 278),
            ('u', 556),
            ('v', 500),
            ('w', 722),
            ('x', 500),
            ('y', 500),
            ('z', 500),
            ('{', 334),
            ('|', 260),
            ('}', 334),
            ('~', 584),
        ];

        for (c, width) in width_data {
            char_widths.insert(c, width);
        }

        Self {
            name: "Helvetica".to_string(),
            char_widths,
            default_width: 500, // Average width for unknown characters
            cap_height: 718,
            x_height: 523,
            ascender: 718,
            descender: -207,
        }
    }

    /// Create Times-Roman font metrics (simplified)
    pub fn times_roman() -> Self {
        let mut char_widths = HashMap::new();

        let width_data = [
            (' ', 250),
            ('!', 333),
            ('"', 408),
            ('#', 500),
            ('$', 500),
            ('%', 833),
            ('&', 778),
            ('\'', 180),
            ('(', 333),
            (')', 333),
            ('*', 500),
            ('+', 564),
            (',', 250),
            ('-', 333),
            ('.', 250),
            ('/', 278),
            ('0', 500),
            ('1', 500),
            ('2', 500),
            ('3', 500),
            ('4', 500),
            ('5', 500),
            ('6', 500),
            ('7', 500),
            ('8', 500),
            ('9', 500),
            (':', 278),
            (';', 278),
            ('<', 564),
            ('=', 564),
            ('>', 564),
            ('?', 444),
            ('@', 921),
            ('A', 722),
            ('B', 667),
            ('C', 667),
            ('D', 722),
            ('E', 611),
            ('F', 556),
            ('G', 722),
            ('H', 722),
            ('I', 333),
            ('J', 389),
            ('K', 722),
            ('L', 611),
            ('M', 889),
            ('N', 722),
            ('O', 722),
            ('P', 556),
            ('Q', 722),
            ('R', 667),
            ('S', 556),
            ('T', 611),
            ('U', 722),
            ('V', 722),
            ('W', 944),
            ('X', 722),
            ('Y', 722),
            ('Z', 611),
            ('[', 333),
            ('\\', 278),
            (']', 333),
            ('^', 469),
            ('_', 500),
            ('`', 333),
            ('a', 444),
            ('b', 500),
            ('c', 444),
            ('d', 500),
            ('e', 444),
            ('f', 333),
            ('g', 500),
            ('h', 500),
            ('i', 278),
            ('j', 278),
            ('k', 500),
            ('l', 278),
            ('m', 778),
            ('n', 500),
            ('o', 500),
            ('p', 500),
            ('q', 500),
            ('r', 333),
            ('s', 389),
            ('t', 278),
            ('u', 500),
            ('v', 500),
            ('w', 722),
            ('x', 500),
            ('y', 500),
            ('z', 444),
            ('{', 480),
            ('|', 200),
            ('}', 480),
            ('~', 541),
        ];

        for (c, width) in width_data {
            char_widths.insert(c, width);
        }

        Self {
            name: "Times-Roman".to_string(),
            char_widths,
            default_width: 444,
            cap_height: 662,
            x_height: 450,
            ascender: 683,
            descender: -217,
        }
    }

    /// Create Courier font metrics (monospace - all characters have same width)
    pub fn courier() -> Self {
        let mut char_widths = HashMap::new();

        // Courier is monospace - all characters have width 600
        let width_data = [
            (' ', 600),
            ('!', 600),
            ('"', 600),
            ('#', 600),
            ('$', 600),
            ('%', 600),
            ('&', 600),
            ('\'', 600),
            ('(', 600),
            (')', 600),
            ('*', 600),
            ('+', 600),
            (',', 600),
            ('-', 600),
            ('.', 600),
            ('/', 600),
            ('0', 600),
            ('1', 600),
            ('2', 600),
            ('3', 600),
            ('4', 600),
            ('5', 600),
            ('6', 600),
            ('7', 600),
            ('8', 600),
            ('9', 600),
            (':', 600),
            (';', 600),
            ('<', 600),
            ('=', 600),
            ('>', 600),
            ('?', 600),
            ('@', 600),
            ('A', 600),
            ('B', 600),
            ('C', 600),
            ('D', 600),
            ('E', 600),
            ('F', 600),
            ('G', 600),
            ('H', 600),
            ('I', 600),
            ('J', 600),
            ('K', 600),
            ('L', 600),
            ('M', 600),
            ('N', 600),
            ('O', 600),
            ('P', 600),
            ('Q', 600),
            ('R', 600),
            ('S', 600),
            ('T', 600),
            ('U', 600),
            ('V', 600),
            ('W', 600),
            ('X', 600),
            ('Y', 600),
            ('Z', 600),
            ('[', 600),
            ('\\', 600),
            (']', 600),
            ('^', 600),
            ('_', 600),
            ('`', 600),
            ('a', 600),
            ('b', 600),
            ('c', 600),
            ('d', 600),
            ('e', 600),
            ('f', 600),
            ('g', 600),
            ('h', 600),
            ('i', 600),
            ('j', 600),
            ('k', 600),
            ('l', 600),
            ('m', 600),
            ('n', 600),
            ('o', 600),
            ('p', 600),
            ('q', 600),
            ('r', 600),
            ('s', 600),
            ('t', 600),
            ('u', 600),
            ('v', 600),
            ('w', 600),
            ('x', 600),
            ('y', 600),
            ('z', 600),
            ('{', 600),
            ('|', 600),
            ('}', 600),
            ('~', 600),
        ];

        for (c, width) in width_data {
            char_widths.insert(c, width);
        }

        Self {
            name: "Courier".to_string(),
            char_widths,
            default_width: 600,
            cap_height: 562,
            x_height: 426,
            ascender: 629,
            descender: -157,
        }
    }

    /// Create Symbol font metrics (Greek letters and math symbols)
    pub fn symbol() -> Self {
        let mut char_widths = HashMap::new();

        // Symbol font character mappings and widths
        let width_data = [
            (' ', 250),
            ('!', 333),
            ('"', 713),
            ('#', 500),
            ('$', 549),
            ('%', 833),
            ('&', 778),
            ('\'', 439),
            ('(', 333),
            (')', 333),
            ('*', 500),
            ('+', 549),
            (',', 250),
            ('-', 549),
            ('.', 250),
            ('/', 278),
            ('0', 500),
            ('1', 500),
            ('2', 500),
            ('3', 500),
            ('4', 500),
            ('5', 500),
            ('6', 500),
            ('7', 500),
            ('8', 500),
            ('9', 500),
            (':', 278),
            (';', 278),
            ('<', 549),
            ('=', 549),
            ('>', 549),
            ('?', 444),
            ('@', 549),
            // Greek uppercase letters
            ('Α', 722), // Alpha
            ('Β', 667), // Beta
            ('Γ', 603), // Gamma
            ('Δ', 612), // Delta
            ('Ε', 611), // Epsilon
            ('Ζ', 611), // Zeta
            ('Η', 722), // Eta
            ('Θ', 741), // Theta
            ('Ι', 333), // Iota
            ('Κ', 722), // Kappa
            ('Λ', 686), // Lambda
            ('Μ', 889), // Mu
            ('Ν', 722), // Nu
            ('Ξ', 645), // Xi
            ('Ο', 722), // Omicron
            ('Π', 768), // Pi
            ('Ρ', 556), // Rho
            ('Σ', 592), // Sigma
            ('Τ', 611), // Tau
            ('Υ', 690), // Upsilon
            ('Φ', 763), // Phi
            ('Χ', 722), // Chi
            ('Ψ', 795), // Psi
            ('Ω', 768), // Omega
            // Greek lowercase letters
            ('α', 631), // alpha
            ('β', 549), // beta
            ('γ', 411), // gamma
            ('δ', 494), // delta
            ('ε', 439), // epsilon
            ('ζ', 494), // zeta
            ('η', 603), // eta
            ('θ', 521), // theta
            ('ι', 329), // iota
            ('κ', 549), // kappa
            ('λ', 549), // lambda
            ('μ', 576), // mu
            ('ν', 521), // nu
            ('ξ', 493), // xi
            ('ο', 549), // omicron
            ('π', 549), // pi
            ('ρ', 549), // rho
            ('σ', 603), // sigma
            ('τ', 439), // tau
            ('υ', 576), // upsilon
            ('φ', 521), // phi
            ('χ', 549), // chi
            ('ψ', 686), // psi
            ('ω', 686), // omega
            // Math symbols
            ('∀', 713), // forall
            ('∂', 494), // partial
            ('∃', 549), // exists
            ('∅', 549), // empty set
            ('∆', 612), // increment
            ('∇', 713), // nabla
            ('∈', 713), // element of
            ('∉', 713), // not element of
            ('∏', 823), // product
            ('∑', 713), // summation
            ('−', 549), // minus
            ('∗', 500), // asterisk operator
            ('√', 549), // square root
            ('∝', 713), // proportional
            ('∞', 713), // infinity
            ('∠', 768), // angle
            ('∧', 603), // logical and
            ('∨', 603), // logical or
            ('∩', 768), // intersection
            ('∪', 768), // union
            ('∫', 274), // integral
            ('∴', 863), // therefore
            ('∼', 549), // similar to
            ('≅', 549), // congruent
            ('≈', 549), // approximately equal
            ('≠', 549), // not equal
            ('≡', 549), // identical
            ('≤', 549), // less than or equal
            ('≥', 549), // greater than or equal
            ('⊂', 713), // subset
            ('⊃', 713), // superset
            ('⊄', 713), // not subset
            ('⊆', 713), // subset or equal
            ('⊇', 713), // superset or equal
            ('⊕', 768), // circled plus
            ('⊗', 768), // circled times
            ('⊥', 658), // perpendicular
            ('⋅', 250), // dot operator
            // Brackets and arrows
            ('[', 333),
            (']', 333),
            ('←', 987),  // left arrow
            ('↑', 603),  // up arrow
            ('→', 987),  // right arrow
            ('↓', 603),  // down arrow
            ('↔', 1042), // left right arrow
            ('⇐', 987),  // left double arrow
            ('⇑', 603),  // up double arrow
            ('⇒', 987),  // right double arrow
            ('⇓', 603),  // down double arrow
            ('⇔', 1042), // left right double arrow
        ];

        for (c, width) in width_data {
            char_widths.insert(c, width);
        }

        Self {
            name: "Symbol".to_string(),
            char_widths,
            default_width: 549,
            cap_height: 673,
            x_height: 500,
            ascender: 673,
            descender: -207,
        }
    }

    /// Create ZapfDingbats font metrics (dingbat characters)
    pub fn zapf_dingbats() -> Self {
        let mut char_widths = HashMap::new();

        // ZapfDingbats character mappings and widths
        let width_data = [
            (' ', 278),
            // Common dingbat characters
            ('✁', 974), // scissors
            ('✂', 961), // scissors
            ('✃', 974), // lower blade scissors
            ('✄', 980), // white scissors
            ('✆', 719), // telephone
            ('✇', 789), // tape drive
            ('✈', 790), // airplane
            ('✉', 791), // envelope
            ('✌', 690), // victory hand
            ('✍', 960), // writing hand
            ('✎', 939), // lower right pencil
            ('✏', 549), // pencil
            ('✐', 855), // upper right pencil
            ('✑', 911), // white nib
            ('✒', 933), // black nib
            ('✓', 911), // check mark
            ('✔', 945), // heavy check mark
            ('✕', 974), // multiplication x
            ('✖', 755), // heavy multiplication x
            ('✗', 846), // ballot x
            ('✘', 762), // heavy ballot x
            ('✙', 761), // outlined Greek cross
            ('✚', 571), // heavy Greek cross
            ('✛', 677), // open center cross
            ('✜', 763), // heavy open center cross
            ('✝', 760), // Latin cross
            ('✞', 759), // shadowed white Latin cross
            ('✟', 754), // outlined Latin cross
            ('✠', 494), // Maltese cross
            ('✡', 552), // Star of David
            ('✢', 537), // four teardrop-spoked asterisk
            ('✣', 577), // four balloon-spoked asterisk
            ('✤', 692), // heavy four balloon-spoked asterisk
            ('✥', 786), // four club-spoked asterisk
            ('✦', 788), // black four pointed star
            ('✧', 788), // white four pointed star
            ('★', 790), // black star
            ('☆', 793), // white star
            ('✩', 794), // stress outlined white star
            ('✪', 816), // circled white star
            ('✫', 823), // open center black star
            ('✬', 789), // black center white star
            ('✭', 841), // outlined black star
            ('✮', 823), // heavy outlined black star
            ('✯', 833), // pinwheel star
            ('✰', 816), // shadowed white star
            ('✱', 831), // heavy asterisk
            ('✲', 923), // open center asterisk
            ('✳', 744), // eight spoked asterisk
            ('✴', 723), // eight pointed black star
            ('✵', 749), // eight pointed pinwheel star
            ('✶', 790), // six pointed black star
            ('✷', 792), // eight pointed rectilinear black star
            ('✸', 695), // heavy eight pointed rectilinear black star
            ('✹', 776), // twelve pointed black star
            ('✺', 768), // sixteen pointed asterisk
            ('✻', 792), // teardrop-spoked asterisk
            ('✼', 759), // open center teardrop-spoked asterisk
            ('✽', 707), // heavy teardrop-spoked asterisk
            ('✾', 708), // six petalled black and white florette
            ('✿', 682), // black florette
            ('❀', 701), // white florette
            ('❁', 826), // eight petalled outlined black florette
            ('❂', 815), // circled open center eight pointed star
            ('❃', 789), // heavy teardrop-spoked pinwheel asterisk
            ('❄', 789), // snowflake
            ('❅', 707), // tight trifoliate snowflake
            ('❆', 687), // heavy chevron snowflake
            ('❇', 696), // sparkle
            ('❈', 689), // heavy sparkle
            ('❉', 786), // balloon-spoked asterisk
            ('❊', 787), // eight teardrop-spoked propeller asterisk
            ('❋', 713), // heavy eight teardrop-spoked propeller asterisk
            // Arrows and pointing fingers
            ('☛', 791), // black right pointing index
            ('☞', 785), // white right pointing index
            ('☜', 791), // white left pointing index
            ('☝', 873), // white up pointing index
            ('☟', 761), // white down pointing index
            // Boxes and shapes
            ('□', 762), // white square
            ('■', 762), // black square
            ('▢', 759), // white square with rounded corners
            ('▣', 759), // white square containing black small square
            ('▤', 892), // square with horizontal fill
            ('▥', 892), // square with vertical fill
            ('▦', 788), // square with orthogonal crosshatch fill
            ('▧', 784), // square with upper left to lower right fill
            ('▨', 438), // square with upper right to lower left fill
            ('▩', 138), // square with diagonal crosshatch fill
            ('◆', 277), // black diamond
            ('◇', 415), // white diamond
            ('◈', 392), // white diamond containing black small diamond
            ('○', 392), // white circle
            ('●', 668), // black circle
            ('◐', 668), // circle with left half black
            ('◑', 390), // circle with right half black
            ('◒', 390), // circle with lower half black
            ('◓', 317), // circle with upper half black
            ('◔', 317), // circle with upper right quadrant black
            ('◕', 317), // circle with all but upper left quadrant black
            // Numbers in circles
            ('①', 974), // circled digit one
            ('②', 974), // circled digit two
            ('③', 974), // circled digit three
            ('④', 974), // circled digit four
            ('⑤', 974), // circled digit five
            ('⑥', 974), // circled digit six
            ('⑦', 974), // circled digit seven
            ('⑧', 974), // circled digit eight
            ('⑨', 974), // circled digit nine
            ('⑩', 974), // circled number ten
        ];

        for (c, width) in width_data {
            char_widths.insert(c, width);
        }

        Self {
            name: "ZapfDingbats".to_string(),
            char_widths,
            default_width: 788,
            cap_height: 718,
            x_height: 500,
            ascender: 718,
            descender: -207,
        }
    }
}

impl fmt::Display for FontMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (ascender: {}, descender: {}, cap-height: {}, x-height: {})",
            self.name, self.ascender, self.descender, self.cap_height, self.x_height
        )
    }
}

/// Font metrics registry
pub struct FontRegistry {
    fonts: HashMap<String, FontMetrics>,
}

impl FontRegistry {
    /// Create a new registry with the 14 standard PDF fonts
    pub fn new() -> Self {
        let mut fonts = HashMap::new();

        // Helvetica family (4 fonts)
        let helvetica = FontMetrics::helvetica();
        fonts.insert("Helvetica".to_string(), helvetica.clone());
        fonts.insert("Helvetica-Bold".to_string(), helvetica.clone());
        fonts.insert("Helvetica-Oblique".to_string(), helvetica.clone());
        fonts.insert("Helvetica-BoldOblique".to_string(), helvetica);

        // Times family (4 fonts)
        let times = FontMetrics::times_roman();
        fonts.insert("Times-Roman".to_string(), times.clone());
        fonts.insert("Times-Bold".to_string(), times.clone());
        fonts.insert("Times-Italic".to_string(), times.clone());
        fonts.insert("Times-BoldItalic".to_string(), times);

        // Courier family (4 fonts)
        let courier = FontMetrics::courier();
        fonts.insert("Courier".to_string(), courier.clone());
        fonts.insert("Courier-Bold".to_string(), courier.clone());
        fonts.insert("Courier-Oblique".to_string(), courier.clone());
        fonts.insert("Courier-BoldOblique".to_string(), courier);

        // Symbol font (1 font)
        let symbol = FontMetrics::symbol();
        fonts.insert("Symbol".to_string(), symbol);

        // ZapfDingbats font (1 font)
        let zapf = FontMetrics::zapf_dingbats();
        fonts.insert("ZapfDingbats".to_string(), zapf);

        Self { fonts }
    }

    /// Get font metrics by name
    pub fn get(&self, name: &str) -> Option<&FontMetrics> {
        self.fonts.get(name)
    }

    /// Get font metrics by name, falling back to Helvetica
    pub fn get_or_default(&self, name: &str) -> &FontMetrics {
        self.fonts
            .get(name)
            .or_else(|| self.fonts.get("Helvetica"))
            .expect("Helvetica font should always be available")
    }
}

impl Default for FontRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helvetica_char_width() {
        let metrics = FontMetrics::helvetica();

        assert_eq!(metrics.char_width('A'), 667);
        assert_eq!(metrics.char_width('a'), 556);
        assert_eq!(metrics.char_width(' '), 278);
    }

    #[test]
    fn test_measure_text() {
        let metrics = FontMetrics::helvetica();
        let font_size = Length::from_pt(12.0);

        // "Hello" = H(722) + e(556) + l(222) + l(222) + o(556) = 2278 units
        // At 12pt: 2278 / 1000 * 12 = 27.336pt
        let width = metrics.measure_text("Hello", font_size);

        // Allow small floating point tolerance
        assert!((width.to_pt() - 27.336).abs() < 0.01);
    }

    #[test]
    fn test_unknown_character() {
        let metrics = FontMetrics::helvetica();

        // Unicode character not in our table should use default width
        assert_eq!(metrics.char_width('€'), 500);
    }

    #[test]
    fn test_font_registry() {
        let registry = FontRegistry::new();

        assert!(registry.get("Helvetica").is_some());
        assert!(registry.get("Times-Roman").is_some());
        assert!(registry.get("NonExistent").is_none());
    }

    #[test]
    fn test_get_or_default() {
        let registry = FontRegistry::new();

        let metrics = registry.get_or_default("NonExistent");
        assert_eq!(metrics.name, "Helvetica");
    }

    #[test]
    fn test_times_roman() {
        let metrics = FontMetrics::times_roman();

        assert_eq!(metrics.char_width('A'), 722);
        assert_eq!(metrics.char_width('a'), 444);
        assert_eq!(metrics.name, "Times-Roman");
    }

    #[test]
    fn test_courier() {
        let metrics = FontMetrics::courier();

        // Courier is monospace - all characters should have width 600
        assert_eq!(metrics.char_width('A'), 600);
        assert_eq!(metrics.char_width('a'), 600);
        assert_eq!(metrics.char_width('i'), 600);
        assert_eq!(metrics.char_width('W'), 600);
        assert_eq!(metrics.char_width(' '), 600);
        assert_eq!(metrics.name, "Courier");
        assert_eq!(metrics.default_width, 600);
    }

    #[test]
    fn test_symbol() {
        let metrics = FontMetrics::symbol();

        // Test Greek letters
        assert_eq!(metrics.char_width('α'), 631); // alpha
        assert_eq!(metrics.char_width('Ω'), 768); // Omega

        // Test math symbols
        assert_eq!(metrics.char_width('∑'), 713); // summation
        assert_eq!(metrics.char_width('∫'), 274); // integral
        assert_eq!(metrics.char_width('∞'), 713); // infinity

        assert_eq!(metrics.name, "Symbol");
    }

    #[test]
    fn test_zapf_dingbats() {
        let metrics = FontMetrics::zapf_dingbats();

        // Test dingbat characters
        assert_eq!(metrics.char_width('✓'), 911); // check mark
        assert_eq!(metrics.char_width('★'), 790); // black star
        assert_eq!(metrics.char_width('✉'), 791); // envelope
        assert_eq!(metrics.char_width('①'), 974); // circled digit one

        assert_eq!(metrics.name, "ZapfDingbats");
    }

    #[test]
    fn test_font_registry_all_14_fonts() {
        let registry = FontRegistry::new();

        // Helvetica family
        assert!(registry.get("Helvetica").is_some());
        assert!(registry.get("Helvetica-Bold").is_some());
        assert!(registry.get("Helvetica-Oblique").is_some());
        assert!(registry.get("Helvetica-BoldOblique").is_some());

        // Times family
        assert!(registry.get("Times-Roman").is_some());
        assert!(registry.get("Times-Bold").is_some());
        assert!(registry.get("Times-Italic").is_some());
        assert!(registry.get("Times-BoldItalic").is_some());

        // Courier family
        assert!(registry.get("Courier").is_some());
        assert!(registry.get("Courier-Bold").is_some());
        assert!(registry.get("Courier-Oblique").is_some());
        assert!(registry.get("Courier-BoldOblique").is_some());

        // Symbol
        assert!(registry.get("Symbol").is_some());

        // ZapfDingbats
        assert!(registry.get("ZapfDingbats").is_some());
    }

    #[test]
    fn test_courier_measure_text() {
        let metrics = FontMetrics::courier();
        let font_size = Length::from_pt(12.0);

        // "Code" = 4 chars * 600 = 2400 units
        // At 12pt: 2400 / 1000 * 12 = 28.8pt
        let width = metrics.measure_text("Code", font_size);

        assert!((width.to_pt() - 28.8).abs() < 0.01);
    }

    #[test]
    fn test_font_metrics_display() {
        let helvetica = FontMetrics::helvetica();
        let display = format!("{}", helvetica);
        assert!(display.contains("Helvetica"));
        assert!(display.contains("ascender: 718"));
        assert!(display.contains("descender: -207"));
        assert!(display.contains("cap-height: 718"));
        assert!(display.contains("x-height: 523"));

        let times = FontMetrics::times_roman();
        let display = format!("{}", times);
        assert!(display.contains("Times-Roman"));
        assert!(display.contains("ascender: 683"));

        let courier = FontMetrics::courier();
        let display = format!("{}", courier);
        assert!(display.contains("Courier"));
        assert!(display.contains("descender: -157"));
    }
}
