//! ID registry for tracking element IDs and cross-references
//!
//! Per XSL-FO spec section 5.2.2, the "id" property uniquely identifies
//! an element within the document for cross-references and linking.

use super::NodeId;
use crate::{FopError, Result};
use std::collections::HashMap;

/// Registry for tracking element IDs and their node locations
///
/// This registry:
/// - Maps ID strings to their NodeId locations in the tree
/// - Validates ID uniqueness (duplicate IDs are errors)
/// - Enables cross-reference resolution for fo:page-number-citation
/// - Enables internal link resolution for fo:basic-link
#[derive(Debug, Default)]
pub struct IdRegistry {
    /// Maps element ID to its node location in the tree
    id_to_node: HashMap<String, NodeId>,
}

impl IdRegistry {
    /// Create a new empty ID registry
    pub fn new() -> Self {
        Self {
            id_to_node: HashMap::new(),
        }
    }

    /// Register an element ID with its node location
    ///
    /// # Arguments
    /// * `id` - The element ID (must be unique within the document)
    /// * `node_id` - The node location in the tree
    ///
    /// # Returns
    /// * `Ok(())` if the ID was registered successfully
    /// * `Err(FopError::DuplicateId)` if the ID is already registered
    ///
    /// # Example
    /// ```ignore
    /// let mut registry = IdRegistry::new();
    /// let node_id = NodeId::from_index(5);
    /// registry.register_id("chapter1".to_string(), node_id)?;
    /// ```
    pub fn register_id(&mut self, id: String, node_id: NodeId) -> Result<()> {
        if let Some(existing_node) = self.id_to_node.get(&id) {
            return Err(FopError::Generic(format!(
                "Duplicate ID '{}': already used by node {} (attempted reuse at node {})",
                id,
                existing_node.index(),
                node_id.index()
            )));
        }

        self.id_to_node.insert(id, node_id);
        Ok(())
    }

    /// Look up a node by its element ID
    ///
    /// # Arguments
    /// * `id` - The element ID to look up
    ///
    /// # Returns
    /// * `Some(NodeId)` if the ID is registered
    /// * `None` if the ID is not found
    ///
    /// # Example
    /// ```ignore
    /// if let Some(node_id) = registry.lookup_id("chapter1") {
    ///     println!("Found node: {}", node_id);
    /// }
    /// ```
    pub fn lookup_id(&self, id: &str) -> Option<NodeId> {
        self.id_to_node.get(id).copied()
    }

    /// Validate that all IDs are unique
    ///
    /// This method always returns `Ok(())` because uniqueness
    /// is enforced during registration. It exists for API completeness
    /// and future validation needs.
    pub fn validate_unique_ids(&self) -> Result<()> {
        // Uniqueness is enforced during register_id()
        // This method exists for API completeness
        Ok(())
    }

    /// Check if an ID is registered
    ///
    /// # Arguments
    /// * `id` - The element ID to check
    ///
    /// # Returns
    /// * `true` if the ID is registered
    /// * `false` if the ID is not found
    pub fn has_id(&self, id: &str) -> bool {
        self.id_to_node.contains_key(id)
    }

    /// Get the total number of registered IDs
    pub fn len(&self) -> usize {
        self.id_to_node.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.id_to_node.is_empty()
    }

    /// Clear all registered IDs
    pub fn clear(&mut self) {
        self.id_to_node.clear();
    }

    /// Get an iterator over all registered IDs and their node locations
    pub fn iter(&self) -> impl Iterator<Item = (&String, &NodeId)> {
        self.id_to_node.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = IdRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_id() {
        let mut registry = IdRegistry::new();
        let node1 = NodeId::from_index(10);

        registry
            .register_id("chapter1".to_string(), node1)
            .expect("test: should succeed");

        assert_eq!(registry.len(), 1);
        assert_eq!(registry.lookup_id("chapter1"), Some(node1));
    }

    #[test]
    fn test_duplicate_id_error() {
        let mut registry = IdRegistry::new();
        let node1 = NodeId::from_index(10);
        let node2 = NodeId::from_index(20);

        registry
            .register_id("chapter1".to_string(), node1)
            .expect("test: should succeed");

        // Attempting to register the same ID again should fail
        let result = registry.register_id("chapter1".to_string(), node2);
        assert!(result.is_err());

        // First registration should still be valid
        assert_eq!(registry.lookup_id("chapter1"), Some(node1));
    }

    #[test]
    fn test_lookup_nonexistent_id() {
        let registry = IdRegistry::new();
        assert_eq!(registry.lookup_id("nonexistent"), None);
    }

    #[test]
    fn test_has_id() {
        let mut registry = IdRegistry::new();
        let node1 = NodeId::from_index(10);

        assert!(!registry.has_id("chapter1"));

        registry
            .register_id("chapter1".to_string(), node1)
            .expect("test: should succeed");

        assert!(registry.has_id("chapter1"));
        assert!(!registry.has_id("chapter2"));
    }

    #[test]
    fn test_multiple_ids() {
        let mut registry = IdRegistry::new();

        let node1 = NodeId::from_index(10);
        let node2 = NodeId::from_index(20);
        let node3 = NodeId::from_index(30);

        registry
            .register_id("chapter1".to_string(), node1)
            .expect("test: should succeed");
        registry
            .register_id("chapter2".to_string(), node2)
            .expect("test: should succeed");
        registry
            .register_id("section-a".to_string(), node3)
            .expect("test: should succeed");

        assert_eq!(registry.len(), 3);
        assert_eq!(registry.lookup_id("chapter1"), Some(node1));
        assert_eq!(registry.lookup_id("chapter2"), Some(node2));
        assert_eq!(registry.lookup_id("section-a"), Some(node3));
    }

    #[test]
    fn test_clear() {
        let mut registry = IdRegistry::new();
        let node1 = NodeId::from_index(10);

        registry
            .register_id("chapter1".to_string(), node1)
            .expect("test: should succeed");
        assert_eq!(registry.len(), 1);

        registry.clear();
        assert!(registry.is_empty());
        assert_eq!(registry.lookup_id("chapter1"), None);
    }

    #[test]
    fn test_validate_unique_ids() {
        let mut registry = IdRegistry::new();
        let node1 = NodeId::from_index(10);

        registry
            .register_id("chapter1".to_string(), node1)
            .expect("test: should succeed");

        // Validation should always succeed since uniqueness is enforced
        assert!(registry.validate_unique_ids().is_ok());
    }

    #[test]
    fn test_iteration() {
        let mut registry = IdRegistry::new();

        let node1 = NodeId::from_index(10);
        let node2 = NodeId::from_index(20);

        registry
            .register_id("chapter1".to_string(), node1)
            .expect("test: should succeed");
        registry
            .register_id("chapter2".to_string(), node2)
            .expect("test: should succeed");

        let ids: Vec<_> = registry.iter().map(|(id, _)| id.clone()).collect();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"chapter1".to_string()));
        assert!(ids.contains(&"chapter2".to_string()));
    }
}
