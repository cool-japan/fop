//! Page-accurate `fo:marker` collection and `fo:retrieve-marker` resolution.
//!
//! XSL-FO running headers/footers pull their content from `fo:marker`s placed
//! in the flow via `fo:retrieve-marker`.  Which marker a given page's static
//! content retrieves depends on **where the marker's content actually landed**:
//! a marker "starts on" the page whose area its generating block was placed on.
//! Because the pagination engine does not split a single flow block across
//! pages (a block lands wholly on one page — see the block-splitting deferral in
//! `pagination.rs`), every marker therefore both *starts* and *ends* on exactly
//! one page: the page its nearest enclosing flow block was placed on.
//!
//! This module records, per page of a `fo:page-sequence`, the markers (by
//! `marker-class-name`, in document order) that start on that page, and resolves
//! `fo:retrieve-marker` against that per-page context honouring the four
//! `retrieve-position` values and the `retrieve-boundary` scope.  Carry-over (a
//! marker from an earlier page still in effect on a page that sets none of its
//! own) is handled for the `page-sequence` and `document` boundaries.

use crate::area::{AreaId, AreaTree};
use fop_core::{tree::RetrievePosition, FoArena, FoNodeData, NodeId, PropertyId, PropertyList};
use std::collections::HashMap;

/// `retrieve-boundary` scope (XSL-FO 1.1 §6.11.4).
///
/// Controls how far back carry-over markers are searched for when the page
/// being formatted does not itself set a qualifying marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RetrieveBoundaryScope {
    /// `page` — only markers that start on the page being formatted qualify;
    /// no carry-over.
    Page,
    /// `page-sequence` — carry-over is searched within the current
    /// `fo:page-sequence` (the default).
    PageSequence,
    /// `document` — carry-over may reach back into earlier page-sequences.
    Document,
}

impl RetrieveBoundaryScope {
    /// Read the `retrieve-boundary` off an `fo:retrieve-marker`'s property list.
    ///
    /// The value is stored as a [`PropertyValue::String`](fop_core::PropertyValue)
    /// when written explicitly, and as the initial `Enum` (`page-sequence`) when
    /// omitted, so both encodings are accepted.
    pub(super) fn from_properties(properties: &PropertyList<'_>) -> Self {
        if let Ok(value) = properties.get(PropertyId::RetrieveBoundary) {
            if let Some(text) = value.as_string() {
                return match text {
                    "page" => Self::Page,
                    "document" => Self::Document,
                    _ => Self::PageSequence,
                };
            }
            if let Some(enum_val) = value.as_enum() {
                return match enum_val {
                    96 => Self::Page,        // EN_PAGE
                    34 => Self::Document,    // EN_DOCUMENT
                    _ => Self::PageSequence, // EN_PAGE_SEQUENCE (97) and any other
                };
            }
        }
        Self::PageSequence
    }
}

/// Document-level marker carry-over threaded across `fo:page-sequence`s.
///
/// Holds, per `marker-class-name`, the marker still in effect at the end of the
/// most recently formatted page-sequence.  It is consumed as the `incoming`
/// carry-over of the next sequence so that `retrieve-boundary="document"` can
/// reach a marker set in an earlier page-sequence.
#[derive(Debug, Default)]
pub(super) struct DocumentMarkerState {
    trailing: HashMap<String, NodeId>,
}

impl DocumentMarkerState {
    /// Create empty document marker state (no markers yet in effect).
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// Snapshot the markers currently in effect (to seed a sequence's
    /// `document`-boundary carry-over).
    pub(super) fn snapshot(&self) -> HashMap<String, NodeId> {
        self.trailing.clone()
    }

    /// Replace the in-effect markers with those trailing a finished sequence.
    pub(super) fn set_trailing(&mut self, trailing: HashMap<String, NodeId>) {
        self.trailing = trailing;
    }
}

/// Page-accurate marker context for a single `fo:page-sequence`.
///
/// `pages[i]` maps each `marker-class-name` to the markers (FO node ids, in
/// document order) that **start on page `i`** of the sequence.  `incoming`
/// carries the marker in effect per class at the end of all *previous*
/// page-sequences, used only when resolving with `retrieve-boundary="document"`.
#[derive(Debug)]
pub(super) struct SequenceMarkers {
    pages: Vec<HashMap<String, Vec<NodeId>>>,
    incoming: HashMap<String, NodeId>,
}

impl SequenceMarkers {
    /// Create an empty per-page context for `num_pages` pages, seeded with the
    /// document-level carry-over (`incoming`) from earlier sequences.
    fn new(num_pages: usize, incoming: HashMap<String, NodeId>) -> Self {
        Self {
            pages: vec![HashMap::new(); num_pages],
            incoming,
        }
    }

    /// Record a marker of `class` that starts on page `page_idx`.
    fn record(&mut self, page_idx: usize, class: &str, node_id: NodeId) {
        if let Some(page) = self.pages.get_mut(page_idx) {
            page.entry(class.to_string()).or_default().push(node_id);
        }
    }

    /// The marker of `class` still in effect at the *start* of `page_idx`: the
    /// last marker on the most recent earlier page (within this sequence) that
    /// set one. `None` when no earlier page in the sequence set the class.
    fn carryover_within_sequence(&self, page_idx: usize, class: &str) -> Option<NodeId> {
        (0..page_idx).rev().find_map(|earlier| {
            self.pages
                .get(earlier)
                .and_then(|page| page.get(class))
                .and_then(|markers| markers.last().copied())
        })
    }

    /// Resolve an `fo:retrieve-marker` on page `page_idx`.
    ///
    /// Honours the four `retrieve-position` values and the `retrieve-boundary`
    /// scope.  Returns the FO node id of the selected `fo:marker`, or `None`
    /// when no marker qualifies (e.g. a `starting-within-page` position on a
    /// page that sets no marker of the class).
    pub(super) fn resolve(
        &self,
        page_idx: usize,
        class: &str,
        position: RetrievePosition,
        boundary: RetrieveBoundaryScope,
    ) -> Option<NodeId> {
        let on_page = self.pages.get(page_idx).and_then(|page| page.get(class));
        let first_on_page = on_page.and_then(|markers| markers.first().copied());
        let last_on_page = on_page.and_then(|markers| markers.last().copied());

        let carryover = match boundary {
            RetrieveBoundaryScope::Page => None,
            RetrieveBoundaryScope::PageSequence => self.carryover_within_sequence(page_idx, class),
            RetrieveBoundaryScope::Document => self
                .carryover_within_sequence(page_idx, class)
                .or_else(|| self.incoming.get(class).copied()),
        };

        match position {
            // Only markers that start on the page qualify; no carry-over.
            RetrievePosition::FirstStartingWithinPage => first_on_page,
            RetrievePosition::LastStartingWithinPage => last_on_page,
            // The carried-over marker (attached to an area that began on an
            // earlier page) precedes the page's own markers in document order,
            // so it is the "first" when present.
            RetrievePosition::FirstIncludingCarryover => carryover.or(first_on_page),
            // The last marker that ends on the page; if none ends here, the
            // carried-over marker is still in effect and is used.
            RetrievePosition::LastEndingWithinPage => last_on_page.or(carryover),
        }
    }

    /// The marker in effect per class at the **end** of this sequence, seeding
    /// the next sequence's `document`-boundary carry-over.  Classes untouched by
    /// this sequence retain their previous (`incoming`) value.
    pub(super) fn trailing_markers(&self) -> HashMap<String, NodeId> {
        let mut trailing = self.incoming.clone();
        for page in &self.pages {
            for (class, markers) in page {
                if let Some(last) = markers.last() {
                    trailing.insert(class.clone(), *last);
                }
            }
        }
        trailing
    }
}

/// A read-only view of one page's marker context, handed to static-content
/// layout so each `fo:retrieve-marker` resolves against the page it is repeated
/// on.
pub(super) struct PageMarkerView<'a> {
    markers: &'a SequenceMarkers,
    page_idx: usize,
}

impl<'a> PageMarkerView<'a> {
    /// Build a view onto page `page_idx` of `markers`.
    pub(super) fn new(markers: &'a SequenceMarkers, page_idx: usize) -> Self {
        Self { markers, page_idx }
    }

    /// Resolve a retrieve-marker on this page (see [`SequenceMarkers::resolve`]).
    pub(super) fn resolve(
        &self,
        class: &str,
        position: RetrievePosition,
        boundary: RetrieveBoundaryScope,
    ) -> Option<NodeId> {
        self.markers
            .resolve(self.page_idx, class, position, boundary)
    }
}

/// Build the per-page marker context for a finished page-sequence.
///
/// `page_ids` are the sequence's page areas in order; `placements` records, in
/// document order, each top-level flow item's placed area together with its FO
/// node id.  Each placed item's final page is found through the area tree
/// (robust to overflow reparenting), and every `fo:marker` in that item's FO
/// subtree is recorded against that page — yielding, per page, the markers that
/// start on it in document order.
pub(super) fn collect_sequence_markers(
    fo_tree: &FoArena,
    area_tree: &AreaTree,
    page_ids: &[AreaId],
    placements: &[(AreaId, NodeId)],
    incoming: HashMap<String, NodeId>,
) -> SequenceMarkers {
    let mut markers = SequenceMarkers::new(page_ids.len(), incoming);

    let page_index: HashMap<AreaId, usize> = page_ids
        .iter()
        .enumerate()
        .map(|(idx, &page_id)| (page_id, idx))
        .collect();

    for &(area_id, node_id) in placements {
        let page_idx = match area_tree
            .find_page_ancestor(area_id)
            .and_then(|page| page_index.get(&page).copied())
        {
            Some(idx) => idx,
            None => continue,
        };
        collect_markers_in_subtree(fo_tree, node_id, &mut |class, marker_node| {
            markers.record(page_idx, class, marker_node);
        });
    }

    markers
}

/// Visit every `fo:marker` in `node_id`'s subtree in document order, invoking
/// `sink(marker_class_name, marker_node_id)` for each.
fn collect_markers_in_subtree(
    fo_tree: &FoArena,
    node_id: NodeId,
    sink: &mut impl FnMut(&str, NodeId),
) {
    if let Some(node) = fo_tree.get(node_id) {
        if let FoNodeData::Marker {
            marker_class_name, ..
        } = &node.data
        {
            sink(marker_class_name, node_id);
        }
        for child_id in fo_tree.children(node_id) {
            collect_markers_in_subtree(fo_tree, child_id, sink);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fop_core::tree::RetrievePosition;

    /// Marker node ids are fabricated from raw indices; only their identity
    /// (not any arena lookup) matters for the resolution table.
    fn nid(index: usize) -> NodeId {
        NodeId::from_index(index)
    }

    /// Build one page's `class -> [marker node]` map from `(class, &[idx])` rows.
    fn page(rows: &[(&str, &[usize])]) -> HashMap<String, Vec<NodeId>> {
        rows.iter()
            .map(|(class, ids)| (class.to_string(), ids.iter().map(|&i| nid(i)).collect()))
            .collect()
    }

    fn sequence(pages: Vec<HashMap<String, Vec<NodeId>>>) -> SequenceMarkers {
        SequenceMarkers {
            pages,
            incoming: HashMap::new(),
        }
    }

    /// On a page carrying two markers of the same class, `first-starting` and
    /// `last-starting` select the first and last respectively (and differ).
    #[test]
    fn first_and_last_starting_within_page_differ() {
        let markers = sequence(vec![page(&[("sec", &[10, 11])])]);
        assert_eq!(
            markers.resolve(
                0,
                "sec",
                RetrievePosition::FirstStartingWithinPage,
                RetrieveBoundaryScope::PageSequence,
            ),
            Some(nid(10)),
            "first-starting must select the first marker on the page"
        );
        assert_eq!(
            markers.resolve(
                0,
                "sec",
                RetrievePosition::LastStartingWithinPage,
                RetrieveBoundaryScope::PageSequence,
            ),
            Some(nid(11)),
            "last-starting must select the last marker on the page"
        );
    }

    /// `*-starting-within-page` positions never carry over: on a page that sets
    /// no marker of the class they resolve to nothing.
    #[test]
    fn starting_positions_do_not_carry_over() {
        let markers = sequence(vec![page(&[("sec", &[10])]), page(&[])]);
        assert_eq!(
            markers.resolve(
                1,
                "sec",
                RetrievePosition::FirstStartingWithinPage,
                RetrieveBoundaryScope::PageSequence,
            ),
            None,
            "first-starting must not see the previous page's marker"
        );
        assert_eq!(
            markers.resolve(
                1,
                "sec",
                RetrievePosition::LastStartingWithinPage,
                RetrieveBoundaryScope::PageSequence,
            ),
            None,
            "last-starting must not see the previous page's marker"
        );
    }

    /// On a page that sets no marker, `first-including-carryover` and
    /// `last-ending-within-page` fall back to the last marker in effect from an
    /// earlier page of the sequence.
    #[test]
    fn carryover_positions_use_prior_in_effect_marker() {
        let markers = sequence(vec![page(&[("sec", &[10, 11])]), page(&[])]);
        assert_eq!(
            markers.resolve(
                1,
                "sec",
                RetrievePosition::FirstIncludingCarryover,
                RetrieveBoundaryScope::PageSequence,
            ),
            Some(nid(11)),
            "carryover is the last marker in effect at the end of the prior page"
        );
        assert_eq!(
            markers.resolve(
                1,
                "sec",
                RetrievePosition::LastEndingWithinPage,
                RetrieveBoundaryScope::PageSequence,
            ),
            Some(nid(11)),
            "last-ending falls back to the carried-over marker"
        );
    }

    /// On a chapter-start page (which sets its own marker) the carried-over
    /// marker from the previous chapter precedes the new one, so
    /// `first-including-carryover` returns the prior chapter while
    /// `first-starting-within-page` returns the new one.
    #[test]
    fn first_including_carryover_prefers_prior_chapter_on_a_new_chapter_page() {
        let markers = sequence(vec![page(&[("sec", &[10])]), page(&[("sec", &[20])])]);
        assert_eq!(
            markers.resolve(
                1,
                "sec",
                RetrievePosition::FirstIncludingCarryover,
                RetrieveBoundaryScope::PageSequence,
            ),
            Some(nid(10)),
            "carryover (previous chapter) is first in document order"
        );
        assert_eq!(
            markers.resolve(
                1,
                "sec",
                RetrievePosition::FirstStartingWithinPage,
                RetrieveBoundaryScope::PageSequence,
            ),
            Some(nid(20)),
            "first-starting picks the marker that starts on this page"
        );
    }

    /// `retrieve-boundary="page"` confines resolution to the page being
    /// formatted: there is no carry-over from earlier pages.
    #[test]
    fn boundary_page_has_no_carryover() {
        let markers = sequence(vec![page(&[("sec", &[10])]), page(&[])]);
        assert_eq!(
            markers.resolve(
                1,
                "sec",
                RetrievePosition::FirstIncludingCarryover,
                RetrieveBoundaryScope::Page,
            ),
            None,
            "boundary=page must not reach the previous page's marker"
        );
    }

    /// `retrieve-boundary="document"` can reach a marker in effect from an
    /// earlier page-sequence (`incoming`), where `page-sequence` cannot.
    #[test]
    fn boundary_document_reaches_incoming_from_earlier_sequence() {
        let mut incoming = HashMap::new();
        incoming.insert("sec".to_string(), nid(99));
        let markers = SequenceMarkers {
            pages: vec![page(&[])],
            incoming,
        };
        assert_eq!(
            markers.resolve(
                0,
                "sec",
                RetrievePosition::FirstIncludingCarryover,
                RetrieveBoundaryScope::Document,
            ),
            Some(nid(99)),
            "document boundary reaches a prior sequence's trailing marker"
        );
        assert_eq!(
            markers.resolve(
                0,
                "sec",
                RetrievePosition::FirstIncludingCarryover,
                RetrieveBoundaryScope::PageSequence,
            ),
            None,
            "page-sequence boundary stops at the start of this sequence"
        );
    }

    /// `trailing_markers` reports the last marker in effect per class across the
    /// whole sequence, retaining incoming classes the sequence never touched.
    #[test]
    fn trailing_markers_report_last_in_effect_per_class() {
        let mut incoming = HashMap::new();
        incoming.insert("kept".to_string(), nid(1));
        let markers = SequenceMarkers {
            pages: vec![page(&[("sec", &[10])]), page(&[("sec", &[11])])],
            incoming,
        };
        let trailing = markers.trailing_markers();
        assert_eq!(
            trailing.get("sec"),
            Some(&nid(11)),
            "the latest page's marker is the sequence's trailing marker"
        );
        assert_eq!(
            trailing.get("kept"),
            Some(&nid(1)),
            "classes untouched by the sequence retain their incoming value"
        );
    }

    /// `retrieve-boundary` parses from both the explicit string encoding and the
    /// initial enum value, defaulting to `page-sequence`.
    #[test]
    fn retrieve_boundary_parses_string_and_default() {
        use fop_core::{PropertyList, PropertyValue};
        use std::borrow::Cow;

        let mut explicit = PropertyList::new();
        explicit.set(
            PropertyId::RetrieveBoundary,
            PropertyValue::String(Cow::Borrowed("page")),
        );
        assert_eq!(
            RetrieveBoundaryScope::from_properties(&explicit),
            RetrieveBoundaryScope::Page
        );

        // Unset → initial value (page-sequence).
        let defaulted = PropertyList::new();
        assert_eq!(
            RetrieveBoundaryScope::from_properties(&defaulted),
            RetrieveBoundaryScope::PageSequence
        );
    }
}
