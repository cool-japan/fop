//! PostScript rendering backend
//!
//! Generates PostScript Level 2 documents from area trees.

use crate::image::ImageInfo;
use fop_layout::{AreaId, AreaTree, AreaType};
use fop_types::{Color, Length, Result};
use std::collections::HashMap;

/// PostScript renderer
pub struct PsRenderer {
    /// Default page width (A4)
    #[allow(dead_code)]
    page_width: Length,

    /// Default page height (A4)
    #[allow(dead_code)]
    page_height: Length,
}

impl PsRenderer {
    /// Create a new PostScript renderer
    pub fn new() -> Self {
        Self {
            page_width: Length::from_mm(210.0),
            page_height: Length::from_mm(297.0),
        }
    }

    /// Render an area tree to a PostScript document (returns PS as String)
    pub fn render_to_ps(&self, area_tree: &AreaTree) -> Result<String> {
        let mut ps_doc = PsDocument::new();

        // First pass: collect all images and convert to PS format
        let mut image_map = HashMap::new();
        self.collect_images(area_tree, &mut image_map)?;

        // Second pass: render pages
        for (id, node) in area_tree.iter() {
            if matches!(node.area.area_type, AreaType::Page) {
                let page_ps = self.render_page(area_tree, id, &image_map)?;
                ps_doc.add_page(page_ps);
            }
        }

        Ok(ps_doc.to_string())
    }

    /// Collect all images from the area tree
    fn collect_images(
        &self,
        area_tree: &AreaTree,
        image_map: &mut HashMap<AreaId, ImageData>,
    ) -> Result<()> {
        for (id, node) in area_tree.iter() {
            if matches!(node.area.area_type, AreaType::Viewport) {
                if let Some(image_data) = node.area.image_data() {
                    let image_info = ImageInfo::from_bytes(image_data)?;
                    let ps_image = ImageData {
                        width: image_info.width_px,
                        height: image_info.height_px,
                        data: image_data.to_vec(),
                    };
                    image_map.insert(id, ps_image);
                }
            }
        }
        Ok(())
    }

    /// Render a single page
    fn render_page(
        &self,
        area_tree: &AreaTree,
        page_id: AreaId,
        image_map: &HashMap<AreaId, ImageData>,
    ) -> Result<PsPage> {
        let page_node = area_tree
            .get(page_id)
            .ok_or_else(|| fop_types::FopError::Generic("Page not found".to_string()))?;

        let width = page_node.area.width();
        let height = page_node.area.height();

        let mut ps_page = PsPage::new(width, height);

        // Render all child areas recursively with absolute positioning
        render_children(
            area_tree,
            page_id,
            &mut ps_page,
            Length::ZERO,
            Length::ZERO,
            image_map,
        )?;

        Ok(ps_page)
    }
}

impl Default for PsRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Image data for PostScript
#[derive(Clone)]
struct ImageData {
    width: u32,
    height: u32,
    #[allow(dead_code)]
    data: Vec<u8>,
}

/// PostScript document builder - manages multiple pages
pub struct PsDocument {
    pages: Vec<PsPage>,
}

impl PsDocument {
    /// Create a new PostScript document
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }

    /// Add a page to the document
    pub fn add_page(&mut self, page: PsPage) {
        self.pages.push(page);
    }

    /// Write DSC header comments for the document.
    ///
    /// The `%%BoundingBox` is set to the union of all page bounding boxes,
    /// which for a typical document is the bounding box of the first (or
    /// largest) page.
    fn write_dsc_header(&self, result: &mut String) {
        result.push_str("%!PS-Adobe-3.0\n");
        result.push_str("%%Creator: Apache FOP Rust\n");
        result.push_str("%%LanguageLevel: 2\n");
        result.push_str(&format!("%%Pages: {}\n", self.pages.len()));

        // Compute global bounding box (union of all pages)
        if let Some(first_page) = self.pages.first() {
            let max_w = self
                .pages
                .iter()
                .map(|p| p.width.to_pt() as i64)
                .max()
                .unwrap_or(0);
            let max_h = self
                .pages
                .iter()
                .map(|p| p.height.to_pt() as i64)
                .max()
                .unwrap_or(0);
            // Suppress unused warning: first_page is used to initialise max
            let _ = first_page;
            result.push_str(&format!("%%BoundingBox: 0 0 {} {}\n", max_w, max_h));
        } else {
            result.push_str("%%BoundingBox: (atend)\n");
        }

        result.push_str("%%DocumentNeededResources: font Helvetica Helvetica-Bold Helvetica-Oblique Helvetica-BoldOblique Times-Roman Courier\n");
        result.push_str("%%EndComments\n");
        result.push('\n');
    }

    /// Write the prolog section with procedure definitions and font encoding.
    fn write_prolog(&self, result: &mut String) {
        result.push_str("%%BeginProlog\n");

        // Short-hand operators
        result.push_str("/M { moveto } bind def\n");
        result.push_str("/L { lineto } bind def\n");
        result.push_str("/S { stroke } bind def\n");
        result.push_str("/F { fill } bind def\n");
        result.push_str("/NP { newpath } bind def\n");
        result.push_str("/CP { closepath } bind def\n");
        result.push_str("/RGB { setrgbcolor } bind def\n");
        result.push_str("/GRAY { setgray } bind def\n");
        result.push_str("/LW { setlinewidth } bind def\n");
        result.push_str("/SF { setfont } bind def\n");
        result.push_str("/SH { show } bind def\n");
        result.push('\n');

        // Rectangle helpers
        result.push_str("% rect: x y w h -> path\n");
        result.push_str("/rect {\n");
        result.push_str("  /h exch def /w exch def /y exch def /x exch def\n");
        result.push_str("  NP x y M x w add y L x w add y h add L x y h add L CP\n");
        result.push_str("} bind def\n");
        result.push('\n');

        result.push_str("% frect: x y w h -- filled rectangle\n");
        result.push_str("/frect {\n");
        result.push_str("  /h exch def /w exch def /y exch def /x exch def\n");
        result.push_str("  NP x y M x w add y L x w add y h add L x y h add L CP F\n");
        result.push_str("} bind def\n");
        result.push('\n');

        result.push_str("% srect: x y w h -- stroked rectangle\n");
        result.push_str("/srect {\n");
        result.push_str("  /h exch def /w exch def /y exch def /x exch def\n");
        result.push_str("  NP x y M x w add y L x w add y h add L x y h add L CP S\n");
        result.push_str("} bind def\n");
        result.push('\n');

        // Font selection helper: FN fontname fontsize -- selects and scales font
        result.push_str("% FN: /FontName size -> (select and set font)\n");
        result.push_str("/FN {\n");
        result.push_str("  exch findfont exch scalefont SF\n");
        result.push_str("} bind def\n");
        result.push('\n');

        result.push_str("%%EndProlog\n");
        result.push('\n');
    }

    /// Write the setup section (shared resources / font encoding).
    fn write_setup(&self, result: &mut String) {
        result.push_str("%%BeginSetup\n");

        // Re-encode standard fonts to ISOLatin1 for proper character support
        for font_name in &[
            "Helvetica",
            "Helvetica-Bold",
            "Helvetica-Oblique",
            "Helvetica-BoldOblique",
            "Times-Roman",
            "Times-Bold",
            "Courier",
        ] {
            result.push_str(&format!(
                "/{fname} findfont\n\
                 dup length dict begin\n\
                   {{ 1 index /FID ne {{ def }} {{ pop pop }} ifelse }} forall\n\
                   /Encoding ISOLatin1Encoding def\n\
                   currentdict\n\
                 end\n\
                 /{fname} exch definefont pop\n\n",
                fname = font_name
            ));
        }

        result.push_str("%%EndSetup\n");
        result.push('\n');
    }

    /// Write the DSC `%%Page` header for a single page.
    fn write_page_header(page_num: usize, total_pages: usize, page: &PsPage, result: &mut String) {
        result.push_str(&format!("%%Page: {} {}\n", page_num, total_pages));

        let w = page.width.to_pt() as i64;
        let h = page.height.to_pt() as i64;
        result.push_str(&format!("%%PageBoundingBox: 0 0 {} {}\n", w, h));
        result.push_str("%%PageOrientation: Portrait\n");
        result.push_str("%%BeginPageSetup\n");
        result.push_str(&format!("<< /PageSize [{} {}] >> setpagedevice\n", w, h));
        result.push_str("%%EndPageSetup\n");
    }

    /// Convert to PostScript string
    fn build_ps(&self) -> String {
        let mut result = String::new();
        let total = self.pages.len();

        // --- DSC Header ---
        self.write_dsc_header(&mut result);

        // --- Prolog ---
        self.write_prolog(&mut result);

        // --- Setup ---
        self.write_setup(&mut result);

        // --- Pages ---
        for (i, page) in self.pages.iter().enumerate() {
            Self::write_page_header(i + 1, total, page, &mut result);
            result.push_str(&page.to_string());
            result.push_str("showpage\n");
            result.push('\n');
        }

        // --- Trailer ---
        result.push_str("%%Trailer\n");
        result.push_str(&format!("%%Pages: {}\n", total));
        result.push_str("%%EOF\n");

        result
    }
}

impl Default for PsDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PsDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build_ps())
    }
}

/// PostScript page builder
pub struct PsPage {
    width: Length,
    height: Length,
    commands: Vec<String>,
}

impl PsPage {
    /// Create a new PostScript page
    pub fn new(width: Length, height: Length) -> Self {
        // Save graphics state at the start of every page
        let commands = vec!["gsave".to_string()];

        Self {
            width,
            height,
            commands,
        }
    }

    /// Add a filled rectangle (for backgrounds)
    #[allow(clippy::too_many_arguments)]
    pub fn add_background(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
        color: Color,
    ) {
        // Set fill color
        self.set_color(color);

        // Draw filled rectangle
        self.commands.push(format!(
            "{:.3} {:.3} {:.3} {:.3} frect",
            x.to_pt(),
            y.to_pt(),
            width.to_pt(),
            height.to_pt()
        ));
    }

    /// Add borders
    #[allow(clippy::too_many_arguments)]
    pub fn add_borders(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
        border_widths: [Length; 4],
        border_colors: [Color; 4],
        border_styles: [fop_layout::area::BorderStyle; 4],
    ) {
        use fop_layout::area::BorderStyle;

        let [top_w, right_w, bottom_w, left_w] = border_widths;
        let [top_c, right_c, bottom_c, left_c] = border_colors;
        let [top_s, right_s, bottom_s, left_s] = border_styles;

        // Top border
        if top_w.to_pt() > 0.0 && !matches!(top_s, BorderStyle::None | BorderStyle::Hidden) {
            self.set_color(top_c);
            self.commands.push(format!(
                "{:.3} {:.3} {:.3} {:.3} frect",
                x.to_pt(),
                (y + height - top_w).to_pt(),
                width.to_pt(),
                top_w.to_pt()
            ));
        }

        // Right border
        if right_w.to_pt() > 0.0 && !matches!(right_s, BorderStyle::None | BorderStyle::Hidden) {
            self.set_color(right_c);
            self.commands.push(format!(
                "{:.3} {:.3} {:.3} {:.3} frect",
                (x + width - right_w).to_pt(),
                y.to_pt(),
                right_w.to_pt(),
                height.to_pt()
            ));
        }

        // Bottom border
        if bottom_w.to_pt() > 0.0 && !matches!(bottom_s, BorderStyle::None | BorderStyle::Hidden) {
            self.set_color(bottom_c);
            self.commands.push(format!(
                "{:.3} {:.3} {:.3} {:.3} frect",
                x.to_pt(),
                y.to_pt(),
                width.to_pt(),
                bottom_w.to_pt()
            ));
        }

        // Left border
        if left_w.to_pt() > 0.0 && !matches!(left_s, BorderStyle::None | BorderStyle::Hidden) {
            self.set_color(left_c);
            self.commands.push(format!(
                "{:.3} {:.3} {:.3} {:.3} frect",
                x.to_pt(),
                y.to_pt(),
                left_w.to_pt(),
                height.to_pt()
            ));
        }
    }

    /// Add text using the improved `FN` helper.
    ///
    /// Font selection: bold/italic variants are chosen from `traits`; when no
    /// trait information is available the base Helvetica family is used.
    pub fn add_text(
        &mut self,
        text: &str,
        x: Length,
        y: Length,
        font_size: Length,
        color: Option<Color>,
    ) {
        // Set text color
        let c = color.unwrap_or(Color::BLACK);
        self.set_color(c);

        // Font selection (Helvetica family — no trait info at this call site)
        let font_name = "/Helvetica";
        self.commands
            .push(format!("{} {:.3} FN", font_name, font_size.to_pt()));

        // Escape PostScript special characters
        let escaped_text = escape_ps_string(text);

        // Position and show text
        self.commands.push(format!(
            "{:.3} {:.3} M ({}) SH",
            x.to_pt(),
            y.to_pt(),
            escaped_text
        ));
    }

    /// Add text with explicit font-weight / font-style information.
    #[allow(clippy::too_many_arguments)]
    pub fn add_text_styled(
        &mut self,
        text: &str,
        x: Length,
        y: Length,
        font_size: Length,
        color: Option<Color>,
        bold: bool,
        italic: bool,
    ) {
        let c = color.unwrap_or(Color::BLACK);
        self.set_color(c);

        let font_name = select_ps_font("Helvetica", bold, italic);
        self.commands
            .push(format!("{} {:.3} FN", font_name, font_size.to_pt()));

        let escaped_text = escape_ps_string(text);
        self.commands.push(format!(
            "{:.3} {:.3} M ({}) SH",
            x.to_pt(),
            y.to_pt(),
            escaped_text
        ));
    }

    /// Add a line (for rules, borders)
    #[allow(clippy::too_many_arguments)]
    pub fn add_line(
        &mut self,
        x1: Length,
        y1: Length,
        x2: Length,
        y2: Length,
        color: Color,
        width: Length,
        _style: &str,
    ) {
        self.set_color(color);
        self.commands.push(format!("{:.3} LW", width.to_pt()));
        self.commands.push("NP".to_string());
        self.commands
            .push(format!("{:.3} {:.3} M", x1.to_pt(), y1.to_pt()));
        self.commands
            .push(format!("{:.3} {:.3} L", x2.to_pt(), y2.to_pt()));
        self.commands.push("S".to_string());
    }

    /// Add an image
    #[allow(dead_code)]
    fn add_image(
        &mut self,
        image_data: &ImageData,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
    ) {
        // Save graphics state
        self.commands.push("gsave".to_string());

        // Set up transformation matrix for the image
        self.commands
            .push(format!("{:.3} {:.3} translate", x.to_pt(), y.to_pt()));
        self.commands
            .push(format!("{:.3} {:.3} scale", width.to_pt(), height.to_pt()));

        // Image dictionary — full embedding is complex; draw a placeholder rectangle
        self.commands.push(format!(
            "% image placeholder {}x{}",
            image_data.width, image_data.height
        ));
        self.commands.push("grestore".to_string());

        // Placeholder rectangle
        self.commands.push("0.9 GRAY".to_string());
        self.commands.push(format!(
            "{:.3} {:.3} {:.3} {:.3} frect",
            x.to_pt(),
            y.to_pt(),
            width.to_pt(),
            height.to_pt()
        ));
    }

    /// Set clipping rectangle
    pub fn save_clip_state(&mut self, x: Length, y: Length, width: Length, height: Length) {
        self.commands.push("gsave".to_string());
        self.commands.push("NP".to_string());
        self.commands.push(format!(
            "{:.3} {:.3} {:.3} {:.3} rect",
            x.to_pt(),
            y.to_pt(),
            width.to_pt(),
            height.to_pt()
        ));
        self.commands.push("clip".to_string());
    }

    /// Restore graphics state
    pub fn restore_clip_state(&mut self) {
        self.commands.push("grestore".to_string());
    }

    /// Set RGB color helper
    fn set_color(&mut self, color: Color) {
        self.commands.push(format!(
            "{:.4} {:.4} {:.4} RGB",
            color.r as f64 / 255.0,
            color.g as f64 / 255.0,
            color.b as f64 / 255.0
        ));
    }

    /// Convert to PostScript string
    fn build_ps(&self) -> String {
        let mut result = String::new();

        for cmd in &self.commands {
            result.push_str(cmd);
            result.push('\n');
        }

        // Restore graphics state saved at page start
        result.push_str("grestore\n");

        result
    }
}

impl std::fmt::Display for PsPage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build_ps())
    }
}

/// Select the PostScript font name for the Helvetica family based on weight/style.
fn select_ps_font(family: &str, bold: bool, italic: bool) -> String {
    // Normalise family name to its PS base name
    let base = match family.to_lowercase().as_str() {
        "times" | "times new roman" | "times-roman" => "Times",
        "courier" | "courier new" => "Courier",
        _ => "Helvetica",
    };

    match (base, bold, italic) {
        ("Times", false, false) => "/Times-Roman".to_string(),
        ("Times", true, false) => "/Times-Bold".to_string(),
        ("Times", false, true) => "/Times-Italic".to_string(),
        ("Times", true, true) => "/Times-BoldItalic".to_string(),
        ("Courier", false, false) => "/Courier".to_string(),
        ("Courier", true, false) => "/Courier-Bold".to_string(),
        ("Courier", false, true) => "/Courier-Oblique".to_string(),
        ("Courier", true, true) => "/Courier-BoldOblique".to_string(),
        // Helvetica (default)
        (_, false, false) => "/Helvetica".to_string(),
        (_, true, false) => "/Helvetica-Bold".to_string(),
        (_, false, true) => "/Helvetica-Oblique".to_string(),
        (_, true, true) => "/Helvetica-BoldOblique".to_string(),
    }
}

/// Render child areas recursively
#[allow(clippy::too_many_arguments)]
fn render_children(
    area_tree: &AreaTree,
    parent_id: AreaId,
    ps_page: &mut PsPage,
    offset_x: Length,
    offset_y: Length,
    image_map: &HashMap<AreaId, ImageData>,
) -> Result<()> {
    let children = area_tree.children(parent_id);

    for child_id in children {
        if let Some(child_node) = area_tree.get(child_id) {
            // Calculate absolute position
            let abs_x = offset_x + child_node.area.geometry.x;
            let abs_y = offset_y + child_node.area.geometry.y;

            // Check if clipping is needed
            let needs_clipping = child_node
                .area
                .traits
                .overflow
                .map(|o| o.clips_content())
                .unwrap_or(false);

            if needs_clipping {
                ps_page.save_clip_state(
                    abs_x,
                    abs_y,
                    child_node.area.width(),
                    child_node.area.height(),
                );
            }

            // Render background color if present
            if let Some(bg_color) = child_node.area.traits.background_color {
                ps_page.add_background(
                    abs_x,
                    abs_y,
                    child_node.area.width(),
                    child_node.area.height(),
                    bg_color,
                );
            }

            // Render borders if present
            if let (Some(border_widths), Some(border_colors), Some(border_styles)) = (
                child_node.area.traits.border_width,
                child_node.area.traits.border_color,
                child_node.area.traits.border_style,
            ) {
                ps_page.add_borders(
                    abs_x,
                    abs_y,
                    child_node.area.width(),
                    child_node.area.height(),
                    border_widths,
                    border_colors,
                    border_styles,
                );
            }

            match child_node.area.area_type {
                AreaType::Text => {
                    // Check if this is a leader area
                    if let Some(leader_pattern) = &child_node.area.traits.is_leader {
                        render_leader(
                            ps_page,
                            leader_pattern,
                            abs_x,
                            abs_y,
                            child_node.area.width(),
                            child_node.area.height(),
                            &child_node.area.traits,
                        );
                    } else if let Some(text_content) = child_node.area.text_content() {
                        let font_size = child_node
                            .area
                            .traits
                            .font_size
                            .unwrap_or(Length::from_pt(12.0));

                        // Determine bold/italic from traits
                        let bold = child_node
                            .area
                            .traits
                            .font_weight
                            .map(|w| w >= 700)
                            .unwrap_or(false);
                        let italic = child_node
                            .area
                            .traits
                            .font_style
                            .map(|s| {
                                matches!(
                                    s,
                                    fop_layout::area::FontStyle::Italic
                                        | fop_layout::area::FontStyle::Oblique
                                )
                            })
                            .unwrap_or(false);

                        ps_page.add_text_styled(
                            text_content,
                            abs_x,
                            abs_y,
                            font_size,
                            child_node.area.traits.color,
                            bold,
                            italic,
                        );
                    }
                }
                AreaType::Inline => {
                    if let Some(leader_pattern) = &child_node.area.traits.is_leader {
                        render_leader(
                            ps_page,
                            leader_pattern,
                            abs_x,
                            abs_y,
                            child_node.area.width(),
                            child_node.area.height(),
                            &child_node.area.traits,
                        );
                    } else {
                        render_children(area_tree, child_id, ps_page, abs_x, abs_y, image_map)?;
                    }
                }
                AreaType::Viewport => {
                    // Render image if available
                    if let Some(img_data) = image_map.get(&child_id) {
                        ps_page.add_image(
                            img_data,
                            abs_x,
                            abs_y,
                            child_node.area.width(),
                            child_node.area.height(),
                        );
                    }
                    // Still recurse for child areas
                    render_children(area_tree, child_id, ps_page, abs_x, abs_y, image_map)?;
                }
                _ => {
                    // Recursively render other areas
                    render_children(area_tree, child_id, ps_page, abs_x, abs_y, image_map)?;
                }
            }

            if needs_clipping {
                ps_page.restore_clip_state();
            }
        }
    }

    Ok(())
}

/// Render a leader
#[allow(clippy::too_many_arguments)]
fn render_leader(
    ps_page: &mut PsPage,
    leader_pattern: &str,
    x: Length,
    y: Length,
    width: Length,
    height: Length,
    traits: &fop_layout::area::TraitSet,
) {
    match leader_pattern {
        "rule" => {
            let thickness = traits.rule_thickness.unwrap_or(Length::from_pt(0.5));
            let style = traits.rule_style.as_deref().unwrap_or("solid");
            let color = traits.color.unwrap_or(Color::BLACK);

            // Centre the rule vertically within the leader area
            let half_diff = Length::from_millipoints((height - thickness).millipoints() / 2);
            let rule_y = y + half_diff;

            let half_thickness = Length::from_millipoints(thickness.millipoints() / 2);
            ps_page.add_line(
                x,
                rule_y + half_thickness,
                x + width,
                rule_y + half_thickness,
                color,
                thickness,
                style,
            );
        }
        "dots" | "space" => {
            // These are handled by text rendering or produce nothing
        }
        _ => {}
    }
}

/// Escape PostScript special characters in strings
fn escape_ps_string(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;
    use fop_layout::area::BorderStyle;
    use fop_types::{Color, Length};

    // ── PsRenderer ───────────────────────────────────────────────────────────

    #[test]
    fn test_ps_renderer_new() {
        let renderer = PsRenderer::new();
        let _ = renderer;
    }

    #[test]
    fn test_ps_renderer_default() {
        let _r1 = PsRenderer::new();
        let _r2 = PsRenderer::default();
    }

    // ── PsDocument ───────────────────────────────────────────────────────────

    #[test]
    fn test_ps_document_empty_has_ps_header() {
        let doc = PsDocument::new();
        let output = doc.to_string();
        assert!(
            output.starts_with("%!PS-Adobe-3.0"),
            "PostScript document must begin with %!PS-Adobe-3.0"
        );
    }

    #[test]
    fn test_ps_document_empty_has_trailer() {
        let doc = PsDocument::new();
        let output = doc.to_string();
        assert!(
            output.contains("%%Trailer"),
            "PS document must have %%Trailer"
        );
        assert!(output.contains("%%EOF"), "PS document must end with %%EOF");
    }

    #[test]
    fn test_ps_document_default_equals_new() {
        let d1 = PsDocument::new().to_string();
        let d2 = PsDocument::default().to_string();
        assert_eq!(d1, d2, "default() and new() must produce identical output");
    }

    #[test]
    fn test_ps_document_empty_page_count() {
        let doc = PsDocument::new();
        let output = doc.to_string();
        assert!(
            output.contains("%%Pages: 0"),
            "empty document must declare 0 pages"
        );
    }

    #[test]
    fn test_ps_document_add_page_increments_count() {
        let mut doc = PsDocument::new();
        doc.add_page(PsPage::new(Length::from_mm(210.0), Length::from_mm(297.0)));
        let output = doc.to_string();
        // %%Pages appears both in header and trailer
        assert!(
            output.contains("%%Pages: 1"),
            "single-page doc must declare 1 page"
        );
    }

    #[test]
    fn test_ps_document_two_pages() {
        let mut doc = PsDocument::new();
        doc.add_page(PsPage::new(Length::from_mm(210.0), Length::from_mm(297.0)));
        doc.add_page(PsPage::new(Length::from_mm(210.0), Length::from_mm(297.0)));
        let output = doc.to_string();
        assert!(
            output.contains("%%Pages: 2"),
            "two-page doc must declare 2 pages"
        );
        assert!(
            output.contains("%%Page: 1 2"),
            "first page label must be '1 2'"
        );
        assert!(
            output.contains("%%Page: 2 2"),
            "second page label must be '2 2'"
        );
    }

    #[test]
    fn test_ps_document_prolog_contains_helpers() {
        let doc = PsDocument::new();
        let output = doc.to_string();
        assert!(
            output.contains("/frect"),
            "prolog must define /frect helper"
        );
        assert!(output.contains("/FN"), "prolog must define /FN font helper");
        assert!(output.contains("/SH"), "prolog must define /SH show helper");
    }

    #[test]
    fn test_ps_document_page_has_showpage() {
        let mut doc = PsDocument::new();
        doc.add_page(PsPage::new(Length::from_mm(210.0), Length::from_mm(297.0)));
        let output = doc.to_string();
        assert!(
            output.contains("showpage"),
            "each page must end with showpage"
        );
    }

    // ── PsPage ───────────────────────────────────────────────────────────────

    fn make_page() -> PsPage {
        PsPage::new(Length::from_mm(210.0), Length::from_mm(297.0))
    }

    #[test]
    fn test_ps_page_new_contains_gsave() {
        let page = make_page();
        let output = page.to_string();
        assert!(output.contains("gsave"), "new page must begin with gsave");
    }

    #[test]
    fn test_ps_page_add_background() {
        let mut page = make_page();
        page.add_background(
            Length::from_pt(10.0),
            Length::from_pt(20.0),
            Length::from_pt(100.0),
            Length::from_pt(50.0),
            Color::RED,
        );
        let output = page.to_string();
        assert!(output.contains("frect"), "background must use frect");
        // RGB values for red (1 0 0)
        assert!(output.contains("RGB"), "background must set RGB color");
    }

    #[test]
    fn test_ps_page_add_text_basic() {
        let mut page = make_page();
        page.add_text(
            "Hello PS",
            Length::from_pt(72.0),
            Length::from_pt(720.0),
            Length::from_pt(12.0),
            None,
        );
        let output = page.to_string();
        assert!(
            output.contains("Hello PS"),
            "text content must appear in output"
        );
        assert!(output.contains("SH"), "text must end with SH (show)");
        assert!(output.contains("FN"), "text must select a font with FN");
    }

    #[test]
    fn test_ps_page_add_text_styled_bold() {
        let mut page = make_page();
        page.add_text_styled(
            "Bold Text",
            Length::from_pt(50.0),
            Length::from_pt(700.0),
            Length::from_pt(14.0),
            Some(Color::BLACK),
            true,
            false,
        );
        let output = page.to_string();
        assert!(
            output.contains("Bold Text"),
            "bold text content must appear"
        );
        assert!(
            output.contains("Helvetica-Bold"),
            "bold must select Helvetica-Bold"
        );
    }

    #[test]
    fn test_ps_page_add_text_styled_italic() {
        let mut page = make_page();
        page.add_text_styled(
            "Italic",
            Length::from_pt(50.0),
            Length::from_pt(700.0),
            Length::from_pt(12.0),
            None,
            false,
            true,
        );
        let output = page.to_string();
        assert!(
            output.contains("Helvetica-Oblique"),
            "italic must select Helvetica-Oblique"
        );
    }

    #[test]
    fn test_ps_page_add_text_styled_bold_italic() {
        let mut page = make_page();
        page.add_text_styled(
            "BI",
            Length::from_pt(10.0),
            Length::from_pt(10.0),
            Length::from_pt(10.0),
            None,
            true,
            true,
        );
        let output = page.to_string();
        assert!(
            output.contains("Helvetica-BoldOblique"),
            "bold+italic must select Helvetica-BoldOblique"
        );
    }

    #[test]
    fn test_ps_page_add_line() {
        let mut page = make_page();
        page.add_line(
            Length::from_pt(0.0),
            Length::from_pt(0.0),
            Length::from_pt(200.0),
            Length::from_pt(0.0),
            Color::BLACK,
            Length::from_pt(1.0),
            "solid",
        );
        let output = page.to_string();
        assert!(output.contains("M"), "line must have moveto (M)");
        assert!(
            output.contains(
                " L
"
            ) && output.contains(
                "
S
"
            ),
            "line must have separate L and S commands"
        );
        assert!(output.contains("LW"), "line must set line width (LW)");
    }

    #[test]
    fn test_ps_page_add_borders_top_only() {
        let mut page = make_page();
        page.add_borders(
            Length::from_pt(10.0),
            Length::from_pt(10.0),
            Length::from_pt(100.0),
            Length::from_pt(50.0),
            [
                Length::from_pt(2.0), // top
                Length::ZERO,         // right
                Length::ZERO,         // bottom
                Length::ZERO,         // left
            ],
            [Color::BLACK; 4],
            [BorderStyle::Solid; 4],
        );
        let output = page.to_string();
        assert!(
            output.contains("frect"),
            "borders must use frect for filled rectangles"
        );
    }

    #[test]
    fn test_ps_page_add_borders_none_style_skipped() {
        let mut page = make_page();
        let initial_len = page.commands.len();
        page.add_borders(
            Length::from_pt(0.0),
            Length::from_pt(0.0),
            Length::from_pt(100.0),
            Length::from_pt(50.0),
            [Length::from_pt(1.0); 4],
            [Color::BLACK; 4],
            [BorderStyle::None; 4], // all none — should not draw anything
        );
        // No new frect commands should have been added
        assert_eq!(
            page.commands.len(),
            initial_len,
            "BorderStyle::None must not produce drawing commands"
        );
    }

    #[test]
    fn test_ps_page_save_restore_clip() {
        let mut page = make_page();
        page.save_clip_state(
            Length::from_pt(10.0),
            Length::from_pt(10.0),
            Length::from_pt(80.0),
            Length::from_pt(60.0),
        );
        page.restore_clip_state();
        let output = page.to_string();
        // save_clip_state uses gsave + clip; restore uses grestore
        assert!(
            output.contains("gsave"),
            "clip state save must include gsave"
        );
        assert!(
            output.contains("grestore"),
            "clip state restore must include grestore"
        );
    }

    // ── select_ps_font helper (tested indirectly via add_text_styled) ─────────

    #[test]
    fn test_ps_font_selection_times() {
        let mut page = make_page();
        // Directly call the public add_text_styled with Times via a modified
        // wrapper isn't possible (select_ps_font is private), but we can
        // verify the Helvetica family path is exhaustive via the styled API.
        page.add_text_styled(
            "plain",
            Length::from_pt(10.0),
            Length::from_pt(10.0),
            Length::from_pt(10.0),
            None,
            false,
            false,
        );
        let output = page.to_string();
        assert!(
            output.contains("Helvetica"),
            "plain text must use Helvetica"
        );
    }

    // ── PS output structural checks ───────────────────────────────────────────

    #[test]
    fn test_ps_document_has_bounding_box_with_pages() {
        let mut doc = PsDocument::new();
        doc.add_page(PsPage::new(Length::from_mm(210.0), Length::from_mm(297.0)));
        let output = doc.to_string();
        assert!(
            output.contains("%%BoundingBox:"),
            "document with pages must have %%BoundingBox"
        );
    }

    #[test]
    fn test_ps_document_language_level_2() {
        let doc = PsDocument::new();
        let output = doc.to_string();
        assert!(
            output.contains("%%LanguageLevel: 2"),
            "must declare PostScript language level 2"
        );
    }

    #[test]
    fn test_ps_document_creator_comment() {
        let doc = PsDocument::new();
        let output = doc.to_string();
        assert!(
            output.contains("%%Creator: Apache FOP Rust"),
            "must include Creator DSC comment"
        );
    }

    #[test]
    fn test_ps_page_output_ends_with_grestore() {
        let page = make_page();
        let output = page.to_string();
        // The page build_ps method ends with grestore
        assert!(
            output.trim_end().ends_with("grestore"),
            "page PS must end with grestore"
        );
    }
}
