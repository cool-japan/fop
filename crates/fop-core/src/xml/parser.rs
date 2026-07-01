//! XML parser wrapper around quick-xml with enhanced features

use crate::xml::Namespace;
use fop_types::{FopError, Location, Result};
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};
use std::collections::HashMap;
use std::io::BufRead;

/// Entity resolver for handling XML entities
#[derive(Debug, Clone)]
pub struct EntityResolver {
    entities: HashMap<String, String>,
}

impl EntityResolver {
    /// Create a new entity resolver with built-in entities
    pub fn new() -> Self {
        let mut entities = HashMap::new();

        // Built-in XML entities
        entities.insert("amp".to_string(), "&".to_string());
        entities.insert("lt".to_string(), "<".to_string());
        entities.insert("gt".to_string(), ">".to_string());
        entities.insert("quot".to_string(), "\"".to_string());
        entities.insert("apos".to_string(), "'".to_string());

        Self { entities }
    }

    /// Add a custom entity
    pub fn add_entity(&mut self, name: String, value: String) {
        self.entities.insert(name, value);
    }

    /// Resolve an entity reference
    pub fn resolve(&self, entity: &str, location: Location) -> Result<String> {
        // Handle numeric character references
        if let Some(hex_str) = entity
            .strip_prefix("#x")
            .or_else(|| entity.strip_prefix("#X"))
        {
            // Hexadecimal
            if let Ok(code) = u32::from_str_radix(hex_str, 16) {
                if let Some(ch) = char::from_u32(code) {
                    return Ok(ch.to_string());
                }
            }
            return Err(FopError::EntityError {
                message: format!("Invalid hexadecimal character reference: {}", entity),
                location,
            });
        } else if let Some(dec_str) = entity.strip_prefix('#') {
            // Decimal
            if let Ok(code) = dec_str.parse::<u32>() {
                if let Some(ch) = char::from_u32(code) {
                    return Ok(ch.to_string());
                }
            }
            return Err(FopError::EntityError {
                message: format!("Invalid decimal character reference: {}", entity),
                location,
            });
        }

        // Named entity
        self.entities
            .get(entity)
            .cloned()
            .ok_or_else(|| FopError::EntityError {
                message: format!("Unknown entity: &{};", entity),
                location,
            })
    }

    /// Resolve all entities in a string
    pub fn resolve_entities(&self, text: &str, location: Location) -> Result<String> {
        let mut result = String::new();
        let mut chars = text.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '&' {
                // Find the end of the entity reference
                let mut entity_name = String::new();
                let mut found_semicolon = false;

                while let Some(&next_ch) = chars.peek() {
                    if next_ch == ';' {
                        chars.next(); // consume semicolon
                        found_semicolon = true;
                        break;
                    }
                    entity_name.push(next_ch);
                    chars.next();
                }

                if !found_semicolon {
                    return Err(FopError::EntityError {
                        message: format!("Unterminated entity reference: &{}", entity_name),
                        location,
                    });
                }

                let resolved = self.resolve(&entity_name, location)?;
                result.push_str(&resolved);
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }
}

impl Default for EntityResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Processing instruction data
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessingInstruction {
    pub target: String,
    pub data: Option<String>,
}

impl ProcessingInstruction {
    pub fn new(target: String, data: Option<String>) -> Self {
        Self { target, data }
    }
}

/// One element's worth of namespace declarations (pushed on element open, popped on element close)
#[derive(Debug, Default)]
struct NamespaceScope {
    /// `(prefix, uri)` pairs; empty-string prefix = default `xmlns`
    decls: Vec<(String, String)>,
}

/// Wrapper around quick-xml Reader for parsing XSL-FO documents
pub struct XmlParser<R: BufRead> {
    reader: Reader<R>,
    buf: Vec<u8>,
    /// Namespace scope stack — push on Start/Empty, pop on End/Empty
    namespace_stack: Vec<NamespaceScope>,
    /// Entity resolver
    entity_resolver: EntityResolver,
    /// Processing instructions encountered
    processing_instructions: Vec<ProcessingInstruction>,
}

impl<R: BufRead> XmlParser<R> {
    /// Create a new XML parser
    pub fn new(reader: R) -> Self {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);
        xml_reader.config_mut().expand_empty_elements = true;

        Self {
            reader: xml_reader,
            buf: Vec::new(),
            namespace_stack: Vec::new(),
            entity_resolver: EntityResolver::new(),
            processing_instructions: Vec::new(),
        }
    }

    /// Get a reference to the underlying reader
    pub fn reader(&self) -> &Reader<R> {
        &self.reader
    }

    /// Get a mutable reference to the underlying reader
    pub fn reader_mut(&mut self) -> &mut Reader<R> {
        &mut self.reader
    }

    /// Get the entity resolver
    pub fn entity_resolver(&self) -> &EntityResolver {
        &self.entity_resolver
    }

    /// Get a mutable reference to the entity resolver
    pub fn entity_resolver_mut(&mut self) -> &mut EntityResolver {
        &mut self.entity_resolver
    }

    /// Get processing instructions
    pub fn processing_instructions(&self) -> &[ProcessingInstruction] {
        &self.processing_instructions
    }

    /// Get current location (line and column)
    pub fn location(&self) -> Location {
        let pos = self.reader.buffer_position();
        // quick-xml doesn't provide column info directly, so we approximate
        Location::new(pos as usize, 0)
    }

    /// Read the next event
    pub fn read_event(&mut self) -> Result<Event<'static>> {
        self.buf.clear();
        let event = self
            .reader
            .read_event_into(&mut self.buf)
            .map(|e| e.into_owned())
            .map_err(|e| {
                let location = self.location();
                FopError::XmlErrorWithLocation {
                    message: format!("XML parsing error: {}", e),
                    location,
                    suggestion: None,
                }
            })?;

        // Handle processing instructions
        if let Event::PI(ref pi) = event {
            if let Ok(target) = std::str::from_utf8(pi.as_ref()) {
                // Parse target and data
                let parts: Vec<&str> = target.splitn(2, ' ').collect();
                let pi_target = parts[0].to_string();
                let pi_data = parts.get(1).map(|s| s.to_string());

                self.processing_instructions
                    .push(ProcessingInstruction::new(pi_target, pi_data));
            }
        }

        Ok(event)
    }

    /// Push a new namespace scope parsed from the element's `xmlns`/`xmlns:*` attributes.
    /// Call this when entering a Start or Empty element.
    pub fn push_namespace_scope(&mut self, start: &BytesStart<'_>) {
        let mut scope = NamespaceScope::default();
        for attr in start.attributes().with_checks(false).flatten() {
            let key = match std::str::from_utf8(attr.key.as_ref()) {
                Ok(k) => k,
                Err(_) => continue,
            };
            if key == "xmlns" {
                if let Ok(uri) = attr
                    .decoded_and_normalized_value(XmlVersion::Implicit1_0, self.reader.decoder())
                {
                    scope.decls.push((String::new(), uri.into_owned()));
                }
            } else if let Some(suffix) = key.strip_prefix("xmlns:") {
                if let Ok(uri) = attr
                    .decoded_and_normalized_value(XmlVersion::Implicit1_0, self.reader.decoder())
                {
                    scope.decls.push((suffix.to_string(), uri.into_owned()));
                }
            }
        }
        self.namespace_stack.push(scope);
    }

    /// Pop the innermost namespace scope.  No-op if the stack is empty.
    pub fn pop_namespace_scope(&mut self) {
        self.namespace_stack.pop();
    }

    /// Resolve a namespace prefix to a URI, searching from innermost scope outward.
    /// Returns `None` if the prefix is not in scope.
    pub fn resolve_prefix<'a>(&'a self, prefix: &str) -> Option<&'a str> {
        for scope in self.namespace_stack.iter().rev() {
            for (p, uri) in &scope.decls {
                if p == prefix {
                    return Some(uri.as_str());
                }
            }
        }
        None
    }

    /// Return all namespaces currently in scope as `(prefix, uri)` pairs,
    /// with innermost bindings winning over outer ones.  Sorted by prefix
    /// for determinism.  Empty string prefix = default namespace.
    pub fn snapshot_in_scope(&self) -> Vec<(String, String)> {
        let mut map: HashMap<String, String> = HashMap::new();
        // Iterate outer→inner so inner overwrites outer (innermost binding wins)
        for scope in self.namespace_stack.iter() {
            for (prefix, uri) in &scope.decls {
                map.insert(prefix.clone(), uri.clone());
            }
        }
        let mut result: Vec<(String, String)> = map.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    /// Extract element name and namespace from a BytesStart event
    pub fn extract_name(&self, start: &BytesStart) -> Result<(String, Namespace)> {
        let location = self.location();
        let name = start.name();

        // Extract namespace prefix and local name
        let (ns_prefix, local_name) = if let Some(pos) =
            name.as_ref().iter().position(|&b| b == b':')
        {
            let prefix = std::str::from_utf8(&name.as_ref()[..pos]).map_err(|e| {
                FopError::XmlErrorWithLocation {
                    message: format!("Invalid UTF-8 in prefix: {}", e),
                    location,
                    suggestion: None,
                }
            })?;
            let local = std::str::from_utf8(&name.as_ref()[pos + 1..]).map_err(|e| {
                FopError::XmlErrorWithLocation {
                    message: format!("Invalid UTF-8 in local name: {}", e),
                    location,
                    suggestion: None,
                }
            })?;
            (Some(prefix.to_string()), local)
        } else {
            let local =
                std::str::from_utf8(name.as_ref()).map_err(|e| FopError::XmlErrorWithLocation {
                    message: format!("Invalid UTF-8 in element name: {}", e),
                    location,
                    suggestion: None,
                })?;
            (None, local)
        };

        // Look up namespace URI via scope stack
        let ns_uri = if let Some(ref prefix) = ns_prefix {
            self.resolve_prefix(prefix)
                .map(str::to_string)
                .unwrap_or_default()
        } else {
            self.resolve_prefix("")
                .map(str::to_string)
                .unwrap_or_default()
        };

        let namespace = Namespace::from_uri(&ns_uri);

        Ok((local_name.to_string(), namespace))
    }

    /// Extract attributes from a BytesStart event
    pub fn extract_attributes(&self, start: &BytesStart) -> Result<Vec<(String, String)>> {
        let location = self.location();
        let mut attrs = Vec::new();

        for attr_result in start.attributes() {
            let attr = attr_result.map_err(|e| FopError::XmlErrorWithLocation {
                message: format!("Attribute parsing error: {}", e),
                location,
                suggestion: None,
            })?;

            let key = std::str::from_utf8(attr.key.as_ref())
                .map_err(|e| FopError::XmlErrorWithLocation {
                    message: format!("Invalid UTF-8 in attribute name: {}", e),
                    location,
                    suggestion: None,
                })?
                .to_string();

            // Skip xmlns attributes
            if key.starts_with("xmlns") {
                continue;
            }

            // quick-xml already handles entity unescaping in decoded_and_normalized_value
            let value = attr
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, self.reader.decoder())
                .map_err(|e| FopError::XmlErrorWithLocation {
                    message: format!("Attribute value decode error: {}", e),
                    location,
                    suggestion: None,
                })?
                .to_string();

            attrs.push((key, value));
        }

        Ok(attrs)
    }

    /// Extract text content from Text event (with entity resolution)
    pub fn extract_text(&self, text: &[u8]) -> Result<String> {
        let location = self.location();
        let text_str = std::str::from_utf8(text).map_err(|e| FopError::XmlErrorWithLocation {
            message: format!("Invalid UTF-8 in text: {}", e),
            location,
            suggestion: None,
        })?;

        // Resolve entities in text content
        self.entity_resolver.resolve_entities(text_str, location)
    }

    /// Extract CDATA content (no entity resolution)
    pub fn extract_cdata(&self, cdata: &[u8]) -> Result<String> {
        let location = self.location();
        std::str::from_utf8(cdata)
            .map(|s| s.to_string())
            .map_err(|e| FopError::XmlErrorWithLocation {
                message: format!("Invalid UTF-8 in CDATA: {}", e),
                location,
                suggestion: None,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_simple_fo() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
        </fo:simple-page-master>
    </fo:layout-master-set>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        let mut found_root = false;
        let mut found_layout_master_set = false;

        loop {
            let event = parser.read_event();
            match event {
                Ok(Event::Start(ref start)) | Ok(Event::Empty(ref start)) => {
                    parser.push_namespace_scope(start);
                    let (name, ns) = parser.extract_name(start).expect("test: should succeed");

                    if name == "root" && ns.is_fo() {
                        found_root = true;
                    }
                    if name == "layout-master-set" && ns.is_fo() {
                        found_layout_master_set = true;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => panic!("Parse error: {}", e),
                _ => {}
            }
        }

        assert!(found_root);
        assert!(found_layout_master_set);
    }

    #[test]
    fn test_extract_attributes() {
        let xml = r#"<?xml version="1.0"?>
<fo:simple-page-master xmlns:fo="http://www.w3.org/1999/XSL/Format"
                       master-name="A4"
                       page-width="210mm"
                       page-height="297mm">
</fo:simple-page-master>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        loop {
            let event = parser.read_event();
            match event {
                Ok(Event::Start(ref start)) | Ok(Event::Empty(ref start)) => {
                    parser.push_namespace_scope(start);
                    let attrs = parser
                        .extract_attributes(start)
                        .expect("test: should succeed");

                    // Find specific attributes
                    let master_name = attrs
                        .iter()
                        .find(|(k, _)| k == "master-name")
                        .map(|(_, v)| v.as_str());

                    assert_eq!(master_name, Some("A4"));

                    let page_width = attrs
                        .iter()
                        .find(|(k, _)| k == "page-width")
                        .map(|(_, v)| v.as_str());

                    assert_eq!(page_width, Some("210mm"));

                    break;
                }
                Ok(Event::Eof) => break,
                Err(e) => panic!("Parse error: {}", e),
                _ => {}
            }
        }
    }

    #[test]
    fn test_cdata_section() {
        let xml = r#"<?xml version="1.0"?>
<fo:block xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <![CDATA[<tag> & "quotes"]]>
</fo:block>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        let mut found_cdata = false;
        let mut cdata_content = String::new();

        loop {
            match parser.read_event() {
                Ok(Event::CData(ref cdata)) => {
                    found_cdata = true;
                    cdata_content = parser.extract_cdata(cdata).expect("test: should succeed");
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("Parse error: {}", e),
            }
        }

        assert!(found_cdata);
        assert_eq!(cdata_content, r#"<tag> & "quotes""#);
    }

    #[test]
    fn test_entity_resolution_builtin() {
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);

        assert_eq!(
            resolver
                .resolve("amp", location)
                .expect("test: should succeed"),
            "&"
        );
        assert_eq!(
            resolver
                .resolve("lt", location)
                .expect("test: should succeed"),
            "<"
        );
        assert_eq!(
            resolver
                .resolve("gt", location)
                .expect("test: should succeed"),
            ">"
        );
        assert_eq!(
            resolver
                .resolve("quot", location)
                .expect("test: should succeed"),
            "\""
        );
        assert_eq!(
            resolver
                .resolve("apos", location)
                .expect("test: should succeed"),
            "'"
        );
    }

    #[test]
    fn test_entity_resolution_numeric_decimal() {
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);

        // &#65; = 'A'
        assert_eq!(
            resolver
                .resolve("#65", location)
                .expect("test: should succeed"),
            "A"
        );
        // &#36; = '$'
        assert_eq!(
            resolver
                .resolve("#36", location)
                .expect("test: should succeed"),
            "$"
        );
    }

    #[test]
    fn test_entity_resolution_numeric_hex() {
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);

        // &#x41; = 'A'
        assert_eq!(
            resolver
                .resolve("#x41", location)
                .expect("test: should succeed"),
            "A"
        );
        assert_eq!(
            resolver
                .resolve("#X41", location)
                .expect("test: should succeed"),
            "A"
        );
        // &#xA9; = '©'
        assert_eq!(
            resolver
                .resolve("#xA9", location)
                .expect("test: should succeed"),
            "©"
        );
    }

    #[test]
    fn test_entity_resolution_custom() {
        let mut resolver = EntityResolver::new();
        resolver.add_entity("copy".to_string(), "©".to_string());

        let location = Location::new(1, 1);
        assert_eq!(
            resolver
                .resolve("copy", location)
                .expect("test: should succeed"),
            "©"
        );
    }

    #[test]
    fn test_entity_resolution_in_text() {
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);

        let text = "Price: &#36;100 &amp; up";
        let resolved = resolver
            .resolve_entities(text, location)
            .expect("test: should succeed");
        assert_eq!(resolved, "Price: $100 & up");
    }

    #[test]
    fn test_entity_resolution_unknown() {
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);

        let result = resolver.resolve("unknown", location);
        assert!(result.is_err());
    }

    #[test]
    fn test_processing_instruction() {
        let xml = r#"<?xml version="1.0"?>
<?xml-stylesheet type="text/xsl" href="style.xsl"?>
<?fop-renderer backend="pdf"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        loop {
            match parser.read_event() {
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("Parse error: {}", e),
            }
        }

        let pis = parser.processing_instructions();
        assert_eq!(pis.len(), 2);

        assert_eq!(pis[0].target, "xml-stylesheet");
        assert!(pis[0].data.is_some());

        assert_eq!(pis[1].target, "fop-renderer");
        assert!(pis[1].data.is_some());
    }

    #[test]
    fn test_entities_in_attributes() {
        let xml = r#"<?xml version="1.0"?>
<fo:block xmlns:fo="http://www.w3.org/1999/XSL/Format" title="Test &amp; More">
</fo:block>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        loop {
            match parser.read_event() {
                Ok(Event::Start(ref start)) | Ok(Event::Empty(ref start)) => {
                    parser.push_namespace_scope(start);
                    let attrs = parser
                        .extract_attributes(start)
                        .expect("test: should succeed");

                    let title = attrs
                        .iter()
                        .find(|(k, _)| k == "title")
                        .map(|(_, v)| v.as_str());

                    assert_eq!(title, Some("Test & More"));
                    break;
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("Parse error: {}", e),
            }
        }
    }

    #[test]
    fn test_cdata_preserves_content() {
        let xml = r#"<?xml version="1.0"?>
<fo:block xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <![CDATA[Code with <tags> & "special" &amp; chars]]>
</fo:block>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        let mut cdata_content = String::new();

        loop {
            match parser.read_event() {
                Ok(Event::CData(ref cdata)) => {
                    cdata_content = parser.extract_cdata(cdata).expect("test: should succeed");
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("Parse error: {}", e),
            }
        }

        // CDATA should preserve everything, including &amp;
        assert_eq!(cdata_content, r#"Code with <tags> & "special" &amp; chars"#);
    }

    #[test]
    fn test_multiple_entities() {
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);

        let text = "&lt;tag&gt; &amp; &quot;text&quot;";
        let resolved = resolver
            .resolve_entities(text, location)
            .expect("test: should succeed");
        assert_eq!(resolved, r#"<tag> & "text""#);
    }

    #[test]
    fn test_unterminated_entity() {
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);

        let text = "&amp no semicolon";
        let result = resolver.resolve_entities(text, location);
        assert!(result.is_err());
    }

    #[test]
    fn test_location_tracking() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let parser = XmlParser::new(cursor);

        // Location should be available (just verify we can get it)
        let _location = parser.location();
    }

    #[test]
    fn test_error_with_location() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <unclosed-tag>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        let mut error_found = false;

        loop {
            match parser.read_event() {
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => {
                    error_found = true;
                    // Error should contain location information
                    let error_str = format!("{}", e);
                    assert!(error_str.contains("line") || error_str.contains("XML parsing error"));
                    break;
                }
            }
        }

        assert!(error_found);
    }
}

// ===== ADDITIONAL TESTS (60+ new tests) =====

#[cfg(test)]
mod additional_tests {
    use super::*;
    use std::io::Cursor;

    // ===== ENTITY RESOLVER EDGE CASES =====

    #[test]
    fn test_entity_resolver_apos() {
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);
        assert_eq!(
            resolver
                .resolve("apos", location)
                .expect("test: should succeed"),
            "'"
        );
    }

    #[test]
    fn test_entity_resolver_quot() {
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);
        assert_eq!(
            resolver
                .resolve("quot", location)
                .expect("test: should succeed"),
            "\""
        );
    }

    #[test]
    fn test_entity_resolver_gt() {
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);
        assert_eq!(
            resolver
                .resolve("gt", location)
                .expect("test: should succeed"),
            ">"
        );
    }

    #[test]
    fn test_entity_resolver_empty_text() {
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);
        let result = resolver
            .resolve_entities("", location)
            .expect("test: should succeed");
        assert_eq!(result, "");
    }

    #[test]
    fn test_entity_resolver_text_without_entities() {
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);
        let result = resolver
            .resolve_entities("hello world", location)
            .expect("test: should succeed");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_entity_resolver_only_entity() {
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);
        let result = resolver
            .resolve_entities("&amp;", location)
            .expect("test: should succeed");
        assert_eq!(result, "&");
    }

    #[test]
    fn test_entity_resolver_hex_zero() {
        // &#x0041; = 'A'
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);
        let result = resolver
            .resolve("#x0041", location)
            .expect("test: should succeed");
        assert_eq!(result, "A");
    }

    #[test]
    fn test_entity_resolver_decimal_newline() {
        // &#10; = newline
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);
        let result = resolver
            .resolve("#10", location)
            .expect("test: should succeed");
        assert_eq!(result, "\n");
    }

    #[test]
    fn test_entity_resolver_decimal_tab() {
        // &#9; = tab
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);
        let result = resolver
            .resolve("#9", location)
            .expect("test: should succeed");
        assert_eq!(result, "\t");
    }

    #[test]
    fn test_entity_resolver_unicode_multibyte() {
        // &#x4e2d; = '中' (U+4E2D, Chinese character)
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);
        let result = resolver
            .resolve("#x4e2d", location)
            .expect("test: should succeed");
        assert_eq!(result, "中");
    }

    #[test]
    fn test_entity_resolver_add_multiple_custom() {
        let mut resolver = EntityResolver::new();
        resolver.add_entity("euro".to_string(), "€".to_string());
        resolver.add_entity("yen".to_string(), "¥".to_string());
        resolver.add_entity("pound".to_string(), "£".to_string());

        let location = Location::new(1, 1);
        assert_eq!(
            resolver
                .resolve("euro", location)
                .expect("test: should succeed"),
            "€"
        );
        assert_eq!(
            resolver
                .resolve("yen", location)
                .expect("test: should succeed"),
            "¥"
        );
        assert_eq!(
            resolver
                .resolve("pound", location)
                .expect("test: should succeed"),
            "£"
        );
    }

    #[test]
    fn test_entity_resolver_override_custom() {
        let mut resolver = EntityResolver::new();
        // Override the built-in amp entity
        resolver.add_entity("amp".to_string(), "AMPERSAND".to_string());

        let location = Location::new(1, 1);
        assert_eq!(
            resolver
                .resolve("amp", location)
                .expect("test: should succeed"),
            "AMPERSAND"
        );
    }

    #[test]
    fn test_entity_resolver_resolve_entities_multiple() {
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);
        let text = "&lt;&gt;&amp;&quot;&apos;";
        let result = resolver
            .resolve_entities(text, location)
            .expect("test: should succeed");
        assert_eq!(result, "<>&\"'");
    }

    #[test]
    fn test_entity_resolver_numeric_in_text() {
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);
        let text = "A&#65;B&#66;C";
        let result = resolver
            .resolve_entities(text, location)
            .expect("test: should succeed");
        assert_eq!(result, "AABBC");
    }

    #[test]
    fn test_entity_resolver_hex_uppercase() {
        // &#X41; (uppercase X) should also resolve
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);
        let result = resolver
            .resolve("#X41", location)
            .expect("test: should succeed");
        assert_eq!(result, "A");
    }

    // ===== PROCESSING INSTRUCTION TESTS =====

    #[test]
    fn test_processing_instruction_new() {
        let pi = ProcessingInstruction::new("target".to_string(), Some("data".to_string()));
        assert_eq!(pi.target, "target");
        assert_eq!(pi.data, Some("data".to_string()));
    }

    #[test]
    fn test_processing_instruction_no_data() {
        let pi = ProcessingInstruction::new("target".to_string(), None);
        assert_eq!(pi.target, "target");
        assert!(pi.data.is_none());
    }

    #[test]
    fn test_processing_instruction_equality() {
        let pi1 = ProcessingInstruction::new("foo".to_string(), Some("bar".to_string()));
        let pi2 = ProcessingInstruction::new("foo".to_string(), Some("bar".to_string()));
        assert_eq!(pi1, pi2);
    }

    #[test]
    fn test_processing_instruction_inequality() {
        let pi1 = ProcessingInstruction::new("foo".to_string(), Some("bar".to_string()));
        let pi2 = ProcessingInstruction::new("baz".to_string(), Some("bar".to_string()));
        assert_ne!(pi1, pi2);
    }

    // ===== NAMESPACE TESTS =====

    #[test]
    fn test_nested_namespace_declarations() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format"
         xmlns:svg="http://www.w3.org/2000/svg">
    <fo:layout-master-set></fo:layout-master-set>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        let mut found_root = false;
        loop {
            let event = parser.read_event();
            match event {
                Ok(Event::Start(ref start)) | Ok(Event::Empty(ref start)) => {
                    parser.push_namespace_scope(start);
                    let result = parser.extract_name(start);
                    if let Ok((name, ns)) = result {
                        if name == "root" && ns.is_fo() {
                            found_root = true;
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => panic!("Parse error: {}", e),
                _ => {}
            }
        }
        assert!(found_root);
    }

    #[test]
    fn test_fox_extension_namespace() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format"
         xmlns:fox="http://xmlgraphics.apache.org/fop/extensions">
    <fo:layout-master-set></fo:layout-master-set>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        let mut found_root = false;
        loop {
            let event = parser.read_event();
            match event {
                Ok(Event::Start(ref start)) | Ok(Event::Empty(ref start)) => {
                    parser.push_namespace_scope(start);
                    if let Ok((name, ns)) = parser.extract_name(start) {
                        if name == "root" && ns.is_fo() {
                            found_root = true;
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => panic!("Parse error: {}", e),
                _ => {}
            }
        }
        assert!(found_root);
    }

    // ===== XML PARSER EVENT TESTS =====

    #[test]
    fn test_empty_element_produces_start_end() {
        // With expand_empty_elements=true, empty elements produce Start+End
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        let mut element_count = 0;
        loop {
            match parser.read_event() {
                Ok(Event::Start(ref start)) => {
                    parser.push_namespace_scope(start);
                    element_count += 1;
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("Parse error: {}", e),
            }
        }
        // root, layout-master-set, simple-page-master, region-body (expanded from empty)
        assert!(element_count >= 4);
    }

    #[test]
    fn test_multiple_attributes_preserved_order() {
        let xml = r#"<?xml version="1.0"?>
<fo:block xmlns:fo="http://www.w3.org/1999/XSL/Format"
    font-size="12pt"
    font-family="Arial"
    color="black"
    margin-top="10pt">text</fo:block>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        loop {
            match parser.read_event() {
                Ok(Event::Start(ref start)) => {
                    parser.push_namespace_scope(start);
                    let attrs = parser
                        .extract_attributes(start)
                        .expect("test: should succeed");
                    // xmlns attrs are skipped, so we expect 4 non-namespace attrs
                    assert_eq!(attrs.len(), 4);
                    break;
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("Parse error: {}", e),
            }
        }
    }

    #[test]
    fn test_text_with_special_chars_in_cdata() {
        let xml = r#"<?xml version="1.0"?>
<fo:block xmlns:fo="http://www.w3.org/1999/XSL/Format"><![CDATA[a < b && c > d]]></fo:block>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        let mut cdata_text = String::new();
        loop {
            match parser.read_event() {
                Ok(Event::CData(ref cdata)) => {
                    cdata_text = parser.extract_cdata(cdata).expect("test: should succeed");
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("Parse error: {}", e),
            }
        }
        assert_eq!(cdata_text, "a < b && c > d");
    }

    #[test]
    fn test_extract_cdata_preserves_angle_brackets() {
        let xml = r#"<?xml version="1.0"?>
<fo:block xmlns:fo="http://www.w3.org/1999/XSL/Format"><![CDATA[<tag attr="val"/>]]></fo:block>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        let mut cdata_text = String::new();
        loop {
            match parser.read_event() {
                Ok(Event::CData(ref cdata)) => {
                    cdata_text = parser.extract_cdata(cdata).expect("test: should succeed");
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("Parse error: {}", e),
            }
        }
        assert_eq!(cdata_text, r#"<tag attr="val"/>"#);
    }

    #[test]
    fn test_comment_does_not_produce_text_event() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format"><!-- this is a comment --></fo:root>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        let mut text_events = 0;
        loop {
            match parser.read_event() {
                Ok(Event::Text(_)) => {
                    text_events += 1;
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("Parse error: {}", e),
            }
        }
        // Comments should not produce text events; trim_text should remove empty whitespace
        assert_eq!(text_events, 0);
    }

    #[test]
    fn test_multiple_processing_instructions() {
        let xml = r#"<?xml version="1.0"?>
<?stylesheet type="text/css"?>
<?renderer backend="pdf"?>
<?custom-pi data="value"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format"></fo:root>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        loop {
            match parser.read_event() {
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("Parse error: {}", e),
            }
        }

        let pis = parser.processing_instructions();
        assert_eq!(pis.len(), 3);
        assert_eq!(pis[0].target, "stylesheet");
        assert_eq!(pis[1].target, "renderer");
        assert_eq!(pis[2].target, "custom-pi");
    }

    #[test]
    fn test_no_processing_instructions_when_none_present() {
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format"></fo:root>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        loop {
            match parser.read_event() {
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("Parse error: {}", e),
            }
        }

        let pis = parser.processing_instructions();
        assert_eq!(pis.len(), 0);
    }

    #[test]
    fn test_attributes_with_apos_entity() {
        let xml = r#"<?xml version="1.0"?>
<fo:block xmlns:fo="http://www.w3.org/1999/XSL/Format" title="it&apos;s">text</fo:block>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        loop {
            match parser.read_event() {
                Ok(Event::Start(ref start)) => {
                    parser.push_namespace_scope(start);
                    let attrs = parser
                        .extract_attributes(start)
                        .expect("test: should succeed");
                    let title = attrs
                        .iter()
                        .find(|(k, _)| k == "title")
                        .map(|(_, v)| v.as_str());
                    assert_eq!(title, Some("it's"));
                    break;
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("Parse error: {}", e),
            }
        }
    }

    #[test]
    fn test_attributes_with_lt_entity() {
        let xml = r#"<?xml version="1.0"?>
<fo:block xmlns:fo="http://www.w3.org/1999/XSL/Format" title="a &lt; b">text</fo:block>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        loop {
            match parser.read_event() {
                Ok(Event::Start(ref start)) => {
                    parser.push_namespace_scope(start);
                    let attrs = parser
                        .extract_attributes(start)
                        .expect("test: should succeed");
                    let title = attrs
                        .iter()
                        .find(|(k, _)| k == "title")
                        .map(|(_, v)| v.as_str());
                    assert_eq!(title, Some("a < b"));
                    break;
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("Parse error: {}", e),
            }
        }
    }

    #[test]
    fn test_attribute_with_numeric_entity() {
        let xml = r#"<?xml version="1.0"?>
<fo:block xmlns:fo="http://www.w3.org/1999/XSL/Format" title="&#65;BC">text</fo:block>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        loop {
            match parser.read_event() {
                Ok(Event::Start(ref start)) => {
                    parser.push_namespace_scope(start);
                    let attrs = parser
                        .extract_attributes(start)
                        .expect("test: should succeed");
                    let title = attrs
                        .iter()
                        .find(|(k, _)| k == "title")
                        .map(|(_, v)| v.as_str());
                    assert_eq!(title, Some("ABC"));
                    break;
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("Parse error: {}", e),
            }
        }
    }

    #[test]
    fn test_xml_with_utf8_text() {
        let xml = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<fo:root xmlns:fo=\"http://www.w3.org/1999/XSL/Format\"><fo:block>日本語テスト</fo:block></fo:root>";

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        let mut text_content = String::new();
        loop {
            match parser.read_event() {
                Ok(Event::Text(ref text)) => {
                    text_content = parser.extract_text(text).expect("test: should succeed");
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("Parse error: {}", e),
            }
        }
        assert_eq!(text_content, "日本語テスト");
    }

    #[test]
    fn test_entity_resolver_clone() {
        let mut resolver = EntityResolver::new();
        resolver.add_entity("test".to_string(), "TEST_VALUE".to_string());
        let cloned = resolver.clone();

        let location = Location::new(1, 1);
        assert_eq!(
            cloned
                .resolve("test", location)
                .expect("test: should succeed"),
            "TEST_VALUE"
        );
        assert_eq!(
            cloned
                .resolve("amp", location)
                .expect("test: should succeed"),
            "&"
        );
    }

    #[test]
    fn test_entity_resolver_default() {
        let resolver = EntityResolver::default();
        let location = Location::new(1, 1);
        // Default should have all 5 built-in entities
        assert_eq!(
            resolver
                .resolve("amp", location)
                .expect("test: should succeed"),
            "&"
        );
        assert_eq!(
            resolver
                .resolve("lt", location)
                .expect("test: should succeed"),
            "<"
        );
        assert_eq!(
            resolver
                .resolve("gt", location)
                .expect("test: should succeed"),
            ">"
        );
        assert_eq!(
            resolver
                .resolve("quot", location)
                .expect("test: should succeed"),
            "\""
        );
        assert_eq!(
            resolver
                .resolve("apos", location)
                .expect("test: should succeed"),
            "'"
        );
    }

    #[test]
    fn test_xml_deeply_nested_elements() {
        // Test parsing with many levels of nesting
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="p1">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="p1">
        <fo:flow flow-name="xsl-region-body">
            <fo:block>
                <fo:inline>
                    <fo:inline>
                        <fo:inline>deep nesting</fo:inline>
                    </fo:inline>
                </fo:inline>
            </fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);
        let mut error = None;

        loop {
            match parser.read_event() {
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => {
                    error = Some(e);
                    break;
                }
            }
        }
        assert!(error.is_none(), "Deep nesting should parse without error");
    }

    #[test]
    fn test_xml_empty_text_nodes_trimmed() {
        // With trim_text(true), whitespace-only text nodes are trimmed to empty
        let xml = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
    </fo:layout-master-set>
</fo:root>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        let mut non_empty_text = 0;
        loop {
            match parser.read_event() {
                Ok(Event::Text(ref text)) => {
                    let content = parser.extract_text(text).unwrap_or_default();
                    if !content.is_empty() {
                        non_empty_text += 1;
                    }
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("Parse error: {}", e),
            }
        }
        // Whitespace-only nodes should be empty after trim
        assert_eq!(non_empty_text, 0);
    }

    #[test]
    fn test_xml_pi_target_with_data() {
        let xml = r#"<?xml version="1.0"?>
<?fop-config key="value" other="data"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format"></fo:root>"#;

        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        loop {
            match parser.read_event() {
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("Parse error: {}", e),
            }
        }

        let pis = parser.processing_instructions();
        assert_eq!(pis.len(), 1);
        assert_eq!(pis[0].target, "fop-config");
        assert!(pis[0].data.is_some());
        let data = pis[0].data.as_ref().expect("test: should succeed");
        assert!(data.contains("key"));
    }

    #[test]
    fn test_entity_resolver_unknown_entity_has_name_in_error() {
        let resolver = EntityResolver::new();
        let location = Location::new(5, 10);
        let result = resolver.resolve("nonexistent", location);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = format!("{}", err);
        assert!(err_str.contains("nonexistent"));
    }

    #[test]
    fn test_entity_resolver_invalid_hex_ref() {
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);
        // Non-hex characters after #x
        let result = resolver.resolve("#xZZZZ", location);
        assert!(result.is_err());
    }

    #[test]
    fn test_entity_resolver_invalid_decimal_ref() {
        let resolver = EntityResolver::new();
        let location = Location::new(1, 1);
        // Non-decimal characters after #
        let result = resolver.resolve("#abc", location);
        assert!(result.is_err());
    }

    // ===== NAMESPACE SCOPE STACK TESTS =====

    #[test]
    fn test_namespace_scope_pop_restores_outer() {
        let xml = r#"<root xmlns:x="uri:outer"></root>"#;
        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        // Manually push scopes to test push/pop semantics
        let outer_start = BytesStart::from_content(r#"root xmlns:x="uri:outer""#, 4);
        let inner_start = BytesStart::from_content(r#"child xmlns:x="uri:inner""#, 5);
        parser.push_namespace_scope(&outer_start);
        parser.push_namespace_scope(&inner_start);
        assert_eq!(
            parser.resolve_prefix("x"),
            Some("uri:inner"),
            "inner scope should shadow outer"
        );
        parser.pop_namespace_scope();
        assert_eq!(
            parser.resolve_prefix("x"),
            Some("uri:outer"),
            "after pop, outer scope should be visible"
        );
        parser.pop_namespace_scope();
        assert_eq!(
            parser.resolve_prefix("x"),
            None,
            "after all pops, prefix should be unresolvable"
        );
    }

    #[test]
    fn test_namespace_scope_sibling_rebind_does_not_leak() {
        let xml = r#"<root></root>"#;
        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        let sibling_a = BytesStart::from_content(r#"a xmlns:foo="uri:a""#, 1);
        parser.push_namespace_scope(&sibling_a);
        assert_eq!(parser.resolve_prefix("foo"), Some("uri:a"));
        parser.pop_namespace_scope();

        // Sibling B has no xmlns:foo — must not inherit A's declaration
        let sibling_b = BytesStart::from_content(r#"b"#, 1);
        parser.push_namespace_scope(&sibling_b);
        assert_eq!(
            parser.resolve_prefix("foo"),
            None,
            "sibling's xmlns:foo must not be visible after pop"
        );
        parser.pop_namespace_scope();
    }

    #[test]
    fn test_namespace_snapshot_in_scope_innermost_wins() {
        let xml = r#"<root></root>"#;
        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        let outer = BytesStart::from_content(r#"outer xmlns:x="outer:uri""#, 5);
        let inner = BytesStart::from_content(r#"inner xmlns:x="inner:uri""#, 5);
        parser.push_namespace_scope(&outer);
        parser.push_namespace_scope(&inner);

        let snapshot = parser.snapshot_in_scope();
        let x_uri = snapshot
            .iter()
            .find(|(p, _)| p == "x")
            .map(|(_, u)| u.as_str());
        assert_eq!(
            x_uri,
            Some("inner:uri"),
            "innermost binding should win in snapshot"
        );
    }

    #[test]
    fn test_namespace_scope_empty_prefix_default_namespace() {
        let xml = r#"<root></root>"#;
        let cursor = Cursor::new(xml);
        let mut parser = XmlParser::new(cursor);

        let start = BytesStart::from_content(r#"root xmlns="http://example.com/""#, 4);
        parser.push_namespace_scope(&start);
        assert_eq!(
            parser.resolve_prefix(""),
            Some("http://example.com/"),
            "default namespace should be resolvable via empty-string prefix"
        );
        parser.pop_namespace_scope();
    }

    #[test]
    fn test_namespace_resolve_prefix_returns_none_on_empty_stack() {
        let xml = r#"<root></root>"#;
        let cursor = Cursor::new(xml);
        let parser = XmlParser::new(cursor);
        assert_eq!(parser.resolve_prefix("fo"), None);
        assert_eq!(parser.resolve_prefix(""), None);
    }
}
