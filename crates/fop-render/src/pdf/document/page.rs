//! PDF page operations
//!
//! Implements rendering operations on individual PDF pages.

use fop_types::Length;

use crate::pdf::font::FontManager;

use super::types::{LinkAnnotation, LinkDestination, PdfPage};

/// Build PDF character spacing (Tc) and word spacing (Tw) operator strings.
///
/// Returns a string with the appropriate operators for inclusion in a BT...ET block.
/// Non-zero spacing values produce "Tc" or "Tw" lines; zero values are omitted.
pub(super) fn build_spacing_ops(
    letter_spacing: Option<Length>,
    word_spacing: Option<Length>,
) -> String {
    let mut ops = String::new();
    if let Some(ls) = letter_spacing {
        let pt = ls.to_pt();
        if pt.abs() > 0.0001 {
            ops.push_str(&format!(
                "{:.4} Tc
",
                pt
            ));
        }
    }
    if let Some(ws) = word_spacing {
        let pt = ws.to_pt();
        if pt.abs() > 0.0001 {
            ops.push_str(&format!(
                "{:.4} Tw
",
                pt
            ));
        }
    }
    ops
}

impl PdfPage {
    /// Create a new PDF page
    pub fn new(width: Length, height: Length) -> Self {
        Self {
            width,
            height,
            content: Vec::new(),
            link_annotations: Vec::new(),
        }
    }

    /// Add a link annotation to the page
    ///
    /// # Arguments
    /// * `x` - X position (PDF coordinates: bottom-left origin)
    /// * `y` - Y position (PDF coordinates: bottom-left origin)
    /// * `width` - Width of the clickable area
    /// * `height` - Height of the clickable area
    /// * `destination` - Link destination (external URL or internal ID)
    pub fn add_link_annotation(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
        destination: LinkDestination,
    ) {
        let rect = [
            x.to_pt(),
            y.to_pt(),
            (x + width).to_pt(),
            (y + height).to_pt(),
        ];
        self.link_annotations
            .push(LinkAnnotation { rect, destination });
    }

    /// Encode text for PDF output using original glyph IDs as 2-byte CIDs.
    ///
    /// When a `char_to_orig_glyph` map is provided, each character is encoded as
    /// its original TrueType GID (a u16, always fits in 4 hex digits).  This is
    /// the correct CID per the module-level invariant in `fop_render::pdf::font`.
    ///
    /// When no map is provided (e.g. plain Helvetica text), the function falls
    /// back to writing the Unicode code point directly, which works correctly for
    /// BMP characters.  Callers that need astral-char support should always supply
    /// the map via `add_text_with_font_tracked` (or equivalent).
    ///
    /// The returned string is a PDF hex string `<XXXX...>`, with no BOM.
    fn encode_pdf_text_with_map(
        text: &str,
        char_to_orig_glyph: Option<&std::collections::HashMap<char, u16>>,
    ) -> String {
        let mut result = String::from("<");

        for c in text.chars() {
            let cid: u32 = match char_to_orig_glyph.and_then(|m| m.get(&c).copied()) {
                Some(orig_gid) => orig_gid as u32,
                None => {
                    // Fallback: Unicode code point (correct for BMP; astral chars
                    // are not representable here, but this path is only hit for
                    // non-tracked text which is limited to F1/Helvetica anyway).
                    c as u32
                }
            };
            // Always write exactly 4 hex digits (2-byte CID, big-endian).
            // BMP characters and GIDs both fit; astral code points in the
            // fallback path would exceed this, but that path is BMP-only.
            result.push_str(&format!("{:04X}", cid & 0xFFFF));
        }

        result.push('>');
        result
    }

    /// Add text to the page using the default Helvetica font (F1)
    pub fn add_text(&mut self, text: &str, x: Length, y: Length, font_size: Length) {
        self.add_text_with_spacing(text, x, y, font_size, None, None);
    }

    /// Add text to the page using the default Helvetica font (F1) with optional letter/word spacing
    ///
    /// # Arguments
    /// * `letter_spacing` - Optional character spacing in points (Tc operator)
    /// * `word_spacing` - Optional word spacing in points (Tw operator)
    pub fn add_text_with_spacing(
        &mut self,
        text: &str,
        x: Length,
        y: Length,
        font_size: Length,
        letter_spacing: Option<Length>,
        word_spacing: Option<Length>,
    ) {
        // Build optional Tc/Tw spacing operators
        let spacing_ops = build_spacing_ops(letter_spacing, word_spacing);
        let ops = format!(
            "BT\n/F1 {} Tf\n{}{} {} Td\n({}) Tj\nET\n",
            font_size.to_pt(),
            spacing_ops,
            x.to_pt(),
            y.to_pt(),
            text
        );
        self.content.extend_from_slice(ops.as_bytes());
    }

    /// Add text to the page using a custom embedded font
    ///
    /// # Arguments
    /// * `text` - The text to display
    /// * `x` - X position
    /// * `y` - Y position
    /// * `font_size` - Font size
    /// * `font_index` - Index of the embedded font (from `embed_font`)
    ///
    /// Note: Character usage tracking must be done separately via FontManager::record_text
    pub fn add_text_with_font(
        &mut self,
        text: &str,
        x: Length,
        y: Length,
        font_size: Length,
        font_index: usize,
    ) {
        self.add_text_with_font_and_spacing(text, x, y, font_size, font_index, None, None);
    }

    /// Add text to the page using a custom embedded font with optional letter/word spacing
    ///
    /// # Arguments
    /// * `text` - The text to display
    /// * `x` - X position
    /// * `y` - Y position
    /// * `font_size` - Font size
    /// * `font_index` - Index of the embedded font (from `embed_font`)
    /// * `letter_spacing` - Optional character spacing in points (Tc operator)
    /// * `word_spacing` - Optional word spacing in points (Tw operator)
    #[allow(clippy::too_many_arguments)]
    pub fn add_text_with_font_and_spacing(
        &mut self,
        text: &str,
        x: Length,
        y: Length,
        font_size: Length,
        font_index: usize,
        letter_spacing: Option<Length>,
        word_spacing: Option<Length>,
    ) {
        // Custom fonts are F2, F3, F4, etc. (F1 is reserved for Helvetica)
        let font_name = format!("F{}", font_index + 2);

        // Encode text using the Unicode code point as a fallback CID (BMP-safe).
        // For astral characters, callers must use add_text_with_font_tracked so
        // that the char_to_orig_glyph map is available for glyph-ID encoding.
        let encoded_text = Self::encode_pdf_text_with_map(text, None);

        // Build optional Tc/Tw spacing operators
        let spacing_ops = build_spacing_ops(letter_spacing, word_spacing);

        let ops = format!(
            "BT\n/{} {} Tf\n{}{} {} Td\n{} Tj\nET\n",
            font_name,
            font_size.to_pt(),
            spacing_ops,
            x.to_pt(),
            y.to_pt(),
            encoded_text
        );
        self.content.extend_from_slice(ops.as_bytes());
    }

    /// Add text to the page using a custom embedded font and track character usage.
    ///
    /// This is the primary method for Unicode text (including astral/non-BMP
    /// characters).  It both records character usage for subsetting and encodes
    /// the text using **original glyph IDs** as 2-byte CIDs — consistent with the
    /// module-level CID invariant in `fop_render::pdf::font`.
    ///
    /// The glyph IDs are obtained from the original (non-subsetted) embedded font
    /// by parsing its `font_data` via `ttf_parser`.  Astral characters such as
    /// U+1F600 (😀) or U+20000 (𠀀) map to their original u16 GIDs which always
    /// fit in the 2-byte CID space.
    ///
    /// # Arguments
    /// * `text` - The text to display (any Unicode, including non-BMP)
    /// * `x` - X position
    /// * `y` - Y position
    /// * `font_size` - Font size
    /// * `font_index` - Index of the embedded font (from `embed_font`)
    /// * `font_manager` - FontManager to record character usage and look up glyph IDs
    pub fn add_text_with_font_tracked(
        &mut self,
        text: &str,
        x: Length,
        y: Length,
        font_size: Length,
        font_index: usize,
        font_manager: &mut FontManager,
    ) {
        // Record character usage for subsetting.
        font_manager.record_text(font_index, text);

        // Encode text using original glyph IDs (CID = orig GID invariant).
        // This correctly handles all Unicode scalars including astral characters.
        let encoded_text = if let Some(font) = font_manager.get_font(font_index) {
            font.encode_text_with_glyph_ids(text)
        } else {
            // Font not found; fall back to Unicode code point encoding.
            Self::encode_pdf_text_with_map(text, None)
        };

        let font_name = format!("F{}", font_index + 2);
        let ops = format!(
            "BT\n/{} {} Tf\n{} {} Td\n{} Tj\nET\n",
            font_name,
            font_size.to_pt(),
            x.to_pt(),
            y.to_pt(),
            encoded_text
        );
        self.content.extend_from_slice(ops.as_bytes());
    }

    /// Add background color to an area
    pub fn add_background(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
        color: fop_types::Color,
    ) {
        self.add_background_with_radius(x, y, width, height, color, None);
    }

    /// Add background color to an area with optional rounded corners
    pub fn add_background_with_radius(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
        color: fop_types::Color,
        border_radius: Option<[Length; 4]>,
    ) {
        use crate::pdf::graphics::PdfGraphics;
        let mut graphics = PdfGraphics::new();
        let _ = graphics.set_fill_color(color);
        let _ = graphics.fill_rectangle_with_radius(x, y, width, height, border_radius);
        self.content
            .extend_from_slice(graphics.content().as_bytes());
    }

    /// Add background color with opacity to an area with optional rounded corners
    ///
    /// # Arguments
    /// * `x, y` - Bottom-left corner of the area (PDF coordinates)
    /// * `width, height` - Dimensions of the area
    /// * `color` - Fill color
    /// * `border_radius` - Optional corner radii
    /// * `gs_index` - Index of the ExtGState resource for opacity
    #[allow(clippy::too_many_arguments)]
    pub fn add_background_with_opacity(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
        color: fop_types::Color,
        border_radius: Option<[Length; 4]>,
        gs_index: usize,
    ) {
        use crate::pdf::graphics::PdfGraphics;
        let mut graphics = PdfGraphics::new();
        let _ = graphics.set_opacity(&format!("GS{}", gs_index));
        let _ = graphics.set_fill_color(color);
        let _ = graphics.fill_rectangle_with_radius(x, y, width, height, border_radius);
        self.content
            .extend_from_slice(graphics.content().as_bytes());
    }

    /// Add gradient background to an area
    ///
    /// # Arguments
    /// * `x, y` - Bottom-left corner of the area (PDF coordinates)
    /// * `width, height` - Dimensions of the area
    /// * `gradient_index` - Index of the gradient in the document's gradient list
    pub fn add_gradient_background(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
        gradient_index: usize,
    ) {
        self.add_gradient_background_with_radius(x, y, width, height, gradient_index, None);
    }

    /// Add gradient background to an area with optional rounded corners
    ///
    /// # Arguments
    /// * `x, y` - Bottom-left corner of the area (PDF coordinates)
    /// * `width, height` - Dimensions of the area
    /// * `gradient_index` - Index of the gradient in the document's gradient list
    /// * `border_radius` - Optional corner radii [top-left, top-right, bottom-right, bottom-left]
    pub fn add_gradient_background_with_radius(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
        gradient_index: usize,
        border_radius: Option<[Length; 4]>,
    ) {
        use crate::pdf::graphics::PdfGraphics;
        let mut graphics = PdfGraphics::new();
        let _ =
            graphics.fill_gradient_with_radius(x, y, width, height, gradient_index, border_radius);
        self.content
            .extend_from_slice(graphics.content().as_bytes());
    }

    /// Add borders to an area
    #[allow(clippy::too_many_arguments)]
    pub fn add_borders(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
        border_widths: [Length; 4],
        border_colors: [fop_types::Color; 4],
        border_styles: [fop_layout::area::BorderStyle; 4],
    ) {
        self.add_borders_with_radius(
            x,
            y,
            width,
            height,
            border_widths,
            border_colors,
            border_styles,
            None,
        );
    }

    /// Add borders to an area with optional rounded corners
    #[allow(clippy::too_many_arguments)]
    pub fn add_borders_with_radius(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
        border_widths: [Length; 4],
        border_colors: [fop_types::Color; 4],
        border_styles: [fop_layout::area::BorderStyle; 4],
        border_radius: Option<[Length; 4]>,
    ) {
        use crate::pdf::graphics::PdfGraphics;
        let mut graphics = PdfGraphics::new();
        let _ = graphics.draw_borders_with_radius(
            x,
            y,
            width,
            height,
            border_widths,
            border_colors,
            border_styles,
            border_radius,
        );
        self.content
            .extend_from_slice(graphics.content().as_bytes());
    }

    /// Add borders with opacity to an area with optional rounded corners
    #[allow(clippy::too_many_arguments)]
    pub fn add_borders_with_opacity(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
        border_widths: [Length; 4],
        border_colors: [fop_types::Color; 4],
        border_styles: [fop_layout::area::BorderStyle; 4],
        border_radius: Option<[Length; 4]>,
        gs_index: usize,
    ) {
        use crate::pdf::graphics::PdfGraphics;
        let mut graphics = PdfGraphics::new();
        let _ = graphics.set_stroke_opacity(&format!("GS{}", gs_index));
        let _ = graphics.draw_borders_with_radius(
            x,
            y,
            width,
            height,
            border_widths,
            border_colors,
            border_styles,
            border_radius,
        );
        self.content
            .extend_from_slice(graphics.content().as_bytes());
    }

    /// Add an image to the page
    ///
    /// # Arguments
    /// * `image_index` - The index of the image XObject in the document's image_xobjects list
    /// * `x` - X position in PDF coordinates (bottom-left origin)
    /// * `y` - Y position in PDF coordinates (bottom-left origin)
    /// * `width` - Display width
    /// * `height` - Display height
    pub fn add_image(
        &mut self,
        image_index: usize,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
    ) {
        use std::fmt::Write;
        let mut ops = String::new();

        // Save graphics state
        let _ = writeln!(&mut ops, "q");

        // Set up transformation matrix: translate, then scale
        // PDF images are 1x1 unit square by default, so we scale to width/height
        let _ = writeln!(
            &mut ops,
            "{:.3} 0 0 {:.3} {:.3} {:.3} cm",
            width.to_pt(),
            height.to_pt(),
            x.to_pt(),
            y.to_pt()
        );

        // Draw the image
        let _ = writeln!(&mut ops, "/Im{} Do", image_index);

        // Restore graphics state
        let _ = writeln!(&mut ops, "Q");

        self.content.extend_from_slice(ops.as_bytes());
    }

    /// Add a horizontal rule (line) to the page
    ///
    /// # Arguments
    /// * `x` - Left edge x-coordinate
    /// * `y` - Bottom edge y-coordinate (PDF coordinate system)
    /// * `width` - Rule width
    /// * `thickness` - Line thickness
    /// * `color` - Line color
    /// * `style` - Line style (solid, dashed, dotted)
    pub fn add_rule(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        thickness: Length,
        color: fop_types::Color,
        style: &str,
    ) {
        use std::fmt::Write;
        let mut ops = String::new();

        // Set stroke color
        let _ = writeln!(
            &mut ops,
            "{:.3} {:.3} {:.3} RG",
            color.r_f32(),
            color.g_f32(),
            color.b_f32()
        );

        // Set line width
        let _ = writeln!(&mut ops, "{:.3} w", thickness.to_pt());

        // Set dash pattern based on style
        match style {
            "dashed" => {
                let _ = writeln!(&mut ops, "[6 3] 0 d");
            }
            "dotted" => {
                let _ = writeln!(&mut ops, "[1 2] 0 d");
            }
            _ => {
                // solid or unknown - use solid line
                let _ = writeln!(&mut ops, "[] 0 d");
            }
        }

        // Draw the line (move to start, line to end, stroke)
        let _ = writeln!(
            &mut ops,
            "{:.3} {:.3} m {:.3} {:.3} l S",
            x.to_pt(),
            y.to_pt(),
            (x + width).to_pt(),
            y.to_pt()
        );

        self.content.extend_from_slice(ops.as_bytes());
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
    ) -> fop_types::Result<()> {
        use std::fmt::Write;
        let mut ops = String::new();

        // Save graphics state
        writeln!(&mut ops, "q").map_err(|e| fop_types::FopError::Generic(e.to_string()))?;

        // Define rectangle path and set as clipping path
        writeln!(
            &mut ops,
            "{:.3} {:.3} {:.3} {:.3} re W n",
            x.to_pt(),
            y.to_pt(),
            width.to_pt(),
            height.to_pt()
        )
        .map_err(|e| fop_types::FopError::Generic(e.to_string()))?;

        self.content.extend_from_slice(ops.as_bytes());
        Ok(())
    }

    /// Restore graphics state after clipping
    ///
    /// This restores the graphics state that was saved by save_clip_state(),
    /// removing the clipping path.
    ///
    /// PDF operator used:
    /// - Q: Restore graphics state
    pub fn restore_clip_state(&mut self) -> fop_types::Result<()> {
        use std::fmt::Write;
        let mut ops = String::new();

        writeln!(&mut ops, "Q").map_err(|e| fop_types::FopError::Generic(e.to_string()))?;

        self.content.extend_from_slice(ops.as_bytes());
        Ok(())
    }
}
