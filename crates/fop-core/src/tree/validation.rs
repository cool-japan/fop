//! Element nesting validation
//!
//! Validates that FO elements are nested according to XSL-FO 1.1 specification rules.

use crate::tree::FoNodeData;
use crate::{FopError, Result};

/// Validator for FO element nesting
pub struct NestingValidator;

impl NestingValidator {
    /// Check if a child element can be nested inside a parent element
    pub fn can_contain(parent: &FoNodeData, child: &FoNodeData) -> Result<()> {
        let parent_name = parent.element_name();
        let child_name = child.element_name();

        let allowed = match parent_name {
            "root" => matches!(
                child_name,
                "layout-master-set" | "declarations" | "page-sequence"
            ),
            "layout-master-set" => {
                matches!(child_name, "simple-page-master" | "page-sequence-master")
            }
            "simple-page-master" => matches!(
                child_name,
                "region-body" | "region-before" | "region-after" | "region-start" | "region-end"
            ),
            "page-sequence" => matches!(child_name, "flow" | "static-content"),
            "flow" | "static-content" => {
                Self::is_block_level(child_name) || child_name == "multi-switch"
            }
            "block" => {
                Self::is_block_or_inline(child_name)
                    || child_name == "#text"
                    || child_name == "multi-switch"
                    || child_name == "multi-toggle"
            }
            "inline" => Self::is_inline_level(child_name) || child_name == "#text",
            "list-block" => child_name == "list-item",
            "list-item" => matches!(child_name, "list-item-label" | "list-item-body"),
            "list-item-label" | "list-item-body" => Self::is_block_level(child_name),
            "table" => matches!(
                child_name,
                "table-column" | "table-header" | "table-footer" | "table-body"
            ),
            "table-header" | "table-footer" | "table-body" => child_name == "table-row",
            "table-row" => child_name == "table-cell",
            "table-cell" => Self::is_block_level(child_name),
            "inline-container" => Self::is_block_level(child_name) || child_name == "#text",
            "table-and-caption" => matches!(child_name, "table" | "table-caption"),
            "table-caption" => Self::is_block_level(child_name),
            "basic-link" => Self::is_inline_level(child_name) || child_name == "#text",
            "multi-switch" => child_name == "multi-case",
            "multi-case" => {
                Self::is_block_or_inline(child_name)
                    || child_name == "#text"
                    || child_name == "multi-toggle"
            }
            "multi-toggle" => Self::is_inline_level(child_name) || child_name == "#text",
            "multi-properties" => child_name == "multi-property-set",
            "region-body" | "region-before" | "region-after" | "region-start" | "region-end" => {
                false // Regions don't contain children directly
            }
            _ => true, // Unknown elements, allow for now
        };

        if allowed {
            Ok(())
        } else {
            Err(FopError::InvalidNesting {
                parent: parent_name.to_string(),
                child: child_name.to_string(),
            })
        }
    }

    /// Check if element name is block-level
    fn is_block_level(name: &str) -> bool {
        matches!(
            name,
            "block" | "block-container" | "list-block" | "table" | "table-and-caption"
        )
    }

    /// Check if element name is inline-level
    fn is_inline_level(name: &str) -> bool {
        matches!(
            name,
            "inline"
                | "inline-container"
                | "external-graphic"
                | "instream-foreign-object"
                | "basic-link"
                | "character"
                | "page-number"
                | "page-number-citation"
        )
    }

    /// Check if element can be block or inline
    fn is_block_or_inline(name: &str) -> bool {
        Self::is_block_level(name) || Self::is_inline_level(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PropertyList;

    #[test]
    fn test_valid_root_children() {
        let root = FoNodeData::Root;
        let layout = FoNodeData::LayoutMasterSet;
        let page_seq = FoNodeData::PageSequence {
            master_reference: String::from("test"),
            format: "1".to_string(),
            grouping_separator: None,
            grouping_size: None,
            properties: PropertyList::new(),
        };

        assert!(NestingValidator::can_contain(&root, &layout).is_ok());
        assert!(NestingValidator::can_contain(&root, &page_seq).is_ok());
    }

    #[test]
    fn test_invalid_root_child() {
        let root = FoNodeData::Root;
        let block = FoNodeData::Block {
            properties: PropertyList::new(),
        };

        assert!(NestingValidator::can_contain(&root, &block).is_err());
    }

    #[test]
    fn test_block_can_contain_text() {
        let block = FoNodeData::Block {
            properties: PropertyList::new(),
        };
        let text = FoNodeData::Text(String::from("Hello"));

        assert!(NestingValidator::can_contain(&block, &text).is_ok());
    }

    #[test]
    fn test_block_can_contain_inline() {
        let block = FoNodeData::Block {
            properties: PropertyList::new(),
        };
        let inline = FoNodeData::Inline {
            properties: PropertyList::new(),
        };

        assert!(NestingValidator::can_contain(&block, &inline).is_ok());
    }

    #[test]
    fn test_list_structure() {
        let list_block = FoNodeData::ListBlock {
            properties: PropertyList::new(),
        };
        let list_item = FoNodeData::ListItem {
            properties: PropertyList::new(),
        };
        let list_label = FoNodeData::ListItemLabel {
            properties: PropertyList::new(),
        };
        let block = FoNodeData::Block {
            properties: PropertyList::new(),
        };

        // list-block can contain list-item
        assert!(NestingValidator::can_contain(&list_block, &list_item).is_ok());

        // list-item can contain list-item-label
        assert!(NestingValidator::can_contain(&list_item, &list_label).is_ok());

        // list-item-label can contain block
        assert!(NestingValidator::can_contain(&list_label, &block).is_ok());

        // list-block cannot contain block directly
        assert!(NestingValidator::can_contain(&list_block, &block).is_err());
    }

    #[test]
    fn test_table_structure() {
        let table = FoNodeData::Table {
            properties: PropertyList::new(),
        };
        let table_body = FoNodeData::TableBody {
            properties: PropertyList::new(),
        };
        let table_row = FoNodeData::TableRow {
            properties: PropertyList::new(),
        };
        let table_cell = FoNodeData::TableCell {
            properties: PropertyList::new(),
        };
        let block = FoNodeData::Block {
            properties: PropertyList::new(),
        };

        // Valid nesting
        assert!(NestingValidator::can_contain(&table, &table_body).is_ok());
        assert!(NestingValidator::can_contain(&table_body, &table_row).is_ok());
        assert!(NestingValidator::can_contain(&table_row, &table_cell).is_ok());
        assert!(NestingValidator::can_contain(&table_cell, &block).is_ok());

        // Invalid nesting
        assert!(NestingValidator::can_contain(&table, &table_row).is_err());
        assert!(NestingValidator::can_contain(&table, &block).is_err());
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;
    use crate::PropertyList;

    fn props() -> PropertyList<'static> {
        PropertyList::new()
    }

    fn block() -> FoNodeData<'static> {
        FoNodeData::Block {
            properties: props(),
        }
    }

    fn inline_node() -> FoNodeData<'static> {
        FoNodeData::Inline {
            properties: props(),
        }
    }

    fn text() -> FoNodeData<'static> {
        FoNodeData::Text("hello".to_string())
    }

    // -----------------------------------------------------------------------
    // root element nesting
    // -----------------------------------------------------------------------

    #[test]
    fn test_root_can_contain_declarations() {
        assert!(
            NestingValidator::can_contain(&FoNodeData::Root, &FoNodeData::Declarations).is_ok()
        );
    }

    #[test]
    fn test_root_cannot_contain_flow() {
        let flow = FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: props(),
        };
        assert!(NestingValidator::can_contain(&FoNodeData::Root, &flow).is_err());
    }

    #[test]
    fn test_root_cannot_contain_inline() {
        assert!(NestingValidator::can_contain(&FoNodeData::Root, &inline_node()).is_err());
    }

    // -----------------------------------------------------------------------
    // layout-master-set
    // -----------------------------------------------------------------------

    #[test]
    fn test_layout_master_set_can_contain_simple_page_master() {
        let lms = FoNodeData::LayoutMasterSet;
        let spm = FoNodeData::SimplePageMaster {
            master_name: "A4".to_string(),
            properties: props(),
        };
        assert!(NestingValidator::can_contain(&lms, &spm).is_ok());
    }

    #[test]
    fn test_layout_master_set_can_contain_page_sequence_master() {
        let lms = FoNodeData::LayoutMasterSet;
        let psm = FoNodeData::PageSequenceMaster {
            master_name: "alternating".to_string(),
        };
        assert!(NestingValidator::can_contain(&lms, &psm).is_ok());
    }

    #[test]
    fn test_layout_master_set_cannot_contain_block() {
        let lms = FoNodeData::LayoutMasterSet;
        assert!(NestingValidator::can_contain(&lms, &block()).is_err());
    }

    // -----------------------------------------------------------------------
    // simple-page-master regions
    // -----------------------------------------------------------------------

    #[test]
    fn test_simple_page_master_can_contain_all_regions() {
        let spm = FoNodeData::SimplePageMaster {
            master_name: "A4".to_string(),
            properties: props(),
        };
        let regions = vec![
            FoNodeData::RegionBody {
                properties: props(),
            },
            FoNodeData::RegionBefore {
                properties: props(),
            },
            FoNodeData::RegionAfter {
                properties: props(),
            },
            FoNodeData::RegionStart {
                properties: props(),
            },
            FoNodeData::RegionEnd {
                properties: props(),
            },
        ];
        for region in &regions {
            assert!(NestingValidator::can_contain(&spm, region).is_ok());
        }
    }

    #[test]
    fn test_simple_page_master_cannot_contain_block() {
        let spm = FoNodeData::SimplePageMaster {
            master_name: "A4".to_string(),
            properties: props(),
        };
        assert!(NestingValidator::can_contain(&spm, &block()).is_err());
    }

    // -----------------------------------------------------------------------
    // page-sequence
    // -----------------------------------------------------------------------

    #[test]
    fn test_page_sequence_can_contain_flow_and_static() {
        let ps = FoNodeData::PageSequence {
            master_reference: "A4".to_string(),
            format: "1".to_string(),
            grouping_separator: None,
            grouping_size: None,
            properties: props(),
        };
        let flow = FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: props(),
        };
        let sc = FoNodeData::StaticContent {
            flow_name: "xsl-region-before".to_string(),
            properties: props(),
        };
        assert!(NestingValidator::can_contain(&ps, &flow).is_ok());
        assert!(NestingValidator::can_contain(&ps, &sc).is_ok());
    }

    #[test]
    fn test_page_sequence_cannot_contain_block() {
        let ps = FoNodeData::PageSequence {
            master_reference: "A4".to_string(),
            format: "1".to_string(),
            grouping_separator: None,
            grouping_size: None,
            properties: props(),
        };
        assert!(NestingValidator::can_contain(&ps, &block()).is_err());
    }

    // -----------------------------------------------------------------------
    // flow / static-content
    // -----------------------------------------------------------------------

    #[test]
    fn test_flow_can_contain_block_level_elements() {
        let flow = FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: props(),
        };
        let list_block = FoNodeData::ListBlock {
            properties: props(),
        };
        let table = FoNodeData::Table {
            properties: props(),
        };
        assert!(NestingValidator::can_contain(&flow, &block()).is_ok());
        assert!(NestingValidator::can_contain(&flow, &list_block).is_ok());
        assert!(NestingValidator::can_contain(&flow, &table).is_ok());
    }

    #[test]
    fn test_flow_cannot_contain_text_directly() {
        let flow = FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: props(),
        };
        assert!(NestingValidator::can_contain(&flow, &text()).is_err());
    }

    // -----------------------------------------------------------------------
    // block
    // -----------------------------------------------------------------------

    #[test]
    fn test_block_can_contain_block_container() {
        let block_container = FoNodeData::BlockContainer {
            properties: props(),
        };
        assert!(NestingValidator::can_contain(&block(), &block_container).is_ok());
    }

    #[test]
    fn test_block_can_contain_nested_block() {
        assert!(NestingValidator::can_contain(&block(), &block()).is_ok());
    }

    // -----------------------------------------------------------------------
    // inline
    // -----------------------------------------------------------------------

    #[test]
    fn test_inline_can_contain_inline() {
        assert!(NestingValidator::can_contain(&inline_node(), &inline_node()).is_ok());
    }

    #[test]
    fn test_inline_can_contain_text() {
        assert!(NestingValidator::can_contain(&inline_node(), &text()).is_ok());
    }

    #[test]
    fn test_inline_cannot_contain_block() {
        assert!(NestingValidator::can_contain(&inline_node(), &block()).is_err());
    }

    // -----------------------------------------------------------------------
    // table structure
    // -----------------------------------------------------------------------

    #[test]
    fn test_table_can_contain_header_and_footer() {
        let table = FoNodeData::Table {
            properties: props(),
        };
        assert!(NestingValidator::can_contain(
            &table,
            &FoNodeData::TableHeader {
                properties: props()
            }
        )
        .is_ok());
        assert!(NestingValidator::can_contain(
            &table,
            &FoNodeData::TableFooter {
                properties: props()
            }
        )
        .is_ok());
        assert!(NestingValidator::can_contain(
            &table,
            &FoNodeData::TableColumn {
                properties: props()
            }
        )
        .is_ok());
    }

    #[test]
    fn test_table_cannot_contain_table_cell_directly() {
        let table = FoNodeData::Table {
            properties: props(),
        };
        assert!(NestingValidator::can_contain(
            &table,
            &FoNodeData::TableCell {
                properties: props()
            }
        )
        .is_err());
    }

    // -----------------------------------------------------------------------
    // basic-link
    // -----------------------------------------------------------------------

    #[test]
    fn test_basic_link_can_contain_inline() {
        let link = FoNodeData::BasicLink {
            internal_destination: None,
            external_destination: Some("http://x.com".to_string()),
            properties: props(),
        };
        assert!(NestingValidator::can_contain(&link, &inline_node()).is_ok());
        assert!(NestingValidator::can_contain(&link, &text()).is_ok());
    }

    #[test]
    fn test_basic_link_cannot_contain_block() {
        let link = FoNodeData::BasicLink {
            internal_destination: None,
            external_destination: Some("http://x.com".to_string()),
            properties: props(),
        };
        assert!(NestingValidator::can_contain(&link, &block()).is_err());
    }

    // -----------------------------------------------------------------------
    // multi-switch / multi-case
    // -----------------------------------------------------------------------

    #[test]
    fn test_multi_switch_can_contain_multi_case() {
        let ms = FoNodeData::MultiSwitch {
            properties: props(),
        };
        let mc = FoNodeData::MultiCase {
            starting_state: "visible".to_string(),
            properties: props(),
        };
        assert!(NestingValidator::can_contain(&ms, &mc).is_ok());
    }

    #[test]
    fn test_multi_switch_cannot_contain_block() {
        let ms = FoNodeData::MultiSwitch {
            properties: props(),
        };
        assert!(NestingValidator::can_contain(&ms, &block()).is_err());
    }

    #[test]
    fn test_multi_case_can_contain_block_and_inline() {
        let mc = FoNodeData::MultiCase {
            starting_state: "visible".to_string(),
            properties: props(),
        };
        assert!(NestingValidator::can_contain(&mc, &block()).is_ok());
        assert!(NestingValidator::can_contain(&mc, &inline_node()).is_ok());
        assert!(NestingValidator::can_contain(&mc, &text()).is_ok());
    }

    // -----------------------------------------------------------------------
    // regions do not contain children
    // -----------------------------------------------------------------------

    #[test]
    fn test_regions_cannot_contain_children() {
        let regions: Vec<FoNodeData> = vec![
            FoNodeData::RegionBody {
                properties: props(),
            },
            FoNodeData::RegionBefore {
                properties: props(),
            },
            FoNodeData::RegionAfter {
                properties: props(),
            },
            FoNodeData::RegionStart {
                properties: props(),
            },
            FoNodeData::RegionEnd {
                properties: props(),
            },
        ];
        for region in &regions {
            assert!(NestingValidator::can_contain(region, &block()).is_err());
            assert!(NestingValidator::can_contain(region, &inline_node()).is_err());
        }
    }

    // -----------------------------------------------------------------------
    // inline-container
    // -----------------------------------------------------------------------

    #[test]
    fn test_inline_container_can_contain_block() {
        let ic = FoNodeData::InlineContainer {
            properties: props(),
        };
        assert!(NestingValidator::can_contain(&ic, &block()).is_ok());
        assert!(NestingValidator::can_contain(&ic, &text()).is_ok());
    }

    // -----------------------------------------------------------------------
    // table-and-caption
    // -----------------------------------------------------------------------

    #[test]
    fn test_table_and_caption_structure() {
        let tac = FoNodeData::TableAndCaption {
            properties: props(),
        };
        let caption = FoNodeData::TableCaption {
            properties: props(),
        };
        let table = FoNodeData::Table {
            properties: props(),
        };
        assert!(NestingValidator::can_contain(&tac, &caption).is_ok());
        assert!(NestingValidator::can_contain(&tac, &table).is_ok());
        assert!(NestingValidator::can_contain(&tac, &block()).is_err());
    }
}
