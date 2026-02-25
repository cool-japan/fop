//! Area tree data structures
//!
//! The area tree represents the geometric layout of content on pages.
//! Each area has a position, dimensions, and may contain child areas.

pub mod area_tree;
pub mod types;

pub use area_tree::{AreaId, AreaNode, AreaTree};
pub use types::{
    Area, AreaContent, AreaType, BorderStyle, Direction, DisplayAlign, FontStretch, FontStyle,
    FontVariant, Span, TextDecoration, TextTransform, TraitSet, WritingMode,
};

#[cfg(test)]
mod tests {
    use super::*;
    use fop_types::{Color, Length, Point, Rect, Size};

    fn make_rect(w: f64, h: f64) -> Rect {
        Rect::from_point_size(
            Point::ZERO,
            Size::new(Length::from_pt(w), Length::from_pt(h)),
        )
    }

    fn make_rect_at(x: f64, y: f64, w: f64, h: f64) -> Rect {
        Rect::from_point_size(
            Point::new(Length::from_pt(x), Length::from_pt(y)),
            Size::new(Length::from_pt(w), Length::from_pt(h)),
        )
    }

    // ---- Area construction tests ----

    #[test]
    fn test_area_new_creates_area_with_correct_type() {
        let area = Area::new(AreaType::Block, make_rect(100.0, 50.0));
        assert_eq!(area.area_type, AreaType::Block);
    }

    #[test]
    fn test_area_new_stores_geometry() {
        let rect = make_rect(200.0, 100.0);
        let area = Area::new(AreaType::Page, rect);
        assert_eq!(area.geometry.width, Length::from_pt(200.0));
        assert_eq!(area.geometry.height, Length::from_pt(100.0));
    }

    #[test]
    fn test_area_new_has_no_content_by_default() {
        let area = Area::new(AreaType::Block, make_rect(100.0, 50.0));
        assert!(area.content.is_none());
        assert!(!area.has_text());
        assert!(!area.has_image_data());
    }

    #[test]
    fn test_area_new_has_default_traits() {
        let area = Area::new(AreaType::Block, make_rect(100.0, 50.0));
        assert!(area.traits.color.is_none());
        assert!(area.traits.background_color.is_none());
        assert!(area.traits.font_size.is_none());
    }

    #[test]
    fn test_area_width_returns_geometry_width() {
        let area = Area::new(AreaType::Block, make_rect(300.0, 50.0));
        assert_eq!(area.width(), Length::from_pt(300.0));
    }

    #[test]
    fn test_area_height_returns_geometry_height() {
        let area = Area::new(AreaType::Block, make_rect(100.0, 75.0));
        assert_eq!(area.height(), Length::from_pt(75.0));
    }

    // ---- Text area tests ----

    #[test]
    fn test_area_text_constructor() {
        let area = Area::text(make_rect(100.0, 12.0), "hello".to_string());
        assert_eq!(area.area_type, AreaType::Text);
        assert!(area.has_text());
    }

    #[test]
    fn test_area_text_content_returns_string() {
        let area = Area::text(make_rect(100.0, 12.0), "hello world".to_string());
        assert_eq!(area.text_content(), Some("hello world"));
    }

    #[test]
    fn test_area_text_content_none_for_block() {
        let area = Area::new(AreaType::Block, make_rect(100.0, 50.0));
        assert!(area.text_content().is_none());
    }

    // ---- Viewport / image tests ----

    #[test]
    fn test_area_viewport_with_image() {
        let image_data = vec![0u8, 1u8, 2u8, 3u8];
        let area = Area::viewport_with_image(make_rect(100.0, 100.0), image_data.clone());
        assert_eq!(area.area_type, AreaType::Viewport);
        assert!(area.has_image_data());
        assert_eq!(area.image_data(), Some(image_data.as_slice()));
    }

    #[test]
    fn test_area_image_data_none_for_text_area() {
        let area = Area::text(make_rect(100.0, 12.0), "text".to_string());
        assert!(area.image_data().is_none());
    }

    // ---- with_traits tests ----

    #[test]
    fn test_area_with_traits_sets_color() {
        let traits = TraitSet {
            color: Some(Color::rgb(255, 0, 0)),
            ..Default::default()
        };
        let area = Area::new(AreaType::Block, make_rect(100.0, 50.0)).with_traits(traits);
        assert!(area.traits.color.is_some());
    }

    #[test]
    fn test_area_with_traits_sets_font_size() {
        let traits = TraitSet {
            font_size: Some(Length::from_pt(14.0)),
            ..Default::default()
        };
        let area = Area::new(AreaType::Text, make_rect(100.0, 14.0)).with_traits(traits);
        assert_eq!(area.traits.font_size, Some(Length::from_pt(14.0)));
    }

    // ---- AreaTree integration via public API ----

    #[test]
    fn test_area_tree_new_is_empty() {
        let tree = AreaTree::new();
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
    }

    #[test]
    fn test_area_tree_add_returns_valid_id() {
        let mut tree = AreaTree::new();
        let id = tree.add_area(Area::new(AreaType::Page, make_rect(595.0, 842.0)));
        assert!(tree.get(id).is_some());
    }

    #[test]
    fn test_area_tree_root_is_first_added() {
        let mut tree = AreaTree::new();
        let id = tree.add_area(Area::new(AreaType::Page, make_rect(595.0, 842.0)));
        let (root_id, _) = tree.root().expect("test: should succeed");
        assert_eq!(root_id, id);
    }

    #[test]
    fn test_area_tree_children_empty_for_new_area() {
        let mut tree = AreaTree::new();
        let id = tree.add_area(Area::new(AreaType::Block, make_rect(100.0, 50.0)));
        let children = tree.children(id);
        assert!(children.is_empty());
    }

    #[test]
    fn test_area_tree_append_child_updates_parent() {
        let mut tree = AreaTree::new();
        let parent = tree.add_area(Area::new(AreaType::Page, make_rect(595.0, 842.0)));
        let child = tree.add_area(Area::new(AreaType::Block, make_rect(400.0, 50.0)));
        tree.append_child(parent, child)
            .expect("test: should succeed");
        assert_eq!(
            tree.get(child).expect("test: should succeed").parent,
            Some(parent)
        );
    }

    #[test]
    fn test_area_tree_multiple_pages() {
        let mut tree = AreaTree::new();
        let p1 = tree.add_area(Area::new(AreaType::Page, make_rect(595.0, 842.0)));
        let p2 = tree.add_area(Area::new(AreaType::Page, make_rect(595.0, 842.0)));
        let _ = (p1, p2);
        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn test_area_tree_geometry_preserved() {
        let mut tree = AreaTree::new();
        let rect = make_rect_at(10.0, 20.0, 200.0, 100.0);
        let id = tree.add_area(Area::new(AreaType::Block, rect));
        let node = tree.get(id).expect("test: should succeed");
        assert_eq!(node.area.geometry.width, Length::from_pt(200.0));
        assert_eq!(node.area.geometry.height, Length::from_pt(100.0));
    }

    // ---- AreaId tests ----

    #[test]
    fn test_area_id_equality() {
        let id_a = AreaId::from_index(5);
        let id_b = AreaId::from_index(5);
        assert_eq!(id_a, id_b);
    }

    #[test]
    fn test_area_id_inequality() {
        let id_a = AreaId::from_index(5);
        let id_b = AreaId::from_index(6);
        assert_ne!(id_a, id_b);
    }

    #[test]
    fn test_area_id_display_format() {
        let id = AreaId::from_index(3);
        assert_eq!(format!("{}", id), "Area(3)");
    }

    // ---- AreaType distinctness ----

    #[test]
    fn test_area_types_page_not_block() {
        assert_ne!(AreaType::Page, AreaType::Block);
    }

    #[test]
    fn test_area_types_region_not_line() {
        assert_ne!(AreaType::Region, AreaType::Line);
    }

    #[test]
    fn test_area_types_text_not_inline() {
        assert_ne!(AreaType::Text, AreaType::Inline);
    }

    // ---- AreaNode tests ----

    #[test]
    fn test_area_node_new_has_no_parent() {
        let node = AreaNode::new(Area::new(AreaType::Block, make_rect(100.0, 50.0)));
        assert!(node.parent.is_none());
        assert!(node.first_child.is_none());
        assert!(node.next_sibling.is_none());
    }

    #[test]
    fn test_area_node_has_children_false_initially() {
        let node = AreaNode::new(Area::new(AreaType::Block, make_rect(100.0, 50.0)));
        assert!(!node.has_children());
    }

    // ---- document_lang field ----

    #[test]
    fn test_area_tree_document_lang_none_by_default() {
        let tree = AreaTree::new();
        assert!(tree.document_lang.is_none());
    }

    #[test]
    fn test_area_tree_document_lang_can_be_set() {
        let mut tree = AreaTree::new();
        tree.document_lang = Some("en-US".to_string());
        assert_eq!(tree.document_lang.as_deref(), Some("en-US"));
    }
}
