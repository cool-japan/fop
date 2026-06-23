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

                // Determine the per-line height: prefer the resolved line-height property
                // (already multiplied against font-size for unitless values by extract_line_height),
                // then fall back to font-size, then to the 12pt default.
                let line_height = traits
                    .line_height
                    .or(traits.font_size)
                    .unwrap_or(Length::from_pt(12.0));

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

                // Process text children and inline elements (including BasicLink).
                // `content_y` is the running vertical cursor inside the block:
                // wrapped text advances it per line, inline children per line each.
                let children = fo_tree.children(node_id);
                let mut is_first_line = true;
                let mut content_y = Length::ZERO;
                for child_id in children {
                    if let Some(child_node) = fo_tree.get(child_id) {
                        match &child_node.data {
                            FoNodeData::Text(text) => {
                                // Break the text run into optimal lines (Knuth-Plass)
                                // and emit one positioned area per line.
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

                                // Whitespace-only runs emit nothing and must not
                                // consume the first-line indent.
                                if new_y != content_y {
                                    is_first_line = false;
                                    content_y = new_y;
                                }
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
                            FoNodeData::Inline {
                                properties: inline_props,
                            } => {
                                // Process inline element
                                let inline_traits = extract_traits(inline_props);
                                let link_children = fo_tree.children(child_id);
                                let had_children = !link_children.is_empty();
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
                                        content_y,
                                    )?;
                                    is_first_line = false;
                                }
                                if had_children {
                                    content_y += line_height;
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
                                    content_y,
                                )?;
                                is_first_line = false;
                                content_y += line_height;
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
                                    Rect::new(x_offset, content_y, line_width, line_height);
                                let citation_area = Area::text(text_rect, "0".to_string())
                                    .with_traits(citation_traits);
                                let citation_id = area_tree.add_area(citation_area);

                                // Register this citation for later resolution
                                resolver.register_citation(citation_id, ref_id.clone());

                                area_tree
                                    .append_child(area_id, citation_id)
                                    .map_err(fop_types::FopError::Generic)?;

                                is_first_line = false;
                                content_y += line_height;
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
                                    content_y,
                                    resolver,
                                )?;
                                is_first_line = false;
                                content_y += line_height;
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
                            _ => {}
                        }
                    }
                }

                // Set the block's content height to the stacked line total. An
                // empty/whitespace-only block keeps one line of height so its
                // geometry matches the resolved line-height (historic behaviour).
                let content_height = content_y.max(line_height);
                if let Some(block_node) = area_tree.get_mut(area_id) {
                    block_node.area.geometry.height = content_height;
                }

                // Register ID if present in block
                if let Some(id) = &node.id {
                    resolver.register_element(id.clone(), area_id);
                }

                Ok(Some(area_id))
            }
            // An fo:table that is a direct child of an fo:flow arrives here (the
            // flow dispatches every child through layout_block).  Route it to the
            // shared table entry point so it actually produces a table area at the
            // current flow y-offset, rather than being silently dropped (GAP 2).
            FoNodeData::Table { properties } => self.layout_table_node(
                fo_tree,
                node_id,
                properties,
                area_tree,
                Some(parent_area),
                available_width,
                y_offset,
                resolver,
            ),
            _ => Ok(None),
        }
    }

    /// Layout a block-level element in multi-column context (single page).
    ///
    /// Drives the **single-page** column flow: applies `space-before` /
    /// `space-after`, advances to the next column when the current one is full
    /// (overflowing the *last* column, which is the legacy single-page
    /// behaviour), and delegates the actual block placement to
    /// [`Self::emit_multicolumn_block`].  Cross-page column flow is driven by the
    /// paginator (`pagination.rs`), which shares the same placement primitive but
    /// starts a new page instead of overflowing the last column.
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
                // Flow-control measurements only; the actual block placement (area
                // creation + inline content + height) is delegated to
                // emit_multicolumn_block so it is shared with the paginator.
                let traits = extract_traits(properties);
                let space_before = extract_space_before(properties);
                let space_after = extract_space_after(properties);
                let break_before = extract_break_before(properties);
                let break_after = extract_break_after(properties);

                // Determine the per-line height: prefer the resolved line-height property
                // (already multiplied against font-size for unitless values by extract_line_height),
                // then fall back to font-size, then to the 12pt default.
                let line_height = traits
                    .line_height
                    .or(traits.font_size)
                    .unwrap_or(Length::from_pt(12.0));

                // Apply space-before.
                multi_col.allocate(space_before);

                // Forced break-before: a non-column forced break is treated as a
                // column break here (real page breaks belong to the paginator,
                // which owns page geometry); preserves legacy single-page behaviour.
                if break_before.forces_break() && !matches!(break_before, BreakValue::Column) {
                    multi_col.next_column();
                }

                // Move to the next column when the current one is full.  On the
                // last column this is a no-op and the block overflows (single-page
                // legacy behaviour; the paginator instead starts a new page).
                let total_height = space_before + line_height + space_after;
                if multi_col.is_column_filled(total_height) {
                    multi_col.next_column();
                }

                // Place the block in the current column.
                let area_id_opt = self.emit_multicolumn_block(
                    fo_tree,
                    node_id,
                    area_tree,
                    parent_area,
                    multi_col,
                    resolver,
                )?;

                // Apply space-after.
                multi_col.allocate(space_after);

                // Forced column break-after.
                if break_after.forces_break() && matches!(break_after, BreakValue::Column) {
                    multi_col.next_column();
                }

                Ok(area_id_opt)
            }
            _ => Ok(None),
        }
    }

    /// Place a single block into the **current** column of `multi_col` under
    /// `parent_area`, laying out its inline content and advancing the column
    /// cursor by the block's resolved height.
    ///
    /// This is the shared placement primitive used by both the single-page column
    /// flow ([`Self::layout_block_multicolumn`]) and the cross-page column
    /// paginator (`pagination.rs`).  Column selection, inter-block spacing
    /// (`space-before` / `space-after`) and column / page breaks are the
    /// **caller's** responsibility; this method only allocates one line for the
    /// block, emits its content, then grows the column cursor by any extra
    /// (wrapped) height.  Sharing it guarantees a block's geometry is produced
    /// identically regardless of which page it lands on.
    pub(super) fn emit_multicolumn_block(
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
                let start_indent = extract_start_indent(properties);
                let end_indent = extract_end_indent(properties);
                let text_indent = extract_text_indent(properties);
                let keep_constraint = extract_keep_constraint(properties);
                let break_before = extract_break_before(properties);
                let break_after = extract_break_after(properties);
                let widows = extract_widows(properties);
                let orphans = extract_orphans(properties);

                // Determine the per-line height: prefer the resolved line-height property
                // (already multiplied against font-size for unitless values by extract_line_height),
                // then fall back to font-size, then to the 12pt default.
                let line_height = traits
                    .line_height
                    .or(traits.font_size)
                    .unwrap_or(Length::from_pt(12.0));
                let content_width = multi_col.column_width() - start_indent - end_indent;

                // Allocate one line for the block at the current column cursor.
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

                // Process text children and inline elements. `content_y` is the
                // vertical cursor inside the block (relative to its top).
                let children = fo_tree.children(node_id);
                let mut is_first_line = true;
                let mut content_y = Length::ZERO;
                for child_id in children {
                    if let Some(child_node) = fo_tree.get(child_id) {
                        match &child_node.data {
                            FoNodeData::Text(text) => {
                                // Break the text run into optimal lines (Knuth-Plass).
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
                            FoNodeData::Inline {
                                properties: inline_props,
                            } => {
                                let inline_traits = extract_traits(inline_props);
                                let link_children = fo_tree.children(child_id);
                                let had_children = !link_children.is_empty();
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
                                        content_y,
                                    )?;
                                    is_first_line = false;
                                }
                                if had_children {
                                    content_y += line_height;
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
                                    content_y,
                                )?;
                                is_first_line = false;
                                content_y += line_height;
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
                                    content_y,
                                    resolver,
                                )?;
                                is_first_line = false;
                                content_y += line_height;
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
                            _ => {}
                        }
                    }
                }

                // Resolve the block's true content height and push the column
                // cursor down by any height beyond the single line already
                // allocated, so following blocks stack below the wrapped text.
                let content_height = content_y.max(line_height);
                if let Some(block_node) = area_tree.get_mut(area_id) {
                    block_node.area.geometry.height = content_height;
                }
                if content_height > line_height {
                    multi_col.column_y += content_height - line_height;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::area::{Area, AreaTree, AreaType};
    use crate::layout::PageNumberResolver;
    use fop_core::{FoArena, FoNode, FoNodeData, PropertyList, PropertyValue};
    use fop_types::{Length, Rect};

    fn make_engine() -> LayoutEngine {
        LayoutEngine::new()
    }

    /// Build a minimal FoArena containing a single `fo:block` with the given properties.
    fn make_block_arena(props: PropertyList) -> (FoArena, NodeId) {
        let mut fo_tree = FoArena::new();
        let node_id = fo_tree.add_node(FoNode::new(FoNodeData::Block { properties: props }));
        (fo_tree, node_id)
    }

    /// Add a dummy page-sized parent to the area tree and return its id.
    fn add_parent(area_tree: &mut AreaTree) -> crate::area::AreaId {
        let page_rect = Rect::new(
            Length::ZERO,
            Length::ZERO,
            Length::from_pt(400.0),
            Length::from_pt(600.0),
        );
        area_tree.add_area(Area::new(AreaType::Page, page_rect))
    }

    // -----------------------------------------------------------------------
    // layout_block — line-height property
    // -----------------------------------------------------------------------

    #[test]
    fn test_layout_block_uses_line_height_property() {
        // font-size=12pt, line-height=18pt → block height must be 18pt
        let mut props = PropertyList::new();
        props.set(
            fop_core::PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(12.0)),
        );
        props.set(
            fop_core::PropertyId::LineHeight,
            PropertyValue::Length(Length::from_pt(18.0)),
        );

        let (fo_tree, node_id) = make_block_arena(props);
        let mut area_tree = AreaTree::new();
        let parent_id = add_parent(&mut area_tree);
        let engine = make_engine();
        let mut resolver = PageNumberResolver::new();

        let result = engine
            .layout_block(
                &fo_tree,
                node_id,
                &mut area_tree,
                parent_id,
                Length::ZERO,
                Length::from_pt(400.0),
                &mut resolver,
            )
            .expect("test: layout_block should succeed");

        let area_id = result.expect("test: layout_block should return an area id");
        let node = area_tree
            .get(area_id)
            .expect("test: area should exist in tree");

        assert_eq!(
            node.area.geometry.height,
            Length::from_pt(18.0),
            "block height must equal line-height (18pt) not font-size (12pt)"
        );
    }

    #[test]
    fn test_layout_block_falls_back_to_font_size_when_line_height_absent() {
        // font-size=14pt, no line-height → block height must be 14pt
        let mut props = PropertyList::new();
        props.set(
            fop_core::PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(14.0)),
        );

        let (fo_tree, node_id) = make_block_arena(props);
        let mut area_tree = AreaTree::new();
        let parent_id = add_parent(&mut area_tree);
        let engine = make_engine();
        let mut resolver = PageNumberResolver::new();

        let result = engine
            .layout_block(
                &fo_tree,
                node_id,
                &mut area_tree,
                parent_id,
                Length::ZERO,
                Length::from_pt(400.0),
                &mut resolver,
            )
            .expect("test: layout_block should succeed");

        let area_id = result.expect("test: layout_block should return an area id");
        let node = area_tree
            .get(area_id)
            .expect("test: area should exist in tree");

        assert_eq!(
            node.area.geometry.height,
            Length::from_pt(14.0),
            "block height must equal font-size (14pt) when no line-height is set"
        );
    }

    #[test]
    fn test_layout_block_defaults_to_12pt_when_neither_set() {
        // no properties → block height must be 12pt
        let props = PropertyList::new();

        let (fo_tree, node_id) = make_block_arena(props);
        let mut area_tree = AreaTree::new();
        let parent_id = add_parent(&mut area_tree);
        let engine = make_engine();
        let mut resolver = PageNumberResolver::new();

        let result = engine
            .layout_block(
                &fo_tree,
                node_id,
                &mut area_tree,
                parent_id,
                Length::ZERO,
                Length::from_pt(400.0),
                &mut resolver,
            )
            .expect("test: layout_block should succeed");

        let area_id = result.expect("test: layout_block should return an area id");
        let node = area_tree
            .get(area_id)
            .expect("test: area should exist in tree");

        assert_eq!(
            node.area.geometry.height,
            Length::from_pt(12.0),
            "block height must default to 12pt when no font-size or line-height is set"
        );
    }

    #[test]
    fn test_layout_block_unitless_line_height_multiplier() {
        // font-size=10pt, line-height=1.5 (unitless) → block height must be ~15pt
        // extract_line_height resolves the multiplier against font-size in the properties
        let mut props = PropertyList::new();
        props.set(
            fop_core::PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(10.0)),
        );
        props.set(fop_core::PropertyId::LineHeight, PropertyValue::Number(1.5));

        let (fo_tree, node_id) = make_block_arena(props);
        let mut area_tree = AreaTree::new();
        let parent_id = add_parent(&mut area_tree);
        let engine = make_engine();
        let mut resolver = PageNumberResolver::new();

        let result = engine
            .layout_block(
                &fo_tree,
                node_id,
                &mut area_tree,
                parent_id,
                Length::ZERO,
                Length::from_pt(400.0),
                &mut resolver,
            )
            .expect("test: layout_block should succeed");

        let area_id = result.expect("test: layout_block should return an area id");
        let node = area_tree
            .get(area_id)
            .expect("test: area should exist in tree");

        let height_pt = node.area.geometry.height.to_pt();
        assert!(
            (height_pt - 15.0).abs() < 0.5,
            "block height should be ~15pt for font-size=10pt, line-height=1.5 (unitless), got {}pt",
            height_pt
        );
    }

    // -----------------------------------------------------------------------
    // layout_block_multicolumn — line-height property
    // -----------------------------------------------------------------------

    #[test]
    fn test_layout_block_multicolumn_uses_line_height_property() {
        // font-size=12pt, line-height=18pt → block height must be 18pt in multi-col path
        let mut props = PropertyList::new();
        props.set(
            fop_core::PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(12.0)),
        );
        props.set(
            fop_core::PropertyId::LineHeight,
            PropertyValue::Length(Length::from_pt(18.0)),
        );

        let (fo_tree, node_id) = make_block_arena(props);
        let mut area_tree = AreaTree::new();
        let parent_id = add_parent(&mut area_tree);
        let engine = make_engine();
        let mut resolver = PageNumberResolver::new();

        let mut multi_col =
            MultiColumnLayout::new(2, Length::from_pt(12.0), Length::from_pt(400.0));

        let result = engine
            .layout_block_multicolumn(
                &fo_tree,
                node_id,
                &mut area_tree,
                parent_id,
                &mut multi_col,
                &mut resolver,
            )
            .expect("test: layout_block_multicolumn should succeed");

        let area_id = result.expect("test: layout_block_multicolumn should return an area id");
        let node = area_tree
            .get(area_id)
            .expect("test: area should exist in tree");

        assert_eq!(
            node.area.geometry.height,
            Length::from_pt(18.0),
            "multicolumn block height must equal line-height (18pt) not font-size (12pt)"
        );
    }

    #[test]
    fn test_layout_block_multicolumn_falls_back_to_font_size() {
        // font-size=16pt, no line-height → block height must be 16pt in multi-col path
        let mut props = PropertyList::new();
        props.set(
            fop_core::PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(16.0)),
        );

        let (fo_tree, node_id) = make_block_arena(props);
        let mut area_tree = AreaTree::new();
        let parent_id = add_parent(&mut area_tree);
        let engine = make_engine();
        let mut resolver = PageNumberResolver::new();

        let mut multi_col =
            MultiColumnLayout::new(2, Length::from_pt(12.0), Length::from_pt(400.0));

        let result = engine
            .layout_block_multicolumn(
                &fo_tree,
                node_id,
                &mut area_tree,
                parent_id,
                &mut multi_col,
                &mut resolver,
            )
            .expect("test: layout_block_multicolumn should succeed");

        let area_id = result.expect("test: layout_block_multicolumn should return an area id");
        let node = area_tree
            .get(area_id)
            .expect("test: area should exist in tree");

        assert_eq!(
            node.area.geometry.height,
            Length::from_pt(16.0),
            "multicolumn block height must equal font-size (16pt) when no line-height is set"
        );
    }
}
