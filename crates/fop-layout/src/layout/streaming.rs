//! Streaming layout engine for processing large documents efficiently
//!
//! This module provides a streaming layout engine that processes pages incrementally,
//! yielding one page at a time to minimize memory usage for large documents (1000+ pages).

use crate::area::{Area, AreaId, AreaTree, AreaType, TraitSet};
use crate::layout::{
    extract_break_after, extract_break_before, extract_column_count, extract_column_gap,
    extract_keep_constraint, extract_space_after, extract_space_before, extract_traits,
    BlockLayoutContext, BreakValue, KnuthPlassBreaker, MultiColumnLayout, PageBreaker,
    PageNumberResolver, TextAlign,
};
use fop_core::{FoArena, FoNodeData, NodeId, PropertyId};
use fop_types::{FontRegistry, Length, Point, Rect, Result, Size};

/// Configuration for streaming layout
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Maximum number of pages to buffer in memory
    /// Default: 10 pages
    pub max_memory_pages: usize,

    /// Default page width (A4)
    pub page_width: Length,

    /// Default page height (A4)
    pub page_height: Length,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            max_memory_pages: 10,
            page_width: Length::from_mm(210.0),
            page_height: Length::from_mm(297.0),
        }
    }
}

/// Streaming layout engine that yields pages one at a time
pub struct StreamingLayoutEngine {
    config: StreamingConfig,
    #[allow(dead_code)]
    font_registry: FontRegistry,
}

impl StreamingLayoutEngine {
    /// Create a new streaming layout engine with default configuration
    pub fn new() -> Self {
        Self::with_config(StreamingConfig::default())
    }

    /// Create a new streaming layout engine with custom configuration
    pub fn with_config(config: StreamingConfig) -> Self {
        Self {
            config,
            font_registry: FontRegistry::new(),
        }
    }

    /// Layout an FO tree in streaming mode, yielding one page at a time
    ///
    /// This method returns an iterator that produces one AreaTree per page.
    /// Each AreaTree contains only the areas for that single page and can be
    /// immediately rendered and discarded to minimize memory usage.
    pub fn layout_streaming<'a, 'b>(
        &'a self,
        fo_tree: &'b FoArena<'b>,
    ) -> StreamingLayoutIterator<'a, 'b> {
        StreamingLayoutIterator::new(self, fo_tree)
    }

    /// Resolve the region-body rectangle used for the flow content.
    ///
    /// Mirrors the streaming engine's fixed 1-inch page margins (the historic
    /// streaming geometry) so existing page/area expectations are preserved while
    /// the body height becomes the overflow budget for pagination.
    fn body_rect(&self) -> Rect {
        Rect::from_point_size(
            Point::new(Length::from_pt(72.0), Length::from_pt(72.0)),
            Size::new(
                self.config.page_width - Length::from_pt(144.0),
                self.config.page_height - Length::from_pt(144.0),
            ),
        )
    }

    /// Build a [`PageBreaker`] matching this engine's page geometry, used for the
    /// keep-constraint break decisions during single-column overflow migration
    /// (mirrors `LayoutEngine::page_breaker_for`).
    fn page_breaker(&self) -> PageBreaker {
        PageBreaker::new(
            self.config.page_width,
            self.config.page_height,
            [
                Length::from_pt(72.0),
                Length::from_pt(72.0),
                Length::from_pt(72.0),
                Length::from_pt(72.0),
            ],
        )
    }

    /// Create a fresh page area-tree: a `Page` root carrying any page-sequence
    /// background, with an empty region-body child ready to receive flow content.
    /// Returns the `(tree, page_id, region_id)`.
    fn new_page(&self, page_traits: &TraitSet, body_traits: &TraitSet) -> Result<PageScaffold> {
        let mut tree = AreaTree::new();
        let page_rect = Rect::from_point_size(
            Point::ZERO,
            Size::new(self.config.page_width, self.config.page_height),
        );
        let page_area = Area::new(AreaType::Page, page_rect).with_traits(page_traits.clone());
        let page_id = tree.add_area(page_area);

        let region = Area::new(AreaType::Region, self.body_rect()).with_traits(body_traits.clone());
        let region_id = tree.add_area(region);
        tree.append_child(page_id, region_id)
            .map_err(fop_types::FopError::Generic)?;

        Ok(PageScaffold {
            tree,
            page_id,
            region_id,
        })
    }

    /// Layout a single page sequence and yield its pages.
    ///
    /// This now performs height-overflow pagination (single-column flows) and
    /// multi-column (`column-count > 1`) pagination, mirroring the main engine's
    /// behaviour (`engine::pagination`): a single-column block that would overflow
    /// the region-body starts a new page (migrating any keep-glued trailing
    /// group); a multi-column flow fills columns left-to-right and starts a new
    /// page once the last column is full.
    fn layout_page_sequence<'b>(
        &self,
        fo_tree: &FoArena<'b>,
        page_seq_id: NodeId,
        resolver: &mut PageNumberResolver,
    ) -> Result<Vec<AreaTree>> {
        // Get the page-sequence node.
        let page_seq_node = fo_tree
            .get(page_seq_id)
            .ok_or_else(|| fop_types::FopError::Generic("Page sequence not found".to_string()))?;

        let properties = match &page_seq_node.data {
            FoNodeData::PageSequence { properties, .. } => properties,
            _ => return Ok(Vec::new()),
        };

        let mut page_traits = TraitSet::default();
        if let Ok(color) = properties.get(PropertyId::BackgroundColor) {
            page_traits.background_color = color.as_color();
        }

        // Find the principal flow and derive the body text colour from it.
        let mut flow_id = None;
        let mut body_traits = TraitSet::default();
        for child_id in fo_tree.children(page_seq_id) {
            if let Some(child) = fo_tree.get(child_id) {
                if let FoNodeData::Flow { properties, .. } = &child.data {
                    if let Ok(color) = properties.get(PropertyId::Color) {
                        body_traits.color = color.as_color();
                    }
                    flow_id = Some(child_id);
                    break;
                }
            }
        }

        // The first page of the sequence; even an empty / flow-less sequence
        // produces exactly one page (matching the previous behaviour and tests).
        let first = self.new_page(&page_traits, &body_traits)?;
        let first_page_id = first.page_id;

        // Register the page-sequence's id against its first page.
        if let Some(id) = &page_seq_node.id {
            resolver.register_element(id.clone(), first_page_id);
        }

        let mut pages: Vec<PageScaffold> = vec![first];

        if let Some(flow_node_id) = flow_id {
            self.paginate_flow(
                fo_tree,
                flow_node_id,
                &mut pages,
                &page_traits,
                &body_traits,
            )?;
        }

        // Advance the page counter past this sequence's pages.
        resolver.set_current_page(resolver.current_page() + pages.len());

        Ok(pages.into_iter().map(|scaffold| scaffold.tree).collect())
    }

    /// Dispatch a flow's block children to either single-column overflow
    /// pagination or multi-column pagination based on `column-count`.
    fn paginate_flow<'b>(
        &self,
        fo_tree: &FoArena<'b>,
        flow_node_id: NodeId,
        pages: &mut Vec<PageScaffold>,
        page_traits: &TraitSet,
        body_traits: &TraitSet,
    ) -> Result<()> {
        let flow_node = fo_tree.get(flow_node_id).ok_or_else(|| {
            fop_types::FopError::Generic(format!("Flow node {} not found", flow_node_id))
        })?;
        let (column_count, column_gap) = match &flow_node.data {
            FoNodeData::Flow { properties, .. } => (
                extract_column_count(properties),
                extract_column_gap(properties),
            ),
            _ => return Ok(()),
        };

        let children = fo_tree.children(flow_node_id);

        if column_count > 1 {
            self.paginate_multicolumn_flow(
                fo_tree,
                &children,
                column_count,
                column_gap,
                pages,
                page_traits,
                body_traits,
            )
        } else {
            self.paginate_single_column_flow(fo_tree, &children, pages, page_traits, body_traits)
        }
    }

    /// Single-column height-overflow pagination.
    ///
    /// Blocks are stacked in the current page's region-body while tracking the
    /// cumulative height against the body box.  When the next block would overflow
    /// the body (and the page already holds content), a new page is started and
    /// the overflowing block — together with any preceding blocks glued to it by
    /// `keep-with-previous` / `keep-with-next` constraints — is re-emitted at the
    /// top of the new page (mirroring `paginate_flow`'s keep-group migration).
    /// Forced `break-before` / `break-after` page breaks also start a fresh page.
    fn paginate_single_column_flow<'b>(
        &self,
        fo_tree: &FoArena<'b>,
        children: &[NodeId],
        pages: &mut Vec<PageScaffold>,
        page_traits: &TraitSet,
        body_traits: &TraitSet,
    ) -> Result<()> {
        let body_rect = self.body_rect();
        let body_width = body_rect.width;
        let body_height = body_rect.height;
        let breaker = self.page_breaker();

        // Blocks placed on the *current* page, in order, recorded with their
        // source FO node so a trailing keep-group can be re-emitted onto a new
        // page on overflow.
        let mut page_blocks: Vec<PlacedBlock> = Vec::new();
        let mut current_y = Length::ZERO;

        for &child_id in children {
            let Some(metrics) = self.measure_block(fo_tree, child_id, body_width) else {
                continue;
            };

            // Forced break-before: start a fresh page before this block (only when
            // the current page already has content).
            if metrics.break_before.forces_page_break() && !page_blocks.is_empty() {
                self.start_new_page(pages, page_traits, body_traits)?;
                page_blocks.clear();
                current_y = Length::ZERO;
            }

            // Emit the block into the current page at the stacking cursor.
            let region_id = pages
                .last()
                .map(|p| p.region_id)
                .ok_or_else(|| fop_types::FopError::Generic("No current page".to_string()))?;
            let block_id = {
                let tree = &mut pages
                    .last_mut()
                    .ok_or_else(|| fop_types::FopError::Generic("No current page".to_string()))?
                    .tree;
                self.emit_block(fo_tree, child_id, tree, region_id, current_y, body_width)?
            };

            if let Some(block_id) = block_id {
                let block_bottom = current_y + metrics.total_height;
                page_blocks.push(PlacedBlock {
                    area_id: block_id,
                    node_id: child_id,
                });

                // Overflow: the block's bottom exceeds the body box.  Migrate the
                // trailing keep-group to a fresh page (unless the page held only
                // this single block, in which case it stays — it cannot fit
                // anywhere better, matching the engine's "sole block on its own
                // page" terminal case).
                if block_bottom > body_height && page_blocks.len() > 1 {
                    current_y = self.migrate_overflow_group(
                        fo_tree,
                        &mut page_blocks,
                        pages,
                        page_traits,
                        body_traits,
                        &breaker,
                        body_width,
                    )?;
                } else {
                    current_y = block_bottom;
                }
            }

            // Forced break-after: subsequent content starts on a fresh page.
            if metrics.break_after.forces_page_break() {
                self.start_new_page(pages, page_traits, body_traits)?;
                page_blocks.clear();
                current_y = Length::ZERO;
            }
        }

        Ok(())
    }

    /// Handle a single-column overflow: determine the trailing keep-group whose
    /// last member overflowed, then start a new page and re-emit that group at its
    /// body top.  Returns the new page's stacking cursor (`current_y`).
    ///
    /// Because each streaming page is an independent area-tree, the migrated areas
    /// cannot be reparented across trees, and the public area-tree API offers no
    /// child removal.  The just-completed page is therefore **rebuilt** from the
    /// node ids of the blocks that stay (re-emitting them into a fresh tree in
    /// place), which both drops the migrated head areas cleanly and keeps the
    /// surviving page geometry identical to a page that was filled directly.
    #[allow(clippy::too_many_arguments)]
    fn migrate_overflow_group<'b>(
        &self,
        fo_tree: &FoArena<'b>,
        page_blocks: &mut Vec<PlacedBlock>,
        pages: &mut Vec<PageScaffold>,
        page_traits: &TraitSet,
        body_traits: &TraitSet,
        breaker: &PageBreaker,
        body_width: Length,
    ) -> Result<Length> {
        // Find the start of the trailing keep-group: the longest suffix of
        // `page_blocks` glued together by keep constraints (mirrors
        // `keep_group_start`).
        let area_ids: Vec<AreaId> = page_blocks.iter().map(|b| b.area_id).collect();
        let current_tree = &pages
            .last()
            .ok_or_else(|| fop_types::FopError::Generic("No current page".to_string()))?
            .tree;
        let mut group_start = keep_group_start(breaker, current_tree, &area_ids);
        // If the whole page is one glued group we cannot honour the keep without
        // looping forever; move the last block alone so layout makes progress.
        if group_start == 0 {
            group_start = page_blocks.len() - 1;
        }

        // The source nodes of the blocks that stay, and those that migrate.
        let staying: Vec<NodeId> = page_blocks[..group_start]
            .iter()
            .map(|b| b.node_id)
            .collect();
        let migrating: Vec<NodeId> = page_blocks[group_start..]
            .iter()
            .map(|b| b.node_id)
            .collect();

        // Rebuild the just-completed page from the `staying` blocks so the
        // migrated head areas leave no dangling nodes behind.
        {
            let rebuilt = self.new_page(page_traits, body_traits)?;
            let region_id = rebuilt.region_id;
            let last = pages
                .last_mut()
                .ok_or_else(|| fop_types::FopError::Generic("No current page".to_string()))?;
            *last = rebuilt;
            let mut staying_blocks = Vec::with_capacity(staying.len());
            let mut y = Length::ZERO;
            for node_id in &staying {
                let Some(metrics) = self.measure_block(fo_tree, *node_id, body_width) else {
                    continue;
                };
                if let Some(block_id) =
                    self.emit_block(fo_tree, *node_id, &mut last.tree, region_id, y, body_width)?
                {
                    staying_blocks.push(PlacedBlock {
                        area_id: block_id,
                        node_id: *node_id,
                    });
                    y += metrics.total_height;
                }
            }
            *page_blocks = staying_blocks;
        }

        // Start the new page and re-emit the migrating group at its body top.
        self.start_new_page(pages, page_traits, body_traits)?;
        let new_region_id = pages
            .last()
            .map(|p| p.region_id)
            .ok_or_else(|| fop_types::FopError::Generic("No current page".to_string()))?;
        let mut current_y = Length::ZERO;
        page_blocks.clear();
        for node_id in migrating {
            let Some(metrics) = self.measure_block(fo_tree, node_id, body_width) else {
                continue;
            };
            let tree = &mut pages
                .last_mut()
                .ok_or_else(|| fop_types::FopError::Generic("No current page".to_string()))?
                .tree;
            if let Some(block_id) =
                self.emit_block(fo_tree, node_id, tree, new_region_id, current_y, body_width)?
            {
                page_blocks.push(PlacedBlock {
                    area_id: block_id,
                    node_id,
                });
                current_y += metrics.total_height;
            }
        }

        Ok(current_y)
    }

    /// Multi-column height-overflow pagination.
    ///
    /// Mirrors `paginate_multicolumn_flow`: columns of the current page fill left
    /// to right; when a block does not fit in the current column the cursor
    /// advances to the next column, and when the **last** column of the page is
    /// full a new page is started and the flow resumes in the first column of the
    /// new page's region-body.  Forced page breaks start a fresh page; forced
    /// column breaks advance to the next column (or a new page on the last column).
    ///
    /// Newspaper-style final-page balancing is **not** performed here — see the
    /// module documentation residual note.
    #[allow(clippy::too_many_arguments)]
    fn paginate_multicolumn_flow<'b>(
        &self,
        fo_tree: &FoArena<'b>,
        children: &[NodeId],
        column_count: i32,
        column_gap: Length,
        pages: &mut Vec<PageScaffold>,
        page_traits: &TraitSet,
        body_traits: &TraitSet,
    ) -> Result<()> {
        let body_rect = self.body_rect();
        let mut multi_col = MultiColumnLayout::new(column_count, column_gap, body_rect.width)
            .with_max_height(body_rect.height);
        let column_width = multi_col.column_width();

        for &child_id in children {
            let Some(metrics) = self.measure_block(fo_tree, child_id, column_width) else {
                // Non-block children take no part in column flow-control.
                continue;
            };

            // Forced break-before: a page break starts a fresh page (only when the
            // current page already holds content); a column break advances to the
            // next column (a new page when the last column is reached).
            if metrics.break_before.forces_page_break() && multicolumn_page_has_content(&multi_col)
            {
                self.start_new_page(pages, page_traits, body_traits)?;
                multi_col.reset();
            } else if matches!(metrics.break_before, BreakValue::Column)
                && multi_col.column_y > Length::ZERO
            {
                self.advance_multicolumn(&mut multi_col, pages, page_traits, body_traits)?;
            }

            // Apply space-before, then decide the column fit (matches the engine's
            // single-page column placement, extended so a full last column starts a
            // new page rather than overflowing).
            multi_col.allocate(metrics.space_before);
            let total_height = metrics.space_before + metrics.line_height + metrics.space_after;
            if multi_col.is_column_filled(total_height) {
                self.advance_multicolumn(&mut multi_col, pages, page_traits, body_traits)?;
            }

            // Place the block in the current column of the current page.
            let region_id = pages
                .last()
                .map(|p| p.region_id)
                .ok_or_else(|| fop_types::FopError::Generic("No current page".to_string()))?;
            let column_x = multi_col.current_column_x();
            let column_y = multi_col.column_y;
            {
                let tree = &mut pages
                    .last_mut()
                    .ok_or_else(|| fop_types::FopError::Generic("No current page".to_string()))?
                    .tree;
                self.emit_block_at(
                    fo_tree,
                    child_id,
                    tree,
                    region_id,
                    column_x,
                    column_y,
                    column_width,
                )?;
            }
            // Advance the column cursor by the block content + trailing space.
            multi_col.allocate(metrics.line_height + metrics.space_after);

            // Forced break-after: a page break starts a fresh page; a column break
            // advances to the next column.
            if metrics.break_after.forces_page_break() {
                self.start_new_page(pages, page_traits, body_traits)?;
                multi_col.reset();
            } else if matches!(metrics.break_after, BreakValue::Column) {
                self.advance_multicolumn(&mut multi_col, pages, page_traits, body_traits)?;
            }
        }

        Ok(())
    }

    /// Advance the multi-column cursor to the next column, or — when the current
    /// column is the last one — start a new page and reset to its first column
    /// (mirrors `LayoutEngine::advance_multicolumn`).
    fn advance_multicolumn(
        &self,
        multi_col: &mut MultiColumnLayout,
        pages: &mut Vec<PageScaffold>,
        page_traits: &TraitSet,
        body_traits: &TraitSet,
    ) -> Result<()> {
        if !multi_col.next_column() {
            self.start_new_page(pages, page_traits, body_traits)?;
            multi_col.reset();
        }
        Ok(())
    }

    /// Append a fresh page to `pages`.
    fn start_new_page(
        &self,
        pages: &mut Vec<PageScaffold>,
        page_traits: &TraitSet,
        body_traits: &TraitSet,
    ) -> Result<()> {
        let page = self.new_page(page_traits, body_traits)?;
        pages.push(page);
        Ok(())
    }

    /// Resolve the flow-control measurements of a block child without committing
    /// any areas, so the destination page/column can be chosen before emission.
    ///
    /// Returns `None` for non-block children (which take no part in flow control).
    /// The line count, and therefore `total_height`, is computed with exactly the
    /// same Knuth-Plass breaking used by [`Self::emit_block`], so the measured
    /// height matches what is emitted.
    fn measure_block<'b>(
        &self,
        fo_tree: &FoArena<'b>,
        node_id: NodeId,
        available_width: Length,
    ) -> Option<BlockMetrics> {
        let node = fo_tree.get(node_id)?;
        let properties = match &node.data {
            FoNodeData::Block { properties } => properties,
            _ => return None,
        };

        let traits = extract_traits(properties);
        let space_before = extract_space_before(properties);
        let space_after = extract_space_after(properties);
        let line_height = traits.font_size.unwrap_or(Length::from_pt(12.0));
        let text_align = traits.text_align.unwrap_or(TextAlign::Left);
        let justify = matches!(text_align, TextAlign::Justify);

        // Count the wrapped lines exactly as emission will.
        let mut line_count = 0usize;
        for child_id in fo_tree.children(node_id) {
            if let Some(child_node) = fo_tree.get(child_id) {
                if let FoNodeData::Text(text) = &child_node.data {
                    let mut text_traits = traits.clone();
                    text_traits.text_align = Some(text_align);
                    let breaker = KnuthPlassBreaker::new(available_width).with_justify(justify);
                    line_count += breaker.break_into_lines(text, &text_traits).len();
                }
            }
        }

        // One line minimum (matches the emitter's `content_y.max(line_height)`).
        let content_height = line_height * (line_count.max(1) as i32);
        Some(BlockMetrics {
            line_height,
            content_height,
            space_before,
            space_after,
            total_height: space_before + content_height + space_after,
            break_before: extract_break_before(properties),
            break_after: extract_break_after(properties),
        })
    }

    /// Emit a block at `y_offset` (x = 0) under `parent_area`, returning the block
    /// area id, or `None` for a non-block node.
    fn emit_block<'b>(
        &self,
        fo_tree: &FoArena<'b>,
        node_id: NodeId,
        area_tree: &mut AreaTree,
        parent_area: AreaId,
        y_offset: Length,
        available_width: Length,
    ) -> Result<Option<AreaId>> {
        self.emit_block_at(
            fo_tree,
            node_id,
            area_tree,
            parent_area,
            Length::ZERO,
            y_offset,
            available_width,
        )
    }

    /// Emit a block-level element at `(x_offset, y_offset)` under `parent_area`.
    ///
    /// Lays the block's text out with Knuth-Plass line breaking, stacking one area
    /// per line, then resolves the block's content height.  Shared by the single
    /// column and multi-column paths (the latter passes the column origin as
    /// `x_offset`).
    #[allow(clippy::too_many_arguments)]
    fn emit_block_at<'b>(
        &self,
        fo_tree: &FoArena<'b>,
        node_id: NodeId,
        area_tree: &mut AreaTree,
        parent_area: AreaId,
        x_offset: Length,
        y_offset: Length,
        available_width: Length,
    ) -> Result<Option<AreaId>> {
        let node = fo_tree
            .get(node_id)
            .ok_or_else(|| fop_types::FopError::Generic(format!("Node {} not found", node_id)))?;

        let properties = match &node.data {
            FoNodeData::Block { properties } => properties,
            _ => return Ok(None),
        };

        let traits = extract_traits(properties);
        let space_before = extract_space_before(properties);
        let space_after = extract_space_after(properties);
        let line_height = traits.font_size.unwrap_or(Length::from_pt(12.0));

        let mut block_ctx = BlockLayoutContext::new(available_width);
        block_ctx.current_y = y_offset;
        let mut block_rect = block_ctx.allocate_with_spacing(
            available_width,
            line_height,
            space_before,
            space_after,
        );
        block_rect.x += x_offset;

        let area = Area::new(AreaType::Block, block_rect).with_traits(traits.clone());
        let area_id = area_tree.add_area(area);
        area_tree
            .append_child(parent_area, area_id)
            .map_err(fop_types::FopError::Generic)?;

        let text_align = traits.text_align.unwrap_or(TextAlign::Left);
        let justify = matches!(text_align, TextAlign::Justify);

        let children = fo_tree.children(node_id);
        let mut content_y = Length::ZERO;
        for child_id in children {
            if let Some(child_node) = fo_tree.get(child_id) {
                if let FoNodeData::Text(text) = &child_node.data {
                    let mut text_traits = traits.clone();
                    text_traits.text_align = Some(text_align);

                    let breaker = KnuthPlassBreaker::new(available_width).with_justify(justify);
                    let lines = breaker.break_into_lines(text, &text_traits);
                    let line_count = lines.len();
                    for (i, line) in lines.iter().enumerate() {
                        let natural = line.natural_width;
                        let is_last = i + 1 == line_count;
                        let (x, width) = match text_align {
                            TextAlign::Left => (Length::ZERO, natural),
                            TextAlign::Right => (available_width - natural, natural),
                            TextAlign::Center => ((available_width - natural) / 2, natural),
                            TextAlign::Justify => {
                                if is_last {
                                    (Length::ZERO, natural)
                                } else {
                                    (Length::ZERO, available_width)
                                }
                            }
                        };

                        let text_rect = Rect::new(x, content_y, width, line_height);
                        let text_area = Area::text(text_rect, line.text.clone())
                            .with_traits(text_traits.clone());
                        let text_id = area_tree.add_area(text_area);
                        area_tree
                            .append_child(area_id, text_id)
                            .map_err(fop_types::FopError::Generic)?;

                        content_y += line_height;
                    }
                }
            }
        }

        // Resolve the block's true content height (one line minimum).
        let content_height = content_y.max(line_height);
        if let Some(block_node) = area_tree.get_mut(area_id) {
            block_node.area.geometry.height = content_height;
            // Carry the keep constraint so overflow keep-group detection works.
            let keep = extract_keep_constraint(properties);
            block_node.area.keep_constraint = Some(keep);
        }

        Ok(Some(area_id))
    }
}

/// A page under construction: its independent area tree plus the ids of its page
/// root and region-body, so the paginator can append flow content lazily.
struct PageScaffold {
    tree: AreaTree,
    page_id: AreaId,
    region_id: AreaId,
}

/// A block placed on the current page, recorded with its source FO node so a
/// trailing keep-group can be re-emitted on overflow.
#[derive(Clone, Copy)]
struct PlacedBlock {
    area_id: AreaId,
    node_id: NodeId,
}

/// The flow-control measurements of a block, resolved up front so the
/// destination page/column is chosen before the block is emitted.
struct BlockMetrics {
    line_height: Length,
    #[allow(dead_code)]
    content_height: Length,
    space_before: Length,
    space_after: Length,
    total_height: Length,
    break_before: BreakValue,
    break_after: BreakValue,
}

/// Whether the current multi-column page already holds content.  A forced page
/// break before the very first block of a page (first column, top) must be a
/// no-op (mirrors `engine::pagination::multicolumn_page_has_content`).
fn multicolumn_page_has_content(multi_col: &MultiColumnLayout) -> bool {
    multi_col.current_column > 0 || multi_col.column_y > Length::ZERO
}

/// Find the smallest index `s` such that `blocks[s..]` must move to the next page
/// together (mirrors `engine::pagination::keep_group_start`): for every `k` in
/// `(s, len)` a break before `blocks[k]` is forbidden by a keep constraint.
fn keep_group_start(breaker: &PageBreaker, area_tree: &AreaTree, blocks: &[AreaId]) -> usize {
    let n = blocks.len();
    if n == 0 {
        return 0;
    }
    let mut start = n - 1;
    while start > 0 {
        if breaker.can_break_before(area_tree, blocks[start], start, blocks) {
            break;
        }
        start -= 1;
    }
    start
}

impl Default for StreamingLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator that yields one page at a time from an FO tree
pub struct StreamingLayoutIterator<'a, 'b> {
    engine: &'a StreamingLayoutEngine,
    fo_tree: &'b FoArena<'b>,
    resolver: PageNumberResolver,
    page_sequences: Vec<NodeId>,
    current_seq_index: usize,
    current_page_buffer: Vec<AreaTree>,
    current_page_index: usize,
}

impl<'a, 'b> StreamingLayoutIterator<'a, 'b> {
    fn new(engine: &'a StreamingLayoutEngine, fo_tree: &'b FoArena<'b>) -> Self {
        // Collect all page-sequence nodes
        let mut page_sequences = Vec::new();
        if let Some((root_id, _)) = fo_tree.root() {
            for child_id in fo_tree.children(root_id) {
                if let Some(child) = fo_tree.get(child_id) {
                    if matches!(child.data, FoNodeData::PageSequence { .. }) {
                        page_sequences.push(child_id);
                    }
                }
            }
        }

        Self {
            engine,
            fo_tree,
            resolver: PageNumberResolver::new(),
            page_sequences,
            current_seq_index: 0,
            current_page_buffer: Vec::new(),
            current_page_index: 0,
        }
    }

    fn load_next_batch(&mut self) -> Result<bool> {
        if self.current_seq_index >= self.page_sequences.len() {
            return Ok(false);
        }

        let seq_id = self.page_sequences[self.current_seq_index];
        self.current_page_buffer =
            self.engine
                .layout_page_sequence(self.fo_tree, seq_id, &mut self.resolver)?;
        self.current_page_index = 0;
        self.current_seq_index += 1;

        Ok(!self.current_page_buffer.is_empty())
    }
}

impl<'a, 'b> Iterator for StreamingLayoutIterator<'a, 'b> {
    type Item = Result<AreaTree>;

    fn next(&mut self) -> Option<Self::Item> {
        // Check if we have pages in the current buffer
        if self.current_page_index < self.current_page_buffer.len() {
            let page = self.current_page_buffer.remove(0);
            return Some(Ok(page));
        }

        // Load next batch of pages
        match self.load_next_batch() {
            Ok(true) => {
                if !self.current_page_buffer.is_empty() {
                    let page = self.current_page_buffer.remove(0);
                    Some(Ok(page))
                } else {
                    None
                }
            }
            Ok(false) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fop_core::{FoNode, FoNodeData, PropertyList};

    #[test]
    fn test_streaming_engine_creation() {
        let engine = StreamingLayoutEngine::new();
        assert_eq!(engine.config.page_width, Length::from_mm(210.0));
        assert_eq!(engine.config.page_height, Length::from_mm(297.0));
        assert_eq!(engine.config.max_memory_pages, 10);
    }

    #[test]
    fn test_custom_config() {
        let config = StreamingConfig {
            max_memory_pages: 5,
            page_width: Length::from_mm(215.9),  // US Letter width
            page_height: Length::from_mm(279.4), // US Letter height
        };
        let engine = StreamingLayoutEngine::with_config(config);
        assert_eq!(engine.config.max_memory_pages, 5);
        assert_eq!(engine.config.page_width, Length::from_mm(215.9));
    }

    #[test]
    fn test_streaming_layout_empty_tree() {
        let engine = StreamingLayoutEngine::new();
        let fo_tree = FoArena::new();

        let mut iter = engine.layout_streaming(&fo_tree);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_streaming_layout_single_page() {
        let engine = StreamingLayoutEngine::new();
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

        // Layout in streaming mode
        let mut page_count = 0;
        for page_result in engine.layout_streaming(&fo_tree) {
            let page = page_result.expect("test: should succeed");
            assert!(!page.is_empty());
            page_count += 1;
        }

        assert_eq!(page_count, 1);
    }

    #[test]
    fn test_streaming_layout_multiple_pages() {
        let engine = StreamingLayoutEngine::new();
        let mut fo_tree = FoArena::new();

        let root = fo_tree.add_node(FoNode::new(FoNodeData::Root));

        // Create 3 page sequences (simulating 3 pages)
        for i in 0..3 {
            let page_seq = fo_tree.add_node(FoNode::new(FoNodeData::PageSequence {
                master_reference: format!("page-{}", i),
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
        }

        // Layout in streaming mode
        let mut page_count = 0;
        for page_result in engine.layout_streaming(&fo_tree) {
            let page = page_result.expect("test: should succeed");
            assert!(!page.is_empty());
            page_count += 1;
            // Page should be dropped here, freeing memory
        }

        assert_eq!(page_count, 3);
    }

    #[test]
    fn test_memory_bounded_processing() {
        let config = StreamingConfig {
            max_memory_pages: 2, // Very small buffer
            ..Default::default()
        };
        let engine = StreamingLayoutEngine::with_config(config);
        let mut fo_tree = FoArena::new();

        let root = fo_tree.add_node(FoNode::new(FoNodeData::Root));

        // Create 10 page sequences
        for i in 0..10 {
            let page_seq = fo_tree.add_node(FoNode::new(FoNodeData::PageSequence {
                master_reference: format!("page-{}", i),
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
        }

        // Process all pages - memory should stay bounded
        let mut page_count = 0;
        for page_result in engine.layout_streaming(&fo_tree) {
            let _page = page_result.expect("test: should succeed");
            page_count += 1;
        }

        assert_eq!(page_count, 10);
    }
}

#[cfg(test)]
mod extended_tests {
    use super::*;
    use fop_core::{FoArena, FoNode, FoNodeData, PropertyList};

    // ---- Helper ----

    fn make_fo_tree_with_n_page_sequences(n: usize) -> FoArena<'static> {
        let mut fo_tree = FoArena::new();
        let root = fo_tree.add_node(FoNode::new(FoNodeData::Root));
        for i in 0..n {
            let page_seq = fo_tree.add_node(FoNode::new(FoNodeData::PageSequence {
                master_reference: format!("master-{}", i),
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
        }
        fo_tree
    }

    // ---- StreamingConfig tests ----

    #[test]
    fn test_streaming_config_default_page_width_is_a4() {
        let config = StreamingConfig::default();
        assert_eq!(config.page_width, Length::from_mm(210.0));
    }

    #[test]
    fn test_streaming_config_default_page_height_is_a4() {
        let config = StreamingConfig::default();
        assert_eq!(config.page_height, Length::from_mm(297.0));
    }

    #[test]
    fn test_streaming_config_default_max_memory_pages_is_ten() {
        let config = StreamingConfig::default();
        assert_eq!(config.max_memory_pages, 10);
    }

    #[test]
    fn test_streaming_config_clone() {
        let config = StreamingConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.max_memory_pages, config.max_memory_pages);
        assert_eq!(cloned.page_width, config.page_width);
        assert_eq!(cloned.page_height, config.page_height);
    }

    #[test]
    fn test_streaming_config_debug() {
        let config = StreamingConfig::default();
        let dbg = format!("{:?}", config);
        assert!(dbg.contains("StreamingConfig"));
    }

    // ---- StreamingLayoutEngine construction tests ----

    #[test]
    fn test_engine_default_equals_new() {
        let a = StreamingLayoutEngine::new();
        let b = StreamingLayoutEngine::default();
        assert_eq!(a.config.max_memory_pages, b.config.max_memory_pages);
        assert_eq!(a.config.page_width, b.config.page_width);
    }

    #[test]
    fn test_engine_with_custom_page_size_letter() {
        let config = StreamingConfig {
            page_width: Length::from_mm(215.9),
            page_height: Length::from_mm(279.4),
            max_memory_pages: 5,
        };
        let engine = StreamingLayoutEngine::with_config(config);
        assert_eq!(engine.config.page_width, Length::from_mm(215.9));
        assert_eq!(engine.config.page_height, Length::from_mm(279.4));
        assert_eq!(engine.config.max_memory_pages, 5);
    }

    #[test]
    fn test_engine_with_large_memory_limit() {
        let config = StreamingConfig {
            max_memory_pages: 1000,
            ..Default::default()
        };
        let engine = StreamingLayoutEngine::with_config(config);
        assert_eq!(engine.config.max_memory_pages, 1000);
    }

    #[test]
    fn test_engine_with_single_page_memory_limit() {
        let config = StreamingConfig {
            max_memory_pages: 1,
            ..Default::default()
        };
        let engine = StreamingLayoutEngine::with_config(config);
        assert_eq!(engine.config.max_memory_pages, 1);
    }

    // ---- Streaming layout with block content ----

    #[test]
    fn test_layout_page_has_root_area() {
        let engine = StreamingLayoutEngine::new();
        let mut fo_tree = FoArena::new();
        let root = fo_tree.add_node(FoNode::new(FoNodeData::Root));
        let ps = fo_tree.add_node(FoNode::new(FoNodeData::PageSequence {
            master_reference: "A4".to_string(),
            format: "1".to_string(),
            grouping_separator: None,
            grouping_size: None,
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(root, ps)
            .expect("test: should succeed");

        let pages: Vec<_> = engine
            .layout_streaming(&fo_tree)
            .collect::<Result<Vec<_>>>()
            .expect("test: should succeed");
        assert_eq!(pages.len(), 1);
        // Root area should exist in the page tree
        assert!(!pages[0].is_empty());
    }

    #[test]
    fn test_layout_page_area_count_at_least_one() {
        let engine = StreamingLayoutEngine::new();
        let fo_tree = make_fo_tree_with_n_page_sequences(1);
        let pages: Vec<_> = engine
            .layout_streaming(&fo_tree)
            .collect::<Result<Vec<_>>>()
            .expect("test: should succeed");
        assert!(!pages[0].is_empty());
    }

    #[test]
    fn test_layout_five_page_sequences() {
        let engine = StreamingLayoutEngine::new();
        let fo_tree = make_fo_tree_with_n_page_sequences(5);
        let pages: Vec<_> = engine
            .layout_streaming(&fo_tree)
            .collect::<Result<Vec<_>>>()
            .expect("test: should succeed");
        assert_eq!(pages.len(), 5);
    }

    #[test]
    fn test_layout_twenty_page_sequences() {
        let engine = StreamingLayoutEngine::new();
        let fo_tree = make_fo_tree_with_n_page_sequences(20);
        let count = engine
            .layout_streaming(&fo_tree)
            .filter(|r| r.is_ok())
            .count();
        assert_eq!(count, 20);
    }

    #[test]
    fn test_each_page_is_independent() {
        let engine = StreamingLayoutEngine::new();
        let fo_tree = make_fo_tree_with_n_page_sequences(3);

        let pages: Vec<_> = engine
            .layout_streaming(&fo_tree)
            .collect::<Result<Vec<_>>>()
            .expect("test: should succeed");

        // Each page tree is independent – modifying one should not affect others
        assert_eq!(pages.len(), 3);
        for page in &pages {
            assert!(!page.is_empty());
        }
    }

    #[test]
    fn test_streaming_with_block_text() {
        let engine = StreamingLayoutEngine::new();
        let mut fo_tree = FoArena::new();

        let root = fo_tree.add_node(FoNode::new(FoNodeData::Root));
        let ps = fo_tree.add_node(FoNode::new(FoNodeData::PageSequence {
            master_reference: "A4".to_string(),
            format: "1".to_string(),
            grouping_separator: None,
            grouping_size: None,
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(root, ps)
            .expect("test: should succeed");

        let flow = fo_tree.add_node(FoNode::new(FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(ps, flow)
            .expect("test: should succeed");

        let block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(flow, block)
            .expect("test: should succeed");

        let text = fo_tree.add_node(FoNode::new(FoNodeData::Text("Hello, World!".to_string())));
        fo_tree
            .append_child(block, text)
            .expect("test: should succeed");

        let pages: Vec<_> = engine
            .layout_streaming(&fo_tree)
            .collect::<Result<Vec<_>>>()
            .expect("test: should succeed");
        assert_eq!(pages.len(), 1);
        // Text area should be present
        assert!(!pages[0].is_empty());
    }

    #[test]
    fn test_streaming_no_page_sequences_in_root() {
        let engine = StreamingLayoutEngine::new();
        let mut fo_tree = FoArena::new();
        // Only a root with no page sequences
        fo_tree.add_node(FoNode::new(FoNodeData::Root));

        let pages: Vec<_> = engine
            .layout_streaming(&fo_tree)
            .collect::<Result<Vec<_>>>()
            .expect("test: should succeed");
        assert!(pages.is_empty());
    }

    #[test]
    fn test_streaming_page_sequence_without_flow() {
        let engine = StreamingLayoutEngine::new();
        let mut fo_tree = FoArena::new();
        let root = fo_tree.add_node(FoNode::new(FoNodeData::Root));
        let ps = fo_tree.add_node(FoNode::new(FoNodeData::PageSequence {
            master_reference: "A4".to_string(),
            format: "1".to_string(),
            grouping_separator: None,
            grouping_size: None,
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(root, ps)
            .expect("test: should succeed");
        // No flow child – should produce one page but minimal area tree
        let pages: Vec<_> = engine
            .layout_streaming(&fo_tree)
            .collect::<Result<Vec<_>>>()
            .expect("test: should succeed");
        assert_eq!(pages.len(), 1);
    }

    #[test]
    fn test_streaming_iterator_is_lazy() {
        // The iterator should not lay out ALL pages when constructed
        // We can verify by checking that it yields items one by one
        let engine = StreamingLayoutEngine::new();
        let fo_tree = make_fo_tree_with_n_page_sequences(5);
        let mut iter = engine.layout_streaming(&fo_tree);

        // First next() should return Some
        let first = iter.next();
        assert!(first.is_some());
        assert!(first.expect("test: should succeed").is_ok());
    }

    #[test]
    fn test_streaming_iterator_terminates() {
        let engine = StreamingLayoutEngine::new();
        let fo_tree = make_fo_tree_with_n_page_sequences(3);
        let iter = engine.layout_streaming(&fo_tree);

        // Collect all and verify termination
        let results: Vec<_> = iter.collect();
        assert_eq!(results.len(), 3);
        // After exhaustion, all should be Ok
        for r in results {
            assert!(r.is_ok());
        }
    }

    #[test]
    fn test_streaming_zero_page_sequences() {
        let engine = StreamingLayoutEngine::new();
        let fo_tree = FoArena::new();
        let count = engine.layout_streaming(&fo_tree).count();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_streaming_produces_page_area_type() {
        let engine = StreamingLayoutEngine::new();
        let fo_tree = make_fo_tree_with_n_page_sequences(1);
        let pages: Vec<_> = engine
            .layout_streaming(&fo_tree)
            .collect::<Result<Vec<_>>>()
            .expect("test: should succeed");
        let page_tree = &pages[0];
        // Root area of the page tree should be of Page type
        if let Some((_, root_node)) = page_tree.root() {
            assert_eq!(root_node.area.area_type, crate::area::AreaType::Page);
        } else {
            panic!("Expected a root area in page tree");
        }
    }
}

/// Tests for the streaming engine's pagination + multi-column parity with the
/// main engine (`engine::pagination`).
#[cfg(test)]
mod pagination_parity_tests {
    use super::*;
    use crate::area::AreaType;
    use fop_core::{FoArena, FoNode, FoNodeData, PropertyId, PropertyList, PropertyValue};
    use fop_types::Length;

    /// Count the `Block` areas directly under the (single) region-body of a page.
    fn block_count(page: &AreaTree) -> usize {
        let (page_id, _) = page.root().expect("page has a root");
        let mut blocks = 0usize;
        for region_id in page.children(page_id) {
            if let Some(region) = page.get(region_id) {
                if region.area.area_type == AreaType::Region {
                    for child_id in page.children(region_id) {
                        if let Some(child) = page.get(child_id) {
                            if child.area.area_type == AreaType::Block {
                                blocks += 1;
                            }
                        }
                    }
                }
            }
        }
        blocks
    }

    /// All `Block` area geometries (x, y, height) under a page's region-body.
    fn block_geometries(page: &AreaTree) -> Vec<(Length, Length, Length)> {
        let (page_id, _) = page.root().expect("page has a root");
        let mut out = Vec::new();
        for region_id in page.children(page_id) {
            if let Some(region) = page.get(region_id) {
                if region.area.area_type == AreaType::Region {
                    for child_id in page.children(region_id) {
                        if let Some(child) = page.get(child_id) {
                            if child.area.area_type == AreaType::Block {
                                let g = child.area.geometry;
                                out.push((g.x, g.y, child.area.height()));
                            }
                        }
                    }
                }
            }
        }
        out
    }

    /// Build a page-sequence whose flow holds `n` tall blocks (each one
    /// `font_size` pt high), optionally multi-column.
    fn flow_of_tall_blocks(
        n: usize,
        font_size_pt: f64,
        column_count: Option<i32>,
    ) -> FoArena<'static> {
        let mut fo_tree = FoArena::new();
        let root = fo_tree.add_node(FoNode::new(FoNodeData::Root));
        let ps = fo_tree.add_node(FoNode::new(FoNodeData::PageSequence {
            master_reference: "A4".to_string(),
            format: "1".to_string(),
            grouping_separator: None,
            grouping_size: None,
            properties: PropertyList::new(),
        }));
        fo_tree
            .append_child(root, ps)
            .expect("append page-sequence");

        let mut flow_props = PropertyList::new();
        if let Some(cc) = column_count {
            flow_props.set(PropertyId::ColumnCount, PropertyValue::Integer(cc));
        }
        let flow = fo_tree.add_node(FoNode::new(FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: flow_props,
        }));
        fo_tree.append_child(ps, flow).expect("append flow");

        for _ in 0..n {
            let mut block_props = PropertyList::new();
            block_props.set(
                PropertyId::FontSize,
                PropertyValue::Length(Length::from_pt(font_size_pt)),
            );
            let block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
                properties: block_props,
            }));
            fo_tree.append_child(flow, block).expect("append block");
        }
        fo_tree
    }

    #[test]
    fn test_long_flow_streams_as_multiple_pages() {
        // Body height = 297mm - 2*72pt ≈ 697.89pt.  With 200pt-tall blocks, three
        // fit per page (600pt) and a fourth overflows onto a new page.
        let engine = StreamingLayoutEngine::new();
        let fo_tree = flow_of_tall_blocks(7, 200.0, None);

        let pages: Vec<_> = engine
            .layout_streaming(&fo_tree)
            .collect::<Result<Vec<_>>>()
            .expect("streaming layout should succeed");

        // 7 blocks at 3-per-page => 3 pages (3 + 3 + 1).
        assert_eq!(
            pages.len(),
            3,
            "long flow must paginate into multiple pages"
        );
        assert_eq!(block_count(&pages[0]), 3, "page 1 holds 3 blocks");
        assert_eq!(block_count(&pages[1]), 3, "page 2 holds 3 blocks");
        assert_eq!(
            block_count(&pages[2]),
            1,
            "page 3 holds the remaining block"
        );

        // Total blocks across all pages equals the input block count.
        let total: usize = pages.iter().map(block_count).sum();
        assert_eq!(total, 7, "no block is dropped or duplicated");

        // The first block of every page starts at the body top (y = 0); blocks
        // stack downward within a page.
        for page in &pages {
            let geoms = block_geometries(page);
            assert!(!geoms.is_empty());
            assert_eq!(geoms[0].1, Length::ZERO, "first block sits at body top");
            for w in geoms.windows(2) {
                assert!(w[1].1 >= w[0].1, "blocks stack downward");
            }
        }
    }

    #[test]
    fn test_single_block_taller_than_page_stays_on_own_page() {
        // A lone block taller than the body cannot fit anywhere better; it stays
        // on its own page (the engine's terminal "sole block" case).
        let engine = StreamingLayoutEngine::new();
        let fo_tree = flow_of_tall_blocks(1, 1000.0, None);
        let pages: Vec<_> = engine
            .layout_streaming(&fo_tree)
            .collect::<Result<Vec<_>>>()
            .expect("streaming layout should succeed");
        assert_eq!(pages.len(), 1);
        assert_eq!(block_count(&pages[0]), 1);
    }

    #[test]
    fn test_two_column_flow_fills_columns_then_new_page() {
        // Two columns, body height ≈ 697.89pt.  200pt blocks => 3 per column.
        // 7 blocks: col0 (3) + col1 (3) on page 1, the 7th opens page 2 col0.
        let engine = StreamingLayoutEngine::new();
        let fo_tree = flow_of_tall_blocks(7, 200.0, Some(2));

        let pages: Vec<_> = engine
            .layout_streaming(&fo_tree)
            .collect::<Result<Vec<_>>>()
            .expect("streaming layout should succeed");

        assert_eq!(pages.len(), 2, "full two columns then a new page");
        assert_eq!(block_count(&pages[0]), 6, "page 1 fills both columns (3+3)");
        assert_eq!(
            block_count(&pages[1]),
            1,
            "page 2 starts a fresh first column"
        );

        // On page 1, exactly two distinct column x-origins must appear (the flow
        // filled left-to-right across two columns).
        let geoms = block_geometries(&pages[0]);
        let mut xs: Vec<i32> = geoms.iter().map(|(x, _, _)| x.millipoints()).collect();
        xs.sort_unstable();
        xs.dedup();
        assert_eq!(xs.len(), 2, "blocks occupy two columns on the full page");

        // The second column's x-origin is strictly to the right of the first.
        assert!(xs[1] > xs[0], "column 1 is to the right of column 0");

        // Page 2's single block sits in the first column (left origin) at the top.
        let geoms2 = block_geometries(&pages[1]);
        assert_eq!(geoms2.len(), 1);
        assert_eq!(geoms2[0].0.millipoints(), xs[0], "resumes in first column");
        assert_eq!(geoms2[0].1, Length::ZERO, "at the column top");
    }

    #[test]
    fn test_two_column_single_page_when_content_fits() {
        // Four 100pt blocks across two columns: col0 (3 => 300pt) then col1; all
        // fit on one page.
        let engine = StreamingLayoutEngine::new();
        let fo_tree = flow_of_tall_blocks(4, 100.0, Some(2));
        let pages: Vec<_> = engine
            .layout_streaming(&fo_tree)
            .collect::<Result<Vec<_>>>()
            .expect("streaming layout should succeed");
        assert_eq!(pages.len(), 1, "content that fits stays on one page");
        assert_eq!(block_count(&pages[0]), 4);
    }
}
