//! Page number resolution for fo:page-number-citation
//!
//! This module implements a two-pass layout system:
//! Pass 1: Layout everything, track element IDs → page mappings
//! Pass 2: Resolve citations and update area content with actual page numbers

use crate::area::AreaId;
use std::collections::HashMap;

/// Tracks page numbers for elements with IDs and resolves page-number-citations
pub struct PageNumberResolver {
    /// Maps element ID to page number
    id_to_page: HashMap<String, usize>,

    /// Maps element ID to the page sequence format string when registered
    id_to_format: HashMap<String, String>,

    /// Maps element ID to area ID (for tracking during layout)
    id_to_area: HashMap<String, AreaId>,

    /// List of citations that need to be resolved (area_id, ref_id)
    citations: Vec<(AreaId, String)>,

    /// Current page number during layout
    current_page: usize,

    /// Current page number format for fo:page-sequence format attribute
    current_format: String,

    /// Current grouping separator (e.g. ',' for "1,000")
    current_grouping_separator: Option<char>,

    /// Current grouping size (e.g. 3 for "1,000")
    current_grouping_size: Option<usize>,
}

impl PageNumberResolver {
    /// Create a new page number resolver
    pub fn new() -> Self {
        Self {
            id_to_page: HashMap::new(),
            id_to_format: HashMap::new(),
            id_to_area: HashMap::new(),
            citations: Vec::new(),
            current_page: 1,
            current_format: "1".to_string(),
            current_grouping_separator: None,
            current_grouping_size: None,
        }
    }

    /// Set the current page number
    pub fn set_current_page(&mut self, page: usize) {
        self.current_page = page;
    }

    /// Get the current page number
    pub fn current_page(&self) -> usize {
        self.current_page
    }

    /// Set the current page number format
    pub fn set_current_format(&mut self, format: String) {
        self.current_format = format;
    }

    /// Get the current page number format
    pub fn current_format(&self) -> &str {
        &self.current_format
    }

    /// Register an element with an ID on the current page
    pub fn register_element(&mut self, id: String, area_id: AreaId) {
        self.id_to_page.insert(id.clone(), self.current_page);
        self.id_to_format
            .insert(id.clone(), self.current_format.clone());
        self.id_to_area.insert(id, area_id);
    }

    /// Get the page number format that was active when the element was registered
    pub fn get_format_for_id(&self, id: &str) -> Option<&str> {
        self.id_to_format.get(id).map(|s| s.as_str())
    }

    /// Set the current grouping separator
    pub fn set_current_grouping_separator(&mut self, sep: Option<char>) {
        self.current_grouping_separator = sep;
    }

    /// Get the current grouping separator
    pub fn current_grouping_separator(&self) -> Option<char> {
        self.current_grouping_separator
    }

    /// Set the current grouping size
    pub fn set_current_grouping_size(&mut self, size: Option<usize>) {
        self.current_grouping_size = size;
    }

    /// Get the current grouping size
    pub fn current_grouping_size(&self) -> Option<usize> {
        self.current_grouping_size
    }

    /// Register a page-number-citation that needs to be resolved
    pub fn register_citation(&mut self, area_id: AreaId, ref_id: String) {
        self.citations.push((area_id, ref_id));
    }

    /// Get the page number for a referenced element
    pub fn get_page_number(&self, ref_id: &str) -> Option<usize> {
        self.id_to_page.get(ref_id).copied()
    }

    /// Get all citations that need to be resolved
    pub fn get_citations(&self) -> &[(AreaId, String)] {
        &self.citations
    }

    /// Check if all citations can be resolved
    pub fn can_resolve_all(&self) -> bool {
        self.citations
            .iter()
            .all(|(_, ref_id)| self.id_to_page.contains_key(ref_id))
    }

    /// Get a list of unresolved citations
    pub fn unresolved_citations(&self) -> Vec<String> {
        self.citations
            .iter()
            .filter_map(|(_, ref_id)| {
                if !self.id_to_page.contains_key(ref_id) {
                    Some(ref_id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Clear all registered data (for a new layout pass)
    pub fn clear(&mut self) {
        self.id_to_page.clear();
        self.id_to_format.clear();
        self.id_to_area.clear();
        self.citations.clear();
        self.current_page = 1;
        self.current_format = "1".to_string();
        self.current_grouping_separator = None;
        self.current_grouping_size = None;
    }
}

impl Default for PageNumberResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolver_creation() {
        let resolver = PageNumberResolver::new();
        assert_eq!(resolver.current_page(), 1);
        assert!(resolver.can_resolve_all());
    }

    #[test]
    fn test_register_element() {
        let mut resolver = PageNumberResolver::new();
        resolver.set_current_page(5);

        let area_id = AreaId::from_index(10);
        resolver.register_element("chapter1".to_string(), area_id);

        assert_eq!(resolver.get_page_number("chapter1"), Some(5));
    }

    #[test]
    fn test_register_citation() {
        let mut resolver = PageNumberResolver::new();
        let area_id = AreaId::from_index(20);

        resolver.register_citation(area_id, "chapter1".to_string());

        assert_eq!(resolver.get_citations().len(), 1);
        assert_eq!(
            resolver.get_citations()[0],
            (area_id, "chapter1".to_string())
        );
    }

    #[test]
    fn test_can_resolve_all() {
        let mut resolver = PageNumberResolver::new();
        let area_id = AreaId::from_index(10);

        // Register a citation
        resolver.register_citation(area_id, "chapter1".to_string());
        assert!(!resolver.can_resolve_all());

        // Register the referenced element
        resolver.register_element("chapter1".to_string(), area_id);
        assert!(resolver.can_resolve_all());
    }

    #[test]
    fn test_unresolved_citations() {
        let mut resolver = PageNumberResolver::new();
        let area_id = AreaId::from_index(10);

        resolver.register_citation(area_id, "chapter1".to_string());
        resolver.register_citation(area_id, "chapter2".to_string());

        // Only register chapter1
        resolver.register_element("chapter1".to_string(), area_id);

        let unresolved = resolver.unresolved_citations();
        assert_eq!(unresolved.len(), 1);
        assert_eq!(unresolved[0], "chapter2");
    }

    #[test]
    fn test_clear() {
        let mut resolver = PageNumberResolver::new();
        let area_id = AreaId::from_index(10);

        resolver.register_element("chapter1".to_string(), area_id);
        resolver.register_citation(area_id, "chapter1".to_string());
        resolver.set_current_page(5);

        resolver.clear();

        assert_eq!(resolver.current_page(), 1);
        assert_eq!(resolver.get_page_number("chapter1"), None);
        assert_eq!(resolver.get_citations().len(), 0);
    }
}

#[cfg(test)]
mod extended_tests {
    use super::*;

    #[test]
    fn test_resolver_default_same_as_new() {
        let r1 = PageNumberResolver::new();
        let r2 = PageNumberResolver::default();
        assert_eq!(r1.current_page(), r2.current_page());
        assert_eq!(r1.current_format(), r2.current_format());
    }

    #[test]
    fn test_set_current_page() {
        let mut resolver = PageNumberResolver::new();
        resolver.set_current_page(10);
        assert_eq!(resolver.current_page(), 10);
    }

    #[test]
    fn test_register_element_tracks_format() {
        let mut resolver = PageNumberResolver::new();
        resolver.set_current_page(3);
        resolver.set_current_format("i".to_string());

        let area_id = AreaId::from_index(5);
        resolver.register_element("section-2".to_string(), area_id);

        // The format for that id should be "i"
        assert_eq!(resolver.get_format_for_id("section-2"), Some("i"));
    }

    #[test]
    fn test_get_format_for_unregistered_id_returns_none() {
        let resolver = PageNumberResolver::new();
        assert_eq!(resolver.get_format_for_id("nonexistent"), None);
    }

    #[test]
    fn test_register_element_multiple_pages() {
        let mut resolver = PageNumberResolver::new();

        let area1 = AreaId::from_index(1);
        let area2 = AreaId::from_index(2);

        resolver.set_current_page(1);
        resolver.register_element("sec-1".to_string(), area1);

        resolver.set_current_page(5);
        resolver.register_element("sec-5".to_string(), area2);

        assert_eq!(resolver.get_page_number("sec-1"), Some(1));
        assert_eq!(resolver.get_page_number("sec-5"), Some(5));
    }

    #[test]
    fn test_get_page_number_for_unregistered_returns_none() {
        let resolver = PageNumberResolver::new();
        assert_eq!(resolver.get_page_number("missing"), None);
    }

    #[test]
    fn test_can_resolve_all_empty_citations() {
        let resolver = PageNumberResolver::new();
        // No citations => can resolve all
        assert!(resolver.can_resolve_all());
    }

    #[test]
    fn test_can_resolve_all_with_unresolved() {
        let mut resolver = PageNumberResolver::new();
        let area_id = AreaId::from_index(1);
        resolver.register_citation(area_id, "unknown-id".to_string());
        assert!(!resolver.can_resolve_all());
    }

    #[test]
    fn test_unresolved_citations_empty_when_all_resolved() {
        let mut resolver = PageNumberResolver::new();
        let area_id = AreaId::from_index(1);
        resolver.register_element("para-1".to_string(), area_id);
        resolver.register_citation(area_id, "para-1".to_string());

        let unresolved = resolver.unresolved_citations();
        assert!(unresolved.is_empty());
    }

    #[test]
    fn test_clear_resets_format_to_default() {
        let mut resolver = PageNumberResolver::new();
        resolver.set_current_format("I".to_string());
        resolver.clear();
        assert_eq!(resolver.current_format(), "1");
    }

    #[test]
    fn test_clear_resets_grouping() {
        let mut resolver = PageNumberResolver::new();
        resolver.set_current_grouping_separator(Some(','));
        resolver.set_current_grouping_size(Some(3));
        resolver.clear();
        assert_eq!(resolver.current_grouping_separator(), None);
        assert_eq!(resolver.current_grouping_size(), None);
    }

    #[test]
    fn test_grouping_separator_get_set() {
        let mut resolver = PageNumberResolver::new();
        assert_eq!(resolver.current_grouping_separator(), None);
        resolver.set_current_grouping_separator(Some('.'));
        assert_eq!(resolver.current_grouping_separator(), Some('.'));
    }

    #[test]
    fn test_grouping_size_get_set() {
        let mut resolver = PageNumberResolver::new();
        assert_eq!(resolver.current_grouping_size(), None);
        resolver.set_current_grouping_size(Some(3));
        assert_eq!(resolver.current_grouping_size(), Some(3));
    }

    #[test]
    fn test_multiple_citations_some_resolved_some_not() {
        let mut resolver = PageNumberResolver::new();
        let area_id = AreaId::from_index(10);

        resolver.register_element("known".to_string(), area_id);
        resolver.register_citation(area_id, "known".to_string());
        resolver.register_citation(area_id, "unknown-a".to_string());
        resolver.register_citation(area_id, "unknown-b".to_string());

        assert!(!resolver.can_resolve_all());

        let unresolved = resolver.unresolved_citations();
        assert_eq!(unresolved.len(), 2);
        assert!(unresolved.contains(&"unknown-a".to_string()));
        assert!(unresolved.contains(&"unknown-b".to_string()));
    }

    #[test]
    fn test_register_citation_count() {
        let mut resolver = PageNumberResolver::new();
        let area_id = AreaId::from_index(1);

        resolver.register_citation(area_id, "ref-1".to_string());
        resolver.register_citation(area_id, "ref-2".to_string());
        resolver.register_citation(area_id, "ref-3".to_string());

        assert_eq!(resolver.get_citations().len(), 3);
    }

    #[test]
    fn test_clear_removes_all_citations() {
        let mut resolver = PageNumberResolver::new();
        let area_id = AreaId::from_index(1);
        resolver.register_citation(area_id, "ref-1".to_string());
        resolver.register_citation(area_id, "ref-2".to_string());

        resolver.clear();
        assert_eq!(resolver.get_citations().len(), 0);
        assert!(resolver.can_resolve_all());
    }
}
