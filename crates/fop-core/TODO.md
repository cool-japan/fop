# TODO - fop-core

## Parsing
- [x] Handle XML processing instructions (stored in XmlParser::processing_instructions)
- [x] Support CDATA sections in text content
- [x] Handle XML entities (beyond basic &amp; &lt; etc.) including numeric character references
- [x] Add line/column tracking for better error messages
- [x] Support `xml:lang` and `xml:space` attributes

## Property System
- [x] Complete shorthand expansion for all compound properties
- [x] Implement `font` shorthand (font-style, font-variant, font-weight, font-size, font-family)
- [x] Add `background` shorthand expansion
- [x] Implement property validation (range checks, allowed enum values)
- [x] Add initial value defaults for all 294 properties
- [x] Support `inherit` keyword as property value (audit 2026-06-23: verified — inheritance now covers the full ~60 XSL-FO 1.1 inheritable properties; was previously limited to 14)
- [x] Support relative property values (e.g., `font-size="larger"`)
- [x] Support `from-parent()` / `from-nearest-specified-value()` functions (treated as `inherit`)

## FO Tree
- [x] Add `fo:marker` and `fo:retrieve-marker` support (audit 2026-06-23: verified — running headers/footers now resolve per page with all four `retrieve-position` values; was previously returning the same marker on every page)
- [x] Add `fo:footnote` and `fo:footnote-body` support
- [x] Add `fo:float` support
- [x] Add `fo:page-number` inline element
- [x] Add `fo:block-container` block-level element
- [x] Support `fo:declarations` for color profiles
- [x] Support `fo:bookmark-tree` and `fo:bookmark` elements
- [x] Implement `id` attribute tracking for cross-references
- [x] Support `fo:wrapper` transparent property-setting container
- [x] Support `fo:character` single character element
- [x] Support `fo:bidi-override` bidirectional text override
- [x] Support `fo:initial-property-set` first-line properties

## Validation
- [x] Validate required properties per element type
- [x] Validate property value ranges
- [x] Validate mutual exclusion constraints
- [x] Warn on unsupported but valid XSL-FO elements

## Quality
- [x] Remove `#![allow(dead_code)]` when APIs stabilize
- [x] Add fuzzing targets for XML parsing robustness
- [x] Add conformance tests from W3C XSL-FO test suite (75 conformance tests)

## XMP Metadata Embedding

- [x] See root `TODO.md` → **XMP Metadata Embedding (Issue #1 follow-up)** for the full plan (Phase A: `arena.rs` + `builder/mod.rs` capture logic).
