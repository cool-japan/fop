# TODO - fop-render

## PDF Renderer
- [x] Embed PNG images as XObject streams
- [x] Embed JPEG images as XObject streams (DCTDecode)
- [x] Implement PDF annotations for hyperlinks
- [x] Generate PDF outline (bookmarks)
- [x] Support PDF metadata (title, author, creation date)
- [x] Implement background color rendering for areas
- [x] Implement 4-side border rendering with styles
- [x] Support opacity/transparency
- [x] Implement PDF/A compliance mode (compliance.rs, --pdfa flag)
- [x] Add font embedding (TrueType as CIDFont)
- [x] Add font subsetting to reduce file size
- [x] PDF encryption/security (security.rs, -o/-u passwords)

## Graphics
- [x] Dashed and dotted line styles
- [x] Rounded corners on rectangles
- [x] Gradient fills
- [x] Clipping paths for overflow content

## Additional Renderers
- [x] SVG renderer backend (src/svg/)
- [x] PNG/JPEG raster output (src/raster/ via resvg)
- [x] Plain text output (src/text/)
- [x] PostScript output (src/ps/)
- [x] Parallel rendering (src/parallel.rs)

## Quality
- [x] Add visual regression tests (render -> compare pixel output)
- [x] Test PDF output with multiple PDF viewers (gs, pdftotext, pdfinfo)
- [x] Validate generated PDFs with `pdfinfo` / `qpdf --check`
- [x] Benchmark rendering speed for large documents
