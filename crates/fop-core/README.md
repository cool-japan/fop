# fop-core

XSL-FO document parser and property system for the Apache FOP Rust implementation. This crate reads XSL-FO XML and produces a typed, arena-allocated FO tree with property inheritance.

## Features

- **294 XSL-FO 1.1 properties** as a compile-time enum (`PropertyId`)
- **Property inheritance** with parent chain lookup and caching
- **Arena-allocated FO tree** with index-based handles (no `Rc<RefCell<>>`)
- **SAX-like streaming parser** built on `quick-xml`
- **29 FO element types** covering pages, blocks, tables, lists, graphics
- **Shorthand expansion** for margin, padding, border shorthands
- **Nesting validation** per XSL-FO 1.1 specification rules

## Usage

```rust
use fop_core::FoTreeBuilder;
use std::io::Cursor;

let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4"
            page-width="210mm" page-height="297mm">
            <fo:region-body margin="1in"/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block font-size="14pt">Hello, FOP!</fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;

let builder = FoTreeBuilder::new();
let arena = builder.parse(Cursor::new(xml)).unwrap();

for (id, node) in arena.iter() {
    println!("{}: {}", id, node.data.element_name());
}
```

## Architecture

### Modules

#### `properties/` - Property System
| File | Lines | Description |
|------|------:|-------------|
| `property_id.rs` | 1,210 | `PropertyId` enum with 294 XSL-FO properties |
| `property_value.rs` | 177 | `PropertyValue` enum (Length, Color, Enum, String...) |
| `property_list.rs` | 275 | `PropertyList` with inheritance and caching |
| `shorthand.rs` | 268 | Shorthand expansion (margin -> 4 sides) |

#### `tree/` - FO Tree
| File | Lines | Description |
|------|------:|-------------|
| `arena.rs` | 276 | `FoArena` index-based allocator |
| `node.rs` | 327 | `FoNode`, `FoNodeData` (29 element variants) |
| `builder.rs` | 426 | `FoTreeBuilder` SAX-like parser |
| `validation.rs` | 195 | `NestingValidator` spec compliance |

#### `xml/` - XML Parsing
| File | Lines | Description |
|------|------:|-------------|
| `parser.rs` | 227 | `XmlParser` wrapper around quick-xml |
| `namespace.rs` | 80 | XSL-FO, FOX, SVG namespace handling |

#### `elements/` - Element Definitions
| File | Lines | Description |
|------|------:|-------------|
| `table.rs` | 284 | Table element structures |
| `list.rs` | 284 | List element structures |

**Total: 4,157 lines across 16 files**

### Supported FO Elements (29 types)

**Page Layout:** Root, LayoutMasterSet, SimplePageMaster, RegionBody, RegionBefore, RegionAfter

**Page Sequence:** PageSequence, Flow, StaticContent

**Block Level:** Block, BlockContainer

**Inline Level:** Inline, Character, PageNumber, PageNumberCitation

**Table:** Table, TableColumn, TableHeader, TableFooter, TableBody, TableRow, TableCell

**List:** ListBlock, ListItem, ListItemLabel, ListItemBody

**Other:** ExternalGraphic, BasicLink, Leader, Wrapper

### Property Inheritance

Properties are looked up through a parent chain. Inheritable properties (like `font-size`, `color`) automatically propagate from parent to child:

```rust
use fop_core::{PropertyId, PropertyValue, PropertyList};
use fop_types::Length;

let mut parent = PropertyList::new();
parent.set(PropertyId::FontSize, PropertyValue::Length(Length::from_pt(14.0)));

let child = PropertyList::with_parent(&parent);
// child inherits font-size=14pt from parent
let size = child.get(PropertyId::FontSize).unwrap();
```

## Tests

41 unit tests covering:
- Property ID name/number mapping
- Property value parsing (lengths, colors, enums)
- Property inheritance chains
- Shorthand expansion (margin, padding, border)
- XSL-FO document parsing (simple and complex)
- Element nesting validation
- XML namespace handling
- Arena allocator operations

## Dependencies

- `fop-types` (internal)
- `quick-xml` 0.39 - XML parsing
- `thiserror` 2.0 - Error handling
- `log` 0.4 - Logging
