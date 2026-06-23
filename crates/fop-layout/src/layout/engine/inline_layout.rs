//! Inline, leader, list-item, and external-graphic layout methods.
//!
//! Handles `fo:inline`, `fo:leader`, `fo:list-item`, `fo:external-graphic`
//! and related image loading logic.

use crate::area::{Area, AreaTree, AreaType, TraitSet};
use crate::layout::{
    extract_space_after, extract_space_before, extract_traits, ListLayout, PageNumberResolver,
};
use fop_core::{FoArena, FoNodeData, NodeId, PropertyId};
use fop_types::{Length, Rect, Result};

use super::types::parse_fo_length;
use super::LayoutEngine;

impl LayoutEngine {
    /// Layout an inline element (text within a link or inline container)
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::only_used_in_recursion)]
    pub(super) fn layout_inline(
        &self,
        fo_tree: &FoArena,
        node_id: NodeId,
        area_tree: &mut AreaTree,
        parent_area: crate::area::AreaId,
        available_width: Length,
        line_height: Length,
        traits: &TraitSet,
        first_line_indent: Length,
        y_offset: Length,
    ) -> Result<()> {
        let node = fo_tree
            .get(node_id)
            .ok_or_else(|| fop_types::FopError::Generic(format!("Node {} not found", node_id)))?;

        match &node.data {
            FoNodeData::Text(text) => {
                // Create text area with traits from parent (link or inline)
                // Apply text-indent to first line; stack at the caller's y_offset.
                let x_offset = first_line_indent;
                let line_width = available_width - first_line_indent;
                let text_rect = Rect::new(x_offset, y_offset, line_width, line_height);
                let text_area = Area::text(text_rect, text.clone()).with_traits(traits.clone());
                let text_id = area_tree.add_area(text_area);
                area_tree
                    .append_child(parent_area, text_id)
                    .map_err(fop_types::FopError::Generic)?;
            }
            FoNodeData::Inline { properties } => {
                // Merge traits from inline element
                let mut merged_traits = traits.clone();
                let inline_traits = extract_traits(properties);

                // Override with inline-specific traits
                if inline_traits.color.is_some() {
                    merged_traits.color = inline_traits.color;
                }
                if inline_traits.text_decoration.is_some() {
                    merged_traits.text_decoration = inline_traits.text_decoration;
                }

                // Process inline children
                let children = fo_tree.children(node_id);
                for child_id in children {
                    self.layout_inline(
                        fo_tree,
                        child_id,
                        area_tree,
                        parent_area,
                        available_width,
                        line_height,
                        &merged_traits,
                        first_line_indent,
                        y_offset,
                    )?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Layout a leader element (dots, rule, space, etc.)
    #[allow(clippy::too_many_arguments)]
    pub(super) fn layout_leader(
        &self,
        _fo_tree: &FoArena,
        _node_id: NodeId,
        properties: &fop_core::PropertyList,
        area_tree: &mut AreaTree,
        parent_area: crate::area::AreaId,
        available_width: Length,
        line_height: Length,
        parent_traits: &TraitSet,
        y_offset: Length,
    ) -> Result<()> {
        use fop_core::PropertyId;

        // Extract leader properties
        let leader_pattern = properties
            .get(PropertyId::LeaderPattern)
            .ok()
            .and_then(|v| v.as_string().map(|s| s.to_string()))
            .unwrap_or_else(|| "space".to_string());

        // For leader-length, we use available width as default
        // In a full implementation, this would parse min/opt/max values
        let leader_width = available_width;

        // Generate leader content based on pattern
        let leader_content = match leader_pattern.as_str() {
            "dots" => {
                // Get pattern width (spacing between dots)
                let pattern_width = properties
                    .get(PropertyId::LeaderPatternWidth)
                    .ok()
                    .and_then(|v| v.as_length())
                    .unwrap_or(Length::from_pt(5.0));

                // Calculate number of dots
                let num_dots = (leader_width.to_pt() / pattern_width.to_pt()).floor() as usize;
                ".".repeat(num_dots.max(1))
            }
            "rule" => {
                // Rule will be rendered as a line, not text
                // We'll use a special marker that the renderer will recognize
                String::new()
            }
            "space" => {
                // Just whitespace
                String::new()
            }
            _ => {
                // Unknown pattern, default to space
                String::new()
            }
        };

        // Create leader area, stacked at the caller's y_offset.
        let leader_rect = Rect::new(Length::ZERO, y_offset, leader_width, line_height);

        // Create traits for leader, inheriting from parent
        let mut leader_traits = parent_traits.clone();

        // Check for rule-specific properties
        if leader_pattern == "rule" {
            // Store rule properties in traits
            if let Ok(thickness) = properties.get(PropertyId::RuleThickness) {
                if let Some(len) = thickness.as_length() {
                    leader_traits.rule_thickness = Some(len);
                }
            }

            if let Ok(style) = properties.get(PropertyId::RuleStyle) {
                if let Some(style_str) = style.as_string() {
                    leader_traits.rule_style = Some(style_str.to_string());
                }
            }
        }

        // Mark this as a leader area for special rendering
        leader_traits.is_leader = Some(leader_pattern.clone());

        let leader_area = if leader_content.is_empty() {
            // For rule and space, create an inline area without text
            Area::new(AreaType::Inline, leader_rect).with_traits(leader_traits)
        } else {
            // For dots, create a text area
            Area::text(leader_rect, leader_content).with_traits(leader_traits)
        };

        let leader_id = area_tree.add_area(leader_area);
        area_tree
            .append_child(parent_area, leader_id)
            .map_err(fop_types::FopError::Generic)?;

        Ok(())
    }

    /// Layout a list item
    ///
    /// Handles `fo:list-item-label` and `fo:list-item-body` children, resolving
    /// `label-end()` and `body-start()` functions using the list layout context.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn layout_list_item(
        &self,
        fo_tree: &FoArena,
        item_id: NodeId,
        area_tree: &mut AreaTree,
        list_id: crate::area::AreaId,
        y_offset: Length,
        _index: usize,
        list_layout: &ListLayout,
        resolver: &mut PageNumberResolver,
    ) -> Result<Length> {
        // Extract space-before and space-after from the list-item properties
        let (item_space_before, item_space_after) = if let Some(item_node) = fo_tree.get(item_id) {
            let props = item_node.data.properties();
            let sb = props
                .map(|p| extract_space_before(p))
                .unwrap_or(Length::ZERO);
            let sa = props
                .map(|p| extract_space_after(p))
                .unwrap_or(Length::ZERO);
            (sb, sa)
        } else {
            (Length::ZERO, Length::ZERO)
        };

        let item_start_y = y_offset + item_space_before;

        let label_x = Length::ZERO;
        let label_width = list_layout.label_end();
        let body_x = list_layout.body_start();
        let body_width = list_layout.body_width();

        // --- Label area ---
        // Placeholder rect; height will be updated after content layout
        let label_rect = Rect::new(label_x, item_start_y, label_width, Length::from_pt(12.0));
        let label_area = Area::new(AreaType::Block, label_rect);
        let label_id = area_tree.add_area(label_area);
        area_tree
            .append_child(list_id, label_id)
            .map_err(fop_types::FopError::Generic)?;

        // --- Body area ---
        let body_rect = Rect::new(body_x, item_start_y, body_width, Length::from_pt(12.0));
        let body_area = Area::new(AreaType::Block, body_rect);
        let body_id = area_tree.add_area(body_area);
        area_tree
            .append_child(list_id, body_id)
            .map_err(fop_types::FopError::Generic)?;

        // --- Process children ---
        let item_children = fo_tree.children(item_id);
        let mut label_height = Length::from_pt(12.0);
        let mut body_height = Length::from_pt(12.0);

        for child_id in &item_children {
            if let Some(child) = fo_tree.get(*child_id) {
                match &child.data {
                    FoNodeData::ListItemLabel { .. } => {
                        // Layout label content
                        let label_children = fo_tree.children(*child_id);
                        let mut label_y = Length::ZERO;
                        for content_id in label_children {
                            if let Some(area_id) = self.layout_block(
                                fo_tree,
                                content_id,
                                area_tree,
                                label_id,
                                label_y,
                                label_width,
                                resolver,
                            )? {
                                if let Some(child_area) = area_tree.get(area_id) {
                                    label_y = child_area.area.geometry.y + child_area.area.height();
                                }
                            }
                        }
                        label_height = label_y.max(Length::from_pt(12.0));
                    }
                    FoNodeData::ListItemBody { .. } => {
                        // Layout body content
                        let body_children = fo_tree.children(*child_id);
                        let mut body_y = Length::ZERO;
                        for content_id in body_children {
                            if let Some(area_id) = self.layout_block(
                                fo_tree, content_id, area_tree, body_id, body_y, body_width,
                                resolver,
                            )? {
                                if let Some(child_area) = area_tree.get(area_id) {
                                    body_y = child_area.area.geometry.y + child_area.area.height();
                                }
                            }
                        }
                        body_height = body_y.max(Length::from_pt(12.0));
                    }
                    _ => {}
                }
            }
        }

        // The item height is the maximum of label and body heights
        let item_height = label_height.max(body_height);

        // Update label and body area heights
        if let Some(label_node) = area_tree.get_mut(label_id) {
            label_node.area.geometry.height = item_height;
        }
        if let Some(body_node) = area_tree.get_mut(body_id) {
            body_node.area.geometry.height = item_height;
        }

        Ok(item_start_y + item_height + item_space_after)
    }

    /// Layout an external-graphic element.
    ///
    /// Handles `content-width` and `content-height` with keyword values
    /// (`scale-to-fit`, `scale-down-to-fit`, `scale-up-to-fit`, `auto`)
    /// as well as explicit lengths.  Supports `scaling="uniform"` (default)
    /// and `scaling="non-uniform"`.  When the image file cannot be loaded
    /// a placeholder area with the requested dimensions is created so that
    /// surrounding content is not disrupted.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn layout_external_graphic(
        &self,
        _fo_tree: &FoArena,
        _node_id: NodeId,
        src: &str,
        content_width_attr: Option<&str>,
        content_height_attr: Option<&str>,
        scaling_attr: Option<&str>,
        properties: &fop_core::PropertyList,
        area_tree: &mut AreaTree,
        parent_area: crate::area::AreaId,
    ) -> Option<crate::area::AreaId> {
        // Attempt to load image; on failure use a placeholder with requested size
        let image_data_opt = self.load_image(src).ok();

        // Intrinsic dimensions from actual image data (if available)
        let intrinsic_dims = image_data_opt
            .as_deref()
            .and_then(|d| self.get_image_dimensions(d));

        // Default intrinsic size when no image is available: 50 mm x 50 mm
        let default_intrinsic = (Length::from_mm(50.0), Length::from_mm(50.0));
        let (intr_w, intr_h) = intrinsic_dims.unwrap_or(default_intrinsic);

        // Explicit `width` / `height` properties set the display viewport
        let explicit_width = properties
            .get(PropertyId::Width)
            .ok()
            .and_then(|v| v.as_length());
        let explicit_height = properties
            .get(PropertyId::Height)
            .ok()
            .and_then(|v| v.as_length());

        // Resolve content-width / content-height string value.
        // Prefer the explicit attribute; fall back to the property list string.
        let cw_str: Option<String> = content_width_attr.map(str::to_string).or_else(|| {
            properties
                .get(PropertyId::ContentWidth)
                .ok()
                .and_then(|v| v.as_string().map(str::to_string))
        });
        let ch_str: Option<String> = content_height_attr.map(str::to_string).or_else(|| {
            properties
                .get(PropertyId::ContentHeight)
                .ok()
                .and_then(|v| v.as_string().map(str::to_string))
        });

        let is_scale_kw = |s: Option<&str>| {
            matches!(
                s,
                Some("scale-to-fit") | Some("scale-down-to-fit") | Some("scale-up-to-fit")
            )
        };

        let cw_is_keyword = is_scale_kw(cw_str.as_deref());
        let ch_is_keyword = is_scale_kw(ch_str.as_deref());

        // Determine the display viewport size
        let display_w = explicit_width.unwrap_or(intr_w);
        let display_h = explicit_height.unwrap_or(intr_h);

        // Compute content (rendered) dimensions
        let (content_w, content_h) = if cw_is_keyword || ch_is_keyword {
            // At least one dimension uses a scaling keyword
            let scale_mode = cw_str
                .as_deref()
                .or(ch_str.as_deref())
                .unwrap_or("scale-to-fit");
            let uniform = scaling_attr.unwrap_or("uniform") != "non-uniform";

            if uniform {
                let scale_x = display_w.to_pt() / intr_w.to_pt().max(f64::EPSILON);
                let scale_y = display_h.to_pt() / intr_h.to_pt().max(f64::EPSILON);
                let scale = match scale_mode {
                    "scale-to-fit" => scale_x.min(scale_y),
                    "scale-down-to-fit" => scale_x.min(scale_y).min(1.0),
                    "scale-up-to-fit" => scale_x.min(scale_y).max(1.0),
                    _ => scale_x.min(scale_y),
                };
                (
                    Length::from_pt(intr_w.to_pt() * scale),
                    Length::from_pt(intr_h.to_pt() * scale),
                )
            } else {
                // Non-uniform: each axis scaled independently
                let scale_x = display_w.to_pt() / intr_w.to_pt().max(f64::EPSILON);
                let scale_y = display_h.to_pt() / intr_h.to_pt().max(f64::EPSILON);
                let (sx, sy) = match scale_mode {
                    "scale-to-fit" => (scale_x, scale_y),
                    "scale-down-to-fit" => (scale_x.min(1.0), scale_y.min(1.0)),
                    "scale-up-to-fit" => (scale_x.max(1.0), scale_y.max(1.0)),
                    _ => (scale_x, scale_y),
                };
                (
                    Length::from_pt(intr_w.to_pt() * sx),
                    Length::from_pt(intr_h.to_pt() * sy),
                )
            }
        } else {
            // No scale keyword: try explicit content-length, then intrinsic
            let cw = cw_str
                .as_deref()
                .filter(|s| *s != "auto")
                .and_then(parse_fo_length)
                .or_else(|| {
                    properties
                        .get(PropertyId::ContentWidth)
                        .ok()
                        .and_then(|v| v.as_length())
                })
                .unwrap_or(intr_w);
            let ch = ch_str
                .as_deref()
                .filter(|s| *s != "auto")
                .and_then(parse_fo_length)
                .or_else(|| {
                    properties
                        .get(PropertyId::ContentHeight)
                        .ok()
                        .and_then(|v| v.as_length())
                })
                .unwrap_or(intr_h);
            (cw, ch)
        };

        // Area rect: display width x content height
        let area_w = display_w;
        let area_h = content_h;
        // content_w stored for potential future use (e.g. clipping)
        let _ = content_w;

        let image_rect = Rect::new(Length::ZERO, Length::ZERO, area_w, area_h);

        let image_area = match image_data_opt {
            Some(data) => Area::viewport_with_image(image_rect, data),
            None => Area::new(AreaType::Block, image_rect),
        };

        let image_id = area_tree.add_area(image_area);
        if area_tree.append_child(parent_area, image_id).is_ok() {
            Some(image_id)
        } else {
            None
        }
    }

    /// Load image data from file path or URL
    pub(super) fn load_image(&self, src: &str) -> Result<Vec<u8>> {
        // For now, only support file paths
        // URL support would require http client

        // Remove "url(" prefix and ")" suffix if present
        let clean_src = src
            .trim()
            .strip_prefix("url(")
            .and_then(|s| s.strip_suffix(')'))
            .unwrap_or(src)
            .trim()
            .trim_matches(|c| c == '"' || c == '\'');

        // Read file
        std::fs::read(clean_src).map_err(|e| {
            fop_types::FopError::Generic(format!("Failed to load image {}: {}", clean_src, e))
        })
    }

    /// Get intrinsic dimensions of an image in points (1/72 inch)
    ///
    /// Returns (width, height) in points, assuming 96 DPI for pixel-based formats.
    pub(super) fn get_image_dimensions(&self, image_data: &[u8]) -> Option<(Length, Length)> {
        use image::GenericImageView;

        // Try to decode the image to get its dimensions
        let img = image::load_from_memory(image_data).ok()?;
        let (width_px, height_px) = img.dimensions();

        // Assume 96 DPI (standard screen resolution) for converting pixels to points
        // 1 point = 1/72 inch, 1 pixel at 96 DPI = 1/96 inch
        // Therefore: points = pixels * (72/96) = pixels * 0.75
        let width = Length::from_pt((width_px as f64) * 0.75);
        let height = Length::from_pt((height_px as f64) * 0.75);

        Some((width, height))
    }
}
