# FOP CLI - Complete Usage Guide

## Quick Start

### Installation

```bash
# Build from source
cargo build --release --package fop-cli

# Install to system
sudo cp target/release/fop /usr/local/bin/

# Or install to user bin
cp target/release/fop ~/.local/bin/
```

### First Conversion

```bash
# Create a simple XSL-FO file
cat > hello.fo <<'EOF'
<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
      <fo:region-body margin="1in"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="18pt">Hello, World!</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>
EOF

# Convert to PDF
fop hello.fo hello.pdf

# View result
xdg-open hello.pdf
```

## Common Use Cases

### 1. Simple Document Conversion

```bash
# Basic conversion
fop document.fo document.pdf

# With progress reporting
fop document.fo document.pdf --verbose

# Quiet mode (for scripts)
fop document.fo document.pdf --quiet
```

### 2. Document Validation

```bash
# Validate without generating PDF
fop document.fo --validate-only

# Strict validation
fop document.fo --validate-only --strict
```

### 3. Performance Optimization

```bash
# Enable compression
fop document.fo document.pdf --compress

# Use multiple threads
fop document.fo document.pdf -j 8

# Both
fop document.fo document.pdf --compress -j 8
```

### 4. Metadata and PDF Options

```bash
# Add PDF metadata
fop document.fo document.pdf \
  --title "Annual Report 2024" \
  --author "Acme Corp" \
  --subject "Financial Report" \
  --keywords "finance, annual, 2024"

# Specify PDF version
fop document.fo document.pdf --pdf-version 1.7

# Complete metadata
fop document.fo document.pdf \
  --pdf-version 1.7 \
  --compress \
  --title "Technical Manual" \
  --author "Engineering Team" \
  --subject "Product Documentation" \
  --keywords "manual, technical, v2.0"
```

### 5. Statistics and Monitoring

```bash
# Show text statistics
fop document.fo document.pdf --stats

# JSON output for monitoring
fop document.fo document.pdf --stats --output-format json

# Redirect JSON to file
fop document.fo document.pdf --stats --output-format json > stats.json
```

### 6. Custom Resources

```bash
# Specify images directory
fop document.fo document.pdf --images-dir ./images

# Specify fonts directory
fop document.fo document.pdf --font-dir ./fonts

# Both
fop document.fo document.pdf \
  --images-dir ./assets/images \
  --font-dir ./assets/fonts
```

## Batch Processing

### Simple Batch Script

```bash
#!/bin/bash
# Convert all .fo files in a directory

for fo_file in *.fo; do
  pdf_file="${fo_file%.fo}.pdf"
  echo "Converting $fo_file to $pdf_file..."
  fop "$fo_file" "$pdf_file" --quiet --compress
done

echo "Batch conversion complete!"
```

### Parallel Processing

```bash
#!/bin/bash
# Process multiple files in parallel using GNU parallel

parallel fop {} {.}.pdf --quiet --compress ::: *.fo
```

### With Error Handling

```bash
#!/bin/bash
# Batch processing with error handling

SUCCESS=0
FAILED=0

for fo_file in *.fo; do
  pdf_file="${fo_file%.fo}.pdf"

  if fop "$fo_file" "$pdf_file" --quiet --compress; then
    echo "✓ $fo_file → $pdf_file"
    SUCCESS=$((SUCCESS + 1))
  else
    echo "✗ Failed: $fo_file" >&2
    FAILED=$((FAILED + 1))
  fi
done

echo ""
echo "Results: $SUCCESS success, $FAILED failed"

if [ $FAILED -gt 0 ]; then
  exit 1
fi
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Generate PDF Documentation

on:
  push:
    paths:
      - 'docs/**/*.fo'

jobs:
  generate-pdfs:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build FOP CLI
        run: cargo build --release --package fop-cli

      - name: Generate PDFs
        run: |
          mkdir -p output
          for fo_file in docs/*.fo; do
            filename=$(basename "$fo_file" .fo)
            ./target/release/fop "$fo_file" "output/${filename}.pdf" \
              --quiet --compress --stats --output-format json > "output/${filename}.json"
          done

      - name: Upload Artifacts
        uses: actions/upload-artifact@v3
        with:
          name: pdf-documentation
          path: output/*.pdf
```

### GitLab CI

```yaml
stages:
  - build
  - generate

build-fop:
  stage: build
  image: rust:latest
  script:
    - cargo build --release --package fop-cli
  artifacts:
    paths:
      - target/release/fop

generate-pdfs:
  stage: generate
  dependencies:
    - build-fop
  script:
    - mkdir -p output
    - |
      for fo_file in docs/*.fo; do
        filename=$(basename "$fo_file" .fo)
        ./target/release/fop "$fo_file" "output/${filename}.pdf" --quiet --compress
      done
  artifacts:
    paths:
      - output/*.pdf
```

### Jenkins Pipeline

```groovy
pipeline {
    agent any

    stages {
        stage('Build') {
            steps {
                sh 'cargo build --release --package fop-cli'
            }
        }

        stage('Generate PDFs') {
            steps {
                sh '''
                    mkdir -p output
                    for fo_file in docs/*.fo; do
                        filename=$(basename "$fo_file" .fo)
                        ./target/release/fop "$fo_file" "output/${filename}.pdf" \
                            --quiet --compress --stats --output-format json > "output/${filename}.json"
                    done
                '''
            }
        }

        stage('Archive') {
            steps {
                archiveArtifacts artifacts: 'output/*.pdf', fingerprint: true
                archiveArtifacts artifacts: 'output/*.json', fingerprint: true
            }
        }
    }
}
```

## Advanced Usage

### Environment Variables

```bash
# Set log level
export RUST_LOG=debug
fop document.fo document.pdf

# Module-specific logging
export RUST_LOG=fop_core=debug,fop_layout=info
fop document.fo document.pdf

# Disable all logging
export RUST_LOG=off
fop document.fo document.pdf --quiet
```

### Custom Configuration

```bash
# Set resource limits
fop document.fo document.pdf --max-memory 1024  # 1GB limit

# Performance tuning
fop document.fo document.pdf \
  -j 8 \              # 8 threads
  --max-memory 2048   # 2GB memory limit
```

### Integration with Other Tools

```bash
# Convert XSLT + XML to FO to PDF
xsltproc stylesheet.xsl data.xml | fop - output.pdf

# Generate from template
cat template.fo | sed 's/{{title}}/My Document/' | fop - output.pdf

# Validate and convert in pipeline
fop document.fo --validate-only && fop document.fo document.pdf
```

## Troubleshooting

### Common Issues

#### 1. Input file not found
```bash
# Error: Input file does not exist
# Solution: Check path
ls -l document.fo
fop "$(pwd)/document.fo" output.pdf
```

#### 2. Output directory doesn't exist
```bash
# Error: Output directory does not exist
# Solution: Create directory first
mkdir -p output
fop document.fo output/document.pdf
```

#### 3. Permission denied
```bash
# Error: Permission denied
# Solution: Check permissions
chmod 644 document.fo
chmod 755 output/
```

#### 4. Out of memory
```bash
# Error: Out of memory
# Solution: Set memory limit or reduce document size
fop document.fo document.pdf --max-memory 4096
```

### Debug Mode

```bash
# Enable verbose logging
RUST_LOG=debug fop document.fo document.pdf --verbose

# Trace all operations
RUST_LOG=trace fop document.fo document.pdf --verbose
```

### Performance Profiling

```bash
# Time the conversion
time fop document.fo document.pdf

# With statistics
fop document.fo document.pdf --stats --output-format json > profile.json
```

## Best Practices

### 1. Development Workflow

```bash
# 1. Validate first
fop document.fo --validate-only

# 2. If valid, generate PDF
if [ $? -eq 0 ]; then
  fop document.fo document.pdf
fi
```

### 2. Production Deployment

```bash
# Use compression and metadata
fop document.fo document.pdf \
  --compress \
  --pdf-version 1.7 \
  --title "Production Document" \
  --author "Your Organization"
```

### 3. Monitoring

```bash
# Collect metrics in JSON
fop document.fo document.pdf \
  --stats \
  --output-format json \
  --quiet > metrics.json

# Send to monitoring system
curl -X POST http://metrics-server/api/fop \
  -H "Content-Type: application/json" \
  -d @metrics.json
```

### 4. Error Handling

```bash
# Always check exit codes
if ! fop document.fo document.pdf --quiet; then
  echo "Conversion failed" >&2
  exit 1
fi

# Use fail-fast in scripts
fop document.fo document.pdf --quiet --fail-fast || exit 1
```

## Performance Tips

### For Large Documents

1. **Enable compression**: Reduces output file size
2. **Use multiple threads**: Faster processing with `-j N`
3. **Suppress progress**: Small performance gain with `--quiet`
4. **Allocate more memory**: Use `--max-memory` for large documents

```bash
fop large-doc.fo large-doc.pdf \
  --compress \
  -j 8 \
  --max-memory 4096 \
  --quiet
```

### For Batch Processing

1. **Use parallel processing**: Process multiple files simultaneously
2. **Disable progress bars**: Use `--quiet` for scripts
3. **Stream output**: Redirect stats to files
4. **Resource limits**: Set `--max-memory` to prevent OOM

```bash
# Process in parallel with GNU parallel
ls *.fo | parallel -j 4 fop {} {.}.pdf --quiet --compress
```

## Output Examples

### Standard Output
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

### Statistics (Text)
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

### Statistics (JSON)
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

## Support

For issues, questions, or contributions:

- GitHub Issues: https://github.com/apache/fop-rust/issues
- Documentation: See README.md in the fop-cli crate
- Examples: Check the examples/ directory

## License

Apache License 2.0
