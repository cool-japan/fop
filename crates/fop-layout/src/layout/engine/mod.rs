//! Main layout engine
//!
//! Coordinates the layout process from FO tree to area tree.

mod block_layout;
mod inline_layout;
mod page_layout;
mod page_master;
mod table_layout;
pub(super) mod types;

use crate::area::{Area, AreaContent, AreaTree, AreaType, TraitSet};
use crate::layout::{
    extract_clear, extract_column_count, extract_column_gap, extract_space_after, extract_traits,
    BlockLayoutContext, BorderCollapse, ColumnInfo, ColumnWidth, ListLayout, PageNumberResolver,
    TableLayout, TableLayoutMode,
};
use fop_core::{FoArena, FoNodeData, NodeId, PropertyId};
use fop_types::{FontRegistry, Length, Point, Rect, Result, Size};

use types::{FloatManager, MarkerMap};

pub use types::{ClearSide, FloatSide, MultiColumnLayout};

/// Layout engine - transforms FO tree to area tree
pub struct LayoutEngine {
    /// Default page width (A4)
    pub(super) page_width: Length,

    /// Default page height (A4)
    pub(super) page_height: Length,

    /// Font registry for text measurement
    #[allow(dead_code)]
    pub(super) font_registry: FontRegistry,

    /// Enable streaming mode for large documents
    /// When true, prefer using layout_streaming() for memory-efficient processing
    pub(super) streaming_mode: bool,
}

impl LayoutEngine {
    /// Create a new layout engine
    pub fn new() -> Self {
        Self {
            page_width: Length::from_mm(210.0),  // A4 width
            page_height: Length::from_mm(297.0), // A4 height
            font_registry: FontRegistry::new(),
            streaming_mode: false,
        }
    }

    /// Enable streaming mode for large documents
    ///
    /// When streaming mode is enabled, the engine is optimized for processing
    /// large documents (1000+ pages) with bounded memory usage.
    pub fn with_streaming_mode(mut self, enabled: bool) -> Self {
        self.streaming_mode = enabled;
        self
    }

    /// Check if streaming mode is enabled
    pub fn is_streaming_mode(&self) -> bool {
        self.streaming_mode
    }

    /// Perform layout on an FO tree, producing an area tree
    pub fn layout(&self, fo_tree: &FoArena) -> Result<AreaTree> {
        let mut area_tree = AreaTree::new();
        let mut resolver = PageNumberResolver::new();
        let mut marker_map = MarkerMap::new();

        // First pass: Layout everything and track IDs
        if let Some((root_id, _)) = fo_tree.root() {
            self.layout_node(
                fo_tree,
                root_id,
                &mut area_tree,
                None,
                &mut resolver,
                &mut marker_map,
            )?;
        }

        // Second pass: Resolve page number citations
        self.resolve_citations(&mut area_tree, &resolver)?;

        Ok(area_tree)
    }

    /// Resolve page number citations in the area tree
    fn resolve_citations(
        &self,
        area_tree: &mut AreaTree,
        resolver: &PageNumberResolver,
    ) -> Result<()> {
        for (area_id, ref_id) in resolver.get_citations() {
            if let Some(page_number) = resolver.get_page_number(ref_id) {
                // Update the area content with the resolved page number
                if let Some(area_node) = area_tree.get_mut(*area_id) {
                    area_node.area.content = Some(AreaContent::Text(page_number.to_string()));
                }
            } else {
                // If the reference cannot be resolved, use "??" as a placeholder
                if let Some(area_node) = area_tree.get_mut(*area_id) {
                    area_node.area.content = Some(AreaContent::Text("??".to_string()));
                }
            }
        }
        Ok(())
    }

    /// Layout a single FO node and its children
    pub(in crate::layout::engine) fn layout_node(
        &self,
        fo_tree: &FoArena,
        node_id: NodeId,
        area_tree: &mut AreaTree,
        parent_area: Option<crate::area::AreaId>,
        resolver: &mut PageNumberResolver,
        marker_map: &mut MarkerMap,
    ) -> Result<Option<crate::area::AreaId>> {
        let node = fo_tree
            .get(node_id)
            .ok_or_else(|| fop_types::FopError::Generic(format!("Node {} not found", node_id)))?;

        let area_id = match &node.data {
            FoNodeData::Root => {
                // Root doesn't produce an area, just process children
                let children = fo_tree.children(node_id);
                for child_id in children {
                    self.layout_node(
                        fo_tree,
                        child_id,
                        area_tree,
                        parent_area,
                        resolver,
                        marker_map,
                    )?;
                }
                None
            }

            FoNodeData::LayoutMasterSet => {
                // Layout master set doesn't produce areas
                None
            }

            FoNodeData::PageSequence {
                properties,
                master_reference,
                ..
            } => {
                // Determine page geometry from the simple-page-master
                let master_ref = master_reference.clone();
                let geom = self.extract_page_region_geometry(fo_tree, &master_ref);

                // Create a page area (each page-sequence creates one page)
                let page_rect = Rect::from_point_size(
                    Point::ZERO,
                    Size::new(geom.page_width, geom.page_height),
                );

                let mut traits = TraitSet::default();
                if let Ok(color) = properties.get(PropertyId::BackgroundColor) {
                    traits.background_color = color.as_color();
                }

                let area = Area::new(AreaType::Page, page_rect).with_traits(traits);
                let area_id = area_tree.add_area(area);

                // Register ID if present
                if let Some(id) = &node.id {
                    resolver.register_element(id.clone(), area_id);
                }

                // Process children (flow, static-content)
                // First pass: collect static-content for all regions
                let children = fo_tree.children(node_id);
                let mut static_before_id = None;
                let mut static_after_id = None;
                let mut static_start_id = None;
                let mut static_end_id = None;
                let mut flow_id = None;

                for child_id in children.clone() {
                    if let Some(child) = fo_tree.get(child_id) {
                        match &child.data {
                            FoNodeData::StaticContent { flow_name, .. } => {
                                match flow_name.as_str() {
                                    "xsl-region-before" => static_before_id = Some(child_id),
                                    "xsl-region-after" => static_after_id = Some(child_id),
                                    "xsl-region-start" => static_start_id = Some(child_id),
                                    "xsl-region-end" => static_end_id = Some(child_id),
                                    _ => {}
                                }
                            }
                            FoNodeData::Flow { .. } => {
                                flow_id = Some(child_id);
                            }
                            _ => {}
                        }
                    }
                }

                // Clear markers for the new page
                marker_map.clear();

                // First pass: Collect markers from the main flow
                if let Some(flow_node_id) = flow_id {
                    self.collect_markers(fo_tree, flow_node_id, marker_map);
                }

                // Layout header (static-content for region-before)
                if let Some(header_id) = static_before_id {
                    self.layout_static_content_in_rect(
                        fo_tree,
                        header_id,
                        area_tree,
                        area_id,
                        geom.before_rect,
                        AreaType::Header,
                        resolver,
                        marker_map,
                    )?;
                }

                // Layout footer (static-content for region-after)
                if let Some(footer_id) = static_after_id {
                    self.layout_static_content_in_rect(
                        fo_tree,
                        footer_id,
                        area_tree,
                        area_id,
                        geom.after_rect,
                        AreaType::Footer,
                        resolver,
                        marker_map,
                    )?;
                }

                // Layout start sidebar (static-content for region-start)
                if let Some(start_id) = static_start_id {
                    self.layout_static_content_in_rect(
                        fo_tree,
                        start_id,
                        area_tree,
                        area_id,
                        geom.start_rect,
                        AreaType::SidebarStart,
                        resolver,
                        marker_map,
                    )?;
                }

                // Layout end sidebar (static-content for region-end)
                if let Some(end_id) = static_end_id {
                    self.layout_static_content_in_rect(
                        fo_tree,
                        end_id,
                        area_tree,
                        area_id,
                        geom.end_rect,
                        AreaType::SidebarEnd,
                        resolver,
                        marker_map,
                    )?;
                }

                // Layout main content (flow) using the computed body rect
                if let Some(flow_node_id) = flow_id {
                    self.layout_flow_in_rect(
                        fo_tree,
                        flow_node_id,
                        area_tree,
                        area_id,
                        geom.body_rect,
                        resolver,
                        marker_map,
                    )?;
                }

                // Place collected footnotes at the bottom of the page body region
                self.place_footnotes_for_page(area_tree, area_id, geom.body_rect)?;

                // Increment page counter after processing this page
                resolver.set_current_page(resolver.current_page() + 1);

                Some(area_id)
            }

            FoNodeData::Flow { properties, .. } => {
                // Flow creates a region area
                let flow_rect = Rect::from_point_size(
                    Point::new(Length::from_pt(72.0), Length::from_pt(72.0)), // 1 inch margins
                    Size::new(
                        self.page_width - Length::from_pt(144.0),
                        self.page_height - Length::from_pt(144.0),
                    ),
                );

                let mut traits = TraitSet::default();
                if let Ok(color) = properties.get(PropertyId::Color) {
                    traits.color = color.as_color();
                }

                let area = Area::new(AreaType::Region, flow_rect).with_traits(traits);
                let area_id = area_tree.add_area(area);

                if let Some(parent) = parent_area {
                    area_tree
                        .append_child(parent, area_id)
                        .map_err(fop_types::FopError::Generic)?;
                }

                // Check for multi-column layout
                let column_count = extract_column_count(properties);
                let column_gap = extract_column_gap(properties);

                let children = fo_tree.children(node_id);

                if column_count > 1 {
                    // Use multi-column layout
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

                        let child_is_float = fo_tree
                            .get(child_id)
                            .map(|n| matches!(n.data, FoNodeData::Float { .. }))
                            .unwrap_or(false);

                        if child_is_float {
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
                                    block_ctx.current_y = float_manager
                                        .get_clear_position(clear, block_ctx.current_y);
                                }
                            }

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
                                    // Update context with actual height used
                                    block_ctx.current_y =
                                        child_area.area.geometry.y + child_area.area.height();
                                }
                            }
                        }
                    }

                    float_manager.clear();
                }

                Some(area_id)
            }

            FoNodeData::Table { properties } => {
                // Extract table-layout property
                let layout_mode = if let Ok(prop) = properties.get(PropertyId::TableLayout) {
                    if let Some(enum_val) = prop.as_enum() {
                        // EN_AUTO = 9, EN_FIXED = 51
                        if enum_val == 9 {
                            TableLayoutMode::Auto
                        } else {
                            TableLayoutMode::Fixed
                        }
                    } else if prop.is_auto() {
                        TableLayoutMode::Auto
                    } else {
                        TableLayoutMode::Fixed
                    }
                } else {
                    TableLayoutMode::Fixed // Default
                };

                // Extract border-collapse property
                let border_collapse = if let Ok(prop) = properties.get(PropertyId::BorderCollapse) {
                    if let Some(enum_val) = prop.as_enum() {
                        // EN_SEPARATE = 102, EN_COLLAPSE = 28
                        if enum_val == 28 {
                            BorderCollapse::Collapse
                        } else {
                            BorderCollapse::Separate
                        }
                    } else if let Some(string_val) = prop.as_string() {
                        if string_val == "collapse" {
                            BorderCollapse::Collapse
                        } else {
                            BorderCollapse::Separate
                        }
                    } else {
                        BorderCollapse::Separate
                    }
                } else {
                    BorderCollapse::Separate // Default
                };

                // Extract border-spacing property (only used for separate borders)
                let border_spacing = if let Ok(prop) = properties.get(PropertyId::BorderSpacing) {
                    prop.as_length().unwrap_or(Length::from_pt(0.0))
                } else {
                    Length::from_pt(0.0) // Default per XSL-FO spec
                };

                // Create table layout
                let available_width = self.page_width - Length::from_pt(144.0); // 1 inch margins
                let table_layout = TableLayout::new(available_width)
                    .with_border_spacing(border_spacing)
                    .with_layout_mode(layout_mode)
                    .with_border_collapse(border_collapse);

                // Extract column definitions and separate table sections
                let children = fo_tree.children(node_id);
                let mut column_widths = Vec::new();
                let mut header_id = None;
                let mut footer_id = None;
                let mut body_ids = Vec::new();

                for child_id in children.clone() {
                    if let Some(child) = fo_tree.get(child_id) {
                        match &child.data {
                            FoNodeData::TableColumn { .. } => {
                                // Parse column width from properties
                                if let Some(props) = child.data.properties() {
                                    if let Ok(width) = props.get(PropertyId::ColumnWidth) {
                                        if let Some(len) = width.as_length() {
                                            column_widths.push(ColumnWidth::Fixed(len));
                                        } else if width.is_auto() {
                                            column_widths.push(ColumnWidth::Auto);
                                        }
                                    } else {
                                        column_widths.push(ColumnWidth::Auto);
                                    }
                                }
                            }
                            FoNodeData::TableHeader { .. } => {
                                header_id = Some(child_id);
                            }
                            FoNodeData::TableFooter { .. } => {
                                footer_id = Some(child_id);
                            }
                            FoNodeData::TableBody { .. } => {
                                body_ids.push(child_id);
                            }
                            _ => {}
                        }
                    }
                }

                // If no explicit columns, use proportional layout
                if column_widths.is_empty() {
                    column_widths.push(ColumnWidth::Proportional(1.0));
                }

                // Compute column widths based on layout mode
                let computed_widths = match layout_mode {
                    TableLayoutMode::Fixed => table_layout.compute_fixed_widths(&column_widths),
                    TableLayoutMode::Auto => {
                        // For auto layout, we need to create ColumnInfo with measurements
                        let mut column_info: Vec<ColumnInfo> = column_widths
                            .iter()
                            .map(|width_spec| ColumnInfo::new(width_spec.clone()))
                            .collect();

                        // Build grid to measure content (simplified - in full implementation
                        // would need actual cell content measurements)
                        let grid = table_layout.create_grid(1, column_info.len());
                        table_layout.update_column_info_from_grid(&mut column_info, &grid);

                        table_layout.compute_auto_widths(&column_info)
                    }
                };

                // Create table area
                let table_height = Length::from_pt(100.0); // Placeholder
                let table_rect =
                    Rect::new(Length::ZERO, Length::ZERO, available_width, table_height);

                let mut traits = TraitSet::default();
                if let Ok(color) = properties.get(PropertyId::BackgroundColor) {
                    traits.background_color = color.as_color();
                }

                let area = Area::new(AreaType::Block, table_rect).with_traits(traits);
                let table_id = area_tree.add_area(area);

                if let Some(parent) = parent_area {
                    area_tree
                        .append_child(parent, table_id)
                        .map_err(fop_types::FopError::Generic)?;
                }

                // Layout table with header, body, and footer
                self.layout_table(
                    fo_tree,
                    area_tree,
                    table_id,
                    header_id,
                    footer_id,
                    &body_ids,
                    &computed_widths,
                    resolver,
                )?;

                Some(table_id)
            }

            FoNodeData::ExternalGraphic {
                src,
                content_width,
                content_height,
                scaling,
                properties,
            } => {
                // Layout external-graphic as Viewport area with image data
                if let Some(parent) = parent_area {
                    self.layout_external_graphic(
                        fo_tree,
                        node_id,
                        src,
                        content_width.as_deref(),
                        content_height.as_deref(),
                        scaling.as_deref(),
                        properties,
                        area_tree,
                        parent,
                    )
                } else {
                    None
                }
            }

            FoNodeData::ListBlock { properties } => {
                // Read provisional-distance-between-starts (default 24pt per XSL-FO spec)
                let provisional_distance = properties
                    .get(PropertyId::ProvisionalDistanceBetweenStarts)
                    .ok()
                    .and_then(|v| v.as_length())
                    .unwrap_or_else(|| Length::from_pt(24.0));

                // Read provisional-label-separation (default 6pt per XSL-FO spec)
                let provisional_label_sep = properties
                    .get(PropertyId::ProvisionalLabelSeparation)
                    .ok()
                    .and_then(|v| v.as_length())
                    .unwrap_or_else(|| Length::from_pt(6.0));

                // space-after on the list block
                let list_space_after = extract_space_after(properties);

                // Determine available width from parent area or fallback to page width minus margins
                let available_width = if let Some(parent) = parent_area {
                    area_tree
                        .get(parent)
                        .map(|n| n.area.width())
                        .filter(|w| *w > Length::ZERO)
                        .unwrap_or_else(|| self.page_width - Length::from_pt(144.0))
                } else {
                    self.page_width - Length::from_pt(144.0)
                };

                // provisional-distance-between-starts sets the label width;
                // label-end() = start_indent + provisional-distance-between-starts - provisional-label-separation
                // body-start() = start_indent + provisional-distance-between-starts
                // For start_indent=0: label width = provisional_distance - provisional_label_sep
                let label_end = provisional_distance - provisional_label_sep;
                let body_start = provisional_distance;

                let list_layout = ListLayout::new(available_width)
                    .with_label_width(label_end)
                    .with_label_separation(provisional_label_sep)
                    .with_body_start(body_start);

                let mut traits = TraitSet::default();
                if let Ok(color) = properties.get(PropertyId::Color) {
                    traits.color = color.as_color();
                }
                if let Ok(bg) = properties.get(PropertyId::BackgroundColor) {
                    traits.background_color = bg.as_color();
                }

                // Initial placeholder rect (height will be updated after layout)
                let list_rect = Rect::new(
                    Length::ZERO,
                    Length::ZERO,
                    available_width,
                    Length::from_pt(10.0),
                );

                let area = Area::new(AreaType::Block, list_rect).with_traits(traits);
                let list_id = area_tree.add_area(area);

                if let Some(parent) = parent_area {
                    area_tree
                        .append_child(parent, list_id)
                        .map_err(fop_types::FopError::Generic)?;
                }

                // Layout list items
                let children = fo_tree.children(node_id);
                let mut item_y = Length::ZERO;
                let mut item_index = 0usize;

                for child_id in children.iter() {
                    if let Some(child) = fo_tree.get(*child_id) {
                        if matches!(child.data, FoNodeData::ListItem { .. }) {
                            item_index += 1;
                            item_y = self.layout_list_item(
                                fo_tree,
                                *child_id,
                                area_tree,
                                list_id,
                                item_y,
                                item_index,
                                &list_layout,
                                resolver,
                            )?;
                        }
                    }
                }

                // Update list block height to match actual content
                let total_height = item_y + list_space_after;
                if let Some(list_node) = area_tree.get_mut(list_id) {
                    list_node.area.geometry.height = total_height.max(Length::from_pt(10.0));
                }

                Some(list_id)
            }

            FoNodeData::Float { properties } => {
                // Extract float property to determine side
                let float_side = if let Ok(prop) = properties.get(PropertyId::Float) {
                    if let Some(enum_val) = prop.as_enum() {
                        // Float enum values from XSL-FO:
                        // EN_BEFORE = 11, EN_START = 104, EN_END = 45, EN_LEFT = 66, EN_RIGHT = 96, EN_NONE = 77
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

                // If float is None, treat as regular block
                if float_side == FloatSide::None {
                    return Ok(None);
                }

                // Create a float area with measured width
                let traits = extract_traits(properties);

                // Measure float width from children (fallback: 1/3 of page width)
                let container_w = self.page_width - Length::from_pt(144.0);
                let float_width = self.measure_float_width(fo_tree, node_id, container_w);

                // X-position defaults to left edge; proper positioning is handled
                // in layout_float_in_flow when called from a flow context.
                let float_rect = Rect::from_point_size(
                    Point::ZERO,
                    Size::new(float_width, Length::from_pt(1.0)),
                );

                let area = Area::new(AreaType::FloatArea, float_rect).with_traits(traits);
                let float_area_id = area_tree.add_area(area);

                if let Some(parent) = parent_area {
                    area_tree
                        .append_child(parent, float_area_id)
                        .map_err(fop_types::FopError::Generic)?;
                }

                // Layout children within the float
                let children = fo_tree.children(node_id);
                let mut float_ctx = BlockLayoutContext::new(float_width);

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
                            float_ctx.current_y =
                                child_area.area.geometry.y + child_area.area.height();
                        }
                    }
                }

                // Update float area height based on content
                let float_height = if float_ctx.current_y > Length::ZERO {
                    float_ctx.current_y
                } else {
                    Length::from_pt(50.0)
                };
                if let Some(float_area_node) = area_tree.get_mut(float_area_id) {
                    float_area_node.area.geometry.height = float_height;
                }

                Some(float_area_id)
            }

            FoNodeData::BlockContainer { .. } => {
                // fo:block-container: process children in current context
                // (absolute positioning would be handled separately)
                let children = fo_tree.children(node_id);
                for child_id in children {
                    self.layout_node(
                        fo_tree,
                        child_id,
                        area_tree,
                        parent_area,
                        resolver,
                        marker_map,
                    )?;
                }
                None
            }

            FoNodeData::Wrapper { .. }
            | FoNodeData::Declarations
            | FoNodeData::ColorProfile { .. }
            | FoNodeData::PageSequenceMaster { .. }
            | FoNodeData::UnsupportedElement { .. } => {
                // These elements don't directly produce areas
                None
            }

            _ => {
                // Other nodes don't produce areas yet
                None
            }
        };

        Ok(area_id)
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::types::{FloatInfo, FloatSide};
    use super::*;
    use fop_core::{FoNode, FoNodeData, PropertyList, PropertyValue};

    #[test]
    fn test_engine_creation() {
        let engine = LayoutEngine::new();
        assert_eq!(engine.page_width, Length::from_mm(210.0));
        assert_eq!(engine.page_height, Length::from_mm(297.0));
    }

    #[test]
    fn test_simple_layout() {
        let mut fo_tree = FoArena::new();

        // Create simple FO tree: root -> page-sequence -> flow -> block
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

        let flow = fo_tree.add_node(FoNode::new(FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, flow)
            .expect("test: should succeed");

        let block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(flow, block)
            .expect("test: should succeed");

        // Layout
        let engine = LayoutEngine::new();
        let area_tree = engine.layout(&fo_tree).expect("test: should succeed");

        // Should have created areas
        assert!(!area_tree.is_empty());
    }

    #[test]
    fn test_static_content_header() {
        let mut fo_tree = FoArena::new();

        // Create FO tree with header static-content
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

        // Add static-content for header
        let header = fo_tree.add_node(FoNode::new(FoNodeData::StaticContent {
            flow_name: "xsl-region-before".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, header)
            .expect("test: should succeed");

        // Add block to header
        let header_block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(header, header_block)
            .expect("test: should succeed");

        // Add text to header block
        let header_text =
            fo_tree.add_node(FoNode::new(FoNodeData::Text("Header Text".to_string())));
        fo_tree
            .append_child(header_block, header_text)
            .expect("test: should succeed");

        // Add flow
        let flow = fo_tree.add_node(FoNode::new(FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, flow)
            .expect("test: should succeed");

        let block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(flow, block)
            .expect("test: should succeed");

        // Layout
        let engine = LayoutEngine::new();
        let area_tree = engine.layout(&fo_tree).expect("test: should succeed");

        // Check for header area
        let mut has_header = false;
        for (_, node) in area_tree.iter() {
            if matches!(node.area.area_type, AreaType::Header) {
                has_header = true;
                break;
            }
        }
        assert!(has_header, "Should have created a header area");
    }

    #[test]
    fn test_static_content_footer() {
        let mut fo_tree = FoArena::new();

        // Create FO tree with footer static-content
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

        // Add static-content for footer
        let footer = fo_tree.add_node(FoNode::new(FoNodeData::StaticContent {
            flow_name: "xsl-region-after".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, footer)
            .expect("test: should succeed");

        // Add block to footer
        let footer_block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(footer, footer_block)
            .expect("test: should succeed");

        // Add text to footer block
        let footer_text =
            fo_tree.add_node(FoNode::new(FoNodeData::Text("Footer Text".to_string())));
        fo_tree
            .append_child(footer_block, footer_text)
            .expect("test: should succeed");

        // Add flow
        let flow = fo_tree.add_node(FoNode::new(FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, flow)
            .expect("test: should succeed");

        let block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(flow, block)
            .expect("test: should succeed");

        // Layout
        let engine = LayoutEngine::new();
        let area_tree = engine.layout(&fo_tree).expect("test: should succeed");

        // Check for footer area
        let mut has_footer = false;
        for (_, node) in area_tree.iter() {
            if matches!(node.area.area_type, AreaType::Footer) {
                has_footer = true;
                break;
            }
        }
        assert!(has_footer, "Should have created a footer area");
    }

    #[test]
    fn test_static_content_both_header_and_footer() {
        let mut fo_tree = FoArena::new();

        // Create FO tree with both header and footer
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

        // Add header
        let header = fo_tree.add_node(FoNode::new(FoNodeData::StaticContent {
            flow_name: "xsl-region-before".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, header)
            .expect("test: should succeed");

        let header_block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(header, header_block)
            .expect("test: should succeed");

        // Add footer
        let footer = fo_tree.add_node(FoNode::new(FoNodeData::StaticContent {
            flow_name: "xsl-region-after".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, footer)
            .expect("test: should succeed");

        let footer_block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(footer, footer_block)
            .expect("test: should succeed");

        // Add flow
        let flow = fo_tree.add_node(FoNode::new(FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, flow)
            .expect("test: should succeed");

        let block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(flow, block)
            .expect("test: should succeed");

        // Layout
        let engine = LayoutEngine::new();
        let area_tree = engine.layout(&fo_tree).expect("test: should succeed");

        // Check for both header and footer areas
        let mut has_header = false;
        let mut has_footer = false;
        for (_, node) in area_tree.iter() {
            match node.area.area_type {
                AreaType::Header => has_header = true,
                AreaType::Footer => has_footer = true,
                _ => {}
            }
        }
        assert!(has_header, "Should have created a header area");
        assert!(has_footer, "Should have created a footer area");
    }

    #[test]
    fn test_float_manager_basic() {
        let mut fm = FloatManager::new();
        let container_width = Length::from_pt(400.0);

        // At y=0 with no floats, full width is available
        let (left_off, avail) = fm.available_width(Length::ZERO, container_width);
        assert_eq!(left_off, Length::ZERO);
        assert_eq!(avail, container_width);

        // Add a left float: 100pt wide, from y=0 to y=50
        let float_area_id = crate::area::AreaId::from_index(0);
        fm.add_float(
            FloatInfo {
                area_id: float_area_id,
                side: FloatSide::Left,
                top: Length::ZERO,
                bottom: Length::from_pt(50.0),
                width: Length::from_pt(100.0),
            },
            true, // is_odd_page
        );

        // At y=10, available width should be reduced by 100pt and left_offset=100pt
        let (left_off, avail) = fm.available_width(Length::from_pt(10.0), container_width);
        assert_eq!(left_off, Length::from_pt(100.0));
        assert_eq!(avail, Length::from_pt(300.0));

        // At y=60 (past float bottom), full width again
        let (left_off, avail) = fm.available_width(Length::from_pt(60.0), container_width);
        assert_eq!(left_off, Length::ZERO);
        assert_eq!(avail, container_width);
    }

    #[test]
    fn test_float_manager_right_float() {
        let mut fm = FloatManager::new();
        let container_width = Length::from_pt(400.0);

        // Add a right float: 120pt wide, from y=0 to y=80
        let float_area_id = crate::area::AreaId::from_index(0);
        fm.add_float(
            FloatInfo {
                area_id: float_area_id,
                side: FloatSide::Right,
                top: Length::ZERO,
                bottom: Length::from_pt(80.0),
                width: Length::from_pt(120.0),
            },
            true,
        );

        // Left offset should be 0, available width reduced by 120pt
        let (left_off, avail) = fm.available_width(Length::from_pt(20.0), container_width);
        assert_eq!(left_off, Length::ZERO);
        assert_eq!(avail, Length::from_pt(280.0));
    }

    #[test]
    fn test_float_manager_remove_expired() {
        let mut fm = FloatManager::new();
        let container_width = Length::from_pt(400.0);

        let float_area_id = crate::area::AreaId::from_index(0);
        fm.add_float(
            FloatInfo {
                area_id: float_area_id,
                side: FloatSide::Left,
                top: Length::ZERO,
                bottom: Length::from_pt(50.0),
                width: Length::from_pt(100.0),
            },
            true,
        );

        // Remove floats above y=60 (float ends at 50, so it is removed)
        fm.remove_floats_above(Length::from_pt(60.0));

        // Now full width should be available
        let (left_off, avail) = fm.available_width(Length::from_pt(60.0), container_width);
        assert_eq!(left_off, Length::ZERO);
        assert_eq!(avail, container_width);
    }

    #[test]
    fn test_layout_with_float_node() {
        let mut fo_tree = FoArena::new();

        // Build: root -> page-sequence -> flow -> [float(left), block]
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

        let flow = fo_tree.add_node(FoNode::new(FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, flow)
            .expect("test: should succeed");

        // Add a float element with "left" side (EN_LEFT = 66)
        let mut float_props = PropertyList::new();
        float_props.set(PropertyId::Float, PropertyValue::Enum(66));
        let float_node = fo_tree.add_node(FoNode::new(FoNodeData::Float {
            properties: float_props,
        }));
        fo_tree
            .append_child(flow, float_node)
            .expect("test: should succeed");

        // Add a block inside the float
        let float_block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(float_node, float_block)
            .expect("test: should succeed");
        let float_text =
            fo_tree.add_node(FoNode::new(FoNodeData::Text("Float content".to_string())));
        fo_tree
            .append_child(float_block, float_text)
            .expect("test: should succeed");

        // Add a regular block after the float
        let block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(flow, block)
            .expect("test: should succeed");
        let text = fo_tree.add_node(FoNode::new(FoNodeData::Text("Normal text".to_string())));
        fo_tree
            .append_child(block, text)
            .expect("test: should succeed");

        // Layout
        let engine = LayoutEngine::new();
        let area_tree = engine.layout(&fo_tree).expect("test: should succeed");

        // Should have produced areas
        assert!(!area_tree.is_empty());

        // Should have a FloatArea
        let mut has_float_area = false;
        for (_, node) in area_tree.iter() {
            if matches!(node.area.area_type, AreaType::FloatArea) {
                has_float_area = true;
                break;
            }
        }
        assert!(has_float_area, "Should have created a FloatArea");
    }

    #[test]
    fn test_float_manager_clear_position() {
        let mut fm = FloatManager::new();

        // Add a left float ending at y=80
        let float_area_id = crate::area::AreaId::from_index(0);
        fm.add_float(
            FloatInfo {
                area_id: float_area_id,
                side: FloatSide::Left,
                top: Length::ZERO,
                bottom: Length::from_pt(80.0),
                width: Length::from_pt(100.0),
            },
            true,
        );

        // Add a right float ending at y=60
        let float_area_id2 = crate::area::AreaId::from_index(1);
        fm.add_float(
            FloatInfo {
                area_id: float_area_id2,
                side: FloatSide::Right,
                top: Length::ZERO,
                bottom: Length::from_pt(60.0),
                width: Length::from_pt(80.0),
            },
            true,
        );

        let current_y = Length::from_pt(10.0);

        // clear: left → advance past left float bottom (80pt)
        assert_eq!(
            fm.get_clear_position(ClearSide::Left, current_y),
            Length::from_pt(80.0)
        );

        // clear: right → advance past right float bottom (60pt)
        assert_eq!(
            fm.get_clear_position(ClearSide::Right, current_y),
            Length::from_pt(60.0)
        );

        // clear: both → advance past max(80, 60) = 80pt
        assert_eq!(
            fm.get_clear_position(ClearSide::Both, current_y),
            Length::from_pt(80.0)
        );

        // clear: none → stay at current_y
        assert_eq!(fm.get_clear_position(ClearSide::None, current_y), current_y);
    }

    #[test]
    fn test_layout_with_float_and_clear() {
        let mut fo_tree = FoArena::new();

        // Build: root -> page-sequence -> flow -> [float(right), block(clear:right)]
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

        let flow = fo_tree.add_node(FoNode::new(FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, flow)
            .expect("test: should succeed");

        // Float on the right
        let mut float_props = PropertyList::new();
        float_props.set(PropertyId::Float, PropertyValue::Enum(96)); // EN_RIGHT = 96
        let float_node = fo_tree.add_node(FoNode::new(FoNodeData::Float {
            properties: float_props,
        }));
        fo_tree
            .append_child(flow, float_node)
            .expect("test: should succeed");

        let float_block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(float_node, float_block)
            .expect("test: should succeed");

        // Block with clear:right (EN_RIGHT = 96)
        let mut clear_props = PropertyList::new();
        clear_props.set(PropertyId::Clear, PropertyValue::Enum(96)); // EN_RIGHT = 96
        let clear_block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: clear_props,
        }));
        fo_tree
            .append_child(flow, clear_block)
            .expect("test: should succeed");
        let text = fo_tree.add_node(FoNode::new(FoNodeData::Text("After float".to_string())));
        fo_tree
            .append_child(clear_block, text)
            .expect("test: should succeed");

        // Layout – should not panic
        let engine = LayoutEngine::new();
        let area_tree = engine.layout(&fo_tree).expect("test: should succeed");

        assert!(!area_tree.is_empty());

        // Should have a FloatArea
        let mut has_float = false;
        for (_, node) in area_tree.iter() {
            if matches!(node.area.area_type, AreaType::FloatArea) {
                has_float = true;
                break;
            }
        }
        assert!(has_float, "Expected a FloatArea in the area tree");
    }

    #[test]
    fn test_layout_with_footnote() {
        let mut fo_tree = FoArena::new();

        // Build: root -> page-sequence -> flow -> block -> footnote
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

        let flow = fo_tree.add_node(FoNode::new(FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, flow)
            .expect("test: should succeed");

        let block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(flow, block)
            .expect("test: should succeed");

        // Add text before footnote
        let text_before = fo_tree.add_node(FoNode::new(FoNodeData::Text("Text with ".to_string())));
        fo_tree
            .append_child(block, text_before)
            .expect("test: should succeed");

        // Add fo:footnote with inline reference mark and body
        let footnote = fo_tree.add_node(FoNode::new(FoNodeData::Footnote {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(block, footnote)
            .expect("test: should succeed");

        // fo:inline child (reference mark "1")
        let ref_mark = fo_tree.add_node(FoNode::new(FoNodeData::Inline {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(footnote, ref_mark)
            .expect("test: should succeed");

        let ref_text = fo_tree.add_node(FoNode::new(FoNodeData::Text("1".to_string())));
        fo_tree
            .append_child(ref_mark, ref_text)
            .expect("test: should succeed");

        // fo:footnote-body child
        let footnote_body = fo_tree.add_node(FoNode::new(FoNodeData::FootnoteBody {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(footnote, footnote_body)
            .expect("test: should succeed");

        // Block inside footnote-body
        let body_block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(footnote_body, body_block)
            .expect("test: should succeed");

        let body_text = fo_tree.add_node(FoNode::new(FoNodeData::Text(
            "1. The footnote text.".to_string(),
        )));
        fo_tree
            .append_child(body_block, body_text)
            .expect("test: should succeed");

        // Layout
        let engine = LayoutEngine::new();
        let area_tree = engine.layout(&fo_tree).expect("test: should succeed");

        // Should have produced areas
        assert!(!area_tree.is_empty());

        // Should have a Footnote area
        let mut has_footnote_area = false;
        for (_, node) in area_tree.iter() {
            if matches!(node.area.area_type, AreaType::Footnote) {
                has_footnote_area = true;
                break;
            }
        }
        assert!(has_footnote_area, "Should have created a Footnote area");
    }

    #[test]
    fn test_static_content_sidebar_start() {
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

        // Add static-content for region-start (left sidebar)
        let sidebar_start = fo_tree.add_node(FoNode::new(FoNodeData::StaticContent {
            flow_name: "xsl-region-start".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, sidebar_start)
            .expect("test: should succeed");

        let sidebar_block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(sidebar_start, sidebar_block)
            .expect("test: should succeed");

        // Add flow
        let flow = fo_tree.add_node(FoNode::new(FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, flow)
            .expect("test: should succeed");

        let block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(flow, block)
            .expect("test: should succeed");

        let engine = LayoutEngine::new();
        let area_tree = engine.layout(&fo_tree).expect("test: should succeed");

        let mut has_sidebar_start = false;
        for (_, node) in area_tree.iter() {
            if matches!(node.area.area_type, AreaType::SidebarStart) {
                has_sidebar_start = true;
                break;
            }
        }
        assert!(has_sidebar_start, "Should have created a SidebarStart area");
    }

    #[test]
    fn test_static_content_sidebar_end() {
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

        // Add static-content for region-end (right sidebar)
        let sidebar_end = fo_tree.add_node(FoNode::new(FoNodeData::StaticContent {
            flow_name: "xsl-region-end".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, sidebar_end)
            .expect("test: should succeed");

        let sidebar_block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(sidebar_end, sidebar_block)
            .expect("test: should succeed");

        // Add flow
        let flow = fo_tree.add_node(FoNode::new(FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, flow)
            .expect("test: should succeed");

        let block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(flow, block)
            .expect("test: should succeed");

        let engine = LayoutEngine::new();
        let area_tree = engine.layout(&fo_tree).expect("test: should succeed");

        let mut has_sidebar_end = false;
        for (_, node) in area_tree.iter() {
            if matches!(node.area.area_type, AreaType::SidebarEnd) {
                has_sidebar_end = true;
                break;
            }
        }
        assert!(has_sidebar_end, "Should have created a SidebarEnd area");
    }

    #[test]
    fn test_page_region_geometry_from_simple_page_master() {
        use fop_core::PropertyValue;
        use fop_types::Length;

        let mut fo_tree = FoArena::new();

        // Build: root -> layout-master-set -> simple-page-master
        //           with: region-before(extent=30pt), region-after(extent=20pt),
        //                 region-start(extent=50pt), region-end(extent=40pt)
        let root = fo_tree.add_node(FoNode::new(FoNodeData::Root));

        let lms = fo_tree.add_node(FoNode::new(FoNodeData::LayoutMasterSet));
        fo_tree
            .append_child(root, lms)
            .expect("test: should succeed");

        let mut spm_props = PropertyList::new();
        spm_props.set(
            PropertyId::PageWidth,
            PropertyValue::Length(Length::from_pt(595.0)),
        );
        spm_props.set(
            PropertyId::PageHeight,
            PropertyValue::Length(Length::from_pt(842.0)),
        );
        spm_props.set(
            PropertyId::MarginTop,
            PropertyValue::Length(Length::from_pt(50.0)),
        );
        spm_props.set(
            PropertyId::MarginBottom,
            PropertyValue::Length(Length::from_pt(50.0)),
        );
        spm_props.set(
            PropertyId::MarginLeft,
            PropertyValue::Length(Length::from_pt(60.0)),
        );
        spm_props.set(
            PropertyId::MarginRight,
            PropertyValue::Length(Length::from_pt(60.0)),
        );

        let spm = fo_tree.add_node(FoNode::new(FoNodeData::SimplePageMaster {
            master_name: "test-master".to_string(),
            properties: spm_props,
        }));
        fo_tree
            .append_child(lms, spm)
            .expect("test: should succeed");

        // region-before extent=30pt
        let mut rb_props = PropertyList::new();
        rb_props.set(
            PropertyId::Extent,
            PropertyValue::Length(Length::from_pt(30.0)),
        );
        let rb = fo_tree.add_node(FoNode::new(FoNodeData::RegionBefore {
            properties: rb_props,
        }));
        fo_tree.append_child(spm, rb).expect("test: should succeed");

        // region-after extent=20pt
        let mut ra_props = PropertyList::new();
        ra_props.set(
            PropertyId::Extent,
            PropertyValue::Length(Length::from_pt(20.0)),
        );
        let ra = fo_tree.add_node(FoNode::new(FoNodeData::RegionAfter {
            properties: ra_props,
        }));
        fo_tree.append_child(spm, ra).expect("test: should succeed");

        // region-start extent=50pt
        let mut rs_props = PropertyList::new();
        rs_props.set(
            PropertyId::Extent,
            PropertyValue::Length(Length::from_pt(50.0)),
        );
        let rs = fo_tree.add_node(FoNode::new(FoNodeData::RegionStart {
            properties: rs_props,
        }));
        fo_tree.append_child(spm, rs).expect("test: should succeed");

        // region-end extent=40pt
        let mut re_props = PropertyList::new();
        re_props.set(
            PropertyId::Extent,
            PropertyValue::Length(Length::from_pt(40.0)),
        );
        let re = fo_tree.add_node(FoNode::new(FoNodeData::RegionEnd {
            properties: re_props,
        }));
        fo_tree.append_child(spm, re).expect("test: should succeed");

        // region-body (no inner margins)
        let body_region = fo_tree.add_node(FoNode::new(FoNodeData::RegionBody {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(spm, body_region)
            .expect("test: should succeed");

        let engine = LayoutEngine::new();
        let geom = engine.extract_page_region_geometry(&fo_tree, "test-master");

        // Page dimensions
        assert_eq!(geom.page_width, Length::from_pt(595.0));
        assert_eq!(geom.page_height, Length::from_pt(842.0));

        // Content area: x=60, y=50, w=595-120=475, h=842-100=742
        // region-before: x=60, y=50, w=475, h=30
        assert_eq!(geom.before_rect.x, Length::from_pt(60.0));
        assert_eq!(geom.before_rect.y, Length::from_pt(50.0));
        assert_eq!(geom.before_rect.width, Length::from_pt(475.0));
        assert_eq!(geom.before_rect.height, Length::from_pt(30.0));

        // region-after: x=60, y=50+742-20=772, w=475, h=20
        assert_eq!(geom.after_rect.x, Length::from_pt(60.0));
        assert_eq!(geom.after_rect.y, Length::from_pt(772.0));
        assert_eq!(geom.after_rect.width, Length::from_pt(475.0));
        assert_eq!(geom.after_rect.height, Length::from_pt(20.0));

        // sidebar top = 50+30=80, sidebar h = 742-30-20=692
        // region-start: x=60, y=80, w=50, h=692
        assert_eq!(geom.start_rect.x, Length::from_pt(60.0));
        assert_eq!(geom.start_rect.y, Length::from_pt(80.0));
        assert_eq!(geom.start_rect.width, Length::from_pt(50.0));
        assert_eq!(geom.start_rect.height, Length::from_pt(692.0));

        // region-end: x=60+475-40=495, y=80, w=40, h=692
        assert_eq!(geom.end_rect.x, Length::from_pt(495.0));
        assert_eq!(geom.end_rect.y, Length::from_pt(80.0));
        assert_eq!(geom.end_rect.width, Length::from_pt(40.0));
        assert_eq!(geom.end_rect.height, Length::from_pt(692.0));

        // region-body: x=60+50=110, y=80, w=475-50-40=385, h=692
        assert_eq!(geom.body_rect.x, Length::from_pt(110.0));
        assert_eq!(geom.body_rect.y, Length::from_pt(80.0));
        assert_eq!(geom.body_rect.width, Length::from_pt(385.0));
        assert_eq!(geom.body_rect.height, Length::from_pt(692.0));
    }

    #[test]
    fn test_all_five_regions_layout() {
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

        // Add all four static-content regions
        for flow_name in &[
            "xsl-region-before",
            "xsl-region-after",
            "xsl-region-start",
            "xsl-region-end",
        ] {
            let sc = fo_tree.add_node(FoNode::new(FoNodeData::StaticContent {
                flow_name: flow_name.to_string(),
                properties: PropertyList::new(),
            }));
            fo_tree
                .append_child(page_seq, sc)
                .expect("test: should succeed");
            let sc_block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
                properties: PropertyList::new(),
            }));
            fo_tree
                .append_child(sc, sc_block)
                .expect("test: should succeed");
        }

        // Add flow (region-body)
        let flow = fo_tree.add_node(FoNode::new(FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(page_seq, flow)
            .expect("test: should succeed");

        let block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(flow, block)
            .expect("test: should succeed");

        let engine = LayoutEngine::new();
        let area_tree = engine.layout(&fo_tree).expect("test: should succeed");

        let mut has_header = false;
        let mut has_footer = false;
        let mut has_sidebar_start = false;
        let mut has_sidebar_end = false;
        let mut has_region = false;

        for (_, node) in area_tree.iter() {
            match node.area.area_type {
                AreaType::Header => has_header = true,
                AreaType::Footer => has_footer = true,
                AreaType::SidebarStart => has_sidebar_start = true,
                AreaType::SidebarEnd => has_sidebar_end = true,
                AreaType::Region => has_region = true,
                _ => {}
            }
        }

        assert!(has_header, "Should have a header area");
        assert!(has_footer, "Should have a footer area");
        assert!(has_sidebar_start, "Should have a sidebar-start area");
        assert!(has_sidebar_end, "Should have a sidebar-end area");
        assert!(has_region, "Should have a body region area");
    }

    /// Parse a length string like "10mm", "72pt", "1.5in" to a Length value
    #[allow(dead_code)]
    fn parse_fo_length_str(s: &str) -> Option<Length> {
        if let Some(v) = s.strip_suffix("pt") {
            v.parse::<f64>().ok().map(Length::from_pt)
        } else if let Some(v) = s.strip_suffix("mm") {
            v.parse::<f64>().ok().map(Length::from_mm)
        } else if let Some(v) = s.strip_suffix("cm") {
            v.parse::<f64>().ok().map(Length::from_cm)
        } else if let Some(v) = s.strip_suffix("in") {
            v.parse::<f64>().ok().map(Length::from_inch)
        } else if let Some(v) = s.strip_suffix("px") {
            v.parse::<f64>().ok().map(|px| Length::from_pt(px * 0.75))
        } else {
            None
        }
    }
}
