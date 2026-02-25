//! Block-level layout methods for the layout engine.
//!
//! Handles `fo:block`, `fo:block-container` in single-column and multi-column contexts.

use crate::area::{Area, AreaTree, AreaType};
use crate::layout::{
    extract_break_after, extract_break_before, extract_end_indent, extract_keep_constraint,
    extract_orphans, extract_space_after, extract_space_before, extract_start_indent,
    extract_text_indent, extract_traits, extract_widows, BlockLayoutContext, BreakValue,
    PageNumberResolver, TextAlign,
};
use fop_core::{FoArena, FoNodeData, NodeId};
use fop_types::{Length, Rect, Result};

use super::types::MultiColumnLayout;
use super::LayoutEngine;

impl LayoutEngine {
    /// Layout a block-level element
    #[allow(clippy::too_many_arguments)]
    pub(super) fn layout_block(
        &self,
        fo_tree: &FoArena,
        node_id: NodeId,
        area_tree: &mut AreaTree,
        parent_area: crate::area::AreaId,
        y_offset: Length,
        available_width: Length,
        resolver: &mut PageNumberResolver,
    ) -> Result<Option<crate::area::AreaId>> {
        let node = fo_tree
            .get(node_id)
            .ok_or_else(|| fop_types::FopError::Generic(format!("Node {} not found", node_id)))?;

        match &node.data {
            FoNodeData::Block { properties } | FoNodeData::BlockContainer { properties } => {
                // Extract properties using the properties module
                let traits = extract_traits(properties);

                // Extract spacing properties
                let space_before = extract_space_before(properties);
                let space_after = extract_space_after(properties);

                // Extract indent properties
                let start_indent = extract_start_indent(properties);
                let end_indent = extract_end_indent(properties);
                let text_indent = extract_text_indent(properties);

                // Extract keep constraints
                let keep_constraint = extract_keep_constraint(properties);

                // Extract break properties
                let break_before = extract_break_before(properties);
                let break_after = extract_break_after(properties);

                // Extract widow and orphan constraints
                let widows = extract_widows(properties);
                let orphans = extract_orphans(properties);

                // Calculate height (simplified - just use font size for now)
                let line_height = traits.font_size.unwrap_or(Length::from_pt(12.0));

                // Calculate the adjusted width and x-position based on indents
                // start-indent acts as left margin, end-indent as right margin
                let content_width = available_width - start_indent - end_indent;

                // Use BlockLayoutContext for spacing
                let mut block_ctx = BlockLayoutContext::new(content_width);
                block_ctx.current_y = y_offset;
                let mut block_rect = block_ctx.allocate_with_spacing(
                    content_width,
                    line_height,
                    space_before,
                    space_after,
                );

                // Adjust x position for start-indent
                block_rect.x = start_indent;

                let mut area = Area::new(AreaType::Block, block_rect).with_traits(traits.clone());

                // Set keep constraint if active
                if keep_constraint.has_constraint() {
                    area = area.with_keep_constraint(keep_constraint);
                }

                // Set break-before if specified
                if break_before.forces_break() {
                    area = area.with_break_before(break_before);
                }

                // Set break-after if specified
                if break_after.forces_break() {
                    area = area.with_break_after(break_after);
                }

                // Set widows and orphans constraints
                area = area.with_widows(widows).with_orphans(orphans);

                let area_id = area_tree.add_area(area);
                area_tree
                    .append_child(parent_area, area_id)
                    .map_err(fop_types::FopError::Generic)?;

                // Get text alignment from traits
                let text_align = traits.text_align.unwrap_or(TextAlign::Left);

                // Process text children and inline elements (including BasicLink)
                let children = fo_tree.children(node_id);
                let mut is_first_line = true;
                for child_id in children {
                    if let Some(child_node) = fo_tree.get(child_id) {
                        match &child_node.data {
                            FoNodeData::Text(text) => {
                                // Create text area with alignment
                                let mut text_traits = traits.clone();
                                text_traits.text_align = Some(text_align);

                                // Apply text-indent to the first line
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

                                // Calculate alignment offset
                                // For simplicity, assume text fills entire width for now
                                // In a real implementation, you would measure text width
                                let text_rect =
                                    Rect::new(x_offset, Length::ZERO, line_width, line_height);

                                let text_area =
                                    Area::text(text_rect, text.clone()).with_traits(text_traits);
                                let text_id = area_tree.add_area(text_area);
                                area_tree
                                    .append_child(area_id, text_id)
                                    .map_err(fop_types::FopError::Generic)?;

                                is_first_line = false;
                            }
                            FoNodeData::BasicLink {
                                external_destination,
                                internal_destination,
                                properties: link_props,
                            } => {
                                // Process basic-link as inline with link trait
                                let mut link_traits = extract_traits(link_props);
                                link_traits.text_align = Some(text_align);

                                // Set link destination (prefer external over internal)
                                link_traits.link_destination = external_destination
                                    .clone()
                                    .or_else(|| internal_destination.clone());

                                // Process link children (usually Inline or Text)
                                let link_children = fo_tree.children(child_id);
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
                                    )?;
                                    is_first_line = false;
                                }
                            }
                            FoNodeData::Inline {
                                properties: inline_props,
                            } => {
                                // Process inline element
                                let inline_traits = extract_traits(inline_props);
                                let link_children = fo_tree.children(child_id);
                                for inline_child_id in link_children {
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
                                    )?;
                                    is_first_line = false;
                                }
                            }
                            FoNodeData::Leader {
                                properties: leader_props,
                            } => {
                                // Process leader element
                                self.layout_leader(
                                    fo_tree,
                                    child_id,
                                    leader_props,
                                    area_tree,
                                    area_id,
                                    content_width,
                                    line_height,
                                    &traits,
                                )?;
                                is_first_line = false;
                            }
                            FoNodeData::PageNumberCitation {
                                ref_id,
                                properties: citation_props,
                            } => {
                                // Process page-number-citation
                                let mut citation_traits = extract_traits(citation_props);
                                citation_traits.text_align = Some(text_align);

                                // Apply text-indent to the first line
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

                                // Create placeholder text area (will be resolved in second pass)
                                let text_rect =
                                    Rect::new(x_offset, Length::ZERO, line_width, line_height);
                                let citation_area = Area::text(text_rect, "0".to_string())
                                    .with_traits(citation_traits);
                                let citation_id = area_tree.add_area(citation_area);

                                // Register this citation for later resolution
                                resolver.register_citation(citation_id, ref_id.clone());

                                area_tree
                                    .append_child(area_id, citation_id)
                                    .map_err(fop_types::FopError::Generic)?;

                                is_first_line = false;
                            }
                            FoNodeData::Footnote { .. } => {
                                // fo:footnote: render inline reference mark, collect body at page bottom
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
                                    resolver,
                                )?;
                                is_first_line = false;
                            }
                            FoNodeData::PageNumber {
                                properties: page_num_props,
                            } => {
                                // fo:page-number: render current page number as inline text
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

                                // Use current page number from resolver
                                let page_num_str = resolver.current_page().to_string();
                                let pn_rect =
                                    Rect::new(x_offset, Length::ZERO, line_width, line_height);
                                let pn_area =
                                    Area::text(pn_rect, page_num_str).with_traits(pn_traits);
                                let pn_id = area_tree.add_area(pn_area);
                                area_tree
                                    .append_child(area_id, pn_id)
                                    .map_err(fop_types::FopError::Generic)?;
                                is_first_line = false;
                            }
                            _ => {}
                        }
                    }
                }

                // Register ID if present in block
                if let Some(id) = &node.id {
                    resolver.register_element(id.clone(), area_id);
                }

                Ok(Some(area_id))
            }
            _ => Ok(None),
        }
    }

    /// Layout a block-level element in multi-column context
    #[allow(clippy::too_many_arguments)]
    pub(super) fn layout_block_multicolumn(
        &self,
        fo_tree: &FoArena,
        node_id: NodeId,
        area_tree: &mut AreaTree,
        parent_area: crate::area::AreaId,
        multi_col: &mut MultiColumnLayout,
        resolver: &mut PageNumberResolver,
    ) -> Result<Option<crate::area::AreaId>> {
        let node = fo_tree
            .get(node_id)
            .ok_or_else(|| fop_types::FopError::Generic(format!("Node {} not found", node_id)))?;

        match &node.data {
            FoNodeData::Block { properties } | FoNodeData::BlockContainer { properties } => {
                // Extract properties
                let traits = extract_traits(properties);
                let space_before = extract_space_before(properties);
                let space_after = extract_space_after(properties);
                let start_indent = extract_start_indent(properties);
                let end_indent = extract_end_indent(properties);
                let text_indent = extract_text_indent(properties);
                let keep_constraint = extract_keep_constraint(properties);
                let break_before = extract_break_before(properties);
                let break_after = extract_break_after(properties);
                let widows = extract_widows(properties);
                let orphans = extract_orphans(properties);

                // Calculate content dimensions
                let line_height = traits.font_size.unwrap_or(Length::from_pt(12.0));
                let content_width = multi_col.column_width() - start_indent - end_indent;

                // Apply space-before
                let (_x_pos, _y_pos) = multi_col.allocate(space_before);

                // Check for column break before
                if break_before.forces_break() && !matches!(break_before, BreakValue::Column) {
                    // Page break takes precedence over column break
                    // For now, just treat it as a column break
                    if !multi_col.next_column() {
                        // Need new page - for now, just continue in same column
                    }
                }

                // Check if block fits in current column
                let total_height = space_before + line_height + space_after;
                if multi_col.is_column_filled(total_height) {
                    // Move to next column
                    if !multi_col.next_column() {
                        // All columns filled - would need new page
                        // For now, continue in current column (overflow)
                    }
                }

                // Allocate space for block
                let (block_x, block_y) = multi_col.allocate(line_height);

                // Create block area with column offset
                let block_rect =
                    Rect::new(block_x + start_indent, block_y, content_width, line_height);

                let mut area = Area::new(AreaType::Block, block_rect).with_traits(traits.clone());

                // Set constraints
                if keep_constraint.has_constraint() {
                    area = area.with_keep_constraint(keep_constraint);
                }
                if break_before.forces_break() {
                    area = area.with_break_before(break_before);
                }
                if break_after.forces_break() {
                    area = area.with_break_after(break_after);
                }
                area = area.with_widows(widows).with_orphans(orphans);

                let area_id = area_tree.add_area(area);
                area_tree
                    .append_child(parent_area, area_id)
                    .map_err(fop_types::FopError::Generic)?;

                // Get text alignment from traits
                let text_align = traits.text_align.unwrap_or(TextAlign::Left);

                // Process text children and inline elements
                let children = fo_tree.children(node_id);
                let mut is_first_line = true;
                for child_id in children {
                    if let Some(child_node) = fo_tree.get(child_id) {
                        match &child_node.data {
                            FoNodeData::Text(text) => {
                                let mut text_traits = traits.clone();
                                text_traits.text_align = Some(text_align);

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
                                    Rect::new(x_offset, Length::ZERO, line_width, line_height);

                                let text_area =
                                    Area::text(text_rect, text.clone()).with_traits(text_traits);
                                let text_id = area_tree.add_area(text_area);
                                area_tree
                                    .append_child(area_id, text_id)
                                    .map_err(fop_types::FopError::Generic)?;

                                is_first_line = false;
                            }
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
                                    )?;
                                    is_first_line = false;
                                }
                            }
                            FoNodeData::Inline {
                                properties: inline_props,
                            } => {
                                let inline_traits = extract_traits(inline_props);
                                let link_children = fo_tree.children(child_id);
                                for inline_child_id in link_children {
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
                                    )?;
                                    is_first_line = false;
                                }
                            }
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
                                )?;
                                is_first_line = false;
                            }
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
                                    Rect::new(x_offset, Length::ZERO, line_width, line_height);
                                let citation_area = Area::text(text_rect, "0".to_string())
                                    .with_traits(citation_traits);
                                let citation_id = area_tree.add_area(citation_area);

                                resolver.register_citation(citation_id, ref_id.clone());

                                area_tree
                                    .append_child(area_id, citation_id)
                                    .map_err(fop_types::FopError::Generic)?;

                                is_first_line = false;
                            }
                            FoNodeData::Footnote { .. } => {
                                // fo:footnote in multi-column: render reference mark inline, collect body
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
                                    resolver,
                                )?;
                                is_first_line = false;
                            }
                            FoNodeData::PageNumber {
                                properties: page_num_props,
                            } => {
                                // fo:page-number: render current page number as inline text
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
                                    Rect::new(x_offset, Length::ZERO, line_width, line_height);
                                let pn_area =
                                    Area::text(pn_rect, page_num_str).with_traits(pn_traits);
                                let pn_id = area_tree.add_area(pn_area);
                                area_tree
                                    .append_child(area_id, pn_id)
                                    .map_err(fop_types::FopError::Generic)?;
                                is_first_line = false;
                            }
                            _ => {}
                        }
                    }
                }

                // Apply space-after
                multi_col.allocate(space_after);

                // Check for column break after
                if break_after.forces_break() && matches!(break_after, BreakValue::Column) {
                    multi_col.next_column();
                }

                // Register ID if present
                if let Some(id) = &node.id {
                    resolver.register_element(id.clone(), area_id);
                }

                Ok(Some(area_id))
            }
            _ => Ok(None),
        }
    }
}
