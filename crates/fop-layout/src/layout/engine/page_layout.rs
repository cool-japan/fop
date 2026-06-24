//! Page, flow, float, static-content, and marker layout methods.
//!
//! Handles page region geometry extraction, flow layout with float support,
//! static-content (header/footer/sidebars), and marker collection/retrieval.

use crate::area::{Area, AreaTree, AreaType, TraitSet};
use crate::layout::{
    extract_end_indent, extract_keep_constraint, extract_space_after, extract_space_before,
    extract_start_indent, extract_text_indent, extract_traits, BlockLayoutContext,
    PageNumberResolver, TextAlign,
};
use fop_core::{FoArena, FoNodeData, NodeId, PropertyId};
use fop_types::{Length, Point, Rect, Result, Size};

use super::markers::{PageMarkerView, RetrieveBoundaryScope};
use super::types::{FloatInfo, FloatManager, FloatSide, PageRegionGeometry};
use super::LayoutEngine;

impl LayoutEngine {
    /// Extract page region geometry from the FO tree for a given master reference.
    ///
    /// Computes rectangles for all five XSL-FO page regions based on the
    /// `simple-page-master` attributes: page-width, page-height, margins, and
    /// region extents.  Falls back to A4 with 1-inch margins when the master
    /// cannot be found in the tree.
    pub(super) fn extract_page_region_geometry(
        &self,
        fo_tree: &FoArena,
        master_reference: &str,
    ) -> PageRegionGeometry {
        // Defaults: A4, 72pt (1 inch) margins, zero region extents
        let default_pw = self.page_width;
        let default_ph = self.page_height;
        let default_margin = Length::from_pt(72.0);
        let zero = Length::ZERO;

        // Try to find the simple-page-master node
        if let Some((root_id, _)) = fo_tree.root() {
            for lms_id in fo_tree.children(root_id) {
                if let Some(lms) = fo_tree.get(lms_id) {
                    if !matches!(lms.data, FoNodeData::LayoutMasterSet) {
                        continue;
                    }
                    for spm_id in fo_tree.children(lms_id) {
                        if let Some(spm) = fo_tree.get(spm_id) {
                            if let FoNodeData::SimplePageMaster {
                                master_name,
                                properties,
                            } = &spm.data
                            {
                                if master_name != master_reference {
                                    continue;
                                }
                                // Found the master — extract dimensions
                                let pw = properties
                                    .get(PropertyId::PageWidth)
                                    .ok()
                                    .and_then(|v| v.as_length())
                                    .unwrap_or(default_pw);
                                let ph = properties
                                    .get(PropertyId::PageHeight)
                                    .ok()
                                    .and_then(|v| v.as_length())
                                    .unwrap_or(default_ph);
                                let m_top = properties
                                    .get(PropertyId::MarginTop)
                                    .ok()
                                    .and_then(|v| v.as_length())
                                    .unwrap_or(default_margin);
                                let m_bottom = properties
                                    .get(PropertyId::MarginBottom)
                                    .ok()
                                    .and_then(|v| v.as_length())
                                    .unwrap_or(default_margin);
                                let m_left = properties
                                    .get(PropertyId::MarginLeft)
                                    .ok()
                                    .and_then(|v| v.as_length())
                                    .unwrap_or(default_margin);
                                let m_right = properties
                                    .get(PropertyId::MarginRight)
                                    .ok()
                                    .and_then(|v| v.as_length())
                                    .unwrap_or(default_margin);

                                // Region extents from child region nodes
                                let mut before_extent = zero;
                                let mut after_extent = zero;
                                let mut start_extent = zero;
                                let mut end_extent = zero;
                                let mut body_margin_top = zero;
                                let mut body_margin_bottom = zero;
                                let mut body_margin_left = zero;
                                let mut body_margin_right = zero;

                                for region_id in fo_tree.children(spm_id) {
                                    if let Some(region) = fo_tree.get(region_id) {
                                        match &region.data {
                                            FoNodeData::RegionBefore { properties: rp } => {
                                                before_extent = rp
                                                    .get(PropertyId::Extent)
                                                    .ok()
                                                    .and_then(|v| v.as_length())
                                                    .unwrap_or(zero);
                                            }
                                            FoNodeData::RegionAfter { properties: rp } => {
                                                after_extent = rp
                                                    .get(PropertyId::Extent)
                                                    .ok()
                                                    .and_then(|v| v.as_length())
                                                    .unwrap_or(zero);
                                            }
                                            FoNodeData::RegionStart { properties: rp } => {
                                                start_extent = rp
                                                    .get(PropertyId::Extent)
                                                    .ok()
                                                    .and_then(|v| v.as_length())
                                                    .unwrap_or(zero);
                                            }
                                            FoNodeData::RegionEnd { properties: rp } => {
                                                end_extent = rp
                                                    .get(PropertyId::Extent)
                                                    .ok()
                                                    .and_then(|v| v.as_length())
                                                    .unwrap_or(zero);
                                            }
                                            FoNodeData::RegionBody { properties: rp } => {
                                                // region-body can have its own inner margins
                                                body_margin_top = rp
                                                    .get(PropertyId::MarginTop)
                                                    .ok()
                                                    .and_then(|v| v.as_length())
                                                    .unwrap_or(zero);
                                                body_margin_bottom = rp
                                                    .get(PropertyId::MarginBottom)
                                                    .ok()
                                                    .and_then(|v| v.as_length())
                                                    .unwrap_or(zero);
                                                body_margin_left = rp
                                                    .get(PropertyId::MarginLeft)
                                                    .ok()
                                                    .and_then(|v| v.as_length())
                                                    .unwrap_or(zero);
                                                body_margin_right = rp
                                                    .get(PropertyId::MarginRight)
                                                    .ok()
                                                    .and_then(|v| v.as_length())
                                                    .unwrap_or(zero);
                                            }
                                            _ => {}
                                        }
                                    }
                                }

                                // Compute region rectangles
                                // Content area (page minus page margins)
                                let content_x = m_left;
                                let content_y = m_top;
                                let content_w = pw - m_left - m_right;
                                let content_h = ph - m_top - m_bottom;

                                // region-before: top strip of content area
                                let before_rect = Rect::from_point_size(
                                    Point::new(content_x, content_y),
                                    Size::new(content_w, before_extent),
                                );

                                // region-after: bottom strip of content area
                                let after_rect = Rect::from_point_size(
                                    Point::new(content_x, content_y + content_h - after_extent),
                                    Size::new(content_w, after_extent),
                                );

                                // region-start: left strip (between before and after vertically)
                                let sidebar_top = content_y + before_extent;
                                let sidebar_height = content_h - before_extent - after_extent;
                                let start_rect = Rect::from_point_size(
                                    Point::new(content_x, sidebar_top),
                                    Size::new(start_extent, sidebar_height),
                                );

                                // region-end: right strip (between before and after vertically)
                                let end_rect = Rect::from_point_size(
                                    Point::new(content_x + content_w - end_extent, sidebar_top),
                                    Size::new(end_extent, sidebar_height),
                                );

                                // region-body: remaining space + region-body inner margins
                                let body_x = content_x + start_extent + body_margin_left;
                                let body_y = sidebar_top + body_margin_top;
                                let body_w = content_w
                                    - start_extent
                                    - end_extent
                                    - body_margin_left
                                    - body_margin_right;
                                let body_h = sidebar_height - body_margin_top - body_margin_bottom;

                                let body_rect = Rect::from_point_size(
                                    Point::new(body_x, body_y),
                                    Size::new(body_w, body_h),
                                );

                                return PageRegionGeometry {
                                    page_width: pw,
                                    page_height: ph,
                                    before_rect,
                                    after_rect,
                                    start_rect,
                                    end_rect,
                                    body_rect,
                                };
                            }
                        }
                    }
                }
            }
        }

        // Fallback: A4 with 1-inch margins, no regions
        let body_rect = Rect::from_point_size(
            Point::new(default_margin, default_margin),
            Size::new(
                default_pw - default_margin * 2,
                default_ph - default_margin * 2,
            ),
        );
        let zero_rect = Rect::from_point_size(Point::ZERO, Size::new(zero, zero));
        PageRegionGeometry {
            page_width: default_pw,
            page_height: default_ph,
            before_rect: zero_rect,
            after_rect: zero_rect,
            start_rect: zero_rect,
            end_rect: zero_rect,
            body_rect,
        }
    }

    /// Layout a fo:float element within a flow, registering it with the FloatManager.
    ///
    /// Determines float side, measures float content, positions the float area at the
    /// correct x/y coordinates, and adds it to the FloatManager so subsequent blocks
    /// avoid the float.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn layout_float_in_flow(
        &self,
        fo_tree: &FoArena,
        node_id: NodeId,
        area_tree: &mut AreaTree,
        parent_area: crate::area::AreaId,
        current_y: Length,
        container_width: Length,
        is_odd_page: bool,
        float_manager: &mut FloatManager,
        resolver: &mut PageNumberResolver,
    ) -> Result<Option<crate::area::AreaId>> {
        let node = fo_tree
            .get(node_id)
            .ok_or_else(|| fop_types::FopError::Generic(format!("Node {} not found", node_id)))?;

        let properties = match &node.data {
            FoNodeData::Float { properties } => properties,
            _ => return Ok(None),
        };

        // Determine float side from the `float` property
        let float_side = if let Ok(prop) = properties.get(PropertyId::Float) {
            if let Some(enum_val) = prop.as_enum() {
                match enum_val {
                    66 => FloatSide::Left,
                    96 => FloatSide::Right,
                    104 => FloatSide::Start,
                    45 => FloatSide::End,
                    _ => FloatSide::None,
                }
            } else if let Some(string_val) = prop.as_string() {
                match string_val {
                    "left" => FloatSide::Left,
                    "right" => FloatSide::Right,
                    "start" => FloatSide::Start,
                    "end" => FloatSide::End,
                    "inside" => FloatSide::Inside,
                    "outside" => FloatSide::Outside,
                    _ => FloatSide::None,
                }
            } else {
                FloatSide::None
            }
        } else {
            FloatSide::None
        };

        if float_side == FloatSide::None {
            // Treat as a regular block if no float side is specified
            return self.layout_block(
                fo_tree,
                node_id,
                area_tree,
                parent_area,
                current_y,
                container_width,
                resolver,
            );
        }

        // Resolve effective float side (inside/outside depend on odd/even page)
        let effective_side = match float_side {
            FloatSide::Inside => {
                if is_odd_page {
                    FloatSide::Left
                } else {
                    FloatSide::Right
                }
            }
            FloatSide::Outside => {
                if is_odd_page {
                    FloatSide::Right
                } else {
                    FloatSide::Left
                }
            }
            other => other,
        };

        // Determine the float width: use 1/3 of container width as default,
        // or extract an explicit width from the float's block child if specified.
        let float_width = self.measure_float_width(fo_tree, node_id, container_width);

        // Compute float X position based on side
        let float_x = match effective_side {
            FloatSide::Left | FloatSide::Start => {
                // Left float: place at the current left offset
                float_manager.get_left_offset(current_y)
            }
            FloatSide::Right | FloatSide::End => {
                // Right float: place at container_width - existing right offsets - float_width
                container_width - float_manager.get_right_offset(current_y) - float_width
            }
            _ => Length::ZERO,
        };

        // Layout the float's children inside the float area
        let traits = extract_traits(properties);
        let mut float_ctx = BlockLayoutContext::new(float_width);

        // Temporary area to measure content height
        let temp_rect = Rect::new(float_x, current_y, float_width, Length::from_pt(1.0));
        let float_area = Area::new(AreaType::FloatArea, temp_rect).with_traits(traits);
        let float_area_id = area_tree.add_area(float_area);

        area_tree
            .append_child(parent_area, float_area_id)
            .map_err(fop_types::FopError::Generic)?;

        // Layout children of the float
        let children = fo_tree.children(node_id);
        for child_id in children {
            if let Some(child_area_id) = self.layout_block(
                fo_tree,
                child_id,
                area_tree,
                float_area_id,
                float_ctx.current_y,
                float_width,
                resolver,
            )? {
                if let Some(child_area) = area_tree.get(child_area_id) {
                    float_ctx.current_y = child_area.area.geometry.y + child_area.area.height();
                }
            }
        }

        let float_height = if float_ctx.current_y > Length::ZERO {
            float_ctx.current_y
        } else {
            Length::from_pt(50.0) // Fallback height
        };

        // Update the float area's actual height
        if let Some(float_area_node) = area_tree.get_mut(float_area_id) {
            float_area_node.area.geometry.height = float_height;
        }

        // Register the float with the float manager
        let float_info = FloatInfo {
            area_id: float_area_id,
            side: effective_side,
            top: current_y,
            bottom: current_y + float_height,
            width: float_width,
        };
        float_manager.add_float(float_info, is_odd_page);

        Ok(Some(float_area_id))
    }

    /// Measure the width of a float's content.
    ///
    /// Looks at the float's block children for an explicit width property.
    /// Falls back to 1/3 of the container width.
    pub(super) fn measure_float_width(
        &self,
        fo_tree: &FoArena,
        float_node_id: NodeId,
        container_width: Length,
    ) -> Length {
        let children = fo_tree.children(float_node_id);
        for child_id in children {
            if let Some(child_node) = fo_tree.get(child_id) {
                if let Some(props) = child_node.data.properties() {
                    // Check for explicit width on block/graphic children
                    if let Ok(width_val) = props.get(PropertyId::Width) {
                        if let Some(len) = width_val.as_length() {
                            if len > Length::ZERO {
                                return len;
                            }
                        }
                    }
                    // Also check inline-progression-dimension
                    if let Ok(ipd) = props.get(PropertyId::InlineProgressionDimension) {
                        if let Some(len) = ipd.as_length() {
                            if len > Length::ZERO {
                                return len;
                            }
                        }
                    }
                }
                // For external-graphic, check content-width
                if let FoNodeData::ExternalGraphic { properties, .. } = &child_node.data {
                    if let Ok(cw) = properties.get(PropertyId::ContentWidth) {
                        if let Some(len) = cw.as_length() {
                            if len > Length::ZERO {
                                return len;
                            }
                        }
                    }
                    if let Ok(w) = properties.get(PropertyId::Width) {
                        if let Some(len) = w.as_length() {
                            if len > Length::ZERO {
                                return len;
                            }
                        }
                    }
                }
            }
        }
        // Default: 1/3 of container width
        container_width / 3
    }

    /// Layout a block element with float-aware x-offset and reduced width.
    ///
    /// This is like `layout_block` but applies a horizontal offset from the left
    /// (due to left floats) and a reduced available width.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn layout_block_float_aware(
        &self,
        fo_tree: &FoArena,
        node_id: NodeId,
        area_tree: &mut AreaTree,
        parent_area: crate::area::AreaId,
        y_offset: Length,
        available_width: Length,
        x_offset: Length,
        resolver: &mut PageNumberResolver,
    ) -> Result<Option<crate::area::AreaId>> {
        // Layout the block normally first
        let result = self.layout_block(
            fo_tree,
            node_id,
            area_tree,
            parent_area,
            y_offset,
            available_width,
            resolver,
        )?;

        // Apply x_offset to the resulting area (shifting right past left floats)
        if let Some(area_id) = result {
            if x_offset > Length::ZERO {
                if let Some(area_node) = area_tree.get_mut(area_id) {
                    area_node.area.geometry.x += x_offset;
                }
            }
            Ok(Some(area_id))
        } else {
            Ok(None)
        }
    }

    /// Layout static content into an explicitly provided rectangle.
    ///
    /// Used for all five region types: header, footer, start sidebar, end sidebar,
    /// and (if ever needed) body.  The `area_type` parameter controls the
    /// `AreaType` stored on the resulting area node.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn layout_static_content_in_rect(
        &self,
        fo_tree: &FoArena,
        node_id: NodeId,
        area_tree: &mut AreaTree,
        page_area_id: crate::area::AreaId,
        static_rect: Rect,
        area_type: AreaType,
        resolver: &mut PageNumberResolver,
        markers: &PageMarkerView,
    ) -> Result<Option<crate::area::AreaId>> {
        let node = fo_tree
            .get(node_id)
            .ok_or_else(|| fop_types::FopError::Generic(format!("Node {} not found", node_id)))?;

        if let FoNodeData::StaticContent { properties, .. } = &node.data {
            let mut traits = TraitSet::default();
            if let Ok(color) = properties.get(PropertyId::Color) {
                traits.color = color.as_color();
            }
            if let Ok(bg_color) = properties.get(PropertyId::BackgroundColor) {
                traits.background_color = bg_color.as_color();
            }

            let area = Area::new(area_type, static_rect).with_traits(traits);
            let area_id = area_tree.add_area(area);

            area_tree
                .append_child(page_area_id, area_id)
                .map_err(fop_types::FopError::Generic)?;

            // Layout block children with stacking.  Route every child through
            // `layout_static_block_with_markers` so that `fo:retrieve-marker`
            // elements are resolved at any nesting depth (not only as direct
            // children of the static-content node).
            let children = fo_tree.children(node_id);
            let mut block_ctx = BlockLayoutContext::new(static_rect.width);

            for child_id in children {
                if let Some(child_area_id) = self.layout_static_block_with_markers(
                    fo_tree,
                    child_id,
                    area_tree,
                    area_id,
                    block_ctx.current_y,
                    static_rect.width,
                    resolver,
                    markers,
                )? {
                    if let Some(child_area) = area_tree.get(child_area_id) {
                        block_ctx.current_y = child_area.area.geometry.y + child_area.area.height();
                    }
                }
            }

            Ok(Some(area_id))
        } else {
            Ok(None)
        }
    }

    /// Layout a block-level node in a static-content subtree, resolving any
    /// `fo:retrieve-marker` descendants against the page's [`PageMarkerView`].
    ///
    /// This is the marker-aware counterpart of [`LayoutEngine::layout_block`].
    /// It is used exclusively within the static-content layout path so that
    /// `fo:retrieve-marker` elements nested at arbitrary depth inside blocks
    /// (e.g. `<fo:block><fo:retrieve-marker .../></fo:block>`) are resolved
    /// and rendered, not silently dropped.
    ///
    /// Behaviour per node variant:
    ///
    /// * `RetrieveMarker` — resolved against `markers`; its content is laid out
    ///   at `y_offset` via [`Self::layout_marker_content`].  Returns `None`
    ///   (no block area created) when the marker does not resolve.
    /// * `Block` / `BlockContainer` — a block area is created (same geometry as
    ///   [`Self::layout_block`]) and its children are iterated.  `RetrieveMarker`
    ///   children are resolved inline; `Block`/`BlockContainer` children recurse
    ///   into this method; all other inline children (`Text`, `Inline`,
    ///   `BasicLink`, `Leader`, `PageNumber`, `PageNumberCitation`, `Footnote`)
    ///   are handled the same way as in [`Self::layout_block`].
    /// * Everything else — delegated to [`Self::layout_block`] unchanged so
    ///   tables, list-blocks, external-graphics, etc. still work.
    ///
    /// `layout_block` itself is **not** modified, preserving the main-flow
    /// layout path and keeping the existing test suite unaffected.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn layout_static_block_with_markers(
        &self,
        fo_tree: &FoArena,
        node_id: NodeId,
        area_tree: &mut AreaTree,
        parent_area: crate::area::AreaId,
        y_offset: Length,
        available_width: Length,
        resolver: &mut PageNumberResolver,
        markers: &PageMarkerView,
    ) -> Result<Option<crate::area::AreaId>> {
        let node = fo_tree
            .get(node_id)
            .ok_or_else(|| fop_types::FopError::Generic(format!("Node {} not found", node_id)))?;

        match &node.data {
            // ---- Direct retrieve-marker child of static-content --------
            FoNodeData::RetrieveMarker {
                retrieve_class_name,
                retrieve_position,
                properties: retrieve_props,
            } => {
                let boundary = RetrieveBoundaryScope::from_properties(retrieve_props);
                if let Some(marker_node_id) =
                    markers.resolve(retrieve_class_name, *retrieve_position, boundary)
                {
                    self.layout_marker_content(
                        fo_tree,
                        marker_node_id,
                        area_tree,
                        parent_area,
                        y_offset,
                        available_width,
                        resolver,
                    )?;
                }
                // RetrieveMarker does not produce a block area of its own.
                Ok(None)
            }

            // ---- fo:block and fo:block-container -----------------------
            FoNodeData::Block { properties } | FoNodeData::BlockContainer { properties } => {
                // Geometry / spacing — mirrors layout_block exactly.
                let traits = extract_traits(properties);
                let space_before = extract_space_before(properties);
                let space_after = extract_space_after(properties);
                let start_indent = extract_start_indent(properties);
                let end_indent = extract_end_indent(properties);
                let text_indent = extract_text_indent(properties);
                let keep_constraint = extract_keep_constraint(properties);

                let line_height = traits
                    .line_height
                    .or(traits.font_size)
                    .unwrap_or(Length::from_pt(12.0));

                let content_width = available_width - start_indent - end_indent;

                let mut block_ctx = BlockLayoutContext::new(content_width);
                block_ctx.current_y = y_offset;
                let mut block_rect = block_ctx.allocate_with_spacing(
                    content_width,
                    line_height,
                    space_before,
                    space_after,
                );
                block_rect.x = start_indent;

                let mut area = Area::new(AreaType::Block, block_rect).with_traits(traits.clone());

                if keep_constraint.has_constraint() {
                    area = area.with_keep_constraint(keep_constraint);
                }

                let area_id = area_tree.add_area(area);
                area_tree
                    .append_child(parent_area, area_id)
                    .map_err(fop_types::FopError::Generic)?;

                let text_align = traits.text_align.unwrap_or(TextAlign::Left);

                // Iterate children, special-casing RetrieveMarker and nested blocks.
                let children = fo_tree.children(node_id);
                let mut is_first_line = true;
                let mut content_y = Length::ZERO;

                for child_id in children {
                    if let Some(child_node) = fo_tree.get(child_id) {
                        match &child_node.data {
                            // ---- Resolved retrieve-marker ---------------
                            FoNodeData::RetrieveMarker {
                                retrieve_class_name,
                                retrieve_position,
                                properties: retrieve_props,
                            } => {
                                let boundary =
                                    RetrieveBoundaryScope::from_properties(retrieve_props);
                                if let Some(marker_node_id) = markers.resolve(
                                    retrieve_class_name,
                                    *retrieve_position,
                                    boundary,
                                ) {
                                    self.layout_marker_content(
                                        fo_tree,
                                        marker_node_id,
                                        area_tree,
                                        area_id,
                                        content_y,
                                        content_width,
                                        resolver,
                                    )?;
                                    // Advance past the marker's content (one
                                    // line height — marker children are blocks
                                    // so they each have their own geometry, but
                                    // we advance the inline cursor conservatively
                                    // to avoid overlapping the next sibling).
                                    content_y += line_height;
                                }
                                is_first_line = false;
                            }

                            // ---- Nested block/block-container ———————————
                            FoNodeData::Block { .. } | FoNodeData::BlockContainer { .. } => {
                                // Recurse with the same marker context.
                                if let Some(child_area_id) = self.layout_static_block_with_markers(
                                    fo_tree,
                                    child_id,
                                    area_tree,
                                    area_id,
                                    content_y,
                                    content_width,
                                    resolver,
                                    markers,
                                )? {
                                    if let Some(child_area) = area_tree.get(child_area_id) {
                                        content_y =
                                            child_area.area.geometry.y + child_area.area.height();
                                    }
                                }
                                is_first_line = false;
                            }

                            // ---- Inline text run ————————————————————————
                            FoNodeData::Text(text) => {
                                let mut text_traits = traits.clone();
                                text_traits.text_align = Some(text_align);
                                let first_indent = if is_first_line {
                                    text_indent
                                } else {
                                    Length::ZERO
                                };
                                let new_y = self.emit_text_lines(
                                    area_tree,
                                    area_id,
                                    text,
                                    &text_traits,
                                    text_align,
                                    content_width,
                                    first_indent,
                                    content_y,
                                    line_height,
                                )?;
                                if new_y != content_y {
                                    is_first_line = false;
                                    content_y = new_y;
                                }
                            }

                            // ---- fo:basic-link —————————————————————————
                            FoNodeData::BasicLink {
                                external_destination,
                                internal_destination,
                                properties: link_props,
                            } => {
                                let mut link_traits = extract_traits(link_props);
                                link_traits.text_align = Some(text_align);
                                link_traits.link_destination = external_destination
                                    .clone()
                                    .or_else(|| internal_destination.clone());

                                let link_children = fo_tree.children(child_id);
                                let had_children = !link_children.is_empty();
                                for link_child_id in link_children {
                                    self.layout_inline(
                                        fo_tree,
                                        link_child_id,
                                        area_tree,
                                        area_id,
                                        content_width,
                                        line_height,
                                        &link_traits,
                                        if is_first_line {
                                            text_indent
                                        } else {
                                            Length::ZERO
                                        },
                                        content_y,
                                    )?;
                                    is_first_line = false;
                                }
                                if had_children {
                                    content_y += line_height;
                                }
                            }

                            // ---- fo:inline —————————————————————————————
                            FoNodeData::Inline {
                                properties: inline_props,
                            } => {
                                let inline_traits = extract_traits(inline_props);
                                let inline_children = fo_tree.children(child_id);
                                let had_children = !inline_children.is_empty();
                                for inline_child_id in inline_children {
                                    self.layout_inline(
                                        fo_tree,
                                        inline_child_id,
                                        area_tree,
                                        area_id,
                                        content_width,
                                        line_height,
                                        &inline_traits,
                                        if is_first_line {
                                            text_indent
                                        } else {
                                            Length::ZERO
                                        },
                                        content_y,
                                    )?;
                                    is_first_line = false;
                                }
                                if had_children {
                                    content_y += line_height;
                                }
                            }

                            // ---- fo:leader —————————————————————————————
                            FoNodeData::Leader {
                                properties: leader_props,
                            } => {
                                self.layout_leader(
                                    fo_tree,
                                    child_id,
                                    leader_props,
                                    area_tree,
                                    area_id,
                                    content_width,
                                    line_height,
                                    &traits,
                                    content_y,
                                )?;
                                is_first_line = false;
                                content_y += line_height;
                            }

                            // ---- fo:page-number-citation ————————————————
                            FoNodeData::PageNumberCitation {
                                ref_id,
                                properties: citation_props,
                            } => {
                                let mut citation_traits = extract_traits(citation_props);
                                citation_traits.text_align = Some(text_align);
                                let x_offset = if is_first_line {
                                    text_indent
                                } else {
                                    Length::ZERO
                                };
                                let line_width = if is_first_line {
                                    content_width - text_indent
                                } else {
                                    content_width
                                };
                                let text_rect =
                                    Rect::new(x_offset, content_y, line_width, line_height);
                                let citation_area = Area::text(text_rect, "0".to_string())
                                    .with_traits(citation_traits);
                                let citation_id = area_tree.add_area(citation_area);
                                resolver.register_citation(citation_id, ref_id.clone());
                                area_tree
                                    .append_child(area_id, citation_id)
                                    .map_err(fop_types::FopError::Generic)?;
                                is_first_line = false;
                                content_y += line_height;
                            }

                            // ---- fo:footnote ————————————————————————————
                            FoNodeData::Footnote { .. } => {
                                self.layout_footnote(
                                    fo_tree,
                                    child_id,
                                    area_tree,
                                    area_id,
                                    parent_area,
                                    content_width,
                                    line_height,
                                    &traits,
                                    if is_first_line {
                                        text_indent
                                    } else {
                                        Length::ZERO
                                    },
                                    content_y,
                                    resolver,
                                )?;
                                is_first_line = false;
                                content_y += line_height;
                            }

                            // ---- fo:page-number ————————————————————————
                            FoNodeData::PageNumber {
                                properties: page_num_props,
                            } => {
                                let mut pn_traits = extract_traits(page_num_props);
                                pn_traits.text_align = Some(text_align);
                                let x_offset = if is_first_line {
                                    text_indent
                                } else {
                                    Length::ZERO
                                };
                                let line_width = if is_first_line {
                                    content_width - text_indent
                                } else {
                                    content_width
                                };
                                let page_num_str = resolver.current_page().to_string();
                                let pn_rect =
                                    Rect::new(x_offset, content_y, line_width, line_height);
                                let pn_area =
                                    Area::text(pn_rect, page_num_str).with_traits(pn_traits);
                                let pn_id = area_tree.add_area(pn_area);
                                area_tree
                                    .append_child(area_id, pn_id)
                                    .map_err(fop_types::FopError::Generic)?;
                                is_first_line = false;
                                content_y += line_height;
                            }

                            // ---- Everything else ————————————————————————
                            _ => {}
                        }
                    }
                }

                let content_height = content_y.max(line_height);
                if let Some(block_node) = area_tree.get_mut(area_id) {
                    block_node.area.geometry.height = content_height;
                }

                if let Some(id) = &node.id {
                    resolver.register_element(id.clone(), area_id);
                }

                Ok(Some(area_id))
            }

            // ---- Anything else: delegate to normal layout_block --------
            _ => self.layout_block(
                fo_tree,
                node_id,
                area_tree,
                parent_area,
                y_offset,
                available_width,
                resolver,
            ),
        }
    }

    /// Layout the content of a marker (its children)
    #[allow(clippy::too_many_arguments)]
    pub(super) fn layout_marker_content(
        &self,
        fo_tree: &FoArena,
        marker_node_id: NodeId,
        area_tree: &mut AreaTree,
        parent_area: crate::area::AreaId,
        y_offset: Length,
        available_width: Length,
        resolver: &mut PageNumberResolver,
    ) -> Result<()> {
        // Get the marker node and layout its children
        let children = fo_tree.children(marker_node_id);
        for child_id in children {
            self.layout_block(
                fo_tree,
                child_id,
                area_tree,
                parent_area,
                y_offset,
                available_width,
                resolver,
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::area::{AreaContent, AreaType};
    use fop_core::tree::RetrievePosition;
    use fop_core::{FoNode, FoNodeData, PropertyList};

    fn make_engine() -> LayoutEngine {
        LayoutEngine::new()
    }

    // -----------------------------------------------------------------------
    // Helper: walk every area in the tree looking for Text areas whose content
    // contains the given needle string.
    // -----------------------------------------------------------------------
    fn has_text_in_area(area_tree: &crate::area::AreaTree, needle: &str) -> bool {
        for (_, node) in area_tree.iter() {
            if let Some(AreaContent::Text(text)) = &node.area.content {
                if text.contains(needle) {
                    return true;
                }
            }
        }
        false
    }

    /// Collect all areas that are descendants of any `Header` area, returning
    /// their text content strings.
    fn header_texts(area_tree: &crate::area::AreaTree) -> Vec<String> {
        // First, find all header area ids.
        let mut header_ids: Vec<crate::area::AreaId> = Vec::new();
        for (id, node) in area_tree.iter() {
            if matches!(node.area.area_type, AreaType::Header) {
                header_ids.push(id);
            }
        }

        // Collect all descendant text areas of any header.
        let mut texts = Vec::new();
        for header_id in &header_ids {
            collect_descendant_texts(area_tree, *header_id, &mut texts);
        }
        texts
    }

    fn collect_descendant_texts(
        area_tree: &crate::area::AreaTree,
        area_id: crate::area::AreaId,
        out: &mut Vec<String>,
    ) {
        if let Some(node) = area_tree.get(area_id) {
            if let Some(AreaContent::Text(t)) = &node.area.content {
                out.push(t.clone());
            }
            // Recurse into children.
            let children = area_tree.children(area_id);
            for child_id in children {
                collect_descendant_texts(area_tree, child_id, out);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Test 1 (regression): direct child retrieve-marker still resolves.
    //
    // Structure:
    //   PageSequence
    //   ├── StaticContent("xsl-region-before")
    //   │   └── RetrieveMarker(class="chap", position=FirstStartingWithinPage)
    //   └── Flow("xsl-region-body")
    //       └── Block
    //           ├── Marker(class="chap")
    //           │   └── Block → Text("Direct Title")
    //           └── Text("body")
    // -----------------------------------------------------------------------
    #[test]
    fn retrieve_marker_direct_child_of_static_content_resolves() {
        let mut fo_tree = FoArena::new();
        let root = fo_tree.add_node(FoNode::new(FoNodeData::Root));

        let page_seq = fo_tree.add_node(FoNode::new(FoNodeData::PageSequence {
            master_reference: "A4".to_string(),
            format: "1".to_string(),
            grouping_separator: None,
            grouping_size: None,
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(root, page_seq)
            .expect("test: should succeed");

        // --- static-content with a DIRECT child retrieve-marker ---
        let header = fo_tree.add_node(FoNode::new(FoNodeData::StaticContent {
            flow_name: "xsl-region-before".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, header)
            .expect("test: should succeed");

        let retrieve = fo_tree.add_node(FoNode::new(FoNodeData::RetrieveMarker {
            retrieve_class_name: "chap".to_string(),
            retrieve_position: RetrievePosition::FirstStartingWithinPage,
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(header, retrieve)
            .expect("test: should succeed");

        // --- flow ---
        let flow = fo_tree.add_node(FoNode::new(FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, flow)
            .expect("test: should succeed");

        let body_block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(flow, body_block)
            .expect("test: should succeed");

        // Marker inside the flow block
        let marker = fo_tree.add_node(FoNode::new(FoNodeData::Marker {
            marker_class_name: "chap".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(body_block, marker)
            .expect("test: should succeed");

        // Marker content: a block with text
        let marker_block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(marker, marker_block)
            .expect("test: should succeed");
        let marker_text =
            fo_tree.add_node(FoNode::new(FoNodeData::Text("Direct Title".to_string())));
        fo_tree
            .append_child(marker_block, marker_text)
            .expect("test: should succeed");

        // Body text
        let body_text = fo_tree.add_node(FoNode::new(FoNodeData::Text("body".to_string())));
        fo_tree
            .append_child(body_block, body_text)
            .expect("test: should succeed");

        let engine = make_engine();
        let area_tree = engine
            .layout(&fo_tree)
            .expect("test: layout should succeed");

        // The header's descendant areas must contain the marker text.
        // `emit_text_lines` may break text into per-word areas, so join them.
        let header_text_list = header_texts(&area_tree);
        let joined = header_text_list.join(" ");
        assert!(
            joined.contains("Direct") && joined.contains("Title"),
            "direct-child retrieve-marker must resolve: expected 'Direct Title' words in header \
             areas, got: {header_text_list:?}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 2 (new feature): retrieve-marker nested inside a block within
    // static-content resolves.
    //
    // Structure:
    //   PageSequence
    //   ├── StaticContent("xsl-region-before")
    //   │   └── Block            ← outer block (NOT a direct retrieve-marker)
    //   │       └── RetrieveMarker(class="nested-chap", FirstStartingWithinPage)
    //   └── Flow("xsl-region-body")
    //       └── Block
    //           ├── Marker(class="nested-chap")
    //           │   └── Block → Text("Nested Chapter Title")
    //           └── Text("body text")
    // -----------------------------------------------------------------------
    #[test]
    fn retrieve_marker_nested_in_block_within_static_content_resolves() {
        let mut fo_tree = FoArena::new();
        let root = fo_tree.add_node(FoNode::new(FoNodeData::Root));

        let page_seq = fo_tree.add_node(FoNode::new(FoNodeData::PageSequence {
            master_reference: "A4".to_string(),
            format: "1".to_string(),
            grouping_separator: None,
            grouping_size: None,
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(root, page_seq)
            .expect("test: should succeed");

        // --- static-content: the retrieve-marker is INSIDE a block ---
        let header = fo_tree.add_node(FoNode::new(FoNodeData::StaticContent {
            flow_name: "xsl-region-before".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, header)
            .expect("test: should succeed");

        // Outer block that wraps the retrieve-marker
        let outer_block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(header, outer_block)
            .expect("test: should succeed");

        // The retrieve-marker is a child of the outer block (not direct child of static-content)
        let retrieve = fo_tree.add_node(FoNode::new(FoNodeData::RetrieveMarker {
            retrieve_class_name: "nested-chap".to_string(),
            retrieve_position: RetrievePosition::FirstStartingWithinPage,
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(outer_block, retrieve)
            .expect("test: should succeed");

        // --- flow ---
        let flow = fo_tree.add_node(FoNode::new(FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, flow)
            .expect("test: should succeed");

        let body_block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(flow, body_block)
            .expect("test: should succeed");

        // Marker inside the flow block
        let marker = fo_tree.add_node(FoNode::new(FoNodeData::Marker {
            marker_class_name: "nested-chap".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(body_block, marker)
            .expect("test: should succeed");

        // Marker content: a block with text — this is what should appear in the header
        let marker_block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(marker, marker_block)
            .expect("test: should succeed");
        let marker_text = fo_tree.add_node(FoNode::new(FoNodeData::Text(
            "Nested Chapter Title".to_string(),
        )));
        fo_tree
            .append_child(marker_block, marker_text)
            .expect("test: should succeed");

        // Body text
        let body_text = fo_tree.add_node(FoNode::new(FoNodeData::Text("body text".to_string())));
        fo_tree
            .append_child(body_block, body_text)
            .expect("test: should succeed");

        let engine = make_engine();
        let area_tree = engine
            .layout(&fo_tree)
            .expect("test: layout should succeed");

        // The header must contain the marker text from the NESTED retrieve-marker.
        // `emit_text_lines` may break text into per-word areas, so join them.
        let header_text_list = header_texts(&area_tree);
        let joined = header_text_list.join(" ");
        assert!(
            joined.contains("Nested") && joined.contains("Chapter") && joined.contains("Title"),
            "nested retrieve-marker must resolve: expected 'Nested Chapter Title' words in \
             header areas, got: {header_text_list:?}"
        );

        // The header should also have a Block area (outer_block).
        let mut has_header = false;
        for (_, node) in area_tree.iter() {
            if matches!(node.area.area_type, AreaType::Header) {
                has_header = true;
                break;
            }
        }
        assert!(has_header, "layout must produce a Header area");
    }

    // -----------------------------------------------------------------------
    // Test 3: retrieve-marker that matches NO marker on the page contributes
    // nothing — header still exists, but no spurious text appears.
    // -----------------------------------------------------------------------
    #[test]
    fn retrieve_marker_nested_with_no_matching_marker_contributes_nothing() {
        let mut fo_tree = FoArena::new();
        let root = fo_tree.add_node(FoNode::new(FoNodeData::Root));

        let page_seq = fo_tree.add_node(FoNode::new(FoNodeData::PageSequence {
            master_reference: "A4".to_string(),
            format: "1".to_string(),
            grouping_separator: None,
            grouping_size: None,
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(root, page_seq)
            .expect("test: should succeed");

        let header = fo_tree.add_node(FoNode::new(FoNodeData::StaticContent {
            flow_name: "xsl-region-before".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, header)
            .expect("test: should succeed");

        let outer_block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(header, outer_block)
            .expect("test: should succeed");

        // retrieve-marker for a class that has NO matching fo:marker in the flow
        let retrieve = fo_tree.add_node(FoNode::new(FoNodeData::RetrieveMarker {
            retrieve_class_name: "nonexistent-class".to_string(),
            retrieve_position: RetrievePosition::FirstStartingWithinPage,
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(outer_block, retrieve)
            .expect("test: should succeed");

        let flow = fo_tree.add_node(FoNode::new(FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, flow)
            .expect("test: should succeed");

        let body_block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(flow, body_block)
            .expect("test: should succeed");
        let body_text = fo_tree.add_node(FoNode::new(FoNodeData::Text("body content".to_string())));
        fo_tree
            .append_child(body_block, body_text)
            .expect("test: should succeed");

        let engine = make_engine();
        let area_tree = engine
            .layout(&fo_tree)
            .expect("test: layout should succeed");

        // The header should exist but contain no text of its own (no marker resolved)
        let header_text_list = header_texts(&area_tree);
        assert!(
            header_text_list.is_empty(),
            "unresolved nested retrieve-marker must contribute no text; \
             got: {header_text_list:?}"
        );

        // The body text must not bleed into the header area check
        assert!(
            !has_text_in_area(&area_tree, "nonexistent-class"),
            "retrieve-class-name string must not appear in any area"
        );
    }
}
