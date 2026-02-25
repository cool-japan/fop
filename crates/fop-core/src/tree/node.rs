//! FO tree node data structures

use super::arena::NodeId;
use crate::properties::PropertyList;

/// A node in the FO tree
pub struct FoNode<'a> {
    /// The FO data for this node
    pub data: FoNodeData<'a>,

    /// Optional element ID for cross-references and linking
    /// Per XSL-FO spec section 5.2.2, the "id" property uniquely identifies an element
    pub id: Option<String>,

    /// Parent node ID (None for root)
    pub parent: Option<NodeId>,

    /// First child node ID
    pub first_child: Option<NodeId>,

    /// Next sibling node ID
    pub next_sibling: Option<NodeId>,
}

impl<'a> FoNode<'a> {
    /// Create a new node with the given data
    pub fn new(data: FoNodeData<'a>) -> Self {
        Self {
            data,
            id: None,
            parent: None,
            first_child: None,
            next_sibling: None,
        }
    }

    /// Create a new node with the given data and ID
    pub fn new_with_id(data: FoNodeData<'a>, id: Option<String>) -> Self {
        Self {
            data,
            id,
            parent: None,
            first_child: None,
            next_sibling: None,
        }
    }

    /// Get the element ID if present
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// Set the element ID
    pub fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }

    /// Check if this node has children
    #[inline]
    pub fn has_children(&self) -> bool {
        self.first_child.is_some()
    }

    /// Check if this node is a root node
    #[inline]
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }
}

/// FO node data - the actual formatting object information
pub enum FoNodeData<'a> {
    /// fo:root - document root
    Root,

    /// fo:layout-master-set - page layout definitions
    LayoutMasterSet,

    /// fo:simple-page-master - single page layout
    SimplePageMaster {
        master_name: String,
        properties: PropertyList<'a>,
    },

    /// fo:repeatable-page-master-alternatives - conditional page master selection
    RepeatablePageMasterAlternatives {
        /// Maximum number of times this can be used (None = unlimited)
        maximum_repeats: Option<i32>,
    },

    /// fo:conditional-page-master-reference - conditional reference to page master
    ConditionalPageMasterReference {
        /// Reference to the simple-page-master to use
        master_reference: String,
        /// Page position constraint (first, last, rest, any)
        page_position: PagePosition,
        /// Odd or even constraint (odd, even, any)
        odd_or_even: OddOrEven,
        /// Blank or not blank constraint (blank, not-blank, any)
        blank_or_not_blank: BlankOrNotBlank,
    },

    /// fo:region-body - main content region
    RegionBody { properties: PropertyList<'a> },

    /// fo:region-before - header region
    RegionBefore { properties: PropertyList<'a> },

    /// fo:region-after - footer region
    RegionAfter { properties: PropertyList<'a> },

    /// fo:region-start - left sidebar region
    RegionStart { properties: PropertyList<'a> },

    /// fo:region-end - right sidebar region
    RegionEnd { properties: PropertyList<'a> },

    /// fo:page-sequence - sequence of pages
    PageSequence {
        master_reference: String,
        format: String,
        grouping_separator: Option<char>,
        grouping_size: Option<usize>,
        properties: PropertyList<'a>,
    },

    /// fo:flow - flowing content
    Flow {
        flow_name: String,
        properties: PropertyList<'a>,
    },

    /// fo:static-content - static content (headers/footers)
    StaticContent {
        flow_name: String,
        properties: PropertyList<'a>,
    },

    /// fo:block - block-level element
    Block { properties: PropertyList<'a> },

    /// fo:inline - inline-level element
    Inline { properties: PropertyList<'a> },

    /// fo:list-block - list container
    ListBlock { properties: PropertyList<'a> },

    /// fo:list-item - list item
    ListItem { properties: PropertyList<'a> },

    /// fo:list-item-label - list item label (bullet/number)
    ListItemLabel { properties: PropertyList<'a> },

    /// fo:list-item-body - list item content
    ListItemBody { properties: PropertyList<'a> },

    /// fo:table - table container
    Table { properties: PropertyList<'a> },

    /// fo:table-column - table column definition
    TableColumn { properties: PropertyList<'a> },

    /// fo:table-header - table header
    TableHeader { properties: PropertyList<'a> },

    /// fo:table-footer - table footer
    TableFooter { properties: PropertyList<'a> },

    /// fo:table-body - table body
    TableBody { properties: PropertyList<'a> },

    /// fo:table-row - table row
    TableRow { properties: PropertyList<'a> },

    /// fo:table-cell - table cell
    TableCell { properties: PropertyList<'a> },

    /// fo:external-graphic - image reference
    ExternalGraphic {
        src: String,
        /// content-width: explicit length, "scale-to-fit", "scale-down-to-fit", "scale-up-to-fit", or "auto"
        content_width: Option<String>,
        /// content-height: explicit length, "scale-to-fit", "scale-down-to-fit", "scale-up-to-fit", or "auto"
        content_height: Option<String>,
        /// scaling: "uniform" or "non-uniform"
        scaling: Option<String>,
        properties: PropertyList<'a>,
    },

    /// fo:instream-foreign-object - embedded content (SVG, etc.)
    InstreamForeignObject {
        content_width: Option<String>,
        content_height: Option<String>,
        scaling: Option<String>,
        foreign_xml: String,
        properties: PropertyList<'a>,
    },

    /// fo:basic-link - hyperlink
    BasicLink {
        internal_destination: Option<String>,
        external_destination: Option<String>,
        properties: PropertyList<'a>,
    },

    /// fo:bookmark-tree - document outline root
    BookmarkTree { properties: PropertyList<'a> },

    /// fo:bookmark - bookmark entry
    Bookmark {
        internal_destination: Option<String>,
        external_destination: Option<String>,
        properties: PropertyList<'a>,
    },

    /// fo:bookmark-title - bookmark title text
    BookmarkTitle { properties: PropertyList<'a> },

    /// fo:page-number-citation - reference to page number of an element
    PageNumberCitation {
        ref_id: String,
        properties: PropertyList<'a>,
    },

    /// fo:page-number-citation-last - reference to last page of an element (Section 8.10)
    PageNumberCitationLast {
        ref_id: String,
        properties: PropertyList<'a>,
    },

    /// fo:leader - visual separator (dots, lines, etc.)
    Leader { properties: PropertyList<'a> },

    /// fo:marker - content for running headers/footers
    Marker {
        marker_class_name: String,
        properties: PropertyList<'a>,
    },

    /// fo:retrieve-marker - retrieves marker content
    RetrieveMarker {
        retrieve_class_name: String,
        retrieve_position: RetrievePosition,
        properties: PropertyList<'a>,
    },

    /// fo:footnote - footnote container
    Footnote { properties: PropertyList<'a> },

    /// fo:footnote-body - footnote content
    FootnoteBody { properties: PropertyList<'a> },

    /// fo:float - floating element (like CSS floats)
    Float { properties: PropertyList<'a> },

    /// fo:page-number - inline element inserting the current page number
    PageNumber { properties: PropertyList<'a> },

    /// fo:block-container - block-level container with absolute/fixed positioning
    BlockContainer { properties: PropertyList<'a> },

    /// fo:inline-container - inline-level block container (Section 8.13)
    InlineContainer { properties: PropertyList<'a> },

    /// fo:table-and-caption - table with an associated caption (Section 9.3.3)
    TableAndCaption { properties: PropertyList<'a> },

    /// fo:table-caption - caption for a table (Section 9.4)
    TableCaption { properties: PropertyList<'a> },

    /// fo:page-sequence-master - sequence of page master alternatives
    PageSequenceMaster { master_name: String },

    /// fo:declarations - document-level declarations (color profiles, etc.)
    Declarations,

    /// fo:color-profile - ICC color profile declaration
    ColorProfile {
        src: String,
        color_profile_name: String,
    },

    /// Placeholder for recognized but unsupported XSL-FO elements
    UnsupportedElement {
        /// The element name, for diagnostic purposes
        element_name: String,
    },

    /// fo:multi-switch - container for multiple alternatives (Section 11.1)
    MultiSwitch { properties: PropertyList<'a> },

    /// fo:multi-case - one alternative within a multi-switch (Section 11.2)
    MultiCase {
        /// Whether this case is visible ("visible" or "hidden")
        starting_state: String,
        properties: PropertyList<'a>,
    },

    /// fo:multi-toggle - interactive trigger within multi-case (Section 11.3)
    MultiToggle { properties: PropertyList<'a> },

    /// fo:multi-properties - property-switching element (Section 11.4)
    MultiProperties { properties: PropertyList<'a> },

    /// fo:multi-property-set - one set of properties within multi-properties (Section 11.5)
    MultiPropertySet { properties: PropertyList<'a> },

    /// fo:wrapper - transparent property-setting container (Section 8.12)
    Wrapper { properties: PropertyList<'a> },

    /// fo:character - single character with full property control (Section 8.5)
    Character {
        /// The character to render
        character: char,
        properties: PropertyList<'a>,
    },

    /// fo:bidi-override - bidirectional text override (Section 8.11)
    BidiOverride {
        /// Direction: "ltr" or "rtl"
        direction: String,
        properties: PropertyList<'a>,
    },

    /// fo:initial-property-set - first line property overrides (Section 8.6)
    InitialPropertySet { properties: PropertyList<'a> },

    /// fo:change-bar-begin - marks start of changed region (Section 12.1)
    ChangeBarBegin {
        /// Unique identifier linking begin/end pair
        change_bar_class: String,
        properties: PropertyList<'a>,
    },

    /// fo:change-bar-end - marks end of changed region (Section 12.2)
    ChangeBarEnd {
        /// Unique identifier linking begin/end pair
        change_bar_class: String,
    },

    /// Text content
    Text(String),
}

/// Position for retrieving markers from the page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrievePosition {
    /// First marker starting on the current page
    FirstStartingWithinPage,
    /// First marker including those carried over from previous pages
    FirstIncludingCarryover,
    /// Last marker starting on the current page
    LastStartingWithinPage,
    /// Last marker ending on the current page
    LastEndingWithinPage,
}

/// Page position for conditional page master selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PagePosition {
    /// First page in the page sequence
    First,
    /// Last page in the page sequence
    Last,
    /// Any page except first and last
    Rest,
    /// Any page (default)
    Any,
}

/// Odd or even page for conditional page master selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OddOrEven {
    /// Odd-numbered pages
    Odd,
    /// Even-numbered pages
    Even,
    /// Any page (default)
    Any,
}

/// Blank or not blank page for conditional page master selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlankOrNotBlank {
    /// Blank pages (pages with no content)
    Blank,
    /// Non-blank pages (pages with content)
    NotBlank,
    /// Any page (default)
    Any,
}

impl<'a> FoNodeData<'a> {
    /// Get the element name for this node
    pub fn element_name(&self) -> &str {
        match self {
            FoNodeData::Root => "root",
            FoNodeData::LayoutMasterSet => "layout-master-set",
            FoNodeData::SimplePageMaster { .. } => "simple-page-master",
            FoNodeData::RepeatablePageMasterAlternatives { .. } => {
                "repeatable-page-master-alternatives"
            }
            FoNodeData::ConditionalPageMasterReference { .. } => {
                "conditional-page-master-reference"
            }
            FoNodeData::RegionBody { .. } => "region-body",
            FoNodeData::RegionBefore { .. } => "region-before",
            FoNodeData::RegionAfter { .. } => "region-after",
            FoNodeData::RegionStart { .. } => "region-start",
            FoNodeData::RegionEnd { .. } => "region-end",
            FoNodeData::PageSequence { .. } => "page-sequence",
            FoNodeData::Flow { .. } => "flow",
            FoNodeData::StaticContent { .. } => "static-content",
            FoNodeData::Block { .. } => "block",
            FoNodeData::Inline { .. } => "inline",
            FoNodeData::ListBlock { .. } => "list-block",
            FoNodeData::ListItem { .. } => "list-item",
            FoNodeData::ListItemLabel { .. } => "list-item-label",
            FoNodeData::ListItemBody { .. } => "list-item-body",
            FoNodeData::Table { .. } => "table",
            FoNodeData::TableColumn { .. } => "table-column",
            FoNodeData::TableHeader { .. } => "table-header",
            FoNodeData::TableFooter { .. } => "table-footer",
            FoNodeData::TableBody { .. } => "table-body",
            FoNodeData::TableRow { .. } => "table-row",
            FoNodeData::TableCell { .. } => "table-cell",
            FoNodeData::ExternalGraphic { .. } => "external-graphic",
            FoNodeData::InstreamForeignObject { .. } => "instream-foreign-object",
            FoNodeData::BasicLink { .. } => "basic-link",
            FoNodeData::BookmarkTree { .. } => "bookmark-tree",
            FoNodeData::Bookmark { .. } => "bookmark",
            FoNodeData::BookmarkTitle { .. } => "bookmark-title",
            FoNodeData::PageNumberCitation { .. } => "page-number-citation",
            FoNodeData::PageNumberCitationLast { .. } => "page-number-citation-last",
            FoNodeData::Leader { .. } => "leader",
            FoNodeData::Marker { .. } => "marker",
            FoNodeData::RetrieveMarker { .. } => "retrieve-marker",
            FoNodeData::Footnote { .. } => "footnote",
            FoNodeData::FootnoteBody { .. } => "footnote-body",
            FoNodeData::Float { .. } => "float",
            FoNodeData::PageNumber { .. } => "page-number",
            FoNodeData::BlockContainer { .. } => "block-container",
            FoNodeData::InlineContainer { .. } => "inline-container",
            FoNodeData::TableAndCaption { .. } => "table-and-caption",
            FoNodeData::TableCaption { .. } => "table-caption",
            FoNodeData::PageSequenceMaster { .. } => "page-sequence-master",
            FoNodeData::Declarations => "declarations",
            FoNodeData::ColorProfile { .. } => "color-profile",
            FoNodeData::UnsupportedElement { .. } => "unsupported-element",
            FoNodeData::MultiSwitch { .. } => "multi-switch",
            FoNodeData::MultiCase { .. } => "multi-case",
            FoNodeData::MultiToggle { .. } => "multi-toggle",
            FoNodeData::MultiProperties { .. } => "multi-properties",
            FoNodeData::MultiPropertySet { .. } => "multi-property-set",
            FoNodeData::Wrapper { .. } => "wrapper",
            FoNodeData::Character { .. } => "character",
            FoNodeData::BidiOverride { .. } => "bidi-override",
            FoNodeData::InitialPropertySet { .. } => "initial-property-set",
            FoNodeData::ChangeBarBegin { .. } => "change-bar-begin",
            FoNodeData::ChangeBarEnd { .. } => "change-bar-end",
            FoNodeData::Text(_) => "#text",
        }
    }

    /// Get mutable property list if this node has one
    pub fn properties_mut(&mut self) -> Option<&mut PropertyList<'a>> {
        match self {
            FoNodeData::SimplePageMaster { properties, .. }
            | FoNodeData::RegionBody { properties }
            | FoNodeData::RegionBefore { properties }
            | FoNodeData::RegionAfter { properties }
            | FoNodeData::RegionStart { properties }
            | FoNodeData::RegionEnd { properties }
            | FoNodeData::PageSequence { properties, .. }
            | FoNodeData::Flow { properties, .. }
            | FoNodeData::StaticContent { properties, .. }
            | FoNodeData::Block { properties }
            | FoNodeData::Inline { properties }
            | FoNodeData::ListBlock { properties }
            | FoNodeData::ListItem { properties }
            | FoNodeData::ListItemLabel { properties }
            | FoNodeData::ListItemBody { properties }
            | FoNodeData::Table { properties }
            | FoNodeData::TableColumn { properties }
            | FoNodeData::TableHeader { properties }
            | FoNodeData::TableFooter { properties }
            | FoNodeData::TableBody { properties }
            | FoNodeData::TableRow { properties }
            | FoNodeData::TableCell { properties }
            | FoNodeData::ExternalGraphic { properties, .. }
            | FoNodeData::InstreamForeignObject { properties, .. }
            | FoNodeData::BasicLink { properties, .. }
            | FoNodeData::BookmarkTree { properties }
            | FoNodeData::Bookmark { properties, .. }
            | FoNodeData::BookmarkTitle { properties }
            | FoNodeData::PageNumberCitation { properties, .. }
            | FoNodeData::PageNumberCitationLast { properties, .. }
            | FoNodeData::Leader { properties }
            | FoNodeData::Marker { properties, .. }
            | FoNodeData::RetrieveMarker { properties, .. }
            | FoNodeData::Footnote { properties }
            | FoNodeData::FootnoteBody { properties }
            | FoNodeData::Float { properties }
            | FoNodeData::Wrapper { properties }
            | FoNodeData::Character { properties, .. }
            | FoNodeData::BidiOverride { properties, .. }
            | FoNodeData::InitialPropertySet { properties }
            | FoNodeData::PageNumber { properties }
            | FoNodeData::BlockContainer { properties }
            | FoNodeData::InlineContainer { properties }
            | FoNodeData::TableAndCaption { properties }
            | FoNodeData::TableCaption { properties }
            | FoNodeData::MultiSwitch { properties }
            | FoNodeData::MultiCase { properties, .. }
            | FoNodeData::MultiToggle { properties }
            | FoNodeData::MultiProperties { properties }
            | FoNodeData::MultiPropertySet { properties }
            | FoNodeData::ChangeBarBegin { properties, .. } => Some(properties),
            FoNodeData::ChangeBarEnd { .. } => None,
            _ => None,
        }
    }

    /// Get immutable property list if this node has one
    pub fn properties(&self) -> Option<&PropertyList<'a>> {
        match self {
            FoNodeData::SimplePageMaster { properties, .. }
            | FoNodeData::RegionBody { properties }
            | FoNodeData::RegionBefore { properties }
            | FoNodeData::RegionAfter { properties }
            | FoNodeData::RegionStart { properties }
            | FoNodeData::RegionEnd { properties }
            | FoNodeData::PageSequence { properties, .. }
            | FoNodeData::Flow { properties, .. }
            | FoNodeData::StaticContent { properties, .. }
            | FoNodeData::Block { properties }
            | FoNodeData::Inline { properties }
            | FoNodeData::ListBlock { properties }
            | FoNodeData::ListItem { properties }
            | FoNodeData::ListItemLabel { properties }
            | FoNodeData::ListItemBody { properties }
            | FoNodeData::Table { properties }
            | FoNodeData::TableColumn { properties }
            | FoNodeData::TableHeader { properties }
            | FoNodeData::TableFooter { properties }
            | FoNodeData::TableBody { properties }
            | FoNodeData::TableRow { properties }
            | FoNodeData::TableCell { properties }
            | FoNodeData::ExternalGraphic { properties, .. }
            | FoNodeData::InstreamForeignObject { properties, .. }
            | FoNodeData::BasicLink { properties, .. }
            | FoNodeData::BookmarkTree { properties }
            | FoNodeData::Bookmark { properties, .. }
            | FoNodeData::BookmarkTitle { properties }
            | FoNodeData::PageNumberCitation { properties, .. }
            | FoNodeData::PageNumberCitationLast { properties, .. }
            | FoNodeData::Leader { properties }
            | FoNodeData::Marker { properties, .. }
            | FoNodeData::RetrieveMarker { properties, .. }
            | FoNodeData::Footnote { properties }
            | FoNodeData::FootnoteBody { properties }
            | FoNodeData::Float { properties }
            | FoNodeData::Wrapper { properties }
            | FoNodeData::Character { properties, .. }
            | FoNodeData::BidiOverride { properties, .. }
            | FoNodeData::InitialPropertySet { properties }
            | FoNodeData::PageNumber { properties }
            | FoNodeData::BlockContainer { properties }
            | FoNodeData::InlineContainer { properties }
            | FoNodeData::TableAndCaption { properties }
            | FoNodeData::TableCaption { properties }
            | FoNodeData::MultiSwitch { properties }
            | FoNodeData::MultiCase { properties, .. }
            | FoNodeData::MultiToggle { properties }
            | FoNodeData::MultiProperties { properties }
            | FoNodeData::MultiPropertySet { properties }
            | FoNodeData::ChangeBarBegin { properties, .. } => Some(properties),
            FoNodeData::ChangeBarEnd { .. } => None,
            _ => None,
        }
    }

    /// Check if this node type can contain text
    pub fn can_contain_text(&self) -> bool {
        matches!(
            self,
            FoNodeData::Block { .. }
                | FoNodeData::Inline { .. }
                | FoNodeData::BasicLink { .. }
                | FoNodeData::BookmarkTitle { .. }
                | FoNodeData::Marker { .. }
                | FoNodeData::FootnoteBody { .. }
        )
    }

    /// Check if this node type is a layout master
    pub fn is_layout_master(&self) -> bool {
        matches!(
            self,
            FoNodeData::LayoutMasterSet
                | FoNodeData::SimplePageMaster { .. }
                | FoNodeData::RepeatablePageMasterAlternatives { .. }
                | FoNodeData::ConditionalPageMasterReference { .. }
        )
    }

    /// Check if this node type is a region
    pub fn is_region(&self) -> bool {
        matches!(
            self,
            FoNodeData::RegionBody { .. }
                | FoNodeData::RegionBefore { .. }
                | FoNodeData::RegionAfter { .. }
                | FoNodeData::RegionStart { .. }
                | FoNodeData::RegionEnd { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = FoNode::new(FoNodeData::Root);
        assert!(node.is_root());
        assert!(!node.has_children());
    }

    #[test]
    fn test_element_names() {
        assert_eq!(FoNodeData::Root.element_name(), "root");
        assert_eq!(
            FoNodeData::LayoutMasterSet.element_name(),
            "layout-master-set"
        );
        assert_eq!(FoNodeData::Text(String::new()).element_name(), "#text");
    }

    #[test]
    fn test_property_access() {
        let props = PropertyList::new();
        let mut data = FoNodeData::Block { properties: props };

        assert!(data.properties().is_some());
        assert!(data.properties_mut().is_some());

        let root = FoNodeData::Root;
        assert!(root.properties().is_none());
    }

    #[test]
    fn test_node_type_checks() {
        let layout = FoNodeData::LayoutMasterSet;
        assert!(layout.is_layout_master());

        let region = FoNodeData::RegionBody {
            properties: PropertyList::new(),
        };
        assert!(region.is_region());

        let block = FoNodeData::Block {
            properties: PropertyList::new(),
        };
        assert!(block.can_contain_text());
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    fn props() -> PropertyList<'static> {
        PropertyList::new()
    }

    // -----------------------------------------------------------------------
    // element_name() coverage for every variant
    // -----------------------------------------------------------------------

    #[test]
    fn test_element_name_root() {
        assert_eq!(FoNodeData::Root.element_name(), "root");
    }

    #[test]
    fn test_element_name_layout_master_set() {
        assert_eq!(
            FoNodeData::LayoutMasterSet.element_name(),
            "layout-master-set"
        );
    }

    #[test]
    fn test_element_name_simple_page_master() {
        let data = FoNodeData::SimplePageMaster {
            master_name: "A4".to_string(),
            properties: props(),
        };
        assert_eq!(data.element_name(), "simple-page-master");
    }

    #[test]
    fn test_element_name_page_sequence() {
        let data = FoNodeData::PageSequence {
            master_reference: "A4".to_string(),
            format: "1".to_string(),
            grouping_separator: None,
            grouping_size: None,
            properties: props(),
        };
        assert_eq!(data.element_name(), "page-sequence");
    }

    #[test]
    fn test_element_name_flow() {
        let data = FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: props(),
        };
        assert_eq!(data.element_name(), "flow");
    }

    #[test]
    fn test_element_name_static_content() {
        let data = FoNodeData::StaticContent {
            flow_name: "xsl-region-before".to_string(),
            properties: props(),
        };
        assert_eq!(data.element_name(), "static-content");
    }

    #[test]
    fn test_element_name_block() {
        assert_eq!(
            FoNodeData::Block {
                properties: props()
            }
            .element_name(),
            "block"
        );
    }

    #[test]
    fn test_element_name_inline() {
        assert_eq!(
            FoNodeData::Inline {
                properties: props()
            }
            .element_name(),
            "inline"
        );
    }

    #[test]
    fn test_element_name_list_elements() {
        assert_eq!(
            FoNodeData::ListBlock {
                properties: props()
            }
            .element_name(),
            "list-block"
        );
        assert_eq!(
            FoNodeData::ListItem {
                properties: props()
            }
            .element_name(),
            "list-item"
        );
        assert_eq!(
            FoNodeData::ListItemLabel {
                properties: props()
            }
            .element_name(),
            "list-item-label"
        );
        assert_eq!(
            FoNodeData::ListItemBody {
                properties: props()
            }
            .element_name(),
            "list-item-body"
        );
    }

    #[test]
    fn test_element_name_table_elements() {
        assert_eq!(
            FoNodeData::Table {
                properties: props()
            }
            .element_name(),
            "table"
        );
        assert_eq!(
            FoNodeData::TableColumn {
                properties: props()
            }
            .element_name(),
            "table-column"
        );
        assert_eq!(
            FoNodeData::TableHeader {
                properties: props()
            }
            .element_name(),
            "table-header"
        );
        assert_eq!(
            FoNodeData::TableFooter {
                properties: props()
            }
            .element_name(),
            "table-footer"
        );
        assert_eq!(
            FoNodeData::TableBody {
                properties: props()
            }
            .element_name(),
            "table-body"
        );
        assert_eq!(
            FoNodeData::TableRow {
                properties: props()
            }
            .element_name(),
            "table-row"
        );
        assert_eq!(
            FoNodeData::TableCell {
                properties: props()
            }
            .element_name(),
            "table-cell"
        );
    }

    #[test]
    fn test_element_name_graphic() {
        let data = FoNodeData::ExternalGraphic {
            src: "img.png".to_string(),
            content_width: None,
            content_height: None,
            scaling: None,
            properties: props(),
        };
        assert_eq!(data.element_name(), "external-graphic");
    }

    #[test]
    fn test_element_name_basic_link() {
        let data = FoNodeData::BasicLink {
            internal_destination: None,
            external_destination: Some("http://example.com".to_string()),
            properties: props(),
        };
        assert_eq!(data.element_name(), "basic-link");
    }

    #[test]
    fn test_element_name_page_number() {
        assert_eq!(
            FoNodeData::PageNumber {
                properties: props()
            }
            .element_name(),
            "page-number"
        );
    }

    #[test]
    fn test_element_name_page_number_citation() {
        let data = FoNodeData::PageNumberCitation {
            ref_id: "sec1".to_string(),
            properties: props(),
        };
        assert_eq!(data.element_name(), "page-number-citation");
    }

    #[test]
    fn test_element_name_page_number_citation_last() {
        let data = FoNodeData::PageNumberCitationLast {
            ref_id: "sec1".to_string(),
            properties: props(),
        };
        assert_eq!(data.element_name(), "page-number-citation-last");
    }

    #[test]
    fn test_element_name_block_container() {
        assert_eq!(
            FoNodeData::BlockContainer {
                properties: props()
            }
            .element_name(),
            "block-container"
        );
    }

    #[test]
    fn test_element_name_inline_container() {
        assert_eq!(
            FoNodeData::InlineContainer {
                properties: props()
            }
            .element_name(),
            "inline-container"
        );
    }

    #[test]
    fn test_element_name_wrapper() {
        assert_eq!(
            FoNodeData::Wrapper {
                properties: props()
            }
            .element_name(),
            "wrapper"
        );
    }

    #[test]
    fn test_element_name_character() {
        let data = FoNodeData::Character {
            character: 'A',
            properties: props(),
        };
        assert_eq!(data.element_name(), "character");
    }

    #[test]
    fn test_element_name_bidi_override() {
        let data = FoNodeData::BidiOverride {
            direction: "rtl".to_string(),
            properties: props(),
        };
        assert_eq!(data.element_name(), "bidi-override");
    }

    #[test]
    fn test_element_name_initial_property_set() {
        assert_eq!(
            FoNodeData::InitialPropertySet {
                properties: props()
            }
            .element_name(),
            "initial-property-set"
        );
    }

    #[test]
    fn test_element_name_declarations() {
        assert_eq!(FoNodeData::Declarations.element_name(), "declarations");
    }

    #[test]
    fn test_element_name_color_profile() {
        let data = FoNodeData::ColorProfile {
            src: "profile.icc".to_string(),
            color_profile_name: "cmyk".to_string(),
        };
        assert_eq!(data.element_name(), "color-profile");
    }

    #[test]
    fn test_element_name_unsupported() {
        let data = FoNodeData::UnsupportedElement {
            element_name: "fo:multi-properties".to_string(),
        };
        assert_eq!(data.element_name(), "unsupported-element");
    }

    #[test]
    fn test_element_name_text() {
        assert_eq!(
            FoNodeData::Text("hello".to_string()).element_name(),
            "#text"
        );
    }

    #[test]
    fn test_element_name_leader() {
        assert_eq!(
            FoNodeData::Leader {
                properties: props()
            }
            .element_name(),
            "leader"
        );
    }

    #[test]
    fn test_element_name_footnote() {
        assert_eq!(
            FoNodeData::Footnote {
                properties: props()
            }
            .element_name(),
            "footnote"
        );
        assert_eq!(
            FoNodeData::FootnoteBody {
                properties: props()
            }
            .element_name(),
            "footnote-body"
        );
    }

    #[test]
    fn test_element_name_bookmark() {
        assert_eq!(
            FoNodeData::BookmarkTree {
                properties: props()
            }
            .element_name(),
            "bookmark-tree"
        );
        let bm = FoNodeData::Bookmark {
            internal_destination: Some("intro".to_string()),
            external_destination: None,
            properties: props(),
        };
        assert_eq!(bm.element_name(), "bookmark");
        assert_eq!(
            FoNodeData::BookmarkTitle {
                properties: props()
            }
            .element_name(),
            "bookmark-title"
        );
    }

    #[test]
    fn test_element_name_marker() {
        let data = FoNodeData::Marker {
            marker_class_name: "header".to_string(),
            properties: props(),
        };
        assert_eq!(data.element_name(), "marker");
    }

    #[test]
    fn test_element_name_retrieve_marker() {
        let data = FoNodeData::RetrieveMarker {
            retrieve_class_name: "header".to_string(),
            retrieve_position: RetrievePosition::FirstStartingWithinPage,
            properties: props(),
        };
        assert_eq!(data.element_name(), "retrieve-marker");
    }

    #[test]
    fn test_element_name_page_sequence_master() {
        let data = FoNodeData::PageSequenceMaster {
            master_name: "alternating".to_string(),
        };
        assert_eq!(data.element_name(), "page-sequence-master");
    }

    #[test]
    fn test_element_name_change_bar() {
        let begin = FoNodeData::ChangeBarBegin {
            change_bar_class: "c1".to_string(),
            properties: props(),
        };
        let end = FoNodeData::ChangeBarEnd {
            change_bar_class: "c1".to_string(),
        };
        assert_eq!(begin.element_name(), "change-bar-begin");
        assert_eq!(end.element_name(), "change-bar-end");
    }

    // -----------------------------------------------------------------------
    // FoNode struct API
    // -----------------------------------------------------------------------

    #[test]
    fn test_fo_node_new_with_id() {
        let node = FoNode::new_with_id(FoNodeData::Root, Some("root-id".to_string()));
        assert_eq!(node.id(), Some("root-id"));
        assert!(node.is_root());
    }

    #[test]
    fn test_fo_node_new_no_id() {
        let node = FoNode::new(FoNodeData::LayoutMasterSet);
        assert!(node.id().is_none());
    }

    #[test]
    fn test_fo_node_set_id() {
        let mut node = FoNode::new(FoNodeData::Root);
        assert!(node.id().is_none());
        node.set_id(Some("my-id".to_string()));
        assert_eq!(node.id(), Some("my-id"));
        node.set_id(None);
        assert!(node.id().is_none());
    }

    #[test]
    fn test_fo_node_has_children() {
        use super::super::arena::NodeId;
        let mut node = FoNode::new(FoNodeData::Root);
        assert!(!node.has_children());
        // Simulate having a first_child
        node.first_child = Some(NodeId::from_index(1));
        assert!(node.has_children());
    }

    #[test]
    fn test_fo_node_is_root_with_parent() {
        use super::super::arena::NodeId;
        let mut node = FoNode::new(FoNodeData::Block {
            properties: props(),
        });
        assert!(node.is_root()); // no parent set yet
        node.parent = Some(NodeId::from_index(0));
        assert!(!node.is_root());
    }

    // -----------------------------------------------------------------------
    // can_contain_text(), is_layout_master(), is_region()
    // -----------------------------------------------------------------------

    #[test]
    fn test_can_contain_text_block_level() {
        assert!(FoNodeData::Block {
            properties: props()
        }
        .can_contain_text());
        assert!(FoNodeData::Inline {
            properties: props()
        }
        .can_contain_text());
    }

    #[test]
    fn test_cannot_contain_text_non_text_containers() {
        assert!(!FoNodeData::Table {
            properties: props()
        }
        .can_contain_text());
        assert!(!FoNodeData::TableRow {
            properties: props()
        }
        .can_contain_text());
        assert!(!FoNodeData::Root.can_contain_text());
        assert!(!FoNodeData::LayoutMasterSet.can_contain_text());
        assert!(!FoNodeData::ListBlock {
            properties: props()
        }
        .can_contain_text());
    }

    #[test]
    fn test_is_layout_master_variants() {
        assert!(FoNodeData::LayoutMasterSet.is_layout_master());
        assert!(FoNodeData::SimplePageMaster {
            master_name: "A4".to_string(),
            properties: props(),
        }
        .is_layout_master());
        assert!(FoNodeData::RepeatablePageMasterAlternatives {
            maximum_repeats: None,
        }
        .is_layout_master());
        assert!(FoNodeData::ConditionalPageMasterReference {
            master_reference: "A4".to_string(),
            page_position: PagePosition::Any,
            odd_or_even: OddOrEven::Any,
            blank_or_not_blank: BlankOrNotBlank::Any,
        }
        .is_layout_master());
    }

    #[test]
    fn test_is_not_layout_master() {
        assert!(!FoNodeData::Root.is_layout_master());
        assert!(!FoNodeData::Block {
            properties: props()
        }
        .is_layout_master());
    }

    #[test]
    fn test_is_region_all_variants() {
        assert!(FoNodeData::RegionBody {
            properties: props()
        }
        .is_region());
        assert!(FoNodeData::RegionBefore {
            properties: props()
        }
        .is_region());
        assert!(FoNodeData::RegionAfter {
            properties: props()
        }
        .is_region());
        assert!(FoNodeData::RegionStart {
            properties: props()
        }
        .is_region());
        assert!(FoNodeData::RegionEnd {
            properties: props()
        }
        .is_region());
    }

    #[test]
    fn test_is_not_region() {
        assert!(!FoNodeData::Root.is_region());
        assert!(!FoNodeData::Block {
            properties: props()
        }
        .is_region());
        assert!(!FoNodeData::LayoutMasterSet.is_region());
    }

    // -----------------------------------------------------------------------
    // properties() / properties_mut()
    // -----------------------------------------------------------------------

    #[test]
    fn test_properties_none_for_propertyless_variants() {
        assert!(FoNodeData::Root.properties().is_none());
        assert!(FoNodeData::LayoutMasterSet.properties().is_none());
        assert!(FoNodeData::Declarations.properties().is_none());
        assert!(FoNodeData::Text("hi".to_string()).properties().is_none());
        let cbe = FoNodeData::ChangeBarEnd {
            change_bar_class: "c1".to_string(),
        };
        assert!(cbe.properties().is_none());
    }

    #[test]
    fn test_properties_some_for_property_bearing_variants() {
        assert!(FoNodeData::Block {
            properties: props()
        }
        .properties()
        .is_some());
        assert!(FoNodeData::Inline {
            properties: props()
        }
        .properties()
        .is_some());
        assert!(FoNodeData::Table {
            properties: props()
        }
        .properties()
        .is_some());
        assert!(FoNodeData::PageNumber {
            properties: props()
        }
        .properties()
        .is_some());
        assert!(FoNodeData::BlockContainer {
            properties: props()
        }
        .properties()
        .is_some());
        assert!(FoNodeData::Wrapper {
            properties: props()
        }
        .properties()
        .is_some());
    }

    #[test]
    fn test_properties_mut_some() {
        let mut data = FoNodeData::Inline {
            properties: props(),
        };
        assert!(data.properties_mut().is_some());
    }

    #[test]
    fn test_properties_mut_none_for_text() {
        let mut data = FoNodeData::Text("hello".to_string());
        assert!(data.properties_mut().is_none());
    }

    // -----------------------------------------------------------------------
    // Enum helper types
    // -----------------------------------------------------------------------

    #[test]
    fn test_page_position_equality() {
        assert_eq!(PagePosition::First, PagePosition::First);
        assert_ne!(PagePosition::First, PagePosition::Last);
        assert_ne!(PagePosition::Rest, PagePosition::Any);
    }

    #[test]
    fn test_odd_or_even_equality() {
        assert_eq!(OddOrEven::Odd, OddOrEven::Odd);
        assert_ne!(OddOrEven::Odd, OddOrEven::Even);
    }

    #[test]
    fn test_blank_or_not_blank_equality() {
        assert_eq!(BlankOrNotBlank::Blank, BlankOrNotBlank::Blank);
        assert_ne!(BlankOrNotBlank::Blank, BlankOrNotBlank::NotBlank);
    }

    #[test]
    fn test_retrieve_position_copy() {
        let p1 = RetrievePosition::FirstStartingWithinPage;
        let p2 = p1;
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_region_element_names() {
        assert_eq!(
            FoNodeData::RegionBody {
                properties: props()
            }
            .element_name(),
            "region-body"
        );
        assert_eq!(
            FoNodeData::RegionBefore {
                properties: props()
            }
            .element_name(),
            "region-before"
        );
        assert_eq!(
            FoNodeData::RegionAfter {
                properties: props()
            }
            .element_name(),
            "region-after"
        );
        assert_eq!(
            FoNodeData::RegionStart {
                properties: props()
            }
            .element_name(),
            "region-start"
        );
        assert_eq!(
            FoNodeData::RegionEnd {
                properties: props()
            }
            .element_name(),
            "region-end"
        );
    }

    #[test]
    fn test_multi_element_names() {
        assert_eq!(
            FoNodeData::MultiSwitch {
                properties: props()
            }
            .element_name(),
            "multi-switch"
        );
        assert_eq!(
            FoNodeData::MultiCase {
                starting_state: "visible".to_string(),
                properties: props(),
            }
            .element_name(),
            "multi-case"
        );
        assert_eq!(
            FoNodeData::MultiToggle {
                properties: props()
            }
            .element_name(),
            "multi-toggle"
        );
        assert_eq!(
            FoNodeData::MultiProperties {
                properties: props()
            }
            .element_name(),
            "multi-properties"
        );
        assert_eq!(
            FoNodeData::MultiPropertySet {
                properties: props()
            }
            .element_name(),
            "multi-property-set"
        );
    }

    #[test]
    fn test_float_element_name() {
        assert_eq!(
            FoNodeData::Float {
                properties: props()
            }
            .element_name(),
            "float"
        );
    }

    #[test]
    fn test_table_and_caption_element_names() {
        assert_eq!(
            FoNodeData::TableAndCaption {
                properties: props()
            }
            .element_name(),
            "table-and-caption"
        );
        assert_eq!(
            FoNodeData::TableCaption {
                properties: props()
            }
            .element_name(),
            "table-caption"
        );
    }

    #[test]
    fn test_instream_foreign_object_element_name() {
        let data = FoNodeData::InstreamForeignObject {
            content_width: None,
            content_height: None,
            scaling: None,
            foreign_xml: "<svg/>".to_string(),
            properties: props(),
        };
        assert_eq!(data.element_name(), "instream-foreign-object");
    }

    #[test]
    fn test_repeatable_page_master_alternatives_element_name() {
        let data = FoNodeData::RepeatablePageMasterAlternatives {
            maximum_repeats: Some(3),
        };
        assert_eq!(data.element_name(), "repeatable-page-master-alternatives");
    }

    #[test]
    fn test_conditional_page_master_reference_element_name() {
        let data = FoNodeData::ConditionalPageMasterReference {
            master_reference: "A4".to_string(),
            page_position: PagePosition::First,
            odd_or_even: OddOrEven::Odd,
            blank_or_not_blank: BlankOrNotBlank::NotBlank,
        };
        assert_eq!(data.element_name(), "conditional-page-master-reference");
    }
}
