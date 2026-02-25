//! PDF outline (bookmarks) extraction from FO tree

use super::document::{PdfOutline, PdfOutlineItem};
use fop_core::{FoArena, FoNodeData, NodeId};
use fop_types::Result;

/// Extract PDF outline from FO tree
pub fn extract_outline_from_fo_tree(fo_tree: &FoArena) -> Result<Option<PdfOutline>> {
    // Find the bookmark-tree element
    if let Some((root_id, _)) = fo_tree.root() {
        if let Some(bookmark_tree_id) = find_bookmark_tree(fo_tree, root_id) {
            let items = extract_outline_items(fo_tree, bookmark_tree_id)?;
            if !items.is_empty() {
                return Ok(Some(PdfOutline { items }));
            }
        }
    }

    Ok(None)
}

/// Find the bookmark-tree node in the FO tree
fn find_bookmark_tree(fo_tree: &FoArena, node_id: NodeId) -> Option<NodeId> {
    let node = fo_tree.get(node_id)?;

    // Check if this is a bookmark-tree
    if matches!(node.data, FoNodeData::BookmarkTree { .. }) {
        return Some(node_id);
    }

    // Search children
    for child_id in fo_tree.children(node_id) {
        if let Some(result) = find_bookmark_tree(fo_tree, child_id) {
            return Some(result);
        }
    }

    None
}

/// Extract outline items from bookmark-tree children
fn extract_outline_items(fo_tree: &FoArena, parent_id: NodeId) -> Result<Vec<PdfOutlineItem>> {
    let mut items = Vec::new();

    for child_id in fo_tree.children(parent_id) {
        if let Some(node) = fo_tree.get(child_id) {
            if let FoNodeData::Bookmark {
                internal_destination,
                external_destination,
                ..
            } = &node.data
            {
                // Extract bookmark title
                let title = extract_bookmark_title(fo_tree, child_id)?;

                // For now, map internal destinations to page index 0
                // In a real implementation, we'd resolve the destination to an actual page
                let page_index = if internal_destination.is_some() {
                    Some(0)
                } else {
                    None
                };

                // Extract nested bookmarks
                let children = extract_nested_bookmarks(fo_tree, child_id)?;

                items.push(PdfOutlineItem {
                    title,
                    page_index,
                    external_destination: external_destination.clone(),
                    children,
                });
            }
        }
    }

    Ok(items)
}

/// Extract nested bookmarks (child fo:bookmark elements)
fn extract_nested_bookmarks(fo_tree: &FoArena, bookmark_id: NodeId) -> Result<Vec<PdfOutlineItem>> {
    let mut items = Vec::new();

    for child_id in fo_tree.children(bookmark_id) {
        if let Some(node) = fo_tree.get(child_id) {
            // Only process child bookmark elements, not bookmark-title
            if let FoNodeData::Bookmark {
                internal_destination,
                external_destination,
                ..
            } = &node.data
            {
                let title = extract_bookmark_title(fo_tree, child_id)?;
                let page_index = if internal_destination.is_some() {
                    Some(0)
                } else {
                    None
                };

                // Recursively extract nested bookmarks
                let children = extract_nested_bookmarks(fo_tree, child_id)?;

                items.push(PdfOutlineItem {
                    title,
                    page_index,
                    external_destination: external_destination.clone(),
                    children,
                });
            }
        }
    }

    Ok(items)
}

/// Extract the title text from a bookmark's bookmark-title child
fn extract_bookmark_title(fo_tree: &FoArena, bookmark_id: NodeId) -> Result<String> {
    let mut title = String::new();

    for child_id in fo_tree.children(bookmark_id) {
        if let Some(node) = fo_tree.get(child_id) {
            if matches!(node.data, FoNodeData::BookmarkTitle { .. }) {
                // Extract text from bookmark-title children
                title = extract_text_content(fo_tree, child_id);
                break;
            }
        }
    }

    if title.is_empty() {
        title = "Untitled".to_string();
    }

    Ok(title)
}

/// Extract text content from a node and its children
fn extract_text_content(fo_tree: &FoArena, node_id: NodeId) -> String {
    let mut text = String::new();

    for child_id in fo_tree.children(node_id) {
        if let Some(node) = fo_tree.get(child_id) {
            if let FoNodeData::Text(content) = &node.data {
                text.push_str(content);
            }
        }
    }

    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use fop_core::FoTreeBuilder;
    use std::io::Cursor;

    #[test]
    fn test_extract_simple_outline() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:bookmark-tree>
        <fo:bookmark internal-destination="ch1">
            <fo:bookmark-title>Chapter 1</fo:bookmark-title>
        </fo:bookmark>
        <fo:bookmark internal-destination="ch2">
            <fo:bookmark-title>Chapter 2</fo:bookmark-title>
        </fo:bookmark>
    </fo:bookmark-tree>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let fo_tree = builder.parse(cursor).expect("test: should succeed");

        let outline = extract_outline_from_fo_tree(&fo_tree).expect("test: should succeed");
        assert!(outline.is_some());

        let outline = outline.expect("test: should succeed");
        assert_eq!(outline.items.len(), 2);
        assert_eq!(outline.items[0].title, "Chapter 1");
        assert_eq!(outline.items[1].title, "Chapter 2");
    }

    #[test]
    fn test_extract_nested_outline() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:bookmark-tree>
        <fo:bookmark internal-destination="ch1">
            <fo:bookmark-title>Chapter 1</fo:bookmark-title>
            <fo:bookmark internal-destination="s1.1">
                <fo:bookmark-title>Section 1.1</fo:bookmark-title>
            </fo:bookmark>
            <fo:bookmark internal-destination="s1.2">
                <fo:bookmark-title>Section 1.2</fo:bookmark-title>
            </fo:bookmark>
        </fo:bookmark>
    </fo:bookmark-tree>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let fo_tree = builder.parse(cursor).expect("test: should succeed");

        let outline = extract_outline_from_fo_tree(&fo_tree).expect("test: should succeed");
        assert!(outline.is_some());

        let outline = outline.expect("test: should succeed");
        assert_eq!(outline.items.len(), 1);
        assert_eq!(outline.items[0].title, "Chapter 1");
        assert_eq!(outline.items[0].children.len(), 2);
        assert_eq!(outline.items[0].children[0].title, "Section 1.1");
        assert_eq!(outline.items[0].children[1].title, "Section 1.2");
    }

    #[test]
    fn test_no_outline() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let fo_tree = builder.parse(cursor).expect("test: should succeed");

        let outline = extract_outline_from_fo_tree(&fo_tree).expect("test: should succeed");
        assert!(outline.is_none());
    }
}

#[cfg(test)]
mod tests_outline_comprehensive {
    use super::super::document::{PdfDocument, PdfOutline, PdfOutlineItem, PdfPage};
    use super::*;
    use fop_core::FoTreeBuilder;
    use fop_types::Length;
    use std::io::Cursor;

    fn parse_fo(xml: &'static str) -> fop_core::FoArena<'static> {
        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        builder.parse(cursor).expect("test: should succeed")
    }

    // ── PdfOutlineItem construction ───────────────────────────────────────────

    #[test]
    fn test_outline_item_title_and_page_ref() {
        let item = PdfOutlineItem {
            title: "Introduction".to_string(),
            page_index: Some(0),
            external_destination: None,
            children: vec![],
        };
        assert_eq!(item.title, "Introduction");
        assert_eq!(item.page_index, Some(0));
        assert!(item.external_destination.is_none());
        assert!(item.children.is_empty());
    }

    #[test]
    fn test_outline_item_external_destination() {
        let item = PdfOutlineItem {
            title: "External Link".to_string(),
            page_index: None,
            external_destination: Some("https://example.com".to_string()),
            children: vec![],
        };
        assert!(item.page_index.is_none());
        assert_eq!(
            item.external_destination.as_deref(),
            Some("https://example.com")
        );
    }

    #[test]
    fn test_outline_item_clone() {
        let item = PdfOutlineItem {
            title: "Chapter".to_string(),
            page_index: Some(2),
            external_destination: None,
            children: vec![],
        };
        let cloned = item.clone();
        assert_eq!(cloned.title, "Chapter");
        assert_eq!(cloned.page_index, Some(2));
    }

    // ── Sibling outline items ─────────────────────────────────────────────────

    #[test]
    fn test_sibling_outline_items_three_siblings() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:bookmark-tree>
        <fo:bookmark internal-destination="p1">
            <fo:bookmark-title>Part 1</fo:bookmark-title>
        </fo:bookmark>
        <fo:bookmark internal-destination="p2">
            <fo:bookmark-title>Part 2</fo:bookmark-title>
        </fo:bookmark>
        <fo:bookmark internal-destination="p3">
            <fo:bookmark-title>Part 3</fo:bookmark-title>
        </fo:bookmark>
    </fo:bookmark-tree>
</fo:root>"#;
        let fo = parse_fo(xml);
        let outline = extract_outline_from_fo_tree(&fo)
            .expect("test: should succeed")
            .expect("test: should succeed");
        assert_eq!(outline.items.len(), 3);
        assert_eq!(outline.items[0].title, "Part 1");
        assert_eq!(outline.items[1].title, "Part 2");
        assert_eq!(outline.items[2].title, "Part 3");
    }

    // ── Nested children (3 levels deep) ──────────────────────────────────────

    #[test]
    fn test_three_level_nested_outline() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:bookmark-tree>
        <fo:bookmark internal-destination="ch1">
            <fo:bookmark-title>Chapter 1</fo:bookmark-title>
            <fo:bookmark internal-destination="s1">
                <fo:bookmark-title>Section 1</fo:bookmark-title>
                <fo:bookmark internal-destination="ss1">
                    <fo:bookmark-title>Subsection 1</fo:bookmark-title>
                </fo:bookmark>
            </fo:bookmark>
        </fo:bookmark>
    </fo:bookmark-tree>
</fo:root>"#;
        let fo = parse_fo(xml);
        let outline = extract_outline_from_fo_tree(&fo)
            .expect("test: should succeed")
            .expect("test: should succeed");
        assert_eq!(outline.items.len(), 1);
        let ch1 = &outline.items[0];
        assert_eq!(ch1.title, "Chapter 1");
        assert_eq!(ch1.children.len(), 1);
        let s1 = &ch1.children[0];
        assert_eq!(s1.title, "Section 1");
        assert_eq!(s1.children.len(), 1);
        assert_eq!(s1.children[0].title, "Subsection 1");
    }

    // ── /Outlines dict structure in generated PDF ─────────────────────────────

    #[test]
    fn test_outlines_dict_present_in_pdf_bytes() {
        let mut doc = PdfDocument::new();
        let outline = PdfOutline {
            items: vec![PdfOutlineItem {
                title: "Only Chapter".to_string(),
                page_index: Some(0),
                external_destination: None,
                children: vec![],
            }],
        };
        doc.set_outline(outline);
        doc.add_page(PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0)));
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/Outlines"));
    }

    #[test]
    fn test_catalog_references_outlines() {
        let mut doc = PdfDocument::new();
        let outline = PdfOutline {
            items: vec![PdfOutlineItem {
                title: "Intro".to_string(),
                page_index: Some(0),
                external_destination: None,
                children: vec![],
            }],
        };
        doc.set_outline(outline);
        doc.add_page(PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0)));
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        // Catalog must reference /Outlines N 0 R
        assert!(s.contains("/Outlines 4 0 R"));
    }

    #[test]
    fn test_outline_title_appears_in_pdf_bytes() {
        let mut doc = PdfDocument::new();
        let outline = PdfOutline {
            items: vec![
                PdfOutlineItem {
                    title: "Alpha Chapter".to_string(),
                    page_index: Some(0),
                    external_destination: None,
                    children: vec![],
                },
                PdfOutlineItem {
                    title: "Beta Chapter".to_string(),
                    page_index: Some(0),
                    external_destination: None,
                    children: vec![],
                },
            ],
        };
        doc.set_outline(outline);
        doc.add_page(PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0)));
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("Alpha Chapter"));
        assert!(s.contains("Beta Chapter"));
    }

    // ── Count field: total descendants ───────────────────────────────────────

    #[test]
    fn test_outline_count_reflected_in_pdf() {
        // Two top-level items → /Count 2 in /Outlines root dict
        let mut doc = PdfDocument::new();
        let outline = PdfOutline {
            items: vec![
                PdfOutlineItem {
                    title: "Item A".to_string(),
                    page_index: Some(0),
                    external_destination: None,
                    children: vec![],
                },
                PdfOutlineItem {
                    title: "Item B".to_string(),
                    page_index: Some(0),
                    external_destination: None,
                    children: vec![],
                },
            ],
        };
        doc.set_outline(outline);
        doc.add_page(PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0)));
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        // Outlines root has /Count 2
        assert!(s.contains("/Count 2"));
    }

    // ── No outline → no /Outlines in catalog ─────────────────────────────────

    #[test]
    fn test_no_outline_no_outlines_in_catalog() {
        let doc = PdfDocument::new();
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(!s.contains("/Outlines 4 0 R"));
    }

    // ── PdfOutline struct ─────────────────────────────────────────────────────

    #[test]
    fn test_pdf_outline_items_count() {
        let outline = PdfOutline {
            items: vec![
                PdfOutlineItem {
                    title: "A".to_string(),
                    page_index: Some(0),
                    external_destination: None,
                    children: vec![],
                },
                PdfOutlineItem {
                    title: "B".to_string(),
                    page_index: Some(1),
                    external_destination: None,
                    children: vec![],
                },
                PdfOutlineItem {
                    title: "C".to_string(),
                    page_index: Some(2),
                    external_destination: None,
                    children: vec![],
                },
            ],
        };
        assert_eq!(outline.items.len(), 3);
    }

    #[test]
    fn test_bookmark_untitled_defaults_to_untitled() {
        // A bookmark without a bookmark-title child should get "Untitled"
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:bookmark-tree>
        <fo:bookmark internal-destination="x">
        </fo:bookmark>
    </fo:bookmark-tree>
</fo:root>"#;
        let fo = parse_fo(xml);
        let outline = extract_outline_from_fo_tree(&fo)
            .expect("test: should succeed")
            .expect("test: should succeed");
        assert_eq!(outline.items[0].title, "Untitled");
    }
}
