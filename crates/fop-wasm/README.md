# @cooljapan/fop

[![npm](https://img.shields.io/npm/v/@cooljapan/fop.svg)](https://www.npmjs.com/package/@cooljapan/fop)
[![Crates.io](https://img.shields.io/crates/v/fop-wasm.svg)](https://crates.io/crates/fop-wasm)
[![docs.rs](https://img.shields.io/docsrs/fop-wasm)](https://docs.rs/fop-wasm)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/cool-japan/fop/blob/main/LICENSE)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-ready-654FF0?logo=webassembly)](https://webassembly.org/)

WebAssembly bindings for [FOP](https://github.com/cool-japan/fop) -- a high-performance, pure-Rust reimplementation of the Apache FOP XSL-FO processor. Published as **`@cooljapan/fop`** on npm, this package exposes XSL-FO to PDF, SVG, and plain-text conversion directly in the browser, Node.js, or any bundler environment via [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen).

**Pure Rust implementation compiled to WebAssembly -- no native dependencies required.**

> **v0.1.1** released on 2026-04-20 -- glyph outline rendering (real TrueType/OpenType outlines via ttf-parser), text extraction API (`extract_text`/`extract_all_text`), and auto-verify pipeline improvements.

---

## Features

- **XSL-FO to PDF** -- converts XSL-FO documents to valid PDF bytes (`Uint8Array`) consumable directly by the browser or Node.js
- **XSL-FO to SVG** -- renders XSL-FO to scalable vector graphics (SVG string)
- **XSL-FO to plain text** -- extracts text content from XSL-FO documents
- **Document validation** -- parses and validates XSL-FO without rendering; returns a JSON result with node count or error details
- **Class-based API** (`FopConverter`) -- stateful object with optional verbose logging
- **One-shot function API** (`convertFoToPdf`, `convertFoToSvg`) -- convenience functions for ad-hoc conversions without instantiating a class
- **Supported format query** (`supportedFormats`) -- returns the list of supported output format identifiers at runtime
- **TypeScript support** -- type declarations (`.d.ts`) included out of the box
- **Pure Rust** -- no native C/Fortran dependencies; compiles cleanly to `wasm32-unknown-unknown`
- **10--1200x faster** than Java FOP for typical documents

---

## Installation

```bash
npm install @cooljapan/fop
# or
yarn add @cooljapan/fop
```

### Rust `[dependencies]`

If you want to use the Rust crate directly (e.g. from another WASM crate):

```toml
[dependencies]
fop-wasm = "0.1"
```

---

## Quick Start

### Browser (web target)

The WASM module must be initialized with `await init()` before calling any conversion function.

```javascript
import init, { FopConverter, convertFoToPdf } from '@cooljapan/fop';

// Initialize the WASM module (required before any API call)
await init();

const fop = new FopConverter();
const pdfBytes = fop.convertToPdf(foXml);

// Display in browser
const blob = new Blob([pdfBytes], { type: 'application/pdf' });
const url = URL.createObjectURL(blob);
window.open(url);
```

### Bundler (Webpack / Vite)

When using a bundler that handles WASM imports natively, no explicit `init()` call is needed:

```javascript
import { FopConverter } from '@cooljapan/fop';

const fop = new FopConverter();
const pdfBytes = fop.convertToPdf(foXml);
```

### Node.js

```javascript
const { FopConverter } = require('@cooljapan/fop');
const fs = require('fs');

const fop = new FopConverter();
const pdfBytes = fop.convertToPdf(foXml);

// Write to file
fs.writeFileSync('output.pdf', Buffer.from(pdfBytes));
console.log('PDF written to output.pdf');
```

---

## Usage Examples

The following examples use a shared XSL-FO document for clarity:

```javascript
const foXml = `<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
                           page-width="210mm" page-height="297mm"
                           margin-top="20mm" margin-bottom="20mm"
                           margin-left="25mm" margin-right="25mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="18pt" font-weight="bold">Hello, FOP!</fo:block>
      <fo:block font-size="12pt" space-before="6pt">
        Rendered entirely in WebAssembly.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>`;
```

### PDF Conversion

```javascript
import init, { FopConverter } from '@cooljapan/fop';
await init();

const fop = new FopConverter();

// Returns Uint8Array containing valid PDF bytes
const pdfBytes = fop.convertToPdf(foXml);

// Browser: display in a new tab
const blob = new Blob([pdfBytes], { type: 'application/pdf' });
const url = URL.createObjectURL(blob);
window.open(url);
```

### SVG Conversion

```javascript
import init, { FopConverter } from '@cooljapan/fop';
await init();

const fop = new FopConverter();

// Returns a string containing <svg>...</svg> markup
const svgString = fop.convertToSvg(foXml);

// Embed directly in the DOM
document.getElementById('preview').innerHTML = svgString;
```

### Text Extraction

```javascript
import init, { FopConverter } from '@cooljapan/fop';
await init();

const fop = new FopConverter();

// Returns plain text content extracted from the XSL-FO document
const textContent = fop.convertToText(foXml);
console.log(textContent);
// Output: "Hello, FOP!\nRendered entirely in WebAssembly."
```

### Document Validation

```javascript
import init, { FopConverter } from '@cooljapan/fop';
await init();

const fop = new FopConverter();

// Returns JSON: {"valid": true, "nodes": 7} or {"valid": false, "error": "..."}
const validationJson = fop.validate(foXml);
const result = JSON.parse(validationJson);

if (result.valid) {
  console.log(`Document is valid (${result.nodes} nodes)`);
} else {
  console.error(`Validation failed: ${result.error}`);
}
```

### One-shot Functions vs FopConverter Class

For simple one-off conversions, use the standalone functions without instantiating a class:

```javascript
import init, { convertFoToPdf, convertFoToSvg, supportedFormats } from '@cooljapan/fop';
await init();

// One-shot PDF conversion
const pdfBytes = convertFoToPdf(foXml);

// One-shot SVG conversion
const svgString = convertFoToSvg(foXml);

// Query available output formats
const formats = supportedFormats(); // ["pdf", "svg", "text"]
console.log('Supported formats:', formats.join(', '));
```

For repeated conversions or when you need verbose logging, use the `FopConverter` class:

```javascript
import init, { FopConverter } from '@cooljapan/fop';
await init();

const fop = new FopConverter();
fop.setVerbose(true); // Enable verbose logging

const pdf1 = fop.convertToPdf(document1);
const pdf2 = fop.convertToPdf(document2);
const svg1 = fop.convertToSvg(document3);

console.log(fop.version()); // e.g. "fop-wasm 0.1.1"
```

### Error Handling

All conversion methods throw JavaScript `Error` objects when conversion fails (via wasm-bindgen's `Result<T, JsValue>` mapping). Use try/catch:

```javascript
try {
  const pdf = fop.convertToPdf(invalidFoXml);
} catch (err) {
  console.error('Conversion failed:', err.message);
}
```

---

## API Reference

### `FopConverter` Class

The primary API surface. Create an instance and call conversion methods on it.

| Method | Signature | Returns | Description |
|---|---|---|---|
| `constructor` | `new FopConverter()` | `FopConverter` | Create a new converter instance |
| `convertToPdf` | `convertToPdf(fo_xml: string)` | `Uint8Array` | Convert XSL-FO to PDF bytes |
| `convertToSvg` | `convertToSvg(fo_xml: string)` | `string` | Convert XSL-FO to SVG markup |
| `convertToText` | `convertToText(fo_xml: string)` | `string` | Extract plain text from XSL-FO |
| `validate` | `validate(fo_xml: string)` | `string` | Validate XSL-FO and return JSON result |
| `setVerbose` | `setVerbose(verbose: boolean)` | `void` | Enable or disable verbose logging |
| `version` | `version()` | `string` | Get version string (e.g. `"fop-wasm 0.1.1"`) |
| `free` | `free()` | `void` | Explicitly free WASM memory (optional; GC handles this) |

### Standalone Functions

| Function | Signature | Returns | Description |
|---|---|---|---|
| `convertFoToPdf` | `convertFoToPdf(fo_xml: string)` | `Uint8Array` | One-shot XSL-FO to PDF conversion |
| `convertFoToSvg` | `convertFoToSvg(fo_xml: string)` | `string` | One-shot XSL-FO to SVG conversion |
| `supportedFormats` | `supportedFormats()` | `string[]` | Returns `["pdf", "svg", "text"]` |

### Initialization

| Function | Signature | Description |
|---|---|---|
| `init` (default export) | `init(input?: RequestInfo \| URL \| Response \| BufferSource \| WebAssembly.Module)` | Initialize the WASM module. Required for browser/web target before any API call. Not needed for bundler or Node.js targets. |

---

## TypeScript Support

Type declarations are included in the package. The generated `.d.ts` provides full type safety:

```typescript
export class FopConverter {
  constructor();
  setVerbose(verbose: boolean): void;
  convertToPdf(fo_xml: string): Uint8Array;
  convertToSvg(fo_xml: string): string;
  convertToText(fo_xml: string): string;
  validate(fo_xml: string): string;
  version(): string;
  free(): void;
}

export function convertFoToPdf(fo_xml: string): Uint8Array;
export function convertFoToSvg(fo_xml: string): string;
export function supportedFormats(): string[];

export default function init(
  input?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module
): Promise<InitOutput>;
```

---

## Platform Details

| Aspect | Browser (`web`) | Bundler (`webpack`/`vite`) | Node.js |
|---|---|---|---|
| Import style | `import init, { ... } from '@cooljapan/fop'` | `import { ... } from '@cooljapan/fop'` | `const { ... } = require('@cooljapan/fop')` |
| Initialization | `await init()` required | Handled by bundler | Not required |
| PDF output | `Uint8Array` -> `Blob` -> object URL | `Uint8Array` | `Buffer.from(pdfBytes)` -> `fs.writeFileSync` |
| SVG output | Set `innerHTML` directly | Set `innerHTML` directly | Write to file or respond via HTTP |

### Webpack / Vite Integration

For projects using Webpack 5 or Vite, configure the bundler to handle `.wasm` files:

```javascript
// vite.config.js
export default {
  optimizeDeps: {
    exclude: ['@cooljapan/fop'],
  },
};
```

```javascript
// Dynamic import pattern for lazy loading
const { default: init, FopConverter } = await import('@cooljapan/fop');
await init();
const fop = new FopConverter();
```

---

## Building from Source

### Prerequisites

```bash
# Install wasm-pack
cargo install wasm-pack

# Install the wasm32-unknown-unknown target
rustup target add wasm32-unknown-unknown
```

### Build commands

```bash
# Browser (ES module)
wasm-pack build crates/fop-wasm --target web --out-dir pkg

# Node.js (CommonJS)
wasm-pack build crates/fop-wasm --target nodejs --out-dir pkg-nodejs

# Bundler (for Webpack/Vite)
wasm-pack build crates/fop-wasm --target bundler --out-dir pkg-bundler

# Release build (optimized, smaller WASM binary)
wasm-pack build crates/fop-wasm --target web --release --out-dir pkg
```

After building, the output directory contains:
- `fop_wasm_bg.wasm` -- the compiled WebAssembly binary
- `fop_wasm.js` -- JavaScript glue code
- `fop_wasm.d.ts` -- TypeScript declarations
- `package.json` -- npm package metadata

### Running WASM tests

```bash
# Run tests in a headless browser (requires wasm-pack)
wasm-pack test crates/fop-wasm --headless --chrome

# Run native (non-WASM) unit tests
cargo test -p fop-wasm
```

---

## Feature Flags

| Feature | Default | Description |
|---|---|---|
| *(none)* | -- | The crate has no optional Cargo features. All output formats (PDF, SVG, text) are always available. The `fop-render` dependency is included with `default-features = false` to minimize WASM binary size. |

---

## Crate Architecture

`fop-wasm` is a thin wasm-bindgen adapter layer over the core FOP pipeline:

```
JavaScript / TypeScript
        |
   wasm-bindgen
        |
   fop-wasm (this crate)
        |
   +-----------+----------+----------+
   |           |          |          |
fop-core   fop-layout  fop-render  fop-types
(parse)    (layout)    (PDF/SVG)   (types/errors)
```

The conversion pipeline in each method follows three deterministic steps:

1. **Parse** -- `FoTreeBuilder::new().parse(cursor)` produces an arena-allocated FO tree
2. **Layout** -- `LayoutEngine::new().layout(&arena)` produces an area tree
3. **Render** -- `PdfRenderer` / `SvgRenderer` / `TextRenderer` serializes the area tree to the target format

Because `FoArena` carries a lifetime parameter for property inheritance, the entire parse-layout-render pipeline executes within a single function call, keeping lifetimes on the stack.

---

## Related Crates

| Crate | Description |
|---|---|
| [`fop-types`](https://crates.io/crates/fop-types) | Shared types: `Length`, `Color`, `Rect`, `FopError` |
| [`fop-core`](https://crates.io/crates/fop-core) | XSL-FO document parsing and property system |
| [`fop-layout`](https://crates.io/crates/fop-layout) | Layout engine: block, inline, table, list |
| [`fop-render`](https://crates.io/crates/fop-render) | Rendering backends: PDF, SVG, text |
| [`fop-cli`](https://crates.io/crates/fop-cli) | Command-line interface |
| [`fop-python`](https://crates.io/crates/fop-python) | Python bindings via PyO3 |

---

## Author

COOLJAPAN OU (Team Kitasan)

## Repository

<https://github.com/cool-japan/fop>

## License

Copyright 2024-2026 COOLJAPAN OU (Team Kitasan)

Licensed under the Apache License, Version 2.0. See [LICENSE](https://github.com/cool-japan/fop/blob/main/LICENSE) for details.

```
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
