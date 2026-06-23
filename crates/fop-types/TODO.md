# TODO - fop-types

## Planned
- [x] Add `Percentage` type for percentage-based values (e.g., `width="50%"`)
- [x] Add `Expression` type for `calc()` expressions in property values
- [x] Add more named colors (full CSS/SVG color keyword list)
- [x] Add `from_em()` and `from_ex()` to `Length` (via `LengthUnit::Em/Ex` with `FontContext`) (audit 2026-06-23: verified — em values now resolve against the parent font-size; was previously silently rendering at 12 pt regardless)
- [x] Extend `FontRegistry` with Courier, Symbol, ZapfDingbats metrics
- [x] TrueType/OpenType font metrics loading (implemented in fop-render/src/pdf/font.rs via ttf-parser)
- [x] Add `#[must_use]` annotations on `Length`, `Color`, `Rect` methods
- [x] `Display` trait implementations for all types (Length, Color, Rect, Point, Size)
