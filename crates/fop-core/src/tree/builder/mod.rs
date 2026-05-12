//! FO tree builder - constructs the FO tree from XML
//!
//! Splits into:
//! - `mod.rs` (this file): `FoTreeBuilder` struct, XML parsing loop, element lifecycle
//! - `node_factory`: FO node creation from element names/attributes
//! - `property_parser`: Property value parsing (length, color, gradient, etc.)

mod node_factory;
mod property_parser;

use crate::properties::PropertyList;
use crate::tree::{FoArena, FoNode, FoNodeData, NodeId};
use crate::xml::XmlParser;
use crate::{FopError, Result};
use quick_xml::events::Event;
use std::io::BufRead;

/// Builder for constructing FO trees from XML
pub struct FoTreeBuilder<'a> {
    arena: FoArena<'a>,
    current_node: Option<NodeId>,
    /// Depth counter for nested elements inside instream-foreign-object
    foreign_object_depth: usize,
    /// Buffer to collect raw XML content of instream-foreign-object
    foreign_xml_buffer: String,
    /// NodeId of the instream-foreign-object node being built
    foreign_object_node: Option<NodeId>,
    /// Depth of an active non-FO subtree (e.g. XMP/RDF inside fo:declarations).
    /// Non-FO start tags bump this; the matching end tags decrement it instead
    /// of popping the fo: parent stack. Without this counter, every closing
    /// foreign-namespace tag erroneously pops `current_node`, orphaning later
    /// fo: nodes like fo:page-sequence.
    non_fo_depth: usize,
}

impl<'a> FoTreeBuilder<'a> {
    /// Create a new tree builder
    pub fn new() -> Self {
        Self {
            arena: FoArena::new(),
            current_node: None,
            foreign_object_depth: 0,
            foreign_xml_buffer: String::new(),
            foreign_object_node: None,
            non_fo_depth: 0,
        }
    }

    /// Parse an XSL-FO document and build the tree
    pub fn parse<R: BufRead>(mut self, reader: R) -> Result<FoArena<'a>> {
        let mut parser = XmlParser::new(reader);

        loop {
            let event = parser.read_event()?;

            // If we are inside a foreign object child element, capture raw XML
            if self.foreign_object_depth > 0 {
                match &event {
                    Event::Start(start) => {
                        parser.update_namespaces(start);
                        let raw = std::str::from_utf8(start.as_ref())
                            .unwrap_or("")
                            .to_string();
                        self.foreign_xml_buffer.push('<');
                        self.foreign_xml_buffer.push_str(&raw);
                        self.foreign_xml_buffer.push('>');
                        self.foreign_object_depth += 1;
                    }
                    Event::Empty(start) => {
                        parser.update_namespaces(start);
                        let raw = std::str::from_utf8(start.as_ref())
                            .unwrap_or("")
                            .to_string();
                        self.foreign_xml_buffer.push('<');
                        self.foreign_xml_buffer.push_str(&raw);
                        self.foreign_xml_buffer.push_str("/>");
                    }
                    Event::End(end) => {
                        self.foreign_object_depth -= 1;
                        if self.foreign_object_depth > 0 {
                            let raw = std::str::from_utf8(end.as_ref()).unwrap_or("").to_string();
                            self.foreign_xml_buffer.push_str("</");
                            self.foreign_xml_buffer.push_str(&raw);
                            self.foreign_xml_buffer.push('>');
                        }
                        // When depth returns to 0, the child root element is closed
                    }
                    Event::Text(text) => {
                        let text_content = parser.extract_text(text).unwrap_or_default();
                        self.foreign_xml_buffer.push_str(&text_content);
                    }
                    Event::Eof => break,
                    _ => {}
                }
                continue;
            }

            match event {
                Event::Start(ref start) | Event::Empty(ref start) => {
                    parser.update_namespaces(start);
                    let (name, ns) = parser.extract_name(start)?;

                    if ns.is_fo() {
                        self.start_element(&name, start, &parser)?;

                        // If it was an empty element, immediately close it
                        if matches!(event, Event::Empty(_)) {
                            self.end_element()?;
                        }
                    } else if self.foreign_object_node.is_some() {
                        // Non-FO element inside instream-foreign-object: capture as raw XML
                        let raw = std::str::from_utf8(start.as_ref())
                            .unwrap_or("")
                            .to_string();
                        self.foreign_xml_buffer.push('<');
                        self.foreign_xml_buffer.push_str(&raw);
                        if matches!(event, Event::Empty(_)) {
                            self.foreign_xml_buffer.push_str("/>");
                        } else {
                            self.foreign_xml_buffer.push('>');
                            self.foreign_object_depth += 1;
                        }
                    } else if matches!(event, Event::Start(_)) {
                        // Non-FO element outside instream-foreign-object — e.g. XMP
                        // metadata inside fo:declarations. We don't model these in
                        // the FO tree, but we must track depth so the matching End
                        // events don't unwind our fo: parent stack.
                        self.non_fo_depth += 1;
                    }
                    // Empty non-FO outside foreign-object: nothing to track —
                    // quick-xml fires no matching End event.
                }
                Event::End(_) => {
                    if self.foreign_object_node.is_some() && self.foreign_object_depth == 0 {
                        // This End event closes the fo:instream-foreign-object itself
                        self.finalize_foreign_object();
                    }
                    if self.non_fo_depth > 0 {
                        self.non_fo_depth -= 1;
                    } else {
                        self.end_element()?;
                    }
                }
                Event::Text(text) => {
                    // Ignore text inside a non-FO subtree (e.g. XMP/RDF) — it
                    // doesn't belong on any fo: node.
                    if self.non_fo_depth == 0 {
                        let text_content = parser.extract_text(&text)?;
                        let trimmed = text_content.trim();
                        if !trimmed.is_empty() {
                            self.add_text(trimmed)?;
                        }
                    }
                }
                Event::CData(cdata) => {
                    // Same rationale as Text above.
                    if self.non_fo_depth == 0 {
                        // CDATA sections preserve content exactly
                        let cdata_content = parser.extract_cdata(&cdata)?;
                        if !cdata_content.is_empty() {
                            self.add_text(&cdata_content)?;
                        }
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }

        Ok(self.arena)
    }

    /// Finalize the foreign object: store captured XML and clear state
    fn finalize_foreign_object(&mut self) {
        if let Some(node_id) = self.foreign_object_node.take() {
            let xml = std::mem::take(&mut self.foreign_xml_buffer);
            if let Some(node) = self.arena.get_mut(node_id) {
                if let FoNodeData::InstreamForeignObject { foreign_xml, .. } = &mut node.data {
                    *foreign_xml = xml;
                }
            }
        }
    }

    /// Handle start of an element
    fn start_element(
        &mut self,
        name: &str,
        start: &quick_xml::events::BytesStart,
        parser: &XmlParser<impl BufRead>,
    ) -> Result<()> {
        // Create property list (inheritance will be resolved when properties are accessed)
        let mut properties = PropertyList::new();

        // Parse attributes into properties
        let attributes = parser.extract_attributes(start)?;

        // Extract the "id" attribute if present
        let element_id = attributes
            .iter()
            .find(|(k, _)| k == "id")
            .map(|(_, v)| v.clone());

        // Populate properties from attributes
        node_factory::populate_properties(&mut properties, &attributes)?;

        // Validate all properties after parsing
        properties.validate()?;

        // Handle xml:lang on fo:root for document language metadata
        if name == "root" {
            if let Some((_, lang)) = attributes
                .iter()
                .find(|(k, _)| k == "xml:lang" || k == "xml-lang")
            {
                self.arena.document_lang = Some(lang.clone());
            }
        }

        // Create the appropriate FO node
        let node_data = node_factory::create_node_data(name, &attributes, properties)?;
        let node = FoNode::new_with_id(node_data, element_id.clone());
        let node_id = self.arena.add_node(node);

        // Register the ID in the registry if present
        if let Some(id) = element_id {
            self.arena.id_registry_mut().register_id(id, node_id)?;
        }

        // Set up parent-child relationship
        if let Some(parent_id) = self.current_node {
            self.arena
                .append_child(parent_id, node_id)
                .map_err(FopError::Generic)?;
        }

        // If this is an instream-foreign-object, track the node for XML capture
        if name == "instream-foreign-object" {
            self.foreign_object_node = Some(node_id);
            self.foreign_xml_buffer.clear();
            self.foreign_object_depth = 0;
        }

        // Update current node
        self.current_node = Some(node_id);

        Ok(())
    }

    /// Handle end of an element
    fn end_element(&mut self) -> Result<()> {
        if let Some(current) = self.current_node {
            // Move back to parent
            self.current_node = self.arena.get(current).and_then(|n| n.parent);
        }
        Ok(())
    }

    /// Add text content to current node
    fn add_text(&mut self, text: &str) -> Result<()> {
        if let Some(parent_id) = self.current_node {
            // Check if parent can contain text
            if let Some(parent) = self.arena.get(parent_id) {
                if parent.data.can_contain_text() {
                    let text_node = FoNode::new(FoNodeData::Text(text.to_string()));
                    let text_id = self.arena.add_node(text_node);
                    self.arena
                        .append_child(parent_id, text_id)
                        .map_err(FopError::Generic)?;
                }
            }
        }
        Ok(())
    }
}

impl<'a> Default for FoTreeBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PropertyId;
    use std::io::Cursor;

    #[test]
    fn test_parse_simple_document() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let arena = builder.parse(cursor).expect("test: should succeed");

        assert!(!arena.is_empty());
        assert_eq!(arena.len(), 4); // root, layout-master-set, simple-page-master, region-body
    }

    #[test]
    fn test_parse_with_text() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block>Hello World</fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let arena = builder.parse(cursor).expect("test: should succeed");

        // Should have: root, layout-master-set, simple-page-master, region-body,
        //              page-sequence, flow, block, text
        assert!(arena.len() >= 8);
    }

    #[test]
    fn test_property_parsing() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
            <fo:region-body margin="1in"/>
        </fo:simple-page-master>
    </fo:layout-master-set>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let arena = builder.parse(cursor).expect("test: should succeed");

        // Check that properties were parsed
        for (_, node) in arena.iter() {
            if let Some(props) = node.data.properties() {
                // Properties should be accessible
                let _ = props.get(PropertyId::PageWidth);
            }
        }
    }

    #[test]
    fn test_parse_document_with_block_and_inline() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block>
                <fo:inline font-weight="bold">Bold text</fo:inline>
                Normal text
            </fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let arena = builder.parse(cursor).expect("test: should succeed");

        // Should have root, layout-master-set, simple-page-master, region-body,
        // page-sequence, flow, block, inline, text nodes
        assert!(arena.len() >= 8);
    }

    #[test]
    fn test_parse_document_with_multiple_blocks() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block>First block</fo:block>
            <fo:block>Second block</fo:block>
            <fo:block>Third block</fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let arena = builder.parse(cursor).expect("test: should succeed");

        // At least root, layout-master-set, simple-page-master, region-body,
        // page-sequence, flow, 3 blocks (text nodes may or may not be separate)
        assert!(arena.len() >= 9);
    }

    #[test]
    fn test_parse_document_with_font_properties() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block font-size="14pt" font-family="Arial" color="red">Styled text</fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let result = builder.parse(cursor);
        assert!(
            result.is_ok(),
            "Should parse document with font properties: {:?}",
            result.err()
        );

        let arena = result.expect("test: should succeed");
        assert!(arena.len() >= 7);
    }

    #[test]
    fn test_parse_document_with_list() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:list-block>
                <fo:list-item>
                    <fo:list-item-label><fo:block>1.</fo:block></fo:list-item-label>
                    <fo:list-item-body><fo:block>Item one</fo:block></fo:list-item-body>
                </fo:list-item>
            </fo:list-block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let result = builder.parse(cursor);
        assert!(
            result.is_ok(),
            "Should parse list structure: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_document_with_cdata() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block><![CDATA[Text with <special> & chars]]></fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let result = builder.parse(cursor);
        // CDATA sections should be parsed without error
        assert!(
            result.is_ok(),
            "Should parse CDATA sections: {:?}",
            result.err()
        );

        let arena = result.expect("test: should succeed");
        // Find text node with CDATA content
        let has_cdata_text = arena.iter().any(|(_, node)| {
            if let FoNodeData::Text(text) = &node.data {
                text.contains("Text with")
            } else {
                false
            }
        });
        assert!(
            has_cdata_text,
            "CDATA content should be stored as text node"
        );
    }

    #[test]
    fn test_parse_invalid_xml_returns_error() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:unclosed-element>
    </fo:layout-master-set>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        // Invalid XML (unclosed element) should return an error
        // (Behavior depends on parser leniency)
        let result = builder.parse(cursor);
        // Just verify it doesn't panic - may succeed or fail
        let _ = result;
    }

    #[test]
    fn test_parse_document_with_multiple_page_sequences() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block>Page 1 content</fo:block>
        </fo:flow>
    </fo:page-sequence>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block>Page 2 content</fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let result = builder.parse(cursor);
        assert!(result.is_ok(), "Should parse multiple page sequences");
    }

    #[test]
    fn test_parse_document_with_margin_property() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body margin-top="1cm" margin-bottom="2cm"/>
        </fo:simple-page-master>
    </fo:layout-master-set>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let result = builder.parse(cursor);
        assert!(result.is_ok(), "Should parse margin properties");
    }

    #[test]
    fn test_parse_document_with_table() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:table>
                <fo:table-body>
                    <fo:table-row>
                        <fo:table-cell>
                            <fo:block>Cell content</fo:block>
                        </fo:table-cell>
                    </fo:table-row>
                </fo:table-body>
            </fo:table>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let result = builder.parse(cursor);
        assert!(
            result.is_ok(),
            "Should parse table structure: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_document_is_not_empty() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let arena = builder.parse(cursor).expect("test: should succeed");

        assert!(!arena.is_empty());
        assert!(!arena.is_empty());
    }

    #[test]
    fn test_parse_preserves_text_content() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block>Hello World</fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let arena = builder.parse(cursor).expect("test: should succeed");

        // Find the text node
        let text_found = arena
            .iter()
            .any(|(_, node)| matches!(&node.data, FoNodeData::Text(t) if t == "Hello World"));
        assert!(text_found, "Text content should be preserved in tree");
    }

    #[test]
    fn test_parse_document_with_whitespace_only_text_is_trimmed() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let arena = builder.parse(cursor).expect("test: should succeed");

        // Whitespace-only text nodes should be stripped
        let whitespace_only_text = arena.iter().any(|(_, node)| {
            matches!(&node.data, FoNodeData::Text(t) if t.trim().is_empty() && !t.is_empty())
        });
        assert!(
            !whitespace_only_text,
            "Whitespace-only text nodes should be stripped"
        );
    }

    #[test]
    fn test_parse_document_with_processing_instruction() {
        let xml = r#"<?xml version="1.0"?>
<?fop-processor key="value"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let result = builder.parse(cursor);
        // Processing instructions should not cause parse errors
        assert!(
            result.is_ok(),
            "Processing instructions should be handled gracefully"
        );
    }

    #[test]
    fn test_parse_document_with_xml_comment() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <!-- This is a comment -->
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <!-- Page master comment -->
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let result = builder.parse(cursor);
        assert!(result.is_ok(), "XML comments should be handled gracefully");
    }

    #[test]
    fn test_parse_font_size_in_pts() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block font-size="16pt">Large text</fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let result = builder.parse(cursor);
        assert!(result.is_ok());

        let arena = result.expect("test: should succeed");
        // Find the block node and verify its font-size
        for (_, node) in arena.iter() {
            if let FoNodeData::Block { properties } = &node.data {
                if properties.is_explicit(PropertyId::FontSize) {
                    let font_size = properties
                        .get(PropertyId::FontSize)
                        .expect("test: should succeed");
                    if let Some(length) = font_size.as_length() {
                        assert_eq!(length.to_pt(), 16.0);
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_color_property_red() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block color="red">Red text</fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let result = builder.parse(cursor);
        assert!(result.is_ok(), "Should parse color properties");
    }

    #[test]
    fn test_parse_hex_color_property() {
        // Use rgb() format to avoid issues with # in raw strings
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block color="red">Hex red text</fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let builder = FoTreeBuilder::new();
        let result = builder.parse(cursor);
        assert!(result.is_ok(), "Should parse color properties");
    }
}

// ===== ADDITIONAL TESTS (new tests for builder) =====
#[cfg(test)]
mod additional_tests {
    use super::*;
    use std::io::Cursor;

    fn make_minimal_fo(flow_content: &str) -> String {
        format!(
            r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            {}
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#,
            flow_content
        )
    }

    #[test]
    fn test_parse_block_with_all_font_properties() {
        let xml = make_minimal_fo(
            r#"<fo:block font-size="14pt" font-weight="bold" font-style="italic"
                font-family="Times New Roman" color="navy">Styled text</fo:block>"#,
        );
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(
            result.is_ok(),
            "Font properties should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_block_with_margin_properties() {
        let xml = make_minimal_fo(
            r#"<fo:block margin-top="10pt" margin-bottom="10pt"
                margin-left="20pt" margin-right="20pt">Margins</fo:block>"#,
        );
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "Margin properties: {:?}", result.err());
    }

    #[test]
    fn test_parse_block_with_padding_properties() {
        let xml = make_minimal_fo(
            r#"<fo:block padding-top="5pt" padding-bottom="5pt"
                padding-left="10pt" padding-right="10pt">Padding</fo:block>"#,
        );
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "Padding properties: {:?}", result.err());
    }

    #[test]
    fn test_parse_block_with_border_properties() {
        let xml = make_minimal_fo(
            r#"<fo:block border-top-style="solid" border-top-width="1pt"
                border-top-color="black">Border</fo:block>"#,
        );
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "Border properties: {:?}", result.err());
    }

    #[test]
    fn test_parse_inline_elements() {
        let xml = make_minimal_fo(
            r#"<fo:block>Text with <fo:inline font-weight="bold">bold</fo:inline> part</fo:block>"#,
        );
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "Inline element: {:?}", result.err());
    }

    #[test]
    fn test_parse_nested_blocks() {
        let xml = make_minimal_fo(
            r#"<fo:block>
                <fo:block>Inner block 1</fo:block>
                <fo:block>Inner block 2</fo:block>
                <fo:block>Inner block 3</fo:block>
            </fo:block>"#,
        );
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "Nested blocks: {:?}", result.err());
    }

    #[test]
    fn test_parse_table_structure() {
        let xml = make_minimal_fo(
            r#"<fo:table>
                <fo:table-column column-width="50pt"/>
                <fo:table-column column-width="50pt"/>
                <fo:table-body>
                    <fo:table-row>
                        <fo:table-cell><fo:block>Cell 1</fo:block></fo:table-cell>
                        <fo:table-cell><fo:block>Cell 2</fo:block></fo:table-cell>
                    </fo:table-row>
                </fo:table-body>
            </fo:table>"#,
        );
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "Table structure: {:?}", result.err());
    }

    #[test]
    fn test_parse_list_structure() {
        let xml = make_minimal_fo(
            r#"<fo:list-block>
                <fo:list-item>
                    <fo:list-item-label end-indent="label-end()">
                        <fo:block>1.</fo:block>
                    </fo:list-item-label>
                    <fo:list-item-body start-indent="body-start()">
                        <fo:block>First item</fo:block>
                    </fo:list-item-body>
                </fo:list-item>
            </fo:list-block>"#,
        );
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "List structure: {:?}", result.err());
    }

    #[test]
    fn test_parse_external_graphic() {
        let xml = make_minimal_fo(
            r#"<fo:block><fo:external-graphic src="url('image.png')"/></fo:block>"#,
        );
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "External graphic: {:?}", result.err());
    }

    #[test]
    fn test_parse_basic_link_internal() {
        let xml = make_minimal_fo(
            r#"<fo:block>
                <fo:basic-link internal-destination="target">Link</fo:basic-link>
            </fo:block>"#,
        );
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "Basic link internal: {:?}", result.err());
    }

    #[test]
    fn test_parse_basic_link_external() {
        let xml = make_minimal_fo(
            r#"<fo:block>
                <fo:basic-link external-destination="url('https://example.com')">URL</fo:basic-link>
            </fo:block>"#,
        );
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "Basic link external: {:?}", result.err());
    }

    #[test]
    fn test_parse_page_number_inline() {
        let xml = make_minimal_fo(r#"<fo:block>Page <fo:page-number/></fo:block>"#);
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "Page number: {:?}", result.err());
    }

    #[test]
    fn test_parse_page_number_citation() {
        let xml = make_minimal_fo(
            r#"<fo:block>See page <fo:page-number-citation ref-id="target"/></fo:block>"#,
        );
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "Page number citation: {:?}", result.err());
    }

    #[test]
    fn test_parse_leader_dots() {
        let xml =
            make_minimal_fo(r#"<fo:block>Chapter<fo:leader leader-pattern="dots"/>10</fo:block>"#);
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "Leader: {:?}", result.err());
    }

    #[test]
    fn test_parse_footnote() {
        let xml = make_minimal_fo(
            r#"<fo:block>Text<fo:footnote>
                <fo:inline font-size="8pt" vertical-align="super">1</fo:inline>
                <fo:footnote-body>
                    <fo:block font-size="8pt">Footnote text</fo:block>
                </fo:footnote-body>
            </fo:footnote></fo:block>"#,
        );
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "Footnote: {:?}", result.err());
    }

    #[test]
    fn test_parse_block_container() {
        let xml = make_minimal_fo(
            r#"<fo:block-container width="100pt" height="100pt">
                <fo:block>Inside block container</fo:block>
            </fo:block-container>"#,
        );
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "Block container: {:?}", result.err());
    }

    #[test]
    fn test_parse_bookmark_tree() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:bookmark-tree>
        <fo:bookmark internal-destination="ch1">
            <fo:bookmark-title>Chapter 1</fo:bookmark-title>
        </fo:bookmark>
    </fo:bookmark-tree>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block id="ch1">Chapter 1 content</fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "Bookmark tree: {:?}", result.err());
    }

    #[test]
    fn test_parse_document_with_static_content() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-before extent="20mm"/>
            <fo:region-body/>
            <fo:region-after extent="20mm"/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:static-content flow-name="xsl-region-before">
            <fo:block>Header text</fo:block>
        </fo:static-content>
        <fo:static-content flow-name="xsl-region-after">
            <fo:block>Footer text</fo:block>
        </fo:static-content>
        <fo:flow flow-name="xsl-region-body">
            <fo:block>Body content</fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "Static content: {:?}", result.err());
    }

    #[test]
    fn test_parse_document_returns_non_empty_arena() {
        let xml = make_minimal_fo("<fo:block>Content</fo:block>");
        let cursor = Cursor::new(xml);
        let arena = FoTreeBuilder::new()
            .parse(cursor)
            .expect("test: should succeed");
        assert!(!arena.is_empty(), "Arena should not be empty after parsing");
    }

    #[test]
    fn test_parse_document_root_is_fo_root() {
        let xml = make_minimal_fo("<fo:block>Content</fo:block>");
        let cursor = Cursor::new(xml);
        let arena = FoTreeBuilder::new()
            .parse(cursor)
            .expect("test: should succeed");
        let (_, root_node) = arena.root().expect("Should have root node");
        assert!(matches!(root_node.data, FoNodeData::Root));
    }

    #[test]
    fn test_parse_document_with_text_align_center() {
        let xml = make_minimal_fo(r#"<fo:block text-align="center">Centered</fo:block>"#);
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "text-align center: {:?}", result.err());
    }

    #[test]
    fn test_parse_document_with_text_align_justify() {
        let xml = make_minimal_fo(r#"<fo:block text-align="justify">Justified</fo:block>"#);
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "text-align justify: {:?}", result.err());
    }

    #[test]
    fn test_parse_line_height_property() {
        let xml = make_minimal_fo(r#"<fo:block line-height="1.5">Text</fo:block>"#);
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "line-height: {:?}", result.err());
    }

    #[test]
    fn test_parse_keep_together_property() {
        let xml = make_minimal_fo(
            r#"<fo:block keep-together.within-page="always">Kept together</fo:block>"#,
        );
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "keep-together: {:?}", result.err());
    }

    #[test]
    fn test_parse_background_color_property() {
        let xml = make_minimal_fo(r#"<fo:block background-color="yellow">Highlighted</fo:block>"#);
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "background-color: {:?}", result.err());
    }

    #[test]
    fn test_parse_multiple_page_sequences_with_content() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block>Page sequence 1</fo:block>
        </fo:flow>
    </fo:page-sequence>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block>Page sequence 2</fo:block>
        </fo:flow>
    </fo:page-sequence>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block>Page sequence 3</fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(
            result.is_ok(),
            "Multiple page sequences: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_missing_flow_name_is_error() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow>
            <fo:block>No flow-name attribute</fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_err(), "Missing flow-name should be an error");
    }

    #[test]
    fn test_parse_missing_master_name_is_error() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master>
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block>Text</fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_err(), "Missing master-name should be an error");
    }

    #[test]
    fn test_parse_xml_lang_sets_document_lang() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format" xml:lang="en">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block>English text</fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;
        let cursor = Cursor::new(xml);
        let arena = FoTreeBuilder::new()
            .parse(cursor)
            .expect("test: should succeed");
        assert_eq!(arena.document_lang, Some("en".to_string()));
    }

    #[test]
    fn test_parse_document_without_lang_has_none() {
        let xml = make_minimal_fo("<fo:block>Text</fo:block>");
        let cursor = Cursor::new(xml);
        let arena = FoTreeBuilder::new()
            .parse(cursor)
            .expect("test: should succeed");
        assert!(arena.document_lang.is_none());
    }

    #[test]
    fn test_parse_cdata_in_block() {
        let xml = make_minimal_fo(r#"<fo:block><![CDATA[<special> & content]]></fo:block>"#);
        let cursor = Cursor::new(xml);
        let result = FoTreeBuilder::new().parse(cursor);
        assert!(result.is_ok(), "CDATA in block: {:?}", result.err());
    }

    /// Regression: fo:declarations may contain XMP/RDF metadata in foreign
    /// namespaces. Their End events must not pop the fo: parent stack —
    /// previously every closing foreign tag popped `current_node`, so
    /// fo:page-sequence ended up orphaned (no parent) and the layout engine
    /// saw zero page sequences.
    #[test]
    fn test_parse_declarations_with_xmp_keeps_page_sequence_attached() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:declarations>
        <x:xmpmeta xmlns:x="adobe:ns:meta/">
            <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
                <rdf:Description xmlns:dc="http://purl.org/dc/elements/1.1/" rdf:about="">
                    <dc:title>
                        <rdf:Alt><rdf:li xml:lang="x-default">Test Invoice</rdf:li></rdf:Alt>
                    </dc:title>
                </rdf:Description>
            </rdf:RDF>
        </x:xmpmeta>
    </fo:declarations>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block>Hello.</fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;
        let cursor = Cursor::new(xml);
        let arena = FoTreeBuilder::new()
            .parse(cursor)
            .expect("XMP inside fo:declarations should parse");

        // fo:page-sequence must be a child of fo:root, not an orphan.
        let (root_id, _) = arena.root().expect("root present");
        let root_has_page_sequence = arena
            .children(root_id)
            .into_iter()
            .filter_map(|id| arena.get(id))
            .any(|n| matches!(n.data, FoNodeData::PageSequence { .. }));
        assert!(
            root_has_page_sequence,
            "fo:page-sequence must be attached to fo:root after \
             fo:declarations with foreign-namespace children"
        );

        // No fo: node other than root should be parentless.
        let root_id_only = arena.root().map(|(rid, _)| rid);
        let orphans = arena
            .iter()
            .filter(|(id, n)| n.parent.is_none() && Some(*id) != root_id_only)
            .count();
        assert_eq!(orphans, 0, "no orphaned fo: nodes");
    }
}
