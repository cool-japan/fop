//! PDF graphics operations
//!
//! Handles drawing operations like borders, rectangles, and lines in PDF.

use fop_layout::area::BorderStyle;
use fop_types::{Color, Length, Result};
use std::fmt::Write as FmtWrite;

/// PDF graphics context for drawing operations
pub struct PdfGraphics {
    /// Content stream operations
    operations: String,
}

impl PdfGraphics {
    /// Create a new graphics context
    pub fn new() -> Self {
        Self {
            operations: String::new(),
        }
    }

    /// Get the content stream
    pub fn content(&self) -> &str {
        &self.operations
    }

    /// Set stroke color (for lines and borders)
    pub fn set_stroke_color(&mut self, color: Color) -> Result<()> {
        writeln!(
            &mut self.operations,
            "{:.3} {:.3} {:.3} RG",
            color.r_f32(),
            color.g_f32(),
            color.b_f32()
        )
        .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;
        Ok(())
    }

    /// Set fill color (for rectangles)
    pub fn set_fill_color(&mut self, color: Color) -> Result<()> {
        writeln!(
            &mut self.operations,
            "{:.3} {:.3} {:.3} rg",
            color.r_f32(),
            color.g_f32(),
            color.b_f32()
        )
        .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;
        Ok(())
    }

    /// Set fill opacity (for fill operations)
    ///
    /// Sets the alpha value for fill operations using the /ca operator
    /// in the graphics state dictionary. This requires ExtGState support.
    ///
    /// # Arguments
    /// * `opacity` - Opacity value from 0.0 (transparent) to 1.0 (opaque)
    /// * `gs_name` - Name of the graphics state resource (e.g., "gs1")
    ///
    /// # PDF Reference
    /// See PDF specification section 8.4 for graphics state parameters.
    pub fn set_opacity(&mut self, gs_name: &str) -> Result<()> {
        writeln!(&mut self.operations, "/{} gs", gs_name)
            .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;
        Ok(())
    }

    /// Set stroke opacity (for stroke operations)
    ///
    /// Sets the alpha value for stroke operations using the /CA operator
    /// in the graphics state dictionary. This requires ExtGState support.
    ///
    /// # Arguments
    /// * `gs_name` - Name of the graphics state resource (e.g., "gs1")
    ///
    /// # PDF Reference
    /// See PDF specification section 8.4 for graphics state parameters.
    pub fn set_stroke_opacity(&mut self, gs_name: &str) -> Result<()> {
        writeln!(&mut self.operations, "/{} gs", gs_name)
            .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;
        Ok(())
    }

    /// Set line width
    pub fn set_line_width(&mut self, width: Length) -> Result<()> {
        writeln!(&mut self.operations, "{:.3} w", width.to_pt())
            .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;
        Ok(())
    }

    /// Set dash pattern for line drawing
    ///
    /// # Arguments
    /// * `dash_array` - Array of on/off lengths (e.g., &[3.0, 2.0] for dashed)
    /// * `phase` - Offset into the dash pattern (usually 0)
    ///
    /// # Examples
    /// - Solid: `set_dash_pattern(&[], 0)` or `[]0 d`
    /// - Dashed: `set_dash_pattern(&[6.0, 3.0], 0)` or `[6 3] 0 d`
    /// - Dotted: `set_dash_pattern(&[1.0, 2.0], 0)` or `[1 2] 0 d`
    pub fn set_dash_pattern(&mut self, dash_array: &[f64], phase: f64) -> Result<()> {
        write!(&mut self.operations, "[")
            .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;
        for (i, &dash) in dash_array.iter().enumerate() {
            if i > 0 {
                write!(&mut self.operations, " ")
                    .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;
            }
            write!(&mut self.operations, "{:.3}", dash)
                .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;
        }
        writeln!(&mut self.operations, "] {:.3} d", phase)
            .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;
        Ok(())
    }

    /// Draw a rectangle (stroke only)
    pub fn draw_rectangle(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
    ) -> Result<()> {
        writeln!(
            &mut self.operations,
            "{:.3} {:.3} {:.3} {:.3} re S",
            x.to_pt(),
            y.to_pt(),
            width.to_pt(),
            height.to_pt()
        )
        .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;
        Ok(())
    }

    /// Fill a rectangle
    pub fn fill_rectangle(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
    ) -> Result<()> {
        self.fill_rectangle_with_radius(x, y, width, height, None)
    }

    /// Fill a rectangle with optional rounded corners
    pub fn fill_rectangle_with_radius(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
        border_radius: Option<[Length; 4]>,
    ) -> Result<()> {
        if let Some(radii) = border_radius {
            // Use rounded rectangle path for filling
            self.draw_rounded_rectangle(x, y, width, height, radii, true)
        } else {
            // Use simple rectangle for filling
            writeln!(
                &mut self.operations,
                "{:.3} {:.3} {:.3} {:.3} re f",
                x.to_pt(),
                y.to_pt(),
                width.to_pt(),
                height.to_pt()
            )
            .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;
            Ok(())
        }
    }

    /// Draw a rounded rectangle with independent corner radii
    ///
    /// Uses Bezier curves to approximate circular arcs for rounded corners.
    /// The magic number 0.552284749831 is used to approximate a circle with cubic Bezier curves.
    /// This is derived from 4*(sqrt(2)-1)/3, which minimizes the error between the arc and the curve.
    ///
    /// # Arguments
    /// * `x, y` - Bottom-left corner of the rectangle
    /// * `width, height` - Dimensions of the rectangle
    /// * `radii` - Corner radii [top-left, top-right, bottom-right, bottom-left]
    /// * `fill` - If true, fill the rectangle; if false, stroke it
    #[allow(clippy::too_many_arguments)]
    pub fn draw_rounded_rectangle(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
        radii: [Length; 4],
        fill: bool,
    ) -> Result<()> {
        let x_pt = x.to_pt();
        let y_pt = y.to_pt();
        let w_pt = width.to_pt();
        let h_pt = height.to_pt();

        // Extract corner radii: [top-left, top-right, bottom-right, bottom-left]
        let [tl, tr, br, bl] = radii;
        let tl_pt = tl.to_pt().min(w_pt / 2.0).min(h_pt / 2.0);
        let tr_pt = tr.to_pt().min(w_pt / 2.0).min(h_pt / 2.0);
        let br_pt = br.to_pt().min(w_pt / 2.0).min(h_pt / 2.0);
        let bl_pt = bl.to_pt().min(w_pt / 2.0).min(h_pt / 2.0);

        // Magic number for Bezier curve approximation of circular arcs
        // This is 4*(sqrt(2)-1)/3 ≈ 0.552284749831
        const KAPPA: f64 = 0.552284749831;

        // Start path from bottom-left corner, moving clockwise
        // Bottom edge, starting after bottom-left corner
        write!(&mut self.operations, "{:.3} {:.3} m ", x_pt + bl_pt, y_pt)
            .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;

        // Bottom edge to bottom-right corner
        write!(
            &mut self.operations,
            "{:.3} {:.3} l ",
            x_pt + w_pt - br_pt,
            y_pt
        )
        .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;

        // Bottom-right corner (if radius > 0)
        if br_pt > 0.0 {
            write!(
                &mut self.operations,
                "{:.3} {:.3} {:.3} {:.3} {:.3} {:.3} c ",
                x_pt + w_pt - br_pt + br_pt * KAPPA,
                y_pt,
                x_pt + w_pt,
                y_pt + br_pt - br_pt * KAPPA,
                x_pt + w_pt,
                y_pt + br_pt
            )
            .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;
        }

        // Right edge to top-right corner
        write!(
            &mut self.operations,
            "{:.3} {:.3} l ",
            x_pt + w_pt,
            y_pt + h_pt - tr_pt
        )
        .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;

        // Top-right corner (if radius > 0)
        if tr_pt > 0.0 {
            write!(
                &mut self.operations,
                "{:.3} {:.3} {:.3} {:.3} {:.3} {:.3} c ",
                x_pt + w_pt,
                y_pt + h_pt - tr_pt + tr_pt * KAPPA,
                x_pt + w_pt - tr_pt + tr_pt * KAPPA,
                y_pt + h_pt,
                x_pt + w_pt - tr_pt,
                y_pt + h_pt
            )
            .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;
        }

        // Top edge to top-left corner
        write!(
            &mut self.operations,
            "{:.3} {:.3} l ",
            x_pt + tl_pt,
            y_pt + h_pt
        )
        .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;

        // Top-left corner (if radius > 0)
        if tl_pt > 0.0 {
            write!(
                &mut self.operations,
                "{:.3} {:.3} {:.3} {:.3} {:.3} {:.3} c ",
                x_pt + tl_pt - tl_pt * KAPPA,
                y_pt + h_pt,
                x_pt,
                y_pt + h_pt - tl_pt + tl_pt * KAPPA,
                x_pt,
                y_pt + h_pt - tl_pt
            )
            .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;
        }

        // Left edge back to bottom-left corner
        write!(&mut self.operations, "{:.3} {:.3} l ", x_pt, y_pt + bl_pt)
            .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;

        // Bottom-left corner (if radius > 0)
        if bl_pt > 0.0 {
            write!(
                &mut self.operations,
                "{:.3} {:.3} {:.3} {:.3} {:.3} {:.3} c ",
                x_pt,
                y_pt + bl_pt - bl_pt * KAPPA,
                x_pt + bl_pt - bl_pt * KAPPA,
                y_pt,
                x_pt + bl_pt,
                y_pt
            )
            .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;
        }

        // Close path and fill or stroke
        if fill {
            writeln!(&mut self.operations, "f")
                .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;
        } else {
            writeln!(&mut self.operations, "S")
                .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;
        }

        Ok(())
    }

    /// Draw a line
    pub fn draw_line(&mut self, x1: Length, y1: Length, x2: Length, y2: Length) -> Result<()> {
        writeln!(
            &mut self.operations,
            "{:.3} {:.3} m {:.3} {:.3} l S",
            x1.to_pt(),
            y1.to_pt(),
            x2.to_pt(),
            y2.to_pt()
        )
        .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;
        Ok(())
    }

    /// Draw borders (all four sides)
    #[allow(clippy::too_many_arguments)]
    pub fn draw_borders(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
        border_widths: [Length; 4], // top, right, bottom, left
        border_colors: [Color; 4],
        border_styles: [BorderStyle; 4],
    ) -> Result<()> {
        self.draw_borders_with_radius(
            x,
            y,
            width,
            height,
            border_widths,
            border_colors,
            border_styles,
            None,
        )
    }

    /// Draw borders with optional rounded corners
    #[allow(clippy::too_many_arguments)]
    pub fn draw_borders_with_radius(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
        border_widths: [Length; 4], // top, right, bottom, left
        border_colors: [Color; 4],
        border_styles: [BorderStyle; 4],
        border_radius: Option<[Length; 4]>, // top-left, top-right, bottom-right, bottom-left
    ) -> Result<()> {
        let [top_width, right_width, bottom_width, left_width] = border_widths;
        let [top_color, right_color, bottom_color, left_color] = border_colors;
        let [top_style, right_style, bottom_style, left_style] = border_styles;

        // If we have rounded corners and uniform border properties, use draw_rounded_rectangle
        if let Some(radii) = border_radius {
            // Check if all borders have the same color, width, and style
            let uniform_color =
                top_color == right_color && top_color == bottom_color && top_color == left_color;
            let uniform_width =
                top_width == right_width && top_width == bottom_width && top_width == left_width;
            let uniform_style =
                top_style == right_style && top_style == bottom_style && top_style == left_style;

            if uniform_color
                && uniform_width
                && uniform_style
                && top_width > Length::ZERO
                && !matches!(top_style, BorderStyle::None | BorderStyle::Hidden)
            {
                // Use the optimized rounded rectangle path
                self.set_stroke_color(top_color)?;
                self.set_line_width(top_width)?;
                self.apply_border_style(top_style)?;
                self.draw_rounded_rectangle(x, y, width, height, radii, false)?;
                // Reset to solid
                self.set_dash_pattern(&[], 0.0)?;
                return Ok(());
            }
        }

        // Fall back to drawing individual border sides
        // For now, we draw straight lines even with border-radius
        // A full implementation would draw arcs for each corner
        // Top border
        if top_width > Length::ZERO && !matches!(top_style, BorderStyle::None | BorderStyle::Hidden)
        {
            self.set_stroke_color(top_color)?;
            self.set_line_width(top_width)?;
            self.apply_border_style(top_style)?;
            let y_top = y + height;
            self.draw_line(x, y_top, x + width, y_top)?;
            // Reset to solid for next border
            self.set_dash_pattern(&[], 0.0)?;
        }

        // Right border
        if right_width > Length::ZERO
            && !matches!(right_style, BorderStyle::None | BorderStyle::Hidden)
        {
            self.set_stroke_color(right_color)?;
            self.set_line_width(right_width)?;
            self.apply_border_style(right_style)?;
            let x_right = x + width;
            self.draw_line(x_right, y, x_right, y + height)?;
            // Reset to solid for next border
            self.set_dash_pattern(&[], 0.0)?;
        }

        // Bottom border
        if bottom_width > Length::ZERO
            && !matches!(bottom_style, BorderStyle::None | BorderStyle::Hidden)
        {
            self.set_stroke_color(bottom_color)?;
            self.set_line_width(bottom_width)?;
            self.apply_border_style(bottom_style)?;
            self.draw_line(x, y, x + width, y)?;
            // Reset to solid for next border
            self.set_dash_pattern(&[], 0.0)?;
        }

        // Left border
        if left_width > Length::ZERO
            && !matches!(left_style, BorderStyle::None | BorderStyle::Hidden)
        {
            self.set_stroke_color(left_color)?;
            self.set_line_width(left_width)?;
            self.apply_border_style(left_style)?;
            self.draw_line(x, y, x, y + height)?;
            // Reset to solid for next border
            self.set_dash_pattern(&[], 0.0)?;
        }

        Ok(())
    }

    /// Apply border style by setting appropriate dash pattern
    fn apply_border_style(&mut self, style: BorderStyle) -> Result<()> {
        match style {
            BorderStyle::Solid => self.set_dash_pattern(&[], 0.0),
            BorderStyle::Dashed => self.set_dash_pattern(&[6.0, 3.0], 0.0),
            BorderStyle::Dotted => self.set_dash_pattern(&[1.0, 2.0], 0.0),
            // For now, treat other styles as solid (double, groove, ridge, inset, outset)
            // These would require more complex rendering
            _ => self.set_dash_pattern(&[], 0.0),
        }
    }

    /// Save graphics state
    pub fn save_state(&mut self) -> Result<()> {
        writeln!(&mut self.operations, "q")
            .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;
        Ok(())
    }

    /// Restore graphics state
    pub fn restore_state(&mut self) -> Result<()> {
        writeln!(&mut self.operations, "Q")
            .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;
        Ok(())
    }

    /// Save graphics state and set clipping path
    ///
    /// This method saves the current graphics state and establishes a rectangular
    /// clipping path. Content drawn after this call will be clipped to the specified
    /// rectangle until restore_clip_state() is called.
    ///
    /// PDF operators used:
    /// - q: Save graphics state
    /// - re: Rectangle path
    /// - W: Set clipping path (intersect with current path)
    /// - n: End path without stroking or filling
    ///
    /// # Arguments
    /// * `x, y` - Bottom-left corner of clipping rectangle (PDF coordinates)
    /// * `width, height` - Dimensions of clipping rectangle
    ///
    /// # PDF Reference
    /// See PDF specification section 8.5 for clipping path details.
    pub fn save_clip_state(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
    ) -> Result<()> {
        // Save graphics state
        writeln!(&mut self.operations, "q")
            .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;

        // Define rectangle path and set as clipping path
        writeln!(
            &mut self.operations,
            "{:.3} {:.3} {:.3} {:.3} re W n",
            x.to_pt(),
            y.to_pt(),
            width.to_pt(),
            height.to_pt()
        )
        .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;

        Ok(())
    }

    /// Restore graphics state after clipping
    ///
    /// This restores the graphics state that was saved by save_clip_state(),
    /// removing the clipping path.
    ///
    /// PDF operator used:
    /// - Q: Restore graphics state
    pub fn restore_clip_state(&mut self) -> Result<()> {
        self.restore_state()
    }

    /// Fill a rectangle with a gradient
    ///
    /// Uses PDF shading pattern to fill the specified rectangle with a gradient.
    ///
    /// # Arguments
    /// * `x, y` - Bottom-left corner of the rectangle
    /// * `width, height` - Dimensions of the rectangle
    /// * `gradient_index` - Index of the gradient shading resource (Sh0, Sh1, etc.)
    ///
    /// # PDF Reference
    /// See PDF specification section 8.7 for shading patterns.
    /// Uses the /sh operator to paint with shading pattern.
    pub fn fill_gradient(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
        gradient_index: usize,
    ) -> Result<()> {
        // Save graphics state
        writeln!(&mut self.operations, "q")
            .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;

        // Set up transformation matrix to map gradient to rectangle
        // We need to translate and scale the coordinate system
        writeln!(
            &mut self.operations,
            "{:.3} 0 0 {:.3} {:.3} {:.3} cm",
            width.to_pt() / 100.0,  // Scale from 0-100 to width
            height.to_pt() / 100.0, // Scale from 0-100 to height
            x.to_pt(),
            y.to_pt()
        )
        .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;

        // Paint with shading pattern
        writeln!(&mut self.operations, "/Sh{} sh", gradient_index)
            .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;

        // Restore graphics state
        writeln!(&mut self.operations, "Q")
            .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;

        Ok(())
    }

    /// Fill a rectangle with a gradient and optional rounded corners
    pub fn fill_gradient_with_radius(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
        gradient_index: usize,
        border_radius: Option<[Length; 4]>,
    ) -> Result<()> {
        if border_radius.is_some() {
            // For rounded corners, we need to clip with a rounded rectangle path first
            // Save state
            self.save_state()?;

            // Create rounded rectangle path for clipping
            if let Some(radii) = border_radius {
                self.draw_rounded_rectangle(x, y, width, height, radii, false)?;
                // Use as clipping path (W n)
                writeln!(&mut self.operations, "W n")
                    .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;
            }

            // Now paint the gradient
            self.fill_gradient(x, y, width, height, gradient_index)?;

            // Restore state (removes clipping)
            self.restore_state()?;
        } else {
            // No rounding, just fill
            self.fill_gradient(x, y, width, height, gradient_index)?;
        }

        Ok(())
    }
}

impl Default for PdfGraphics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphics_creation() {
        let graphics = PdfGraphics::new();
        assert_eq!(graphics.content(), "");
    }

    #[test]
    fn test_set_stroke_color() {
        let mut graphics = PdfGraphics::new();
        graphics
            .set_stroke_color(Color::RED)
            .expect("test: should succeed");

        assert!(graphics.content().contains("1.000 0.000 0.000 RG"));
    }

    #[test]
    fn test_set_fill_color() {
        let mut graphics = PdfGraphics::new();
        graphics
            .set_fill_color(Color::BLUE)
            .expect("test: should succeed");

        assert!(graphics.content().contains("0.000 0.000 1.000 rg"));
    }

    #[test]
    fn test_set_line_width() {
        let mut graphics = PdfGraphics::new();
        graphics
            .set_line_width(Length::from_pt(2.0))
            .expect("test: should succeed");

        assert!(graphics.content().contains("2.000 w"));
    }

    #[test]
    fn test_draw_rectangle() {
        let mut graphics = PdfGraphics::new();
        graphics
            .draw_rectangle(
                Length::from_pt(10.0),
                Length::from_pt(20.0),
                Length::from_pt(100.0),
                Length::from_pt(50.0),
            )
            .expect("test: should succeed");

        assert!(graphics
            .content()
            .contains("10.000 20.000 100.000 50.000 re S"));
    }

    #[test]
    fn test_fill_rectangle() {
        let mut graphics = PdfGraphics::new();
        graphics
            .fill_rectangle(
                Length::from_pt(0.0),
                Length::from_pt(0.0),
                Length::from_pt(50.0),
                Length::from_pt(50.0),
            )
            .expect("test: should succeed");

        assert!(graphics.content().contains("re f"));
    }

    #[test]
    fn test_draw_line() {
        let mut graphics = PdfGraphics::new();
        graphics
            .draw_line(
                Length::from_pt(0.0),
                Length::from_pt(0.0),
                Length::from_pt(100.0),
                Length::from_pt(100.0),
            )
            .expect("test: should succeed");

        assert!(graphics.content().contains("m"));
        assert!(graphics.content().contains("l S"));
    }

    #[test]
    fn test_draw_borders() {
        let mut graphics = PdfGraphics::new();
        graphics
            .draw_borders(
                Length::from_pt(10.0),
                Length::from_pt(10.0),
                Length::from_pt(100.0),
                Length::from_pt(50.0),
                [Length::from_pt(1.0); 4],
                [Color::BLACK; 4],
                [BorderStyle::Solid; 4],
            )
            .expect("test: should succeed");

        // Should contain line drawing operations
        assert!(graphics.content().contains("m"));
        assert!(graphics.content().contains("l S"));
    }

    #[test]
    fn test_save_restore_state() {
        let mut graphics = PdfGraphics::new();
        graphics.save_state().expect("test: should succeed");
        graphics.restore_state().expect("test: should succeed");

        assert!(graphics.content().contains("q"));
        assert!(graphics.content().contains("Q"));
    }

    #[test]
    fn test_set_dash_pattern() {
        let mut graphics = PdfGraphics::new();

        // Solid (no dash)
        graphics
            .set_dash_pattern(&[], 0.0)
            .expect("test: should succeed");
        assert!(graphics.content().contains("[] 0.000 d"));

        // Dashed
        let mut graphics2 = PdfGraphics::new();
        graphics2
            .set_dash_pattern(&[6.0, 3.0], 0.0)
            .expect("test: should succeed");
        assert!(graphics2.content().contains("[6.000 3.000] 0.000 d"));

        // Dotted
        let mut graphics3 = PdfGraphics::new();
        graphics3
            .set_dash_pattern(&[1.0, 2.0], 0.0)
            .expect("test: should succeed");
        assert!(graphics3.content().contains("[1.000 2.000] 0.000 d"));
    }

    #[test]
    fn test_border_styles() {
        let mut graphics = PdfGraphics::new();

        // Test dashed border
        graphics
            .draw_borders(
                Length::from_pt(10.0),
                Length::from_pt(10.0),
                Length::from_pt(100.0),
                Length::from_pt(50.0),
                [Length::from_pt(2.0); 4],
                [Color::RED; 4],
                [BorderStyle::Dashed; 4],
            )
            .expect("test: should succeed");

        assert!(graphics.content().contains("[6.000 3.000] 0.000 d"));
        assert!(graphics.content().contains("1.000 0.000 0.000 RG")); // Red color
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    // ── Color tests ──────────────────────────────────────────────────────────

    #[test]
    fn test_set_stroke_color_green() {
        let mut g = PdfGraphics::new();
        g.set_stroke_color(Color::GREEN)
            .expect("test: should succeed");
        assert!(g.content().contains("0.000 1.000 0.000 RG"));
    }

    #[test]
    fn test_set_stroke_color_black() {
        let mut g = PdfGraphics::new();
        g.set_stroke_color(Color::BLACK)
            .expect("test: should succeed");
        assert!(g.content().contains("0.000 0.000 0.000 RG"));
    }

    #[test]
    fn test_set_stroke_color_white() {
        let mut g = PdfGraphics::new();
        g.set_stroke_color(Color::WHITE)
            .expect("test: should succeed");
        assert!(g.content().contains("1.000 1.000 1.000 RG"));
    }

    #[test]
    fn test_set_fill_color_red() {
        let mut g = PdfGraphics::new();
        g.set_fill_color(Color::RED).expect("test: should succeed");
        assert!(g.content().contains("1.000 0.000 0.000 rg"));
    }

    #[test]
    fn test_set_fill_color_green() {
        let mut g = PdfGraphics::new();
        g.set_fill_color(Color::GREEN)
            .expect("test: should succeed");
        assert!(g.content().contains("0.000 1.000 0.000 rg"));
    }

    #[test]
    fn test_set_fill_color_custom_rgb() {
        let mut g = PdfGraphics::new();
        // rgb(128, 64, 32) → f32 ≈ 0.502, 0.251, 0.125
        g.set_fill_color(Color::rgb(128, 64, 32))
            .expect("test: should succeed");
        let content = g.content().to_string();
        assert!(content.contains("rg"), "fill operator missing: {}", content);
        // Verify it does NOT contain the uppercase stroke operator
        assert!(!content.contains("RG"), "should use lowercase rg not RG");
    }

    #[test]
    fn test_stroke_uses_rg_uppercase_operator() {
        let mut g = PdfGraphics::new();
        g.set_stroke_color(Color::BLUE)
            .expect("test: should succeed");
        // PDF stroke color operator is RG (uppercase)
        assert!(g.content().contains("RG"));
        // Must not accidentally use fill operator
        assert!(!g.content().contains(" rg"));
    }

    #[test]
    fn test_fill_uses_rg_lowercase_operator() {
        let mut g = PdfGraphics::new();
        g.set_fill_color(Color::BLUE).expect("test: should succeed");
        // PDF fill color operator is rg (lowercase)
        assert!(g.content().contains(" rg") || g.content().ends_with("rg\n"));
        // Must not accidentally use stroke operator
        assert!(!g.content().contains("RG"));
    }

    // ── Line width tests ─────────────────────────────────────────────────────

    #[test]
    fn test_line_width_zero() {
        let mut g = PdfGraphics::new();
        g.set_line_width(Length::ZERO)
            .expect("test: should succeed");
        assert!(g.content().contains("0.000 w"));
    }

    #[test]
    fn test_line_width_fractional() {
        let mut g = PdfGraphics::new();
        g.set_line_width(Length::from_pt(0.5))
            .expect("test: should succeed");
        assert!(g.content().contains("0.500 w"));
    }

    #[test]
    fn test_line_width_large() {
        let mut g = PdfGraphics::new();
        g.set_line_width(Length::from_pt(10.0))
            .expect("test: should succeed");
        assert!(g.content().contains("10.000 w"));
    }

    // ── Path construction operator tests ─────────────────────────────────────

    #[test]
    fn test_draw_line_uses_m_operator() {
        let mut g = PdfGraphics::new();
        g.draw_line(
            Length::from_pt(5.0),
            Length::from_pt(10.0),
            Length::from_pt(50.0),
            Length::from_pt(100.0),
        )
        .expect("test: should succeed");
        let c = g.content();
        // moveto
        assert!(c.contains("5.000 10.000 m"), "expected 'm' operator: {}", c);
        // lineto
        assert!(
            c.contains("50.000 100.000 l"),
            "expected 'l' operator: {}",
            c
        );
        // stroke
        assert!(c.contains("S"), "expected 'S' operator: {}", c);
    }

    #[test]
    fn test_draw_rectangle_uses_re_operator() {
        let mut g = PdfGraphics::new();
        g.draw_rectangle(
            Length::from_pt(1.0),
            Length::from_pt(2.0),
            Length::from_pt(30.0),
            Length::from_pt(40.0),
        )
        .expect("test: should succeed");
        let c = g.content();
        // `re` appended by `S` (stroke)
        assert!(c.contains("re S"), "expected 're S': {}", c);
        assert!(c.contains("1.000 2.000 30.000 40.000"));
    }

    #[test]
    fn test_fill_rectangle_uses_re_f_operator() {
        let mut g = PdfGraphics::new();
        g.fill_rectangle(
            Length::from_pt(3.0),
            Length::from_pt(4.0),
            Length::from_pt(60.0),
            Length::from_pt(80.0),
        )
        .expect("test: should succeed");
        let c = g.content();
        // `re` appended by `f` (fill)
        assert!(c.contains("re f"), "expected 're f': {}", c);
        assert!(c.contains("3.000 4.000 60.000 80.000"));
    }

    // ── Path painting operator tests ─────────────────────────────────────────

    #[test]
    fn test_draw_line_stroke_operator_s() {
        let mut g = PdfGraphics::new();
        g.draw_line(
            Length::from_pt(0.0),
            Length::from_pt(0.0),
            Length::from_pt(100.0),
            Length::from_pt(0.0),
        )
        .expect("test: should succeed");
        // S = stroke path
        assert!(g.content().contains("l S"), "stroke 'S' missing");
    }

    #[test]
    fn test_fill_rectangle_f_operator() {
        let mut g = PdfGraphics::new();
        g.fill_rectangle(
            Length::ZERO,
            Length::ZERO,
            Length::from_pt(50.0),
            Length::from_pt(50.0),
        )
        .expect("test: should succeed");
        // f = fill path
        assert!(g.content().contains("re f"));
    }

    // ── Graphics state save/restore ──────────────────────────────────────────

    #[test]
    fn test_save_state_q_operator() {
        let mut g = PdfGraphics::new();
        g.save_state().expect("test: should succeed");
        assert!(g.content().contains("q\n"), "q operator missing");
    }

    #[test]
    fn test_restore_state_q_operator() {
        let mut g = PdfGraphics::new();
        g.restore_state().expect("test: should succeed");
        assert!(g.content().contains("Q\n"), "Q operator missing");
    }

    #[test]
    fn test_save_restore_nesting() {
        let mut g = PdfGraphics::new();
        g.save_state().expect("test: should succeed");
        g.save_state().expect("test: should succeed");
        g.restore_state().expect("test: should succeed");
        g.restore_state().expect("test: should succeed");
        let c = g.content();
        assert_eq!(c.matches("q\n").count(), 2);
        assert_eq!(c.matches("Q\n").count(), 2);
    }

    // ── Dash pattern tests ───────────────────────────────────────────────────

    #[test]
    fn test_dash_pattern_single_value() {
        let mut g = PdfGraphics::new();
        g.set_dash_pattern(&[3.0], 0.0)
            .expect("test: should succeed");
        assert!(g.content().contains("[3.000] 0.000 d"));
    }

    #[test]
    fn test_dash_pattern_with_phase() {
        let mut g = PdfGraphics::new();
        g.set_dash_pattern(&[4.0, 2.0], 1.0)
            .expect("test: should succeed");
        assert!(g.content().contains("[4.000 2.000] 1.000 d"));
    }

    #[test]
    fn test_dash_pattern_reset_to_solid() {
        let mut g = PdfGraphics::new();
        g.set_dash_pattern(&[6.0, 3.0], 0.0)
            .expect("test: should succeed");
        g.set_dash_pattern(&[], 0.0).expect("test: should succeed");
        let c = g.content();
        assert!(c.contains("[] 0.000 d"), "solid reset missing: {}", c);
    }

    // ── Clipping path tests ──────────────────────────────────────────────────

    #[test]
    fn test_save_clip_state_uses_w_n() {
        let mut g = PdfGraphics::new();
        g.save_clip_state(
            Length::from_pt(10.0),
            Length::from_pt(20.0),
            Length::from_pt(100.0),
            Length::from_pt(50.0),
        )
        .expect("test: should succeed");
        let c = g.content();
        // q saves state
        assert!(c.contains("q\n"), "q missing: {}", c);
        // W sets clipping path, n ends path
        assert!(c.contains("re W n"), "clipping 'W n' missing: {}", c);
        // Rectangle coordinates present
        assert!(c.contains("10.000 20.000 100.000 50.000"), "coords missing");
    }

    #[test]
    fn test_restore_clip_state_uses_q_operator() {
        let mut g = PdfGraphics::new();
        g.save_clip_state(
            Length::ZERO,
            Length::ZERO,
            Length::from_pt(200.0),
            Length::from_pt(100.0),
        )
        .expect("test: should succeed");
        g.restore_clip_state().expect("test: should succeed");
        let c = g.content();
        assert!(
            c.contains("Q\n"),
            "Q operator missing after restore_clip_state"
        );
    }

    // ── CTM / transformation matrix tests ───────────────────────────────────

    #[test]
    fn test_fill_gradient_cm_operator() {
        let mut g = PdfGraphics::new();
        g.fill_gradient(
            Length::from_pt(10.0),
            Length::from_pt(20.0),
            Length::from_pt(100.0),
            Length::from_pt(50.0),
            0,
        )
        .expect("test: should succeed");
        let c = g.content();
        // cm = current transformation matrix
        assert!(c.contains("cm"), "cm operator missing: {}", c);
        // Shading operator
        assert!(c.contains("/Sh0 sh"), "shading ref missing: {}", c);
        // State saved and restored around gradient
        assert!(c.contains("q\n"), "q missing");
        assert!(c.contains("Q\n"), "Q missing");
    }

    #[test]
    fn test_fill_gradient_index_increments() {
        let mut g = PdfGraphics::new();
        g.fill_gradient(
            Length::ZERO,
            Length::ZERO,
            Length::from_pt(50.0),
            Length::from_pt(50.0),
            1,
        )
        .expect("test: should succeed");
        assert!(g.content().contains("/Sh1 sh"));
    }

    // ── Opacity graphics state operator ─────────────────────────────────────

    #[test]
    fn test_set_opacity_gs_operator() {
        let mut g = PdfGraphics::new();
        g.set_opacity("gs1").expect("test: should succeed");
        assert!(g.content().contains("/gs1 gs"));
    }

    #[test]
    fn test_set_stroke_opacity_gs_operator() {
        let mut g = PdfGraphics::new();
        g.set_stroke_opacity("gs2").expect("test: should succeed");
        assert!(g.content().contains("/gs2 gs"));
    }

    // ── Border style tests ───────────────────────────────────────────────────

    #[test]
    fn test_draw_borders_dotted() {
        let mut g = PdfGraphics::new();
        g.draw_borders(
            Length::from_pt(5.0),
            Length::from_pt(5.0),
            Length::from_pt(80.0),
            Length::from_pt(40.0),
            [Length::from_pt(1.0); 4],
            [Color::BLACK; 4],
            [BorderStyle::Dotted; 4],
        )
        .expect("test: should succeed");
        // Dotted → [1.000 2.000] dash pattern
        assert!(
            g.content().contains("[1.000 2.000]"),
            "dotted dash pattern missing"
        );
    }

    #[test]
    fn test_draw_borders_none_produces_no_lines() {
        let mut g = PdfGraphics::new();
        g.draw_borders(
            Length::from_pt(5.0),
            Length::from_pt(5.0),
            Length::from_pt(80.0),
            Length::from_pt(40.0),
            [Length::from_pt(1.0); 4],
            [Color::BLACK; 4],
            [BorderStyle::None; 4],
        )
        .expect("test: should succeed");
        // No-draw borders should produce empty or no stroke operations
        let c = g.content();
        assert!(
            !c.contains("l S"),
            "none border style should not produce 'l S': {}",
            c
        );
    }

    #[test]
    fn test_draw_borders_zero_width_produces_no_lines() {
        let mut g = PdfGraphics::new();
        g.draw_borders(
            Length::from_pt(0.0),
            Length::from_pt(0.0),
            Length::from_pt(100.0),
            Length::from_pt(50.0),
            [Length::ZERO; 4],
            [Color::BLACK; 4],
            [BorderStyle::Solid; 4],
        )
        .expect("test: should succeed");
        // Zero-width borders should not produce stroke operations
        assert!(!g.content().contains("l S"), "zero-width should not stroke");
    }

    // ── Rounded rectangle tests ──────────────────────────────────────────────

    #[test]
    fn test_fill_rectangle_with_radius_uses_bezier() {
        let mut g = PdfGraphics::new();
        let radii = [Length::from_pt(5.0); 4];
        g.fill_rectangle_with_radius(
            Length::from_pt(10.0),
            Length::from_pt(10.0),
            Length::from_pt(100.0),
            Length::from_pt(50.0),
            Some(radii),
        )
        .expect("test: should succeed");
        let c = g.content();
        // Bezier curves use the `c` operator
        assert!(c.contains(" c "), "Bezier 'c' operator missing: {}", c);
        // Closed with fill
        assert!(c.contains("f\n"), "fill 'f' missing");
    }

    #[test]
    fn test_fill_rectangle_no_radius_uses_simple_re() {
        let mut g = PdfGraphics::new();
        g.fill_rectangle_with_radius(
            Length::from_pt(10.0),
            Length::from_pt(10.0),
            Length::from_pt(100.0),
            Length::from_pt(50.0),
            None,
        )
        .expect("test: should succeed");
        let c = g.content();
        // Simple rectangle uses `re f`
        assert!(c.contains("re f"), "expected 're f': {}", c);
    }

    // ── Default trait test ───────────────────────────────────────────────────

    #[test]
    fn test_default_creates_empty_graphics() {
        let g = PdfGraphics::default();
        assert_eq!(g.content(), "");
    }
}
