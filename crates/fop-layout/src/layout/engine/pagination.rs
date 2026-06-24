//! Height-overflow pagination for page-sequence flow content.
//!
//! This module owns the live pagination path used by [`LayoutEngine::layout`].
//! It drives a per-page-sequence loop that:
//!
//! * instantiates pages from the page master geometry,
//! * repeats the static-content regions (header / footer / start / end sidebar)
//!   on every page,
//! * stacks the flow's block content in the region-body while tracking the
//!   cumulative height against the body box,
//! * starts a new page and **reparents** the overflowing block (and any blocks
//!   glued to it by keep constraints) onto the new page's region-body when the
//!   next block would not fit, and
//! * honours forced `break-before` / `break-after` page breaks (including the
//!   even/odd-page variants).
//!
//! Block geometry is stored relative to its parent region, so relocating a
//! block between pages only requires fixing the block's own `y` — its laid-out
//! descendants keep their region-relative positions (see
//! [`AreaTree::reparent`](crate::area::AreaTree::reparent)).
//!
//! Multi-column flows (`column-count > 1`) are paginated by a sibling driver,
//! [`LayoutEngine::paginate_multicolumn_flow`], which fills the columns of each
//! page left to right and starts a new page (reusing the same page-construction
//! machinery, so static content repeats) once the last column is full.  Because
//! the destination page is chosen *before* each block is emitted, multi-column
//! blocks are born under the correct region-body and need no reparenting.
//!
//! Full pages keep this sequential fill (they legitimately fill every column to
//! capacity), but the **final** page is re-balanced newspaper-style so its
//! columns end at roughly equal heights — the ordered blocks are partitioned
//! into `column_count` contiguous groups minimising the tallest column (via a
//! binary search over the target height with a greedy feasibility check) and the
//! already-emitted block areas are repositioned in place.  See
//! [`LayoutEngine::balance_multicolumn_page`].

use crate::area::{Area, AreaId, AreaTree, AreaType, TraitSet};
use crate::layout::PageNumberResolver;
use crate::layout::{
    extract_break_after, extract_break_before, extract_clear, extract_column_count,
    extract_column_gap, extract_space_after, extract_space_before, extract_traits, BreakValue,
    PageBreaker,
};
use fop_core::{FoArena, FoNodeData, NodeId, PropertyId};
use fop_types::{Length, Point, Rect, Result, Size};

use super::markers::{collect_sequence_markers, DocumentMarkerState, PageMarkerView};
use super::types::{FloatManager, MultiColumnLayout, PageContext, PageRegionGeometry};
use super::LayoutEngine;

use std::cell::RefCell;

/// Owned, borrow-free description of everything needed to build each page of a
/// single `fo:page-sequence`. Built once up front so the pagination loop never
/// holds a borrow into the FO tree across the area-tree mutations.
///
/// # Per-page geometry
///
/// `geom` is the **pagination geometry**: the page/region rectangles used to
/// drive flow overflow decisions (body height, the page breaker).  When the
/// page-sequence's `master-reference` names a `fo:simple-page-master` it also
/// *is* every page's geometry (the common, legacy case).
///
/// When the reference names a `fo:page-sequence-master`, `conditional` is
/// `true` and each constructed page additionally resolves its *own*
/// [`PageRegionGeometry`] from the conditional alternative that matches that
/// page's [`PageContext`](super::types::PageContext) — used for the page rect,
/// the region-body area rect and the static-content regions.  `geom` (resolved
/// from the *first* page's selected master) still governs flow pagination, so
/// the forward pass remains stable; this is exactly correct whenever the
/// per-page masters share the first page's body geometry (see the module-level
/// note on the differing-body-geometry residual).
struct SequenceLayout {
    /// Page/region geometry that drives flow pagination (body height + breaker).
    geom: PageRegionGeometry,
    /// The page-sequence's `master-reference` (a simple-page-master *or* a
    /// page-sequence-master name).
    master_reference: String,
    /// `true` when `master_reference` names a `fo:page-sequence-master`, so each
    /// page must resolve its own geometry conditionally.
    conditional: bool,
    /// Per-page resolved geometry, pushed in lock-step with `page_ids` as each
    /// page is constructed.  Index `i` is the geometry of `page_ids[i]`.  In the
    /// non-conditional case every entry equals `geom`.  Wrapped in a `RefCell`
    /// so the page-construction helpers (which only borrow `&seq`) can record
    /// each page's geometry without threading an extra `&mut` parameter through
    /// the whole pagination call graph.
    page_geoms: RefCell<Vec<PageRegionGeometry>>,
    /// Traits applied to every page area (e.g. page background colour).
    page_traits: TraitSet,
    /// Traits applied to every region-body area (e.g. inherited text colour).
    body_traits: TraitSet,
    /// `static-content` node for `xsl-region-before` (header), if present.
    static_before: Option<NodeId>,
    /// `static-content` node for `xsl-region-after` (footer), if present.
    static_after: Option<NodeId>,
    /// `static-content` node for `xsl-region-start` (start sidebar), if present.
    static_start: Option<NodeId>,
    /// `static-content` node for `xsl-region-end` (end sidebar), if present.
    static_end: Option<NodeId>,
}

/// The flow-control measurements of a single multi-column block, pulled off its
/// FO node up front so the multi-column paginator can decide column/page breaks
/// without holding a borrow into the FO tree across area-tree mutations.
///
/// These mirror exactly the values the single-page placement primitive
/// ([`LayoutEngine::emit_multicolumn_block`]) and column flow-control
/// ([`LayoutEngine::layout_block_multicolumn`]) derive, so the paginated and
/// single-page column paths agree on where a column fills.
struct MultiColumnMetrics {
    /// `space-before` applied before the block.
    space_before: Length,
    /// `space-after` applied after the block.
    space_after: Length,
    /// Resolved per-line height (the height reserved for the column-fit check).
    line_height: Length,
    /// Forced `break-before` value.
    break_before: BreakValue,
    /// Forced `break-after` value.
    break_after: BreakValue,
}

/// One block placed on the *current* multi-column page, recorded so the final
/// page can be re-balanced ([`LayoutEngine::balance_multicolumn_page`]).
///
/// `occupied` is the exact vertical extent the block consumed in its column
/// (`space-before` + laid-out content height + `space-after`), captured as the
/// `column_y` delta around its emission.  `forces_column_boundary` is `true`
/// when a mandatory column / page boundary precedes the block (a forced
/// `break-before = column | page | even-page | odd-page`, or the implicit
/// boundary created when a `break-after` ended the previous block); the balanced
/// partition must start a new column at every such block so forced breaks remain
/// honoured.
#[derive(Debug, Clone, Copy)]
struct BalanceEntry {
    /// The emitted block area.
    area_id: AreaId,
    /// Vertical extent the block consumed in its column.
    occupied: Length,
    /// Whether a mandatory column boundary must precede this block.
    forces_column_boundary: bool,
}

impl LayoutEngine {
    /// Lay out a whole `fo:page-sequence`, producing one or more page areas.
    ///
    /// Returns the id of the **first** page area (the page-sequence's principal
    /// area, used for id registration). Subsequent pages are appended to the
    /// area tree as additional top-level `Page` areas.
    pub(in crate::layout::engine) fn layout_page_sequence(
        &self,
        fo_tree: &FoArena,
        node_id: NodeId,
        area_tree: &mut AreaTree,
        resolver: &mut PageNumberResolver,
        doc_markers: &mut DocumentMarkerState,
    ) -> Result<Option<AreaId>> {
        let node = fo_tree
            .get(node_id)
            .ok_or_else(|| fop_types::FopError::Generic(format!("Node {} not found", node_id)))?;

        let (master_reference, node_user_id, page_traits) = match &node.data {
            FoNodeData::PageSequence {
                properties,
                master_reference,
                ..
            } => {
                let mut page_traits = TraitSet::default();
                if let Ok(color) = properties.get(PropertyId::BackgroundColor) {
                    page_traits.background_color = color.as_color();
                }
                (master_reference.clone(), node.id.clone(), page_traits)
            }
            _ => return Ok(None),
        };

        // Decide whether per-page conditional master selection is needed. When
        // `master_reference` names a plain `fo:simple-page-master` this is
        // `false` and the legacy single-geometry path is taken unchanged.
        let conditional = self.is_page_sequence_master(fo_tree, &master_reference);

        // The geometry that drives flow pagination (body height + page breaker).
        // For a plain simple-page-master this is the page's only geometry.  For a
        // page-sequence-master it is the *first* page's geometry: the conditional
        // alternative that matches the first page (page index 0, the opening
        // page number).  PASS 1 paginates against this stable body height; pages
        // whose selected master shares this body geometry (the required case)
        // render exactly correctly, and a differing body geometry is the
        // recorded reflow residual (see the deferred sub-item).
        let first_page_number = resolver.current_page();
        let geom = if conditional {
            let first_ctx = PageContext::for_page(first_page_number, true, false);
            let first_master =
                self.resolve_page_master_for_page(fo_tree, &master_reference, &first_ctx, false);
            self.extract_page_region_geometry(fo_tree, &first_master)
        } else {
            self.extract_page_region_geometry(fo_tree, &master_reference)
        };

        // Classify the page-sequence children into the static-content regions
        // and the principal flow.
        let mut static_before = None;
        let mut static_after = None;
        let mut static_start = None;
        let mut static_end = None;
        let mut flow_id = None;
        for child_id in fo_tree.children(node_id) {
            if let Some(child) = fo_tree.get(child_id) {
                match &child.data {
                    FoNodeData::StaticContent { flow_name, .. } => match flow_name.as_str() {
                        "xsl-region-before" => static_before = Some(child_id),
                        "xsl-region-after" => static_after = Some(child_id),
                        "xsl-region-start" => static_start = Some(child_id),
                        "xsl-region-end" => static_end = Some(child_id),
                        _ => {}
                    },
                    FoNodeData::Flow { .. } => flow_id = Some(child_id),
                    _ => {}
                }
            }
        }

        // Body region inherits the flow's text colour (matching the previous
        // single-page behaviour).
        let mut body_traits = TraitSet::default();
        if let Some(flow_node_id) = flow_id {
            if let Some(flow_node) = fo_tree.get(flow_node_id) {
                if let FoNodeData::Flow { properties, .. } = &flow_node.data {
                    if let Ok(color) = properties.get(PropertyId::Color) {
                        body_traits.color = color.as_color();
                    }
                }
            }
        }

        let seq = SequenceLayout {
            geom,
            master_reference,
            conditional,
            page_geoms: RefCell::new(Vec::new()),
            page_traits,
            body_traits,
            static_before,
            static_after,
            static_start,
            static_end,
        };

        // `first_page_number` is the page number the sequence opens on (the
        // resolver is positioned on it by the previous sequence / its initial
        // value of 1).  Pages are created sequentially incrementing the resolver
        // by exactly one each, so page `i` of this sequence carries page number
        // `first_page_number + i` — used below to re-establish the per-page
        // number when the deferred static content (running headers/footers) is
        // laid out.

        // PASS 1 — flow.  Build the pages and lay out the flow with
        // height-overflow pagination, recording each placed top-level flow item
        // (`placements`) so the markers it carries can later be attributed to
        // the page it actually landed on.  Static content is intentionally NOT
        // laid out yet: a page's running header needs the markers that occur in
        // that page's flow, which are only known once the flow is placed.
        let mut page_ids: Vec<AreaId> = Vec::new();
        let mut placements: Vec<(AreaId, NodeId)> = Vec::new();
        let first_geom = self.resolve_and_record_page_geom(fo_tree, &seq, first_page_number, false);
        let (first_page_id, first_region_id) =
            self.build_page_with_geom(area_tree, &seq, &first_geom)?;
        page_ids.push(first_page_id);

        // Register the page-sequence's own id against its first page.
        if let Some(id) = &node_user_id {
            resolver.register_element(id.clone(), first_page_id);
        }

        if let Some(flow_node_id) = flow_id {
            self.paginate_flow(
                fo_tree,
                flow_node_id,
                area_tree,
                &seq,
                first_region_id,
                &mut page_ids,
                resolver,
                &mut placements,
            )?;
        }

        // `last`-conditional fix-up.  The total page count is now known, so the
        // final page's `last` condition is resolvable.  Re-resolve its master
        // with `is_last = true`; if that selects a *different* geometry whose
        // body matches the forward-pass body (so the already-placed flow stays
        // valid), adopt it for the final page (page rect + region-body rect +
        // recorded geometry, which the static-content/footnote passes consult).
        // A `last` master whose **body geometry differs** would require reflowing
        // the final page's flow into the new body box — out of safe scope this
        // session and left unapplied rather than emitting inconsistent geometry
        // (see the deferred sub-item in the report).
        if seq.conditional && !page_ids.is_empty() {
            self.apply_last_page_master(fo_tree, area_tree, &seq, &page_ids, first_page_number)?;
        }

        // Place collected footnotes at the bottom of every page's body region,
        // using that page's resolved geometry.
        for (page_idx, page_id) in page_ids.iter().enumerate() {
            let body_rect = seq
                .page_geoms
                .borrow()
                .get(page_idx)
                .map(|g| g.body_rect)
                .unwrap_or(seq.geom.body_rect);
            self.place_footnotes_for_page(area_tree, *page_id, body_rect)?;
        }

        // Build the page-accurate marker context from the finished flow.
        let seq_markers = collect_sequence_markers(
            fo_tree,
            area_tree,
            &page_ids,
            &placements,
            doc_markers.snapshot(),
        );

        // PASS 2 — static content.  Now that every page's markers are known,
        // lay out the repeated static-content regions on each page, resolving
        // `fo:retrieve-marker` against that page's marker context.  The page
        // number is re-established first so per-page `fo:page-number` resolves
        // correctly in headers/footers.
        for (page_idx, page_id) in page_ids.iter().enumerate() {
            resolver.set_current_page(first_page_number + page_idx);
            let view = PageMarkerView::new(&seq_markers, page_idx);
            let geom = seq
                .page_geoms
                .borrow()
                .get(page_idx)
                .copied()
                .unwrap_or(seq.geom);
            self.layout_page_static_content(
                fo_tree, area_tree, &seq, &geom, *page_id, resolver, &view,
            )?;
        }

        // Carry the markers in effect at the sequence end into the next
        // sequence (for `retrieve-boundary="document"`).
        doc_markers.set_trailing(seq_markers.trailing_markers());

        // Advance the page counter to the first page of the next sequence.
        resolver.set_current_page(first_page_number + page_ids.len());

        Ok(Some(first_page_id))
    }

    /// Apply the `last`-page conditional master to the final page of a
    /// conditional sequence, now that the total page count is known.
    ///
    /// Re-resolves the final page's geometry with `is_last = true` (and the page
    /// being blank only if it already was — the last page being a blank
    /// even/odd-page intermediate is preserved).  When the re-resolved geometry
    /// differs from the forward-pass geometry:
    ///
    /// * **Body geometry unchanged** — the page rect and its region-body area
    ///   rect are updated to the new geometry and the recorded per-page geometry
    ///   is overwritten, so the subsequent static-content / footnote passes use
    ///   the `last` master's regions.  The already-placed flow stays valid
    ///   because the body box it filled is identical.
    /// * **Body geometry differs** — applying it would require reflowing the
    ///   final page's flow into a different body box.  That reflow is out of safe
    ///   scope this session, so the new geometry is **not** applied (the
    ///   forward-pass geometry is kept) rather than emitting a page whose body
    ///   content no longer matches its region — an honest, recorded residual.
    fn apply_last_page_master(
        &self,
        fo_tree: &FoArena,
        area_tree: &mut AreaTree,
        seq: &SequenceLayout,
        page_ids: &[AreaId],
        first_page_number: usize,
    ) -> Result<()> {
        let last_idx = page_ids.len() - 1;
        let page_id = page_ids[last_idx];
        let forward_geom = match seq.page_geoms.borrow().get(last_idx).copied() {
            Some(g) => g,
            None => return Ok(()),
        };

        // A single-page sequence's only page is simultaneously first and last.
        let is_first = last_idx == 0;
        let page_number = first_page_number + last_idx;
        let ctx = PageContext::for_page(page_number, is_first, true);
        // The forward pass never marks pages blank here (blank intermediates are
        // never the trailing content page in practice); resolve as not-blank.
        let last_master =
            self.resolve_page_master_for_page(fo_tree, &seq.master_reference, &ctx, false);
        let last_geom = self.extract_page_region_geometry(fo_tree, &last_master);

        // Nothing to do when the `last` condition selected an identical
        // geometry (same master, or a different master with the same regions).
        if geometries_equal(&last_geom, &forward_geom) {
            return Ok(());
        }

        // Only safe to adopt when the body box is identical (the flow was
        // paginated against it); otherwise keep the forward geometry.
        if last_geom.body_rect != forward_geom.body_rect {
            return Ok(());
        }

        // Adopt the `last` geometry: overwrite the recorded entry, resize the
        // page area and its region-body area.
        if let Some(slot) = seq.page_geoms.borrow_mut().get_mut(last_idx) {
            *slot = last_geom;
        }
        if let Some(page_node) = area_tree.get_mut(page_id) {
            page_node.area.geometry = Rect::from_point_size(
                Point::ZERO,
                Size::new(last_geom.page_width, last_geom.page_height),
            );
        }
        // The region-body is the page's `Region`/`Column` child; resize it.
        for child_id in area_tree.children(page_id) {
            let is_body = area_tree
                .get(child_id)
                .map(|n| matches!(n.area.area_type, AreaType::Region | AreaType::Column))
                .unwrap_or(false);
            if is_body {
                if let Some(node) = area_tree.get_mut(child_id) {
                    node.area.geometry = last_geom.body_rect;
                }
                break;
            }
        }

        Ok(())
    }

    /// Lay out a single page's repeated static-content regions (header / footer
    /// / start & end sidebars), resolving each `fo:retrieve-marker` against
    /// `view` (this page's marker context), then restore the canonical page
    /// child order (static regions, then region-body, then footnotes).
    ///
    /// Static content is laid out here — after the flow — rather than at page
    /// construction, because the markers a page's header retrieves are only
    /// known once that page's flow has been placed.  The reorder compensates for
    /// the append-only area tree so the static areas precede the region-body
    /// exactly as if they had been built first.
    #[allow(clippy::too_many_arguments)]
    fn layout_page_static_content(
        &self,
        fo_tree: &FoArena,
        area_tree: &mut AreaTree,
        seq: &SequenceLayout,
        geom: &PageRegionGeometry,
        page_id: AreaId,
        resolver: &mut PageNumberResolver,
        view: &PageMarkerView,
    ) -> Result<()> {
        if let Some(header_id) = seq.static_before {
            self.layout_static_content_in_rect(
                fo_tree,
                header_id,
                area_tree,
                page_id,
                geom.before_rect,
                AreaType::Header,
                resolver,
                view,
            )?;
        }
        if let Some(footer_id) = seq.static_after {
            self.layout_static_content_in_rect(
                fo_tree,
                footer_id,
                area_tree,
                page_id,
                geom.after_rect,
                AreaType::Footer,
                resolver,
                view,
            )?;
        }
        if let Some(start_id) = seq.static_start {
            self.layout_static_content_in_rect(
                fo_tree,
                start_id,
                area_tree,
                page_id,
                geom.start_rect,
                AreaType::SidebarStart,
                resolver,
                view,
            )?;
        }
        if let Some(end_id) = seq.static_end {
            self.layout_static_content_in_rect(
                fo_tree,
                end_id,
                area_tree,
                page_id,
                geom.end_rect,
                AreaType::SidebarEnd,
                resolver,
                view,
            )?;
        }

        area_tree.reorder_children(page_id, page_child_priority);
        Ok(())
    }

    /// Resolve the [`PageRegionGeometry`] a page should use given its
    /// construction context, **and record it** in `seq.page_geoms` so the
    /// static-content / footnote passes (and the `last` fix-up) can later look
    /// it up by page index.
    ///
    /// * Non-conditional sequences (plain simple-page-master) always use
    ///   `seq.geom`.
    /// * Conditional sequences resolve the concrete simple-page-master from the
    ///   page's [`PageContext`]: `is_first` is true only for sequence index 0,
    ///   `odd`/`even` follow the absolute `page_number`, and `is_blank` marks
    ///   the blank intermediate pages inserted for `even-page`/`odd-page` forced
    ///   breaks.  `last` is **not** resolvable here (the total page count is
    ///   unknown during the forward pass), so `is_last` is always `false`; the
    ///   final page's `last`-conditional is applied by the post-pass fix-up in
    ///   [`Self::layout_page_sequence`].
    fn resolve_and_record_page_geom(
        &self,
        fo_tree: &FoArena,
        seq: &SequenceLayout,
        page_number: usize,
        is_blank: bool,
    ) -> PageRegionGeometry {
        let geom = if seq.conditional {
            let seq_index = seq.page_geoms.borrow().len();
            let is_first = seq_index == 0;
            let ctx = PageContext::for_page(page_number, is_first, false);
            let master =
                self.resolve_page_master_for_page(fo_tree, &seq.master_reference, &ctx, is_blank);
            self.extract_page_region_geometry(fo_tree, &master)
        } else {
            seq.geom
        };
        seq.page_geoms.borrow_mut().push(geom);
        geom
    }

    /// Create a new top-level page area and its (empty) region-body, using the
    /// supplied per-page `geom`.
    ///
    /// Returns `(page_id, region_body_id)`. The repeated static content
    /// (headers / footers / sidebars) is **not** built here: it is laid out in
    /// a second pass once the flow's per-page markers are known (see
    /// [`Self::layout_page_static_content`]).
    fn build_page_with_geom(
        &self,
        area_tree: &mut AreaTree,
        seq: &SequenceLayout,
        geom: &PageRegionGeometry,
    ) -> Result<(AreaId, AreaId)> {
        let page_rect =
            Rect::from_point_size(Point::ZERO, Size::new(geom.page_width, geom.page_height));
        let page_area = Area::new(AreaType::Page, page_rect).with_traits(seq.page_traits.clone());
        let page_id = area_tree.add_area(page_area);

        // Region-body that will receive flow content.
        let region =
            Area::new(AreaType::Region, geom.body_rect).with_traits(seq.body_traits.clone());
        let region_id = area_tree.add_area(region);
        area_tree
            .append_child(page_id, region_id)
            .map_err(fop_types::FopError::Generic)?;

        Ok((page_id, region_id))
    }

    /// Lay out a flow's block children with height-overflow pagination.
    #[allow(clippy::too_many_arguments)]
    fn paginate_flow(
        &self,
        fo_tree: &FoArena,
        flow_node_id: NodeId,
        area_tree: &mut AreaTree,
        seq: &SequenceLayout,
        first_region_id: AreaId,
        page_ids: &mut Vec<AreaId>,
        resolver: &mut PageNumberResolver,
        placements: &mut Vec<(AreaId, NodeId)>,
    ) -> Result<()> {
        let flow_node = fo_tree.get(flow_node_id).ok_or_else(|| {
            fop_types::FopError::Generic(format!("Flow node {} not found", flow_node_id))
        })?;
        let column_count = match &flow_node.data {
            FoNodeData::Flow { properties, .. } => extract_column_count(properties),
            _ => return Ok(()),
        };
        let column_gap = match &flow_node.data {
            FoNodeData::Flow { properties, .. } => extract_column_gap(properties),
            _ => Length::ZERO,
        };

        let body_rect = seq.geom.body_rect;
        let body_width = body_rect.width;
        let body_height = body_rect.height;
        let children = fo_tree.children(flow_node_id);

        // Multi-column flows are paginated across pages: columns fill left to
        // right on each page, and a new page is started (repeating static
        // content) once the last column of the current page is full.  Handled by
        // a sibling driver that shares this module's page-construction machinery.
        if column_count > 1 {
            return self.paginate_multicolumn_flow(
                fo_tree,
                &children,
                column_count,
                column_gap,
                area_tree,
                seq,
                first_region_id,
                page_ids,
                resolver,
                placements,
            );
        }

        // Single-column paginated layout. `page_blocks` tracks the blocks placed
        // on the *current* page (in order, floats excluded) for keep-group
        // migration; `current_y` is the region-relative stacking cursor.
        let breaker = self.page_breaker_for(&seq.geom);
        let mut current_region_id = first_region_id;
        let mut page_blocks: Vec<AreaId> = Vec::new();
        let mut current_y = Length::ZERO;
        let mut float_manager = FloatManager::new();

        for child_id in children {
            float_manager.remove_floats_above(current_y);

            // Pull everything we need off the FO node up front so we hold no
            // borrow into `fo_tree` across the area-tree mutations below.
            let (is_float, break_before, break_after) = match fo_tree.get(child_id) {
                Some(n) => {
                    let is_float = matches!(n.data, FoNodeData::Float { .. });
                    let (bb, ba) = match n.data.properties() {
                        Some(p) => (extract_break_before(p), extract_break_after(p)),
                        None => (BreakValue::Auto, BreakValue::Auto),
                    };
                    (is_float, bb, ba)
                }
                None => continue,
            };

            if is_float {
                let is_odd_page = resolver.current_page() % 2 == 1;
                if let Some(float_area_id) = self.layout_float_in_flow(
                    fo_tree,
                    child_id,
                    area_tree,
                    current_region_id,
                    current_y,
                    body_width,
                    is_odd_page,
                    &mut float_manager,
                    resolver,
                )? {
                    // A float's content can also carry markers; attribute them to
                    // the page the float landed on.
                    placements.push((float_area_id, child_id));
                }
                continue;
            }

            // Forced break-before: move to a fresh page before laying out this
            // block (only if the current page already has content).
            if break_before.forces_page_break()
                && (!page_blocks.is_empty() || current_y > Length::ZERO)
            {
                current_region_id = self.start_new_page_for_break(
                    fo_tree,
                    area_tree,
                    seq,
                    page_ids,
                    resolver,
                    break_before,
                )?;
                page_blocks.clear();
                current_y = Length::ZERO;
                float_manager.clear();
            }

            // Apply `clear`: advance past active floats if requested.
            if let Some(props) = fo_tree.get(child_id).and_then(|n| n.data.properties()) {
                current_y = float_manager.get_clear_position(extract_clear(props), current_y);
            }

            // Float-aware horizontal offset and available width.
            let (left_offset, avail_width) = float_manager.available_width(current_y, body_width);

            let block_id_opt = self.layout_block_float_aware(
                fo_tree,
                child_id,
                area_tree,
                current_region_id,
                current_y,
                avail_width,
                left_offset,
                resolver,
            )?;

            if let Some(block_id) = block_id_opt {
                page_blocks.push(block_id);
                placements.push((block_id, child_id));

                let (block_y, block_h) = match area_tree.get(block_id) {
                    Some(n) => (n.area.geometry.y, n.area.height()),
                    None => (current_y, Length::ZERO),
                };
                let block_bottom = block_y + block_h;

                // Overflow when the block's bottom exceeds the body box.  Unlike
                // the previous deferral, a block that overflows even an *empty*
                // region-body (`current_y == 0`) is no longer clipped: it is
                // split at a line-box boundary (widow/orphan-governed) so the
                // head fills the page and the tail continues onto the next.
                if block_bottom > body_height {
                    let (next_region, next_y, next_blocks) = self.place_overflowing_block(
                        fo_tree,
                        area_tree,
                        seq,
                        page_ids,
                        resolver,
                        &breaker,
                        block_id,
                        block_y,
                        body_height,
                        current_y == Length::ZERO,
                        std::mem::take(&mut page_blocks),
                    )?;
                    current_region_id = next_region;
                    current_y = next_y;
                    page_blocks = next_blocks;
                    float_manager.clear();
                } else {
                    current_y = block_bottom;
                }
            }

            // Forced break-after: subsequent content starts on a fresh page.
            if break_after.forces_page_break() {
                current_region_id = self.start_new_page_for_break(
                    fo_tree,
                    area_tree,
                    seq,
                    page_ids,
                    resolver,
                    break_after,
                )?;
                page_blocks.clear();
                current_y = Length::ZERO;
                float_manager.clear();
            }
        }

        float_manager.clear();
        Ok(())
    }

    /// Lay out a multi-column flow with cross-page pagination.
    ///
    /// Columns of the current page fill left to right; when a block does not fit
    /// in the current column the cursor advances to the next column, and when the
    /// **last** column of the page is full a new page is started (via
    /// [`Self::start_new_page`], so static content repeats) and the flow resumes
    /// in the first column of the new page's region-body.  Forced page breaks
    /// (`break-before` / `break-after` = `page` / `even-page` / `odd-page`) start
    /// a fresh page (mirroring the single-column path); column breaks advance to
    /// the next column (or a new page when the last column is reached).
    ///
    /// Blocks are placed directly under the region-body of the page they belong
    /// to, so their geometry is page-relative and correct without any later
    /// reparenting (the page is chosen *before* the block is emitted).
    #[allow(clippy::too_many_arguments)]
    fn paginate_multicolumn_flow(
        &self,
        fo_tree: &FoArena,
        children: &[NodeId],
        column_count: i32,
        column_gap: Length,
        area_tree: &mut AreaTree,
        seq: &SequenceLayout,
        first_region_id: AreaId,
        page_ids: &mut Vec<AreaId>,
        resolver: &mut PageNumberResolver,
        placements: &mut Vec<(AreaId, NodeId)>,
    ) -> Result<()> {
        let body_rect = seq.geom.body_rect;
        let mut multi_col = MultiColumnLayout::new(column_count, column_gap, body_rect.width)
            .with_max_height(body_rect.height);
        let mut current_region_id = first_region_id;

        // Blocks placed on the page currently being filled, recorded so the
        // *final* page can be re-balanced once the flow ends (see
        // [`Self::balance_multicolumn_page`]).  Whenever a new page is started
        // the just-completed page was filled to capacity and is left on the
        // sequential path, so the accumulator is reset rather than balanced.
        let mut page_entries: Vec<BalanceEntry> = Vec::new();
        // `true` once the next block must start a fresh column (a forced
        // `break-after = column` ran, leaving the cursor mid-column with a
        // mandatory boundary ahead of the next block).
        let mut pending_column_boundary = false;

        for &child_id in children {
            // Non-block flow children (e.g. floats/tables) have no column
            // flow-control; emit them in place (the placement primitive returns
            // `None` for them, matching the legacy single-page behaviour).
            let metrics = match self.measure_multicolumn_block(fo_tree, child_id) {
                Some(metrics) => metrics,
                None => {
                    if let Some(area_id) = self.emit_multicolumn_block(
                        fo_tree,
                        child_id,
                        area_tree,
                        current_region_id,
                        &mut multi_col,
                        resolver,
                    )? {
                        placements.push((area_id, child_id));
                    }
                    continue;
                }
            };

            // Track whether this block is preceded by a mandatory column
            // boundary, so the balanced partition can respect it.
            let mut forces_boundary = pending_column_boundary;
            pending_column_boundary = false;

            // Forced break-before. A page break starts a fresh page (only when the
            // current page already holds content); a column break advances to the
            // next column (starting a new page when the last column is reached).
            let pages_before = page_ids.len();
            if metrics.break_before.forces_page_break() && multicolumn_page_has_content(&multi_col)
            {
                current_region_id = self.start_new_page_for_break(
                    fo_tree,
                    area_tree,
                    seq,
                    page_ids,
                    resolver,
                    metrics.break_before,
                )?;
                multi_col.reset();
            } else if matches!(metrics.break_before, BreakValue::Column)
                && multi_col.column_y > Length::ZERO
            {
                current_region_id = self.advance_multicolumn(
                    fo_tree,
                    area_tree,
                    seq,
                    page_ids,
                    resolver,
                    &mut multi_col,
                    current_region_id,
                )?;
                forces_boundary = true;
            }

            // Apply space-before, then decide the column fit.  This mirrors the
            // single-page column placement exactly, extended so that a full last
            // column starts a new page rather than overflowing.
            multi_col.allocate(metrics.space_before);
            let total_height = metrics.space_before + metrics.line_height + metrics.space_after;
            if multi_col.is_column_filled(total_height) {
                current_region_id = self.advance_multicolumn(
                    fo_tree,
                    area_tree,
                    seq,
                    page_ids,
                    resolver,
                    &mut multi_col,
                    current_region_id,
                )?;
            }

            // A new page was started while placing this block: the previous page
            // was filled to capacity, so it stays sequential — discard its
            // accumulated entries (the freshly started page begins empty).
            if page_ids.len() > pages_before {
                page_entries.clear();
                forces_boundary = false;
            }

            // Place the block in the current column of the current page, capturing
            // the exact vertical extent it consumes (for the balance partition).
            let column_y_before = multi_col.column_y - metrics.space_before;
            if let Some(area_id) = self.emit_multicolumn_block(
                fo_tree,
                child_id,
                area_tree,
                current_region_id,
                &mut multi_col,
                resolver,
            )? {
                placements.push((area_id, child_id));
                // Apply space-after, then record what the block consumed.
                multi_col.allocate(metrics.space_after);
                let occupied = (multi_col.column_y - column_y_before).max(Length::ZERO);
                page_entries.push(BalanceEntry {
                    area_id,
                    occupied,
                    forces_column_boundary: forces_boundary,
                });
            } else {
                multi_col.allocate(metrics.space_after);
            }

            // Forced break-after: a page break starts a fresh page for the next
            // block; a column break advances to the next column.
            if metrics.break_after.forces_page_break() {
                current_region_id = self.start_new_page_for_break(
                    fo_tree,
                    area_tree,
                    seq,
                    page_ids,
                    resolver,
                    metrics.break_after,
                )?;
                multi_col.reset();
                page_entries.clear();
            } else if matches!(metrics.break_after, BreakValue::Column) {
                current_region_id = self.advance_multicolumn(
                    fo_tree,
                    area_tree,
                    seq,
                    page_ids,
                    resolver,
                    &mut multi_col,
                    current_region_id,
                )?;
                pending_column_boundary = true;
            }
        }

        // The flow has ended; `page_entries` describes the final page.  Re-balance
        // its columns so they end at roughly equal heights (newspaper style),
        // honouring any mandatory column boundaries recorded along the way.  Full
        // intermediate pages were never accumulated, so they keep the sequential
        // fill.
        self.balance_multicolumn_page(area_tree, &page_entries, column_count, &multi_col)?;

        Ok(())
    }

    /// Re-balance the columns of the final multi-column page so they end at
    /// approximately equal heights (the newspaper-style balanced-columns
    /// objective), repositioning the already-emitted block areas in place.
    ///
    /// # Algorithm
    ///
    /// The ordered `entries` are partitioned into at most `column_count`
    /// *contiguous* groups (a block can never move to an earlier column than a
    /// preceding block — document order is preserved both within and across
    /// columns).  The partition minimises the **maximum column height**:
    ///
    /// 1. Binary-search the target column height `target` over the closed range
    ///    `[lo, hi]`, where `lo` is the tallest single block (no column can be
    ///    shorter than its tallest member) and `hi` is the total height of all
    ///    blocks (one column holds everything, always feasible height-wise).
    /// 2. For a candidate `target`, a greedy feasibility walk
    ///    ([`balance_columns_needed`]) packs blocks into columns, opening a new
    ///    column whenever the next block would overflow `target` **or** carries a
    ///    mandatory `forces_column_boundary`.  The candidate is feasible when no
    ///    more than `column_count` columns are needed.
    /// 3. The smallest feasible `target` yields the balanced partition.  A single
    ///    block taller than `target` simply occupies its own column and sets the
    ///    realised maximum (blocks are never split across columns here — that is a
    ///    follow-up).
    ///
    /// # Edge cases
    ///
    /// * Fewer blocks than columns ⇒ trailing columns stay empty (each block can
    ///   take its own column), which still minimises the maximum.
    /// * A mandatory boundary forces a column break regardless of `target`; if the
    ///   mandatory boundaries alone demand more than `column_count` columns the
    ///   greedy walk reports infeasibility for every `target`, so the search
    ///   clamps to `hi` and the partition degrades gracefully (later columns
    ///   simply overflow, matching the un-balanced fallback).
    /// * An empty page (no entries) is a no-op.
    fn balance_multicolumn_page(
        &self,
        area_tree: &mut AreaTree,
        entries: &[BalanceEntry],
        column_count: i32,
        multi_col: &MultiColumnLayout,
    ) -> Result<()> {
        if entries.is_empty() || column_count <= 1 {
            return Ok(());
        }

        let columns = balance_partition(entries, column_count);

        // Re-place each block at the origin of its assigned column, stacking the
        // blocks of a column from the body top while preserving their inter-block
        // spacing (the gap each block carried over its laid-out content height).
        let stride = multi_col.column_width() + multi_col.column_gap;
        for (column_index, group) in columns.iter().enumerate() {
            // `MultiColumnLayout::current_column_x` multiplies the column stride
            // by the (i32) column index; replicate it for the balanced index.
            let column_x = stride * column_index as i32;

            let mut column_y = Length::ZERO;
            for entry in group {
                let (old_x, height) = match area_tree.get(entry.area_id) {
                    Some(node) => (node.area.geometry.x, node.area.height()),
                    None => continue,
                };
                // The block's start-indent is the horizontal offset of its origin
                // beyond its *original* column origin; preserve it when moving to
                // the new column so indented blocks stay indented.
                let original_column_x = Self::nearest_column_origin(multi_col, old_x);
                let start_indent = (old_x - original_column_x).max(Length::ZERO);

                if let Some(node) = area_tree.get_mut(entry.area_id) {
                    node.area.geometry.x = column_x + start_indent;
                    node.area.geometry.y = column_y;
                }
                // Stack the next block below this one; `occupied` folds in the
                // block's leading/trailing space, and never under-counts the
                // laid-out content height.
                column_y += entry.occupied.max(height);
            }
        }

        Ok(())
    }

    /// The origin (`x`) of the column whose span contains `x`, used to recover a
    /// block's start-indent independent of which column it was originally placed
    /// in.
    fn nearest_column_origin(multi_col: &MultiColumnLayout, x: Length) -> Length {
        let stride = multi_col.column_width() + multi_col.column_gap;
        if stride <= Length::ZERO {
            return Length::ZERO;
        }
        let index = (x.to_pt() / stride.to_pt()).floor().max(0.0) as i32;
        stride * index
    }

    /// Advance the multi-column cursor to the next column, or — when the current
    /// column is the last one on the page — start a new page (repeating static
    /// content) and reset to its first column.  Returns the region-body the flow
    /// should continue placing blocks into.
    #[allow(clippy::too_many_arguments)]
    fn advance_multicolumn(
        &self,
        fo_tree: &FoArena,
        area_tree: &mut AreaTree,
        seq: &SequenceLayout,
        page_ids: &mut Vec<AreaId>,
        resolver: &mut PageNumberResolver,
        multi_col: &mut MultiColumnLayout,
        current_region_id: AreaId,
    ) -> Result<AreaId> {
        if multi_col.next_column() {
            // Moved to the next column on the same page.
            Ok(current_region_id)
        } else {
            // The last column was full: start a new page and reset to column one.
            let new_region = self.start_new_page(fo_tree, area_tree, seq, page_ids, resolver)?;
            multi_col.reset();
            Ok(new_region)
        }
    }

    /// Pull the column flow-control measurements off a flow child, or `None` when
    /// the child is not a block (and therefore takes part in no column logic).
    ///
    /// The resolved values mirror exactly those derived by the single-page column
    /// placement so the paginated and single-page paths agree on column fill.
    fn measure_multicolumn_block(
        &self,
        fo_tree: &FoArena,
        node_id: NodeId,
    ) -> Option<MultiColumnMetrics> {
        let node = fo_tree.get(node_id)?;
        let properties = match &node.data {
            FoNodeData::Block { properties } | FoNodeData::BlockContainer { properties } => {
                properties
            }
            _ => return None,
        };
        let traits = extract_traits(properties);
        let line_height = traits
            .line_height
            .or(traits.font_size)
            .unwrap_or(Length::from_pt(12.0));
        Some(MultiColumnMetrics {
            space_before: extract_space_before(properties),
            space_after: extract_space_after(properties),
            line_height,
            break_before: extract_break_before(properties),
            break_after: extract_break_after(properties),
        })
    }

    /// Increment the page counter, build the next page (empty region-body, no
    /// static content yet), record it, and return its region-body id.
    /// Handle a block whose bottom edge overflows the region-body, splitting it
    /// across the page boundary when widow/orphan control permits and otherwise
    /// migrating the keep-group whole (the legacy behaviour).
    ///
    /// `block_y` is the overflowing block's region-relative top; `body_height`
    /// the region-body content height; `empty_page` is `true` when the block is
    /// the first content of the current (otherwise empty) page.  `page_blocks`
    /// is taken by value (the blocks placed on the current page) and the updated
    /// list for the *new* current page is returned.
    ///
    /// Returns `(new_region_id, new_current_y, new_page_blocks)`.
    ///
    /// # Splitting loop
    ///
    /// 1. Attempt to split the block at `body_height - block_y` of available
    ///    head space ([`PageBreaker::split_area`]).  When it splits, the head
    ///    stays on the current page; a fresh page is started and the
    ///    continuation is placed at its body top.  The continuation may itself
    ///    overflow, so the loop repeats — a single block can span 3+ pages.
    /// 2. When the block refuses to split (keep-together, too few lines for
    ///    widows+orphans, or a non-empty page with no fitting head line), fall
    ///    back to migrating the trailing keep-group to a new page, exactly as
    ///    before.  A continuation produced by step 1 that still cannot split is
    ///    placed on its own fresh page (it is then the sole block, so migrating
    ///    it whole always makes progress).
    #[allow(clippy::too_many_arguments)]
    fn place_overflowing_block(
        &self,
        fo_tree: &FoArena,
        area_tree: &mut AreaTree,
        seq: &SequenceLayout,
        page_ids: &mut Vec<AreaId>,
        resolver: &mut PageNumberResolver,
        breaker: &PageBreaker,
        block_id: AreaId,
        block_y: Length,
        body_height: Length,
        empty_page: bool,
        page_blocks: Vec<AreaId>,
        // returns the new page state
    ) -> Result<(AreaId, Length, Vec<AreaId>)> {
        // The head space available to the overflowing block is from its top edge
        // down to the body bottom.
        let available = (body_height - block_y).max(Length::ZERO);

        if let Some(continuation_id) =
            breaker.split_area(area_tree, block_id, available, empty_page)?
        {
            // The head stayed on the current page; start a new page for the
            // continuation and keep splitting it until the tail fits.
            let mut region_id = self.start_new_page(fo_tree, area_tree, seq, page_ids, resolver)?;
            let mut cont_id = continuation_id;
            loop {
                // Place the continuation flush at the new page's body top.
                area_tree
                    .reparent(cont_id, region_id)
                    .map_err(fop_types::FopError::Generic)?;
                if let Some(node) = area_tree.get_mut(cont_id) {
                    node.area.geometry.y = Length::ZERO;
                }
                let cont_h = area_tree
                    .get(cont_id)
                    .map(|n| n.area.height())
                    .unwrap_or(Length::ZERO);

                if cont_h <= body_height {
                    // The tail fits on this page; it becomes the page's first
                    // (and so far only) block.
                    return Ok((region_id, cont_h, vec![cont_id]));
                }

                // The continuation still overflows an empty page: split again.
                match breaker.split_area(area_tree, cont_id, body_height, true)? {
                    Some(next_cont) => {
                        region_id =
                            self.start_new_page(fo_tree, area_tree, seq, page_ids, resolver)?;
                        cont_id = next_cont;
                    }
                    None => {
                        // Cannot split further (e.g. fewer than widows+orphans
                        // tail lines, or keep-together): leave it whole on this
                        // page even though it overflows — it is the sole block,
                        // so this is the best achievable placement.
                        return Ok((region_id, cont_h, vec![cont_id]));
                    }
                }
            }
        }

        // The block did not split — migrate its trailing keep-group whole, as
        // the pre-splitting implementation did.  `block_id` is already the last
        // entry of `page_blocks` (pushed by the caller before overflow handling).
        debug_assert_eq!(page_blocks.last().copied(), Some(block_id));
        let group_start = keep_group_start(breaker, area_tree, &page_blocks);
        // If the whole page is one glued group we cannot honour the keep without
        // overflowing; fall back to moving the last block alone so layout still
        // makes progress.
        let effective_start = if group_start == 0 && page_blocks.len() > 1 {
            page_blocks.len() - 1
        } else {
            group_start
        };
        let group: Vec<AreaId> = page_blocks[effective_start..].to_vec();

        let region_id = self.start_new_page(fo_tree, area_tree, seq, page_ids, resolver)?;
        let new_y = migrate_blocks(area_tree, &group, region_id)?;
        Ok((region_id, new_y, group))
    }

    fn start_new_page(
        &self,
        fo_tree: &FoArena,
        area_tree: &mut AreaTree,
        seq: &SequenceLayout,
        page_ids: &mut Vec<AreaId>,
        resolver: &mut PageNumberResolver,
    ) -> Result<AreaId> {
        self.start_new_page_inner(fo_tree, area_tree, seq, page_ids, resolver, false)
    }

    /// `start_new_page` with an explicit `is_blank` flag for the geometry
    /// resolver (the blank intermediates inserted by even/odd-page forced breaks
    /// must be matched by `blank-or-not-blank="blank"` conditionals).
    fn start_new_page_inner(
        &self,
        fo_tree: &FoArena,
        area_tree: &mut AreaTree,
        seq: &SequenceLayout,
        page_ids: &mut Vec<AreaId>,
        resolver: &mut PageNumberResolver,
        is_blank: bool,
    ) -> Result<AreaId> {
        resolver.set_current_page(resolver.current_page() + 1);
        let geom =
            self.resolve_and_record_page_geom(fo_tree, seq, resolver.current_page(), is_blank);
        let (page_id, region_id) = self.build_page_with_geom(area_tree, seq, &geom)?;
        page_ids.push(page_id);
        Ok(region_id)
    }

    /// Like [`Self::start_new_page`], but additionally inserts a blank page when
    /// the break requires a specific page parity (`break-before/after =
    /// even-page | odd-page`) that the freshly created page does not satisfy.
    fn start_new_page_for_break(
        &self,
        fo_tree: &FoArena,
        area_tree: &mut AreaTree,
        seq: &SequenceLayout,
        page_ids: &mut Vec<AreaId>,
        resolver: &mut PageNumberResolver,
        break_value: BreakValue,
    ) -> Result<AreaId> {
        // Decide up front whether a blank intermediate page is needed: the
        // freshly created page would carry parity `current_page + 1`; if that
        // mismatches the break's required parity it becomes the blank page and
        // content lands on the following (correct-parity) page.
        let next_is_odd = (resolver.current_page() + 1) % 2 == 1;
        let needs_extra = (break_value.requires_even_page() && next_is_odd)
            || (break_value.requires_odd_page() && !next_is_odd);

        // The first created page is blank exactly when an extra page is needed.
        let mut region_id =
            self.start_new_page_inner(fo_tree, area_tree, seq, page_ids, resolver, needs_extra)?;
        if needs_extra {
            region_id = self.start_new_page(fo_tree, area_tree, seq, page_ids, resolver)?;
        }

        Ok(region_id)
    }

    /// Build a [`PageBreaker`] carrying this sequence's page geometry, used for
    /// its keep-constraint break decisions (`can_break_before`).
    fn page_breaker_for(&self, geom: &PageRegionGeometry) -> PageBreaker {
        let body = geom.body_rect;
        let margin_top = body.y;
        let margin_left = body.x;
        let margin_bottom = geom.page_height - body.y - body.height;
        let margin_right = geom.page_width - body.x - body.width;
        PageBreaker::new(
            geom.page_width,
            geom.page_height,
            [margin_top, margin_right, margin_bottom, margin_left],
        )
    }
}

/// Whether two [`PageRegionGeometry`] values describe identical page/region
/// rectangles.  Used by the `last`-conditional fix-up to decide whether the
/// final page's geometry actually changed (and so whether any area resizing is
/// needed at all).
fn geometries_equal(a: &PageRegionGeometry, b: &PageRegionGeometry) -> bool {
    a.page_width == b.page_width
        && a.page_height == b.page_height
        && a.before_rect == b.before_rect
        && a.after_rect == b.after_rect
        && a.start_rect == b.start_rect
        && a.end_rect == b.end_rect
        && a.body_rect == b.body_rect
}

/// Whether the current page already holds multi-column content.  A forced page
/// break before the very first block of a page (first column, top) must be a
/// no-op — otherwise it would emit a spurious blank leading page — exactly as
/// the single-column path only breaks when the page already has content.
fn multicolumn_page_has_content(multi_col: &MultiColumnLayout) -> bool {
    multi_col.current_column > 0 || multi_col.column_y > Length::ZERO
}

/// Canonical sibling order of a page's direct child areas, used to restore the
/// natural order after the deferred static content is appended.  Static regions
/// (header / footer / sidebars) paint first, then the region-body, then any
/// footnote separator and footnote areas — matching the order produced when
/// static content was built before the flow.
fn page_child_priority(area_type: AreaType) -> u8 {
    match area_type {
        AreaType::Header | AreaType::Footer | AreaType::SidebarStart | AreaType::SidebarEnd => 0,
        AreaType::Region | AreaType::Column => 1,
        _ => 2,
    }
}

/// Find the smallest index `s` such that `blocks[s..]` must move to the next
/// page together, i.e. for every `k` in `(s, len)` a break before `blocks[k]`
/// is forbidden by a keep constraint. Scans backward from the last block.
fn keep_group_start(breaker: &PageBreaker, area_tree: &AreaTree, blocks: &[AreaId]) -> usize {
    let n = blocks.len();
    if n == 0 {
        return 0;
    }
    let mut start = n - 1;
    while start > 0 {
        // A legal break before `blocks[start]` ends the glued run.
        if breaker.can_break_before(area_tree, blocks[start], start, blocks) {
            break;
        }
        start -= 1;
    }
    start
}

/// Partition the ordered `entries` into at most `column_count` contiguous groups
/// (one per column) minimising the maximum column height, honouring mandatory
/// column boundaries.  Returns exactly `column_count` groups (trailing ones may
/// be empty when there are fewer blocks than columns).
///
/// See [`LayoutEngine::balance_multicolumn_page`] for the full algorithm; this
/// is the pure partition step (binary search over the target height with a
/// greedy feasibility check).
fn balance_partition(entries: &[BalanceEntry], column_count: i32) -> Vec<Vec<BalanceEntry>> {
    let columns = column_count.max(1) as usize;
    let mut groups: Vec<Vec<BalanceEntry>> = vec![Vec::new(); columns];
    if entries.is_empty() {
        return groups;
    }

    // Search bounds: `hi` (one column holds everything) is always feasible for
    // the *height* constraint; `lo` is the tallest single block, below which no
    // partition can pack any column.  Lengths are integer EMU under the hood, so
    // a unit-step binary search terminates exactly.
    let total: Length = entries
        .iter()
        .fold(Length::ZERO, |acc, e| acc + e.occupied.max(Length::ZERO));
    let max_block: Length = entries
        .iter()
        .fold(Length::ZERO, |acc, e| acc.max(e.occupied.max(Length::ZERO)));

    // Lengths are stored as integer millipoints, so the unit-step binary search
    // over `[lo, hi]` terminates exactly.
    let mut lo = max_block.millipoints();
    let mut hi = total.millipoints().max(lo);

    // If even `hi` is infeasible (mandatory boundaries alone demand more than
    // `column_count` columns) the partition cannot satisfy the constraint; fall
    // back to the greedy packing at `hi`, which simply lets later columns
    // overflow — graceful degradation matching the un-balanced behaviour.
    let feasible_at = |target_milli: i32| -> bool {
        balance_columns_needed(entries, Length::from_millipoints(target_milli)) <= columns
    };

    if feasible_at(hi) {
        // Binary-search the smallest feasible target in [lo, hi].
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if feasible_at(mid) {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
    }
    let target = Length::from_millipoints(hi);

    // Re-run the greedy packing at the chosen target to materialise the groups.
    let mut column_index = 0usize;
    let mut column_height = Length::ZERO;
    for (i, entry) in entries.iter().enumerate() {
        let height = entry.occupied.max(Length::ZERO);
        let must_break = entry.forces_column_boundary && i > 0;
        let would_overflow = column_height + height > target && column_height > Length::ZERO;
        if (must_break || would_overflow) && column_index + 1 < columns {
            column_index += 1;
            column_height = Length::ZERO;
        }
        groups[column_index].push(*entry);
        column_height += height;
    }

    groups
}

/// Number of columns the greedy packing needs to keep every column at or below
/// `target` height (a new column opens when the next block would overflow
/// `target` or carries a mandatory boundary).  A single block taller than
/// `target` occupies its own column without forcing infeasibility on its own.
fn balance_columns_needed(entries: &[BalanceEntry], target: Length) -> usize {
    if entries.is_empty() {
        return 0;
    }
    let mut columns = 1usize;
    let mut column_height = Length::ZERO;
    for (i, entry) in entries.iter().enumerate() {
        let height = entry.occupied.max(Length::ZERO);
        let must_break = entry.forces_column_boundary && i > 0;
        let would_overflow = column_height + height > target && column_height > Length::ZERO;
        if must_break || would_overflow {
            columns += 1;
            column_height = Length::ZERO;
        }
        column_height += height;
    }
    columns
}

/// Reparent each block in `group` (in order) under `new_region`, restacking
/// them from the body top while preserving the original inter-block spacing.
/// Returns the resulting stacking cursor (the new page's `current_y`).
fn migrate_blocks(
    area_tree: &mut AreaTree,
    group: &[AreaId],
    new_region: AreaId,
) -> Result<Length> {
    let mut new_y = Length::ZERO;
    let mut prev_old_bottom: Option<Length> = None;

    for &block_id in group {
        let (old_y, height) = match area_tree.get(block_id) {
            Some(n) => (n.area.geometry.y, n.area.height()),
            None => continue,
        };

        // Preserve the gap (space-before) that separated this block from its
        // predecessor; the first block of the group drops its leading space and
        // sits flush at the body top.
        let gap = match prev_old_bottom {
            Some(prev_bottom) => (old_y - prev_bottom).max(Length::ZERO),
            None => Length::ZERO,
        };
        new_y += gap;

        area_tree
            .reparent(block_id, new_region)
            .map_err(fop_types::FopError::Generic)?;
        if let Some(node) = area_tree.get_mut(block_id) {
            node.area.geometry.y = new_y;
        }

        new_y += height;
        prev_old_bottom = Some(old_y + height);
    }

    Ok(new_y)
}

#[cfg(test)]
#[path = "pagination_tests.rs"]
mod tests;
