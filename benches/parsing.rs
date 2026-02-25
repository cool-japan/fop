//! Benchmarks for FO parsing

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use fop_core::FoTreeBuilder;
use std::hint::black_box;
use std::io::Cursor;

fn generate_fo_document(num_blocks: usize) -> String {
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
"##,
    );

    for i in 0..num_blocks {
        xml.push_str(&format!(
            r##"            <fo:block font-size="12pt" color="black">
                Block {} with some text content that should be parsed correctly.
            </fo:block>
"##,
            i
        ));
    }

    xml.push_str(
        r##"        </fo:flow>
    </fo:page-sequence>
</fo:root>"##,
    );

    xml
}

fn parse_document(xml: &str) {
    let cursor = Cursor::new(xml);
    let builder = FoTreeBuilder::new();
    let _ = builder
        .parse(cursor)
        .expect("bench/example: should succeed");
}

fn bench_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");

    for num_blocks in [10, 50, 100, 500].iter() {
        let xml = generate_fo_document(*num_blocks);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_blocks", num_blocks)),
            &xml,
            |b, xml| {
                b.iter(|| parse_document(black_box(xml)));
            },
        );
    }

    group.finish();
}

fn bench_property_access(c: &mut Criterion) {
    use fop_core::{Length, PropertyId, PropertyList, PropertyValue};

    let mut props = PropertyList::new();
    props.set(
        PropertyId::FontSize,
        PropertyValue::Length(Length::from_pt(12.0)),
    );
    props.set(
        PropertyId::Color,
        PropertyValue::Color(fop_core::Color::BLACK),
    );

    c.bench_function("property_get", |b| {
        b.iter(|| {
            let _ = black_box(props.get(PropertyId::FontSize));
            let _ = black_box(props.get(PropertyId::Color));
        });
    });
}

fn bench_length_conversions(c: &mut Criterion) {
    use fop_core::Length;

    c.bench_function("length_pt_to_mm", |b| {
        b.iter(|| {
            let len = Length::from_pt(black_box(72.0));
            black_box(len.to_mm())
        });
    });

    c.bench_function("length_arithmetic", |b| {
        b.iter(|| {
            let a = Length::from_pt(black_box(10.0));
            let b = Length::from_pt(black_box(20.0));
            black_box(a + b)
        });
    });
}

criterion_group!(
    benches,
    bench_parsing,
    bench_property_access,
    bench_length_conversions
);
criterion_main!(benches);
