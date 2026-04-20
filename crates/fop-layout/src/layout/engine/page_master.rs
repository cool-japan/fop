//! Page master selection logic for the layout engine.
//!
//! Handles `fo:page-sequence-master`, `fo:repeatable-page-master-alternatives`,
//! and `fo:conditional-page-master-reference` processing.

use fop_core::{FoArena, FoNodeData, NodeId};
use fop_types::Result;

use super::types::PageContext;
use super::LayoutEngine;

impl LayoutEngine {
    /// Select the appropriate page master based on page context and conditions
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
