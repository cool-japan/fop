# TODO - Apache FOP Rust

## Audit Findings (2026-06-23) — Discovered Gaps (fabrication audit, branch 0.1.3)

A deep audit (beyond the loud-marker scan) found genuine gaps the checklists below
over-claim as done. Build/tests/clippy are green, but these are real correctness /
fidelity / security defects. This is the live convergence target for this session.
`[sonnet]` / `[opus]` = implementation model routing.

### Milestone 1 — Contained fixes (file-isolated, low cascade risk)
- [x] [sonnet] AES-256 encryption uses deterministic SHA-derived salts/IVs → use an OS CSPRNG (`getrandom`, pure Rust; not `rand`) (`fop-render/src/pdf/security.rs:308-316,626-633`)
- [x] [sonnet] ParallelRenderer builds an empty font cache → custom/embedded fonts silently become Helvetica under `--jobs N` (`fop-render/src/parallel.rs:45-53`)
- [x] [sonnet] PNG indexed-color decoded as grayscale (index-as-gray) → wrong colors; use `png` EXPAND or palette lookup (`fop-pdf-renderer/src/image.rs:115-118`)
- [x] [sonnet] PostScript `add_image` draws a gray box, never reads pixel data → emit real Level-2 `image`/`colorimage` (`fop-render/src/ps/mod.rs:499-533`)
- [x] [sonnet] `font-size="1.5em"` stored as Percentage but `extract_font_size` ignores it → silently 12pt; add a Percentage arm resolving vs parent (`fop-core/.../builder/property_parser.rs` + `fop-layout/.../properties/extraction.rs`)
- [x] [sonnet] FontRegistry's 14 standard fonts share 5 metrics objects → bold/italic use regular widths; add real AFM bold/oblique metrics (`fop-types/src/font_metrics.rs:724-756`)
- [x] [sonnet] Inheritance registry lists only 14 of ~60 inheritable XSL-FO properties → add the rest from XSL-FO 1.1 §B (`fop-core/src/properties/property_list.rs:13-36`)
- [x] [sonnet] Block height ignores `line-height` (always = font-size) → read `traits.line_height` (`fop-layout/.../engine/block_layout.rs:60`)
- [x] [sonnet] Table outer-area height hardcoded 100pt, never updated → sum row heights after `layout_table` (`fop-layout/.../engine/mod.rs:520-549`)
- [x] [opus] Font subsetting is a no-op (returns full font on both branches) → real TTF subsetting (rebuild glyf/loca/cmap/head/hhea/hmtx/maxp + checksums, or a pure-Rust subsetter) (`fop-render/src/pdf/font.rs:616-644`)
- [x] [opus] PDF clip paths `W`/`W*` discarded (`path.clear()` "skip for now") → apply tiny-skia clip mask, restore on `Q` (`fop-pdf-renderer/src/content.rs:641-643` + rasterizer)
- [x] [sonnet] PDF/UA-1 emits `/Marked true` + empty `/StructTreeRoot` → falsely claims compliance; make honest (error or stop emitting false markers) until real tagging exists (`fop-render/src/pdf/document/mod.rs:375-382`)

### Milestone 2 — Layout architecture (LARGE, interdependent; rewrites area-tree geometry → cascades across the 3063-test suite. Do carefully/sequentially; hand off if budget runs out)
- [x] [opus] `layout_block` emits ONE full-width area per text run — no word-wrap; wire real line-breaking (`fop-layout/.../engine/block_layout.rs:107-144`)
- [x] [opus] Knuth-Plass breaker ignores glue stretch/shrink (greedy, not optimal) AND is never called by the engine — implement true adjustment-ratio/demerit model + wire it (`fop-layout/src/layout/knuth_plass.rs`)
- [x] [opus] `PageBreaker::break_into_pages` never reparents areas (page containers stay empty) and is never invoked — real pagination + overflow detection (`fop-layout/src/layout/page_break.rs:65-161`)
- [x] [opus] Multi-column overflow silently continues in last column instead of paginating (`fop-layout/.../engine/block_layout.rs:362-374`)
- [x] [opus] `table-layout="auto"` builds an empty grid — cell content never measured; equal-width result (`fop-layout/.../engine/mod.rs:502-518`)
- [x] [opus] `fo:marker` collection hardcodes `(starts_on_page=true, ends_on_page=true)` → running headers wrong on multi-chapter docs (`fop-layout/.../engine/page_layout.rs:845-848`)

### Docs / honesty
- [x] Correct the over-claiming `[x]` entries (font subsetting, Knuth-Plass) in this file, per-crate `TODO.md`, and `CHANGELOG.md` so they reflect reality.

### Deferred refinements (surfaced WHILE fixing Milestone 2 — beyond the original audit; honest next layer, NOT regressions)
- [ ] [opus] Block splitting across a page boundary: a block taller than the region-body stays whole (clips) instead of splitting; `split_area`/`count_line_areas` retained for it.
- [ ] [opus] Widow/orphan control wired into the page-break decision (governs splitting; traits recorded but not yet enforced during pagination).
- [ ] [sonnet] `fo:retrieve-marker` nested inside a block within `fo:static-content` (only direct children of static-content currently resolve a marker; pre-existing scope limit).
- [ ] [opus] Newspaper-style multi-column balancing (cross-page multi-column currently fills sequentially — correct but unbalanced).
- [ ] [sonnet] Conditional page-master selection (first/last/odd/even `conditional-page-master-reference`) — selection logic exists but is dormant in the live path.
- [ ] [opus] Streaming engine (`streaming.rs`) lacks the multi-column + pagination parity the main engine now has.
- [ ] [sonnet] Non-BMP (surrogate-pair) text: PDF `W`/`CIDToGIDMap` keyed by full code point is inconsistent for astral characters (pre-existing).
- [ ] [sonnet] `pagination.rs` ~1930 lines — split its integration tests into a sibling module before further growth (2000-line policy).

---

## Phase 5: Advanced Features

### Image Rendering
- [x] Add `png` crate dependency for PNG decoding
- [x] Add `jpeg-decoder` crate dependency for JPEG decoding
- [x] Implement PDF XObject image embedding
- [x] Wire `fo:external-graphic` through rendering pipeline
- [x] Support width/height/content-width/content-height properties
- [x] Test with various image sizes and formats

### Background Colors and Borders
- [x] Render background-color on `fo:block` areas
- [x] Render background-color on `fo:table-cell` areas
- [x] Implement 4-side border rendering (top/right/bottom/left)
- [x] Support border styles: solid, dashed, dotted
- [x] Wire `PdfGraphics.draw_borders()` into area rendering

### Links and Bookmarks
- [x] Implement internal links (`fo:basic-link` with `internal-destination`)
- [x] Implement external links (`fo:basic-link` with `external-destination`)
- [x] Generate PDF annotations for links
- [x] Implement PDF outline (bookmarks) from `fo:bookmark-tree`
- [x] Support `fo:page-number-citation`

### Font Embedding
- [x] Add `ttf-parser` crate for TrueType/OpenType parsing
- [x] Implement font metrics extraction from TTF files
- [x] Implement font subsetting (embed only used glyphs)
- [x] Add Courier, Symbol, ZapfDingbats to built-in fonts
- [x] Wire custom fonts through the layout pipeline
- [x] **Unicode font support** - Extended from ASCII (32-126) to full Unicode (u32)
- [x] **ToUnicode CMap generation** - Proper PDF Unicode character mapping
- [x] **Smart text encoding** - Hex strings for Unicode, literal for ASCII
- [x] **Font selection from FO properties** - Read font-family and use embedded fonts
- [x] **Font configuration system** - Map font names to font files (font_config.rs)
- [x] **System font discovery** - Automatic font loading from system

## Phase 6: Optimization and Polish

### Performance
- [x] Profile hot paths with `criterion` benchmarks
- [x] Implement streaming mode for large documents (>1000 pages)
- [x] Optimize memory usage in area tree
- [x] Consider parallel rendering for multi-page documents (ParallelRenderer in parallel.rs)

### Standards Compliance
- [x] PDF/A compliance for archival output (compliance.rs, --pdfa CLI flag)
- [x] Validate against XSL-FO 1.1 conformance test suite (60+ conformance tests covering all major XSL-FO 1.1 features)
- [x] Handle all shorthand property expansions (margin, padding, border, font, background - all implemented with 80+ tests)
- [x] Complete property initial value defaults (all 295 properties have initial values verified)

### Additional Output Formats
- [x] SVG renderer backend (fop-render/src/svg/)
- [x] PNG/JPEG raster output (fop-render/src/raster/ via resvg)
- [x] Plain text output (fop-render/src/text/)

### PDF Renderer (Pure Rust - Self-Verification)
**Goal:** Replace `pdftoppm` dependency with pure Rust PDF-to-image renderer

**Motivation:**
- Enable self-verification without C++ dependencies (poppler)
- Complete the cycle: Generate PDF → Render PDF → Verify output
- Pure Rust stack (no external tools needed)
- WASM-compatible for browser-based testing
- Better integration testing capabilities

**New Crate:** `fop-pdf-renderer`
```
fop/
└── crates/
    └── fop-pdf-renderer/
        ├── src/
        │   ├── lib.rs         - Public API
        │   ├── parser.rs      - PDF structure parser
        │   ├── content.rs     - Content stream interpreter
        │   ├── graphics.rs    - Graphics state machine
        │   ├── text.rs        - Text rendering
        │   ├── font.rs        - Font handling (TrueType, Type 0, CID)
        │   ├── image.rs       - Image XObject handling
        │   └── rasterizer.rs  - Pixel output (via tiny-skia or similar)
        └── Cargo.toml
```

**Implementation Tasks:**
- [x] **Phase 1: PDF Parser**
  - [x] Parse PDF structure (objects, xref table, trailer)
  - [x] Extract page tree
  - [x] Parse content streams
  - [x] Handle compressed streams (FlateDecode)

- [x] **Phase 2: Graphics Engine**
  - [x] Implement PDF graphics state machine
  - [x] Path construction and stroking
  - [x] Color spaces (DeviceRGB, DeviceGray)
  - [x] Transformation matrices (CTM)

- [x] **Phase 3: Text Rendering**
  - [x] Text state operators (Tf, Tm, Td, Tj, TJ)
  - [x] Font loading (TrueType from embedded streams)
  - [x] Glyph rasterization
  - [x] UTF-16BE text decoding
  - [x] CID font support (Type 0, CIDFontType2)
  - [x] ToUnicode CMap parsing

- [x] **Phase 4: Rasterization**
  - [x] Render to RGBA bitmap (tiny-skia)
  - [x] PNG encoding (png crate)
  - [x] DPI/scale support
  - [x] Anti-aliasing

- [x] **Phase 5: Integration**
  - [x] Public API: `PdfRenderer::from_bytes(pdf_data)`
  - [x] Public API: `renderer.render_page(page_num, dpi) -> RasterPage`
  - [x] Public API: `renderer.save_as_png(page, path)`
  - [x] CLI tool: `fop-render-pdf input.pdf output.png` (src/bin/fop_render_pdf.rs)
  - [x] Integration with auto-verify workflow
    - [x] Replace external-tool tests in `regression_tests.rs` with pure-Rust verifications
    - [x] Expand `verify_tests.rs` fixture coverage from 4 → 17 fixtures
    - [x] Add `--render-verify` CLI flag wired to `fop-pdf-renderer`
    - [x] Add `.github/workflows/ci.yml` running `fmt`/`clippy`/`nextest`/`doc` on Linux+macOS+Windows without poppler/gs
    - [x] Add `PdfRenderer::extract_text(page_index) -> Result<String>` to `fop-pdf-renderer` (prerequisite for regression tests)
- [x] **Phase 6: Glyph Outline Rendering** (completed 2026-04-20)
  - [x] Create `glyph.rs` with `OutlinePathBuilder` (ttf-parser → tiny-skia bridge) and standard-14 substitute font discovery with TTC index support
  - [x] Parse `/CIDToGIDMap` binary stream in `font.rs` (replaces `HashMap::new()` placeholder); add `SimpleEncoding` enum + WinAnsi/Standard tables
  - [x] Wire `PageRasterizer.font_cache` with `LoadedFontExt` struct; add `base_font` field to `LoadedFont`; remove `#[allow(dead_code)]`
  - [x] Replace filled-rectangle placeholder at `rasterizer.rs:160-178` with real ttf-parser outline fill via tiny-skia

**Dependencies (Pure Rust):**
- `pdf` or `lopdf` - PDF parsing (evaluate both)
- `tiny-skia` or `raqote` - 2D graphics rasterization
- `ttf-parser` - TrueType font parsing (already used)
- `png` - PNG encoding (already used)
- `image` - Image manipulation (already used)

**Benefits:**
- ✅ Auto-verification without `pdftoppm` (no C++ dependency)
- ✅ Cross-platform (Windows, macOS, Linux, WASM)
- ✅ Consistent rendering across platforms
- ✅ Better error messages (know our own format)
- ✅ Faster CI/CD (no external tool installation)
- ✅ Self-contained testing infrastructure

**Target Usage:**
```rust
use fop_pdf_renderer::PdfRenderer;

// Load PDF
let pdf_data = std::fs::read("output.pdf")?;
let renderer = PdfRenderer::from_bytes(&pdf_data)?;

// Render to image
let image = renderer.render_page(0, 150)?; // page 0, 150 DPI

// Save as PNG
renderer.save_as_png(0, "output.png")?;

// Auto-verification
assert!(image.contains_text("請求書"));
assert!(!image.contains_glyph(".notdef"));
```

**Estimated Effort:** 2-3 weeks for basic rendering, 4-6 weeks for production quality

**Priority:** Medium (useful but not blocking current PDF generation work)

### Advanced Layout
- [x] Headers and footers via `fo:static-content`
- [x] Page masters with multiple regions (SidebarStart/SidebarEnd AreaType)
- [x] Table column/row spanning (`number-columns-spanned`, `number-rows-spanned`)
- [x] Widow/orphan control (fully implemented with page breaking constraints, 8 comprehensive tests)
- [x] Keep-together / keep-with-next / keep-with-previous
- [x] Float support (`fo:float`) (FloatManager in layout engine)
- [x] Footnotes (`fo:footnote`) (inline reference marks + page-bottom body)
- [x] Leaders (`fo:leader`)

## Technical Debt
- [x] Remove `#![allow(dead_code)]` from `fop-core` when APIs stabilize (removed from lib.rs, targeted annotations added where needed)
- [x] Add `#[must_use]` annotations where appropriate (Result types are already #[must_use])
- [x] Increase test coverage for edge cases in property inheritance (added 14 comprehensive edge case tests covering: explicit inherit keyword, percentages, enums, lists, deep chains, border properties, caching, 4-level inheritance, partial overrides)
- [x] Add integration tests with real-world XSL-FO documents (4 new tests: invoice, report, form, Unicode)
- [x] Add fuzzing targets for XML parsing (fuzz_xml_parser and fuzz_property_parser)
- [x] Update `README.md` with accurate test count, file/LoC statistics, and new feature documentation (`--render-verify` CLI flag, `PdfRenderer::extract_text` API, `fop-pdf-renderer` crate, `fop-render-pdf` binary, `.github/workflows/ci.yml` CI workflow) (completed 2026-04-20)

## Version 0.1.1 Fixes and Enhancements (completed 2026-04-20)
- [x] **PyO3 0.28 migration** — updated `fop-python` to PyO3 0.28 and ported to the new `Python::attach` API (replaces `Python::with_gil`)
- [x] **macOS build.rs linker fix** — added `build.rs` to `fop-python` to resolve PyO3 ABI3 linker issues on macOS (`-undefined dynamic_lookup` flag)
- [x] **fop-wasm invalid XML test fixes** — corrected WASM binding error-handling tests to match updated error message format for malformed XML inputs

## Version 0.1.2 Enhancements (last updated 2026-05-16)
- [x] **XMP metadata embedding from `<fo:declarations>`** — capture `<x:xmpmeta>` packets, write `/Metadata` stream in PDF catalog, sync `/Info` dictionary from Dublin Core fields (completed 2026-05-14)
- [x] **Namespace scope stack in `XmlParser`** — replaced flat `namespace_map` with a proper `Vec<NamespaceScope>` scope stack; captures inherited `xmlns:` prefixes and injects only used ones into captured XMP/SVG root elements; `Event::CData`/`Event::Comment` round-trip in both capture buffers (completed 2026-05-15)
- [x] **`/Info` PDF string value escaping in `PdfDocument::to_bytes()`** — applied `escape_pdf_string` to `/Title`, `/Author`, `/Subject`, `/CreationDate` in the trailer write path (completed 2026-05-16)
- [x] **Extended `dc:date`/`dc:rights`/`dc:language` extraction from XMP** — added three fields to `DcFields` in `compliance.rs`, extraction logic, and unit tests (completed 2026-05-16)
- [x] **SimpleDocumentBuilder XMP support** — wired `<x:xmpmeta>` packets, full `/Info` metadata surface, Dublin-Core → `/Info` sync, deterministic `/ID` trailer, and `/Catalog /Metadata` stream into `SimpleDocumentBuilder` (completed 2026-05-16)

## XMP Metadata Embedding (Issue #1 follow-up)

- [x] XMP metadata embedding — capture `<x:xmpmeta>` from `fo:declarations`, write the PDF `/Metadata` stream, and sync the `/Info` dictionary (issue #1 follow-up) (completed 2026-05-14)
  - **Goal:** An FO document whose `<fo:declarations>` contains an `<x:xmpmeta>` RDF/XML packet produces a PDF in which (a) the catalog has a `/Metadata N 0 R` reference, (b) object N is a `/Type /Metadata /Subtype /XML` stream containing the source XMP packet (xpacket-wrapped), and (c) the `/Info` dictionary's `/Title`, `/Author`, `/Subject` are auto-populated from the packet's Dublin Core fields. The issue #1 reporter's exact FO round-trips: `extract_xmp_metadata()` returns the packet, `extract_text()` still returns "Hello.", `page_count() == 1`.
  - **Design:**
    - **Phase A — capture (`fop-core`).**
      - `arena.rs`: add `pub xmp_packets: Vec<String>` to `FoArena`, initialised in both `new()` and `with_capacity()`. *`Vec` (not `Option`) chosen consciously:* the builder's event loop just `push`es as it finalises each packet — no "already captured?" guard in the hot path — and the consumer takes `.first()`. Captures all, uses first; honest, and preserves info for a future multi-packet decision.
      - `builder/mod.rs`: extend the existing non-FO branch (the issue #1 `non_fo_depth` machinery). Add field `xmp_buffer: Option<String>`. When a non-FO `Event::Start` arrives with `non_fo_depth == 0`, `current_node` is a `FoNodeData::Declarations`, and the element local-name is `xmpmeta`: start a buffer. While `xmp_buffer.is_some()`, reconstruct **every** non-FO `Start`/`Empty`/`End`/`Text`/`CData` event into the buffer. When the matching `End` brings `non_fo_depth` back to 0, push the buffer to `arena.xmp_packets`. Check `foreign_object_node` first, then `xmp_buffer`, then the plain `non_fo_depth` skip — mutually exclusive in practice.
    - **Phase B — write `/Metadata` stream (`fop-render`).**
      - `document/types.rs`: add `pub xmp_metadata: Option<String>` to `PdfDocument`; init `None` in `new()`.
      - `document/mod.rs`: add `set_xmp_metadata(&mut self, xmp: String)`. In `to_bytes()`: compute `let needs_xmp = needs_compliance || self.xmp_metadata.is_some();`. Change xmp_obj_count gate, catalog ref, and stream-emit block to use `needs_xmp`. Stream content: `match &self.xmp_metadata { Some(src) => reconcile_xmp(src, self.compliance), None => generate_xmp_metadata(...) }`.
      - `compliance.rs`: add `reconcile_xmp(source: &str, compliance: PdfCompliance) -> String` and `extract_dc_fields(xmp: &str) -> DcFields`.
    - **Phase B.5 — sync `/Info` from Dublin Core (`fop-render`).** Bridge `fo_tree.xmp_packets.first()` → `doc.set_xmp_metadata(...)` in `render_with_fo()`, then fill any unset `doc.info.title/author/subject` from `extract_dc_fields`. Do not overwrite values already set.
    - **Phase C — extract + round-trip (`fop-pdf-renderer`).**
      - `parser.rs`: add `pub fn get_metadata_stream(&self) -> Option<Vec<u8>>` — `find_catalog` → catalog `/Metadata` ref → `decode_stream(obj_num)`.
      - `lib.rs`: add `pub fn extract_xmp_metadata(&self) -> Option<String>` delegating to it via `String::from_utf8(...).ok()`.
  - **Files:**
    - `crates/fop-core/src/tree/arena.rs` — `xmp_packets` field
    - `crates/fop-core/src/tree/builder/mod.rs` — capture logic in the non-FO branch
    - `crates/fop-render/src/pdf/document/types.rs` — `PdfDocument.xmp_metadata` field
    - `crates/fop-render/src/pdf/document/mod.rs` — `set_xmp_metadata()`, `to_bytes()` ID chain + catalog ref + stream emit
    - `crates/fop-render/src/pdf/compliance.rs` — `reconcile_xmp()`, `extract_dc_fields()`
    - `crates/fop-render/src/pdf/writer.rs` — `render_with_fo()` bridge + `/Info` sync
    - `crates/fop-pdf-renderer/src/parser.rs` — `get_metadata_stream()`
    - `crates/fop-pdf-renderer/src/lib.rs` — `extract_xmp_metadata()`
    - `tests/integration/regression_tests.rs` — round-trip regression test
    - `TODO.md` (root) — this plan block; `crates/fop-core/TODO.md` + `crates/fop-render/TODO.md` — one-line back-reference
  - **Prerequisites:** none external. Phases are strictly sequential within one subagent.
  - **Tests:**
    - `fop-core` unit: `test_xmp_packet_captured_from_declarations`
    - `fop-render` unit: `test_pdf_document_writes_metadata_stream`, `test_reconcile_xmp_standard_wraps_xpacket`, `test_reconcile_xmp_pdfa_splices_identifiers`, `test_extract_dc_fields`
    - integration: `test_issue_1_xmp_metadata_roundtrip`
    - Full workspace: `cargo nextest run --all-features` — 2828+ tests green; clippy clean.
  - **Risk:**
    - Object-ID chain fragility: switching xmp gate from `needs_compliance` to `needs_xmp` shifts later IDs. Mitigation: pdf-renderer parses via xref (renumber-robust); full regression test suite catches off-by-one.
    - Namespace self-containment assumption: XMP subtree must declare its own xmlns prefixes.

## Proposed follow-ups

- [x] SimpleDocumentBuilder XMP support — wire `<x:xmpmeta>` packets, full `/Info` metadata surface (author / subject / creation_date / lang), Dublin-Core → `/Info` sync, deterministic `/ID` trailer array, and `/Catalog /Metadata` stream into **both** the fast path (`PdfDocument::to_bytes()` delegation) and the slow path (`write_minimal_pdf`) of `SimpleDocumentBuilder` (done 2026-05-16)
- [x] XMP namespace-inheritance hardening — proper namespace scope stack in `XmlParser`; capture-time injection of *only the in-scope `xmlns:` prefixes actually used by the subtree* into the captured root open tag; same fix applied to `instream-foreign-object` capture; round-trip `Event::CData` / `Event::Comment` in both capture buffers (planned 2026-05-15)
  - **Goal:** When an FO document declares `xmlns:x`, `xmlns:rdf`, `xmlns:dc` on `<fo:root>` (the standard XSL-FO authoring style) and then writes `<x:xmpmeta>…</x:xmpmeta>` inside `<fo:declarations>` without redeclaring those prefixes locally, the captured packet stored in `FoArena.xmp_packets[0]` is **standalone-well-formed RDF/XML** (no undefined prefixes, parseable by a strict namespace-aware reader, all in-scope `xmlns:` decls that the subtree actually references appear on the captured `<x:xmpmeta>` root element verbatim). The same guarantee holds for `instream-foreign-object` (SVG inside `fo:instream-foreign-object` survives ancestor `xmlns:svg`). CData sections and comments inside either capture round-trip byte-for-byte. The PDF `/Metadata` stream a user gets from `extract_xmp_metadata()` is therefore parseable by any conforming RDF/XML reader. No regression in the 3024 existing tests.
  - **Design:**
    - **Phase 1 — proper namespace scope stack (`fop-core/src/xml/parser.rs`).**
      - Replace `namespace_map: HashMap<String, String>` with `namespace_stack: Vec<NamespaceScope>` where `NamespaceScope { decls: Vec<(String, String)> }` (prefix, uri pairs declared on one element's open tag). Always push (even an empty frame) so pop is symmetric.
      - Add `push_namespace_scope(&mut self, start: &BytesStart)` — parse `xmlns`/`xmlns:*` attrs, push new scope.
      - Add `pop_namespace_scope(&mut self)` — soft pop (no panic on underflow).
      - Add `resolve_prefix(&self, prefix: &str) -> Option<&str>` — scan stack top-down, return first match.
      - Add `snapshot_in_scope(&self) -> Vec<(String, String)>` — fold bottom-up (innermost wins), sort by prefix, return owned Vec.
      - Update `extract_name()` to use `resolve_prefix` instead of `namespace_map.get`.
      - Delete `update_namespaces()` — replaced by loop-top push in the builder.
    - **Phase 2 — push/pop integration in the builder (`fop-core/src/tree/builder/mod.rs`).**
      - At the very top of the parse loop: on `Event::Start` push before dispatch; on `Event::End` pop after dispatch; on `Event::Empty` push+dispatch+pop.
      - Remove the three legacy `parser.update_namespaces(start)` calls (Phase 1 deletes the method so these would fail to compile anyway).
    - **Phase 3 — capture-time prefix tracking + injection (`builder/mod.rs` + new `builder/xmlns.rs`).**
      - New file `xmlns.rs` (~120 lines): pure helpers `extract_prefix`, `scan_prefixes_used`, `declared_on_element`, `render_xmlns_attrs`, `inject_namespace_decls`. Each has unit tests.
      - Promote `xmp_buffer: Option<(String, usize)>` to `xmp_buffer: Option<CaptureNs>` with fields: `buffer`, `depth`, `root_close_byte`, `in_scope_at_start`, `declared_on_root`, `used_in_subtree`. Parallel `foreign_object_capture: Option<CaptureNs>`.
      - Capture-start: record root open tag bytes, `root_close_byte`, snapshot in-scope namespaces, declare-on-root set, seed used-prefixes.
      - Capture-body: add `Event::CData` and `Event::Comment` handling (previously dropped via `_ => {}`); update `used_in_subtree` on every Start/Empty.
      - Capture-finalise: compute `to_inject = used − declared_on_root`, look up URIs from `in_scope_at_start`, splice via `inject_namespace_decls` at `root_close_byte`, push patched packet.
    - **Phase 4 — tests.**
      - `test_xmp_namespace_inheritance_captures_inherited_xmlns` — `xmlns:x/rdf/dc` on `<fo:root>` only; assert injected on captured root.
      - `test_xmp_well_formed_via_ns_reader` — feed captured packet to `quick_xml::NsReader`; any `ResolveResult::Unknown` fails.
      - `test_namespace_scope_pop_restores_outer` and `test_namespace_scope_sibling_rebind_does_not_leak` (parser unit).
      - `test_foreign_object_inherits_xmlns_svg` — SVG with inherited `xmlns:svg`.
      - `test_xmp_capture_round_trips_cdata` and `test_xmp_capture_round_trips_comment`.
      - `test_xmlns_inject_self_closing_root` (xmlns helper unit).
      - `test_issue_1_namespace_inheritance_pdf_roundtrip` (integration).
  - **Files:**
    - `crates/fop-core/src/xml/parser.rs` — `NamespaceScope` struct; replace `namespace_map`; new scope methods; update `extract_name`; delete `update_namespaces`.
    - `crates/fop-core/src/tree/builder/mod.rs` — loop-top push/pop; remove 3 legacy calls; promote capture state to `CaptureNs`; CData/Comment append; finalize injection.
    - `crates/fop-core/src/tree/builder/xmlns.rs` (NEW) — pure helpers + unit tests.
    - `crates/fop-core/src/tree/builder.rs` or `mod.rs` — `mod xmlns;` line.
    - `tests/integration/regression_tests.rs` — `test_issue_1_namespace_inheritance_pdf_roundtrip`.
  - **Tests:** see Phase 4. Acceptance: `test_xmp_well_formed_via_ns_reader` passes for the inherited-xmlns shape; 3024 existing tests stay green.
  - **Risk:**
    - *Behavioural change in `extract_name`.* Strictly lexical scope now — correctness improvement but a test accidentally relying on the old leak would fail. Full 3024-test sweep is the mitigation.
    - *Push/pop symmetry.* `expand_empty_elements=true` should make `Event::Empty` unreachable; guard it defensively anyway.
    - *Capture-start timing.* Push happens loop-top before dispatch, so the root's own `xmlns:` decls are already on the stack when snapshot is taken at capture-start.
    - *Sibling-prefix-rebinding leak.* Old flat `namespace_map` silently mutated; `test_namespace_scope_sibling_rebind_does_not_leak` is the new guard.
- [x] Escape `/Info` string values in `PdfDocument::to_bytes()` (`crates/fop-render/src/pdf/document/mod.rs`) — applied `escape_pdf_string` to `/Title`, `/Author`, `/Subject`, `/CreationDate` in the trailer write path; added `test_info_escapes_parentheses_in_title` (done 2026-05-16).
- [x] Extend `extract_dc_fields` (`crates/fop-render/src/pdf/compliance.rs`) to extract `dc:date`, `dc:rights`, `dc:language` — added three fields to `DcFields`, extraction calls, and unit tests (done 2026-05-16).
- [ ] Refactor `write_minimal_pdf` away by teaching `PdfDocument::to_bytes()` to handle multiple Type1 builtin fonts — removes ~120 lines of duplicated serialisation in `simple.rs`; touches embedded-font/image/gradient object-ID arithmetic in `document/mod.rs`, meaningful regression surface; defer until there is a second motivation beyond aesthetics.
