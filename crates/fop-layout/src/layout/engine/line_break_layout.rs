//! Text line-breaking and per-line emission for block layout.
//!
//! This bridges the [`KnuthPlassBreaker`] optimal line breaker and the area
//! tree: it tokenizes a text run, breaks it into lines over the block's
//! content width using the resolved font metrics, and appends one positioned
//! [`Area::text`] per line, honouring `text-align` for horizontal placement and
//! width.

use crate::area::{Area, AreaId, AreaTree, TraitSet};
use crate::layout::{KnuthPlassBreaker, TextAlign};
use fop_types::{Length, Rect, Result};

use super::LayoutEngine;

impl LayoutEngine {
    /// Break `text` into optimal lines over `content_width` and append one text
    /// area per line under `block_area_id`.
    ///
    /// Lines are stacked starting at vertical offset `start_y`, each advancing
    /// by `line_height`. The first emitted line is indented by
    /// `first_line_indent` (used for `text-indent`); subsequent lines start at
    /// the block's start edge. Horizontal placement and width follow
    /// `text_align`:
    ///
    /// * **Left** — flush start, area width = the line's natural width.
    /// * **Right** — flush end (right edge at `content_width`).
    /// * **Center** — centred in the available measure.
    /// * **Justify** — non-final lines fill the whole measure (`width = avail`);
    ///   the final line is set ragged (flush start at natural width).
    ///
    /// Returns the vertical offset after the last line. Empty or
    /// whitespace-only `text` emits nothing and returns `start_y` unchanged, so
    /// the caller can preserve the historic single-line block height for empty
    /// blocks.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_text_lines(
        &self,
        area_tree: &mut AreaTree,
        block_area_id: AreaId,
        text: &str,
        traits: &TraitSet,
        text_align: TextAlign,
        content_width: Length,
        first_line_indent: Length,
        start_y: Length,
        line_height: Length,
    ) -> Result<Length> {
        let justify = matches!(text_align, TextAlign::Justify);
        let breaker = KnuthPlassBreaker::new(content_width)
            .with_justify(justify)
            .with_first_line_indent(first_line_indent);
        let lines = breaker.break_into_lines(text, traits);

        let mut y = start_y;
        let line_count = lines.len();
        for (i, line) in lines.iter().enumerate() {
            let indent = if i == 0 {
                first_line_indent
            } else {
                Length::ZERO
            };
            let avail = content_width - indent;
            let natural = line.natural_width;
            let is_last = i + 1 == line_count;

            let (x, width) = match text_align {
                TextAlign::Left => (indent, natural),
                TextAlign::Right => (content_width - natural, natural),
                TextAlign::Center => (indent + (avail - natural) / 2, natural),
                TextAlign::Justify => {
                    if is_last {
                        (indent, natural)
                    } else {
                        (indent, avail)
                    }
                }
            };

            let rect = Rect::new(x, y, width, line_height);
            let area = Area::text(rect, line.text.clone()).with_traits(traits.clone());
            let id = area_tree.add_area(area);
            area_tree
                .append_child(block_area_id, id)
                .map_err(fop_types::FopError::Generic)?;

            y += line_height;
        }

        Ok(y)
    }
}

#[cfg(test)]
mod tests {
    use crate::area::{Area, AreaTree, AreaType};
    use crate::layout::engine::LayoutEngine;
    use crate::layout::PageNumberResolver;
    use fop_core::{FoArena, FoNode, FoNodeData, NodeId, PropertyId, PropertyList, PropertyValue};
    use fop_types::{Length, Rect};

    /// Build an `fo:block` with the given font-size, line-height and a single
    /// text child, returning the arena and the block node id.
    fn make_text_block(
        text: &str,
        font_size_pt: f64,
        line_height_pt: f64,
        text_align: Option<&str>,
    ) -> (FoArena<'static>, NodeId) {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(font_size_pt)),
        );
        props.set(
            PropertyId::LineHeight,
            PropertyValue::Length(Length::from_pt(line_height_pt)),
        );
        if let Some(align) = text_align {
            props.set(
                PropertyId::TextAlign,
                PropertyValue::String(std::borrow::Cow::Owned(align.to_string())),
            );
        }
        let mut fo_tree = FoArena::new();
        let block = fo_tree.add_node(FoNode::new(FoNodeData::Block { properties: props }));
        let text_node = fo_tree.add_node(FoNode::new(FoNodeData::Text(text.to_string())));
        fo_tree
            .append_child(block, text_node)
            .expect("test: append text child");
        (fo_tree, block)
    }

    fn add_page_parent(area_tree: &mut AreaTree, width_pt: f64) -> crate::area::AreaId {
        let rect = Rect::new(
            Length::ZERO,
            Length::ZERO,
            Length::from_pt(width_pt),
            Length::from_pt(800.0),
        );
        area_tree.add_area(Area::new(AreaType::Page, rect))
    }

    /// Lay out a single block and return (block area id, area tree).
    fn layout_one_block(
        fo_tree: &FoArena,
        block: NodeId,
        available_width_pt: f64,
    ) -> (crate::area::AreaId, AreaTree) {
        let mut area_tree = AreaTree::new();
        let parent = add_page_parent(&mut area_tree, available_width_pt);
        let engine = LayoutEngine::new();
        let mut resolver = PageNumberResolver::new();
        let area_id = engine
            .layout_block(
                fo_tree,
                block,
                &mut area_tree,
                parent,
                Length::ZERO,
                Length::from_pt(available_width_pt),
                &mut resolver,
            )
            .expect("test: layout_block ok")
            .expect("test: block area id");
        (area_id, area_tree)
    }

    fn text_children(
        area_tree: &AreaTree,
        block_id: crate::area::AreaId,
    ) -> Vec<crate::area::AreaId> {
        area_tree
            .children(block_id)
            .into_iter()
            .filter(|id| {
                area_tree
                    .get(*id)
                    .map(|n| n.area.area_type == AreaType::Text)
                    .unwrap_or(false)
            })
            .collect()
    }

    // ── A long paragraph wraps into multiple line areas ──────────────────────

    #[test]
    fn test_long_paragraph_wraps_into_multiple_lines() {
        let text = "The quick brown fox jumps over the lazy dog while the sun sets slowly";
        let (fo_tree, block) = make_text_block(text, 12.0, 14.0, None);
        let (block_id, area_tree) = layout_one_block(&fo_tree, block, 150.0);

        let lines = text_children(&area_tree, block_id);
        assert!(
            lines.len() >= 2,
            "expected the paragraph to wrap into >= 2 lines, got {}",
            lines.len()
        );
    }

    #[test]
    fn test_each_line_width_within_content_width() {
        let content_width = 150.0;
        let text = "The quick brown fox jumps over the lazy dog while the sun sets slowly today";
        let (fo_tree, block) = make_text_block(text, 12.0, 14.0, None); // left/ragged
        let (block_id, area_tree) = layout_one_block(&fo_tree, block, content_width);

        for id in text_children(&area_tree, block_id) {
            let node = area_tree.get(id).expect("test: text node");
            assert!(
                node.area.geometry.width <= Length::from_pt(content_width),
                "line width {}pt exceeds content width {}pt for {:?}",
                node.area.geometry.width.to_pt(),
                content_width,
                node.area.text_content()
            );
        }
    }

    #[test]
    fn test_block_height_equals_line_count_times_line_height() {
        let line_height = 14.0;
        let text = "The quick brown fox jumps over the lazy dog while the sun sets slowly today";
        let (fo_tree, block) = make_text_block(text, 12.0, line_height, None);
        let (block_id, area_tree) = layout_one_block(&fo_tree, block, 150.0);

        let n = text_children(&area_tree, block_id).len();
        assert!(n >= 2, "precondition: text should wrap");
        let block_height = area_tree
            .get(block_id)
            .expect("test: block node")
            .area
            .geometry
            .height;
        assert_eq!(
            block_height,
            Length::from_pt(line_height) * n as i32,
            "block height must equal n_lines ({}) * line_height ({}pt)",
            n,
            line_height
        );
    }

    #[test]
    fn test_lines_stack_at_increasing_y() {
        let line_height = 14.0;
        let text = "The quick brown fox jumps over the lazy dog while the sun sets slowly today";
        let (fo_tree, block) = make_text_block(text, 12.0, line_height, None);
        let (block_id, area_tree) = layout_one_block(&fo_tree, block, 150.0);

        let lines = text_children(&area_tree, block_id);
        for (i, id) in lines.iter().enumerate() {
            let y = area_tree.get(*id).expect("test: node").area.geometry.y;
            assert_eq!(
                y,
                Length::from_pt(line_height) * i as i32,
                "line {} should sit at y = {} * {}pt",
                i,
                i,
                line_height
            );
        }
    }

    // ── A single over-long word does not panic and occupies one line ─────────

    #[test]
    fn test_overlong_word_does_not_panic() {
        let text = "Supercalifragilisticexpialidocious";
        let (fo_tree, block) = make_text_block(text, 12.0, 14.0, None);
        let (block_id, area_tree) = layout_one_block(&fo_tree, block, 40.0);

        let lines = text_children(&area_tree, block_id);
        assert_eq!(lines.len(), 1, "an unbreakable word stays on one line");
        let node = area_tree.get(lines[0]).expect("test: node");
        assert_eq!(node.area.text_content(), Some(text));
    }

    // ── justify vs left differ in line geometry ──────────────────────────────

    #[test]
    fn test_justify_vs_left_first_line_width_differs() {
        let content_width = 150.0;
        let text = "The quick brown fox jumps over the lazy dog while the sun sets slowly today";

        let (left_tree_arena, left_block) = make_text_block(text, 12.0, 14.0, Some("left"));
        let (left_id, left_tree) = layout_one_block(&left_tree_arena, left_block, content_width);

        let (just_arena, just_block) = make_text_block(text, 12.0, 14.0, Some("justify"));
        let (just_id, just_tree) = layout_one_block(&just_arena, just_block, content_width);

        let left_lines = text_children(&left_tree, left_id);
        let just_lines = text_children(&just_tree, just_id);
        assert!(left_lines.len() >= 2 && just_lines.len() >= 2);

        // The first (non-final) justified line is stretched to the full measure,
        // whereas the left-aligned first line keeps its natural (shorter) width.
        let left_first = left_tree
            .get(left_lines[0])
            .expect("test")
            .area
            .geometry
            .width;
        let just_first = just_tree
            .get(just_lines[0])
            .expect("test")
            .area
            .geometry
            .width;
        assert_eq!(just_first, Length::from_pt(content_width));
        assert!(
            just_first > left_first,
            "justified first line ({}pt) should be wider than left ({}pt)",
            just_first.to_pt(),
            left_first.to_pt()
        );
    }

    #[test]
    fn test_right_align_flushes_to_right_edge() {
        let content_width = 200.0;
        let text = "alpha beta gamma";
        let (fo_tree, block) = make_text_block(text, 12.0, 14.0, Some("right"));
        let (block_id, area_tree) = layout_one_block(&fo_tree, block, content_width);

        let lines = text_children(&area_tree, block_id);
        assert_eq!(lines.len(), 1, "short text fits one line");
        let node = area_tree.get(lines[0]).expect("test: node");
        // Right edge (x + width) should land at the content width.
        let right_edge = node.area.geometry.x + node.area.geometry.width;
        assert_eq!(right_edge, Length::from_pt(content_width));
    }

    // ── Empty block keeps a single line of height ────────────────────────────

    #[test]
    fn test_empty_block_keeps_one_line_height() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::LineHeight,
            PropertyValue::Length(Length::from_pt(16.0)),
        );
        let mut fo_tree = FoArena::new();
        let block = fo_tree.add_node(FoNode::new(FoNodeData::Block { properties: props }));
        let (block_id, area_tree) = layout_one_block(&fo_tree, block, 200.0);

        assert!(text_children(&area_tree, block_id).is_empty());
        let height = area_tree
            .get(block_id)
            .expect("test: block")
            .area
            .geometry
            .height;
        assert_eq!(height, Length::from_pt(16.0));
    }
}
