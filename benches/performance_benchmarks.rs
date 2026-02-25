//! Comprehensive Performance Benchmarking Suite for Apache FOP Rust
//!
//! This suite provides detailed performance metrics across all major subsystems:
//!
//! 1. **Parsing Speed** - Document parsing across sizes (small/medium/large)
//! 2. **Layout Engine** - Block layout, inline layout, table layout, list layout, page breaking
//! 3. **PDF Rendering** - Text rendering, graphics, images, complete pipeline
//! 4. **SVG Rendering** - SVG output generation and serialization
//! 5. **Encryption Overhead** - PDF encryption impact on performance
//! 6. **Memory Usage** - Memory consumption patterns and allocation benchmarks
//!
//! All benchmarks are reproducible and use the Criterion framework for statistical analysis.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::{PdfPermissions, PdfRenderer, PdfSecurity, SvgRenderer};
use std::hint::black_box;
use std::io::Cursor;

// ============================================================================
// Document Generators - Reproducible test documents
// ============================================================================

/// Generate a small FO document (1 page, ~100 elements, ~2KB)
fn generate_small_document() -> String {
    generate_fo_document(1, 20, false, false, false)
}

/// Generate a medium FO document (10 pages, ~1000 elements, ~50KB)
fn generate_medium_document() -> String {
    generate_fo_document(10, 100, true, true, false)
}

/// Generate a large FO document (100 pages, ~10000 elements, ~500KB)
fn generate_large_document() -> String {
    generate_fo_document(100, 100, true, true, false)
}

/// Generate very large document (500 pages, ~50000 elements, ~2.5MB)
fn generate_very_large_document() -> String {
    generate_fo_document(500, 100, true, true, false)
}

/// Generate document with complex styling and graphics
fn generate_complex_styled_document() -> String {
    generate_fo_document(5, 50, true, true, true)
}

/// Generate document with embedded images (PNG/JPEG)
#[allow(dead_code)]
fn generate_image_document(num_images: usize) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
            <fo:region-body margin="1in"/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
"#,
    );

    for i in 0..num_images {
        xml.push_str(&format!(
            r#"            <fo:block space-after="12pt">
                <fo:block>Image {}</fo:block>
                <fo:external-graphic src="test-image-{}.png" content-width="100mm" content-height="50mm"/>
            </fo:block>
"#,
            i, i
        ));
    }

    xml.push_str(
        r#"        </fo:flow>
    </fo:page-sequence>
</fo:root>"#,
    );

    xml
}

/// Generate document with complex table layout
fn generate_complex_table_document(rows: usize, cols: usize) -> String {
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
            <fo:table border="1pt solid black" table-layout="fixed" width="100%">
                <fo:table-header background-color="#CCCCCC">
                    <fo:table-row>
"##,
    );

    // Header row
    for j in 0..cols {
        xml.push_str(&format!(
            r##"                        <fo:table-cell border="0.5pt solid gray" padding="3pt">
                            <fo:block font-weight="bold">Header {}</fo:block>
                        </fo:table-cell>
"##,
            j
        ));
    }
    xml.push_str("                    </fo:table-row>\n");
    xml.push_str("                </fo:table-header>\n");
    xml.push_str("                <fo:table-body>\n");

    // Data rows
    for i in 0..rows {
        xml.push_str("                    <fo:table-row>\n");
        for j in 0..cols {
            let bg_color = if i % 2 == 0 {
                r##" background-color="#F5F5F5""##
            } else {
                ""
            };
            xml.push_str(&format!(
                r##"                        <fo:table-cell border="0.5pt solid gray" padding="3pt"{}>
                            <fo:block>Cell ({},{})</fo:block>
                        </fo:table-cell>
"##,
                bg_color, i, j
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

/// Generate document with nested lists
fn generate_nested_list_document(depth: usize, items_per_level: usize) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
            <fo:region-body margin="1in"/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
"#,
    );

    fn add_list_level(xml: &mut String, current_depth: usize, max_depth: usize, items: usize) {
        let indent = "    ".repeat(current_depth + 3);
        xml.push_str(&format!("{}            <fo:list-block>\n", indent));

        for i in 0..items {
            xml.push_str(&format!("{}                <fo:list-item>\n", indent));
            xml.push_str(&format!(
                "{}                    <fo:list-item-label><fo:block>•</fo:block></fo:list-item-label>\n",
                indent
            ));
            xml.push_str(&format!(
                "{}                    <fo:list-item-body start-indent=\"body-start()\">\n",
                indent
            ));
            xml.push_str(&format!(
                "{}                        <fo:block>Level {} Item {}</fo:block>\n",
                indent, current_depth, i
            ));

            if current_depth < max_depth {
                add_list_level(xml, current_depth + 1, max_depth, items);
            }

            xml.push_str(&format!(
                "{}                    </fo:list-item-body>\n",
                indent
            ));
            xml.push_str(&format!("{}                </fo:list-item>\n", indent));
        }

        xml.push_str(&format!("{}            </fo:list-block>\n", indent));
    }

    add_list_level(&mut xml, 1, depth, items_per_level);

    xml.push_str(
        r#"        </fo:flow>
    </fo:page-sequence>
</fo:root>"#,
    );

    xml
}

/// Generate generic FO document with configurable complexity
#[allow(clippy::too_many_arguments)]
fn generate_fo_document(
    num_pages: usize,
    blocks_per_page: usize,
    with_inline: bool,
    with_styles: bool,
    with_graphics: bool,
) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
            <fo:region-body margin="1in"/>
        </fo:simple-page-master>
    </fo:layout-master-set>
"#,
    );

    for page_num in 0..num_pages {
        xml.push_str("    <fo:page-sequence master-reference=\"A4\">\n");
        xml.push_str("        <fo:flow flow-name=\"xsl-region-body\">\n");

        for i in 0..blocks_per_page {
            let style_attrs = if with_styles {
                let colors = ["#333333", "#666666", "#999999", "#000000"];
                let sizes = ["10pt", "11pt", "12pt", "14pt"];
                let color = colors[i % colors.len()];
                let size = sizes[i % sizes.len()];

                format!(
                    r##" font-size="{}" color="{}" space-after="6pt" padding="3pt" background-color="#F8F8F8" border="0.5pt solid #DDDDDD""##,
                    size, color
                )
            } else {
                String::new()
            };

            xml.push_str(&format!("            <fo:block{}>\n", style_attrs));

            if with_inline {
                xml.push_str(&format!(
                    "                Page {} Block {}: This text contains <fo:inline font-weight=\"bold\">bold</fo:inline>, <fo:inline font-style=\"italic\">italic</fo:inline>, and <fo:inline text-decoration=\"underline\">underlined</fo:inline> content.\n",
                    page_num, i
                ));
            } else {
                xml.push_str(&format!(
                    "                Page {} Block {}: Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore.\n",
                    page_num, i
                ));
            }

            // Add graphics elements if requested
            if with_graphics && i % 5 == 0 {
                xml.push_str(
                    r##"                <fo:block space-before="6pt" space-after="6pt">
                    <fo:leader leader-pattern="rule" leader-length="100%" rule-thickness="1pt" color="#CCCCCC"/>
                </fo:block>
"##,
                );
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
// 1. PARSING SPEED BENCHMARKS
// ============================================================================

fn bench_parsing_small(c: &mut Criterion) {
    let xml = generate_small_document();
    let size = xml.len();

    let mut group = c.benchmark_group("parsing/small_document");
    group.throughput(Throughput::Bytes(size as u64));

    group.bench_function("1_page_100_elements", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(xml.as_bytes()));
            let builder = FoTreeBuilder::new();
            builder
                .parse(cursor)
                .expect("bench/example: should succeed")
        });
    });

    group.finish();
}

fn bench_parsing_medium(c: &mut Criterion) {
    let xml = generate_medium_document();
    let size = xml.len();

    let mut group = c.benchmark_group("parsing/medium_document");
    group.throughput(Throughput::Bytes(size as u64));

    group.bench_function("10_pages_1000_elements", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(xml.as_bytes()));
            let builder = FoTreeBuilder::new();
            builder
                .parse(cursor)
                .expect("bench/example: should succeed")
        });
    });

    group.finish();
}

fn bench_parsing_large(c: &mut Criterion) {
    let xml = generate_large_document();
    let size = xml.len();

    let mut group = c.benchmark_group("parsing/large_document");
    group.throughput(Throughput::Bytes(size as u64));
    group.sample_size(20);

    group.bench_function("100_pages_10000_elements", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(xml.as_bytes()));
            let builder = FoTreeBuilder::new();
            builder
                .parse(cursor)
                .expect("bench/example: should succeed")
        });
    });

    group.finish();
}

fn bench_parsing_very_large(c: &mut Criterion) {
    let xml = generate_very_large_document();
    let size = xml.len();

    let mut group = c.benchmark_group("parsing/very_large_document");
    group.throughput(Throughput::Bytes(size as u64));
    group.sample_size(10);

    group.bench_function("500_pages_50000_elements", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(xml.as_bytes()));
            let builder = FoTreeBuilder::new();
            builder
                .parse(cursor)
                .expect("bench/example: should succeed")
        });
    });

    group.finish();
}

fn bench_parsing_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing/scaling_by_elements");

    for num_blocks in [10, 50, 100, 200, 500, 1000, 2000].iter() {
        let xml = generate_fo_document(1, *num_blocks, false, false, false);
        let size = xml.len();

        group.throughput(Throughput::Bytes(size as u64));
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
// 2. LAYOUT ENGINE BENCHMARKS
// ============================================================================

fn bench_layout_simple_blocks(c: &mut Criterion) {
    let xml = generate_fo_document(1, 100, false, false, false);
    let cursor = Cursor::new(xml.as_bytes());
    let builder = FoTreeBuilder::new();
    let arena = builder
        .parse(cursor)
        .expect("bench/example: should succeed");

    c.bench_function("layout/simple_blocks_100", |b| {
        b.iter(|| {
            let engine = LayoutEngine::new();
            engine
                .layout(black_box(&arena))
                .expect("bench/example: should succeed")
        });
    });
}

fn bench_layout_complex_blocks(c: &mut Criterion) {
    let xml = generate_fo_document(1, 100, true, true, true);
    let cursor = Cursor::new(xml.as_bytes());
    let builder = FoTreeBuilder::new();
    let arena = builder
        .parse(cursor)
        .expect("bench/example: should succeed");

    c.bench_function("layout/complex_styled_blocks_100", |b| {
        b.iter(|| {
            let engine = LayoutEngine::new();
            engine
                .layout(black_box(&arena))
                .expect("bench/example: should succeed")
        });
    });
}

fn bench_layout_inline_with_breaking(c: &mut Criterion) {
    let xml = generate_fo_document(1, 50, true, true, false);
    let cursor = Cursor::new(xml.as_bytes());
    let builder = FoTreeBuilder::new();
    let arena = builder
        .parse(cursor)
        .expect("bench/example: should succeed");

    c.bench_function("layout/inline_text_with_breaking", |b| {
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

    for (rows, cols) in [(5, 3), (10, 5), (20, 10), (50, 10), (100, 5)].iter() {
        let xml = generate_complex_table_document(*rows, *cols);
        let cursor = Cursor::new(xml.as_bytes());
        let builder = FoTreeBuilder::new();
        let arena = builder
            .parse(cursor)
            .expect("bench/example: should succeed");

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}x{}_cells", rows, cols)),
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

fn bench_layout_nested_lists(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout/nested_lists");

    for (depth, items) in [(2, 5), (3, 3), (4, 2)].iter() {
        let xml = generate_nested_list_document(*depth, *items);
        let cursor = Cursor::new(xml.as_bytes());
        let builder = FoTreeBuilder::new();
        let arena = builder
            .parse(cursor)
            .expect("bench/example: should succeed");

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("depth{}_items{}", depth, items)),
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

fn bench_layout_page_breaking(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout/page_breaking");

    for num_pages in [1, 5, 10, 20, 50].iter() {
        let xml = generate_fo_document(*num_pages, 50, false, true, false);
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
// 3. PDF RENDERING BENCHMARKS
// ============================================================================

fn bench_pdf_render_text_only(c: &mut Criterion) {
    let xml = generate_fo_document(1, 100, false, false, false);
    let cursor = Cursor::new(xml.as_bytes());
    let builder = FoTreeBuilder::new();
    let arena = builder
        .parse(cursor)
        .expect("bench/example: should succeed");
    let engine = LayoutEngine::new();
    let area_tree = engine
        .layout(&arena)
        .expect("bench/example: should succeed");

    c.bench_function("pdf_rendering/text_only_100_blocks", |b| {
        b.iter(|| {
            let renderer = PdfRenderer::new();
            renderer
                .render(black_box(&area_tree))
                .expect("bench/example: should succeed")
        });
    });
}

fn bench_pdf_render_styled_graphics(c: &mut Criterion) {
    let xml = generate_complex_styled_document();
    let cursor = Cursor::new(xml.as_bytes());
    let builder = FoTreeBuilder::new();
    let arena = builder
        .parse(cursor)
        .expect("bench/example: should succeed");
    let engine = LayoutEngine::new();
    let area_tree = engine
        .layout(&arena)
        .expect("bench/example: should succeed");

    c.bench_function("pdf_rendering/styled_with_graphics", |b| {
        b.iter(|| {
            let renderer = PdfRenderer::new();
            renderer
                .render(black_box(&area_tree))
                .expect("bench/example: should succeed")
        });
    });
}

fn bench_pdf_render_tables(c: &mut Criterion) {
    let xml = generate_complex_table_document(20, 5);
    let cursor = Cursor::new(xml.as_bytes());
    let builder = FoTreeBuilder::new();
    let arena = builder
        .parse(cursor)
        .expect("bench/example: should succeed");
    let engine = LayoutEngine::new();
    let area_tree = engine
        .layout(&arena)
        .expect("bench/example: should succeed");

    c.bench_function("pdf_rendering/table_20x5", |b| {
        b.iter(|| {
            let renderer = PdfRenderer::new();
            renderer
                .render(black_box(&area_tree))
                .expect("bench/example: should succeed")
        });
    });
}

fn bench_pdf_complete_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("pdf_rendering/complete_pipeline");

    for num_pages in [1, 5, 10, 20].iter() {
        let xml = generate_fo_document(*num_pages, 50, true, true, true);
        let size = xml.len();

        group.throughput(Throughput::Bytes(size as u64));
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

                    // Render to PDF
                    let renderer = PdfRenderer::new();
                    let pdf_doc = renderer
                        .render(&area_tree)
                        .expect("bench/example: should succeed");

                    // Serialize to bytes
                    pdf_doc.to_bytes().expect("bench/example: should succeed")
                });
            },
        );
    }

    group.finish();
}

fn bench_pdf_serialization(c: &mut Criterion) {
    let xml = generate_medium_document();
    let cursor = Cursor::new(xml.as_bytes());
    let builder = FoTreeBuilder::new();
    let arena = builder
        .parse(cursor)
        .expect("bench/example: should succeed");
    let engine = LayoutEngine::new();
    let area_tree = engine
        .layout(&arena)
        .expect("bench/example: should succeed");
    let renderer = PdfRenderer::new();
    let pdf_doc = renderer
        .render(&area_tree)
        .expect("bench/example: should succeed");

    c.bench_function("pdf_rendering/serialization_to_bytes", |b| {
        b.iter(|| {
            black_box(&pdf_doc)
                .to_bytes()
                .expect("bench/example: should succeed")
        });
    });
}

// ============================================================================
// 4. SVG RENDERING BENCHMARKS
// ============================================================================

fn bench_svg_render_simple(c: &mut Criterion) {
    let xml = generate_fo_document(1, 50, false, false, false);
    let cursor = Cursor::new(xml.as_bytes());
    let builder = FoTreeBuilder::new();
    let arena = builder
        .parse(cursor)
        .expect("bench/example: should succeed");
    let engine = LayoutEngine::new();
    let area_tree = engine
        .layout(&arena)
        .expect("bench/example: should succeed");

    c.bench_function("svg_rendering/simple_document", |b| {
        b.iter(|| {
            let renderer = SvgRenderer::new();
            renderer
                .render_to_svg(black_box(&area_tree))
                .expect("bench/example: should succeed")
        });
    });
}

fn bench_svg_render_styled(c: &mut Criterion) {
    let xml = generate_complex_styled_document();
    let cursor = Cursor::new(xml.as_bytes());
    let builder = FoTreeBuilder::new();
    let arena = builder
        .parse(cursor)
        .expect("bench/example: should succeed");
    let engine = LayoutEngine::new();
    let area_tree = engine
        .layout(&arena)
        .expect("bench/example: should succeed");

    c.bench_function("svg_rendering/styled_graphics", |b| {
        b.iter(|| {
            let renderer = SvgRenderer::new();
            renderer
                .render_to_svg(black_box(&area_tree))
                .expect("bench/example: should succeed")
        });
    });
}

fn bench_svg_render_multipage(c: &mut Criterion) {
    let mut group = c.benchmark_group("svg_rendering/multipage");

    for num_pages in [1, 5, 10].iter() {
        let xml = generate_fo_document(*num_pages, 30, true, true, false);
        let cursor = Cursor::new(xml.as_bytes());
        let builder = FoTreeBuilder::new();
        let arena = builder
            .parse(cursor)
            .expect("bench/example: should succeed");
        let engine = LayoutEngine::new();
        let area_tree = engine
            .layout(&arena)
            .expect("bench/example: should succeed");

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_pages", num_pages)),
            &area_tree,
            |b, area_tree| {
                b.iter(|| {
                    let renderer = SvgRenderer::new();
                    renderer
                        .render_to_svg(black_box(area_tree))
                        .expect("bench/example: should succeed")
                });
            },
        );
    }

    group.finish();
}

fn bench_svg_complete_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("svg_rendering/complete_pipeline");

    for num_pages in [1, 5, 10].iter() {
        let xml = generate_fo_document(*num_pages, 40, true, true, false);

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

                    // Render to SVG
                    let renderer = SvgRenderer::new();
                    renderer
                        .render_to_svg(&area_tree)
                        .expect("bench/example: should succeed")
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// 5. ENCRYPTION OVERHEAD BENCHMARKS
// ============================================================================

fn bench_encryption_setup(c: &mut Criterion) {
    let permissions = PdfPermissions::default();

    c.bench_function("encryption/setup_encryption_dict", |b| {
        b.iter(|| {
            let security = PdfSecurity::new(
                black_box("owner"),
                black_box("user"),
                black_box(permissions),
            );
            let file_id = fop_render::pdf::security::generate_file_id("test-doc");
            security.compute_encryption_dict(&file_id)
        });
    });
}

fn bench_encryption_encrypt_data(c: &mut Criterion) {
    let permissions = PdfPermissions::default();
    let security = PdfSecurity::new("owner", "user", permissions);
    let file_id = fop_render::pdf::security::generate_file_id("test-doc");
    let encryption_dict = security.compute_encryption_dict(&file_id);

    let test_data = b"This is a sample PDF stream content that needs to be encrypted. It contains multiple sentences and should be representative of typical PDF content.";

    let mut group = c.benchmark_group("encryption/encrypt_data");
    group.throughput(Throughput::Bytes(test_data.len() as u64));

    group.bench_function("small_stream_150bytes", |b| {
        b.iter(|| encryption_dict.encrypt_data(black_box(test_data), black_box(5), black_box(0)));
    });

    group.finish();
}

fn bench_encryption_encrypt_large_data(c: &mut Criterion) {
    let permissions = PdfPermissions::default();
    let security = PdfSecurity::new("owner", "user", permissions);
    let file_id = fop_render::pdf::security::generate_file_id("test-doc");
    let encryption_dict = security.compute_encryption_dict(&file_id);

    let mut group = c.benchmark_group("encryption/encrypt_large_data");

    for size in [1024, 10240, 102400, 1024000].iter() {
        let data = vec![b'A'; *size];
        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}bytes", size)),
            &data,
            |b, data| {
                b.iter(|| {
                    encryption_dict.encrypt_data(black_box(data), black_box(10), black_box(0))
                });
            },
        );
    }

    group.finish();
}

fn bench_encryption_complete_pipeline_overhead(c: &mut Criterion) {
    let xml = generate_medium_document();
    let permissions = PdfPermissions::default();
    let security = PdfSecurity::new("owner", "user", permissions);

    let mut group = c.benchmark_group("encryption/pipeline_overhead");

    // Benchmark without encryption
    group.bench_function("without_encryption", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(xml.as_bytes()));
            let builder = FoTreeBuilder::new();
            let arena = builder
                .parse(cursor)
                .expect("bench/example: should succeed");
            let engine = LayoutEngine::new();
            let area_tree = engine
                .layout(&arena)
                .expect("bench/example: should succeed");
            let renderer = PdfRenderer::new();
            let pdf_doc = renderer
                .render(&area_tree)
                .expect("bench/example: should succeed");
            pdf_doc.to_bytes().expect("bench/example: should succeed")
        });
    });

    // Benchmark with encryption
    group.bench_function("with_encryption", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(xml.as_bytes()));
            let builder = FoTreeBuilder::new();
            let arena = builder
                .parse(cursor)
                .expect("bench/example: should succeed");
            let engine = LayoutEngine::new();
            let area_tree = engine
                .layout(&arena)
                .expect("bench/example: should succeed");
            let renderer = PdfRenderer::new();
            let mut pdf_doc = renderer
                .render(&area_tree)
                .expect("bench/example: should succeed");

            // Apply encryption to the document
            let file_id = fop_render::pdf::security::generate_file_id("benchmark-doc");
            let encryption_dict = black_box(security.clone()).compute_encryption_dict(&file_id);
            pdf_doc
                .set_encryption(encryption_dict, file_id)
                .expect("bench/example: should succeed");

            pdf_doc.to_bytes().expect("bench/example: should succeed")
        });
    });

    group.finish();
}

// ============================================================================
// 6. MEMORY USAGE BENCHMARKS
// ============================================================================

fn bench_memory_arena_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory/arena_allocation");

    for num_blocks in [100, 500, 1000, 5000].iter() {
        let xml = generate_fo_document(1, *num_blocks, false, false, false);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_nodes", num_blocks)),
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

fn bench_memory_area_tree_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory/area_tree_construction");

    for num_blocks in [100, 500, 1000, 2000].iter() {
        let xml = generate_fo_document(1, *num_blocks, true, true, false);
        let cursor = Cursor::new(xml.as_bytes());
        let builder = FoTreeBuilder::new();
        let arena = builder
            .parse(cursor)
            .expect("bench/example: should succeed");

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_blocks", num_blocks)),
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

fn bench_memory_pdf_document_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory/pdf_document_building");

    for num_pages in [1, 5, 10, 20].iter() {
        let xml = generate_fo_document(*num_pages, 50, true, true, false);
        let cursor = Cursor::new(xml.as_bytes());
        let builder = FoTreeBuilder::new();
        let arena = builder
            .parse(cursor)
            .expect("bench/example: should succeed");
        let engine = LayoutEngine::new();
        let area_tree = engine
            .layout(&arena)
            .expect("bench/example: should succeed");

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_pages", num_pages)),
            &area_tree,
            |b, area_tree| {
                b.iter(|| {
                    let renderer = PdfRenderer::new();
                    renderer
                        .render(black_box(area_tree))
                        .expect("bench/example: should succeed")
                });
            },
        );
    }

    group.finish();
}

fn bench_memory_string_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory/string_operations");

    let svg_doc = {
        let xml = generate_medium_document();
        let cursor = Cursor::new(xml.as_bytes());
        let builder = FoTreeBuilder::new();
        let arena = builder
            .parse(cursor)
            .expect("bench/example: should succeed");
        let engine = LayoutEngine::new();
        let area_tree = engine
            .layout(&arena)
            .expect("bench/example: should succeed");
        let renderer = SvgRenderer::new();
        renderer
            .render_to_svg(&area_tree)
            .expect("bench/example: should succeed")
    };

    group.bench_function("svg_to_string", |b| {
        b.iter(|| {
            let _s = black_box(&svg_doc).clone();
        });
    });

    let pdf_doc = {
        let xml = generate_medium_document();
        let cursor = Cursor::new(xml.as_bytes());
        let builder = FoTreeBuilder::new();
        let arena = builder
            .parse(cursor)
            .expect("bench/example: should succeed");
        let engine = LayoutEngine::new();
        let area_tree = engine
            .layout(&arena)
            .expect("bench/example: should succeed");
        let renderer = PdfRenderer::new();
        renderer
            .render(&area_tree)
            .expect("bench/example: should succeed")
    };

    group.bench_function("pdf_to_bytes", |b| {
        b.iter(|| {
            black_box(&pdf_doc)
                .to_bytes()
                .expect("bench/example: should succeed")
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark Groups Configuration
// ============================================================================

criterion_group!(
    name = parsing_benches;
    config = Criterion::default();
    targets =
        bench_parsing_small,
        bench_parsing_medium,
        bench_parsing_large,
        bench_parsing_very_large,
        bench_parsing_scaling
);

criterion_group!(
    name = layout_benches;
    config = Criterion::default();
    targets =
        bench_layout_simple_blocks,
        bench_layout_complex_blocks,
        bench_layout_inline_with_breaking,
        bench_layout_tables,
        bench_layout_nested_lists,
        bench_layout_page_breaking
);

criterion_group!(
    name = pdf_rendering_benches;
    config = Criterion::default();
    targets =
        bench_pdf_render_text_only,
        bench_pdf_render_styled_graphics,
        bench_pdf_render_tables,
        bench_pdf_complete_pipeline,
        bench_pdf_serialization
);

criterion_group!(
    name = svg_rendering_benches;
    config = Criterion::default();
    targets =
        bench_svg_render_simple,
        bench_svg_render_styled,
        bench_svg_render_multipage,
        bench_svg_complete_pipeline
);

criterion_group!(
    name = encryption_benches;
    config = Criterion::default();
    targets =
        bench_encryption_setup,
        bench_encryption_encrypt_data,
        bench_encryption_encrypt_large_data,
        bench_encryption_complete_pipeline_overhead
);

criterion_group!(
    name = memory_benches;
    config = Criterion::default();
    targets =
        bench_memory_arena_allocation,
        bench_memory_area_tree_construction,
        bench_memory_pdf_document_building,
        bench_memory_string_allocation
);

criterion_main!(
    parsing_benches,
    layout_benches,
    pdf_rendering_benches,
    svg_rendering_benches,
    encryption_benches,
    memory_benches
);
