//! Page breaking logic
//!
//! Splits content across multiple pages when it exceeds available space.

use crate::area::{Area, AreaId, AreaTree, AreaType};
use fop_types::{Length, Point, Rect, Result, Size};

/// Page breaker - splits content into pages
pub struct PageBreaker {
    /// Page width
    page_width: Length,

    /// Page height
    page_height: Length,

    /// Content area margins (top, right, bottom, left)
    margins: [Length; 4],
}

impl PageBreaker {
    /// Create a new page breaker
    pub fn new(page_width: Length, page_height: Length, margins: [Length; 4]) -> Self {
        Self {
            page_width,
            page_height,
            margins,
        }
    }

    /// Calculate available content height
    pub fn content_height(&self) -> Length {
        self.page_height - self.margins[0] - self.margins[2] // height - top - bottom
    }

    /// Calculate available content height accounting for footnotes
    pub fn content_height_with_footnotes(&self, footnote_height: Length) -> Length {
        self.content_height() - footnote_height
    }

    /// Calculate available content width
    pub fn content_width(&self) -> Length {
        self.page_width - self.margins[1] - self.margins[3] // width - right - left
    }

    /// Break an area tree into pages
    pub fn break_into_pages(
        &self,
        area_tree: &mut AreaTree,
        root_id: AreaId,
    ) -> Result<Vec<AreaId>> {
        let mut page_ids = Vec::new();
        let content_height = self.content_height();
        let _content_width = self.content_width();

        // Get all block-level children
        let children = area_tree.children(root_id);

        if children.is_empty() {
            // No content, create one empty page
            let page_id = self.create_page(area_tree)?;
            page_ids.push(page_id);
            return Ok(page_ids);
        }

        // Implement multi-page breaking with overflow detection and keep constraints
        let mut current_page_id = self.create_page(area_tree)?;
        page_ids.push(current_page_id);

        let mut current_height = Length::ZERO;

        for (idx, child_id) in children.iter().enumerate() {
            // Extract all needed information before any mutable operations
            let (child_height, break_before_opt, break_after_opt) =
                if let Some(child_node) = area_tree.get(*child_id) {
                    (
                        child_node.area.height(),
                        child_node.area.break_before,
                        child_node.area.break_after,
                    )
                } else {
                    continue;
                };

            // Check for forced break-before
            let mut force_break_before = false;
            let mut need_even_page_before = false;
            let mut need_odd_page_before = false;

            if let Some(break_before) = break_before_opt {
                if break_before.forces_page_break() {
                    force_break_before = true;
                    need_even_page_before = break_before.requires_even_page();
                    need_odd_page_before = break_before.requires_odd_page();
                }
            }

            // Handle even/odd page requirements before break
            if need_even_page_before {
                let current_page_num = page_ids.len();
                if current_page_num % 2 == 1 {
                    // Current page is odd, need to insert blank page
                    current_page_id = self.create_page(area_tree)?;
                    page_ids.push(current_page_id);
                }
            } else if need_odd_page_before {
                let current_page_num = page_ids.len();
                if current_page_num % 2 == 0 {
                    // Current page is even, need to insert blank page
                    current_page_id = self.create_page(area_tree)?;
                    page_ids.push(current_page_id);
                }
            }

            // Check if we can break before this area
            let can_break = self.can_break_before(area_tree, *child_id, idx, &children);

            // Check if content overflows current page or forced break
            if ((current_height + child_height > content_height
                && current_height > Length::ZERO
                && can_break)
                || force_break_before)
                && current_height > Length::ZERO
            {
                // Create new page
                current_page_id = self.create_page(area_tree)?;
                page_ids.push(current_page_id);
                current_height = Length::ZERO;
            }

            // Reparent the child under the current page's region-body and
            // position it at the running vertical offset (region-relative y).
            // This is the real reparenting that the previous implementation
            // only pretended to do.
            let region_id = area_tree
                .children(current_page_id)
                .into_iter()
                .next()
                .ok_or_else(|| {
                    fop_types::FopError::Generic(
                        "page area is missing its region-body child".to_string(),
                    )
                })?;
            area_tree
                .reparent(*child_id, region_id)
                .map_err(fop_types::FopError::Generic)?;
            if let Some(child_node) = area_tree.get_mut(*child_id) {
                child_node.area.geometry.y = current_height;
            }
            current_height += child_height;

            // Check for forced break-after
            if let Some(break_after) = break_after_opt {
                if break_after.forces_page_break() {
                    // Create new page for next content
                    current_page_id = self.create_page(area_tree)?;
                    page_ids.push(current_page_id);
                    current_height = Length::ZERO;

                    // Handle even/odd page requirements after break
                    if break_after.requires_even_page() {
                        let current_page_num = page_ids.len();
                        if current_page_num % 2 == 1 {
                            // Current page is odd, need to insert blank page
                            current_page_id = self.create_page(area_tree)?;
                            page_ids.push(current_page_id);
                        }
                    } else if break_after.requires_odd_page() {
                        let current_page_num = page_ids.len();
                        if current_page_num % 2 == 0 {
                            // Current page is even, need to insert blank page
                            current_page_id = self.create_page(area_tree)?;
                            page_ids.push(current_page_id);
                        }
                    }
                }
            }
        }

        Ok(page_ids)
    }

    /// Check whether a page break may occur *before* the given area, honouring
    /// keep constraints (`keep-together`, `keep-with-previous` on this area, and
    /// `keep-with-next` on the preceding area).
    ///
    /// Returns `false` when a keep constraint forbids breaking before this
    /// area, `true` otherwise. Used by both [`PageBreaker::break_into_pages`]
    /// and the layout engine's pagination loop to decide whether an overflowing
    /// block may start a new page on its own or must drag glued neighbours with
    /// it. See the note inside about why widows/orphans do not participate.
    pub fn can_break_before(
        &self,
        area_tree: &AreaTree,
        area_id: AreaId,
        index: usize,
        all_children: &[AreaId],
    ) -> bool {
        // Get the current area
        let current_area = match area_tree.get(area_id) {
            Some(node) => &node.area,
            None => return true, // If area doesn't exist, allow break
        };

        // Check keep-together constraint (don't split this area)
        if let Some(constraint) = &current_area.keep_constraint {
            if constraint.must_keep_together() {
                // For keep-together, we would need to track if we're in the middle
                // of splitting. For now, we prevent breaks if area is too large.
                // This is a simplified implementation.
                return false;
            }

            // Check keep-with-previous constraint
            if constraint.must_keep_with_previous() && index > 0 {
                // Don't break before this area if it has keep-with-previous
                return false;
            }
        }

        // Check if previous area has keep-with-next constraint
        if index > 0 {
            if let Some(prev_area_id) = all_children.get(index - 1) {
                if let Some(prev_node) = area_tree.get(*prev_area_id) {
                    if let Some(constraint) = &prev_node.area.keep_constraint {
                        if constraint.must_keep_with_next() {
                            // Previous area wants to stay with this one
                            return false;
                        }
                    }
                }
            }
        }

        // Widows/orphans intentionally do NOT gate this decision.
        //
        // This predicate answers "may a page break occur *before* this whole
        // block?" — i.e. the block moves to the next page in one piece. Widow
        // and orphan control only constrains where a block may be *split*
        // across a page boundary, which is a separate (currently deferred)
        // concern; applying the line-count test here would wrongly glue every
        // block shorter than the widow/orphan minimum to its predecessor. The
        // `widows`/`orphans` traits remain recorded on each area for the future
        // block-splitting implementation, and `count_line_areas` is retained as
        // a public helper for that work.

        // No keep constraint prevents breaking before this block.
        true
    }

    /// Count the number of line areas within a block area.
    ///
    /// Counts `Line`, `Text`, and `Inline` areas that are direct or indirect
    /// children of the given area. Retained as a public helper for the future
    /// widow/orphan-aware block-splitting implementation (it no longer gates
    /// [`PageBreaker::can_break_before`], which now considers keep constraints
    /// only — see the note there).
    #[allow(clippy::only_used_in_recursion)]
    pub fn count_line_areas(&self, area_tree: &AreaTree, area_id: AreaId) -> i32 {
        let mut count = 0;

        // Get the area
        if let Some(node) = area_tree.get(area_id) {
            // If this is a line or text area, count it
            if matches!(
                node.area.area_type,
                AreaType::Line | AreaType::Text | AreaType::Inline
            ) {
                count += 1;
            }

            // Recursively count in children
            let children = area_tree.children(area_id);
            for child_id in children {
                count += self.count_line_areas(area_tree, child_id);
            }
        }

        count
    }

    /// Create a new page area
    fn create_page(&self, area_tree: &mut AreaTree) -> Result<AreaId> {
        // Create page area
        let page_rect =
            Rect::from_point_size(Point::ZERO, Size::new(self.page_width, self.page_height));
        let page_area = Area::new(AreaType::Page, page_rect);
        let page_id = area_tree.add_area(page_area);

        // Create region-body area (content area with margins)
        let region_rect = Rect::from_point_size(
            Point::new(self.margins[3], self.margins[0]), // x = left margin, y = top margin
            Size::new(self.content_width(), self.content_height()),
        );
        let region_area = Area::new(AreaType::Region, region_rect);
        let region_id = area_tree.add_area(region_area);

        // Attach region to page
        area_tree
            .append_child(page_id, region_id)
            .map_err(fop_types::FopError::Generic)?;

        Ok(page_id)
    }

    /// Place footnotes at bottom of page with separator line
    pub fn place_footnotes(&self, area_tree: &mut AreaTree, page_id: AreaId) -> Result<()> {
        let footnotes = match area_tree.get_footnotes(page_id) {
            Some(f) if !f.is_empty() => f.clone(),
            _ => return Ok(()), // No footnotes
        };

        // Calculate separator position (at bottom of main content)
        let footnote_total_height = area_tree.footnote_height(page_id);
        let separator_y = self.margins[0] + self.content_height() - footnote_total_height;

        // Create footnote separator (thin horizontal line)
        let separator_rect = Rect::from_point_size(
            Point::new(self.margins[3], separator_y),
            Size::new(Length::from_pt(72.0), Length::from_pt(1.0)), // 1 inch wide, 1pt thick
        );
        let mut separator_area = Area::new(AreaType::FootnoteSeparator, separator_rect);

        // Set separator line style (thin black line)
        use crate::area::{BorderStyle, TraitSet};
        use fop_types::Color;
        let traits = TraitSet {
            border_width: Some([
                Length::from_pt(1.0),
                Length::ZERO,
                Length::ZERO,
                Length::ZERO,
            ]),
            border_color: Some([Color::BLACK, Color::BLACK, Color::BLACK, Color::BLACK]),
            border_style: Some([
                BorderStyle::Solid,
                BorderStyle::None,
                BorderStyle::None,
                BorderStyle::None,
            ]),
            ..Default::default()
        };
        separator_area.traits = traits;

        let separator_id = area_tree.add_area(separator_area);
        area_tree
            .append_child(page_id, separator_id)
            .map_err(fop_types::FopError::Generic)?;

        // Place footnotes below separator
        let mut current_y = separator_y + Length::from_pt(7.0); // Separator + spacing

        for footnote_id in footnotes {
            // Get height first, then mutate
            let footnote_height = if let Some(footnote_node) = area_tree.get(footnote_id) {
                footnote_node.area.height()
            } else {
                continue;
            };

            if let Some(footnote_node) = area_tree.get_mut(footnote_id) {
                // Position footnote
                footnote_node.area.geometry.x = self.margins[3];
                footnote_node.area.geometry.y = current_y;
            }

            // Attach footnote to page
            area_tree
                .append_child(page_id, footnote_id)
                .map_err(fop_types::FopError::Generic)?;

            current_y += footnote_height;
        }

        Ok(())
    }

    /// Check if content fits on current page
    pub fn fits_on_page(&self, current_height: Length, content_height: Length) -> bool {
        current_height + content_height <= self.content_height()
    }

    /// Choose a widow/orphan-valid split index for a block whose direct line
    /// boxes are described (in stacking order) by `(top, bottom)` pairs
    /// (block-relative, in points-as-`Length`).
    ///
    /// The returned `Some(k)` means *"keep lines `[0, k)` on the current page
    /// (the head) and carry lines `[k, n)` to the continuation (the tail)"*; a
    /// `None` means the block must **not** be split and should move whole.
    ///
    /// # Algorithm
    ///
    /// `available_height` is the vertical space the block may occupy on the
    /// current page (from the block's top edge down to the region-body bottom).
    /// Let `n` be the number of line boxes.
    ///
    /// 1. **Fit count** — the largest `f` such that the first `f` lines all sit
    ///    fully within `available_height` (`bottom[f-1] <= available_height`).
    ///    This is the *most* head lines the page can physically hold.
    /// 2. **Widows** — the tail must contain at least `widows` lines, so the
    ///    head may keep at most `n - widows` lines.  Cap the head accordingly.
    /// 3. **Orphans** — the head must contain at least `orphans` lines.  If the
    ///    capped head already has fewer than `orphans` lines, no split point can
    ///    satisfy both constraints simultaneously, so we refuse to split
    ///    (`None`) — the caller then moves the whole block to the next page.
    /// 4. The chosen `k` is the smallest of *fit count* and *widow cap*, which
    ///    is the largest head that fits while still leaving enough widows.  It
    ///    is `>= orphans` by step 3 and yields a tail `>= widows`, so both
    ///    constraints hold.
    ///
    /// When `empty_page` is `true` (the block is being split off an otherwise
    /// empty region-body — e.g. it is taller than a full body), the orphan
    /// **floor** is relaxed: keeping the block whole would only clip it, so we
    /// keep as many head lines as fit (down to a single line) to guarantee
    /// forward progress.  We still honour orphans when the page can hold at
    /// least `orphans` lines; we only drop below it when the page genuinely
    /// cannot hold that many.  Widows are still honoured on an empty page when
    /// possible, but never at the cost of placing zero head lines.
    pub fn widow_orphan_split_index(
        line_bottoms: &[Length],
        available_height: Length,
        widows: i32,
        orphans: i32,
        empty_page: bool,
    ) -> Option<usize> {
        let n = line_bottoms.len();
        if n == 0 {
            return None;
        }
        let widows = widows.max(0) as usize;
        let orphans = orphans.max(0) as usize;

        // Step 1: how many leading lines physically fit above the body bottom.
        let mut fit_count = 0usize;
        for &bottom in line_bottoms {
            if bottom <= available_height {
                fit_count += 1;
            } else {
                break;
            }
        }

        if !empty_page {
            // A non-empty page with no fitting line cannot host any head: defer
            // the whole block to the next page.
            if fit_count == 0 {
                return None;
            }
            // Step 2: cap the head so at least `widows` lines remain as tail.
            // If the block has fewer than `widows + orphans` lines it can never
            // be split legally — `widow_cap < orphans` triggers the `None`
            // below.
            let widow_cap = n.saturating_sub(widows);
            let head = fit_count.min(widow_cap);
            // Step 3: orphans floor.  Also reject a degenerate "tail is empty"
            // (head == n) which would not actually split anything.
            if head < orphans || head == 0 || head >= n {
                return None;
            }
            Some(head)
        } else {
            // Empty-page split: we *must* make progress.  Keep as many head
            // lines as fit; if not even one fits (pathological — a single line
            // taller than the whole body), keep exactly one so the block always
            // advances and the remainder continues onto the next page.
            let mut head = fit_count.max(1).min(n);
            // Honour widows when the page can still hold a full head while
            // leaving `widows` lines behind, but never reduce the head to zero.
            let widow_cap = n.saturating_sub(widows);
            if widow_cap >= 1 {
                head = head.min(widow_cap);
            }
            // Honour orphans only when the page can hold at least `orphans`
            // lines (otherwise the floor is physically impossible and we keep
            // what fits).  Never grow the head beyond what fits — that would
            // reintroduce overflow.
            if orphans >= 1 && fit_count >= orphans {
                head = head.max(orphans).min(fit_count.max(1));
            }
            head = head.clamp(1, n);
            if head >= n {
                // Everything fits after all — nothing to split.
                return None;
            }
            Some(head)
        }
    }

    /// Split the block area `area_id` at a line-box boundary, moving the tail
    /// line boxes into a freshly created **continuation block** and returning
    /// its id (`Ok(Some(continuation_id))`).  Returns `Ok(None)` when the block
    /// must not be split — either it already fits, it has no line-box children,
    /// `keep-together="always"` forbids splitting, or widow/orphan control
    /// admits no legal split point — in which case the caller moves the whole
    /// block to the next page.
    ///
    /// `available_height` is the space the block may occupy on the current page
    /// (its top edge to the region-body bottom).  `empty_page` should be `true`
    /// when the block is the first content of an otherwise empty region-body
    /// (it relaxes the orphan floor so a block taller than a full body still
    /// makes progress rather than clipping).
    ///
    /// # Geometry surgery
    ///
    /// The block's direct children are line boxes (`Line` / `Text` / `Inline`)
    /// whose `geometry.y` is block-relative and increasing.  The head lines
    /// `[0, k)` stay in place; the tail lines `[k, n)` are reparented under the
    /// continuation block and re-offset so the first tail line sits at `y = 0`.
    /// The original block's height shrinks to the head extent; the continuation
    /// inherits the block's `area_type`, width, x and traits (so background,
    /// borders and padding are preserved on the broken area).  The border on
    /// the break edge is intentionally left on **both** halves: full
    /// break-edge border conditionality (`border-*-conditional`) is a separate,
    /// deliberately out-of-scope refinement; keeping both edges never loses a
    /// painted border and matches Apache FOP's default conditional behaviour
    /// closely enough for the common (no explicit conditionality) case.
    pub fn split_area(
        &self,
        area_tree: &mut AreaTree,
        area_id: AreaId,
        available_height: Length,
        empty_page: bool,
    ) -> Result<Option<AreaId>> {
        // Snapshot what we need from the block before mutating the tree.
        let (block_area, area_height, widows, orphans, keeps_together) =
            match area_tree.get(area_id) {
                Some(node) => (
                    node.area.clone(),
                    node.area.height(),
                    node.area.widows,
                    node.area.orphans,
                    node.area
                        .keep_constraint
                        .as_ref()
                        .map(|c| c.must_keep_together())
                        .unwrap_or(false),
                ),
                None => return Ok(None),
            };

        // `keep-together="always"` forbids splitting the block.
        if keeps_together {
            return Ok(None);
        }

        // If the whole block already fits, there is nothing to split.
        if area_height <= available_height {
            return Ok(None);
        }

        // Gather the direct line-box children (the splittable units) in order,
        // along with their block-relative top/bottom edges.  Non-line children
        // are not split points and are kept with whichever side their preceding
        // line ends up on; in practice a text block's direct children are all
        // line boxes, so this collects exactly the lines.
        let mut line_ids: Vec<AreaId> = Vec::new();
        let mut line_tops: Vec<Length> = Vec::new();
        let mut line_bottoms: Vec<Length> = Vec::new();
        for child_id in area_tree.children(area_id) {
            if let Some(child) = area_tree.get(child_id) {
                if matches!(
                    child.area.area_type,
                    AreaType::Line | AreaType::Text | AreaType::Inline
                ) {
                    let top = child.area.geometry.y;
                    line_ids.push(child_id);
                    line_tops.push(top);
                    line_bottoms.push(top + child.area.height());
                }
            }
        }

        // A block with no line-box children (e.g. an image/leaf block) cannot
        // be split at a line boundary — defer it whole.
        if line_ids.len() < 2 {
            return Ok(None);
        }

        let split = match Self::widow_orphan_split_index(
            &line_bottoms,
            available_height,
            widows,
            orphans,
            empty_page,
        ) {
            Some(k) => k,
            None => return Ok(None),
        };

        // Head extent: the bottom edge of the last head line bounds the height
        // the original block now occupies on the current page.
        let head_bottom = line_bottoms[split - 1];
        // Tail origin: the top of the first tail line; tail lines are shifted up
        // by this so the continuation's first line sits flush at its top.
        let tail_origin = line_tops[split];

        // Build the continuation block, preserving area_type, width, x and
        // traits (background / borders / padding) of the broken area.  Its
        // height is the remaining tail extent; its y is left at the broken
        // block's y for now and is overwritten by the caller when it places the
        // continuation on the next page (region-relative).
        let mut continuation = block_area;
        continuation.content = None; // line boxes carry the content
        let tail_height = (area_height - tail_origin).max(Length::ZERO);
        continuation.geometry.height = tail_height;
        // A continuation block must not re-emit a forced break-before that was
        // meant for the block's original start, and keeps no break-after until
        // its true end (which it inherits, since it is the real end of the
        // block).  break-before only applies to the first piece.
        continuation.break_before = None;
        let continuation_id = area_tree.add_area(continuation);

        // Move the tail line boxes into the continuation, re-offsetting them so
        // the first tail line starts at y = 0 within the continuation block.
        for &line_id in &line_ids[split..] {
            area_tree
                .reparent(line_id, continuation_id)
                .map_err(fop_types::FopError::Generic)?;
            if let Some(node) = area_tree.get_mut(line_id) {
                node.area.geometry.y -= tail_origin;
            }
        }

        // Shrink the original (head) block to the head extent and clear its
        // break-after — the break-after belongs to the block's true end, now on
        // the continuation.
        if let Some(node) = area_tree.get_mut(area_id) {
            node.area.geometry.height = head_bottom;
            node.area.break_after = None;
        }

        Ok(Some(continuation_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_breaker_dimensions() {
        let breaker = PageBreaker::new(
            Length::from_mm(210.0), // A4 width
            Length::from_mm(297.0), // A4 height
            [
                Length::from_mm(20.0), // top
                Length::from_mm(20.0), // right
                Length::from_mm(20.0), // bottom
                Length::from_mm(20.0), // left
            ],
        );

        let content_width = breaker.content_width();
        let content_height = breaker.content_height();

        assert_eq!(content_width, Length::from_mm(170.0)); // 210 - 20 - 20
        assert_eq!(content_height, Length::from_mm(257.0)); // 297 - 20 - 20
    }

    #[test]
    fn test_create_empty_page() {
        let breaker = PageBreaker::new(
            Length::from_pt(595.0),
            Length::from_pt(842.0),
            [Length::from_pt(72.0); 4],
        );

        let mut tree = AreaTree::new();
        let page_id = breaker
            .create_page(&mut tree)
            .expect("test: should succeed");

        assert!(tree.get(page_id).is_some());
        let page_node = tree.get(page_id).expect("test: should succeed");
        assert_eq!(page_node.area.area_type, AreaType::Page);
    }

    #[test]
    fn test_break_empty_tree() {
        let breaker = PageBreaker::new(
            Length::from_pt(595.0),
            Length::from_pt(842.0),
            [Length::from_pt(72.0); 4],
        );

        let mut tree = AreaTree::new();
        let root = Area::new(
            AreaType::Block,
            Rect::from_point_size(Point::ZERO, Size::new(Length::ZERO, Length::ZERO)),
        );
        let root_id = tree.add_area(root);

        let pages = breaker
            .break_into_pages(&mut tree, root_id)
            .expect("test: should succeed");

        // Should create at least one page
        assert_eq!(pages.len(), 1);
    }

    #[test]
    fn test_overflow_detection() {
        let breaker = PageBreaker::new(
            Length::from_pt(595.0),
            Length::from_pt(842.0),
            [Length::from_pt(72.0); 4], // 1 inch margins
        );

        // Content height: 842 - 72 - 72 = 698pt
        assert_eq!(breaker.content_height(), Length::from_pt(698.0));

        // Check if content fits
        assert!(breaker.fits_on_page(Length::from_pt(100.0), Length::from_pt(200.0)));
        assert!(!breaker.fits_on_page(Length::from_pt(600.0), Length::from_pt(200.0)));
    }

    #[test]
    fn test_multi_page_breaking() {
        let breaker = PageBreaker::new(
            Length::from_pt(595.0),
            Length::from_pt(842.0),
            [Length::from_pt(72.0); 4],
        );

        let mut tree = AreaTree::new();
        let root = Area::new(
            AreaType::Block,
            Rect::from_point_size(Point::ZERO, Size::new(Length::ZERO, Length::ZERO)),
        );
        let root_id = tree.add_area(root);

        // Add blocks that overflow one page
        for _ in 0..5 {
            let block = Area::new(
                AreaType::Block,
                Rect::from_point_size(
                    Point::ZERO,
                    Size::new(Length::from_pt(400.0), Length::from_pt(200.0)),
                ),
            );
            let block_id = tree.add_area(block);
            tree.append_child(root_id, block_id)
                .expect("test: should succeed");
        }

        let pages = breaker
            .break_into_pages(&mut tree, root_id)
            .expect("test: should succeed");

        // With 5 blocks of 200pt each (1000pt total) and content height 698pt,
        // should create at least 2 pages
        assert!(pages.len() >= 2);

        // Real reparenting: the blocks must no longer hang off the root — they
        // were moved under the pages' region-body areas.
        assert!(
            tree.children(root_id).is_empty(),
            "all blocks must be reparented away from the root"
        );

        // Every page's single region-body must hold the blocks assigned to it,
        // each positioned with a region-relative y that fits the content box,
        // and the union of all region children must be exactly the 5 blocks.
        let content_height = breaker.content_height();
        let mut reparented_blocks = 0;
        for page_id in &pages {
            let regions = tree.children(*page_id);
            assert_eq!(regions.len(), 1, "each page has exactly one region-body");
            let region_id = regions[0];
            assert_eq!(
                tree.get(region_id)
                    .expect("test: region exists")
                    .area
                    .area_type,
                AreaType::Region
            );
            for block_id in tree.children(region_id) {
                let block = tree.get(block_id).expect("test: block exists");
                assert_eq!(block.area.area_type, AreaType::Block);
                // Block's bottom edge must lie within the region content box.
                let bottom = block.area.geometry.y + block.area.height();
                assert!(
                    bottom <= content_height,
                    "reparented block overflows its page region: bottom {}pt > {}pt",
                    bottom.to_pt(),
                    content_height.to_pt()
                );
                reparented_blocks += 1;
            }
        }
        assert_eq!(
            reparented_blocks, 5,
            "all 5 blocks must be reparented under a page region"
        );
    }

    /// Helper: build a Block populated with `count` stacked line boxes of
    /// `line_height` each, returning the block id.  Mirrors the structure the
    /// line-breaker emits (direct `Text` children with block-relative y).
    fn make_lined_block(
        tree: &mut AreaTree,
        count: usize,
        line_height: Length,
        width: Length,
    ) -> AreaId {
        let block = Area::new(
            AreaType::Block,
            Rect::from_point_size(Point::ZERO, Size::new(width, line_height * count as i32)),
        );
        let block_id = tree.add_area(block);
        for i in 0..count {
            let line = Area::new(
                AreaType::Line,
                Rect::new(Length::ZERO, line_height * i as i32, width, line_height),
            );
            let line_id = tree.add_area(line);
            tree.append_child(block_id, line_id)
                .expect("test: append line");
        }
        block_id
    }

    #[test]
    fn test_split_area_leaf_block_is_not_split() {
        // A block with no line-box children cannot be split at a line boundary.
        let breaker = PageBreaker::new(
            Length::from_pt(595.0),
            Length::from_pt(842.0),
            [Length::from_pt(72.0); 4],
        );

        let mut tree = AreaTree::new();
        let large_block = Area::new(
            AreaType::Block,
            Rect::from_point_size(
                Point::ZERO,
                Size::new(Length::from_pt(400.0), Length::from_pt(800.0)),
            ),
        );
        let block_id = tree.add_area(large_block);

        let continuation = breaker
            .split_area(&mut tree, block_id, Length::from_pt(300.0), false)
            .expect("test: should succeed");
        assert!(
            continuation.is_none(),
            "a childless block has no line boundary to split at"
        );
    }

    #[test]
    fn test_split_area_honours_orphans_and_widows() {
        // 10 lines of 12pt = 120pt tall; 50pt available => 4 lines fit.
        // widows = orphans = 2, so head = min(fit=4, n-widows=8) = 4 (>= 2),
        // tail = 6 (>= 2): split at index 4.
        let breaker = PageBreaker::new(
            Length::from_pt(595.0),
            Length::from_pt(842.0),
            [Length::from_pt(72.0); 4],
        );
        let mut tree = AreaTree::new();
        let line_h = Length::from_pt(12.0);
        let block_id = make_lined_block(&mut tree, 10, line_h, Length::from_pt(400.0));

        let cont = breaker
            .split_area(&mut tree, block_id, Length::from_pt(50.0), false)
            .expect("test: should succeed")
            .expect("test: block must split");

        // Head keeps 4 lines; tail (continuation) keeps 6.
        assert_eq!(tree.children(block_id).len(), 4, "head line count");
        assert_eq!(tree.children(cont).len(), 6, "tail line count");

        // Head block height shrinks to 4 * 12 = 48pt; continuation = 6 * 12.
        assert_eq!(
            tree.get(block_id).expect("test: head").area.height(),
            Length::from_pt(48.0)
        );
        assert_eq!(
            tree.get(cont).expect("test: tail").area.height(),
            Length::from_pt(72.0)
        );

        // First tail line is re-offset to y = 0 within the continuation.
        let first_tail = tree.children(cont)[0];
        assert_eq!(
            tree.get(first_tail).expect("test: line").area.geometry.y,
            Length::ZERO
        );
    }

    #[test]
    fn test_split_area_refused_when_fewer_than_widows_plus_orphans() {
        // 3 lines, widows = orphans = 2 => widows + orphans = 4 > 3, so no legal
        // split point exists; the block must move whole (None).
        let breaker = PageBreaker::new(
            Length::from_pt(595.0),
            Length::from_pt(842.0),
            [Length::from_pt(72.0); 4],
        );
        let mut tree = AreaTree::new();
        let line_h = Length::from_pt(12.0);
        // Force overflow: only 12pt available (1 line fits) but 3 lines tall.
        let block_id = make_lined_block(&mut tree, 3, line_h, Length::from_pt(400.0));

        let cont = breaker
            .split_area(&mut tree, block_id, Length::from_pt(12.0), false)
            .expect("test: should succeed");
        assert!(
            cont.is_none(),
            "block with fewer than widows+orphans lines must not split"
        );
        // Block is untouched: still 3 lines.
        assert_eq!(tree.children(block_id).len(), 3);
    }

    #[test]
    fn test_split_area_taller_than_empty_page_splits() {
        // A block taller than the full body must split on an empty page rather
        // than clip.  60 lines of 12pt = 720pt; body content height = 698pt =>
        // 58 lines fit. widow cap = 60 - 2 = 58, so head = 58, tail = 2.
        let breaker = PageBreaker::new(
            Length::from_pt(595.0),
            Length::from_pt(842.0),
            [Length::from_pt(72.0); 4],
        );
        let mut tree = AreaTree::new();
        let line_h = Length::from_pt(12.0);
        let block_id = make_lined_block(&mut tree, 60, line_h, Length::from_pt(400.0));
        let avail = breaker.content_height(); // 698pt, full empty body

        let cont = breaker
            .split_area(&mut tree, block_id, avail, true)
            .expect("test: should succeed")
            .expect("test: tall block must split on empty page");

        assert_eq!(tree.children(block_id).len(), 58, "head fits 58 lines");
        assert_eq!(tree.children(cont).len(), 2, "tail carries the remainder");
    }

    #[test]
    fn test_split_area_keep_together_not_split() {
        use crate::layout::{Keep, KeepConstraint};

        let breaker = PageBreaker::new(
            Length::from_pt(595.0),
            Length::from_pt(842.0),
            [Length::from_pt(72.0); 4],
        );
        let mut tree = AreaTree::new();
        let line_h = Length::from_pt(12.0);
        let block_id = make_lined_block(&mut tree, 10, line_h, Length::from_pt(400.0));

        // Apply keep-together="always".
        let mut constraint = KeepConstraint::new();
        constraint.keep_together = Keep::Always;
        tree.get_mut(block_id)
            .expect("test: block")
            .area
            .keep_constraint = Some(constraint);

        let cont = breaker
            .split_area(&mut tree, block_id, Length::from_pt(50.0), false)
            .expect("test: should succeed");
        assert!(
            cont.is_none(),
            "keep-together block must never be split (moves whole)"
        );
        assert_eq!(tree.children(block_id).len(), 10);
    }

    #[test]
    fn test_widow_orphan_split_index_unit() {
        // 10 lines, 4 fit, widows=orphans=2 -> head 4.
        let bottoms: Vec<Length> = (1..=10).map(|i| Length::from_pt(12.0 * i as f64)).collect();
        assert_eq!(
            PageBreaker::widow_orphan_split_index(&bottoms, Length::from_pt(50.0), 2, 2, false),
            Some(4)
        );
        // Widows force the head below the fit count: 9 fit, widows=3 -> head 7.
        assert_eq!(
            PageBreaker::widow_orphan_split_index(&bottoms, Length::from_pt(115.0), 3, 2, false),
            Some(7)
        );
        // Only 1 line fits but orphans=2: no legal split, defer whole.
        assert_eq!(
            PageBreaker::widow_orphan_split_index(&bottoms, Length::from_pt(12.0), 2, 2, false),
            None
        );
        // Empty page relaxes the orphan floor: 1 line fits -> head 1.
        assert_eq!(
            PageBreaker::widow_orphan_split_index(&bottoms, Length::from_pt(12.0), 2, 2, true),
            Some(1)
        );
    }

    #[test]
    fn test_keep_with_previous_prevents_break() {
        use crate::layout::{Keep, KeepConstraint};

        let breaker = PageBreaker::new(
            Length::from_pt(595.0),
            Length::from_pt(842.0),
            [Length::from_pt(72.0); 4],
        );

        let mut tree = AreaTree::new();

        // Create two blocks
        let block1 = Area::new(
            AreaType::Block,
            Rect::from_point_size(
                Point::ZERO,
                Size::new(Length::from_pt(400.0), Length::from_pt(200.0)),
            ),
        );
        let block1_id = tree.add_area(block1);

        let mut constraint = KeepConstraint::new();
        constraint.keep_with_previous = Keep::Always;

        let block2 = Area::new(
            AreaType::Block,
            Rect::from_point_size(
                Point::ZERO,
                Size::new(Length::from_pt(400.0), Length::from_pt(200.0)),
            ),
        )
        .with_keep_constraint(constraint);
        let block2_id = tree.add_area(block2);

        let children = vec![block1_id, block2_id];

        // Can break before first block (no previous)
        assert!(breaker.can_break_before(&tree, block1_id, 0, &children));

        // Cannot break before second block (has keep-with-previous)
        assert!(!breaker.can_break_before(&tree, block2_id, 1, &children));
    }

    #[test]
    fn test_keep_with_next_prevents_break() {
        use crate::layout::{Keep, KeepConstraint};

        let breaker = PageBreaker::new(
            Length::from_pt(595.0),
            Length::from_pt(842.0),
            [Length::from_pt(72.0); 4],
        );

        let mut tree = AreaTree::new();

        // Create two blocks
        let mut constraint = KeepConstraint::new();
        constraint.keep_with_next = Keep::Always;

        let block1 = Area::new(
            AreaType::Block,
            Rect::from_point_size(
                Point::ZERO,
                Size::new(Length::from_pt(400.0), Length::from_pt(200.0)),
            ),
        )
        .with_keep_constraint(constraint);
        let block1_id = tree.add_area(block1);

        let block2 = Area::new(
            AreaType::Block,
            Rect::from_point_size(
                Point::ZERO,
                Size::new(Length::from_pt(400.0), Length::from_pt(200.0)),
            ),
        );
        let block2_id = tree.add_area(block2);

        let children = vec![block1_id, block2_id];

        // Cannot break before second block (previous has keep-with-next)
        assert!(!breaker.can_break_before(&tree, block2_id, 1, &children));
    }

    #[test]
    fn test_keep_together_prevents_break() {
        use crate::layout::{Keep, KeepConstraint};

        let breaker = PageBreaker::new(
            Length::from_pt(595.0),
            Length::from_pt(842.0),
            [Length::from_pt(72.0); 4],
        );

        let mut tree = AreaTree::new();

        let mut constraint = KeepConstraint::new();
        constraint.keep_together = Keep::Always;

        let block = Area::new(
            AreaType::Block,
            Rect::from_point_size(
                Point::ZERO,
                Size::new(Length::from_pt(400.0), Length::from_pt(200.0)),
            ),
        )
        .with_keep_constraint(constraint);
        let block_id = tree.add_area(block);

        let children = vec![block_id];

        // Cannot break (keep-together is active)
        assert!(!breaker.can_break_before(&tree, block_id, 0, &children));
    }

    #[test]
    fn test_no_keep_allows_break() {
        let breaker = PageBreaker::new(
            Length::from_pt(595.0),
            Length::from_pt(842.0),
            [Length::from_pt(72.0); 4],
        );

        let mut tree = AreaTree::new();

        // Create blocks without keep constraints
        let block1 = Area::new(
            AreaType::Block,
            Rect::from_point_size(
                Point::ZERO,
                Size::new(Length::from_pt(400.0), Length::from_pt(200.0)),
            ),
        );
        let block1_id = tree.add_area(block1);

        let block2 = Area::new(
            AreaType::Block,
            Rect::from_point_size(
                Point::ZERO,
                Size::new(Length::from_pt(400.0), Length::from_pt(200.0)),
            ),
        );
        let block2_id = tree.add_area(block2);

        let children = vec![block1_id, block2_id];

        // Can break before both blocks (no constraints)
        assert!(breaker.can_break_before(&tree, block1_id, 0, &children));
        assert!(breaker.can_break_before(&tree, block2_id, 1, &children));
    }
}

#[cfg(test)]
mod extended_tests {
    use super::*;
    use crate::area::AreaType;

    fn make_breaker() -> PageBreaker {
        PageBreaker::new(
            Length::from_pt(595.0),
            Length::from_pt(842.0),
            [Length::from_pt(72.0); 4],
        )
    }

    #[test]
    fn test_content_width_calculation() {
        // 595 - 72 (right) - 72 (left) = 451
        let breaker = make_breaker();
        assert_eq!(breaker.content_width(), Length::from_pt(451.0));
    }

    #[test]
    fn test_content_height_calculation() {
        // 842 - 72 (top) - 72 (bottom) = 698
        let breaker = make_breaker();
        assert_eq!(breaker.content_height(), Length::from_pt(698.0));
    }

    #[test]
    fn test_content_height_with_footnotes_reduces_available() {
        let breaker = make_breaker();
        let base = breaker.content_height();
        let with_footnote = breaker.content_height_with_footnotes(Length::from_pt(50.0));
        assert_eq!(with_footnote, base - Length::from_pt(50.0));
    }

    #[test]
    fn test_fits_on_page_exactly_at_boundary() {
        let breaker = make_breaker();
        let content_h = breaker.content_height();
        // Exactly at boundary should fit
        assert!(breaker.fits_on_page(Length::ZERO, content_h));
    }

    #[test]
    fn test_fits_on_page_over_boundary_does_not_fit() {
        let breaker = make_breaker();
        let content_h = breaker.content_height();
        assert!(!breaker.fits_on_page(Length::from_pt(1.0), content_h));
    }

    #[test]
    fn test_split_area_fits_returns_none() {
        let breaker = make_breaker();
        let mut tree = AreaTree::new();
        // A block whose total height already fits never splits, regardless of
        // its line children.
        let block = Area::new(
            AreaType::Block,
            Rect::from_point_size(
                Point::ZERO,
                Size::new(Length::from_pt(400.0), Length::from_pt(100.0)),
            ),
        );
        let block_id = tree.add_area(block);
        for i in 0..5 {
            let line = Area::new(
                AreaType::Line,
                Rect::new(
                    Length::ZERO,
                    Length::from_pt(20.0 * i as f64),
                    Length::from_pt(400.0),
                    Length::from_pt(20.0),
                ),
            );
            let line_id = tree.add_area(line);
            tree.append_child(block_id, line_id)
                .expect("test: append line");
        }

        // 100pt fits in 200pt available => no split needed
        let result = breaker
            .split_area(&mut tree, block_id, Length::from_pt(200.0), false)
            .expect("test: should succeed");
        assert!(result.is_none());
    }

    #[test]
    fn test_split_area_overflow_creates_continuation() {
        let breaker = make_breaker();
        let mut tree = AreaTree::new();
        // 15 lines of 20pt = 300pt; 200pt available => 10 fit, widow cap 13,
        // head = 10, tail = 5.  Continuation carries the remaining 5 lines.
        let block = Area::new(
            AreaType::Block,
            Rect::from_point_size(
                Point::ZERO,
                Size::new(Length::from_pt(400.0), Length::from_pt(300.0)),
            ),
        );
        let block_id = tree.add_area(block);
        for i in 0..15 {
            let line = Area::new(
                AreaType::Line,
                Rect::new(
                    Length::ZERO,
                    Length::from_pt(20.0 * i as f64),
                    Length::from_pt(400.0),
                    Length::from_pt(20.0),
                ),
            );
            let line_id = tree.add_area(line);
            tree.append_child(block_id, line_id)
                .expect("test: append line");
        }

        let continuation = breaker
            .split_area(&mut tree, block_id, Length::from_pt(200.0), false)
            .expect("test: should succeed");
        assert!(continuation.is_some());

        let cont_id = continuation.expect("test: should succeed");
        let cont_node = tree.get(cont_id).expect("test: should succeed");
        // Continuation should have the remaining 5 lines => 100pt.
        assert_eq!(cont_node.area.height(), Length::from_pt(100.0));
        assert_eq!(tree.children(cont_id).len(), 5);
        assert_eq!(tree.children(block_id).len(), 10);
    }

    #[test]
    fn test_break_into_pages_single_block_fits() {
        let breaker = make_breaker();
        let mut tree = AreaTree::new();
        let root = Area::new(
            AreaType::Block,
            Rect::from_point_size(Point::ZERO, Size::new(Length::ZERO, Length::ZERO)),
        );
        let root_id = tree.add_area(root);

        let small_block = Area::new(
            AreaType::Block,
            Rect::from_point_size(
                Point::ZERO,
                Size::new(Length::from_pt(400.0), Length::from_pt(100.0)),
            ),
        );
        let block_id = tree.add_area(small_block);
        tree.append_child(root_id, block_id)
            .expect("test: should succeed");

        let pages = breaker
            .break_into_pages(&mut tree, root_id)
            .expect("test: should succeed");
        // All fits on one page
        assert_eq!(pages.len(), 1);
    }

    #[test]
    fn test_count_line_areas_in_block_with_children() {
        let breaker = make_breaker();
        let mut tree = AreaTree::new();

        // Create a block with line children
        let block = Area::new(
            AreaType::Block,
            Rect::from_point_size(
                Point::ZERO,
                Size::new(Length::from_pt(400.0), Length::from_pt(50.0)),
            ),
        );
        let block_id = tree.add_area(block);

        let line1 = Area::new(
            AreaType::Line,
            Rect::from_point_size(
                Point::ZERO,
                Size::new(Length::from_pt(400.0), Length::from_pt(12.0)),
            ),
        );
        let line2 = Area::new(
            AreaType::Line,
            Rect::from_point_size(
                Point::ZERO,
                Size::new(Length::from_pt(400.0), Length::from_pt(12.0)),
            ),
        );
        let line1_id = tree.add_area(line1);
        let line2_id = tree.add_area(line2);

        tree.append_child(block_id, line1_id)
            .expect("test: should succeed");
        tree.append_child(block_id, line2_id)
            .expect("test: should succeed");

        let count = breaker.count_line_areas(&tree, block_id);
        assert_eq!(count, 2, "Block with 2 line children should count 2");
    }

    #[test]
    fn test_page_has_region_child() {
        let breaker = make_breaker();
        let mut tree = AreaTree::new();
        let page_id = breaker
            .create_page(&mut tree)
            .expect("test: should succeed");

        let children = tree.children(page_id);
        assert_eq!(children.len(), 1, "Page should have one region child");

        let region_node = tree.get(children[0]).expect("test: should succeed");
        assert_eq!(region_node.area.area_type, AreaType::Region);
    }

    #[test]
    fn test_asymmetric_margins() {
        let breaker = PageBreaker::new(
            Length::from_pt(612.0), // US Letter width
            Length::from_pt(792.0), // US Letter height
            [
                Length::from_pt(72.0), // top 1 inch
                Length::from_pt(54.0), // right 0.75 inch
                Length::from_pt(72.0), // bottom 1 inch
                Length::from_pt(54.0), // left 0.75 inch
            ],
        );

        // content_width = 612 - 54 - 54 = 504
        // content_height = 792 - 72 - 72 = 648
        assert_eq!(breaker.content_width(), Length::from_pt(504.0));
        assert_eq!(breaker.content_height(), Length::from_pt(648.0));
    }
}
