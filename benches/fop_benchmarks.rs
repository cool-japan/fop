//! Comprehensive performance benchmarks for Apache FOP Rust implementation
//!
//! Benchmarks cover:
//! - Parsing (small, medium, large documents)
//! - Layout (blocks, inlines, tables, lists, page breaking)
//! - Rendering (text, images, graphics, complete pipeline)
//! - Property system (parsing, inheritance, shorthand expansion)
//!
//! Target metrics:
//! - Parsing: <1ms per page
//! - Layout: <5ms per page
//! - Rendering: <2ms per page
//! - Total: <10ms per page for simple documents

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use fop_core::{Color, FoTreeBuilder, Length, PropertyId, PropertyList, PropertyValue};
use fop_layout::LayoutEngine;
use fop_render::PdfRenderer;
use std::borrow::Cow;
use std::hint::black_box;
use std::io::Cursor;

// ============================================================================
// Document Generators
// ============================================================================

/// Generate small FO document (1 page, ~100 elements)
fn generate_small_document() -> String {
    generate_fo_document(1, 20, false, false)
}

/// Generate medium FO document (10 pages, ~1000 elements)
fn generate_medium_document() -> String {
    generate_fo_document(10, 100, false, false)
}

/// Generate large FO document (100 pages, ~10000 elements)
fn generate_large_document() -> String {
    generate_fo_document(100, 100, false, false)
}

/// Generate document with tables
fn generate_table_document(rows: usize, cols: usize) -> String {
    let mut xml = String::from(
        r##"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
            <fo:region-body margin="1in"/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:table border="1pt solid black">
                <fo:table-body>
"##,
    );

    for i in 0..rows {
        xml.push_str("                    <fo:table-row>\n");
        for j in 0..cols {
            xml.push_str(&format!(
                r##"                        <fo:table-cell border="0.5pt solid gray">
                            <fo:block>Cell {},{}</fo:block>
                        </fo:table-cell>
"##,
                i, j
            ));
        }
        xml.push_str("                    </fo:table-row>\n");
    }

    xml.push_str(
        r##"                </fo:table-body>
            </fo:table>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"##,
    );

    xml
}

/// Generate document with lists
fn generate_list_document(num_items: usize) -> String {
    let mut xml = String::from(
        r##"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
            <fo:region-body margin="1in"/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:list-block>
"##,
    );

    for i in 0..num_items {
        xml.push_str(&format!(
            r##"                <fo:list-item>
                    <fo:list-item-label><fo:block>•</fo:block></fo:list-item-label>
                    <fo:list-item-body><fo:block>List item {}</fo:block></fo:list-item-body>
                </fo:list-item>
"##,
            i
        ));
    }

    xml.push_str(
        r##"            </fo:list-block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"##,
    );

    xml
}

/// Generate generic FO document with configurable complexity
fn generate_fo_document(
    num_pages: usize,
    blocks_per_page: usize,
    with_inline: bool,
    with_styles: bool,
) -> String {
    let mut xml = String::from(
        r##"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
            <fo:region-body margin="1in"/>
        </fo:simple-page-master>
    </fo:layout-master-set>
"##,
    );

    for page_num in 0..num_pages {
        xml.push_str("    <fo:page-sequence master-reference=\"A4\">\n");
        xml.push_str("        <fo:flow flow-name=\"xsl-region-body\">\n");

        for i in 0..blocks_per_page {
            let style_attrs = if with_styles {
                r##" font-size="12pt" color="#333333" space-after="6pt" padding="3pt" background-color="#F0F0F0""##
            } else {
                ""
            };

            xml.push_str(&format!("            <fo:block{}>\n", style_attrs));

            if with_inline {
                xml.push_str(&format!(
                    "                Page {} Block {}: This is <fo:inline font-weight=\"bold\">inline text</fo:inline> with some content.\n",
                    page_num, i
                ));
            } else {
                xml.push_str(&format!(
                    "                Page {} Block {}: This is regular text with some content for benchmarking.\n",
                    page_num, i
                ));
            }

            xml.push_str("            </fo:block>\n");
        }

        xml.push_str("        </fo:flow>\n");
        xml.push_str("    </fo:page-sequence>\n");
    }

    xml.push_str("</fo:root>");
    xml
}

// ============================================================================
// Parsing Benchmarks
// ============================================================================

fn bench_parsing_small(c: &mut Criterion) {
    let xml = generate_small_document();
    c.bench_function("parse/small_1page_100elements", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(xml.as_bytes()));
            let builder = FoTreeBuilder::new();
            builder
                .parse(cursor)
                .expect("bench/example: should succeed")
        });
    });
}

fn bench_parsing_medium(c: &mut Criterion) {
    let xml = generate_medium_document();
    c.bench_function("parse/medium_10pages_1000elements", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(xml.as_bytes()));
            let builder = FoTreeBuilder::new();
            builder
                .parse(cursor)
                .expect("bench/example: should succeed")
        });
    });
}

fn bench_parsing_large(c: &mut Criterion) {
    let xml = generate_large_document();
    c.bench_function("parse/large_100pages_10000elements", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(xml.as_bytes()));
            let builder = FoTreeBuilder::new();
            builder
                .parse(cursor)
                .expect("bench/example: should succeed")
        });
    });
}

fn bench_parsing_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse/scaling");

    for num_blocks in [10, 50, 100, 500, 1000].iter() {
        let xml = generate_fo_document(1, *num_blocks, false, false);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_blocks", num_blocks)),
            &xml,
            |b, xml| {
                b.iter(|| {
                    let cursor = Cursor::new(black_box(xml.as_bytes()));
                    let builder = FoTreeBuilder::new();
                    builder
                        .parse(cursor)
                        .expect("bench/example: should succeed")
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Layout Benchmarks
// ============================================================================

fn bench_layout_blocks(c: &mut Criterion) {
    let xml = generate_fo_document(1, 100, false, true);
    let cursor = Cursor::new(xml.as_bytes());
    let builder = FoTreeBuilder::new();
    let arena = builder
        .parse(cursor)
        .expect("bench/example: should succeed");

    c.bench_function("layout/blocks_100", |b| {
        b.iter(|| {
            let engine = LayoutEngine::new();
            engine
                .layout(black_box(&arena))
                .expect("bench/example: should succeed")
        });
    });
}

fn bench_layout_inline(c: &mut Criterion) {
    let xml = generate_fo_document(1, 50, true, true);
    let cursor = Cursor::new(xml.as_bytes());
    let builder = FoTreeBuilder::new();
    let arena = builder
        .parse(cursor)
        .expect("bench/example: should succeed");

    c.bench_function("layout/inline_with_breaking", |b| {
        b.iter(|| {
            let engine = LayoutEngine::new();
            engine
                .layout(black_box(&arena))
                .expect("bench/example: should succeed")
        });
    });
}

fn bench_layout_tables(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout/tables");

    for (rows, cols) in [(5, 3), (10, 5), (20, 10)].iter() {
        let xml = generate_table_document(*rows, *cols);
        let cursor = Cursor::new(xml.as_bytes());
        let builder = FoTreeBuilder::new();
        let arena = builder
            .parse(cursor)
            .expect("bench/example: should succeed");

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}x{}", rows, cols)),
            &arena,
            |b, arena| {
                b.iter(|| {
                    let engine = LayoutEngine::new();
                    engine
                        .layout(black_box(arena))
                        .expect("bench/example: should succeed")
                });
            },
        );
    }

    group.finish();
}

fn bench_layout_lists(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout/lists");

    for num_items in [10, 50, 100].iter() {
        let xml = generate_list_document(*num_items);
        let cursor = Cursor::new(xml.as_bytes());
        let builder = FoTreeBuilder::new();
        let arena = builder
            .parse(cursor)
            .expect("bench/example: should succeed");

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_items", num_items)),
            &arena,
            |b, arena| {
                b.iter(|| {
                    let engine = LayoutEngine::new();
                    engine
                        .layout(black_box(arena))
                        .expect("bench/example: should succeed")
                });
            },
        );
    }

    group.finish();
}

fn bench_layout_multipage(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout/multipage");

    for num_pages in [1, 5, 10].iter() {
        let xml = generate_fo_document(*num_pages, 50, false, true);
        let cursor = Cursor::new(xml.as_bytes());
        let builder = FoTreeBuilder::new();
        let arena = builder
            .parse(cursor)
            .expect("bench/example: should succeed");

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_pages", num_pages)),
            &arena,
            |b, arena| {
                b.iter(|| {
                    let engine = LayoutEngine::new();
                    engine
                        .layout(black_box(arena))
                        .expect("bench/example: should succeed")
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Rendering Benchmarks
// ============================================================================

fn bench_render_text(c: &mut Criterion) {
    let xml = generate_fo_document(1, 100, false, false);
    let cursor = Cursor::new(xml.as_bytes());
    let builder = FoTreeBuilder::new();
    let arena = builder
        .parse(cursor)
        .expect("bench/example: should succeed");
    let engine = LayoutEngine::new();
    let area_tree = engine
        .layout(&arena)
        .expect("bench/example: should succeed");

    c.bench_function("render/text_100blocks", |b| {
        b.iter(|| {
            let renderer = PdfRenderer::new();
            renderer
                .render(black_box(&area_tree))
                .expect("bench/example: should succeed")
        });
    });
}

fn bench_render_styled(c: &mut Criterion) {
    let xml = generate_fo_document(1, 100, false, true);
    let cursor = Cursor::new(xml.as_bytes());
    let builder = FoTreeBuilder::new();
    let arena = builder
        .parse(cursor)
        .expect("bench/example: should succeed");
    let engine = LayoutEngine::new();
    let area_tree = engine
        .layout(&arena)
        .expect("bench/example: should succeed");

    c.bench_function("render/styled_text_graphics", |b| {
        b.iter(|| {
            let renderer = PdfRenderer::new();
            renderer
                .render(black_box(&area_tree))
                .expect("bench/example: should succeed")
        });
    });
}

fn bench_render_complete_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("render/complete_pipeline");

    for num_pages in [1, 5, 10].iter() {
        let xml = generate_fo_document(*num_pages, 50, false, true);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_pages", num_pages)),
            &xml,
            |b, xml| {
                b.iter(|| {
                    // Parse
                    let cursor = Cursor::new(black_box(xml.as_bytes()));
                    let builder = FoTreeBuilder::new();
                    let arena = builder
                        .parse(cursor)
                        .expect("bench/example: should succeed");

                    // Layout
                    let engine = LayoutEngine::new();
                    let area_tree = engine
                        .layout(&arena)
                        .expect("bench/example: should succeed");

                    // Render
                    let renderer = PdfRenderer::new();
                    let pdf_doc = renderer
                        .render(&area_tree)
                        .expect("bench/example: should succeed");

                    // Serialize
                    pdf_doc.to_bytes().expect("bench/example: should succeed")
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Property System Benchmarks
// ============================================================================

fn bench_property_parsing(c: &mut Criterion) {
    c.bench_function("property/parse_length", |b| {
        b.iter(|| {
            let _ = black_box(Length::from_pt(12.0));
            let _ = black_box(Length::from_mm(25.4));
            let _ = black_box(Length::from_pt(72.0));
        });
    });
}

fn bench_property_access(c: &mut Criterion) {
    let mut props = PropertyList::new();
    props.set(
        PropertyId::FontSize,
        PropertyValue::Length(Length::from_pt(12.0)),
    );
    props.set(PropertyId::Color, PropertyValue::Color(Color::BLACK));
    props.set(
        PropertyId::MarginTop,
        PropertyValue::Length(Length::from_pt(10.0)),
    );

    c.bench_function("property/access_get", |b| {
        b.iter(|| {
            let _ = black_box(props.get(PropertyId::FontSize));
            let _ = black_box(props.get(PropertyId::Color));
            let _ = black_box(props.get(PropertyId::MarginTop));
        });
    });
}

fn bench_property_inheritance(c: &mut Criterion) {
    // Create parent property list
    let mut parent = PropertyList::new();
    parent.set(
        PropertyId::FontSize,
        PropertyValue::Length(Length::from_pt(12.0)),
    );
    parent.set(PropertyId::Color, PropertyValue::Color(Color::BLACK));

    // Create child that inherits
    let mut child = PropertyList::new();
    child.set(
        PropertyId::FontWeight,
        PropertyValue::String(Cow::Borrowed("bold")),
    );

    c.bench_function("property/inheritance_lookup", |b| {
        b.iter(|| {
            // Simulate inheritance lookup
            let _ = black_box(child.get(PropertyId::FontWeight));
            let _ = black_box(parent.get(PropertyId::FontSize));
            let _ = black_box(parent.get(PropertyId::Color));
        });
    });
}

fn bench_length_conversions(c: &mut Criterion) {
    c.bench_function("property/length_pt_to_mm", |b| {
        b.iter(|| {
            let len = Length::from_pt(black_box(72.0));
            black_box(len.to_mm())
        });
    });

    c.bench_function("property/length_arithmetic", |b| {
        b.iter(|| {
            let a = Length::from_pt(black_box(10.0));
            let b = Length::from_pt(black_box(20.0));
            black_box(a + b)
        });
    });
}

// ============================================================================
// Benchmark Groups
// ============================================================================

criterion_group!(
    parsing_benches,
    bench_parsing_small,
    bench_parsing_medium,
    bench_parsing_large,
    bench_parsing_scaling
);

criterion_group!(
    layout_benches,
    bench_layout_blocks,
    bench_layout_inline,
    bench_layout_tables,
    bench_layout_lists,
    bench_layout_multipage
);

criterion_group!(
    rendering_benches,
    bench_render_text,
    bench_render_styled,
    bench_render_complete_pipeline
);

criterion_group!(
    property_benches,
    bench_property_parsing,
    bench_property_access,
    bench_property_inheritance,
    bench_length_conversions
);

criterion_main!(
    parsing_benches,
    layout_benches,
    rendering_benches,
    property_benches
);
