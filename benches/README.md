# FOP Performance Benchmarks

This directory contains comprehensive performance benchmarks for the Apache FOP Rust implementation using the Criterion framework.

## Benchmark Suites

### 1. `performance_benchmarks.rs` - Comprehensive Performance Suite

The main performance benchmarking suite covering all aspects of the FOP implementation:

#### Parsing Speed Benchmarks
- **Small documents** (1 page, ~100 elements, ~2KB)
- **Medium documents** (10 pages, ~1000 elements, ~50KB)
- **Large documents** (100 pages, ~10000 elements, ~500KB)
- **Very large documents** (500 pages, ~50000 elements, ~2.5MB)
- **Scaling analysis** (10 to 2000 blocks)

#### Layout Engine Benchmarks
- Simple block layout (100 blocks)
- Complex styled blocks with graphics
- Inline text with line breaking
- Table layout (various sizes: 5x3 to 100x5)
- Nested lists (depth 2-4)
- Page breaking (1 to 50 pages)

#### PDF Rendering Benchmarks
- Text-only rendering
- Styled graphics rendering
- Table rendering
- Complete pipeline (parse → layout → render → serialize)
- PDF serialization

#### SVG Rendering Benchmarks
- Simple document rendering
- Styled graphics rendering
- Multi-page SVG output
- Complete SVG pipeline

#### Encryption Overhead Benchmarks
- Encryption setup (dictionary computation)
- Data encryption (various sizes: 150 bytes to 1MB)
- Complete pipeline comparison (with/without encryption)

#### Memory Usage Benchmarks
- Arena allocation patterns
- Area tree construction
- PDF document building
- String operations (SVG/PDF serialization)

## Running Benchmarks

### Run all performance benchmarks
```bash
cargo bench --bench performance_benchmarks
```

### Run specific benchmark groups
```bash
# Parsing benchmarks only
cargo bench --bench performance_benchmarks -- "parsing"

# Layout benchmarks only
cargo bench --bench performance_benchmarks -- "layout"

# PDF rendering benchmarks
cargo bench --bench performance_benchmarks -- "pdf_rendering"

# SVG rendering benchmarks
cargo bench --bench performance_benchmarks -- "svg_rendering"

# Encryption benchmarks
cargo bench --bench performance_benchmarks -- "encryption"

# Memory benchmarks
cargo bench --bench performance_benchmarks -- "memory"
```

### Run specific individual benchmarks
```bash
# Small document parsing
cargo bench --bench performance_benchmarks -- "parsing/small"

# Table layout benchmarks
cargo bench --bench performance_benchmarks -- "layout/tables"

# Complete PDF pipeline
cargo bench --bench performance_benchmarks -- "pdf_rendering/complete_pipeline"
```

### Quick benchmarks (faster, less accurate)
```bash
cargo bench --bench performance_benchmarks -- --quick
```

### Save baseline for comparison
```bash
# Save baseline
cargo bench --bench performance_benchmarks -- --save-baseline main

# Compare against baseline
cargo bench --bench performance_benchmarks -- --baseline main
```

## Other Benchmark Suites

### `fop_benchmarks.rs`
Original comprehensive benchmarks covering:
- Parsing (small/medium/large documents, scaling)
- Layout (blocks, inline, tables, lists, multi-page)
- Rendering (text, styled, complete pipeline)
- Property system (parsing, access, inheritance, length conversions)

### `comparison_benchmarks.rs`
Comparative benchmarks for different approaches and optimizations.

## Understanding Results

Criterion provides detailed statistical analysis including:
- **Time**: Mean execution time with confidence intervals
- **Throughput**: Bytes/second for data processing benchmarks
- **Comparison**: Performance changes compared to previous runs
- **Plots**: Visual representation of timing distributions (if gnuplot is installed)

### Example Output
```
parsing/small_document/1_page_100_elements
                        time:   [241.38 µs 243.95 µs 254.23 µs]
                        thrpt:  [15.192 MiB/s 15.833 MiB/s 16.001 MiB/s]
```

This shows:
- Mean time: 243.95 µs (with 95% confidence interval)
- Throughput: 15.833 MiB/s (average)

## Performance Targets

Based on the benchmark design, the implementation aims for:

- **Parsing**: < 1ms per page for simple documents
- **Layout**: < 5ms per page for typical documents
- **Rendering**: < 2ms per page for text-heavy documents
- **Total pipeline**: < 10ms per page for simple documents
- **Encryption overhead**: < 20% impact on total pipeline time

## Reproducibility

All benchmarks are designed to be:
- **Deterministic**: Same input always produces same output
- **Reproducible**: Results should be consistent across runs
- **Scalable**: Test various document sizes and complexities
- **Meaningful**: Test real-world scenarios

## Tips for Benchmarking

1. **Close other applications** to reduce system noise
2. **Run on battery power** (if laptop) or with performance governor
3. **Disable CPU frequency scaling** for more consistent results
4. **Use release mode** (Criterion does this automatically)
5. **Run multiple times** to ensure statistical significance
6. **Compare baselines** when making optimizations

## Interpreting Results

When analyzing benchmark results:

1. **Look at trends**: Is performance linear with document size?
2. **Identify bottlenecks**: Which phase takes the most time?
3. **Check memory usage**: Are there excessive allocations?
4. **Compare with baselines**: Did optimizations help?
5. **Consider real-world impact**: Does it matter for typical use cases?

## Adding New Benchmarks

When adding new benchmarks:

1. Use clear, descriptive names
2. Include document generators for reproducibility
3. Test multiple sizes/complexities
4. Set appropriate throughput measurements
5. Document expected performance characteristics
6. Follow existing patterns for consistency

## Resources

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [FOP Project Documentation](../README.md)
