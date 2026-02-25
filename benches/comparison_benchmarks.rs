//! Performance benchmarks comparing different implementations
//!
//! These benchmarks measure key operations and can be used to compare
//! against Java FOP or track performance regressions.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::PdfRenderer;
use std::hint::black_box;
use std::io::Cursor;

/// Benchmark parsing XSL-FO documents of various sizes
fn bench_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");

    for size in [10, 50, 100, 500].iter() {
        let fo_doc = generate_fo_document(*size);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_blocks", size)),
            size,
            |b, _| {
                b.iter(|| {
                    let builder = FoTreeBuilder::new();
                    let cursor = Cursor::new(fo_doc.as_bytes());
                    let _ = builder
                        .parse(cursor)
                        .expect("bench/example: should succeed");
                });
            },
        );
    }

    group.finish();
}

/// Benchmark layout engine performance
fn bench_layout(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout");

    for size in [10, 50, 100].iter() {
        let fo_doc = generate_fo_document(*size);
        let builder = FoTreeBuilder::new();
        let fo_tree = builder
            .parse(Cursor::new(fo_doc.as_bytes()))
            .expect("bench/example: should succeed");

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_blocks", size)),
            size,
            |b, _| {
                b.iter(|| {
                    let engine = LayoutEngine::new();
                    let _ = engine
                        .layout(black_box(&fo_tree))
                        .expect("bench/example: should succeed");
                });
            },
        );
    }

    group.finish();
}

/// Benchmark PDF rendering
fn bench_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("rendering");

    for size in [10, 50, 100].iter() {
        let fo_doc = generate_fo_document(*size);
        let builder = FoTreeBuilder::new();
        let fo_tree = builder
            .parse(Cursor::new(fo_doc.as_bytes()))
            .expect("bench/example: should succeed");
        let engine = LayoutEngine::new();
        let area_tree = engine
            .layout(&fo_tree)
            .expect("bench/example: should succeed");

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_blocks", size)),
            size,
            |b, _| {
                b.iter(|| {
                    let renderer = PdfRenderer::new();
                    let _ = renderer
                        .render(black_box(&area_tree))
                        .expect("bench/example: should succeed");
                });
            },
        );
    }

    group.finish();
}

/// Benchmark complete end-to-end pipeline
fn bench_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");

    for size in [10, 50, 100].iter() {
        let fo_doc = generate_fo_document(*size);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_blocks", size)),
            size,
            |b, _| {
                b.iter(|| {
                    // Parse
                    let builder = FoTreeBuilder::new();
                    let fo_tree = builder
                        .parse(Cursor::new(fo_doc.as_bytes()))
                        .expect("bench/example: should succeed");

                    // Layout
                    let engine = LayoutEngine::new();
                    let area_tree = engine
                        .layout(&fo_tree)
                        .expect("bench/example: should succeed");

                    // Render
                    let renderer = PdfRenderer::new();
                    let pdf = renderer
                        .render(&area_tree)
                        .expect("bench/example: should succeed");

                    // Serialize
                    let _ = pdf.to_bytes().expect("bench/example: should succeed");
                });
            },
        );
    }

    group.finish();
}

/// Benchmark table layout specifically
fn bench_table_layout(c: &mut Criterion) {
    let table_doc = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="page" page-width="8.5in" page-height="11in" margin="1in">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="page">
    <fo:flow flow-name="xsl-region-body">
      <fo:table table-layout="fixed" width="100%">
        <fo:table-column column-width="25%"/>
        <fo:table-column column-width="25%"/>
        <fo:table-column column-width="25%"/>
        <fo:table-column column-width="25%"/>
        <fo:table-body>
          TABLE_ROWS
        </fo:table-body>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    let mut group = c.benchmark_group("table_layout");

    for rows in [10, 50, 100].iter() {
        let mut table_rows = String::new();
        for i in 0..*rows {
            table_rows.push_str(&format!(
                r#"
          <fo:table-row>
            <fo:table-cell padding="2pt" border="0.5pt solid black">
              <fo:block>Cell {}-1</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2pt" border="0.5pt solid black">
              <fo:block>Cell {}-2</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2pt" border="0.5pt solid black">
              <fo:block>Cell {}-3</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2pt" border="0.5pt solid black">
              <fo:block>Cell {}-4</fo:block>
            </fo:table-cell>
          </fo:table-row>"#,
                i, i, i, i
            ));
        }

        let doc = table_doc.replace("TABLE_ROWS", &table_rows);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_rows", rows)),
            rows,
            |b, _| {
                b.iter(|| {
                    let builder = FoTreeBuilder::new();
                    let fo_tree = builder
                        .parse(Cursor::new(doc.as_bytes()))
                        .expect("bench/example: should succeed");
                    let engine = LayoutEngine::new();
                    let _ = engine
                        .layout(&fo_tree)
                        .expect("bench/example: should succeed");
                });
            },
        );
    }

    group.finish();
}

/// Generate a test FO document with the specified number of blocks
fn generate_fo_document(num_blocks: usize) -> String {
    let mut blocks = String::new();
    for i in 0..num_blocks {
        blocks.push_str(&format!(
            r#"<fo:block font-size="12pt" space-after="6pt">
        Block {} - Lorem ipsum dolor sit amet, consectetur adipiscing elit.
      </fo:block>
      "#,
            i
        ));
    }

    format!(
        r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="page" page-width="8.5in" page-height="11in" margin="1in">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="page">
    <fo:flow flow-name="xsl-region-body">
      {}
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#,
        blocks
    )
}

criterion_group!(
    benches,
    bench_parsing,
    bench_layout,
    bench_rendering,
    bench_end_to_end,
    bench_table_layout
);
criterion_main!(benches);
