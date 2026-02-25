//! PDF text state and text operators
//!
//! Manages the text state (font, size, matrix, spacing) and converts
//! text-showing operators into positioned character sequences.

use crate::graphics::Matrix;

/// PDF text state
#[derive(Debug, Clone)]
pub struct TextState {
    /// Current font resource name (e.g. "F1")
    pub font_name: String,
    /// Font size in text space
    pub font_size: f32,
    /// Character spacing (Tc)
    pub char_spacing: f32,
    /// Word spacing (Tw)
    pub word_spacing: f32,
    /// Horizontal scaling (Th, in %)
    pub horiz_scale: f32,
    /// Text leading (Tl)
    pub leading: f32,
    /// Text rise (Ts)
    pub text_rise: f32,
    /// Text rendering mode (0=fill, 1=stroke, 2=fill+stroke, 3=invisible)
    pub rendering_mode: u8,
    /// Text matrix (Tm)
    pub text_matrix: Matrix,
    /// Text line matrix (updated by Td/TD/T*)
    pub text_line_matrix: Matrix,
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            font_name: String::new(),
            font_size: 12.0,
            char_spacing: 0.0,
            word_spacing: 0.0,
            horiz_scale: 100.0,
            leading: 0.0,
            text_rise: 0.0,
            rendering_mode: 0,
            text_matrix: Matrix::identity(),
            text_line_matrix: Matrix::identity(),
        }
    }
}

impl TextState {
    /// Apply Td (move text position)
    pub fn td(&mut self, tx: f32, ty: f32) {
        let m = Matrix {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: tx,
            f: ty,
        };
        self.text_line_matrix = m.concat(&self.text_line_matrix);
        self.text_matrix = self.text_line_matrix;
    }

    /// Apply TD (move text position and set leading)
    pub fn capital_td(&mut self, tx: f32, ty: f32) {
        self.leading = -ty;
        self.td(tx, ty);
    }

    /// Apply T* (move to start of next line)
    pub fn t_star(&mut self) {
        self.td(0.0, -self.leading);
    }

    /// Apply Tm (set text matrix)
    pub fn tm(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        let m = Matrix { a, b, c, d, e, f };
        self.text_matrix = m;
        self.text_line_matrix = m;
    }

    /// Get the combined text rendering matrix for a character at the current position.
    /// Returns (matrix, advance_x) where advance is in user space.
    pub fn glyph_matrix(&self, ctm: &Matrix) -> Matrix {
        // Text rendering matrix = [font_size * Th/100, 0, 0, font_size, 0, text_rise] * Tm * CTM
        let scale = self.font_size;
        let th = self.horiz_scale / 100.0;
        let tr = self.text_rise;

        let font_m = Matrix {
            a: scale * th,
            b: 0.0,
            c: 0.0,
            d: scale,
            e: 0.0,
            f: tr,
        };
        font_m.concat(&self.text_matrix).concat(ctm)
    }

    /// Advance the text matrix by `width` (in unscaled text units, i.e. 1/1000 of font size)
    /// after rendering a character with the given character code.
    pub fn advance(&mut self, width_units: f32, is_space: bool) {
        // Advance = (w0 - Tc / font_size) * font_size * Th/100 + Tc + (if space: Tw)
        // Per spec: tx = (w - Tc/Tf) * Tf + Tc + Tw*(is_space)
        // Simplified: tx = width_units/1000 * font_size * (horiz_scale/100) + char_spacing + (word_spacing if space)
        let scale = self.horiz_scale / 100.0;
        let glyph_advance = (width_units / 1000.0) * self.font_size * scale;
        let extra = self.char_spacing + if is_space { self.word_spacing } else { 0.0 };
        let tx = glyph_advance + extra;

        // Advance text matrix by (tx, 0) in text space
        self.text_matrix.e += tx * self.text_matrix.a;
        self.text_matrix.f += tx * self.text_matrix.b;
    }
}

/// A positioned glyph for rendering
#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    /// User space position (after applying all matrices)
    pub x: f32,
    pub y: f32,
    /// Font size in user space
    pub font_size: f32,
    /// Unicode character (if known)
    pub character: Option<char>,
    /// CID (character/glyph identifier in the font)
    pub cid: u32,
    /// Font resource name
    pub font_name: String,
    /// Glyph advance width in user space
    pub advance: f32,
    /// Fill color
    pub color: crate::graphics::Color,
}

/// Decode a PDF string operand into a sequence of CIDs.
/// For Type0 (composite) fonts, each CID is 2 bytes big-endian.
/// For simple fonts, each CID is 1 byte.
pub fn decode_string_to_cids(bytes: &[u8], is_composite: bool) -> Vec<u32> {
    if is_composite {
        bytes
            .chunks(2)
            .map(|chunk| {
                if chunk.len() == 2 {
                    ((chunk[0] as u32) << 8) | chunk[1] as u32
                } else {
                    chunk[0] as u32
                }
            })
            .collect()
    } else {
        bytes.iter().map(|&b| b as u32).collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // decode_string_to_cids
    // -----------------------------------------------------------------------

    #[test]
    fn test_decode_simple_font_single_byte_cids() {
        let bytes = b"ABC";
        let cids = decode_string_to_cids(bytes, false);
        assert_eq!(cids, vec![65, 66, 67]);
    }

    #[test]
    fn test_decode_simple_font_empty_string() {
        let cids = decode_string_to_cids(b"", false);
        assert!(cids.is_empty());
    }

    #[test]
    fn test_decode_composite_font_two_byte_cids() {
        // Each pair of bytes forms a 16-bit CID big-endian
        let bytes: &[u8] = &[0x00, 0x41, 0x00, 0x42]; // CIDs 0x0041=65, 0x0042=66
        let cids = decode_string_to_cids(bytes, true);
        assert_eq!(cids, vec![0x0041, 0x0042]);
    }

    #[test]
    fn test_decode_composite_font_odd_byte_count() {
        // Odd byte count: last byte is treated as single-byte CID
        let bytes: &[u8] = &[0x00, 0x41, 0x42];
        let cids = decode_string_to_cids(bytes, true);
        assert_eq!(cids.len(), 2);
        assert_eq!(cids[0], 0x0041);
        assert_eq!(cids[1], 0x42);
    }

    #[test]
    fn test_decode_simple_font_high_byte_values() {
        let bytes: &[u8] = &[0xFF, 0x80, 0x00];
        let cids = decode_string_to_cids(bytes, false);
        assert_eq!(cids, vec![255, 128, 0]);
    }

    #[test]
    fn test_decode_composite_font_zero_cid() {
        let bytes: &[u8] = &[0x00, 0x00];
        let cids = decode_string_to_cids(bytes, true);
        assert_eq!(cids, vec![0]);
    }

    // -----------------------------------------------------------------------
    // TextState defaults
    // -----------------------------------------------------------------------

    #[test]
    fn test_text_state_default_font_size() {
        let ts = TextState::default();
        assert!((ts.font_size - 12.0).abs() < 1e-6);
    }

    #[test]
    fn test_text_state_default_horiz_scale() {
        let ts = TextState::default();
        assert!((ts.horiz_scale - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_text_state_default_spacing_zero() {
        let ts = TextState::default();
        assert_eq!(ts.char_spacing, 0.0);
        assert_eq!(ts.word_spacing, 0.0);
        assert_eq!(ts.leading, 0.0);
        assert_eq!(ts.text_rise, 0.0);
    }

    #[test]
    fn test_text_state_default_rendering_mode_fill() {
        let ts = TextState::default();
        assert_eq!(ts.rendering_mode, 0);
    }

    #[test]
    fn test_text_state_default_font_name_empty() {
        let ts = TextState::default();
        assert!(ts.font_name.is_empty());
    }

    // -----------------------------------------------------------------------
    // TextState::td (Td operator)
    // -----------------------------------------------------------------------

    #[test]
    fn test_td_moves_text_position() {
        let mut ts = TextState::default();
        ts.td(10.0, 20.0);
        // text_line_matrix and text_matrix should both update
        assert!((ts.text_matrix.e - 10.0).abs() < 1e-5);
        assert!((ts.text_matrix.f - 20.0).abs() < 1e-5);
    }

    #[test]
    fn test_td_zero_offset_no_change() {
        let mut ts = TextState::default();
        let e_before = ts.text_matrix.e;
        let f_before = ts.text_matrix.f;
        ts.td(0.0, 0.0);
        assert!((ts.text_matrix.e - e_before).abs() < 1e-6);
        assert!((ts.text_matrix.f - f_before).abs() < 1e-6);
    }

    #[test]
    fn test_td_negative_offset() {
        let mut ts = TextState::default();
        ts.td(-5.0, -3.0);
        assert!((ts.text_matrix.e - (-5.0)).abs() < 1e-5);
        assert!((ts.text_matrix.f - (-3.0)).abs() < 1e-5);
    }

    // -----------------------------------------------------------------------
    // TextState::capital_td (TD operator)
    // -----------------------------------------------------------------------

    #[test]
    fn test_capital_td_sets_leading() {
        let mut ts = TextState::default();
        ts.capital_td(0.0, -14.0);
        assert!((ts.leading - 14.0).abs() < 1e-5);
    }

    #[test]
    fn test_capital_td_moves_position_same_as_td() {
        let mut ts1 = TextState::default();
        let mut ts2 = TextState::default();
        ts1.capital_td(5.0, -12.0);
        ts2.td(5.0, -12.0);
        assert!((ts1.text_matrix.e - ts2.text_matrix.e).abs() < 1e-5);
        assert!((ts1.text_matrix.f - ts2.text_matrix.f).abs() < 1e-5);
    }

    // -----------------------------------------------------------------------
    // TextState::t_star (T* operator)
    // -----------------------------------------------------------------------

    #[test]
    fn test_t_star_moves_by_leading() {
        let mut ts = TextState {
            leading: 14.0,
            ..Default::default()
        };
        ts.t_star();
        assert!((ts.text_matrix.f - (-14.0)).abs() < 1e-5);
    }

    #[test]
    fn test_t_star_with_zero_leading_no_movement() {
        let mut ts = TextState {
            leading: 0.0,
            ..Default::default()
        };
        ts.t_star();
        assert!((ts.text_matrix.f - 0.0).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // TextState::tm (Tm operator)
    // -----------------------------------------------------------------------

    #[test]
    fn test_tm_sets_text_matrix() {
        let mut ts = TextState::default();
        ts.tm(2.0, 0.0, 0.0, 2.0, 100.0, 200.0);
        assert!((ts.text_matrix.a - 2.0).abs() < 1e-6);
        assert!((ts.text_matrix.d - 2.0).abs() < 1e-6);
        assert!((ts.text_matrix.e - 100.0).abs() < 1e-6);
        assert!((ts.text_matrix.f - 200.0).abs() < 1e-6);
    }

    #[test]
    fn test_tm_sets_line_matrix_equal_to_text_matrix() {
        let mut ts = TextState::default();
        ts.tm(1.0, 0.0, 0.0, 1.0, 50.0, 60.0);
        assert!((ts.text_line_matrix.e - ts.text_matrix.e).abs() < 1e-6);
        assert!((ts.text_line_matrix.f - ts.text_matrix.f).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // TextState::advance
    // -----------------------------------------------------------------------

    #[test]
    fn test_advance_moves_text_position() {
        let mut ts = TextState {
            font_size: 12.0,
            horiz_scale: 100.0,
            ..Default::default()
        };
        let e_before = ts.text_matrix.e;
        ts.advance(1000.0, false); // 1000 units = 1 em = font_size
                                   // Advance should be approximately font_size = 12.0
        assert!(ts.text_matrix.e > e_before);
    }

    #[test]
    fn test_advance_with_word_spacing() {
        let mut ts = TextState {
            font_size: 12.0,
            horiz_scale: 100.0,
            word_spacing: 5.0,
            ..Default::default()
        };
        let e_before = ts.text_matrix.e;
        ts.advance(0.0, true); // is_space: adds word_spacing
        assert!((ts.text_matrix.e - e_before - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_advance_no_spacing_non_space() {
        let mut ts = TextState {
            font_size: 10.0,
            horiz_scale: 100.0,
            char_spacing: 0.0,
            word_spacing: 5.0,
            ..Default::default()
        };
        let e_before = ts.text_matrix.e;
        ts.advance(500.0, false); // 500/1000 * 10 = 5.0, no word_spacing
        assert!((ts.text_matrix.e - e_before - 5.0).abs() < 1e-4);
    }
}
