# TODO - fop-layout

## Layout Engine
- [x] Support `fo:static-content` for headers and footers
- [x] Handle `space-before` / `space-after` properties on blocks
- [x] Handle `start-indent` / `end-indent` properties
- [x] Implement `text-align` (left, center, right, justify) in inline layout
- [x] Support `text-indent` for first-line indentation
- [x] Support `line-height` property in line box calculation
- [x] Support `letter-spacing` and `word-spacing`

## Table Layout
- [x] Implement `number-columns-spanned` (colspan)
- [x] Implement `number-rows-spanned` (rowspan)
- [x] Support `fo:table-header` and `fo:table-footer` repetition across pages
- [x] Implement auto column width based on content measurement
- [x] Support `table-layout="auto"` vs `table-layout="fixed"`
- [x] Handle `border-collapse` vs `border-separate` models

## List Layout
- [x] Improve marker positioning with baseline alignment
- [x] Support custom markers from `fo:list-item-label` content
- [x] Handle variable-height list items correctly

## Page Breaking
- [x] Implement `break-before` / `break-after` properties
- [x] Implement `keep-together` / `keep-with-next` / `keep-with-previous`
- [x] Implement widow and orphan control
- [x] Support forced page breaks
- [x] Handle page-break within tables (repeating headers)
- [x] Improve `split_area()` to handle inline content splitting

## Advanced Layout
- [x] Float support (`fo:float`, `clear` property)
- [x] Footnote placement (`fo:footnote`)
- [x] Leader rendering (`fo:leader` with dots, rules, space)
- [x] Block container (`fo:block-container`) basic support
- [x] Block containers with absolute positioning (`absolute-position="absolute"` with top/left)
- [x] Multi-column layout (`column-count` property)
- [x] Conditional page masters based on page position (first, last, odd, even)

## Area Tree
- [x] Add area tree serialization for debugging (`AreaTree::serialize()`)
- [x] Implement area tree diffing for regression testing (`AreaTree::diff()`)
- [x] Add area-level property caching for rendering (`margin` field in TraitSet)
