//! Page master selection logic for the layout engine.
//!
//! Handles `fo:page-sequence-master`, `fo:repeatable-page-master-alternatives`,
//! and `fo:conditional-page-master-reference` processing.

use fop_core::{FoArena, FoNodeData, NodeId};
use fop_types::Result;

use super::types::PageContext;
use super::LayoutEngine;

impl LayoutEngine {
    /// Resolve a page-sequence `master-reference` to the concrete
    /// simple-page-master name to use for a single page.
    ///
    /// * When `master_reference` names a `fo:simple-page-master` directly, that
    ///   name is returned unchanged (the common case — behaviour identical to
    ///   the pre-conditional path).
    /// * When it names a `fo:page-sequence-master`, its
    ///   `fo:repeatable-page-master-alternatives` are evaluated against `context`
    ///   / `is_blank` and the first matching
    ///   `fo:conditional-page-master-reference` wins.  If no alternative matches
    ///   (a malformed master with no `any` fallback), the original reference is
    ///   returned so geometry resolution degrades to the A4 fallback rather than
    ///   panicking.
    ///
    /// Returns the concrete simple-page-master name that
    /// [`LayoutEngine::extract_page_region_geometry`](crate::layout::engine::LayoutEngine)
    /// should be asked to resolve.
    pub(super) fn resolve_page_master_for_page(
        &self,
        fo_tree: &FoArena,
        master_reference: &str,
        context: &PageContext,
        is_blank: bool,
    ) -> String {
        match self.select_page_master(fo_tree, master_reference, context, is_blank) {
            Ok(Some(name)) => name,
            // No `page-sequence-master` matched (either it is a plain
            // simple-page-master, or no conditional alternative matched): keep
            // the original reference.  A direct simple-page-master resolves
            // correctly; an unmatched sequence-master falls through to the
            // geometry resolver's A4 fallback.
            Ok(None) | Err(_) => master_reference.to_string(),
        }
    }

    /// Whether `master_reference` names a `fo:page-sequence-master` (as opposed
    /// to a `fo:simple-page-master`).  Used to decide whether per-page
    /// conditional master selection is needed for a page-sequence at all — when
    /// it names a simple-page-master the legacy single-geometry path is kept.
    pub(super) fn is_page_sequence_master(
        &self,
        fo_tree: &FoArena,
        master_reference: &str,
    ) -> bool {
        if let Some((root_id, _)) = fo_tree.root() {
            for child_id in fo_tree.children(root_id) {
                if let Some(child) = fo_tree.get(child_id) {
                    if matches!(child.data, FoNodeData::LayoutMasterSet) {
                        for master_id in fo_tree.children(child_id) {
                            if let Some(master) = fo_tree.get(master_id) {
                                if let FoNodeData::PageSequenceMaster { master_name } = &master.data
                                {
                                    if master_name == master_reference {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Select the appropriate page master based on page context and conditions
    pub(super) fn select_page_master(
        &self,
        fo_tree: &FoArena,
        master_reference: &str,
        context: &PageContext,
        _is_blank: bool,
    ) -> Result<Option<String>> {
        // First, look for the master reference in the layout-master-set
        if let Some((root_id, _)) = fo_tree.root() {
            let children = fo_tree.children(root_id);
            for child_id in children {
                if let Some(child) = fo_tree.get(child_id) {
                    if matches!(child.data, FoNodeData::LayoutMasterSet) {
                        // Found layout-master-set, search for the master reference
                        return self.find_page_master(
                            fo_tree,
                            child_id,
                            master_reference,
                            context,
                            _is_blank,
                        );
                    }
                }
            }
        }
        Ok(None)
    }

    /// Find the page master within the layout-master-set
    pub(super) fn find_page_master(
        &self,
        fo_tree: &FoArena,
        layout_master_set_id: NodeId,
        master_reference: &str,
        context: &PageContext,
        is_blank: bool,
    ) -> Result<Option<String>> {
        let children = fo_tree.children(layout_master_set_id);

        for child_id in children {
            if let Some(child) = fo_tree.get(child_id) {
                match &child.data {
                    // Direct simple-page-master reference
                    FoNodeData::SimplePageMaster { master_name, .. }
                        if master_name == master_reference =>
                    {
                        return Ok(Some(master_name.clone()));
                    }
                    // Page sequence master with alternatives
                    FoNodeData::PageSequenceMaster { master_name, .. }
                        if master_name == master_reference =>
                    {
                        // Look through alternatives to find matching conditions
                        let alternatives = fo_tree.children(child_id);
                        for alt_id in alternatives {
                            if let Some(alt) = fo_tree.get(alt_id) {
                                // fo:single-page-master-reference and
                                // fo:repeatable-page-master-reference are not yet
                                // represented as distinct FoNodeData variants;
                                // they are handled by falling through to the default.
                                if let FoNodeData::RepeatablePageMasterAlternatives { .. } =
                                    &alt.data
                                {
                                    if let Some(selected) = self.evaluate_conditional_masters(
                                        fo_tree, alt_id, context, is_blank,
                                    )? {
                                        return Ok(Some(selected));
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(None)
    }

    /// Evaluate conditional page master references and return the first matching one
    pub(super) fn evaluate_conditional_masters(
        &self,
        fo_tree: &FoArena,
        alternatives_id: NodeId,
        context: &PageContext,
        is_blank: bool,
    ) -> Result<Option<String>> {
        let children = fo_tree.children(alternatives_id);

        for child_id in children {
            if let Some(child) = fo_tree.get(child_id) {
                if let FoNodeData::ConditionalPageMasterReference {
                    master_reference,
                    page_position,
                    odd_or_even,
                    blank_or_not_blank,
                } = &child.data
                {
                    // Check if all conditions match
                    if self.matches_page_position(page_position, context)
                        && self.matches_odd_or_even(odd_or_even, context)
                        && self.matches_blank_or_not_blank(blank_or_not_blank, is_blank)
                    {
                        return Ok(Some(master_reference.clone()));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Check if page position matches
    pub(super) fn matches_page_position(
        &self,
        page_position: &fop_core::tree::PagePosition,
        context: &PageContext,
    ) -> bool {
        use fop_core::tree::PagePosition;
        match page_position {
            PagePosition::First => context.is_first_page(),
            PagePosition::Last => context.is_last_page(),
            PagePosition::Rest => !context.is_first_page() && !context.is_last_page(),
            PagePosition::Any => true,
        }
    }

    /// Check if odd/even matches
    pub(super) fn matches_odd_or_even(
        &self,
        odd_or_even: &fop_core::tree::OddOrEven,
        context: &PageContext,
    ) -> bool {
        use fop_core::tree::OddOrEven;
        match odd_or_even {
            OddOrEven::Odd => context.is_odd_page(),
            OddOrEven::Even => context.is_even_page(),
            OddOrEven::Any => true,
        }
    }

    /// Check if blank/not-blank matches
    pub(super) fn matches_blank_or_not_blank(
        &self,
        blank_or_not_blank: &fop_core::tree::BlankOrNotBlank,
        is_blank: bool,
    ) -> bool {
        use fop_core::tree::BlankOrNotBlank;
        match blank_or_not_blank {
            BlankOrNotBlank::Blank => is_blank,
            BlankOrNotBlank::NotBlank => !is_blank,
            BlankOrNotBlank::Any => true,
        }
    }
}
