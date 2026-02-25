# FOP CLI - Apache FOP Command-Line Interface

A high-performance command-line tool for converting XSL-FO documents to PDF format.

## Features

- **Fast conversion** - Rust-based implementation for optimal performance
- **Progress reporting** - Real-time progress bars for large documents
- **Statistics** - Detailed processing metrics and timing information
- **Validation** - Validate XSL-FO documents without generating output
- **Flexible options** - Support for custom fonts, images, and PDF metadata
- **Multiple output formats** - JSON or text-based statistics

## Installation

### From Source

```bash
cargo install --path crates/fop-cli
```

### From Workspace

```bash
cargo build --release --package fop-cli
cp target/release/fop /usr/local/bin/
```

## Usage

### Basic Conversion

```bash
fop input.fo output.pdf
```

### With Progress and Statistics

```bash
fop input.fo output.pdf --stats
```

### Validation Only

```bash
fop input.fo --validate-only
```

### Verbose Output

```bash
fop input.fo output.pdf --verbose
```

### Custom Resources

```bash
fop input.fo output.pdf \
    --images-dir ./images \
    --font-dir ./fonts
```

### PDF Metadata

```bash
fop input.fo output.pdf \
    --title "My Document" \
    --author "John Doe" \
    --subject "Technical Report" \
    --keywords "fop, pdf, xsl-fo"
```

### Advanced Options

```bash
fop input.fo output.pdf \
    --compress \
    --pdf-version 1.7 \
    --jobs 4 \
    --strict
```

## Command-Line Options

### Input/Output

- `<INPUT>` - Input XSL-FO file (required)
- `[OUTPUT]` - Output PDF file (optional with `--validate-only`)

### Processing Options

- `-v, --verbose` - Enable verbose logging
- `--stats` - Show detailed statistics after conversion
- `--validate-only` - Only validate the XSL-FO document
- `-q, --quiet` - Suppress progress bars and animations
- `--no-progress` - Disable progress reporting (for scripting)

### Resources

- `--images-dir <DIR>` - Directory containing images
- `--font-dir <DIR>` - Directory containing fonts

### PDF Options

- `--compress` - Enable PDF compression (reduces file size)
- `--pdf-version <VERSION>` - PDF version (1.4, 1.5, 1.6, 1.7, 2.0) [default: 1.4]
- `--author <AUTHOR>` - Set PDF author metadata
- `--title <TITLE>` - Set PDF title metadata
- `--subject <SUBJECT>` - Set PDF subject metadata
- `--keywords <KEYWORDS>` - Set PDF keywords metadata

### Performance

- `-j, --jobs <N>` - Number of threads to use (default: auto-detect)
- `--max-memory <MB>` - Maximum memory usage in MB

### Validation

- `--strict` - Enable strict XSL-FO validation
- `--fail-fast` - Stop processing after first error

### Output Format

- `--output-format <FORMAT>` - Output format for validation results (text, json) [default: text]

## Examples

### Basic Document Conversion

```bash
# Simple conversion
fop document.fo document.pdf

# With progress and stats
fop document.fo document.pdf --stats
```

### Large Document Processing

```bash
# Show progress for large documents
fop large-report.fo large-report.pdf --verbose --stats

# Use multiple threads
fop large-report.fo large-report.pdf -j 8
```

### Validation Workflow

```bash
# Validate before generating
fop document.fo --validate-only

# If valid, generate PDF
fop document.fo document.pdf
```

### Scripting Integration

```bash
# Disable progress bars for scripts
fop document.fo document.pdf --quiet --stats --output-format json > stats.json

# Exit code indicates success (0) or failure (non-zero)
if fop document.fo document.pdf --quiet; then
    echo "Success"
else
    echo "Failed"
fi
```

### Production Deployment

```bash
# Compressed PDF with metadata
fop report.fo report.pdf \
    --compress \
    --pdf-version 1.7 \
    --title "Annual Report 2024" \
    --author "Acme Corporation" \
    --subject "Financial Report" \
    --keywords "finance, annual, 2024"
```

## Output

### Progress Output

```
Apache FOP
Rust Implementation

✓ Read input file: document.fo (5ms)
✓ Parsed FO tree (1243 nodes) (245ms)
✓ Layout complete (3456 areas) (1.2s)
✓ PDF rendered (42 pages) (890ms)
✓ Saved to document.pdf (15ms)

✓ Success!

  Input:   document.fo
  Output:  document.pdf
  Size:    234.5 KB → 1.2 MB
  Pages:   42
  Time:    2.35s
```

### Statistics Output (Text)

```
Processing Statistics:

  ⚙️ Parsing:      245ms
  ⚙️ Layout:       1.20s
  ⚙️ Rendering:    890ms
  ✨Total:        2.35s

  •FO Nodes:     1243
  •Areas:        3456
  📄Pages:        42

  Input size:    234.5 KB
  Output size:   1.2 MB
  Size ratio:    5.12x

  Throughput:    17.87 pages/sec
```

### Statistics Output (JSON)

```json
{
  "parsing": {
    "duration_ms": 245,
    "nodes": 1243
  },
  "layout": {
    "duration_ms": 1200,
    "areas": 3456
  },
  "rendering": {
    "duration_ms": 890,
    "pages": 42
  },
  "total": {
    "duration_ms": 2350,
    "input_bytes": 240128,
    "output_bytes": 1258291,
    "warnings": 0,
    "errors": 0
  }
}
```

## Performance Tips

### For Large Documents

1. **Use multiple threads**: `-j 8` (adjust based on CPU cores)
2. **Enable compression**: `--compress` (reduces output size)
3. **Suppress progress**: `--quiet` (slight performance gain)

### For Batch Processing

```bash
#!/bin/bash
for fo_file in *.fo; do
    pdf_file="${fo_file%.fo}.pdf"
    fop "$fo_file" "$pdf_file" --quiet --compress
done
```

### For CI/CD Pipelines

```yaml
# Example GitHub Actions workflow
- name: Generate PDF
  run: |
    fop document.fo document.pdf \
      --quiet \
      --fail-fast \
      --stats \
      --output-format json > stats.json
```

## Error Handling

The CLI returns different exit codes:

- `0` - Success
- `1` - Error (parsing, layout, rendering, or I/O)

Errors are printed to stderr with detailed context:

```
Error: Failed to parse XSL-FO document

Caused by:
  0: Unexpected element: fo:invalid-element
  1: Line 42, column 5
```

## Logging

Set `RUST_LOG` environment variable for fine-grained control:

```bash
# Debug all FOP modules
RUST_LOG=debug fop document.fo document.pdf

# Only show errors
RUST_LOG=error fop document.fo document.pdf

# Module-specific logging
RUST_LOG=fop_core=debug,fop_layout=info fop document.fo document.pdf
```

## Building

```bash
# Debug build
cargo build --package fop-cli

# Release build (optimized)
cargo build --release --package fop-cli

# Run tests
cargo test --package fop-cli

# Run clippy
cargo clippy --package fop-cli -- -D warnings
```

## License

Apache License 2.0

## See Also

- [FOP Core](../fop-core/README.md) - FO tree parsing
- [FOP Layout](../fop-layout/README.md) - Layout engine
- [FOP Render](../fop-render/README.md) - PDF rendering
