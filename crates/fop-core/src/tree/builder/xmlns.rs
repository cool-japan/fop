//! Namespace-prefix helpers for the FO tree builder.
//!
//! Used to inject ancestor `xmlns:*` declarations into captured XMP and
//! foreign-object subtrees so that the captured fragments are standalone
//! well-formed XML.

use std::collections::BTreeSet;

use quick_xml::events::BytesStart;

/// Extract the namespace prefix from a qualified element/attribute name (raw bytes).
///
/// Returns:
/// - `""` for unqualified names (`xmpmeta`) → default namespace
/// - `"foo"` for `foo:bar`
/// - `""` for malformed cases like `foo:` (trailing colon)
pub(super) fn extract_element_prefix(qname: &[u8]) -> &str {
    match qname.iter().position(|&b| b == b':') {
        None => "",     // no colon → default namespace
        Some(0) => "",  // leading colon → malformed, treat as default
        Some(pos) => {
            if pos + 1 >= qname.len() {
                "" // trailing colon → malformed, treat as default
            } else {
                std::str::from_utf8(&qname[..pos]).unwrap_or("")
            }
        }
    }
}

/// Collect all namespace prefixes *used* (not declared) by the element name
/// and its non-xmlns attributes, inserting them into `used`.
///
/// The default namespace is tracked as `""`.  Unprefixed attributes are NOT
/// in any namespace per the XML Namespaces spec, so they are not inserted.
pub(super) fn scan_prefixes_used(start: &BytesStart<'_>, used: &mut BTreeSet<String>) {
    // Element prefix
    let name = start.name();
    let elem_prefix = extract_element_prefix(name.as_ref());
    used.insert(elem_prefix.to_string());

    // Attribute prefixes (skip xmlns declarations)
    for attr in start.attributes().with_checks(false).flatten() {
        let key = attr.key.as_ref();
        if key == b"xmlns" || key.starts_with(b"xmlns:") {
            continue;
        }
        let attr_prefix = extract_element_prefix(key);
        // Only track prefixed attributes (unprefixed attrs are namespace-less)
        if !attr_prefix.is_empty() {
            used.insert(attr_prefix.to_string());
        }
    }
}

/// Return the set of prefix strings declared directly on this element via
/// `xmlns="…"` (empty-string prefix) or `xmlns:foo="…"` attributes.
pub(super) fn declared_on_element(start: &BytesStart<'_>) -> BTreeSet<String> {
    let mut declared = BTreeSet::new();
    for attr in start.attributes().with_checks(false).flatten() {
        let key = attr.key.as_ref();
        if key == b"xmlns" {
            declared.insert(String::new());
        } else if let Some(suffix) = key.strip_prefix(b"xmlns:") {
            if let Ok(s) = std::str::from_utf8(suffix) {
                declared.insert(s.to_string());
            }
        }
    }
    declared
}

/// Build the `xmlns:foo="bar" xmlns:rdf="baz"` fragment (with leading space)
/// to splice into a captured root's open tag.
///
/// `decls` should already be filtered to "needed but not declared on the root".
/// Sorted by prefix for determinism.  Empty-string prefix → bare `xmlns="…"`.
pub(super) fn render_xmlns_attrs(decls: &[(String, String)]) -> String {
    if decls.is_empty() {
        return String::new();
    }
    let mut sorted = decls.to_vec();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = String::new();
    for (prefix, uri) in &sorted {
        if prefix.is_empty() {
            out.push_str(&format!(r#" xmlns="{}""#, uri));
        } else {
            out.push_str(&format!(r#" xmlns:{}="{}""#, prefix, uri));
        }
    }
    out
}

/// Splice `decls_block` (the result of `render_xmlns_attrs`) into `open_tag`
/// just before the closing `>` or `/>`.
///
/// `open_tag` is the serialised opening element, e.g. `<x:xmpmeta rdf:about="">`.
/// `root_close_byte` is the byte-index of the final `>` character in `open_tag`.
pub(super) fn inject_namespace_decls(
    open_tag: &str,
    decls_block: &str,
    root_close_byte: usize,
) -> String {
    if decls_block.is_empty() {
        return open_tag.to_string();
    }
    // Clamp to valid range
    let byte_pos = root_close_byte.min(open_tag.len().saturating_sub(1));
    // If it's a self-closer `/>`  insert before the `/`
    let insert_pos = if byte_pos > 0
        && open_tag.as_bytes().get(byte_pos.saturating_sub(1)) == Some(&b'/')
    {
        byte_pos - 1
    } else {
        byte_pos
    };
    let mut result = open_tag.to_string();
    result.insert_str(insert_pos, decls_block);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_element_prefix_no_colon() {
        assert_eq!(extract_element_prefix(b"xmpmeta"), "");
    }

    #[test]
    fn test_extract_element_prefix_with_colon() {
        assert_eq!(extract_element_prefix(b"x:xmpmeta"), "x");
    }

    #[test]
    fn test_extract_element_prefix_trailing_colon() {
        assert_eq!(extract_element_prefix(b"x:"), "");
    }

    #[test]
    fn test_extract_element_prefix_leading_colon() {
        assert_eq!(extract_element_prefix(b":foo"), "");
    }

    #[test]
    fn test_render_xmlns_attrs_empty() {
        assert_eq!(render_xmlns_attrs(&[]), "");
    }

    #[test]
    fn test_render_xmlns_attrs_sorted() {
        let decls = vec![
            (
                "rdf".to_string(),
                "http://www.w3.org/1999/02/22-rdf-syntax-ns#".to_string(),
            ),
            (
                "dc".to_string(),
                "http://purl.org/dc/elements/1.1/".to_string(),
            ),
            ("x".to_string(), "adobe:ns:meta/".to_string()),
        ];
        let out = render_xmlns_attrs(&decls);
        // dc < rdf < x alphabetically
        let dc_pos = out.find("xmlns:dc").expect("test: dc must be present");
        let rdf_pos = out.find("xmlns:rdf").expect("test: rdf must be present");
        let x_pos = out.find("xmlns:x").expect("test: x must be present");
        assert!(dc_pos < rdf_pos, "dc should come before rdf");
        assert!(rdf_pos < x_pos, "rdf should come before x");
    }

    #[test]
    fn test_render_xmlns_attrs_default_namespace() {
        let decls = vec![("".to_string(), "http://example.com/".to_string())];
        let out = render_xmlns_attrs(&decls);
        assert!(
            out.contains(r#"xmlns="http://example.com/""#),
            "got: {out}"
        );
        assert!(!out.contains("xmlns:"), "bare xmlns should not have colon");
    }

    #[test]
    fn test_inject_namespace_decls_basic() {
        let open_tag = r#"<x:xmpmeta>"#;
        let root_close_byte = open_tag.len() - 1; // index of `>`
        let decls_block = r#" xmlns:x="adobe:ns:meta/""#;
        let result = inject_namespace_decls(open_tag, decls_block, root_close_byte);
        assert!(result.starts_with(r#"<x:xmpmeta xmlns:x=""#), "got: {result}");
        assert!(result.ends_with('>'), "got: {result}");
    }

    #[test]
    fn test_inject_namespace_decls_self_closing() {
        let open_tag = r#"<x:foo/>"#;
        let root_close_byte = open_tag.len() - 1; // index of `>`
        let decls_block = r#" xmlns:x="test""#;
        let result = inject_namespace_decls(open_tag, decls_block, root_close_byte);
        assert!(
            result.contains(r#" xmlns:x="test"/>"#),
            "got: {result}"
        );
    }

    #[test]
    fn test_inject_namespace_decls_empty_decls() {
        let open_tag = r#"<x:xmpmeta>"#;
        let result = inject_namespace_decls(open_tag, "", open_tag.len() - 1);
        assert_eq!(result, open_tag);
    }

    #[test]
    fn test_scan_prefixes_used_element_only() {
        let start = BytesStart::from_content(
            r#"rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#""#,
            7,
        );
        let mut used = BTreeSet::new();
        scan_prefixes_used(&start, &mut used);
        assert!(used.contains("rdf"), "element prefix rdf must be detected");
        // xmlns:rdf is a declaration, not a use — should not appear
        assert_eq!(used.len(), 1);
    }

    #[test]
    fn test_scan_prefixes_used_with_attr() {
        let start =
            BytesStart::from_content(r#"rdf:Description rdf:about="" dc:title="foo""#, 15);
        let mut used = BTreeSet::new();
        scan_prefixes_used(&start, &mut used);
        assert!(used.contains("rdf"), "element prefix");
        assert!(used.contains("dc"), "dc from attribute");
    }

    #[test]
    fn test_declared_on_element_empty() {
        let start = BytesStart::from_content(r#"rdf:RDF rdf:about="""#, 7);
        let declared = declared_on_element(&start);
        assert!(declared.is_empty());
    }

    #[test]
    fn test_declared_on_element_with_xmlns() {
        let start = BytesStart::from_content(
            r#"x:xmpmeta xmlns:x="adobe:ns:meta/" xmlns:rdf="http://example.com/""#,
            9,
        );
        let declared = declared_on_element(&start);
        assert!(declared.contains("x"), "x prefix must be declared");
        assert!(declared.contains("rdf"), "rdf prefix must be declared");
        assert_eq!(declared.len(), 2);
    }
}
