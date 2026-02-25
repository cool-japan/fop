//! Arena allocator for FO tree nodes
//!
//! Uses index-based handles instead of Rc<RefCell<>> for better performance
//! and memory efficiency.

use super::id_registry::IdRegistry;
use super::node::FoNode;
use std::fmt;

/// Index-based handle to a node in the arena
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

impl NodeId {
    /// Get the raw index value
    #[inline]
    pub fn index(self) -> usize {
        self.0
    }

    /// Create a NodeId from a raw index (unsafe - use with caution)
    #[inline]
    pub const fn from_index(index: usize) -> Self {
        Self(index)
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node({})", self.0)
    }
}

/// Arena allocator for FO tree nodes
///
/// Stores all nodes in a contiguous vector for cache efficiency.
/// Nodes are accessed by index-based handles (NodeId).
pub struct FoArena<'a> {
    nodes: Vec<FoNode<'a>>,
    id_registry: IdRegistry,
    /// Document language from xml:lang on fo:root (e.g. "en", "ja")
    pub document_lang: Option<String>,
}

impl<'a> FoArena<'a> {
    /// Create a new empty arena
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            id_registry: IdRegistry::new(),
            document_lang: None,
        }
    }

    /// Create a new arena with preallocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            id_registry: IdRegistry::new(),
            document_lang: None,
        }
    }

    /// Get a reference to the ID registry
    pub fn id_registry(&self) -> &IdRegistry {
        &self.id_registry
    }

    /// Get a mutable reference to the ID registry
    pub fn id_registry_mut(&mut self) -> &mut IdRegistry {
        &mut self.id_registry
    }

    /// Add a node to the arena and return its ID
    pub fn add_node(&mut self, node: FoNode<'a>) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(node);
        id
    }

    /// Get a reference to a node by ID
    pub fn get(&self, id: NodeId) -> Option<&FoNode<'a>> {
        self.nodes.get(id.0)
    }

    /// Get a mutable reference to a node by ID
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut FoNode<'a>> {
        self.nodes.get_mut(id.0)
    }

    /// Get the number of nodes in the arena
    #[inline]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the arena is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Iterate over all nodes
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &FoNode<'a>)> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (NodeId(i), node))
    }

    /// Get the root node (first node in arena)
    pub fn root(&self) -> Option<(NodeId, &FoNode<'a>)> {
        if self.nodes.is_empty() {
            None
        } else {
            Some((NodeId(0), &self.nodes[0]))
        }
    }

    /// Set parent-child relationship
    pub fn set_parent(&mut self, child: NodeId, parent: NodeId) -> Result<(), String> {
        if child.0 >= self.nodes.len() {
            return Err(format!("Child node {} does not exist", child.0));
        }
        if parent.0 >= self.nodes.len() {
            return Err(format!("Parent node {} does not exist", parent.0));
        }

        // Set parent of child
        self.nodes[child.0].parent = Some(parent);

        Ok(())
    }

    /// Append a child to a parent node
    pub fn append_child(&mut self, parent: NodeId, child: NodeId) -> Result<(), String> {
        if child.0 >= self.nodes.len() {
            return Err(format!("Child node {} does not exist", child.0));
        }
        if parent.0 >= self.nodes.len() {
            return Err(format!("Parent node {} does not exist", parent.0));
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

    /// Get all children of a node
    pub fn children(&self, parent: NodeId) -> Vec<NodeId> {
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

    /// Get the depth of a node in the tree
    pub fn depth(&self, mut node: NodeId) -> usize {
        let mut depth = 0;
        while let Some(parent) = self.get(node).and_then(|n| n.parent) {
            depth += 1;
            node = parent;
        }
        depth
    }
}

impl<'a> Default for FoArena<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::node::FoNodeData;

    #[test]
    fn test_arena_creation() {
        let arena: FoArena = FoArena::new();
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
    }

    #[test]
    fn test_add_node() {
        let mut arena = FoArena::new();
        let node = FoNode::new(FoNodeData::Root);
        let id = arena.add_node(node);

        assert_eq!(arena.len(), 1);
        assert_eq!(id.index(), 0);
        assert!(arena.get(id).is_some());
    }

    #[test]
    fn test_parent_child() {
        let mut arena = FoArena::new();

        // Create root and child nodes
        let root = arena.add_node(FoNode::new(FoNodeData::Root));
        let child = arena.add_node(FoNode::new(FoNodeData::Block {
            properties: crate::properties::PropertyList::new(),
        }));

        // Set up relationship
        arena
            .append_child(root, child)
            .expect("test: should succeed");

        // Verify relationship
        assert_eq!(
            arena.get(child).expect("test: should succeed").parent,
            Some(root)
        );
        assert_eq!(
            arena.get(root).expect("test: should succeed").first_child,
            Some(child)
        );
    }

    #[test]
    fn test_multiple_children() {
        let mut arena = FoArena::new();

        let root = arena.add_node(FoNode::new(FoNodeData::Root));
        let child1 = arena.add_node(FoNode::new(FoNodeData::Block {
            properties: crate::properties::PropertyList::new(),
        }));
        let child2 = arena.add_node(FoNode::new(FoNodeData::Block {
            properties: crate::properties::PropertyList::new(),
        }));
        let child3 = arena.add_node(FoNode::new(FoNodeData::Block {
            properties: crate::properties::PropertyList::new(),
        }));

        arena
            .append_child(root, child1)
            .expect("test: should succeed");
        arena
            .append_child(root, child2)
            .expect("test: should succeed");
        arena
            .append_child(root, child3)
            .expect("test: should succeed");

        let children = arena.children(root);
        assert_eq!(children.len(), 3);
        assert_eq!(children[0], child1);
        assert_eq!(children[1], child2);
        assert_eq!(children[2], child3);
    }

    #[test]
    fn test_depth() {
        let mut arena = FoArena::new();

        let root = arena.add_node(FoNode::new(FoNodeData::Root));
        let child = arena.add_node(FoNode::new(FoNodeData::Block {
            properties: crate::properties::PropertyList::new(),
        }));
        let grandchild = arena.add_node(FoNode::new(FoNodeData::Inline {
            properties: crate::properties::PropertyList::new(),
        }));

        arena
            .append_child(root, child)
            .expect("test: should succeed");
        arena
            .append_child(child, grandchild)
            .expect("test: should succeed");

        assert_eq!(arena.depth(root), 0);
        assert_eq!(arena.depth(child), 1);
        assert_eq!(arena.depth(grandchild), 2);
    }

    #[test]
    fn test_iteration() {
        let mut arena = FoArena::new();

        arena.add_node(FoNode::new(FoNodeData::Root));
        arena.add_node(FoNode::new(FoNodeData::Block {
            properties: crate::properties::PropertyList::new(),
        }));
        arena.add_node(FoNode::new(FoNodeData::Inline {
            properties: crate::properties::PropertyList::new(),
        }));

        let count = arena.iter().count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_node_id_display() {
        let id = NodeId::from_index(42);
        assert_eq!(format!("{}", id), "Node(42)");
    }

    #[test]
    fn test_node_id_equality() {
        let id1 = NodeId::from_index(5);
        let id2 = NodeId::from_index(5);
        let id3 = NodeId::from_index(6);
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_node_id_copy() {
        let id1 = NodeId::from_index(3);
        let id2 = id1;
        assert_eq!(id1.index(), id2.index());
    }

    #[test]
    fn test_with_capacity() {
        let arena: FoArena = FoArena::with_capacity(64);
        assert!(arena.is_empty());
        assert_eq!(arena.len(), 0);
    }

    #[test]
    fn test_default() {
        let arena: FoArena = FoArena::default();
        assert!(arena.is_empty());
    }

    #[test]
    fn test_root_on_empty_arena() {
        let arena: FoArena = FoArena::new();
        assert!(arena.root().is_none());
    }

    #[test]
    fn test_root_returns_first_node() {
        let mut arena = FoArena::new();
        let root_id = arena.add_node(FoNode::new(FoNodeData::Root));
        arena.add_node(FoNode::new(FoNodeData::LayoutMasterSet));

        let (id, node) = arena.root().expect("test: should succeed");
        assert_eq!(id, root_id);
        assert!(matches!(node.data, FoNodeData::Root));
    }

    #[test]
    fn test_get_invalid_id() {
        let arena: FoArena = FoArena::new();
        let invalid = NodeId::from_index(999);
        assert!(arena.get(invalid).is_none());
    }

    #[test]
    fn test_get_mut_invalid_id() {
        let mut arena: FoArena = FoArena::new();
        let invalid = NodeId::from_index(999);
        assert!(arena.get_mut(invalid).is_none());
    }

    #[test]
    fn test_set_parent_invalid_child() {
        let mut arena = FoArena::new();
        let root = arena.add_node(FoNode::new(FoNodeData::Root));
        let invalid = NodeId::from_index(999);
        assert!(arena.set_parent(invalid, root).is_err());
    }

    #[test]
    fn test_set_parent_invalid_parent() {
        let mut arena = FoArena::new();
        let child = arena.add_node(FoNode::new(FoNodeData::Root));
        let invalid = NodeId::from_index(999);
        assert!(arena.set_parent(child, invalid).is_err());
    }

    #[test]
    fn test_append_child_invalid_child() {
        let mut arena = FoArena::new();
        let root = arena.add_node(FoNode::new(FoNodeData::Root));
        let invalid = NodeId::from_index(999);
        assert!(arena.append_child(root, invalid).is_err());
    }

    #[test]
    fn test_append_child_invalid_parent() {
        let mut arena = FoArena::new();
        let child = arena.add_node(FoNode::new(FoNodeData::Root));
        let invalid = NodeId::from_index(999);
        assert!(arena.append_child(invalid, child).is_err());
    }

    #[test]
    fn test_children_of_leaf() {
        let mut arena = FoArena::new();
        let root = arena.add_node(FoNode::new(FoNodeData::Root));
        let children = arena.children(root);
        assert!(children.is_empty());
    }

    #[test]
    fn test_document_lang() {
        let mut arena: FoArena = FoArena::new();
        assert!(arena.document_lang.is_none());
        arena.document_lang = Some("en".to_string());
        assert_eq!(arena.document_lang.as_deref(), Some("en"));
    }

    #[test]
    fn test_id_registry() {
        let arena: FoArena = FoArena::new();
        // id_registry() is accessible and returns a valid reference
        let _reg = arena.id_registry();
    }

    #[test]
    fn test_id_registry_mut() {
        let mut arena: FoArena = FoArena::new();
        let _reg = arena.id_registry_mut();
    }

    #[test]
    fn test_iter_yields_correct_ids() {
        let mut arena = FoArena::new();
        let id0 = arena.add_node(FoNode::new(FoNodeData::Root));
        let id1 = arena.add_node(FoNode::new(FoNodeData::LayoutMasterSet));

        let items: Vec<_> = arena.iter().collect();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].0, id0);
        assert_eq!(items[1].0, id1);
    }

    #[test]
    fn test_depth_three_levels() {
        let mut arena = FoArena::new();
        let root = arena.add_node(FoNode::new(FoNodeData::Root));
        let child = arena.add_node(FoNode::new(FoNodeData::Block {
            properties: crate::properties::PropertyList::new(),
        }));
        let grandchild = arena.add_node(FoNode::new(FoNodeData::Inline {
            properties: crate::properties::PropertyList::new(),
        }));
        let great_grandchild = arena.add_node(FoNode::new(FoNodeData::Text("hi".to_string())));

        arena
            .append_child(root, child)
            .expect("test: should succeed");
        arena
            .append_child(child, grandchild)
            .expect("test: should succeed");
        arena
            .append_child(grandchild, great_grandchild)
            .expect("test: should succeed");

        assert_eq!(arena.depth(root), 0);
        assert_eq!(arena.depth(child), 1);
        assert_eq!(arena.depth(grandchild), 2);
        assert_eq!(arena.depth(great_grandchild), 3);
    }

    #[test]
    fn test_sibling_ordering() {
        let mut arena = FoArena::new();
        let parent = arena.add_node(FoNode::new(FoNodeData::Root));

        let mut ids = vec![];
        for _ in 0..5 {
            let child = arena.add_node(FoNode::new(FoNodeData::Block {
                properties: crate::properties::PropertyList::new(),
            }));
            arena
                .append_child(parent, child)
                .expect("test: should succeed");
            ids.push(child);
        }

        let children = arena.children(parent);
        assert_eq!(children, ids);
    }
}
