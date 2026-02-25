//! Page, flow, float, static-content, and marker layout methods.
//!
//! Handles page region geometry extraction, flow layout with float support,
//! static-content (header/footer/sidebars), and marker collection/retrieval.

use crate::area::{Area, AreaTree, AreaType, TraitSet};
use crate::layout::{
    extract_clear, extract_column_count, extract_column_gap, extract_traits, BlockLayoutContext,
    PageNumberResolver,
};
use fop_core::{FoArena, FoNodeData, NodeId, PropertyId};
use fop_types::{Length, Point, Rect, Result, Size};

use super::types::{
    FloatInfo, FloatManager, FloatSide, MarkerMap, MultiColumnLayout, PageRegionGeometry,
};
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

    /// Layout a flow node using an explicit body rectangle.
    ///
    /// Called from `PageSequence` processing where the body rect is already
    /// computed from the page master geometry.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn layout_flow_in_rect(
        &self,
        fo_tree: &FoArena,
        node_id: NodeId,
        area_tree: &mut AreaTree,
        page_area_id: crate::area::AreaId,
        flow_rect: Rect,
        resolver: &mut PageNumberResolver,
        marker_map: &mut MarkerMap,
    ) -> Result<Option<crate::area::AreaId>> {
        let node = fo_tree
            .get(node_id)
            .ok_or_else(|| fop_types::FopError::Generic(format!("Node {} not found", node_id)))?;

        if let FoNodeData::Flow { properties, .. } = &node.data {
            let mut traits = TraitSet::default();
            if let Ok(color) = properties.get(PropertyId::Color) {
                traits.color = color.as_color();
            }

            let area = Area::new(AreaType::Region, flow_rect).with_traits(traits);
            let area_id = area_tree.add_area(area);

            area_tree
                .append_child(page_area_id, area_id)
                .map_err(fop_types::FopError::Generic)?;

            // Check for multi-column layout
            let column_count = extract_column_count(properties);
            let column_gap = extract_column_gap(properties);

            let children = fo_tree.children(node_id);

            if column_count > 1 {
                let mut multi_col =
                    MultiColumnLayout::new(column_count, column_gap, flow_rect.width)
                        .with_max_height(flow_rect.height);

                for child_id in children {
                    self.layout_block_multicolumn(
                        fo_tree,
                        child_id,
                        area_tree,
                        area_id,
                        &mut multi_col,
                        resolver,
                    )?;
                }
            } else {
                // Single column layout with float support
                let mut block_ctx = BlockLayoutContext::new(flow_rect.width);
                let mut float_manager = FloatManager::new();
                let is_odd_page = resolver.current_page() % 2 == 1;

                for child_id in children {
                    // Remove floats that have ended before the current Y position
                    float_manager.remove_floats_above(block_ctx.current_y);

                    // Check if this child is a float element
                    let child_is_float = fo_tree
                        .get(child_id)
                        .map(|n| matches!(n.data, FoNodeData::Float { .. }))
                        .unwrap_or(false);

                    if child_is_float {
                        // Layout the float and register it with the float manager
                        self.layout_float_in_flow(
                            fo_tree,
                            child_id,
                            area_tree,
                            area_id,
                            block_ctx.current_y,
                            flow_rect.width,
                            is_odd_page,
                            &mut float_manager,
                            resolver,
                        )?;
                    } else {
                        // Apply `clear` property: advance past active floats if requested
                        if let Some(child_node) = fo_tree.get(child_id) {
                            if let Some(props) = child_node.data.properties() {
                                let clear = extract_clear(props);
                                block_ctx.current_y =
                                    float_manager.get_clear_position(clear, block_ctx.current_y);
                            }
                        }

                        // Compute available width and x-offset considering active floats
                        let (left_offset, avail_width) =
                            float_manager.available_width(block_ctx.current_y, flow_rect.width);

                        if let Some(child_area_id) = self.layout_block_float_aware(
                            fo_tree,
                            child_id,
                            area_tree,
                            area_id,
                            block_ctx.current_y,
                            avail_width,
                            left_offset,
                            resolver,
                        )? {
                            if let Some(child_area) = area_tree.get(child_area_id) {
                                block_ctx.current_y =
                                    child_area.area.geometry.y + child_area.area.height();
                            }
                        }
                    }
                }

                // Clear floats for cleanup
                float_manager.clear();
            }

            Ok(Some(area_id))
        } else {
            // Fallback: route through normal layout_node
            self.layout_node(
                fo_tree,
                node_id,
                area_tree,
                Some(page_area_id),
                resolver,
                marker_map,
            )
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
        marker_map: &MarkerMap,
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

            // Layout block children with stacking
            let children = fo_tree.children(node_id);
            let mut block_ctx = BlockLayoutContext::new(static_rect.width);

            for child_id in children {
                // Check if this is a retrieve-marker and handle it specially
                if let Some(child_node) = fo_tree.get(child_id) {
                    if let FoNodeData::RetrieveMarker {
                        retrieve_class_name,
                        retrieve_position,
                        properties: _,
                    } = &child_node.data
                    {
                        if let Some(marker_node_id) =
                            marker_map.retrieve_marker(retrieve_class_name, *retrieve_position)
                        {
                            self.layout_marker_content(
                                fo_tree,
                                marker_node_id,
                                area_tree,
                                area_id,
                                block_ctx.current_y,
                                static_rect.width,
                                resolver,
                            )?;
                        }
                        continue;
                    }
                }

                if let Some(child_area_id) = self.layout_block(
                    fo_tree,
                    child_id,
                    area_tree,
                    area_id,
                    block_ctx.current_y,
                    static_rect.width,
                    resolver,
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

    /// Layout static content (headers and footers) - legacy fallback with hardcoded margins.
    ///
    /// Superseded by `layout_static_content_in_rect` which uses geometry from
    /// the page master.  Kept as a fallback for callers that do not have page
    /// geometry available.
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn layout_static_content(
        &self,
        fo_tree: &FoArena,
        node_id: NodeId,
        area_tree: &mut AreaTree,
        page_area_id: crate::area::AreaId,
        is_header: bool,
        resolver: &mut PageNumberResolver,
        marker_map: &MarkerMap,
    ) -> Result<Option<crate::area::AreaId>> {
        let node = fo_tree
            .get(node_id)
            .ok_or_else(|| fop_types::FopError::Generic(format!("Node {} not found", node_id)))?;

        if let FoNodeData::StaticContent { properties, .. } = &node.data {
            // Define header/footer dimensions
            let header_height = Length::from_pt(72.0); // 1 inch
            let footer_height = Length::from_pt(72.0); // 1 inch
            let margin_x = Length::from_pt(72.0); // 1 inch side margins

            let area_type = if is_header {
                AreaType::Header
            } else {
                AreaType::Footer
            };

            let y_position = if is_header {
                Length::ZERO
            } else {
                self.page_height - footer_height
            };

            let static_rect = Rect::from_point_size(
                Point::new(margin_x, y_position),
                Size::new(
                    self.page_width - Length::from_pt(144.0), // 2 inches for margins
                    if is_header {
                        header_height
                    } else {
                        footer_height
                    },
                ),
            );

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

            // Layout block children with stacking
            let children = fo_tree.children(node_id);
            let mut block_ctx = BlockLayoutContext::new(static_rect.width);

            for child_id in children {
                // Check if this is a retrieve-marker and handle it specially
                if let Some(child_node) = fo_tree.get(child_id) {
                    if let FoNodeData::RetrieveMarker {
                        retrieve_class_name,
                        retrieve_position,
                        properties: _,
                    } = &child_node.data
                    {
                        // Retrieve the marker content
                        if let Some(marker_node_id) =
                            marker_map.retrieve_marker(retrieve_class_name, *retrieve_position)
                        {
                            // Layout the marker's children as if they were direct children
                            self.layout_marker_content(
                                fo_tree,
                                marker_node_id,
                                area_tree,
                                area_id,
                                block_ctx.current_y,
                                static_rect.width,
                                resolver,
                            )?;
                        }
                        continue;
                    }
                }

                if let Some(child_area_id) = self.layout_block(
                    fo_tree,
                    child_id,
                    area_tree,
                    area_id,
                    block_ctx.current_y,
                    static_rect.width,
                    resolver,
                )? {
                    if let Some(child_area) = area_tree.get(child_area_id) {
                        // Update context with actual height used
                        block_ctx.current_y = child_area.area.geometry.y + child_area.area.height();
                    }
                }
            }

            Ok(Some(area_id))
        } else {
            Ok(None)
        }
    }

    /// Collect markers from the flow content
    #[allow(clippy::only_used_in_recursion)]
    pub(super) fn collect_markers(
        &self,
        fo_tree: &FoArena,
        node_id: NodeId,
        marker_map: &mut MarkerMap,
    ) {
        // Traverse the tree and collect all markers
        if let Some(node) = fo_tree.get(node_id) {
            if let FoNodeData::Marker {
                marker_class_name, ..
            } = &node.data
            {
                // For now, assume all markers start and end on the page
                // In a full implementation, this would track page breaks
                marker_map.add_marker(marker_class_name.clone(), node_id, true, true);
            }

            // Recursively collect from children
            let children = fo_tree.children(node_id);
            for child_id in children {
                self.collect_markers(fo_tree, child_id, marker_map);
            }
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
