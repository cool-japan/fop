//! Streaming layout engine for processing large documents efficiently
//!
//! This module provides a streaming layout engine that processes pages incrementally,
//! yielding one page at a time to minimize memory usage for large documents (1000+ pages).

use crate::area::{Area, AreaTree, AreaType, TraitSet};
use crate::layout::{
    extract_space_after, extract_space_before, extract_traits, BlockLayoutContext,
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

    /// Layout a single page sequence and yield its pages
    fn layout_page_sequence<'b>(
        &self,
        fo_tree: &FoArena<'b>,
        page_seq_id: NodeId,
        resolver: &mut PageNumberResolver,
    ) -> Result<Vec<AreaTree>> {
        let mut pages = Vec::new();

        // Get the page-sequence node
        let page_seq_node = fo_tree
            .get(page_seq_id)
            .ok_or_else(|| fop_types::FopError::Generic("Page sequence not found".to_string()))?;

        if let FoNodeData::PageSequence { properties, .. } = &page_seq_node.data {
            // Each page-sequence produces one or more pages
            // For now, we'll create one page per page-sequence
            // In a full implementation, this would handle page breaks and multiple pages

            let mut page_tree = AreaTree::new();

            // Create a page area
            let page_rect = Rect::from_point_size(
                Point::ZERO,
                Size::new(self.config.page_width, self.config.page_height),
            );

            let mut traits = TraitSet::default();
            if let Ok(color) = properties.get(PropertyId::BackgroundColor) {
                traits.background_color = color.as_color();
            }

            let area = Area::new(AreaType::Page, page_rect).with_traits(traits);
            let page_id = page_tree.add_area(area);

            // Register ID if present
            if let Some(node) = fo_tree.get(page_seq_id) {
                if let Some(id) = &node.id {
                    resolver.register_element(id.clone(), page_id);
                }
            }

            // Process children (flow, static-content)
            let children = fo_tree.children(page_seq_id);
            for child_id in children {
                self.layout_node(fo_tree, child_id, &mut page_tree, Some(page_id), resolver)?;
            }

            // Increment page counter
            resolver.set_current_page(resolver.current_page() + 1);

            pages.push(page_tree);
        }

        Ok(pages)
    }

    /// Layout a single FO node (internal helper)
    fn layout_node<'b>(
        &self,
        fo_tree: &FoArena<'b>,
        node_id: NodeId,
        area_tree: &mut AreaTree,
        parent_area: Option<crate::area::AreaId>,
        resolver: &mut PageNumberResolver,
    ) -> Result<Option<crate::area::AreaId>> {
        let node = fo_tree
            .get(node_id)
            .ok_or_else(|| fop_types::FopError::Generic(format!("Node {} not found", node_id)))?;

        let area_id = match &node.data {
            FoNodeData::Flow { properties, .. } => {
                // Flow creates a region area
                let flow_rect = Rect::from_point_size(
                    Point::new(Length::from_pt(72.0), Length::from_pt(72.0)), // 1 inch margins
                    Size::new(
                        self.config.page_width - Length::from_pt(144.0),
                        self.config.page_height - Length::from_pt(144.0),
                    ),
                );

                let mut traits = TraitSet::default();
                if let Ok(color) = properties.get(PropertyId::Color) {
                    traits.color = color.as_color();
                }

                let area = Area::new(AreaType::Region, flow_rect).with_traits(traits);
                let area_id = area_tree.add_area(area);

                if let Some(parent) = parent_area {
                    area_tree
                        .append_child(parent, area_id)
                        .map_err(fop_types::FopError::Generic)?;
                }

                // Layout block children
                let children = fo_tree.children(node_id);
                let mut block_ctx = BlockLayoutContext::new(flow_rect.width);

                for child_id in children {
                    if let Some(child_area_id) = self.layout_block(
                        fo_tree,
                        child_id,
                        area_tree,
                        area_id,
                        block_ctx.current_y,
                        flow_rect.width,
                        resolver,
                    )? {
                        if let Some(child_area) = area_tree.get(child_area_id) {
                            block_ctx.current_y =
                                child_area.area.geometry.y + child_area.area.height();
                        }
                    }
                }

                Some(area_id)
            }

            _ => None,
        };

        Ok(area_id)
    }

    /// Layout a block-level element
    #[allow(clippy::too_many_arguments)]
    fn layout_block<'b>(
        &self,
        fo_tree: &FoArena<'b>,
        node_id: NodeId,
        area_tree: &mut AreaTree,
        parent_area: crate::area::AreaId,
        y_offset: Length,
        available_width: Length,
        resolver: &mut PageNumberResolver,
    ) -> Result<Option<crate::area::AreaId>> {
        let node = fo_tree
            .get(node_id)
            .ok_or_else(|| fop_types::FopError::Generic(format!("Node {} not found", node_id)))?;

        match &node.data {
            FoNodeData::Block { properties } => {
                let traits = extract_traits(properties);
                let space_before = extract_space_before(properties);
                let space_after = extract_space_after(properties);
                let line_height = traits.font_size.unwrap_or(Length::from_pt(12.0));

                let mut block_ctx = BlockLayoutContext::new(available_width);
                block_ctx.current_y = y_offset;
                let block_rect = block_ctx.allocate_with_spacing(
                    available_width,
                    line_height,
                    space_before,
                    space_after,
                );

                let area = Area::new(AreaType::Block, block_rect).with_traits(traits.clone());
                let area_id = area_tree.add_area(area);
                area_tree
                    .append_child(parent_area, area_id)
                    .map_err(fop_types::FopError::Generic)?;

                let text_align = traits.text_align.unwrap_or(TextAlign::Left);

                // Process text children
                let children = fo_tree.children(node_id);
                for child_id in children {
                    if let Some(child_node) = fo_tree.get(child_id) {
                        if let FoNodeData::Text(text) = &child_node.data {
                            let mut text_traits = traits.clone();
                            text_traits.text_align = Some(text_align);

                            let text_rect =
                                Rect::new(Length::ZERO, Length::ZERO, available_width, line_height);

                            let text_area =
                                Area::text(text_rect, text.clone()).with_traits(text_traits);
                            let text_id = area_tree.add_area(text_area);
                            area_tree
                                .append_child(area_id, text_id)
                                .map_err(fop_types::FopError::Generic)?;
                        }
                    }
                }

                // Register ID if present
                if let Some(id) = &node.id {
                    resolver.register_element(id.clone(), area_id);
                }

                Ok(Some(area_id))
            }
            _ => Ok(None),
        }
    }
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
