//! Example demonstrating element nesting validation

use fop_core::{FoNodeData, NestingValidator, PropertyList};

fn main() {
    println!("=== FO Element Nesting Validation ===\n");

    // Valid nesting examples
    println!("Valid nestings:");

    let root = FoNodeData::Root;
    let layout = FoNodeData::LayoutMasterSet;
    match NestingValidator::can_contain(&root, &layout) {
        Ok(_) => println!("  ✓ root can contain layout-master-set"),
        Err(e) => println!("  ✗ {}", e),
    }

    let block = FoNodeData::Block {
        properties: PropertyList::new(),
    };
    let text = FoNodeData::Text(String::from("Hello"));
    match NestingValidator::can_contain(&block, &text) {
        Ok(_) => println!("  ✓ block can contain text"),
        Err(e) => println!("  ✗ {}", e),
    }

    let inline = FoNodeData::Inline {
        properties: PropertyList::new(),
    };
    match NestingValidator::can_contain(&block, &inline) {
        Ok(_) => println!("  ✓ block can contain inline"),
        Err(e) => println!("  ✗ {}", e),
    }

    // Invalid nesting examples
    println!("\nInvalid nestings:");

    let block2 = FoNodeData::Block {
        properties: PropertyList::new(),
    };
    match NestingValidator::can_contain(&root, &block2) {
        Ok(_) => println!("  ✓ root can contain block (unexpected!)"),
        Err(e) => println!("  ✗ root cannot contain block: {}", e),
    }

    let table = FoNodeData::Table {
        properties: PropertyList::new(),
    };
    let table_row = FoNodeData::TableRow {
        properties: PropertyList::new(),
    };
    match NestingValidator::can_contain(&table, &table_row) {
        Ok(_) => println!("  ✓ table can contain table-row (unexpected!)"),
        Err(e) => println!("  ✗ table cannot contain table-row directly: {}", e),
    }

    // Complex structure validation
    println!("\nList structure validation:");

    let list_block = FoNodeData::ListBlock {
        properties: PropertyList::new(),
    };
    let list_item = FoNodeData::ListItem {
        properties: PropertyList::new(),
    };
    let list_label = FoNodeData::ListItemLabel {
        properties: PropertyList::new(),
    };

    match NestingValidator::can_contain(&list_block, &list_item) {
        Ok(_) => println!("  ✓ list-block can contain list-item"),
        Err(e) => println!("  ✗ {}", e),
    }

    match NestingValidator::can_contain(&list_item, &list_label) {
        Ok(_) => println!("  ✓ list-item can contain list-item-label"),
        Err(e) => println!("  ✗ {}", e),
    }

    let block3 = FoNodeData::Block {
        properties: PropertyList::new(),
    };
    match NestingValidator::can_contain(&list_label, &block3) {
        Ok(_) => println!("  ✓ list-item-label can contain block"),
        Err(e) => println!("  ✗ {}", e),
    }

    match NestingValidator::can_contain(&list_block, &block3) {
        Ok(_) => println!("  ✓ list-block can contain block (unexpected!)"),
        Err(e) => println!("  ✗ list-block cannot contain block directly: {}", e),
    }
}
