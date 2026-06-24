
use super::*;
use crate::area::AreaType;
use fop_core::{FoNode, PropertyList, PropertyValue};
use std::borrow::Cow;

/// Specification for one flow block: its (line-)height in points and an
/// optional forced `break-before` value (e.g. `"page"`).
struct BlockSpec {
    height_pt: f64,
    break_before: Option<&'static str>,
}

fn block(height_pt: f64) -> BlockSpec {
    BlockSpec {
        height_pt,
        break_before: None,
    }
}

fn block_break_before(height_pt: f64, value: &'static str) -> BlockSpec {
    BlockSpec {
        height_pt,
        break_before: Some(value),
    }
}

/// Build a single-page-sequence document with an explicit simple-page-master
/// so the region geometry is fully deterministic.
///
/// * `page_w_pt` / `page_h_pt` — page size; all four page margins are 0.
/// * `before_extent_pt` — `region-before` extent (0 ⇒ no header region).
/// * `header` — if true, a `static-content` (region-before) with one block.
/// * `blocks` — the flow's blocks; each block's height equals its line-height
///   (the blocks carry no text, so block height == resolved line-height).
fn build_doc(
    page_w_pt: f64,
    page_h_pt: f64,
    before_extent_pt: f64,
    header: bool,
    blocks: &[BlockSpec],
) -> FoArena<'static> {
    build_doc_columns(
        page_w_pt,
        page_h_pt,
        before_extent_pt,
        header,
        1,
        0.0,
        blocks,
    )
}

/// Like [`build_doc`], but sets `column-count` / `column-gap` on the flow so
/// the multi-column paginator is exercised.  `column_count <= 1` yields a
/// single-column flow identical to [`build_doc`].
#[allow(clippy::too_many_arguments)]
fn build_doc_columns(
    page_w_pt: f64,
    page_h_pt: f64,
    before_extent_pt: f64,
    header: bool,
    column_count: i32,
    column_gap_pt: f64,
    blocks: &[BlockSpec],
) -> FoArena<'static> {
    let mut fo = FoArena::new();
    let root = fo.add_node(FoNode::new(FoNodeData::Root));

    // --- layout-master-set / simple-page-master ---
    let lms = fo.add_node(FoNode::new(FoNodeData::LayoutMasterSet));
    fo.append_child(root, lms).expect("test: append lms");

    let mut spm_props = PropertyList::new();
    spm_props.set(
        PropertyId::PageWidth,
        PropertyValue::Length(Length::from_pt(page_w_pt)),
    );
    spm_props.set(
        PropertyId::PageHeight,
        PropertyValue::Length(Length::from_pt(page_h_pt)),
    );
    for margin in [
        PropertyId::MarginTop,
        PropertyId::MarginBottom,
        PropertyId::MarginLeft,
        PropertyId::MarginRight,
    ] {
        spm_props.set(margin, PropertyValue::Length(Length::ZERO));
    }
    let spm = fo.add_node(FoNode::new(FoNodeData::SimplePageMaster {
        master_name: "pm".to_string(),
        properties: spm_props,
    }));
    fo.append_child(lms, spm).expect("test: append spm");

    if before_extent_pt > 0.0 {
        let mut rb_props = PropertyList::new();
        rb_props.set(
            PropertyId::Extent,
            PropertyValue::Length(Length::from_pt(before_extent_pt)),
        );
        let rb = fo.add_node(FoNode::new(FoNodeData::RegionBefore {
            properties: rb_props,
        }));
        fo.append_child(spm, rb)
            .expect("test: append region-before");
    }
    let body = fo.add_node(FoNode::new(FoNodeData::RegionBody {
        properties: PropertyList::new(),
    }));
    fo.append_child(spm, body)
        .expect("test: append region-body");

    // --- page-sequence ---
    let ps = fo.add_node(FoNode::new(FoNodeData::PageSequence {
        master_reference: "pm".to_string(),
        format: "1".to_string(),
        grouping_separator: None,
        grouping_size: None,
        properties: PropertyList::new(),
    }));
    fo.append_child(root, ps)
        .expect("test: append page-sequence");

    if header {
        let sc = fo.add_node(FoNode::new(FoNodeData::StaticContent {
            flow_name: "xsl-region-before".to_string(),
            properties: PropertyList::new(),
        }));
        fo.append_child(ps, sc)
            .expect("test: append static-content");
        let mut hp = PropertyList::new();
        hp.set(
            PropertyId::LineHeight,
            PropertyValue::Length(Length::from_pt(20.0)),
        );
        let hb = fo.add_node(FoNode::new(FoNodeData::Block { properties: hp }));
        fo.append_child(sc, hb).expect("test: append header block");
        let ht = fo.add_node(FoNode::new(FoNodeData::Text("HEADER".to_string())));
        fo.append_child(hb, ht).expect("test: append header text");
    }

    let mut flow_props = PropertyList::new();
    if column_count > 1 {
        flow_props.set(
            PropertyId::ColumnCount,
            PropertyValue::Integer(column_count),
        );
        flow_props.set(
            PropertyId::ColumnGap,
            PropertyValue::Length(Length::from_pt(column_gap_pt)),
        );
    }
    let flow = fo.add_node(FoNode::new(FoNodeData::Flow {
        flow_name: "xsl-region-body".to_string(),
        properties: flow_props,
    }));
    fo.append_child(ps, flow).expect("test: append flow");

    for spec in blocks {
        let mut bp = PropertyList::new();
        bp.set(
            PropertyId::LineHeight,
            PropertyValue::Length(Length::from_pt(spec.height_pt)),
        );
        if let Some(bb) = spec.break_before {
            bp.set(
                PropertyId::BreakBefore,
                PropertyValue::String(Cow::Borrowed(bb)),
            );
        }
        let b = fo.add_node(FoNode::new(FoNodeData::Block { properties: bp }));
        fo.append_child(flow, b).expect("test: append flow block");
    }

    fo
}

/// All top-level `Page` areas in tree order.
fn page_ids(tree: &AreaTree) -> Vec<AreaId> {
    tree.iter()
        .filter(|(_, node)| node.area.area_type == AreaType::Page)
        .map(|(id, _)| id)
        .collect()
}

/// The region-body of a page (its single `Region`-typed child).
fn region_of(tree: &AreaTree, page_id: AreaId) -> AreaId {
    tree.children(page_id)
        .into_iter()
        .find(|c| {
            tree.get(*c)
                .map(|n| n.area.area_type == AreaType::Region)
                .unwrap_or(false)
        })
        .expect("test: page must have a region-body")
}

/// The `Block` children of a region-body.
fn blocks_of(tree: &AreaTree, region_id: AreaId) -> Vec<AreaId> {
    tree.children(region_id)
        .into_iter()
        .filter(|c| {
            tree.get(*c)
                .map(|n| n.area.area_type == AreaType::Block)
                .unwrap_or(false)
        })
        .collect()
}

/// Tall flow ⇒ ≥2 pages, real reparenting, and every page's content fits
/// inside its region-body.
///
/// Geometry: page 200×250pt, margins 0, no header ⇒ body-rect = 200×250pt.
/// Blocks: 4 × 100pt. Two blocks fill 200pt (≤250); a third would reach
/// 300pt (>250) ⇒ 2 blocks per page ⇒ ceil(4/2) = 2 pages.
#[test]
fn test_overflow_produces_multiple_pages_with_reparenting() {
    let doc = build_doc(
        200.0,
        250.0,
        0.0,
        false,
        &[block(100.0), block(100.0), block(100.0), block(100.0)],
    );
    let engine = LayoutEngine::new();
    let tree = engine.layout(&doc).expect("test: layout should succeed");

    let pages = page_ids(&tree);
    assert_eq!(
        pages.len(),
        2,
        "4×100pt blocks in a 250pt body must paginate to 2 pages"
    );

    // Every page's region-body content must fit within the body height.
    let mut total_blocks = 0;
    for page_id in &pages {
        let region_id = region_of(&tree, *page_id);
        let body_height = tree
            .get(region_id)
            .expect("test: region exists")
            .area
            .height();
        for block_id in blocks_of(&tree, region_id) {
            let b = tree.get(block_id).expect("test: block exists");
            let bottom = b.area.geometry.y + b.area.height();
            assert!(
                bottom <= body_height,
                "block bottom {}pt must not exceed body height {}pt",
                bottom.to_pt(),
                body_height.to_pt()
            );
            total_blocks += 1;
        }
    }
    assert_eq!(total_blocks, 4, "all 4 blocks must be placed exactly once");

    // Real reparenting: the second page's blocks must genuinely live under
    // the second page's region-body (verified through the parent links).
    let page2 = pages[1];
    let region2 = region_of(&tree, page2);
    let page2_blocks = blocks_of(&tree, region2);
    assert_eq!(page2_blocks.len(), 2, "page 2 holds the 2 overflow blocks");
    for block_id in page2_blocks {
        let block = tree.get(block_id).expect("test: block exists");
        assert_eq!(
            block.parent,
            Some(region2),
            "overflow block must be parented to page 2's region-body"
        );
        // The first overflow block restarts at the body top (y = 0).
    }
    assert_eq!(
        tree.get(region2).expect("test: region2 exists").parent,
        Some(page2),
        "region-body must be parented to its page"
    );
    // The first overflow block of page 2 sits flush at the body top.
    let first_p2_block = blocks_of(&tree, region2)[0];
    assert_eq!(
        tree.get(first_p2_block)
            .expect("test: block exists")
            .area
            .geometry
            .y,
        Length::ZERO,
        "first block on a new page restarts at the body top"
    );
}

/// Static content (a header) repeats on every page of a multi-page sequence.
///
/// Geometry: page 200×300pt, margins 0, region-before extent 40pt ⇒
/// body height = 300 − 40 = 260pt. Blocks: 5 × 100pt ⇒ 2 per page
/// (3rd would reach 300pt > 260) ⇒ ceil(5/2) = 3 pages ⇒ 3 headers.
#[test]
fn test_header_repeats_on_every_page() {
    let doc = build_doc(
        200.0,
        300.0,
        40.0,
        true,
        &[
            block(100.0),
            block(100.0),
            block(100.0),
            block(100.0),
            block(100.0),
        ],
    );
    let engine = LayoutEngine::new();
    let tree = engine.layout(&doc).expect("test: layout should succeed");

    let pages = page_ids(&tree);
    assert_eq!(
        pages.len(),
        3,
        "5×100pt blocks in a 260pt body must paginate to 3 pages"
    );

    // Exactly one Header area per page, each parented to a distinct page.
    let mut header_pages = Vec::new();
    for (id, node) in tree.iter() {
        if node.area.area_type == AreaType::Header {
            header_pages.push(node.parent.expect("test: header has a parent"));
            // Sanity: the header area id is real.
            let _ = id;
        }
    }
    assert_eq!(
        header_pages.len(),
        3,
        "the header static-content must repeat on all 3 pages"
    );
    header_pages.sort_by_key(|p| p.index());
    header_pages.dedup();
    assert_eq!(
        header_pages.len(),
        3,
        "each repeated header must belong to a distinct page"
    );
}

/// A short document still produces exactly one page (regression guard).
///
/// Geometry: page 200×250pt, margins 0 ⇒ body 250pt. Two 100pt blocks total
/// 200pt ≤ 250pt ⇒ a single page.
#[test]
fn test_short_document_is_single_page() {
    let doc = build_doc(200.0, 250.0, 0.0, false, &[block(100.0), block(100.0)]);
    let engine = LayoutEngine::new();
    let tree = engine.layout(&doc).expect("test: layout should succeed");

    let pages = page_ids(&tree);
    assert_eq!(pages.len(), 1, "200pt of content fits one 250pt body");

    let region_id = region_of(&tree, pages[0]);
    assert_eq!(
        blocks_of(&tree, region_id).len(),
        2,
        "both blocks live on the single page"
    );
}

/// A forced `break-before="page"` starts a new page even when the content
/// would otherwise fit, and the block is reparented onto the new page.
///
/// Geometry: page 200×600pt, margins 0 ⇒ body 600pt (no height overflow).
/// Two 50pt blocks easily fit, but block 2 carries break-before=page.
#[test]
fn test_forced_break_before_starts_new_page() {
    let doc = build_doc(
        200.0,
        600.0,
        0.0,
        false,
        &[block(50.0), block_break_before(50.0, "page")],
    );
    let engine = LayoutEngine::new();
    let tree = engine.layout(&doc).expect("test: layout should succeed");

    let pages = page_ids(&tree);
    assert_eq!(
        pages.len(),
        2,
        "break-before=page must force a 2nd page despite the content fitting"
    );

    let region1 = region_of(&tree, pages[0]);
    let region2 = region_of(&tree, pages[1]);
    assert_eq!(
        blocks_of(&tree, region1).len(),
        1,
        "the first block stays on page 1"
    );
    let p2_blocks = blocks_of(&tree, region2);
    assert_eq!(p2_blocks.len(), 1, "the break-before block moves to page 2");
    assert_eq!(
        tree.get(p2_blocks[0]).expect("test: block exists").parent,
        Some(region2),
        "the break-before block is parented under page 2's region-body"
    );
}

/// Keep-with-previous drags the preceding block onto the new page so the
/// pair is not split by a height-overflow break.
///
/// Geometry: page 200×250pt, margins 0 ⇒ body 250pt. Blocks: b1=100pt,
/// b2=100pt, b3=100pt with keep-with-previous. Naively b1,b2 fill page 1
/// (200pt) and b3 overflows alone; but keep-with-previous glues b3 to b2, so
/// the b2+b3 pair migrates together ⇒ page 1 = [b1], page 2 = [b2, b3].
#[test]
fn test_keep_with_previous_migrates_pair() {
    let mut fo = FoArena::new();
    let root = fo.add_node(FoNode::new(FoNodeData::Root));
    let lms = fo.add_node(FoNode::new(FoNodeData::LayoutMasterSet));
    fo.append_child(root, lms).expect("test: append lms");

    let mut spm_props = PropertyList::new();
    spm_props.set(
        PropertyId::PageWidth,
        PropertyValue::Length(Length::from_pt(200.0)),
    );
    spm_props.set(
        PropertyId::PageHeight,
        PropertyValue::Length(Length::from_pt(250.0)),
    );
    for margin in [
        PropertyId::MarginTop,
        PropertyId::MarginBottom,
        PropertyId::MarginLeft,
        PropertyId::MarginRight,
    ] {
        spm_props.set(margin, PropertyValue::Length(Length::ZERO));
    }
    let spm = fo.add_node(FoNode::new(FoNodeData::SimplePageMaster {
        master_name: "pm".to_string(),
        properties: spm_props,
    }));
    fo.append_child(lms, spm).expect("test: append spm");
    let body = fo.add_node(FoNode::new(FoNodeData::RegionBody {
        properties: PropertyList::new(),
    }));
    fo.append_child(spm, body)
        .expect("test: append region-body");

    let ps = fo.add_node(FoNode::new(FoNodeData::PageSequence {
        master_reference: "pm".to_string(),
        format: "1".to_string(),
        grouping_separator: None,
        grouping_size: None,
        properties: PropertyList::new(),
    }));
    fo.append_child(root, ps)
        .expect("test: append page-sequence");
    let flow = fo.add_node(FoNode::new(FoNodeData::Flow {
        flow_name: "xsl-region-body".to_string(),
        properties: PropertyList::new(),
    }));
    fo.append_child(ps, flow).expect("test: append flow");

    // b1, b2 (plain 100pt) and b3 (100pt, keep-with-previous=always).
    for keep in [false, false, true] {
        let mut bp = PropertyList::new();
        bp.set(
            PropertyId::LineHeight,
            PropertyValue::Length(Length::from_pt(100.0)),
        );
        if keep {
            bp.set(
                PropertyId::KeepWithPrevious,
                PropertyValue::String(Cow::Borrowed("always")),
            );
        }
        let b = fo.add_node(FoNode::new(FoNodeData::Block { properties: bp }));
        fo.append_child(flow, b).expect("test: append block");
    }

    let engine = LayoutEngine::new();
    let tree = engine.layout(&fo).expect("test: layout should succeed");

    let pages = page_ids(&tree);
    assert_eq!(pages.len(), 2, "the glued pair forces a 2-page layout");

    let region1 = region_of(&tree, pages[0]);
    let region2 = region_of(&tree, pages[1]);
    assert_eq!(
        blocks_of(&tree, region1).len(),
        1,
        "page 1 keeps only b1 — b2 is dragged forward by b3's keep-with-previous"
    );
    assert_eq!(
        blocks_of(&tree, region2).len(),
        2,
        "page 2 holds the glued b2+b3 pair"
    );
}

// -----------------------------------------------------------------------
// Multi-column cross-page pagination
// -----------------------------------------------------------------------

/// The x (column) offset of a block area, in points.
fn block_x_pt(tree: &AreaTree, block_id: AreaId) -> f64 {
    tree.get(block_id)
        .expect("test: block exists")
        .area
        .geometry
        .x
        .to_pt()
}

/// The y (in-column) offset of a block area, in points.
fn block_y_pt(tree: &AreaTree, block_id: AreaId) -> f64 {
    tree.get(block_id)
        .expect("test: block exists")
        .area
        .geometry
        .y
        .to_pt()
}

/// A 2-column flow with more content than fits both columns of one page must
/// paginate to ≥2 pages, placing every block exactly once.
///
/// Geometry: page 200×250pt, margins 0, no header ⇒ body 200×250pt, 2 columns
/// gap 0 ⇒ each column is 100pt wide and 250pt tall ⇒ holds two 100pt blocks
/// (a third reaches 300 > 250).  6 blocks ⇒ page 1 fills both columns (4
/// blocks) and 2 spill onto page 2.
#[test]
fn test_multicolumn_overflow_produces_multiple_pages() {
    let doc = build_doc_columns(
        200.0,
        250.0,
        0.0,
        false,
        2,
        0.0,
        &[
            block(100.0),
            block(100.0),
            block(100.0),
            block(100.0),
            block(100.0),
            block(100.0),
        ],
    );
    let engine = LayoutEngine::new();
    let tree = engine.layout(&doc).expect("test: layout should succeed");

    let pages = page_ids(&tree);
    assert_eq!(
        pages.len(),
        2,
        "6×100pt blocks in two 250pt columns must paginate to 2 pages"
    );

    let region1 = region_of(&tree, pages[0]);
    let region2 = region_of(&tree, pages[1]);
    assert_eq!(
        blocks_of(&tree, region1).len(),
        4,
        "page 1 fills both columns (2 blocks each)"
    );
    assert_eq!(
        blocks_of(&tree, region2).len(),
        2,
        "the 2 overflow blocks land on page 2"
    );

    // Every block placed exactly once across both pages.
    let total: usize = pages
        .iter()
        .map(|p| blocks_of(&tree, region_of(&tree, *p)).len())
        .sum();
    assert_eq!(total, 6, "all 6 blocks placed exactly once");
}

/// Columns fill left then right before the page breaks: on page 1 the first
/// two blocks sit in the left column (x = 0) and the next two in the right
/// column (x = 100pt), with the right column restarting at the body top.
#[test]
fn test_multicolumn_fills_left_then_right_before_break() {
    let doc = build_doc_columns(
        200.0,
        250.0,
        0.0,
        false,
        2,
        0.0,
        &[
            block(100.0),
            block(100.0),
            block(100.0),
            block(100.0),
            block(100.0),
            block(100.0),
        ],
    );
    let engine = LayoutEngine::new();
    let tree = engine.layout(&doc).expect("test: layout should succeed");

    let pages = page_ids(&tree);
    let region1 = region_of(&tree, pages[0]);
    let p1 = blocks_of(&tree, region1);
    assert_eq!(p1.len(), 4, "page 1 holds 4 blocks (2 per column)");

    // Left column (x = 0) fills first: blocks 0 and 1.
    assert!(
        block_x_pt(&tree, p1[0]).abs() < 0.01,
        "block 1 is in the left column"
    );
    assert!(
        block_x_pt(&tree, p1[1]).abs() < 0.01,
        "block 2 is in the left column"
    );
    assert!(
        block_y_pt(&tree, p1[0]).abs() < 0.01,
        "block 1 sits at the column top"
    );
    assert!(
        (block_y_pt(&tree, p1[1]) - 100.0).abs() < 0.01,
        "block 2 stacks below block 1 in the left column"
    );

    // Right column (x = 100pt) only after the left column is full.
    assert!(
        (block_x_pt(&tree, p1[2]) - 100.0).abs() < 0.01,
        "block 3 starts the right column"
    );
    assert!(
        (block_x_pt(&tree, p1[3]) - 100.0).abs() < 0.01,
        "block 4 is in the right column"
    );
    assert!(
        block_y_pt(&tree, p1[2]).abs() < 0.01,
        "the right column restarts at the body top"
    );
    assert!(
        (block_y_pt(&tree, p1[3]) - 100.0).abs() < 0.01,
        "block 4 stacks below block 3 in the right column"
    );
}

/// Static content (a header) repeats on every page of a multi-column,
/// multi-page sequence.
///
/// Geometry: page 200×300pt, region-before extent 40 ⇒ body 200×260pt, 2
/// columns ⇒ each column holds two 100pt blocks (third reaches 300 > 260).
/// 6 blocks ⇒ page 1 fills both columns (4 blocks), 2 spill to page 2 ⇒ 2
/// pages ⇒ 2 repeated headers.
#[test]
fn test_multicolumn_header_repeats_on_every_page() {
    let doc = build_doc_columns(
        200.0,
        300.0,
        40.0,
        true,
        2,
        0.0,
        &[
            block(100.0),
            block(100.0),
            block(100.0),
            block(100.0),
            block(100.0),
            block(100.0),
        ],
    );
    let engine = LayoutEngine::new();
    let tree = engine.layout(&doc).expect("test: layout should succeed");

    let pages = page_ids(&tree);
    assert_eq!(
        pages.len(),
        2,
        "6 blocks across two 2-column pages ⇒ 2 pages"
    );

    let mut header_pages = Vec::new();
    for (_, node) in tree.iter() {
        if node.area.area_type == AreaType::Header {
            header_pages.push(node.parent.expect("test: header has a parent"));
        }
    }
    assert_eq!(
        header_pages.len(),
        2,
        "the header static-content must repeat on both pages"
    );
    header_pages.sort_by_key(|p| p.index());
    header_pages.dedup();
    assert_eq!(
        header_pages.len(),
        2,
        "each repeated header must belong to a distinct page"
    );
}

/// A short 2-column document stays on a single page; the second column is
/// used once the first is full, but no new page is started.
///
/// Geometry: page 200×250pt, 2 columns ⇒ each column holds two 100pt blocks.
/// 3 blocks ⇒ left column holds blocks 1 & 2, block 3 starts the right
/// column — still one page.
#[test]
fn test_multicolumn_short_document_single_page() {
    let doc = build_doc_columns(
        200.0,
        250.0,
        0.0,
        false,
        2,
        0.0,
        &[block(100.0), block(100.0), block(100.0)],
    );
    let engine = LayoutEngine::new();
    let tree = engine.layout(&doc).expect("test: layout should succeed");

    let pages = page_ids(&tree);
    assert_eq!(pages.len(), 1, "3 blocks fit within one 2-column page");

    let region = region_of(&tree, pages[0]);
    let blocks = blocks_of(&tree, region);
    assert_eq!(blocks.len(), 3, "all 3 blocks live on the single page");

    // Blocks 1 & 2 in the left column, block 3 in the right column.
    assert!(
        block_x_pt(&tree, blocks[0]).abs() < 0.01,
        "block 1 in left column"
    );
    assert!(
        block_x_pt(&tree, blocks[1]).abs() < 0.01,
        "block 2 in left column"
    );
    assert!(
        (block_x_pt(&tree, blocks[2]) - 100.0).abs() < 0.01,
        "block 3 spills into the right column"
    );
}

/// Regression: an explicit `column-count="1"` flow is routed through the
/// single-column paginator and behaves exactly like the default
/// single-column path — vertical stacking (every block at x = 0) with
/// height-overflow pagination.
///
/// Geometry: page 200×250pt, body 250pt, 4×100pt blocks ⇒ 2 blocks per page
/// ⇒ 2 pages, all blocks in a single column at x = 0.
#[test]
fn test_column_count_one_uses_single_column_pagination() {
    let doc = build_doc_columns(
        200.0,
        250.0,
        0.0,
        false,
        1,
        0.0,
        &[block(100.0), block(100.0), block(100.0), block(100.0)],
    );
    let engine = LayoutEngine::new();
    let tree = engine.layout(&doc).expect("test: layout should succeed");

    let pages = page_ids(&tree);
    assert_eq!(
        pages.len(),
        2,
        "column-count=1 must paginate by height like the single-column path"
    );

    for page_id in &pages {
        let region = region_of(&tree, *page_id);
        let blocks = blocks_of(&tree, region);
        assert_eq!(blocks.len(), 2, "2 blocks per page in a single column");
        for block_id in blocks {
            assert!(
                block_x_pt(&tree, block_id).abs() < 0.01,
                "single-column blocks all stack at x = 0"
            );
        }
    }
}

// -----------------------------------------------------------------------
// Newspaper-style multi-column balancing (final page)
// -----------------------------------------------------------------------

/// (a) A short multi-column flow — fewer blocks than would fill one column —
/// is balanced across the columns to minimise the tallest column rather than
/// being packed into column 1.
///
/// Geometry: page 300×1000pt, margins 0, 3 columns gap 0 ⇒ each column 100pt
/// wide and 1000pt tall.  3 × 100pt blocks would all pack into column 1 under
/// the old sequential fill (the column never fills).  Balanced, the smallest
/// feasible target column height is 100pt (one block each), so each block
/// takes its own column: x = 0, 100, 200; every block at the column top.
#[test]
fn test_multicolumn_balances_short_final_page() {
    let doc = build_doc_columns(
        300.0,
        1000.0,
        0.0,
        false,
        3,
        0.0,
        &[block(100.0), block(100.0), block(100.0)],
    );
    let engine = LayoutEngine::new();
    let tree = engine.layout(&doc).expect("test: layout should succeed");

    let pages = page_ids(&tree);
    assert_eq!(pages.len(), 1, "3 short blocks fit on one 3-column page");

    let blocks = blocks_of(&tree, region_of(&tree, pages[0]));
    assert_eq!(blocks.len(), 3, "all 3 blocks placed on the page");

    // Balanced ⇒ one block per column (NOT all stacked in column 1).
    for (i, expected_x) in [0.0, 100.0, 200.0].iter().enumerate() {
        assert!(
            (block_x_pt(&tree, blocks[i]) - expected_x).abs() < 0.01,
            "block {} balanced into its own column at x = {}pt (got {}pt)",
            i,
            expected_x,
            block_x_pt(&tree, blocks[i])
        );
        assert!(
            block_y_pt(&tree, blocks[i]).abs() < 0.01,
            "block {} sits at the top of its column",
            i
        );
    }
}

/// (b) A full first page keeps the sequential left-to-right fill on that page,
/// while the final page is balanced.
///
/// Geometry: page 200×250pt, margins 0, 2 columns gap 0 ⇒ each column 100pt
/// wide, 250pt tall (holds two 100pt blocks; a third reaches 300 > 250).  6 ×
/// 100pt blocks ⇒ page 1 fills both columns (blocks 0,1 then 2,3) and is left
/// sequential; blocks 4,5 spill to page 2 — the final page — which is balanced
/// so they sit side-by-side (one per column) rather than stacked in column 1.
#[test]
fn test_multicolumn_full_page_sequential_final_page_balanced() {
    let doc = build_doc_columns(
        200.0,
        250.0,
        0.0,
        false,
        2,
        0.0,
        &[
            block(100.0),
            block(100.0),
            block(100.0),
            block(100.0),
            block(100.0),
            block(100.0),
        ],
    );
    let engine = LayoutEngine::new();
    let tree = engine.layout(&doc).expect("test: layout should succeed");

    let pages = page_ids(&tree);
    assert_eq!(
        pages.len(),
        2,
        "6 blocks ⇒ a full page 1 plus a final page 2"
    );

    // Page 1 stays sequential: left column (x = 0) packed, then right (x = 100).
    let p1 = blocks_of(&tree, region_of(&tree, pages[0]));
    assert_eq!(p1.len(), 4, "page 1 holds 4 blocks (full, 2 per column)");
    assert!(
        block_x_pt(&tree, p1[0]).abs() < 0.01,
        "p1 block 0 left column"
    );
    assert!(
        block_x_pt(&tree, p1[1]).abs() < 0.01,
        "p1 block 1 left column"
    );
    assert!(
        block_y_pt(&tree, p1[0]).abs() < 0.01,
        "p1 block 0 at column top"
    );
    assert!(
        (block_y_pt(&tree, p1[1]) - 100.0).abs() < 0.01,
        "p1 block 1 stacks below block 0 (sequential, not balanced)"
    );
    assert!(
        (block_x_pt(&tree, p1[2]) - 100.0).abs() < 0.01,
        "p1 block 2 right column"
    );
    assert!(
        (block_x_pt(&tree, p1[3]) - 100.0).abs() < 0.01,
        "p1 block 3 right column"
    );

    // Page 2 (final) is balanced: the two blocks sit one per column, both at
    // the column top — NOT stacked in column 1 as the sequential fill would.
    let p2 = blocks_of(&tree, region_of(&tree, pages[1]));
    assert_eq!(p2.len(), 2, "the 2 trailing blocks land on the final page");
    assert!(
        block_x_pt(&tree, p2[0]).abs() < 0.01,
        "final-page block 0 in the left column"
    );
    assert!(
        (block_x_pt(&tree, p2[1]) - 100.0).abs() < 0.01,
        "final-page block 1 balanced into the right column (x = 100pt)"
    );
    assert!(
        block_y_pt(&tree, p2[0]).abs() < 0.01,
        "final-page block 0 at the column top"
    );
    assert!(
        block_y_pt(&tree, p2[1]).abs() < 0.01,
        "final-page block 1 at the column top (balanced, not stacked)"
    );
}

/// (c) A forced `break-before = column` is honoured under balancing: the
/// flagged block starts a new column even when the balanced partition would
/// otherwise have kept it in the previous column.
///
/// Geometry: page 200×500pt, margins 0, 2 columns gap 0 ⇒ each column 100pt
/// wide, 500pt tall.  Blocks: b0(100), b1(100, break-before=column),
/// b2(100), b3(100).  Without the forced break the balanced partition would be
/// col1 = {b0,b1}, col2 = {b2,b3} (target 200pt).  The mandatory column
/// boundary before b1 forces it into column 2, so the smallest feasible target
/// rises to 300pt and the partition becomes col1 = {b0}, col2 = {b1,b2,b3}.
#[test]
fn test_multicolumn_forced_column_break_honoured_under_balancing() {
    let doc = build_doc_columns(
        200.0,
        500.0,
        0.0,
        false,
        2,
        0.0,
        &[
            block(100.0),
            block_break_before(100.0, "column"),
            block(100.0),
            block(100.0),
        ],
    );
    let engine = LayoutEngine::new();
    let tree = engine.layout(&doc).expect("test: layout should succeed");

    let pages = page_ids(&tree);
    assert_eq!(pages.len(), 1, "4 blocks fit on one tall 2-column page");

    let blocks = blocks_of(&tree, region_of(&tree, pages[0]));
    assert_eq!(blocks.len(), 4, "all 4 blocks placed on the page");

    // b0 alone in the left column.
    assert!(
        block_x_pt(&tree, blocks[0]).abs() < 0.01,
        "b0 in left column"
    );
    assert!(
        block_y_pt(&tree, blocks[0]).abs() < 0.01,
        "b0 at column top"
    );

    // The forced column break moves b1 to the right column even though the
    // left column still had room — b1, b2, b3 stack in column 2.
    for (i, idx) in [1usize, 2, 3].iter().enumerate() {
        assert!(
            (block_x_pt(&tree, blocks[*idx]) - 100.0).abs() < 0.01,
            "b{} in the right column (forced break honoured)",
            idx
        );
        assert!(
            (block_y_pt(&tree, blocks[*idx]) - (i as f64) * 100.0).abs() < 0.01,
            "b{} stacks at y = {}pt in the right column",
            idx,
            (i as f64) * 100.0
        );
    }
}

// -----------------------------------------------------------------------
// Page-accurate fo:marker / fo:retrieve-marker (running headers)
// -----------------------------------------------------------------------

use fop_core::tree::RetrievePosition;

/// Build a paginated document whose region-before header retrieves the
/// `sec` marker, and whose flow stacks one block per `block_markers` entry —
/// each `Some(text)` block carrying an `fo:marker marker-class-name="sec"`
/// whose content is that `text`, each `None` block carrying no marker.
///
/// * page margins are 0 and `before_extent_pt` sizes the header region, so
///   the body height is `page_h_pt - before_extent_pt`; with `block_height_pt`
///   chosen against it the test controls how many marker blocks land per page.
/// * the header's `fo:retrieve-marker` is a direct child of the
///   `fo:static-content` (the form the engine resolves) using
///   `retrieve_position` and `retrieve_boundary`.
#[allow(clippy::too_many_arguments)]
fn build_marker_doc(
    page_w_pt: f64,
    page_h_pt: f64,
    before_extent_pt: f64,
    retrieve_position: RetrievePosition,
    retrieve_boundary: &'static str,
    block_height_pt: f64,
    block_markers: &[Option<&str>],
) -> FoArena<'static> {
    let mut fo = FoArena::new();
    let root = fo.add_node(FoNode::new(FoNodeData::Root));

    let lms = fo.add_node(FoNode::new(FoNodeData::LayoutMasterSet));
    fo.append_child(root, lms).expect("test: append lms");

    let mut spm_props = PropertyList::new();
    spm_props.set(
        PropertyId::PageWidth,
        PropertyValue::Length(Length::from_pt(page_w_pt)),
    );
    spm_props.set(
        PropertyId::PageHeight,
        PropertyValue::Length(Length::from_pt(page_h_pt)),
    );
    for margin in [
        PropertyId::MarginTop,
        PropertyId::MarginBottom,
        PropertyId::MarginLeft,
        PropertyId::MarginRight,
    ] {
        spm_props.set(margin, PropertyValue::Length(Length::ZERO));
    }
    let spm = fo.add_node(FoNode::new(FoNodeData::SimplePageMaster {
        master_name: "pm".to_string(),
        properties: spm_props,
    }));
    fo.append_child(lms, spm).expect("test: append spm");

    let mut rb_props = PropertyList::new();
    rb_props.set(
        PropertyId::Extent,
        PropertyValue::Length(Length::from_pt(before_extent_pt)),
    );
    let rb = fo.add_node(FoNode::new(FoNodeData::RegionBefore {
        properties: rb_props,
    }));
    fo.append_child(spm, rb)
        .expect("test: append region-before");

    let body = fo.add_node(FoNode::new(FoNodeData::RegionBody {
        properties: PropertyList::new(),
    }));
    fo.append_child(spm, body)
        .expect("test: append region-body");

    let ps = fo.add_node(FoNode::new(FoNodeData::PageSequence {
        master_reference: "pm".to_string(),
        format: "1".to_string(),
        grouping_separator: None,
        grouping_size: None,
        properties: PropertyList::new(),
    }));
    fo.append_child(root, ps)
        .expect("test: append page-sequence");

    // Header: a static-content whose direct child is the retrieve-marker.
    let sc = fo.add_node(FoNode::new(FoNodeData::StaticContent {
        flow_name: "xsl-region-before".to_string(),
        properties: PropertyList::new(),
    }));
    fo.append_child(ps, sc)
        .expect("test: append static-content");
    let mut rm_props = PropertyList::new();
    rm_props.set(
        PropertyId::RetrieveBoundary,
        PropertyValue::String(Cow::Borrowed(retrieve_boundary)),
    );
    let rm = fo.add_node(FoNode::new(FoNodeData::RetrieveMarker {
        retrieve_class_name: "sec".to_string(),
        retrieve_position,
        properties: rm_props,
    }));
    fo.append_child(sc, rm)
        .expect("test: append retrieve-marker");

    // Flow: one block per entry, optionally carrying a `sec` marker.
    let flow = fo.add_node(FoNode::new(FoNodeData::Flow {
        flow_name: "xsl-region-body".to_string(),
        properties: PropertyList::new(),
    }));
    fo.append_child(ps, flow).expect("test: append flow");

    for marker_text in block_markers {
        let mut bp = PropertyList::new();
        bp.set(
            PropertyId::LineHeight,
            PropertyValue::Length(Length::from_pt(block_height_pt)),
        );
        let block = fo.add_node(FoNode::new(FoNodeData::Block { properties: bp }));
        fo.append_child(flow, block)
            .expect("test: append flow block");

        if let Some(text) = marker_text {
            let marker = fo.add_node(FoNode::new(FoNodeData::Marker {
                marker_class_name: "sec".to_string(),
                properties: PropertyList::new(),
            }));
            fo.append_child(block, marker).expect("test: append marker");
            let mut mbp = PropertyList::new();
            mbp.set(
                PropertyId::LineHeight,
                PropertyValue::Length(Length::from_pt(12.0)),
            );
            let marker_block = fo.add_node(FoNode::new(FoNodeData::Block { properties: mbp }));
            fo.append_child(marker, marker_block)
                .expect("test: append marker block");
            let marker_text_node = fo.add_node(FoNode::new(FoNodeData::Text(text.to_string())));
            fo.append_child(marker_block, marker_text_node)
                .expect("test: append marker text");
        }
    }

    fo
}

/// Concatenated, trimmed text rendered into a page's `Header` area (the
/// content the running header's retrieve-marker resolved to).
fn header_text(tree: &AreaTree, page_id: AreaId) -> String {
    let header = tree.children(page_id).into_iter().find(|&child| {
        tree.get(child)
            .map(|n| n.area.area_type == AreaType::Header)
            .unwrap_or(false)
    });
    let mut out = String::new();
    if let Some(header_id) = header {
        collect_area_text(tree, header_id, &mut out);
    }
    out.trim().to_string()
}

/// Append all `Text` content under `id` (depth-first) to `out`.
fn collect_area_text(tree: &AreaTree, id: AreaId, out: &mut String) {
    if let Some(node) = tree.get(id) {
        if let Some(text) = node.area.text_content() {
            out.push_str(text);
        }
        for child_id in tree.children(id) {
            collect_area_text(tree, child_id, out);
        }
    }
}

/// Each page's running header shows the marker that starts on *that* page —
/// not a single marker repeated on every page (the bug this fixes).
///
/// Geometry: page 200×300pt, region-before extent 40 ⇒ body 260pt; 100pt
/// marker blocks ⇒ 2 per page.  Blocks carry Alpha, Bravo, Charlie, Delta ⇒
/// page 1 = {Alpha, Bravo}, page 2 = {Charlie, Delta}.  With
/// `first-starting-within-page` page 1's header is Alpha and page 2's is
/// Charlie.
#[test]
fn test_marker_resolves_per_page_first_starting() {
    let doc = build_marker_doc(
        200.0,
        300.0,
        40.0,
        RetrievePosition::FirstStartingWithinPage,
        "page-sequence",
        100.0,
        &[Some("Alpha"), Some("Bravo"), Some("Charlie"), Some("Delta")],
    );
    let engine = LayoutEngine::new();
    let tree = engine.layout(&doc).expect("test: layout should succeed");

    let pages = page_ids(&tree);
    assert_eq!(
        pages.len(),
        2,
        "4×100pt marker blocks in a 260pt body ⇒ 2 pages"
    );

    let p1 = header_text(&tree, pages[0]);
    let p2 = header_text(&tree, pages[1]);
    assert!(
        p1.contains("Alpha") && !p1.contains("Charlie"),
        "page 1 header must show the marker starting on page 1 (Alpha), got {:?}",
        p1
    );
    assert!(
        p2.contains("Charlie") && !p2.contains("Alpha"),
        "page 2 header must show the marker starting on page 2 (Charlie), got {:?}",
        p2
    );
    assert_ne!(p1, p2, "the two pages must show different markers");
}

/// On a page with two markers of the class, `first-starting-within-page` and
/// `last-starting-within-page` select different markers.
///
/// Same geometry/flow as above: page 1 = {Alpha, Bravo}.  `first-starting`
/// yields Alpha; `last-starting` yields Bravo.
#[test]
fn test_marker_first_vs_last_starting_within_page() {
    let blocks = [Some("Alpha"), Some("Bravo"), Some("Charlie"), Some("Delta")];

    let first_doc = build_marker_doc(
        200.0,
        300.0,
        40.0,
        RetrievePosition::FirstStartingWithinPage,
        "page-sequence",
        100.0,
        &blocks,
    );
    let last_doc = build_marker_doc(
        200.0,
        300.0,
        40.0,
        RetrievePosition::LastStartingWithinPage,
        "page-sequence",
        100.0,
        &blocks,
    );
    let engine = LayoutEngine::new();
    let first_tree = engine
        .layout(&first_doc)
        .expect("test: layout should succeed");
    let last_tree = engine
        .layout(&last_doc)
        .expect("test: layout should succeed");

    let first_p1 = header_text(&first_tree, page_ids(&first_tree)[0]);
    let last_p1 = header_text(&last_tree, page_ids(&last_tree)[0]);

    assert!(
        first_p1.contains("Alpha") && !first_p1.contains("Bravo"),
        "first-starting must pick the first of two same-page markers (Alpha), got {:?}",
        first_p1
    );
    assert!(
        last_p1.contains("Bravo") && !last_p1.contains("Alpha"),
        "last-starting must pick the last of two same-page markers (Bravo), got {:?}",
        last_p1
    );
    assert_ne!(
        first_p1, last_p1,
        "first-starting and last-starting must differ on a 2-marker page"
    );
}

/// A page that sets no marker carries over the previous page's marker under
/// `last-ending-within-page` (the position that, like a running "current
/// section" header, shows the page's own marker when present and otherwise
/// the one still in effect).  `last-starting-within-page` leaves the
/// markerless page's header empty (starting positions do not carry over).
///
/// Geometry: page 200×180pt, region-before extent 40 ⇒ body 140pt; 100pt
/// blocks ⇒ one per page.  Blocks carry Alpha, (none), Bravo ⇒ page 1 sets
/// Alpha, page 2 sets nothing, page 3 sets Bravo.  `last-ending-within-page`
/// ⇒ page 1 Alpha, page 2 carries over Alpha, page 3 shows its own Bravo.
#[test]
fn test_marker_carryover_when_page_has_no_fresh_marker() {
    let flow = [Some("Alpha"), None, Some("Bravo")];

    let carry_doc = build_marker_doc(
        200.0,
        180.0,
        40.0,
        RetrievePosition::LastEndingWithinPage,
        "page-sequence",
        100.0,
        &flow,
    );
    let starting_doc = build_marker_doc(
        200.0,
        180.0,
        40.0,
        RetrievePosition::LastStartingWithinPage,
        "page-sequence",
        100.0,
        &flow,
    );
    let engine = LayoutEngine::new();
    let carry_tree = engine
        .layout(&carry_doc)
        .expect("test: layout should succeed");
    let starting_tree = engine
        .layout(&starting_doc)
        .expect("test: layout should succeed");

    let carry_pages = page_ids(&carry_tree);
    assert_eq!(
        carry_pages.len(),
        3,
        "3×100pt blocks in a 140pt body ⇒ 3 pages"
    );

    let p1 = header_text(&carry_tree, carry_pages[0]);
    let p2 = header_text(&carry_tree, carry_pages[1]);
    let p3 = header_text(&carry_tree, carry_pages[2]);
    assert!(
        p1.contains("Alpha") && !p1.contains("Bravo"),
        "page 1 header shows its own marker (Alpha), got {:?}",
        p1
    );
    assert!(
        p2.contains("Alpha") && !p2.contains("Bravo"),
        "page 2 sets no marker and must carry over Alpha, got {:?}",
        p2
    );
    assert!(
        p3.contains("Bravo") && !p3.contains("Alpha"),
        "page 3 sets and shows its own marker (Bravo), got {:?}",
        p3
    );

    // last-starting-within-page: page 2 has no qualifying marker ⇒ empty
    // (starting positions never carry over).
    let starting_pages = page_ids(&starting_tree);
    assert!(
        header_text(&starting_tree, starting_pages[1]).is_empty(),
        "last-starting must not carry over: page 2's header is empty, got {:?}",
        header_text(&starting_tree, starting_pages[1])
    );
}

// =====================================================================
// Conditional page-master selection (fo:page-sequence-master)
// =====================================================================
//
// These tests exercise the live per-page geometry path: a page-sequence
// whose `master-reference` names a `fo:page-sequence-master` selects a
// concrete `fo:simple-page-master` per page from its
// `repeatable-page-master-alternatives` / `conditional-page-master-reference`
// children.
//
// The masters here differ only in `page-width`, and the page count is
// controlled by explicit `break-before="page"` forced breaks (not by flow
// overflow), so pagination is deterministic regardless of the body height
// used for overflow — each page's resolved geometry is asserted via its
// top-level `Page` area width.

use fop_core::tree::{BlankOrNotBlank, OddOrEven, PagePosition};

/// One simple-page-master to register in the layout-master-set.
struct MasterSpec {
    name: &'static str,
    page_w_pt: f64,
    /// Right page margin in pt — used to absorb a page-width difference so
    /// two masters can share an identical body box while differing in their
    /// page rectangle (needed for the `last` same-body case).
    margin_right_pt: f64,
}

/// One conditional alternative inside the repeatable-page-master-alternatives.
struct AltSpec {
    master_reference: &'static str,
    page_position: PagePosition,
    odd_or_even: OddOrEven,
    blank_or_not_blank: BlankOrNotBlank,
}

fn add_simple_master(fo: &mut FoArena<'static>, lms: NodeId, spec: &MasterSpec) {
    let mut props = PropertyList::new();
    props.set(
        PropertyId::PageWidth,
        PropertyValue::Length(Length::from_pt(spec.page_w_pt)),
    );
    // A tall page so the short flows below never overflow — page count is
    // driven purely by the explicit forced breaks.
    props.set(
        PropertyId::PageHeight,
        PropertyValue::Length(Length::from_pt(2000.0)),
    );
    for margin in [
        PropertyId::MarginTop,
        PropertyId::MarginBottom,
        PropertyId::MarginLeft,
    ] {
        props.set(margin, PropertyValue::Length(Length::ZERO));
    }
    props.set(
        PropertyId::MarginRight,
        PropertyValue::Length(Length::from_pt(spec.margin_right_pt)),
    );
    let spm = fo.add_node(FoNode::new(FoNodeData::SimplePageMaster {
        master_name: spec.name.to_string(),
        properties: props,
    }));
    fo.append_child(lms, spm).expect("test: append spm");
    let body = fo.add_node(FoNode::new(FoNodeData::RegionBody {
        properties: PropertyList::new(),
    }));
    fo.append_child(spm, body)
        .expect("test: append region-body");
}

/// Build a document whose single page-sequence references a
/// `page-sequence-master` (named `"seqmaster"`) made of the supplied
/// conditional alternatives, with `n_pages` flow blocks each separated by a
/// forced page break (so exactly `n_pages` pages are produced).
fn build_conditional_doc(
    masters: &[MasterSpec],
    alternatives: &[AltSpec],
    n_pages: usize,
) -> FoArena<'static> {
    let mut fo = FoArena::new();
    let root = fo.add_node(FoNode::new(FoNodeData::Root));

    let lms = fo.add_node(FoNode::new(FoNodeData::LayoutMasterSet));
    fo.append_child(root, lms).expect("test: append lms");

    for m in masters {
        add_simple_master(&mut fo, lms, m);
    }

    // page-sequence-master with repeatable-page-master-alternatives.
    let psm = fo.add_node(FoNode::new(FoNodeData::PageSequenceMaster {
        master_name: "seqmaster".to_string(),
    }));
    fo.append_child(lms, psm).expect("test: append psm");
    let rpma = fo.add_node(FoNode::new(FoNodeData::RepeatablePageMasterAlternatives {
        maximum_repeats: None,
    }));
    fo.append_child(psm, rpma).expect("test: append rpma");
    for alt in alternatives {
        let cpmr = fo.add_node(FoNode::new(FoNodeData::ConditionalPageMasterReference {
            master_reference: alt.master_reference.to_string(),
            page_position: alt.page_position,
            odd_or_even: alt.odd_or_even,
            blank_or_not_blank: alt.blank_or_not_blank,
        }));
        fo.append_child(rpma, cpmr).expect("test: append cpmr");
    }

    // page-sequence referencing the page-sequence-master.
    let ps = fo.add_node(FoNode::new(FoNodeData::PageSequence {
        master_reference: "seqmaster".to_string(),
        format: "1".to_string(),
        grouping_separator: None,
        grouping_size: None,
        properties: PropertyList::new(),
    }));
    fo.append_child(root, ps)
        .expect("test: append page-sequence");

    let flow = fo.add_node(FoNode::new(FoNodeData::Flow {
        flow_name: "xsl-region-body".to_string(),
        properties: PropertyList::new(),
    }));
    fo.append_child(ps, flow).expect("test: append flow");

    // `n_pages` blocks; every block after the first forces a page break, so
    // the sequence produces exactly `n_pages` pages.
    for i in 0..n_pages {
        let mut bp = PropertyList::new();
        bp.set(
            PropertyId::LineHeight,
            PropertyValue::Length(Length::from_pt(20.0)),
        );
        if i > 0 {
            bp.set(
                PropertyId::BreakBefore,
                PropertyValue::String(Cow::Borrowed("page")),
            );
        }
        let b = fo.add_node(FoNode::new(FoNodeData::Block { properties: bp }));
        fo.append_child(flow, b).expect("test: append flow block");
    }

    fo
}

/// The width of a top-level `Page` area (its resolved page-width).
fn page_width(tree: &AreaTree, page_id: AreaId) -> Length {
    tree.get(page_id)
        .map(|n| n.area.geometry.width)
        .expect("test: page area must exist")
}

// ---------------------------------------------------------------------
// (a) first / rest alternatives: the first page gets a different master
//     (geometry) than the remaining pages.
// ---------------------------------------------------------------------
#[test]
fn conditional_first_rest_gives_first_page_distinct_geometry() {
    let doc = build_conditional_doc(
        &[
            MasterSpec {
                name: "first-pm",
                page_w_pt: 300.0,
                margin_right_pt: 0.0,
            },
            MasterSpec {
                name: "rest-pm",
                page_w_pt: 500.0,
                margin_right_pt: 0.0,
            },
        ],
        &[
            AltSpec {
                master_reference: "first-pm",
                page_position: PagePosition::First,
                odd_or_even: OddOrEven::Any,
                blank_or_not_blank: BlankOrNotBlank::Any,
            },
            AltSpec {
                master_reference: "rest-pm",
                page_position: PagePosition::Any,
                odd_or_even: OddOrEven::Any,
                blank_or_not_blank: BlankOrNotBlank::Any,
            },
        ],
        3,
    );

    let engine = LayoutEngine::new();
    let tree = engine.layout(&doc).expect("test: layout should succeed");
    let pages = page_ids(&tree);
    assert_eq!(pages.len(), 3, "expected 3 pages");

    assert_eq!(
        page_width(&tree, pages[0]),
        Length::from_pt(300.0),
        "first page must use first-pm (width 300pt)"
    );
    assert_eq!(
        page_width(&tree, pages[1]),
        Length::from_pt(500.0),
        "second page must use rest-pm (width 500pt)"
    );
    assert_eq!(
        page_width(&tree, pages[2]),
        Length::from_pt(500.0),
        "third page must use rest-pm (width 500pt)"
    );
}

// ---------------------------------------------------------------------
// (b) odd / even alternatives select by page parity across a multi-page
//     flow.  Pages 1,3 (odd) ⇒ odd-pm; pages 2,4 (even) ⇒ even-pm.
// ---------------------------------------------------------------------
#[test]
fn conditional_odd_even_selects_by_parity() {
    let doc = build_conditional_doc(
        &[
            MasterSpec {
                name: "odd-pm",
                page_w_pt: 310.0,
                margin_right_pt: 0.0,
            },
            MasterSpec {
                name: "even-pm",
                page_w_pt: 420.0,
                margin_right_pt: 0.0,
            },
        ],
        &[
            AltSpec {
                master_reference: "odd-pm",
                page_position: PagePosition::Any,
                odd_or_even: OddOrEven::Odd,
                blank_or_not_blank: BlankOrNotBlank::Any,
            },
            AltSpec {
                master_reference: "even-pm",
                page_position: PagePosition::Any,
                odd_or_even: OddOrEven::Even,
                blank_or_not_blank: BlankOrNotBlank::Any,
            },
        ],
        4,
    );

    let engine = LayoutEngine::new();
    let tree = engine.layout(&doc).expect("test: layout should succeed");
    let pages = page_ids(&tree);
    assert_eq!(pages.len(), 4, "expected 4 pages");

    // Absolute page numbers 1..=4: odd ⇒ 310, even ⇒ 420.
    assert_eq!(
        page_width(&tree, pages[0]),
        Length::from_pt(310.0),
        "page 1 (odd) ⇒ odd-pm"
    );
    assert_eq!(
        page_width(&tree, pages[1]),
        Length::from_pt(420.0),
        "page 2 (even) ⇒ even-pm"
    );
    assert_eq!(
        page_width(&tree, pages[2]),
        Length::from_pt(310.0),
        "page 3 (odd) ⇒ odd-pm"
    );
    assert_eq!(
        page_width(&tree, pages[3]),
        Length::from_pt(420.0),
        "page 4 (even) ⇒ even-pm"
    );
}

// ---------------------------------------------------------------------
// (c) last alternative selects the final page's master (same-body case).
//
//     The two masters share an IDENTICAL body box (x=0, y=0, w=400,
//     h=2000) yet differ in their page rectangle: `body-pm` is 400pt wide
//     with no right margin, while `last-pm` is 650pt wide with a 250pt right
//     margin that absorbs the extra width — so `body_w = page_w -
//     margin_right = 400` for both.  Because the body box is identical, the
//     forward pass (which paginates against the first page's body) stays
//     valid for the final page, and the `last`-conditional fix-up safely
//     adopts `last-pm`'s page geometry for the final page.
// ---------------------------------------------------------------------
#[test]
fn conditional_last_selects_final_page_master_same_body() {
    let doc = build_conditional_doc(
        &[
            MasterSpec {
                name: "body-pm",
                page_w_pt: 400.0,
                margin_right_pt: 0.0,
            },
            MasterSpec {
                name: "last-pm",
                page_w_pt: 650.0,
                margin_right_pt: 250.0,
            },
        ],
        &[
            AltSpec {
                master_reference: "last-pm",
                page_position: PagePosition::Last,
                odd_or_even: OddOrEven::Any,
                blank_or_not_blank: BlankOrNotBlank::Any,
            },
            AltSpec {
                master_reference: "body-pm",
                page_position: PagePosition::Any,
                odd_or_even: OddOrEven::Any,
                blank_or_not_blank: BlankOrNotBlank::Any,
            },
        ],
        3,
    );

    let engine = LayoutEngine::new();
    let tree = engine.layout(&doc).expect("test: layout should succeed");
    let pages = page_ids(&tree);
    assert_eq!(pages.len(), 3, "expected 3 pages");

    // Pages 1 and 2 use body-pm (page width 400pt); the LAST page uses
    // last-pm (page width 650pt) via the `last`-conditional fix-up.
    assert_eq!(
        page_width(&tree, pages[0]),
        Length::from_pt(400.0),
        "page 1 ⇒ body-pm (page width 400pt)"
    );
    assert_eq!(
        page_width(&tree, pages[1]),
        Length::from_pt(400.0),
        "page 2 ⇒ body-pm (page width 400pt)"
    );
    assert_eq!(
        page_width(&tree, pages[2]),
        Length::from_pt(650.0),
        "last page ⇒ last-pm (page width 650pt) via the last-conditional fix-up"
    );

    // The shared body box (width 400pt) is identical on every page — that is
    // exactly why the fix-up could safely adopt last-pm: the final page's
    // already-paginated body content remains valid.
    for (idx, page) in pages.iter().enumerate() {
        let region = region_of(&tree, *page);
        assert_eq!(
            tree.get(region)
                .map(|n| n.area.geometry.width)
                .expect("test: region must exist"),
            Length::from_pt(400.0),
            "page {} region-body width must be the shared 400pt body",
            idx + 1
        );
    }
}
