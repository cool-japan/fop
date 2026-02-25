//! FO node factory - creates FoNodeData from parsed XML element names and attributes
//!
//! The `create_node_data` function is the central dispatcher that matches
//! XSL-FO element names to their corresponding `FoNodeData` variants.

use super::property_parser;
use crate::properties::PropertyList;
use crate::tree::{BlankOrNotBlank, FoNodeData, OddOrEven, PagePosition, RetrievePosition};
use crate::{FopError, Result};

/// Create `FoNodeData` from an element name, its parsed attributes, and a pre-allocated
/// property list that has already been populated with common property attributes.
pub(super) fn create_node_data<'a>(
    name: &str,
    attributes: &[(String, String)],
    properties: PropertyList<'a>,
) -> Result<FoNodeData<'a>> {
    match name {
        "root" => Ok(FoNodeData::Root),
        "layout-master-set" => Ok(FoNodeData::LayoutMasterSet),
        "repeatable-page-master-alternatives" => {
            let maximum_repeats = attributes
                .iter()
                .find(|(k, _)| k == "maximum-repeats")
                .and_then(|(_, v)| v.parse::<i32>().ok());

            Ok(FoNodeData::RepeatablePageMasterAlternatives { maximum_repeats })
        }
        "conditional-page-master-reference" => {
            let master_reference = attributes
                .iter()
                .find(|(k, _)| k == "master-reference")
                .map(|(_, v)| v.clone())
                .ok_or_else(|| FopError::MissingAttribute {
                    element: name.to_string(),
                    attribute: "master-reference".to_string(),
                })?;

            // Parse page-position (default: any)
            let page_position = attributes
                .iter()
                .find(|(k, _)| k == "page-position")
                .map(|(_, v)| match v.as_str() {
                    "first" => PagePosition::First,
                    "last" => PagePosition::Last,
                    "rest" => PagePosition::Rest,
                    _ => PagePosition::Any,
                })
                .unwrap_or(PagePosition::Any);

            // Parse odd-or-even (default: any)
            let odd_or_even = attributes
                .iter()
                .find(|(k, _)| k == "odd-or-even")
                .map(|(_, v)| match v.as_str() {
                    "odd" => OddOrEven::Odd,
                    "even" => OddOrEven::Even,
                    _ => OddOrEven::Any,
                })
                .unwrap_or(OddOrEven::Any);

            // Parse blank-or-not-blank (default: any)
            let blank_or_not_blank = attributes
                .iter()
                .find(|(k, _)| k == "blank-or-not-blank")
                .map(|(_, v)| match v.as_str() {
                    "blank" => BlankOrNotBlank::Blank,
                    "not-blank" => BlankOrNotBlank::NotBlank,
                    _ => BlankOrNotBlank::Any,
                })
                .unwrap_or(BlankOrNotBlank::Any);

            Ok(FoNodeData::ConditionalPageMasterReference {
                master_reference,
                page_position,
                odd_or_even,
                blank_or_not_blank,
            })
        }
        "simple-page-master" => {
            let master_name = attributes
                .iter()
                .find(|(k, _)| k == "master-name")
                .map(|(_, v)| v.clone())
                .ok_or_else(|| FopError::MissingAttribute {
                    element: name.to_string(),
                    attribute: "master-name".to_string(),
                })?;

            Ok(FoNodeData::SimplePageMaster {
                master_name,
                properties,
            })
        }
        "region-body" => Ok(FoNodeData::RegionBody { properties }),
        "region-before" => Ok(FoNodeData::RegionBefore { properties }),
        "region-after" => Ok(FoNodeData::RegionAfter { properties }),
        "region-start" => Ok(FoNodeData::RegionStart { properties }),
        "region-end" => Ok(FoNodeData::RegionEnd { properties }),
        "page-sequence" => {
            let master_reference = attributes
                .iter()
                .find(|(k, _)| k == "master-reference")
                .map(|(_, v)| v.clone())
                .ok_or_else(|| FopError::MissingAttribute {
                    element: name.to_string(),
                    attribute: "master-reference".to_string(),
                })?;

            let format = attributes
                .iter()
                .find(|(k, _)| k == "format")
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "1".to_string());

            let grouping_separator = attributes
                .iter()
                .find(|(k, _)| k == "grouping-separator")
                .and_then(|(_, v)| v.chars().next());

            let grouping_size = attributes
                .iter()
                .find(|(k, _)| k == "grouping-size")
                .and_then(|(_, v)| v.parse::<usize>().ok());

            Ok(FoNodeData::PageSequence {
                master_reference,
                format,
                grouping_separator,
                grouping_size,
                properties,
            })
        }
        "flow" => {
            let flow_name = attributes
                .iter()
                .find(|(k, _)| k == "flow-name")
                .map(|(_, v)| v.clone())
                .ok_or_else(|| FopError::MissingAttribute {
                    element: name.to_string(),
                    attribute: "flow-name".to_string(),
                })?;

            Ok(FoNodeData::Flow {
                flow_name,
                properties,
            })
        }
        "static-content" => {
            let flow_name = attributes
                .iter()
                .find(|(k, _)| k == "flow-name")
                .map(|(_, v)| v.clone())
                .ok_or_else(|| FopError::MissingAttribute {
                    element: name.to_string(),
                    attribute: "flow-name".to_string(),
                })?;

            Ok(FoNodeData::StaticContent {
                flow_name,
                properties,
            })
        }
        "block" => Ok(FoNodeData::Block { properties }),
        "inline" => Ok(FoNodeData::Inline { properties }),
        "list-block" => Ok(FoNodeData::ListBlock { properties }),
        "list-item" => Ok(FoNodeData::ListItem { properties }),
        "list-item-label" => Ok(FoNodeData::ListItemLabel { properties }),
        "list-item-body" => Ok(FoNodeData::ListItemBody { properties }),
        "table" => Ok(FoNodeData::Table { properties }),
        "table-column" => Ok(FoNodeData::TableColumn { properties }),
        "table-header" => Ok(FoNodeData::TableHeader { properties }),
        "table-footer" => Ok(FoNodeData::TableFooter { properties }),
        "table-body" => Ok(FoNodeData::TableBody { properties }),
        "table-row" => Ok(FoNodeData::TableRow { properties }),
        "table-cell" => Ok(FoNodeData::TableCell { properties }),
        "external-graphic" => {
            let src = attributes
                .iter()
                .find(|(k, _)| k == "src")
                .map(|(_, v)| v.clone())
                .ok_or_else(|| FopError::MissingAttribute {
                    element: name.to_string(),
                    attribute: "src".to_string(),
                })?;

            let content_width = attributes
                .iter()
                .find(|(k, _)| k == "content-width")
                .map(|(_, v)| v.clone());
            let content_height = attributes
                .iter()
                .find(|(k, _)| k == "content-height")
                .map(|(_, v)| v.clone());
            let scaling = attributes
                .iter()
                .find(|(k, _)| k == "scaling")
                .map(|(_, v)| v.clone());

            Ok(FoNodeData::ExternalGraphic {
                src,
                content_width,
                content_height,
                scaling,
                properties,
            })
        }
        "instream-foreign-object" => {
            let content_width = attributes
                .iter()
                .find(|(k, _)| k == "content-width")
                .map(|(_, v)| v.clone());
            let content_height = attributes
                .iter()
                .find(|(k, _)| k == "content-height")
                .map(|(_, v)| v.clone());
            let scaling = attributes
                .iter()
                .find(|(k, _)| k == "scaling")
                .map(|(_, v)| v.clone());
            Ok(FoNodeData::InstreamForeignObject {
                content_width,
                content_height,
                scaling,
                foreign_xml: String::new(),
                properties,
            })
        }
        "basic-link" => {
            let internal_destination = attributes
                .iter()
                .find(|(k, _)| k == "internal-destination")
                .map(|(_, v)| v.clone());

            let external_destination = attributes
                .iter()
                .find(|(k, _)| k == "external-destination")
                .map(|(_, v)| v.clone());

            Ok(FoNodeData::BasicLink {
                internal_destination,
                external_destination,
                properties,
            })
        }
        "bookmark-tree" => Ok(FoNodeData::BookmarkTree { properties }),
        "bookmark" => {
            let internal_destination = attributes
                .iter()
                .find(|(k, _)| k == "internal-destination")
                .map(|(_, v)| v.clone());

            let external_destination = attributes
                .iter()
                .find(|(k, _)| k == "external-destination")
                .map(|(_, v)| v.clone());

            Ok(FoNodeData::Bookmark {
                internal_destination,
                external_destination,
                properties,
            })
        }
        "bookmark-title" => Ok(FoNodeData::BookmarkTitle { properties }),
        "page-number-citation" => {
            let ref_id = attributes
                .iter()
                .find(|(k, _)| k == "ref-id")
                .map(|(_, v)| v.clone())
                .ok_or_else(|| FopError::MissingAttribute {
                    element: name.to_string(),
                    attribute: "ref-id".to_string(),
                })?;

            Ok(FoNodeData::PageNumberCitation { ref_id, properties })
        }
        "page-number-citation-last" => {
            let ref_id = attributes
                .iter()
                .find(|(k, _)| k == "ref-id")
                .map(|(_, v)| v.clone())
                .unwrap_or_default();

            Ok(FoNodeData::PageNumberCitationLast { ref_id, properties })
        }
        "leader" => Ok(FoNodeData::Leader { properties }),
        "footnote" => Ok(FoNodeData::Footnote { properties }),
        "footnote-body" => Ok(FoNodeData::FootnoteBody { properties }),
        "float" => Ok(FoNodeData::Float { properties }),
        "marker" => {
            let marker_class_name = attributes
                .iter()
                .find(|(k, _)| k == "marker-class-name")
                .map(|(_, v)| v.clone())
                .ok_or_else(|| FopError::MissingAttribute {
                    element: name.to_string(),
                    attribute: "marker-class-name".to_string(),
                })?;

            Ok(FoNodeData::Marker {
                marker_class_name,
                properties,
            })
        }
        "retrieve-marker" => {
            let retrieve_class_name = attributes
                .iter()
                .find(|(k, _)| k == "retrieve-class-name")
                .map(|(_, v)| v.clone())
                .ok_or_else(|| FopError::MissingAttribute {
                    element: name.to_string(),
                    attribute: "retrieve-class-name".to_string(),
                })?;

            // Parse retrieve-position attribute (default: first-starting-within-page)
            let retrieve_position = attributes
                .iter()
                .find(|(k, _)| k == "retrieve-position")
                .map(|(_, v)| match v.as_str() {
                    "first-starting-within-page" => RetrievePosition::FirstStartingWithinPage,
                    "first-including-carryover" => RetrievePosition::FirstIncludingCarryover,
                    "last-starting-within-page" => RetrievePosition::LastStartingWithinPage,
                    "last-ending-within-page" => RetrievePosition::LastEndingWithinPage,
                    _ => RetrievePosition::FirstStartingWithinPage, // default
                })
                .unwrap_or(RetrievePosition::FirstStartingWithinPage);

            Ok(FoNodeData::RetrieveMarker {
                retrieve_class_name,
                retrieve_position,
                properties,
            })
        }
        "bidi-override" => {
            let direction = attributes
                .iter()
                .find(|(k, _)| k == "direction")
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "ltr".to_string());
            Ok(FoNodeData::BidiOverride {
                direction,
                properties,
            })
        }

        "initial-property-set" => Ok(FoNodeData::InitialPropertySet { properties }),

        "wrapper" => Ok(FoNodeData::Wrapper { properties }),

        "page-number" => Ok(FoNodeData::PageNumber { properties }),

        "block-container" => Ok(FoNodeData::BlockContainer { properties }),

        "inline-container" => Ok(FoNodeData::InlineContainer { properties }),

        "table-and-caption" => Ok(FoNodeData::TableAndCaption { properties }),

        "table-caption" => Ok(FoNodeData::TableCaption { properties }),

        "page-sequence-master" => {
            let master_name = attributes
                .iter()
                .find(|(k, _)| k == "master-name")
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            Ok(FoNodeData::PageSequenceMaster { master_name })
        }

        "declarations" => Ok(FoNodeData::Declarations),

        "color-profile" => {
            let src = attributes
                .iter()
                .find(|(k, _)| k == "src")
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            let color_profile_name = attributes
                .iter()
                .find(|(k, _)| k == "color-profile-name")
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            Ok(FoNodeData::ColorProfile {
                src,
                color_profile_name,
            })
        }

        "character" => {
            let char_value = attributes
                .iter()
                .find(|(k, _)| k == "character")
                .and_then(|(_, v)| v.chars().next())
                .unwrap_or('\u{FFFD}'); // replacement character if missing
            Ok(FoNodeData::Character {
                character: char_value,
                properties,
            })
        }

        "multi-switch" => Ok(FoNodeData::MultiSwitch { properties }),

        "multi-case" => {
            let starting_state = attributes
                .iter()
                .find(|(k, _)| k == "starting-state")
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "visible".to_string());
            Ok(FoNodeData::MultiCase {
                starting_state,
                properties,
            })
        }

        "multi-toggle" => Ok(FoNodeData::MultiToggle { properties }),
        "multi-properties" => Ok(FoNodeData::MultiProperties { properties }),
        "multi-property-set" => Ok(FoNodeData::MultiPropertySet { properties }),

        "change-bar-begin" => {
            let change_bar_class = attributes
                .iter()
                .find(|(k, _)| k == "change-bar-class")
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "default".to_string());
            Ok(FoNodeData::ChangeBarBegin {
                change_bar_class,
                properties,
            })
        }

        "change-bar-end" => {
            let change_bar_class = attributes
                .iter()
                .find(|(k, _)| k == "change-bar-class")
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "default".to_string());
            Ok(FoNodeData::ChangeBarEnd { change_bar_class })
        }

        _ => Err(FopError::InvalidElement(format!(
            "Unknown FO element: {}",
            name
        ))),
    }
}

/// Populate a property list from a slice of (key, value) attribute pairs.
///
/// Non-property structural attributes are silently skipped.
pub(super) fn populate_properties<'a>(
    properties: &mut PropertyList<'a>,
    attributes: &[(String, String)],
) -> Result<()> {
    for (key, value) in attributes {
        property_parser::parse_property(properties, key, value)?;
    }
    Ok(())
}
