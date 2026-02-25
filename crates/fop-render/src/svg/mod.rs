//! SVG rendering backend
//!
//! Generates SVG documents from area trees.

use crate::image::ImageInfo;
use fop_layout::{AreaId, AreaTree, AreaType};
use fop_types::{Color, Length, Result};
use std::collections::HashMap;

/// SVG renderer
pub struct SvgRenderer {
    /// Default page width (A4)
    #[allow(dead_code)]
    page_width: Length,

    /// Default page height (A4)
    #[allow(dead_code)]
    page_height: Length,
}

impl SvgRenderer {
    /// Create a new SVG renderer
    pub fn new() -> Self {
        Self {
            page_width: Length::from_mm(210.0),
            page_height: Length::from_mm(297.0),
        }
    }

    /// Render an area tree to an SVG document (returns SVG XML as String)
    pub fn render_to_svg(&self, area_tree: &AreaTree) -> Result<String> {
        let mut svg_doc = SvgDocument::new();

        // First pass: collect all images and convert to data URIs
        let mut image_map = HashMap::new();
        self.collect_images(area_tree, &mut image_map)?;

        // Second pass: render pages
        for (id, node) in area_tree.iter() {
            if matches!(node.area.area_type, AreaType::Page) {
                let page_svg = self.render_page(area_tree, id, &image_map)?;
                svg_doc.add_page_svg(page_svg);
            }
        }

        Ok(svg_doc.to_string())
    }

    /// Render an area tree to separate SVG documents, one per page
    ///
    /// Returns a vector of SVG strings, one for each page in the area tree.
    pub fn render_to_svg_pages(&self, area_tree: &AreaTree) -> Result<Vec<String>> {
        // First pass: collect all images and convert to data URIs
        let mut image_map = HashMap::new();
        self.collect_images(area_tree, &mut image_map)?;

        // Second pass: render each page as a separate SVG document
        let mut pages = Vec::new();
        for (id, node) in area_tree.iter() {
            if matches!(node.area.area_type, AreaType::Page) {
                let page_svg = self.render_page(area_tree, id, &image_map)?;

                // Create a standalone SVG document for this page
                let mut svg_doc = SvgDocument::new();
                svg_doc.add_page_svg(page_svg);
                pages.push(svg_doc.to_string());
            }
        }

        Ok(pages)
    }

    /// Collect all images from the area tree and convert to data URIs
    fn collect_images(
        &self,
        area_tree: &AreaTree,
        image_map: &mut HashMap<AreaId, String>,
    ) -> Result<()> {
        for (id, node) in area_tree.iter() {
            if matches!(node.area.area_type, AreaType::Viewport) {
                if let Some(image_data) = node.area.image_data() {
                    let data_uri = self.create_data_uri(image_data)?;
                    image_map.insert(id, data_uri);
                }
            }
        }
        Ok(())
    }

    /// Create a data URI from image data
    fn create_data_uri(&self, image_data: &[u8]) -> Result<String> {
        let image_info = ImageInfo::from_bytes(image_data)?;
        let mime_type = match image_info.format {
            crate::image::ImageFormat::PNG => "image/png",
            crate::image::ImageFormat::JPEG => "image/jpeg",
            crate::image::ImageFormat::Unknown => "application/octet-stream",
        };

        // Base64 encode the image data
        let encoded = base64_encode(image_data);
        Ok(format!("data:{};base64,{}", mime_type, encoded))
    }

    /// Render a single page
    fn render_page(
        &self,
        area_tree: &AreaTree,
        page_id: AreaId,
        image_map: &HashMap<AreaId, String>,
    ) -> Result<String> {
        let page_node = area_tree
            .get(page_id)
            .ok_or_else(|| fop_types::FopError::Generic("Page not found".to_string()))?;

        let width = page_node.area.width();
        let height = page_node.area.height();

        let mut svg = SvgGraphics::new(width, height);

        // Render all child areas recursively with absolute positioning
        render_children(
            area_tree,
            page_id,
            &mut svg,
            Length::ZERO,
            Length::ZERO,
            image_map,
        )?;

        Ok(svg.to_string())
    }
}

impl Default for SvgRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// SVG document builder - manages multiple pages
pub struct SvgDocument {
    pages: Vec<String>,
}

impl SvgDocument {
    /// Create a new SVG document
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }

    /// Add a page SVG to the document
    pub fn add_page_svg(&mut self, page_svg: String) {
        self.pages.push(page_svg);
    }

    /// Convert to SVG XML string
    fn build_svg(&self) -> String {
        if self.pages.is_empty() {
            return String::from(
                r#"<?xml version="1.0" encoding="UTF-8"?><svg xmlns="http://www.w3.org/2000/svg"/>"#,
            );
        }

        // For multi-page SVG, we'll stack them vertically
        // Each page is a separate <g> group
        let mut result = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        result.push('\n');

        if self.pages.len() == 1 {
            // Single page - just return it
            result.push_str(&self.pages[0]);
        } else {
            // Multiple pages - stack them vertically
            result.push_str(r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">"#);
            result.push('\n');

            for (i, page) in self.pages.iter().enumerate() {
                result.push_str(&format!(r#"  <g id="page-{}">"#, i + 1));
                result.push('\n');
                // Extract content from page SVG (skip XML declaration and outer svg tag)
                if let Some(content) = extract_svg_content(page) {
                    for line in content.lines() {
                        result.push_str("    ");
                        result.push_str(line);
                        result.push('\n');
                    }
                }
                result.push_str("  </g>\n");
            }

            result.push_str("</svg>\n");
        }

        result
    }
}

impl Default for SvgDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SvgDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build_svg())
    }
}

/// SVG graphics builder for a single page
pub struct SvgGraphics {
    width: Length,
    height: Length,
    elements: Vec<String>,
    gradients: Vec<String>,
}

impl SvgGraphics {
    /// Create a new SVG graphics builder
    pub fn new(width: Length, height: Length) -> Self {
        Self {
            width,
            height,
            elements: Vec::new(),
            gradients: Vec::new(),
        }
    }

    /// Add a rectangle (for backgrounds, borders)
    #[allow(clippy::too_many_arguments)]
    pub fn add_rect(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
        fill: Option<Color>,
        stroke: Option<Color>,
        stroke_width: Option<Length>,
        opacity: Option<f64>,
        rx: Option<Length>,
    ) {
        let mut rect = format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}""#,
            x.to_pt(),
            y.to_pt(),
            width.to_pt(),
            height.to_pt()
        );

        if let Some(color) = fill {
            rect.push_str(&format!(r#" fill="{}""#, color_to_svg(&color)));
        } else {
            rect.push_str(r#" fill="none""#);
        }

        if let Some(color) = stroke {
            rect.push_str(&format!(r#" stroke="{}""#, color_to_svg(&color)));
        }

        if let Some(sw) = stroke_width {
            rect.push_str(&format!(r#" stroke-width="{}""#, sw.to_pt()));
        }

        if let Some(op) = opacity {
            if (op - 1.0).abs() > f64::EPSILON {
                rect.push_str(&format!(r#" opacity="{}""#, op));
            }
        }

        if let Some(radius) = rx {
            if radius.to_pt() > 0.0 {
                rect.push_str(&format!(r#" rx="{}""#, radius.to_pt()));
            }
        }

        rect.push_str(" />");
        self.elements.push(rect);
    }

    /// Add text with full font styling
    #[allow(clippy::too_many_arguments)]
    pub fn add_text(
        &mut self,
        text: &str,
        x: Length,
        y: Length,
        font_size: Length,
        color: Option<Color>,
    ) {
        self.add_text_styled(text, x, y, font_size, color, None, None, None, None);
    }

    /// Add text with full styling (font-family, font-weight, font-style, text-decoration)
    #[allow(clippy::too_many_arguments)]
    pub fn add_text_styled(
        &mut self,
        text: &str,
        x: Length,
        y: Length,
        font_size: Length,
        color: Option<Color>,
        font_family: Option<&str>,
        font_weight: Option<u16>,
        font_style_italic: Option<bool>,
        text_decoration: Option<(bool, bool, bool)>,
    ) {
        let fill_color = color.unwrap_or(Color::BLACK);

        // Escape XML special characters
        let escaped_text = escape_xml(text);

        let mut text_elem = format!(
            r#"<text x="{}" y="{}" font-size="{}" fill="{}""#,
            x.to_pt(),
            y.to_pt(),
            font_size.to_pt(),
            color_to_svg(&fill_color),
        );

        // Add font-family
        if let Some(family) = font_family {
            if !family.is_empty() {
                text_elem.push_str(&format!(r#" font-family="{}""#, escape_xml(family)));
            }
        }

        // Add font-weight
        if let Some(weight) = font_weight {
            if weight != 400 {
                text_elem.push_str(&format!(r#" font-weight="{}""#, weight));
            }
        }

        // Add font-style
        if let Some(is_italic) = font_style_italic {
            if is_italic {
                text_elem.push_str(r#" font-style="italic""#);
            }
        }

        // Add text-decoration
        if let Some((underline, overline, line_through)) = text_decoration {
            let mut decorations = Vec::new();
            if underline {
                decorations.push("underline");
            }
            if overline {
                decorations.push("overline");
            }
            if line_through {
                decorations.push("line-through");
            }
            if !decorations.is_empty() {
                text_elem.push_str(&format!(r#" text-decoration="{}""#, decorations.join(" ")));
            }
        }

        text_elem.push_str(&format!(">{}</text>", escaped_text));
        self.elements.push(text_elem);
    }

    /// Add a linear gradient to the defs section
    pub fn add_linear_gradient(
        &mut self,
        id: &str,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        stops: &[(f64, Color)],
    ) {
        let mut grad = format!(
            r#"<linearGradient id="{}" x1="{}%" y1="{}%" x2="{}%" y2="{}%">"#,
            id,
            x1 * 100.0,
            y1 * 100.0,
            x2 * 100.0,
            y2 * 100.0
        );
        for (offset, color) in stops {
            grad.push_str(&format!(
                r#"<stop offset="{}%" stop-color="{}"/>"#,
                offset * 100.0,
                color_to_svg(color)
            ));
        }
        grad.push_str("</linearGradient>");
        self.gradients.push(grad);
    }

    /// Add a rect filled with a gradient (reference gradient by id)
    pub fn add_gradient_rect(
        &mut self,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
        gradient_id: &str,
    ) {
        let rect = format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="url(#{})" />"#,
            x.to_pt(),
            y.to_pt(),
            width.to_pt(),
            height.to_pt(),
            gradient_id
        );
        self.elements.push(rect);
    }

    /// Add an image
    pub fn add_image(
        &mut self,
        data_uri: &str,
        x: Length,
        y: Length,
        width: Length,
        height: Length,
    ) {
        let image_elem = format!(
            r#"<image x="{}" y="{}" width="{}" height="{}" href="{}" />"#,
            x.to_pt(),
            y.to_pt(),
            width.to_pt(),
            height.to_pt(),
            data_uri
        );
        self.elements.push(image_elem);
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
        style: &str,
    ) {
        let mut line = format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}""#,
            x1.to_pt(),
            y1.to_pt(),
            x2.to_pt(),
            y2.to_pt(),
            color_to_svg(&color),
            width.to_pt()
        );

        // Handle line styles
        match style {
            "dashed" => line.push_str(r#" stroke-dasharray="5,5""#),
            "dotted" => line.push_str(r#" stroke-dasharray="2,2""#),
            _ => {} // solid is default
        }

        line.push_str(" />");
        self.elements.push(line);
    }

    /// Start a clipping path
    pub fn start_clip(&mut self, x: Length, y: Length, width: Length, height: Length) {
        let clip = format!(
            r#"<g clip-path="url(#clip-{}-{})"><defs><clipPath id="clip-{}-{}"><rect x="{}" y="{}" width="{}" height="{}"/></clipPath></defs>"#,
            x.to_pt(),
            y.to_pt(),
            x.to_pt(),
            y.to_pt(),
            x.to_pt(),
            y.to_pt(),
            width.to_pt(),
            height.to_pt()
        );
        self.elements.push(clip);
    }

    /// End a clipping path
    pub fn end_clip(&mut self) {
        self.elements.push("</g>".to_string());
    }

    /// Convert to SVG XML string
    fn build_svg(&self) -> String {
        let mut svg = String::new();

        svg.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{}" height="{}" viewBox="0 0 {} {}">"#,
            self.width.to_pt(),
            self.height.to_pt(),
            self.width.to_pt(),
            self.height.to_pt()
        ));
        svg.push('\n');

        // Add gradients if any
        if !self.gradients.is_empty() {
            svg.push_str("  <defs>\n");
            for gradient in &self.gradients {
                svg.push_str("    ");
                svg.push_str(gradient);
                svg.push('\n');
            }
            svg.push_str("  </defs>\n");
        }

        // Add elements
        for element in &self.elements {
            svg.push_str("  ");
            svg.push_str(element);
            svg.push('\n');
        }

        svg.push_str("</svg>");
        svg
    }
}

impl std::fmt::Display for SvgGraphics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build_svg())
    }
}

/// Render child areas recursively
#[allow(clippy::too_many_arguments)]
fn render_children(
    area_tree: &AreaTree,
    parent_id: AreaId,
    svg: &mut SvgGraphics,
    offset_x: Length,
    offset_y: Length,
    image_map: &HashMap<AreaId, String>,
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
                svg.start_clip(
                    abs_x,
                    abs_y,
                    child_node.area.width(),
                    child_node.area.height(),
                );
            }

            // Get opacity if set
            let opacity = child_node.area.traits.opacity;

            // Render background color if present
            if let Some(bg_color) = child_node.area.traits.background_color {
                let border_radius = child_node.area.traits.border_radius.map(|r| r[0]);
                svg.add_rect(
                    abs_x,
                    abs_y,
                    child_node.area.width(),
                    child_node.area.height(),
                    Some(bg_color),
                    None,
                    None,
                    opacity,
                    border_radius,
                );
            }

            // Render borders if present
            if let (Some(border_widths), Some(border_colors), Some(border_styles)) = (
                child_node.area.traits.border_width,
                child_node.area.traits.border_color,
                child_node.area.traits.border_style,
            ) {
                render_borders(
                    svg,
                    abs_x,
                    abs_y,
                    child_node.area.width(),
                    child_node.area.height(),
                    border_widths,
                    border_colors,
                    border_styles,
                    opacity,
                );
            }

            match child_node.area.area_type {
                AreaType::Text => {
                    // Check if this is a leader area
                    if let Some(leader_pattern) = &child_node.area.traits.is_leader {
                        render_leader(
                            svg,
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

                        // SVG text baseline is at the bottom, so we add font_size to y
                        let text_y = abs_y + font_size;

                        let font_family = child_node.area.traits.font_family.as_deref();
                        let font_weight = child_node.area.traits.font_weight;
                        let font_style_italic = child_node.area.traits.font_style.map(|s| {
                            matches!(
                                s,
                                fop_layout::area::FontStyle::Italic
                                    | fop_layout::area::FontStyle::Oblique
                            )
                        });
                        let text_deco = child_node
                            .area
                            .traits
                            .text_decoration
                            .map(|td| (td.underline, td.overline, td.line_through));

                        svg.add_text_styled(
                            text_content,
                            abs_x,
                            text_y,
                            font_size,
                            child_node.area.traits.color,
                            font_family,
                            font_weight,
                            font_style_italic,
                            text_deco,
                        );
                    }
                }
                AreaType::Inline => {
                    if let Some(leader_pattern) = &child_node.area.traits.is_leader {
                        render_leader(
                            svg,
                            leader_pattern,
                            abs_x,
                            abs_y,
                            child_node.area.width(),
                            child_node.area.height(),
                            &child_node.area.traits,
                        );
                    } else {
                        render_children(area_tree, child_id, svg, abs_x, abs_y, image_map)?;
                    }
                }
                AreaType::Viewport => {
                    // Render image if available
                    if let Some(data_uri) = image_map.get(&child_id) {
                        svg.add_image(
                            data_uri,
                            abs_x,
                            abs_y,
                            child_node.area.width(),
                            child_node.area.height(),
                        );
                    }
                    // Still recurse for child areas
                    render_children(area_tree, child_id, svg, abs_x, abs_y, image_map)?;
                }
                _ => {
                    // Recursively render other areas
                    render_children(area_tree, child_id, svg, abs_x, abs_y, image_map)?;
                }
            }

            if needs_clipping {
                svg.end_clip();
            }
        }
    }

    Ok(())
}

/// Render borders
#[allow(clippy::too_many_arguments)]
fn render_borders(
    svg: &mut SvgGraphics,
    x: Length,
    y: Length,
    width: Length,
    height: Length,
    border_widths: [Length; 4],
    border_colors: [Color; 4],
    border_styles: [fop_layout::area::BorderStyle; 4],
    opacity: Option<f64>,
) {
    use fop_layout::area::BorderStyle;

    let [top_w, right_w, bottom_w, left_w] = border_widths;
    let [top_c, right_c, bottom_c, left_c] = border_colors;
    let [top_s, right_s, bottom_s, left_s] = border_styles;

    // Top border
    if top_w.to_pt() > 0.0 && !matches!(top_s, BorderStyle::None | BorderStyle::Hidden) {
        let mut rect = format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="none""#,
            x.to_pt(),
            y.to_pt(),
            width.to_pt(),
            top_w.to_pt(),
            color_to_svg(&top_c)
        );
        if let Some(op) = opacity {
            if (op - 1.0).abs() > f64::EPSILON {
                rect.push_str(&format!(r#" opacity="{}""#, op));
            }
        }
        rect.push_str(" />");
        svg.elements.push(rect);
    }

    // Right border
    if right_w.to_pt() > 0.0 && !matches!(right_s, BorderStyle::None | BorderStyle::Hidden) {
        let mut rect = format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="none""#,
            (x + width - right_w).to_pt(),
            y.to_pt(),
            right_w.to_pt(),
            height.to_pt(),
            color_to_svg(&right_c)
        );
        if let Some(op) = opacity {
            if (op - 1.0).abs() > f64::EPSILON {
                rect.push_str(&format!(r#" opacity="{}""#, op));
            }
        }
        rect.push_str(" />");
        svg.elements.push(rect);
    }

    // Bottom border
    if bottom_w.to_pt() > 0.0 && !matches!(bottom_s, BorderStyle::None | BorderStyle::Hidden) {
        let mut rect = format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="none""#,
            x.to_pt(),
            (y + height - bottom_w).to_pt(),
            width.to_pt(),
            bottom_w.to_pt(),
            color_to_svg(&bottom_c)
        );
        if let Some(op) = opacity {
            if (op - 1.0).abs() > f64::EPSILON {
                rect.push_str(&format!(r#" opacity="{}""#, op));
            }
        }
        rect.push_str(" />");
        svg.elements.push(rect);
    }

    // Left border
    if left_w.to_pt() > 0.0 && !matches!(left_s, BorderStyle::None | BorderStyle::Hidden) {
        let mut rect = format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="none""#,
            x.to_pt(),
            y.to_pt(),
            left_w.to_pt(),
            height.to_pt(),
            color_to_svg(&left_c)
        );
        if let Some(op) = opacity {
            if (op - 1.0).abs() > f64::EPSILON {
                rect.push_str(&format!(r#" opacity="{}""#, op));
            }
        }
        rect.push_str(" />");
        svg.elements.push(rect);
    }
}

/// Render a leader
#[allow(clippy::too_many_arguments)]
fn render_leader(
    svg: &mut SvgGraphics,
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

            // Center the rule vertically
            let half_diff = Length::from_millipoints((height - thickness).millipoints() / 2);
            let rule_y = y + half_diff;

            let half_thickness = Length::from_millipoints(thickness.millipoints() / 2);
            svg.add_line(
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
            // These are handled by text rendering or render nothing
        }
        _ => {}
    }
}

/// Convert Color to SVG color string
fn color_to_svg(color: &Color) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
}

/// Convert BorderStyle to SVG stroke-dasharray style
#[allow(dead_code)]
fn border_style_to_svg(style: &fop_layout::area::BorderStyle) -> &'static str {
    use fop_layout::area::BorderStyle;
    match style {
        BorderStyle::Solid => "solid",
        BorderStyle::Dashed => "dashed",
        BorderStyle::Dotted => "dotted",
        BorderStyle::Double => "double",
        _ => "solid",
    }
}

/// Escape XML special characters
fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Extract content from SVG (remove XML declaration and outer svg tags)
fn extract_svg_content(svg: &str) -> Option<String> {
    let mut lines: Vec<&str> = svg.lines().collect();

    // Skip XML declaration
    if let Some(first) = lines.first() {
        if first.starts_with("<?xml") {
            lines.remove(0);
        }
    }

    // Skip opening <svg> tag
    if let Some(first) = lines.first() {
        if first.trim().starts_with("<svg") {
            lines.remove(0);
        }
    }

    // Skip closing </svg> tag
    if let Some(last) = lines.last() {
        if last.trim() == "</svg>" {
            lines.pop();
        }
    }

    Some(lines.join("\n"))
}

/// Simple base64 encoding (without external dependency)
fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::new();
    let mut i = 0;

    while i < data.len() {
        let b1 = data[i];
        let b2 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b3 = if i + 2 < data.len() { data[i + 2] } else { 0 };

        let c1 = (b1 >> 2) as usize;
        let c2 = (((b1 & 0x03) << 4) | (b2 >> 4)) as usize;
        let c3 = (((b2 & 0x0F) << 2) | (b3 >> 6)) as usize;
        let c4 = (b3 & 0x3F) as usize;

        result.push(CHARSET[c1] as char);
        result.push(CHARSET[c2] as char);

        if i + 1 < data.len() {
            result.push(CHARSET[c3] as char);
        } else {
            result.push('=');
        }

        if i + 2 < data.len() {
            result.push(CHARSET[c4] as char);
        } else {
            result.push('=');
        }

        i += 3;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use fop_types::{Color, Length};

    // ── SvgRenderer ──────────────────────────────────────────────────────────

    #[test]
    fn test_svg_renderer_new() {
        let renderer = SvgRenderer::new();
        let _ = renderer; // construction must not panic
    }

    #[test]
    fn test_svg_renderer_default() {
        let _r1 = SvgRenderer::new();
        let _r2 = SvgRenderer::default();
        // both variants must be constructible
    }

    // ── SvgDocument ──────────────────────────────────────────────────────────

    #[test]
    fn test_svg_document_empty_output() {
        let doc = SvgDocument::new();
        let output = doc.to_string();
        // empty document should still produce valid SVG
        assert!(
            output.contains("<svg"),
            "empty document must have <svg element"
        );
    }

    #[test]
    fn test_svg_document_default_equals_new() {
        let doc_new = SvgDocument::new();
        let doc_default = SvgDocument::default();
        assert_eq!(doc_new.to_string(), doc_default.to_string());
    }

    #[test]
    fn test_svg_document_single_page() {
        let mut doc = SvgDocument::new();
        doc.add_page_svg(r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100"/></svg>"#.to_string());
        let output = doc.to_string();
        assert!(
            output.contains("<svg"),
            "single-page document must contain <svg"
        );
    }

    #[test]
    fn test_svg_document_multi_page_contains_svg() {
        let mut doc = SvgDocument::new();
        doc.add_page_svg(r#"<svg xmlns="http://www.w3.org/2000/svg"><rect/></svg>"#.to_string());
        doc.add_page_svg(r#"<svg xmlns="http://www.w3.org/2000/svg"><circle/></svg>"#.to_string());
        let output = doc.to_string();
        assert!(
            output.contains("<svg"),
            "multi-page document must contain <svg"
        );
        // multi-page wraps in groups
        assert!(output.contains("page-1"), "must label first page group");
        assert!(output.contains("page-2"), "must label second page group");
    }

    #[test]
    fn test_svg_document_xml_declaration() {
        let doc = SvgDocument::new();
        let output = doc.to_string();
        assert!(
            output.starts_with("<?xml"),
            "SVG document must start with XML declaration"
        );
    }

    // ── SvgGraphics ──────────────────────────────────────────────────────────

    fn make_graphics() -> SvgGraphics {
        SvgGraphics::new(Length::from_mm(210.0), Length::from_mm(297.0))
    }

    #[test]
    fn test_svg_graphics_new_produces_svg_element() {
        let g = make_graphics();
        let output = g.to_string();
        assert!(
            output.contains("<svg"),
            "graphics must produce <svg element"
        );
        assert!(output.contains("</svg>"), "graphics must close </svg>");
    }

    #[test]
    fn test_svg_graphics_width_height_in_output() {
        let g = SvgGraphics::new(Length::from_pt(200.0), Length::from_pt(400.0));
        let output = g.to_string();
        // viewBox and width/height should reflect the supplied values
        assert!(output.contains("200"), "width should appear in SVG output");
        assert!(output.contains("400"), "height should appear in SVG output");
    }

    #[test]
    fn test_svg_graphics_add_rect_with_fill() {
        let mut g = make_graphics();
        g.add_rect(
            Length::from_pt(10.0),
            Length::from_pt(20.0),
            Length::from_pt(100.0),
            Length::from_pt(50.0),
            Some(Color::RED),
            None,
            None,
            None,
            None,
        );
        let output = g.to_string();
        assert!(output.contains("<rect"), "must contain rect element");
        assert!(output.contains("fill"), "must include fill attribute");
        assert!(
            output.contains("#ff0000"),
            "red fill should be #ff0000 but got: {output}"
        );
    }

    #[test]
    fn test_svg_graphics_add_rect_no_fill() {
        let mut g = make_graphics();
        g.add_rect(
            Length::ZERO,
            Length::ZERO,
            Length::from_pt(50.0),
            Length::from_pt(50.0),
            None,
            None,
            None,
            None,
            None,
        );
        let output = g.to_string();
        assert!(
            output.contains(r#"fill="none""#),
            "unfilled rect should have fill=\"none\""
        );
    }

    #[test]
    fn test_svg_graphics_add_rect_with_stroke() {
        let mut g = make_graphics();
        g.add_rect(
            Length::ZERO,
            Length::ZERO,
            Length::from_pt(80.0),
            Length::from_pt(40.0),
            None,
            Some(Color::BLACK),
            Some(Length::from_pt(1.0)),
            None,
            None,
        );
        let output = g.to_string();
        assert!(output.contains("stroke"), "must include stroke attribute");
    }

    #[test]
    fn test_svg_graphics_add_rect_with_opacity() {
        let mut g = make_graphics();
        g.add_rect(
            Length::ZERO,
            Length::ZERO,
            Length::from_pt(80.0),
            Length::from_pt(40.0),
            Some(Color::BLUE),
            None,
            None,
            Some(0.5),
            None,
        );
        let output = g.to_string();
        assert!(
            output.contains("opacity"),
            "partial opacity should appear in output"
        );
    }

    #[test]
    fn test_svg_graphics_add_rect_with_radius() {
        let mut g = make_graphics();
        g.add_rect(
            Length::ZERO,
            Length::ZERO,
            Length::from_pt(80.0),
            Length::from_pt(40.0),
            Some(Color::GREEN),
            None,
            None,
            None,
            Some(Length::from_pt(5.0)),
        );
        let output = g.to_string();
        assert!(
            output.contains("rx"),
            "border-radius must produce rx attribute"
        );
    }

    #[test]
    fn test_svg_graphics_add_text() {
        let mut g = make_graphics();
        g.add_text(
            "Hello SVG",
            Length::from_pt(50.0),
            Length::from_pt(100.0),
            Length::from_pt(12.0),
            Some(Color::BLACK),
        );
        let output = g.to_string();
        assert!(output.contains("<text"), "must produce <text element");
        assert!(output.contains("Hello SVG"), "text content must be present");
    }

    #[test]
    fn test_svg_graphics_add_text_xml_escape() {
        let mut g = make_graphics();
        g.add_text(
            "a < b & c > d",
            Length::from_pt(10.0),
            Length::from_pt(20.0),
            Length::from_pt(10.0),
            None,
        );
        let output = g.to_string();
        assert!(output.contains("&lt;"), "< must be escaped as &lt;");
        assert!(output.contains("&amp;"), "& must be escaped as &amp;");
        assert!(output.contains("&gt;"), "> must be escaped as &gt;");
    }

    #[test]
    fn test_svg_graphics_add_text_styled_bold() {
        let mut g = make_graphics();
        g.add_text_styled(
            "Bold Text",
            Length::from_pt(10.0),
            Length::from_pt(20.0),
            Length::from_pt(14.0),
            Some(Color::BLACK),
            Some("Helvetica"),
            Some(700),
            None,
            None,
        );
        let output = g.to_string();
        assert!(output.contains("font-weight"), "bold must set font-weight");
        assert!(output.contains("Bold Text"), "text content must be present");
    }

    #[test]
    fn test_svg_graphics_add_text_styled_italic() {
        let mut g = make_graphics();
        g.add_text_styled(
            "Italic Text",
            Length::from_pt(10.0),
            Length::from_pt(20.0),
            Length::from_pt(12.0),
            None,
            None,
            None,
            Some(true),
            None,
        );
        let output = g.to_string();
        assert!(output.contains("font-style"), "italic must set font-style");
        assert!(output.contains("italic"), "font-style must be italic");
    }

    #[test]
    fn test_svg_graphics_add_line_solid() {
        let mut g = make_graphics();
        g.add_line(
            Length::from_pt(0.0),
            Length::from_pt(0.0),
            Length::from_pt(100.0),
            Length::from_pt(100.0),
            Color::BLACK,
            Length::from_pt(1.0),
            "solid",
        );
        let output = g.to_string();
        assert!(output.contains("<line"), "must produce <line element");
        assert!(
            !output.contains("stroke-dasharray"),
            "solid line must not have dasharray"
        );
    }

    #[test]
    fn test_svg_graphics_add_line_dashed() {
        let mut g = make_graphics();
        g.add_line(
            Length::from_pt(0.0),
            Length::from_pt(0.0),
            Length::from_pt(50.0),
            Length::from_pt(0.0),
            Color::BLACK,
            Length::from_pt(1.0),
            "dashed",
        );
        let output = g.to_string();
        assert!(
            output.contains("stroke-dasharray"),
            "dashed line must have dasharray"
        );
        assert!(output.contains("5,5"), "dashed dasharray should be 5,5");
    }

    #[test]
    fn test_svg_graphics_add_line_dotted() {
        let mut g = make_graphics();
        g.add_line(
            Length::from_pt(0.0),
            Length::from_pt(0.0),
            Length::from_pt(50.0),
            Length::from_pt(0.0),
            Color::BLACK,
            Length::from_pt(1.0),
            "dotted",
        );
        let output = g.to_string();
        assert!(output.contains("2,2"), "dotted dasharray should be 2,2");
    }

    #[test]
    fn test_svg_graphics_start_end_clip() {
        let mut g = make_graphics();
        g.start_clip(
            Length::from_pt(10.0),
            Length::from_pt(10.0),
            Length::from_pt(80.0),
            Length::from_pt(60.0),
        );
        g.end_clip();
        let output = g.to_string();
        assert!(
            output.contains("clip-path"),
            "clip must produce clip-path attribute"
        );
        assert!(output.contains("clipPath"), "clip must define a <clipPath>");
        assert!(output.contains("</g>"), "end_clip must close the group");
    }

    // ── color_to_svg helper (private, tested via add_rect) ───────────────────

    #[test]
    fn test_color_to_svg_black() {
        let mut g = make_graphics();
        g.add_rect(
            Length::ZERO,
            Length::ZERO,
            Length::from_pt(10.0),
            Length::from_pt(10.0),
            Some(Color::BLACK),
            None,
            None,
            None,
            None,
        );
        assert!(g.to_string().contains("#000000"));
    }

    #[test]
    fn test_color_to_svg_white() {
        let mut g = make_graphics();
        g.add_rect(
            Length::ZERO,
            Length::ZERO,
            Length::from_pt(10.0),
            Length::from_pt(10.0),
            Some(Color::WHITE),
            None,
            None,
            None,
            None,
        );
        assert!(g.to_string().contains("#ffffff"));
    }

    // ── Namespace / structure validity ────────────────────────────────────────

    #[test]
    fn test_svg_graphics_has_xmlns() {
        let g = make_graphics();
        let output = g.to_string();
        assert!(
            output.contains(r#"xmlns="http://www.w3.org/2000/svg""#),
            "SVG must declare the SVG namespace"
        );
    }

    #[test]
    fn test_svg_graphics_has_xlink_ns() {
        let g = make_graphics();
        let output = g.to_string();
        assert!(
            output.contains("xmlns:xlink"),
            "SVG must declare the xlink namespace for image support"
        );
    }

    #[test]
    fn test_svg_graphics_multiple_elements_order() {
        let mut g = make_graphics();
        g.add_rect(
            Length::ZERO,
            Length::ZERO,
            Length::from_pt(50.0),
            Length::from_pt(50.0),
            Some(Color::RED),
            None,
            None,
            None,
            None,
        );
        g.add_text(
            "First",
            Length::from_pt(5.0),
            Length::from_pt(10.0),
            Length::from_pt(12.0),
            None,
        );
        let output = g.to_string();
        let rect_pos = output.find("<rect").expect("test: should succeed");
        let text_pos = output.find("<text").expect("test: should succeed");
        assert!(
            rect_pos < text_pos,
            "rect added first must appear before text in output"
        );
    }
}
