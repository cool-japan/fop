//! Plain text rendering backend
//!
//! Generates plain text output from area trees for accessibility and text extraction.

use fop_layout::area::{AreaNode, AreaTree, AreaType};
use fop_layout::AreaId;
use fop_types::Result;

/// Text renderer - converts area trees to plain text
pub struct TextRenderer {
    /// Use form feed for page separators (default: true)
    use_form_feed: bool,
}

impl TextRenderer {
    /// Create a new text renderer
    pub fn new() -> Self {
        Self {
            use_form_feed: true,
        }
    }

    /// Create a text renderer with custom page separator
    pub fn with_page_separator(use_form_feed: bool) -> Self {
        Self { use_form_feed }
    }

    /// Render an area tree to plain text
    pub fn render_to_text(&self, area_tree: &AreaTree) -> Result<String> {
        let mut output = String::new();

        // Collect all page IDs in tree iteration order
        let page_ids: Vec<AreaId> = area_tree
            .iter()
            .filter_map(|(id, node)| {
                if matches!(node.area.area_type, AreaType::Page) {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();

        for (page_num, id) in page_ids.into_iter().enumerate() {
            if page_num > 0 {
                // Add page separator between pages
                if self.use_form_feed {
                    output.push('\x0C'); // Form feed character
                } else {
                    output.push_str("\n\n");
                }
                output.push_str(&format!("--- Page {} ---\n\n", page_num + 1));
            }

            // Render the page content
            self.render_area(area_tree, id, &mut output);
        }

        // Trim trailing whitespace/newlines then add a final newline
        let trimmed = output.trim_end().to_string();
        if trimmed.is_empty() {
            Ok(String::new())
        } else {
            Ok(format!("{}\n", trimmed))
        }
    }

    /// Render a single area and its children recursively
    fn render_area(&self, area_tree: &AreaTree, area_id: AreaId, output: &mut String) {
        let node = match area_tree.get(area_id) {
            Some(n) => n,
            None => return,
        };

        match node.area.area_type {
            AreaType::Page | AreaType::Region | AreaType::Column => {
                // Container areas - render children
                self.render_children(area_tree, area_id, output);
            }
            AreaType::Block => {
                // Block area - render children then add newline
                self.render_children(area_tree, area_id, output);
                output.push('\n');
            }
            AreaType::Line => {
                // Line area - render children and add newline
                self.render_children(area_tree, area_id, output);
                output.push('\n');
            }
            AreaType::Text => {
                // Text area - extract text content
                if let Some(text) = node.area.text_content() {
                    output.push_str(text);
                }
            }
            AreaType::Space => {
                // Space area - add a space
                output.push(' ');
            }
            AreaType::Inline => {
                // Inline area - render children
                self.render_children(area_tree, area_id, output);
            }
            AreaType::Viewport => {
                // Image placeholder
                if node.area.has_image_data() {
                    output.push_str("[IMAGE]");
                } else {
                    self.render_children(area_tree, area_id, output);
                }
            }
            AreaType::Header => {
                // Header - render content then add a divider line
                let start = output.len();
                self.render_children(area_tree, area_id, output);
                let header_text = output[start..].trim().to_string();
                output.truncate(start);
                if !header_text.is_empty() {
                    output.push_str(&header_text);
                    output.push('\n');
                    output.push_str(&"-".repeat(40));
                    output.push('\n');
                }
            }
            AreaType::Footer => {
                // Footer - add a divider line then render content
                let start = output.len();
                self.render_children(area_tree, area_id, output);
                let footer_text = output[start..].trim().to_string();
                output.truncate(start);
                if !footer_text.is_empty() {
                    output.push_str(&"-".repeat(40));
                    output.push('\n');
                    output.push_str(&footer_text);
                    output.push('\n');
                }
            }
            AreaType::Footnote => {
                // Footnote - render with marker
                output.push_str("\n[Footnote] ");
                self.render_children(area_tree, area_id, output);
                output.push('\n');
            }
            AreaType::FootnoteSeparator => {
                // Footnote separator - use a line
                output.push_str("\n---\n");
            }
            AreaType::FloatArea => {
                // Float area - render children inline
                self.render_children(area_tree, area_id, output);
            }
            AreaType::SidebarStart | AreaType::SidebarEnd => {
                // Sidebar markers - render children
                self.render_children(area_tree, area_id, output);
            }
        }
    }

    /// Render all children of an area
    fn render_children(&self, area_tree: &AreaTree, parent_id: AreaId, output: &mut String) {
        let children = area_tree.children(parent_id);
        for child_id in children {
            self.render_area(area_tree, child_id, output);
        }
    }

    /// Extract text from an area tree without formatting
    pub fn extract_text(&self, area_tree: &AreaTree) -> Result<String> {
        let mut output = String::new();

        for (id, _node) in area_tree.iter() {
            self.extract_text_from_area(area_tree, id, &mut output);
        }

        Ok(output)
    }

    /// Extract raw text from a single area (no formatting)
    fn extract_text_from_area(&self, area_tree: &AreaTree, area_id: AreaId, output: &mut String) {
        if let Some(node) = area_tree.get(area_id) {
            if let Some(text) = node.area.text_content() {
                output.push_str(text);
                output.push(' ');
            }
        }
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to check if an area should add a line break after
#[allow(dead_code)]
fn should_add_line_break(node: &AreaNode) -> bool {
    matches!(
        node.area.area_type,
        AreaType::Block | AreaType::Line | AreaType::Header | AreaType::Footer
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use fop_layout::area::{Area, AreaTree, AreaType};
    use fop_types::{Length, Point, Rect, Size};

    fn make_rect(w: f64, h: f64) -> Rect {
        Rect::from_point_size(
            Point::ZERO,
            Size::new(Length::from_pt(w), Length::from_pt(h)),
        )
    }

    // -----------------------------------------------------------------------
    // Unit tests for TextRenderer internals (direct area tree construction)
    // -----------------------------------------------------------------------

    #[test]
    fn test_text_renderer_creation() {
        let renderer = TextRenderer::new();
        assert!(renderer.use_form_feed);
    }

    #[test]
    fn test_text_renderer_no_form_feed() {
        let renderer = TextRenderer::with_page_separator(false);
        assert!(!renderer.use_form_feed);
    }

    #[test]
    fn test_text_renderer_default() {
        let renderer = TextRenderer::default();
        assert!(renderer.use_form_feed);
    }

    #[test]
    fn test_simple_text_extraction() {
        let mut tree = AreaTree::new();
        let page = tree.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        let block = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 20.0)));
        let text = tree.add_area(Area::text(make_rect(50.0, 12.0), "Hello World".to_string()));

        tree.append_child(page, block)
            .expect("test: should succeed");
        tree.append_child(block, text)
            .expect("test: should succeed");

        let renderer = TextRenderer::new();
        let result = renderer
            .render_to_text(&tree)
            .expect("test: should succeed");

        assert!(result.contains("Hello World"), "got: {:?}", result);
    }

    #[test]
    fn test_multiple_lines() {
        let mut tree = AreaTree::new();
        let page = tree.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        let block = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 40.0)));
        let line1 = tree.add_area(Area::new(AreaType::Line, make_rect(100.0, 12.0)));
        let text1 = tree.add_area(Area::text(make_rect(50.0, 12.0), "First line".to_string()));
        let line2 = tree.add_area(Area::new(AreaType::Line, make_rect(100.0, 12.0)));
        let text2 = tree.add_area(Area::text(make_rect(50.0, 12.0), "Second line".to_string()));

        tree.append_child(page, block)
            .expect("test: should succeed");
        tree.append_child(block, line1)
            .expect("test: should succeed");
        tree.append_child(line1, text1)
            .expect("test: should succeed");
        tree.append_child(block, line2)
            .expect("test: should succeed");
        tree.append_child(line2, text2)
            .expect("test: should succeed");

        let renderer = TextRenderer::new();
        let result = renderer
            .render_to_text(&tree)
            .expect("test: should succeed");

        assert!(result.contains("First line"), "got: {:?}", result);
        assert!(result.contains("Second line"), "got: {:?}", result);
    }

    #[test]
    fn test_multipage_form_feed() {
        let mut tree = AreaTree::new();
        let page1 = tree.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        let block1 = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 20.0)));
        let text1 = tree.add_area(Area::text(make_rect(50.0, 12.0), "Page one".to_string()));
        let page2 = tree.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        let block2 = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 20.0)));
        let text2 = tree.add_area(Area::text(make_rect(50.0, 12.0), "Page two".to_string()));

        tree.append_child(page1, block1)
            .expect("test: should succeed");
        tree.append_child(block1, text1)
            .expect("test: should succeed");
        tree.append_child(page2, block2)
            .expect("test: should succeed");
        tree.append_child(block2, text2)
            .expect("test: should succeed");

        let renderer = TextRenderer::new();
        let result = renderer
            .render_to_text(&tree)
            .expect("test: should succeed");

        assert!(result.contains("Page one"), "got: {:?}", result);
        assert!(result.contains("Page two"), "got: {:?}", result);
        // Form feed character between pages
        assert!(
            result.contains('\x0C'),
            "expected form feed, got: {:?}",
            result
        );
    }

    #[test]
    fn test_multipage_no_form_feed_uses_separator() {
        let mut tree = AreaTree::new();
        let page1 = tree.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        let block1 = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 20.0)));
        let text1 = tree.add_area(Area::text(make_rect(50.0, 12.0), "Alpha".to_string()));
        let page2 = tree.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        let block2 = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 20.0)));
        let text2 = tree.add_area(Area::text(make_rect(50.0, 12.0), "Beta".to_string()));

        tree.append_child(page1, block1)
            .expect("test: should succeed");
        tree.append_child(block1, text1)
            .expect("test: should succeed");
        tree.append_child(page2, block2)
            .expect("test: should succeed");
        tree.append_child(block2, text2)
            .expect("test: should succeed");

        let renderer = TextRenderer::with_page_separator(false);
        let result = renderer
            .render_to_text(&tree)
            .expect("test: should succeed");

        assert!(result.contains("Alpha"), "got: {:?}", result);
        assert!(result.contains("Beta"), "got: {:?}", result);
        assert!(
            result.contains("--- Page 2 ---"),
            "expected page separator, got: {:?}",
            result
        );
    }

    #[test]
    fn test_empty_tree_produces_empty_output() {
        let tree = AreaTree::new();
        let renderer = TextRenderer::new();
        let result = renderer
            .render_to_text(&tree)
            .expect("test: should succeed");
        assert!(
            result.is_empty(),
            "empty tree should produce empty output, got: {:?}",
            result
        );
    }

    #[test]
    fn test_space_area() {
        let mut tree = AreaTree::new();
        let page = tree.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        let block = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 20.0)));
        let line = tree.add_area(Area::new(AreaType::Line, make_rect(100.0, 12.0)));
        let text1 = tree.add_area(Area::text(make_rect(30.0, 12.0), "foo".to_string()));
        let space = tree.add_area(Area::new(AreaType::Space, make_rect(5.0, 12.0)));
        let text2 = tree.add_area(Area::text(make_rect(30.0, 12.0), "bar".to_string()));

        tree.append_child(page, block)
            .expect("test: should succeed");
        tree.append_child(block, line)
            .expect("test: should succeed");
        tree.append_child(line, text1)
            .expect("test: should succeed");
        tree.append_child(line, space)
            .expect("test: should succeed");
        tree.append_child(line, text2)
            .expect("test: should succeed");

        let renderer = TextRenderer::new();
        let result = renderer
            .render_to_text(&tree)
            .expect("test: should succeed");

        assert!(
            result.contains("foo bar"),
            "expected 'foo bar', got: {:?}",
            result
        );
    }

    #[test]
    fn test_footnote_area() {
        let mut tree = AreaTree::new();
        let page = tree.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        let block = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 20.0)));
        let body_text = tree.add_area(Area::text(make_rect(50.0, 12.0), "Body text".to_string()));
        let sep = tree.add_area(Area::new(
            AreaType::FootnoteSeparator,
            make_rect(100.0, 2.0),
        ));
        let footnote = tree.add_area(Area::new(AreaType::Footnote, make_rect(100.0, 20.0)));
        let fn_text = tree.add_area(Area::text(
            make_rect(50.0, 12.0),
            "Footnote content".to_string(),
        ));

        tree.append_child(page, block)
            .expect("test: should succeed");
        tree.append_child(block, body_text)
            .expect("test: should succeed");
        tree.append_child(page, sep).expect("test: should succeed");
        tree.append_child(page, footnote)
            .expect("test: should succeed");
        tree.append_child(footnote, fn_text)
            .expect("test: should succeed");

        let renderer = TextRenderer::new();
        let result = renderer
            .render_to_text(&tree)
            .expect("test: should succeed");

        assert!(result.contains("Body text"), "got: {:?}", result);
        assert!(result.contains("[Footnote]"), "got: {:?}", result);
        assert!(result.contains("Footnote content"), "got: {:?}", result);
        assert!(
            result.contains("---"),
            "expected separator, got: {:?}",
            result
        );
    }

    #[test]
    fn test_viewport_image_placeholder() {
        let mut tree = AreaTree::new();
        let page = tree.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        let viewport = tree.add_area(Area::viewport_with_image(
            make_rect(50.0, 50.0),
            vec![0u8; 10],
        ));

        tree.append_child(page, viewport)
            .expect("test: should succeed");

        let renderer = TextRenderer::new();
        let result = renderer
            .render_to_text(&tree)
            .expect("test: should succeed");

        assert!(
            result.contains("[IMAGE]"),
            "expected [IMAGE] placeholder, got: {:?}",
            result
        );
    }

    #[test]
    fn test_extract_text() {
        let mut tree = AreaTree::new();
        let page = tree.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        let block = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 20.0)));
        let text = tree.add_area(Area::text(make_rect(50.0, 12.0), "ExtractMe".to_string()));

        tree.append_child(page, block)
            .expect("test: should succeed");
        tree.append_child(block, text)
            .expect("test: should succeed");

        let renderer = TextRenderer::new();
        let result = renderer.extract_text(&tree).expect("test: should succeed");

        assert!(result.contains("ExtractMe"), "got: {:?}", result);
    }

    #[test]
    fn test_output_ends_with_newline() {
        let mut tree = AreaTree::new();
        let page = tree.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        let block = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 20.0)));
        let text = tree.add_area(Area::text(make_rect(50.0, 12.0), "Content".to_string()));

        tree.append_child(page, block)
            .expect("test: should succeed");
        tree.append_child(block, text)
            .expect("test: should succeed");

        let renderer = TextRenderer::new();
        let result = renderer
            .render_to_text(&tree)
            .expect("test: should succeed");

        assert!(
            result.ends_with('\n'),
            "output should end with newline, got: {:?}",
            result
        );
    }

    #[test]
    fn test_header_with_divider() {
        let mut tree = AreaTree::new();
        let page = tree.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        let header = tree.add_area(Area::new(AreaType::Header, make_rect(210.0, 15.0)));
        let htext = tree.add_area(Area::text(make_rect(100.0, 12.0), "My Header".to_string()));

        tree.append_child(page, header)
            .expect("test: should succeed");
        tree.append_child(header, htext)
            .expect("test: should succeed");

        let renderer = TextRenderer::new();
        let result = renderer
            .render_to_text(&tree)
            .expect("test: should succeed");

        assert!(result.contains("My Header"), "got: {:?}", result);
        // Should have a divider after the header
        assert!(
            result.contains("---"),
            "expected divider after header, got: {:?}",
            result
        );
    }

    #[test]
    fn test_footer_with_divider() {
        let mut tree = AreaTree::new();
        let page = tree.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
        let footer = tree.add_area(Area::new(AreaType::Footer, make_rect(210.0, 15.0)));
        let ftext = tree.add_area(Area::text(make_rect(100.0, 12.0), "My Footer".to_string()));

        tree.append_child(page, footer)
            .expect("test: should succeed");
        tree.append_child(footer, ftext)
            .expect("test: should succeed");

        let renderer = TextRenderer::new();
        let result = renderer
            .render_to_text(&tree)
            .expect("test: should succeed");

        assert!(result.contains("My Footer"), "got: {:?}", result);
        assert!(
            result.contains("---"),
            "expected divider before footer, got: {:?}",
            result
        );
    }

    #[test]
    fn test_three_pages_separators() {
        let mut tree = AreaTree::new();
        for i in 1..=3 {
            let page = tree.add_area(Area::new(AreaType::Page, make_rect(210.0, 297.0)));
            let block = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 20.0)));
            let text = tree.add_area(Area::text(make_rect(50.0, 12.0), format!("Content {}", i)));
            tree.append_child(page, block)
                .expect("test: should succeed");
            tree.append_child(block, text)
                .expect("test: should succeed");
        }

        let renderer = TextRenderer::with_page_separator(false);
        let result = renderer
            .render_to_text(&tree)
            .expect("test: should succeed");

        assert!(result.contains("Content 1"), "got: {:?}", result);
        assert!(result.contains("Content 2"), "got: {:?}", result);
        assert!(result.contains("Content 3"), "got: {:?}", result);
        assert!(
            result.contains("--- Page 2 ---"),
            "expected page 2 separator, got: {:?}",
            result
        );
        assert!(
            result.contains("--- Page 3 ---"),
            "expected page 3 separator, got: {:?}",
            result
        );
    }
}
