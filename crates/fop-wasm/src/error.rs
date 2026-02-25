//! Error mapping from FopError to JsValue for WASM

use wasm_bindgen::JsValue;

/// Convert a FopError into a JsValue for JavaScript consumption
pub fn fop_error_to_js(err: fop_types::FopError) -> JsValue {
    JsValue::from_str(&err.to_string())
}

/// Convert any error implementing Display to JsValue
pub fn error_to_js<E: std::fmt::Display>(err: E) -> JsValue {
    JsValue::from_str(&err.to_string())
}

// ============================================================================
// Unit tests
// ============================================================================
//
// Note: `JsValue::from_str` panics unconditionally on non-wasm32 targets
// (wasm-bindgen stubs are not implemented for native).  We therefore test the
// *Display* output of `FopError` — which is exactly what `fop_error_to_js` and
// `error_to_js` convert to a JsValue — and a few structural properties instead.
// The JsValue-producing functions are covered by the converter integration tests
// that run in a wasm test harness.

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use fop_types::FopError;

    // ------------------------------------------------------------------
    // FopError Display formatting
    // ------------------------------------------------------------------

    #[test]
    fn test_display_parse_error_contains_message() {
        let err = FopError::ParseError("bad XML".to_string());
        let s = err.to_string();
        assert!(
            s.contains("bad XML"),
            "ParseError display must contain message, got: {}",
            s
        );
    }

    #[test]
    fn test_display_parse_error_prefix() {
        let err = FopError::ParseError("something".to_string());
        let s = err.to_string();
        assert!(
            s.contains("Parse error"),
            "ParseError must say 'Parse error', got: {}",
            s
        );
    }

    #[test]
    fn test_display_xml_error_contains_message() {
        let err = FopError::XmlError("malformed tag".to_string());
        let s = err.to_string();
        assert!(
            s.contains("malformed tag"),
            "XmlError display must contain message, got: {}",
            s
        );
    }

    #[test]
    fn test_display_xml_error_prefix() {
        let err = FopError::XmlError("x".to_string());
        let s = err.to_string();
        assert!(
            s.contains("XML parsing error"),
            "Must mention XML, got: {}",
            s
        );
    }

    #[test]
    fn test_display_generic_error_contains_message() {
        let err = FopError::Generic("generic failure".to_string());
        let s = err.to_string();
        assert!(
            s.contains("generic failure"),
            "Generic display must contain message, got: {}",
            s
        );
    }

    #[test]
    fn test_display_generic_error_empty_message() {
        let err = FopError::Generic(String::new());
        let s = err.to_string();
        // Display of Generic("") should be an empty string or whitespace
        assert!(
            s.trim().is_empty() || s.is_empty(),
            "Generic with empty msg should yield empty display, got: {:?}",
            s
        );
    }

    #[test]
    fn test_display_unknown_property_contains_name() {
        let err = FopError::UnknownProperty("color-bleh".to_string());
        let s = err.to_string();
        assert!(
            s.contains("color-bleh"),
            "UnknownProperty display must contain name, got: {}",
            s
        );
    }

    #[test]
    fn test_display_unknown_property_prefix() {
        let err = FopError::UnknownProperty("x".to_string());
        let s = err.to_string();
        assert!(
            s.contains("Unknown property"),
            "Must say 'Unknown property', got: {}",
            s
        );
    }

    #[test]
    fn test_display_invalid_element_contains_name() {
        let err = FopError::InvalidElement("fo:bogus".to_string());
        let s = err.to_string();
        assert!(
            s.contains("fo:bogus"),
            "InvalidElement display must contain name, got: {}",
            s
        );
    }

    #[test]
    fn test_display_invalid_property_value_contains_property_and_value() {
        let err = FopError::InvalidPropertyValue {
            property: "font-size".to_string(),
            value: "huge".to_string(),
        };
        let s = err.to_string();
        assert!(
            s.contains("font-size"),
            "Must contain property name, got: {}",
            s
        );
        assert!(s.contains("huge"), "Must contain value, got: {}", s);
    }

    #[test]
    fn test_display_missing_attribute_contains_element_and_attribute() {
        let err = FopError::MissingAttribute {
            element: "fo:block".to_string(),
            attribute: "font-size".to_string(),
        };
        let s = err.to_string();
        assert!(
            s.contains("fo:block"),
            "Must contain element name, got: {}",
            s
        );
        assert!(
            s.contains("font-size"),
            "Must contain attribute name, got: {}",
            s
        );
    }

    #[test]
    fn test_display_invalid_nesting_contains_parent_and_child() {
        let err = FopError::InvalidNesting {
            parent: "fo:inline".to_string(),
            child: "fo:block".to_string(),
        };
        let s = err.to_string();
        assert!(
            s.contains("fo:inline"),
            "Must contain parent name, got: {}",
            s
        );
        assert!(
            s.contains("fo:block"),
            "Must contain child name, got: {}",
            s
        );
    }

    #[test]
    fn test_display_io_error_contains_message() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = FopError::IoError(io_err);
        let s = err.to_string();
        assert!(!s.is_empty(), "IoError display must not be empty");
        assert!(
            s.contains("file not found"),
            "Must contain io message, got: {}",
            s
        );
    }

    #[test]
    fn test_display_io_error_prefix() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        let err = FopError::IoError(io_err);
        let s = err.to_string();
        assert!(s.contains("I/O error"), "Must say 'I/O error', got: {}", s);
    }

    #[test]
    fn test_display_xml_error_with_location() {
        use fop_types::Location;
        let err = FopError::XmlErrorWithLocation {
            message: "unexpected token".to_string(),
            location: Location::new(5, 12),
            suggestion: Some("check closing tag".to_string()),
        };
        let s = err.to_string();
        assert!(
            s.contains("unexpected token"),
            "Must contain message, got: {}",
            s
        );
        assert!(s.contains('5'), "Must contain line number 5, got: {}", s);
    }

    #[test]
    fn test_display_property_validation_contains_all_fields() {
        let err = FopError::PropertyValidation {
            property: "margin-top".to_string(),
            value: "-1pt".to_string(),
            reason: "must be non-negative".to_string(),
        };
        let s = err.to_string();
        assert!(
            s.contains("margin-top"),
            "Must contain property, got: {}",
            s
        );
        assert!(s.contains("-1pt"), "Must contain value, got: {}", s);
        assert!(
            s.contains("non-negative"),
            "Must contain reason, got: {}",
            s
        );
    }

    #[test]
    fn test_display_entity_error_contains_message() {
        use fop_types::Location;
        let err = FopError::EntityError {
            message: "unresolved entity".to_string(),
            location: Location::new(3, 7),
        };
        let s = err.to_string();
        assert!(
            s.contains("unresolved entity"),
            "Must contain message, got: {}",
            s
        );
    }

    #[test]
    fn test_display_preserves_special_characters() {
        let msg = r#"error with <>&"' special chars"#;
        let err = FopError::Generic(msg.to_string());
        let s = err.to_string();
        assert!(
            s.contains(msg),
            "Full message with special chars must be preserved, got: {}",
            s
        );
    }

    // ------------------------------------------------------------------
    // From<std::io::Error> conversion
    // ------------------------------------------------------------------

    #[test]
    fn test_from_io_error_not_found() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let fop_err: FopError = io_err.into();
        match fop_err {
            FopError::IoError(_) => {} // correct variant
            other => panic!("Expected IoError, got {:?}", other),
        }
    }

    #[test]
    fn test_from_io_error_permission_denied() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let fop_err: FopError = io_err.into();
        assert!(
            matches!(fop_err, FopError::IoError(_)),
            "Must map to IoError"
        );
    }

    // ------------------------------------------------------------------
    // Debug output
    // ------------------------------------------------------------------

    #[test]
    fn test_debug_output_contains_variant_name() {
        let err = FopError::ParseError("dbg test".to_string());
        let dbg = format!("{:?}", err);
        assert!(
            dbg.contains("ParseError"),
            "Debug must include variant name, got: {}",
            dbg
        );
    }

    #[test]
    fn test_debug_output_contains_message() {
        let err = FopError::Generic("debug message".to_string());
        let dbg = format!("{:?}", err);
        assert!(
            dbg.contains("debug message"),
            "Debug must include message, got: {}",
            dbg
        );
    }

    // ------------------------------------------------------------------
    // fop_error_to_js / error_to_js: verify that the Display string fed to
    // JsValue matches expected content (without calling from_str at runtime).
    // We simulate what those functions do: `err.to_string()`.
    // ------------------------------------------------------------------

    #[test]
    fn test_js_string_for_parse_error_matches_display() {
        // fop_error_to_js(err) == JsValue::from_str(&err.to_string())
        // We verify the string that *would* be passed to JsValue::from_str.
        let err = FopError::ParseError("round-trip".to_string());
        let would_be_js = err.to_string();
        assert!(
            would_be_js.contains("round-trip"),
            "String fed to JsValue must contain message"
        );
    }

    #[test]
    fn test_js_string_for_generic_error_is_the_message_itself() {
        // Generic(msg).to_string() == msg (the thiserror format is just "{0}")
        let msg = "the exact message";
        let err = FopError::Generic(msg.to_string());
        let would_be_js = err.to_string();
        assert_eq!(
            would_be_js, msg,
            "Generic error Display must equal the raw message"
        );
    }

    #[test]
    fn test_error_to_js_string_for_io_error() {
        // Simulate what error_to_js does for an io::Error
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
        let would_be_js = format!("{}", io_err);
        assert!(
            would_be_js.contains("no such file"),
            "IoError Display must contain message"
        );
    }
}
