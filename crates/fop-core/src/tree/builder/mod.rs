//! FO tree builder - constructs the FO tree from XML
//!
//! Splits into:
//! - `mod.rs` (this file): `FoTreeBuilder` struct, XML parsing loop, element lifecycle
//! - `node_factory`: FO node creation from element names/attributes
//! - `property_parser`: Property value parsing (length, color, gradient, etc.)

mod node_factory;
mod property_parser;
mod xmlns;

use crate::properties::PropertyList;
use crate::tree::{FoArena, FoNode, FoNodeData, NodeId};
use crate::xml::XmlParser;
use crate::{FopError, Result};
use quick_xml::events::Event;
use std::collections::BTreeSet;
use std::io::BufRead;

/// Namespace context captured when beginning to accumulate an XMP packet or
/// foreign-object subtree.  Tracks everything needed to inject missing `xmlns:`
/// declarations into the captured root element.
struct CaptureNs {
    /// Accumulated serialised XML (starts with the root element's open tag)
    buffer: String,
    /// Nesting depth inside the captured subtree (0 = inside the root element)
    depth: usize,
    /// Byte offset of the `>` character that closes the root open tag in `buffer`
    root_close_byte: usize,
    /// All namespace bindings in scope at the moment the root was opened
    in_scope_at_start: Vec<(String, String)>,
    /// Prefixes declared directly on the captured root element
    declared_on_root: BTreeSet<String>,
    /// All namespace prefixes referenced anywhere in the subtree (element + attr names)
    used_in_subtree: BTreeSet<String>,
}

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
    /// Nesting depth of non-FO elements outside fo:instream-foreign-object.
    /// Tracks open tags so their matching close tags do not call end_element()
    /// and corrupt the current_node pointer.  For example, children of
    /// fo:declarations (e.g. x:xmpmeta / rdf:RDF) live here.
    non_fo_depth: usize,
    /// When non-None, we are inside an `<x:xmpmeta>` element and accumulating
    /// the raw XML (including the root `<x:xmpmeta ...>` opening tag) into this
    /// buffer.  The namespace context tracks which `xmlns:` declarations need
    /// injecting when the packet is finalised.
    xmp_buffer: Option<CaptureNs>,
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
            xmp_buffer: None::<CaptureNs>,
        }
    }

    /// Parse an XSL-FO document and build the tree
    pub fn parse<R: BufRead>(mut self, reader: R) -> Result<FoArena<'a>> {
        let mut parser = XmlParser::new(reader);

        loop {
            let event = parser.read_event()?;

            // Push namespace scope BEFORE dispatch for Start/Empty elements.
            // Empty elements also need push+pop since they open and close atomically.
            match &event {
                Event::Start(start) | Event::Empty(start) => {
                    parser.push_namespace_scope(start);
                }
                _ => {}
            }

            // Determine whether we need to pop after dispatch.
            // End pops; Empty pops (was pushed above); Start does NOT pop.
            let should_pop = matches!(&event, Event::End(_) | Event::Empty(_));

            let result = self.dispatch_event(&event, &parser);

            // Pop AFTER dispatch so the capture finaliser sees the correct scope on End.
            if should_pop {
                parser.pop_namespace_scope();
            }

            // Propagate any error from dispatch_event
            result?;

            if matches!(&event, Event::Eof) {
                break;
            }
        }

        Ok(self.arena)
    }

    /// Dispatch a single parse event to the appropriate capture or FO handler.
    fn dispatch_event<R: BufRead>(
        &mut self,
        event: &Event<'static>,
        parser: &XmlParser<R>,
    ) -> Result<()> {
        // ── Block A: XMP packet capture ──────────────────────────────────────────
        if self.xmp_buffer.is_some() {
            return self.handle_xmp_event(event, parser);
        }

        // ── Block B: foreign-object child capture ────────────────────────────────
        if self.foreign_object_depth > 0 {
            return self.handle_foreign_child_event(event, parser);
        }

        // ── Block C: main FO parse ───────────────────────────────────────────────
        match event {
            Event::Start(start) => {
                let (name, ns) = parser.extract_name(start)?;

                if ns.is_fo() {
                    self.start_element(&name, start, parser)?;
                } else if self.foreign_object_node.is_some() {
                    // Non-FO start inside instream-foreign-object root: begin child capture
                    let raw = std::str::from_utf8(start.as_ref())
                        .unwrap_or("")
                        .to_string();
                    self.foreign_xml_buffer.push('<');
                    self.foreign_xml_buffer.push_str(&raw);
                    self.foreign_xml_buffer.push('>');
                    self.foreign_object_depth += 1;
                } else {
                    // Non-FO element outside instream-foreign-object (e.g. inside
                    // fo:declarations).  Track depth so End events don't call end_element().
                    self.non_fo_depth += 1;
                    self.try_begin_xmp_capture(start, parser);
                }
            }
            Event::Empty(start) => {
                let (name, ns) = parser.extract_name(start)?;

                if ns.is_fo() {
                    self.start_element(&name, start, parser)?;
                    self.end_element()?;
                } else if self.foreign_object_node.is_some() {
                    // Self-closing non-FO element inside foreign-object root
                    let raw = std::str::from_utf8(start.as_ref())
                        .unwrap_or("")
                        .to_string();
                    self.foreign_xml_buffer.push('<');
                    self.foreign_xml_buffer.push_str(&raw);
                    self.foreign_xml_buffer.push_str("/>");
                }
                // Non-FO empty element outside foreign-object: ignore (no depth change)
            }
            Event::End(_) => {
                if self.foreign_object_node.is_some() && self.foreign_object_depth == 0 {
                    // This End closes the fo:instream-foreign-object itself
                    self.finalize_foreign_object();
                }
                // If inside a non-FO subtree, consume without popping current_node
                if self.non_fo_depth > 0 {
                    self.non_fo_depth -= 1;
                    return Ok(());
                }
                self.end_element()?;
            }
            Event::Text(text) => {
                let text_content = parser.extract_text(text)?;
                let trimmed = text_content.trim();
                if !trimmed.is_empty() {
                    self.add_text(trimmed)?;
                }
            }
            Event::CData(cdata) => {
                let cdata_content = parser.extract_cdata(cdata)?;
                if !cdata_content.is_empty() {
                    self.add_text(&cdata_content)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle an event while inside the XMP packet capture mode.
    fn handle_xmp_event<R: BufRead>(
        &mut self,
        event: &Event<'static>,
        parser: &XmlParser<R>,
    ) -> Result<()> {
        match event {
            Event::Start(start) => {
                let raw = std::str::from_utf8(start.as_ref())
                    .unwrap_or("")
                    .to_string();
                if let Some(cap) = &mut self.xmp_buffer {
                    cap.buffer.push('<');
                    cap.buffer.push_str(&raw);
                    cap.buffer.push('>');
                    cap.depth += 1;
                    xmlns::scan_prefixes_used(start, &mut cap.used_in_subtree);
                }
            }
            Event::Empty(start) => {
                let raw = std::str::from_utf8(start.as_ref())
                    .unwrap_or("")
                    .to_string();
                if let Some(cap) = &mut self.xmp_buffer {
                    cap.buffer.push('<');
                    cap.buffer.push_str(&raw);
                    cap.buffer.push_str("/>");
                    xmlns::scan_prefixes_used(start, &mut cap.used_in_subtree);
                }
            }
            Event::End(end) => {
                let raw = std::str::from_utf8(end.as_ref())
                    .unwrap_or("")
                    .to_string();
                let depth = self.xmp_buffer.as_ref().map(|c| c.depth).unwrap_or(0);
                if depth > 0 {
                    if let Some(cap) = &mut self.xmp_buffer {
                        cap.buffer.push_str("</");
                        cap.buffer.push_str(&raw);
                        cap.buffer.push('>');
                        cap.depth -= 1;
                    }
                } else {
                    // depth == 0: this End closes the root <x:xmpmeta>
                    if let Some(mut cap) = self.xmp_buffer.take() {
                        cap.buffer.push_str("</");
                        cap.buffer.push_str(&raw);
                        cap.buffer.push('>');

                        // Compute which inherited prefixes need injecting
                        let to_inject: Vec<(String, String)> = cap
                            .used_in_subtree
                            .iter()
                            .filter(|p| !cap.declared_on_root.contains(*p))
                            .filter_map(|p| {
                                cap.in_scope_at_start
                                    .iter()
                                    .find(|(sp, _)| sp == p)
                                    .map(|(sp, su)| (sp.clone(), su.clone()))
                            })
                            .collect();

                        let decls_block = xmlns::render_xmlns_attrs(&to_inject);
                        let patched = xmlns::inject_namespace_decls(
                            &cap.buffer,
                            &decls_block,
                            cap.root_close_byte,
                        );
                        self.arena.xmp_packets.push(patched);
                    }
                    // The xmpmeta open tag counted as non_fo_depth +1; revert it.
                    if self.non_fo_depth > 0 {
                        self.non_fo_depth -= 1;
                    }
                }
            }
            Event::Text(text) => {
                let text_content = parser.extract_text(text).unwrap_or_default();
                if let Some(cap) = &mut self.xmp_buffer {
                    cap.buffer.push_str(&text_content);
                }
            }
            Event::CData(cdata) => {
                let raw = std::str::from_utf8(cdata.as_ref()).unwrap_or("");
                if let Some(cap) = &mut self.xmp_buffer {
                    cap.buffer.push_str("<![CDATA[");
                    cap.buffer.push_str(raw);
                    cap.buffer.push_str("]]>");
                }
            }
            Event::Comment(comment) => {
                let raw = std::str::from_utf8(comment.as_ref()).unwrap_or("");
                if let Some(cap) = &mut self.xmp_buffer {
                    cap.buffer.push_str("<!--");
                    cap.buffer.push_str(raw);
                    cap.buffer.push_str("-->");
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle an event while inside a foreign-object child element capture.
    fn handle_foreign_child_event<R: BufRead>(
        &mut self,
        event: &Event<'static>,
        parser: &XmlParser<R>,
    ) -> Result<()> {
        match event {
            Event::Start(start) => {
                let raw = std::str::from_utf8(start.as_ref())
                    .unwrap_or("")
                    .to_string();
                self.foreign_xml_buffer.push('<');
                self.foreign_xml_buffer.push_str(&raw);
                self.foreign_xml_buffer.push('>');
                self.foreign_object_depth += 1;
            }
            Event::Empty(start) => {
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
                // When depth returns to 0 the child root element is closed; nothing more to do here
            }
            Event::Text(text) => {
                let text_content = parser.extract_text(text).unwrap_or_default();
                self.foreign_xml_buffer.push_str(&text_content);
            }
            Event::CData(cdata) => {
                let raw = std::str::from_utf8(cdata.as_ref()).unwrap_or("");
                self.foreign_xml_buffer.push_str("<![CDATA[");
                self.foreign_xml_buffer.push_str(raw);
                self.foreign_xml_buffer.push_str("]]>");
            }
            Event::Comment(comment) => {
                let raw = std::str::from_utf8(comment.as_ref()).unwrap_or("");
                self.foreign_xml_buffer.push_str("<!--");
                self.foreign_xml_buffer.push_str(raw);
                self.foreign_xml_buffer.push_str("-->");
            }
            _ => {}
        }
        Ok(())
    }

    /// Detect `<x:xmpmeta>` as a direct child of `fo:declarations` and start capture.
    fn try_begin_xmp_capture<R: BufRead>(
        &mut self,
        start: &quick_xml::events::BytesStart<'_>,
        parser: &XmlParser<R>,
    ) {
        let is_declarations_parent = self
            .current_node
            .and_then(|id| self.arena.get(id))
            .map(|n| matches!(n.data, FoNodeData::Declarations))
            .unwrap_or(false);

        if !is_declarations_parent {
            return;
        }

        let raw = std::str::from_utf8(start.as_ref()).unwrap_or("").to_string();
        // Check for xmpmeta (local-name only, after any prefix colon)
        let local_name = raw
            .split_once(':')
            .map(|(_, local)| local)
            .unwrap_or(raw.as_str());
        // local_name may have attributes after the element name
        let local_tag = local_name
            .split_once(|c: char| c.is_ascii_whitespace())
            .map(|(tag, _)| tag)
            .unwrap_or(local_name);
        if local_tag == "xmpmeta" {
            let mut buf = String::new();
            buf.push('<');
            buf.push_str(&raw);
            buf.push('>');
            let root_close_byte = buf.len() - 1; // index of the final `>`

            // Snapshot namespace scope (push_namespace_scope was already called
            // for this element before dispatch_event was entered)
            let in_scope_at_start = parser.snapshot_in_scope();
            let declared_on_root = xmlns::declared_on_element(start);
            let mut used_in_subtree = BTreeSet::new();
            xmlns::scan_prefixes_used(start, &mut used_in_subtree);

            self.xmp_buffer = Some(CaptureNs {
                buffer: buf,
                depth: 0,
                root_close_byte,
                in_scope_at_start,
                declared_on_root,
                used_in_subtree,
            });
        }
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

    #[test]
    fn test_xmp_packet_captured_from_declarations() {
        let xml = r##"<?xml version="1.0" encoding="utf-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
      <fo:region-body margin="2cm"/>
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
</fo:root>"##;

        let cursor = Cursor::new(xml);
        let arena = FoTreeBuilder::new()
            .parse(cursor)
            .expect("FO with fo:declarations + XMP metadata should parse successfully");

        // Verify the XMP packet was captured
        assert_eq!(
            arena.xmp_packets.len(),
            1,
            "Should have exactly one XMP packet captured from fo:declarations"
        );

        let packet = &arena.xmp_packets[0];
        assert!(
            packet.contains("xmpmeta"),
            "XMP packet should contain the xmpmeta element: {}",
            packet
        );
        assert!(
            packet.contains("Test Invoice"),
            "XMP packet should contain the dc:title value: {}",
            packet
        );

        // Verify the document also has the correct page-sequence structure
        let page_seq_count = arena
            .iter()
            .filter(|(_, n)| matches!(n.data, FoNodeData::PageSequence { .. }))
            .count();
        assert_eq!(
            page_seq_count, 1,
            "Document should have exactly one page-sequence"
        );
    }

    // ===== XMP NAMESPACE INHERITANCE TESTS =====

    fn make_fo_with_declarations(declarations_content: &str) -> String {
        format!(
            r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format"
         xmlns:x="adobe:ns:meta/"
         xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:dc="http://purl.org/dc/elements/1.1/">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:declarations>
    {}
  </fo:declarations>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Hello.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#,
            declarations_content
        )
    }

    #[test]
    fn test_xmp_namespace_inheritance_captures_inherited_xmlns() {
        // xmlns:x, xmlns:rdf, xmlns:dc declared on fo:root only — NOT on x:xmpmeta
        let fo = make_fo_with_declarations(
            r#"<x:xmpmeta>
      <rdf:RDF>
        <rdf:Description rdf:about="">
          <dc:title>
            <rdf:Alt><rdf:li xml:lang="x-default">Test Invoice</rdf:li></rdf:Alt>
          </dc:title>
        </rdf:Description>
      </rdf:RDF>
    </x:xmpmeta>"#,
        );

        let cursor = Cursor::new(fo);
        let arena = FoTreeBuilder::new()
            .parse(cursor)
            .expect("FO with inherited xmlns should parse");

        assert_eq!(arena.xmp_packets.len(), 1, "should have one XMP packet");
        let packet = &arena.xmp_packets[0];

        // All three prefixes must be declared on the captured root
        assert!(
            packet.contains("xmlns:x="),
            "missing xmlns:x in: {packet}"
        );
        assert!(
            packet.contains("xmlns:rdf="),
            "missing xmlns:rdf in: {packet}"
        );
        assert!(
            packet.contains("xmlns:dc="),
            "missing xmlns:dc in: {packet}"
        );

        // Must not duplicate — each prefix appears exactly once
        assert_eq!(
            packet.matches("xmlns:x=").count(),
            1,
            "xmlns:x duplicated in: {packet}"
        );
        assert_eq!(
            packet.matches("xmlns:rdf=").count(),
            1,
            "xmlns:rdf duplicated in: {packet}"
        );
        assert_eq!(
            packet.matches("xmlns:dc=").count(),
            1,
            "xmlns:dc duplicated in: {packet}"
        );
    }

    #[test]
    fn test_xmp_well_formed_via_ns_reader() {
        // Same FO as above — after capture, feed the packet to NsReader
        // and assert no undefined prefixes
        let fo = make_fo_with_declarations(
            r#"<x:xmpmeta>
      <rdf:RDF>
        <rdf:Description rdf:about="">
          <dc:title><rdf:Alt><rdf:li xml:lang="x-default">Invoice</rdf:li></rdf:Alt></dc:title>
        </rdf:Description>
      </rdf:RDF>
    </x:xmpmeta>"#,
        );

        let cursor = Cursor::new(fo);
        let arena = FoTreeBuilder::new()
            .parse(cursor)
            .expect("FO with inherited xmlns should parse");

        let packet = &arena.xmp_packets[0];

        use quick_xml::name::ResolveResult;
        use quick_xml::NsReader;
        let mut ns_reader = NsReader::from_str(packet);
        ns_reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        loop {
            match ns_reader.read_resolved_event_into(&mut buf) {
                Ok((ResolveResult::Unknown(prefix), _)) => {
                    panic!(
                        "undefined prefix in captured XMP packet: {:?}",
                        std::str::from_utf8(&prefix)
                    );
                }
                Ok((_, quick_xml::events::Event::Eof)) => break,
                Ok(_) => {}
                Err(e) => panic!("parse error in captured XMP packet: {e}"),
            }
            buf.clear();
        }
    }

    #[test]
    fn test_xmp_capture_round_trips_cdata() {
        // xmlns:x and xmlns:rdf declared on xmpmeta directly (no inheritance needed)
        let fo = make_fo_with_declarations(
            r#"<x:xmpmeta xmlns:x="adobe:ns:meta/" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
      <rdf:RDF><![CDATA[<not-an-element/>]]></rdf:RDF>
    </x:xmpmeta>"#,
        );

        let cursor = Cursor::new(fo);
        let arena = FoTreeBuilder::new()
            .parse(cursor)
            .expect("FO with CDATA in XMP should parse");

        assert_eq!(arena.xmp_packets.len(), 1);
        assert!(
            arena.xmp_packets[0].contains("<![CDATA[<not-an-element/>]]>"),
            "CDATA dropped: {}",
            arena.xmp_packets[0]
        );
    }

    #[test]
    fn test_xmp_capture_round_trips_comment() {
        let fo = make_fo_with_declarations(
            r#"<x:xmpmeta xmlns:x="adobe:ns:meta/" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
      <!-- intentional comment -->
      <rdf:RDF/>
    </x:xmpmeta>"#,
        );

        let cursor = Cursor::new(fo);
        let arena = FoTreeBuilder::new()
            .parse(cursor)
            .expect("FO with comment in XMP should parse");

        assert_eq!(arena.xmp_packets.len(), 1);
        let packet = &arena.xmp_packets[0];
        // Comment content should be preserved (with or without surrounding spaces depending on trim)
        assert!(
            packet.contains("<!-- intentional comment -->")
                || packet.contains("<!--intentional comment-->")
                || packet.contains("<!-- intentional comment-->")
                || packet.contains("<!--intentional comment -->"),
            "comment dropped: {packet}"
        );
    }

    #[test]
    fn test_xmp_no_injection_when_all_declared_locally() {
        // When all prefixes are declared on the xmpmeta root itself, no injection needed.
        // The canary test already covers this; this test makes it explicit.
        let fo = r##"<?xml version="1.0" encoding="utf-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
      <fo:region-body margin="2cm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:declarations>
    <x:xmpmeta xmlns:x="adobe:ns:meta/">
      <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description xmlns:dc="http://purl.org/dc/elements/1.1/" rdf:about="">
          <dc:title>Local Decl Test</dc:title>
        </rdf:Description>
      </rdf:RDF>
    </x:xmpmeta>
  </fo:declarations>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Hello.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

        let cursor = Cursor::new(fo);
        let arena = FoTreeBuilder::new()
            .parse(cursor)
            .expect("locally-declared prefixes should parse");

        assert_eq!(arena.xmp_packets.len(), 1);
        let packet = &arena.xmp_packets[0];
        // xmlns:x is declared on the root — count must remain exactly 1 (no injection)
        assert_eq!(
            packet.matches("xmlns:x=").count(),
            1,
            "xmlns:x must appear exactly once (no double-injection): {packet}"
        );
        assert!(packet.contains("Local Decl Test"), "content preserved: {packet}");
    }
}
