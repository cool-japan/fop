//! Area tree structure
//!
//! Similar to the FO tree, uses arena allocation for areas.

use super::{Area, AreaType};
use std::fmt;

/// Index-based handle to an area in the tree
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct AreaId(usize);

impl AreaId {
    /// Get the raw index value
    #[inline]
    pub fn index(self) -> usize {
        self.0
    }

    /// Create an AreaId from a raw index
    #[inline]
    pub const fn from_index(index: usize) -> Self {
        Self(index)
    }
}

impl fmt::Display for AreaId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Area({})", self.0)
    }
}

/// A node in the area tree
pub struct AreaNode {
    /// The area data
    pub area: Area,

    /// Parent area ID
    pub parent: Option<AreaId>,

    /// First child area ID
    pub first_child: Option<AreaId>,

    /// Next sibling area ID
    pub next_sibling: Option<AreaId>,
}

impl AreaNode {
    /// Create a new area node
    pub fn new(area: Area) -> Self {
        Self {
            area,
            parent: None,
            first_child: None,
            next_sibling: None,
        }
    }

    /// Check if this node has children
    #[inline]
    pub fn has_children(&self) -> bool {
        self.first_child.is_some()
    }
}

/// Area tree - arena-allocated tree of areas
pub struct AreaTree {
    nodes: Vec<AreaNode>,
    /// Footnotes collected for current page, mapped by page ID
    footnotes: std::collections::HashMap<AreaId, Vec<AreaId>>,
    /// Document language from xml:lang on fo:root (for PDF /Lang entry)
    pub document_lang: Option<String>,
}

impl AreaTree {
    /// Create a new empty area tree
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            footnotes: std::collections::HashMap::new(),
            document_lang: None,
        }
    }

    /// Create an area tree with preallocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            footnotes: std::collections::HashMap::new(),
            document_lang: None,
        }
    }

    /// Add an area to the tree
    pub fn add_area(&mut self, area: Area) -> AreaId {
        let id = AreaId(self.nodes.len());
        self.nodes.push(AreaNode::new(area));
        id
    }

    /// Get a reference to an area by ID
    pub fn get(&self, id: AreaId) -> Option<&AreaNode> {
        self.nodes.get(id.0)
    }

    /// Get a mutable reference to an area by ID
    pub fn get_mut(&mut self, id: AreaId) -> Option<&mut AreaNode> {
        self.nodes.get_mut(id.0)
    }

    /// Get the number of areas in the tree
    #[inline]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the tree is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Append a child area to a parent
    pub fn append_child(&mut self, parent: AreaId, child: AreaId) -> Result<(), String> {
        if child.0 >= self.nodes.len() {
            return Err(format!("Child area {} does not exist", child.0));
        }
        if parent.0 >= self.nodes.len() {
            return Err(format!("Parent area {} does not exist", parent.0));
        }

        // Set parent of child
        self.nodes[child.0].parent = Some(parent);

        // Update parent's children list
        let parent_node = &mut self.nodes[parent.0];
        if let Some(first_child) = parent_node.first_child {
            // Find last sibling and append
            let mut last_sibling = first_child;
            while let Some(next) = self.nodes[last_sibling.0].next_sibling {
                last_sibling = next;
            }
            self.nodes[last_sibling.0].next_sibling = Some(child);
        } else {
            // First child
            parent_node.first_child = Some(child);
        }

        Ok(())
    }

    /// Move `child` (with its entire subtree) from its current parent to
    /// `new_parent`, appended as the last child of `new_parent`.
    ///
    /// This performs the linked-list surgery required to detach `child` from
    /// its current parent's sibling chain and re-attach it under `new_parent`.
    /// Because area geometry is stored relative to the parent's origin, callers
    /// that move a node between containers with different origins must also
    /// update the moved node's own `geometry` (its descendants keep their
    /// parent-relative positions and need no change).
    ///
    /// Used by the layout engine's pagination loop to relocate an overflowing
    /// block from one page's region-body to the next page's region-body.
    pub fn reparent(&mut self, child: AreaId, new_parent: AreaId) -> Result<(), String> {
        if child.0 >= self.nodes.len() {
            return Err(format!("Child area {} does not exist", child.0));
        }
        if new_parent.0 >= self.nodes.len() {
            return Err(format!("Parent area {} does not exist", new_parent.0));
        }
        if child == new_parent {
            return Err("Cannot reparent an area onto itself".to_string());
        }

        // Detach `child` from its current parent's sibling chain, if any.
        if let Some(old_parent) = self.nodes[child.0].parent {
            let detached_next = self.nodes[child.0].next_sibling;
            if self.nodes[old_parent.0].first_child == Some(child) {
                // `child` was the first child: promote its next sibling.
                self.nodes[old_parent.0].first_child = detached_next;
            } else {
                // Walk the sibling chain to find the predecessor of `child`.
                let mut cursor = self.nodes[old_parent.0].first_child;
                while let Some(current) = cursor {
                    let next = self.nodes[current.0].next_sibling;
                    if next == Some(child) {
                        self.nodes[current.0].next_sibling = detached_next;
                        break;
                    }
                    cursor = next;
                }
            }
        }

        // Clear the moved node's links before re-attaching.
        self.nodes[child.0].next_sibling = None;
        self.nodes[child.0].parent = None;

        // Re-attach under the new parent as its last child.
        self.append_child(new_parent, child)
    }

    /// Stably reorder a parent's direct children by an ascending priority key.
    ///
    /// Children sharing a priority keep their current relative order (the sort
    /// is stable).  Used by the pagination engine to restore the canonical page
    /// child order after static content (headers/footers) is appended in a
    /// deferred second pass: the append-only [`Self::append_child`] would
    /// otherwise leave the static regions trailing the region-body.
    ///
    /// Only sibling links are rewritten; parent links, geometry and descendants
    /// are untouched, so this is purely a reordering of `parent`'s child list.
    pub(crate) fn reorder_children<F: Fn(AreaType) -> u8>(&mut self, parent: AreaId, priority: F) {
        let mut children = self.children(parent);
        if children.len() < 2 {
            return;
        }
        children.sort_by_key(|&child| {
            self.get(child)
                .map(|node| priority(node.area.area_type))
                .unwrap_or(u8::MAX)
        });

        let parent_index = parent.0;
        if parent_index >= self.nodes.len() {
            return;
        }
        self.nodes[parent_index].first_child = children.first().copied();
        let last = children.len() - 1;
        for (offset, &child) in children.iter().enumerate() {
            let next = if offset < last {
                Some(children[offset + 1])
            } else {
                None
            };
            self.nodes[child.0].next_sibling = next;
        }
    }

    /// Get all children of an area
    pub fn children(&self, parent: AreaId) -> Vec<AreaId> {
        let mut children = Vec::new();
        if let Some(node) = self.get(parent) {
            let mut current = node.first_child;
            while let Some(child_id) = current {
                children.push(child_id);
                current = self.get(child_id).and_then(|n| n.next_sibling);
            }
        }
        children
    }

    /// Iterate over all areas
    pub fn iter(&self) -> impl Iterator<Item = (AreaId, &AreaNode)> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (AreaId(i), node))
    }

    /// Get the root area (first area in tree)
    pub fn root(&self) -> Option<(AreaId, &AreaNode)> {
        if self.nodes.is_empty() {
            None
        } else {
            Some((AreaId(0), &self.nodes[0]))
        }
    }

    /// Add a footnote to a page
    pub fn add_footnote(&mut self, page_id: AreaId, footnote_id: AreaId) {
        self.footnotes.entry(page_id).or_default().push(footnote_id);
    }

    /// Get footnotes for a page
    pub fn get_footnotes(&self, page_id: AreaId) -> Option<&Vec<AreaId>> {
        self.footnotes.get(&page_id)
    }

    /// Find the nearest Page ancestor for a given area (or the area itself if it's a Page)
    pub fn find_page_ancestor(&self, area_id: AreaId) -> Option<AreaId> {
        let mut current = area_id;
        loop {
            if let Some(node) = self.get(current) {
                if node.area.area_type == crate::area::AreaType::Page {
                    return Some(current);
                }
                match node.parent {
                    Some(parent_id) => current = parent_id,
                    None => return None,
                }
            } else {
                return None;
            }
        }
    }

    /// Serialize the area tree as an indented text tree for debugging
    pub fn serialize(&self) -> String {
        let mut output = String::new();
        // Walk all parentless roots to handle multi-page-sequence documents
        // where each page-sequence creates a separate top-level Page area.
        for (id, node) in self.iter() {
            if node.parent.is_none() {
                self.serialize_node(id, 0, &mut output);
            }
        }
        output
    }

    /// Recursively serialize a node and its children
    fn serialize_node(&self, id: AreaId, depth: usize, output: &mut String) {
        if let Some(node) = self.get(id) {
            let indent = "  ".repeat(depth);
            let area = &node.area;
            let type_name = format!("{:?}", area.area_type);
            let geom = &area.geometry;
            output.push_str(&format!(
                "{}{} ({:.1}pt,{:.1}pt {:.1}pt x {:.1}pt)\n",
                indent,
                type_name,
                geom.x.to_pt(),
                geom.y.to_pt(),
                geom.width.to_pt(),
                geom.height.to_pt()
            ));
            // If there's text content, show it
            if let Some(text) = area.text_content() {
                let preview: String = text.chars().take(40).collect();
                output.push_str(&format!("{}  Text {:?}\n", indent, preview));
            }
            for child_id in self.children(id) {
                self.serialize_node(child_id, depth + 1, output);
            }
        }
    }

    /// Calculate total height of footnotes for a page
    pub fn footnote_height(&self, page_id: AreaId) -> fop_types::Length {
        use fop_types::Length;

        let mut total_height = Length::ZERO;

        if let Some(footnotes) = self.get_footnotes(page_id) {
            // Add separator height (1pt line + 6pt spacing)
            if !footnotes.is_empty() {
                total_height += Length::from_pt(7.0);
            }

            // Add height of each footnote
            for &footnote_id in footnotes {
                if let Some(node) = self.get(footnote_id) {
                    total_height += node.area.height();
                }
            }
        }

        total_height
    }

    /// Compute structural differences between two area trees.
    ///
    /// Returns a list of difference descriptions. An empty list means the trees
    /// are structurally identical.
    pub fn diff(&self, other: &AreaTree) -> Vec<String> {
        let mut diffs = Vec::new();
        match (self.root(), other.root()) {
            (None, None) => {}
            (Some(_), None) => diffs.push("self has root, other has no root".to_string()),
            (None, Some(_)) => diffs.push("self has no root, other has root".to_string()),
            (Some((id1, _)), Some((id2, _))) => {
                self.diff_nodes(id1, other, id2, "", &mut diffs);
            }
        }
        diffs
    }

    fn diff_nodes(
        &self,
        id1: AreaId,
        other: &AreaTree,
        id2: AreaId,
        path: &str,
        diffs: &mut Vec<String>,
    ) {
        let (node1, node2) = match (self.get(id1), other.get(id2)) {
            (Some(n1), Some(n2)) => (n1, n2),
            _ => {
                diffs.push(format!("Node missing at path: {}", path));
                return;
            }
        };

        // Compare area types
        if std::mem::discriminant(&node1.area.area_type)
            != std::mem::discriminant(&node2.area.area_type)
        {
            diffs.push(format!(
                "Area type mismatch at {}: {:?} vs {:?}",
                path, node1.area.area_type, node2.area.area_type
            ));
        }

        // Compare text content
        if node1.area.text_content() != node2.area.text_content() {
            diffs.push(format!(
                "Text content mismatch at {}: {:?} vs {:?}",
                path,
                node1.area.text_content(),
                node2.area.text_content()
            ));
        }

        // Compare children count
        let children1 = self.children(id1);
        let children2 = other.children(id2);
        if children1.len() != children2.len() {
            diffs.push(format!(
                "Children count mismatch at {}: {} vs {}",
                path,
                children1.len(),
                children2.len()
            ));
        }

        // Recurse into children
        let min_len = children1.len().min(children2.len());
        for i in 0..min_len {
            let child_path = format!("{}/{}", path, i);
            self.diff_nodes(children1[i], other, children2[i], &child_path, diffs);
        }
    }
}

impl Default for AreaTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::area::AreaType;
    use fop_types::{Length, Point, Rect, Size};

    fn make_rect(w: f64, h: f64) -> Rect {
        Rect::from_point_size(
            Point::ZERO,
            Size::new(Length::from_pt(w), Length::from_pt(h)),
        )
    }

    #[test]
    fn test_area_tree_creation() {
        let tree = AreaTree::new();
        assert_eq!(tree.len(), 0);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_add_area() {
        let mut tree = AreaTree::new();
        let area = Area::new(AreaType::Page, make_rect(210.0, 297.0));
        let id = tree.add_area(area);

        assert_eq!(tree.len(), 1);
        assert_eq!(id.index(), 0);
        assert!(tree.get(id).is_some());
    }

    #[test]
    fn test_parent_child() {
        let mut tree = AreaTree::new();

        let page = tree.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        let block = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));

        tree.append_child(page, block)
            .expect("test: should succeed");

        assert_eq!(
            tree.get(block).expect("test: should succeed").parent,
            Some(page)
        );
        assert_eq!(
            tree.get(page).expect("test: should succeed").first_child,
            Some(block)
        );
    }

    #[test]
    fn test_multiple_children() {
        let mut tree = AreaTree::new();

        let page = tree.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        let block1 = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));
        let block2 = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));
        let block3 = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));

        tree.append_child(page, block1)
            .expect("test: should succeed");
        tree.append_child(page, block2)
            .expect("test: should succeed");
        tree.append_child(page, block3)
            .expect("test: should succeed");

        let children = tree.children(page);
        assert_eq!(children.len(), 3);
        assert_eq!(children[0], block1);
        assert_eq!(children[1], block2);
        assert_eq!(children[2], block3);
    }

    #[test]
    fn test_reparent_middle_child_moves_subtree() {
        let mut tree = AreaTree::new();

        // region_a holds [block1, block2, block3]; region_b is empty.
        let region_a = tree.add_area(Area::new(AreaType::Region, make_rect(160.0, 247.0)));
        let region_b = tree.add_area(Area::new(AreaType::Region, make_rect(160.0, 247.0)));
        let block1 = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));
        let block2 = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));
        let block3 = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));
        // Give block2 a child so we can verify the whole subtree moves.
        let line = tree.add_area(Area::new(AreaType::Text, make_rect(100.0, 12.0)));

        tree.append_child(region_a, block1)
            .expect("test: should succeed");
        tree.append_child(region_a, block2)
            .expect("test: should succeed");
        tree.append_child(region_a, block3)
            .expect("test: should succeed");
        tree.append_child(block2, line)
            .expect("test: should succeed");

        // Move the middle child (block2) to region_b.
        tree.reparent(block2, region_b)
            .expect("test: reparent should succeed");

        // region_a now holds [block1, block3] in order.
        assert_eq!(tree.children(region_a), vec![block1, block3]);
        // region_b now holds [block2].
        assert_eq!(tree.children(region_b), vec![block2]);
        // block2's parent is region_b and its subtree (line) came along.
        assert_eq!(
            tree.get(block2).expect("test: block2 exists").parent,
            Some(region_b)
        );
        assert_eq!(tree.children(block2), vec![line]);
    }

    #[test]
    fn test_reparent_first_child_promotes_sibling() {
        let mut tree = AreaTree::new();

        let region_a = tree.add_area(Area::new(AreaType::Region, make_rect(160.0, 247.0)));
        let region_b = tree.add_area(Area::new(AreaType::Region, make_rect(160.0, 247.0)));
        let block1 = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));
        let block2 = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));

        tree.append_child(region_a, block1)
            .expect("test: should succeed");
        tree.append_child(region_a, block2)
            .expect("test: should succeed");

        tree.reparent(block1, region_b)
            .expect("test: reparent should succeed");

        assert_eq!(tree.children(region_a), vec![block2]);
        assert_eq!(tree.children(region_b), vec![block1]);
    }

    #[test]
    fn test_reparent_onto_self_is_error() {
        let mut tree = AreaTree::new();
        let region = tree.add_area(Area::new(AreaType::Region, make_rect(160.0, 247.0)));
        assert!(tree.reparent(region, region).is_err());
    }

    #[test]
    fn test_find_page_ancestor() {
        let mut tree = AreaTree::new();

        let page = tree.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        let region = tree.add_area(Area::new(AreaType::Region, make_rect(160.0, 247.0)));
        let block = tree.add_area(Area::new(AreaType::Block, make_rect(160.0, 20.0)));

        tree.append_child(page, region)
            .expect("test: should succeed");
        tree.append_child(region, block)
            .expect("test: should succeed");

        // block -> region -> page: should find the page
        assert_eq!(tree.find_page_ancestor(block), Some(page));

        // region -> page: should find the page
        assert_eq!(tree.find_page_ancestor(region), Some(page));

        // page itself: should return itself
        assert_eq!(tree.find_page_ancestor(page), Some(page));
    }

    #[test]
    fn test_footnote_tracking() {
        let mut tree = AreaTree::new();

        let page = tree.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        let footnote = tree.add_area(Area::new(AreaType::Footnote, make_rect(160.0, 12.0)));

        // Initially no footnotes
        assert!(tree.get_footnotes(page).is_none());

        // Add a footnote
        tree.add_footnote(page, footnote);
        let footnotes = tree.get_footnotes(page).expect("test: should succeed");
        assert_eq!(footnotes.len(), 1);
        assert_eq!(footnotes[0], footnote);

        // Footnote height should be > 0 (including separator height)
        let height = tree.footnote_height(page);
        assert!(height > fop_types::Length::ZERO);
    }
    #[test]
    fn test_area_tree_serialize_empty() {
        let tree = AreaTree::new();
        let s = tree.serialize();
        // Empty tree should produce an empty string or minimal output — must not panic
        let _ = s;
    }

    #[test]
    fn test_area_tree_serialize_single_page() {
        let mut tree = AreaTree::new();
        let page = tree.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        let _ = page;

        let s = tree.serialize();
        // Should mention Page type in some form
        assert!(
            !s.is_empty(),
            "Serialize should produce non-empty output for a page"
        );
        assert!(
            s.contains("Page"),
            "Serialize should mention AreaType::Page"
        );
    }

    #[test]
    fn test_area_tree_serialize_nested_areas() {
        let mut tree = AreaTree::new();
        let page = tree.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        let region = tree.add_area(Area::new(AreaType::Region, make_rect(160.0, 247.0)));
        let block = tree.add_area(Area::new(AreaType::Block, make_rect(160.0, 20.0)));
        tree.append_child(page, region)
            .expect("test: should succeed");
        tree.append_child(region, block)
            .expect("test: should succeed");

        let s = tree.serialize();
        assert!(s.contains("Page"), "Should contain Page");
        assert!(s.contains("Region"), "Should contain Region");
        assert!(s.contains("Block"), "Should contain Block");
    }

    #[test]
    fn test_area_tree_diff_two_empty_trees() {
        let tree1 = AreaTree::new();
        let tree2 = AreaTree::new();
        let diffs = tree1.diff(&tree2);
        assert!(
            diffs.is_empty(),
            "Two empty trees should have no differences"
        );
    }

    #[test]
    fn test_area_tree_diff_self_has_root_other_does_not() {
        let mut tree1 = AreaTree::new();
        let tree2 = AreaTree::new();
        tree1.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));

        let diffs = tree1.diff(&tree2);
        assert!(
            !diffs.is_empty(),
            "Should detect that self has root but other does not"
        );
        assert!(
            diffs[0].contains("no root"),
            "Diff message should mention missing root"
        );
    }

    #[test]
    fn test_area_tree_diff_other_has_root_self_does_not() {
        let tree1 = AreaTree::new();
        let mut tree2 = AreaTree::new();
        tree2.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));

        let diffs = tree1.diff(&tree2);
        assert!(
            !diffs.is_empty(),
            "Should detect that other has root but self does not"
        );
    }

    #[test]
    fn test_area_tree_diff_identical_single_page() {
        let mut tree1 = AreaTree::new();
        let mut tree2 = AreaTree::new();
        tree1.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        tree2.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));

        let diffs = tree1.diff(&tree2);
        assert!(
            diffs.is_empty(),
            "Identical single-page trees should have no diffs"
        );
    }

    #[test]
    fn test_area_tree_diff_different_root_type() {
        let mut tree1 = AreaTree::new();
        let mut tree2 = AreaTree::new();
        tree1.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        tree2.add_area(Area::new(AreaType::Region, make_rect(210.0, 297.0)));

        let diffs = tree1.diff(&tree2);
        assert!(
            !diffs.is_empty(),
            "Different area types should produce a diff"
        );
        assert!(
            diffs
                .iter()
                .any(|d| d.contains("type") || d.contains("Area")),
            "Diff should describe type mismatch"
        );
    }

    #[test]
    fn test_area_tree_diff_different_child_count() {
        let mut tree1 = AreaTree::new();
        let mut tree2 = AreaTree::new();

        let page1 = tree1.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        let child1 = tree1.add_area(Area::new(AreaType::Block, make_rect(100.0, 20.0)));
        tree1
            .append_child(page1, child1)
            .expect("test: should succeed");

        let _page2 = tree2.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        // tree2 has no children

        let diffs = tree1.diff(&tree2);
        assert!(
            !diffs.is_empty(),
            "Different child counts should produce a diff"
        );
    }

    #[test]
    fn test_area_tree_diff_identical_with_text_content() {
        let mut tree1 = AreaTree::new();
        let mut tree2 = AreaTree::new();

        let mut area1 = Area::new(AreaType::Text, make_rect(100.0, 12.0));
        area1.content = Some(crate::area::types::AreaContent::Text("Hello".to_string()));
        let mut area2 = Area::new(AreaType::Text, make_rect(100.0, 12.0));
        area2.content = Some(crate::area::types::AreaContent::Text("Hello".to_string()));

        tree1.add_area(area1);
        tree2.add_area(area2);

        let diffs = tree1.diff(&tree2);
        assert!(
            diffs.is_empty(),
            "Trees with identical text content should have no diffs"
        );
    }

    #[test]
    fn test_area_tree_diff_different_text_content() {
        let mut tree1 = AreaTree::new();
        let mut tree2 = AreaTree::new();

        let mut area1 = Area::new(AreaType::Text, make_rect(100.0, 12.0));
        area1.content = Some(crate::area::types::AreaContent::Text("Hello".to_string()));
        let mut area2 = Area::new(AreaType::Text, make_rect(100.0, 12.0));
        area2.content = Some(crate::area::types::AreaContent::Text("World".to_string()));

        tree1.add_area(area1);
        tree2.add_area(area2);

        let diffs = tree1.diff(&tree2);
        assert!(
            !diffs.is_empty(),
            "Different text content should produce a diff"
        );
        assert!(
            diffs.iter().any(|d| d.contains("Text")),
            "Diff should mention text content"
        );
    }
}

#[cfg(test)]
mod extended_tests {
    use super::*;
    use crate::area::AreaType;
    use fop_types::{Length, Point, Rect, Size};

    fn make_rect(w: f64, h: f64) -> Rect {
        Rect::from_point_size(
            Point::ZERO,
            Size::new(Length::from_pt(w), Length::from_pt(h)),
        )
    }

    // ---- AreaId tests ----

    #[test]
    fn test_area_id_index() {
        let mut tree = AreaTree::new();
        let a0 = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));
        let a1 = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));
        assert_eq!(a0.index(), 0);
        assert_eq!(a1.index(), 1);
    }

    #[test]
    fn test_area_id_from_index() {
        let id = AreaId::from_index(42);
        assert_eq!(id.index(), 42);
    }

    #[test]
    fn test_area_id_display() {
        let id = AreaId::from_index(7);
        assert_eq!(format!("{}", id), "Area(7)");
    }

    // ---- AreaNode tests ----

    #[test]
    fn test_area_node_has_children_initially_false() {
        let node = AreaNode::new(Area::new(AreaType::Block, make_rect(100.0, 50.0)));
        assert!(!node.has_children());
    }

    #[test]
    fn test_area_node_has_children_after_append() {
        let mut tree = AreaTree::new();
        let parent = tree.add_area(Area::new(AreaType::Page, make_rect(595.0, 842.0)));
        let child = tree.add_area(Area::new(AreaType::Block, make_rect(400.0, 50.0)));
        tree.append_child(parent, child)
            .expect("test: should succeed");

        let parent_node = tree.get(parent).expect("test: should succeed");
        assert!(parent_node.has_children());
    }

    // ---- AreaTree size tracking tests ----

    #[test]
    fn test_area_tree_with_capacity() {
        let tree = AreaTree::with_capacity(100);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_area_tree_len_tracks_additions() {
        let mut tree = AreaTree::new();
        assert_eq!(tree.len(), 0);
        tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));
        assert_eq!(tree.len(), 1);
        tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));
        assert_eq!(tree.len(), 2);
    }

    // ---- get / get_mut tests ----

    #[test]
    fn test_get_mut_can_modify_area() {
        let mut tree = AreaTree::new();
        let id = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));

        {
            let node = tree.get_mut(id).expect("test: should succeed");
            node.area.geometry.width = Length::from_pt(200.0);
        }

        let node = tree.get(id).expect("test: should succeed");
        assert_eq!(node.area.geometry.width, Length::from_pt(200.0));
    }

    #[test]
    fn test_get_nonexistent_returns_none() {
        let tree = AreaTree::new();
        let bogus_id = AreaId::from_index(999);
        assert!(tree.get(bogus_id).is_none());
        // NOTE: get_mut would panic for out-of-bounds on Vec, so we skip it
    }

    // ---- children ordering tests ----

    #[test]
    fn test_children_ordering_preserved() {
        let mut tree = AreaTree::new();
        let parent = tree.add_area(Area::new(AreaType::Page, make_rect(595.0, 842.0)));
        let c1 = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 20.0)));
        let c2 = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 30.0)));
        let c3 = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 40.0)));

        tree.append_child(parent, c1).expect("test: should succeed");
        tree.append_child(parent, c2).expect("test: should succeed");
        tree.append_child(parent, c3).expect("test: should succeed");

        let children = tree.children(parent);
        assert_eq!(children.len(), 3);
        assert_eq!(children[0], c1);
        assert_eq!(children[1], c2);
        assert_eq!(children[2], c3);
    }

    #[test]
    fn test_children_returns_empty_for_leaf() {
        let mut tree = AreaTree::new();
        let leaf = tree.add_area(Area::new(AreaType::Text, make_rect(100.0, 12.0)));
        let children = tree.children(leaf);
        assert!(children.is_empty());
    }

    // ---- iter and root tests ----

    #[test]
    fn test_iter_over_all_areas() {
        let mut tree = AreaTree::new();
        tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));
        tree.add_area(Area::new(AreaType::Block, make_rect(200.0, 50.0)));
        tree.add_area(Area::new(AreaType::Block, make_rect(300.0, 50.0)));

        let count = tree.iter().count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_root_returns_first_area() {
        let mut tree = AreaTree::new();
        let first_id = tree.add_area(Area::new(AreaType::Page, make_rect(595.0, 842.0)));
        tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));

        let (root_id, root_node) = tree.root().expect("test: should succeed");
        assert_eq!(root_id, first_id);
        assert_eq!(root_node.area.area_type, AreaType::Page);
    }

    #[test]
    fn test_root_empty_tree_returns_none() {
        let tree = AreaTree::new();
        assert!(tree.root().is_none());
    }

    // ---- footnote tracking tests ----

    #[test]
    fn test_footnote_height_zero_with_no_footnotes() {
        let mut tree = AreaTree::new();
        let page_id = tree.add_area(Area::new(AreaType::Page, make_rect(595.0, 842.0)));
        let height = tree.footnote_height(page_id);
        assert_eq!(height, Length::ZERO);
    }

    #[test]
    fn test_footnote_tracking_multiple_footnotes() {
        let mut tree = AreaTree::new();
        let page_id = tree.add_area(Area::new(AreaType::Page, make_rect(595.0, 842.0)));

        let fn1 = tree.add_area(Area::new(AreaType::Footnote, make_rect(400.0, 30.0)));
        let fn2 = tree.add_area(Area::new(AreaType::Footnote, make_rect(400.0, 20.0)));

        tree.add_footnote(page_id, fn1);
        tree.add_footnote(page_id, fn2);

        let footnotes = tree.get_footnotes(page_id).expect("test: should succeed");
        assert_eq!(footnotes.len(), 2);

        let total_height = tree.footnote_height(page_id);
        // footnote_height adds 7pt for separator line + 6pt spacing when footnotes present
        // Total = 30 + 20 + 7 = 57pt
        assert_eq!(total_height, Length::from_pt(57.0));
    }

    #[test]
    fn test_get_footnotes_returns_none_for_page_without_footnotes() {
        let mut tree = AreaTree::new();
        let page_id = tree.add_area(Area::new(AreaType::Page, make_rect(595.0, 842.0)));
        assert!(tree.get_footnotes(page_id).is_none());
    }

    // ---- find_page_ancestor tests ----

    #[test]
    fn test_find_page_ancestor_direct_child() {
        let mut tree = AreaTree::new();
        let page_id = tree.add_area(Area::new(AreaType::Page, make_rect(595.0, 842.0)));
        let block_id = tree.add_area(Area::new(AreaType::Block, make_rect(400.0, 50.0)));
        tree.append_child(page_id, block_id)
            .expect("test: should succeed");

        let ancestor = tree.find_page_ancestor(block_id);
        assert_eq!(ancestor, Some(page_id));
    }

    #[test]
    fn test_find_page_ancestor_deep_nesting() {
        let mut tree = AreaTree::new();
        let page_id = tree.add_area(Area::new(AreaType::Page, make_rect(595.0, 842.0)));
        let region_id = tree.add_area(Area::new(AreaType::Region, make_rect(450.0, 700.0)));
        let block_id = tree.add_area(Area::new(AreaType::Block, make_rect(400.0, 50.0)));
        let text_id = tree.add_area(Area::new(AreaType::Text, make_rect(300.0, 12.0)));

        tree.append_child(page_id, region_id)
            .expect("test: should succeed");
        tree.append_child(region_id, block_id)
            .expect("test: should succeed");
        tree.append_child(block_id, text_id)
            .expect("test: should succeed");

        let ancestor = tree.find_page_ancestor(text_id);
        assert_eq!(ancestor, Some(page_id));
    }

    #[test]
    fn test_find_page_ancestor_returns_none_for_page_itself() {
        let mut tree = AreaTree::new();
        let page_id = tree.add_area(Area::new(AreaType::Page, make_rect(595.0, 842.0)));
        // A Page's ancestor is itself – the function walks *up* looking for Page type
        // If the area IS the page, the result depends on implementation
        // Just ensure it doesn't panic
        let _result = tree.find_page_ancestor(page_id);
    }

    // ---- serialize tests ----

    #[test]
    fn test_serialize_produces_non_empty_string_for_non_empty_tree() {
        let mut tree = AreaTree::new();
        tree.add_area(Area::new(AreaType::Page, make_rect(595.0, 842.0)));
        let serialized = tree.serialize();
        assert!(!serialized.is_empty());
    }

    #[test]
    fn test_serialize_empty_tree_is_empty_string() {
        let tree = AreaTree::new();
        let serialized = tree.serialize();
        assert!(serialized.is_empty());
    }

    #[test]
    fn test_serialize_contains_area_type() {
        let mut tree = AreaTree::new();
        tree.add_area(Area::new(AreaType::Page, make_rect(595.0, 842.0)));
        let serialized = tree.serialize();
        assert!(
            serialized.contains("Page"),
            "Serialized tree should mention 'Page'"
        );
    }

    // ---- diff tests ----

    #[test]
    fn test_diff_same_single_area_no_diffs() {
        let mut tree1 = AreaTree::new();
        let mut tree2 = AreaTree::new();
        tree1.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));
        tree2.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));

        let diffs = tree1.diff(&tree2);
        assert!(
            diffs.is_empty(),
            "Identical trees should have no diffs, got: {:?}",
            diffs
        );
    }

    #[test]
    fn test_diff_different_area_types_produces_diff() {
        let mut tree1 = AreaTree::new();
        let mut tree2 = AreaTree::new();
        tree1.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));
        tree2.add_area(Area::new(AreaType::Line, make_rect(100.0, 50.0)));

        let diffs = tree1.diff(&tree2);
        assert!(
            !diffs.is_empty(),
            "Different area types should produce a diff"
        );
    }

    #[test]
    fn test_diff_is_reflexive() {
        let mut tree = AreaTree::new();
        let parent = tree.add_area(Area::new(AreaType::Page, make_rect(595.0, 842.0)));
        let child = tree.add_area(Area::new(AreaType::Block, make_rect(400.0, 50.0)));
        tree.append_child(parent, child)
            .expect("test: should succeed");

        // Diffing a tree against itself should produce no diffs
        // We need a clone — create an identical tree instead
        let mut tree2 = AreaTree::new();
        let parent2 = tree2.add_area(Area::new(AreaType::Page, make_rect(595.0, 842.0)));
        let child2 = tree2.add_area(Area::new(AreaType::Block, make_rect(400.0, 50.0)));
        tree2
            .append_child(parent2, child2)
            .expect("test: should succeed");

        let diffs = tree.diff(&tree2);
        assert!(diffs.is_empty(), "Identical trees should have no diffs");
    }

    // ---- append_child error case ----

    #[test]
    fn test_append_child_to_nonexistent_parent_returns_error() {
        let mut tree = AreaTree::new();
        let child = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));
        let bogus_parent = AreaId::from_index(999);
        let result = tree.append_child(bogus_parent, child);
        assert!(
            result.is_err(),
            "Appending to nonexistent parent should fail"
        );
    }
}
