//! PDF outline (bookmark) serialization
//!
//! Functions to write PDF outline (bookmark) objects into a PDF byte stream.

use super::types::{PdfOutline, PdfOutlineItem};

/// Count total number of outline objects needed (root + all items)
pub(super) fn count_outline_objects(outline: &PdfOutline) -> usize {
    1 + count_outline_items(&outline.items)
}

/// Count outline items recursively
pub(super) fn count_outline_items(items: &[PdfOutlineItem]) -> usize {
    let mut count = items.len();
    for item in items {
        count += count_outline_items(&item.children);
    }
    count
}

/// Write outline objects to PDF bytes
pub(super) fn write_outline_objects(
    outline: &PdfOutline,
    bytes: &mut Vec<u8>,
    xref_offsets: &mut Vec<usize>,
    first_obj_id: usize,
    page_obj_ids: &[usize],
) {
    let mut next_obj_id = first_obj_id;

    // Object 4: Outlines root
    let outlines_root_id = next_obj_id;
    next_obj_id += 1;

    let total_count = count_outline_items(&outline.items);

    // Write outlines root
    xref_offsets.push(bytes.len());
    bytes.extend_from_slice(format!("{} 0 obj\n", outlines_root_id).as_bytes());
    bytes.extend_from_slice(b"<<\n");
    bytes.extend_from_slice(b"/Type /Outlines\n");

    if !outline.items.is_empty() {
        let first_child_id = next_obj_id;
        let last_child_id = next_obj_id + outline.items.len() - 1;
        bytes.extend_from_slice(format!("/First {} 0 R\n", first_child_id).as_bytes());
        bytes.extend_from_slice(format!("/Last {} 0 R\n", last_child_id).as_bytes());
    }

    bytes.extend_from_slice(format!("/Count {}\n", total_count).as_bytes());
    bytes.extend_from_slice(b">>\n");
    bytes.extend_from_slice(b"endobj\n");

    // Write outline items
    if !outline.items.is_empty() {
        write_outline_items(
            &outline.items,
            bytes,
            xref_offsets,
            &mut next_obj_id,
            outlines_root_id,
            page_obj_ids,
        );
    }
}

/// Write outline items recursively
#[allow(clippy::too_many_arguments)]
pub(super) fn write_outline_items(
    items: &[PdfOutlineItem],
    bytes: &mut Vec<u8>,
    xref_offsets: &mut Vec<usize>,
    next_obj_id: &mut usize,
    parent_id: usize,
    page_obj_ids: &[usize],
) {
    let start_obj_id = *next_obj_id;
    let item_count = items.len();

    // Reserve object IDs for all items at this level
    let end_obj_id = start_obj_id + item_count;

    for (idx, item) in items.iter().enumerate() {
        let obj_id = start_obj_id + idx;
        *next_obj_id = obj_id + 1;

        xref_offsets.push(bytes.len());
        bytes.extend_from_slice(format!("{} 0 obj\n", obj_id).as_bytes());
        bytes.extend_from_slice(b"<<\n");

        // Escape title for PDF string
        let escaped_title = escape_pdf_string(&item.title);
        bytes.extend_from_slice(format!("/Title ({})\n", escaped_title).as_bytes());
        bytes.extend_from_slice(format!("/Parent {} 0 R\n", parent_id).as_bytes());

        // Add prev/next pointers
        if idx > 0 {
            bytes.extend_from_slice(format!("/Prev {} 0 R\n", obj_id - 1).as_bytes());
        }
        if idx < item_count - 1 {
            bytes.extend_from_slice(format!("/Next {} 0 R\n", obj_id + 1).as_bytes());
        }

        // Add destination
        if let Some(page_idx) = item.page_index {
            if page_idx < page_obj_ids.len() {
                let page_obj_id = page_obj_ids[page_idx];
                bytes.extend_from_slice(
                    format!("/Dest [{} 0 R /XYZ 0 792 0]\n", page_obj_id).as_bytes(),
                );
            }
        }

        // Add children if present
        if !item.children.is_empty() {
            let first_child_id = end_obj_id + count_child_offset(items, idx);
            let child_count = item.children.len();
            let last_child_id = first_child_id + child_count - 1;

            bytes.extend_from_slice(format!("/First {} 0 R\n", first_child_id).as_bytes());
            bytes.extend_from_slice(format!("/Last {} 0 R\n", last_child_id).as_bytes());
            bytes.extend_from_slice(format!("/Count {}\n", child_count).as_bytes());
        }

        bytes.extend_from_slice(b">>\n");
        bytes.extend_from_slice(b"endobj\n");
    }

    // Now write all children
    *next_obj_id = end_obj_id;
    for (idx, item) in items.iter().enumerate() {
        if !item.children.is_empty() {
            let parent = start_obj_id + idx;
            write_outline_items(
                &item.children,
                bytes,
                xref_offsets,
                next_obj_id,
                parent,
                page_obj_ids,
            );
        }
    }
}

/// Calculate offset to first child of an item
fn count_child_offset(items: &[PdfOutlineItem], current_idx: usize) -> usize {
    let mut offset = 0;
    for item in items.iter().take(current_idx) {
        offset += count_outline_items(&item.children);
    }
    offset
}

/// Escape special characters in PDF strings
pub(super) fn escape_pdf_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}
