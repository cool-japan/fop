//! Error mapping from FopError to PyErr for Python

use pyo3::exceptions::PyRuntimeError;
use pyo3::PyErr;

/// Convert a FopError into a Python exception
pub fn fop_error_to_py(err: fop_types::FopError) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}

// ============================================================================
// Unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use fop_types::FopError;
    use pyo3::exceptions::{PyIOError, PyRuntimeError, PyValueError};
    use pyo3::prelude::*;

    // ------------------------------------------------------------------
    // fop_error_to_py — currently maps all variants to PyRuntimeError.
    // Tests verify the behaviour of the current implementation plus the
    // Display strings that the Python exception message will carry.
    // ------------------------------------------------------------------

    #[test]
    fn test_fop_error_to_py_parse_error_is_runtime_error() {
        use super::fop_error_to_py;
        let err = FopError::ParseError("bad XML".to_string());
        let py_err = fop_error_to_py(err);
        Python::attach(|py| {
            assert!(
                py_err.is_instance_of::<PyRuntimeError>(py),
                "ParseError must map to PyRuntimeError"
            );
        });
    }

    #[test]
    fn test_fop_error_to_py_xml_error_is_runtime_error() {
        use super::fop_error_to_py;
        let err = FopError::XmlError("malformed".to_string());
        let py_err = fop_error_to_py(err);
        Python::attach(|py| {
            assert!(py_err.is_instance_of::<PyRuntimeError>(py));
        });
    }

    #[test]
    fn test_fop_error_to_py_generic_error_is_runtime_error() {
        use super::fop_error_to_py;
        let err = FopError::Generic("generic failure".to_string());
        let py_err = fop_error_to_py(err);
        Python::attach(|py| {
            assert!(py_err.is_instance_of::<PyRuntimeError>(py));
        });
    }

    #[test]
    fn test_fop_error_to_py_io_error_is_runtime_error() {
        use super::fop_error_to_py;
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = FopError::IoError(io_err);
        let py_err = fop_error_to_py(err);
        Python::attach(|py| {
            assert!(py_err.is_instance_of::<PyRuntimeError>(py));
        });
    }

    #[test]
    fn test_fop_error_to_py_unknown_property_is_runtime_error() {
        use super::fop_error_to_py;
        let err = FopError::UnknownProperty("color-bleh".to_string());
        let py_err = fop_error_to_py(err);
        Python::attach(|py| {
            assert!(py_err.is_instance_of::<PyRuntimeError>(py));
        });
    }

    #[test]
    fn test_fop_error_to_py_invalid_element_is_runtime_error() {
        use super::fop_error_to_py;
        let err = FopError::InvalidElement("fo:bogus".to_string());
        let py_err = fop_error_to_py(err);
        Python::attach(|py| {
            assert!(py_err.is_instance_of::<PyRuntimeError>(py));
        });
    }

    #[test]
    fn test_fop_error_to_py_invalid_nesting_is_runtime_error() {
        use super::fop_error_to_py;
        let err = FopError::InvalidNesting {
            parent: "fo:inline".to_string(),
            child: "fo:block".to_string(),
        };
        let py_err = fop_error_to_py(err);
        Python::attach(|py| {
            assert!(py_err.is_instance_of::<PyRuntimeError>(py));
        });
    }

    // ------------------------------------------------------------------
    // Verify message preservation through FopError → PyErr
    // ------------------------------------------------------------------

    #[test]
    fn test_fop_error_to_py_preserves_parse_error_message() {
        use super::fop_error_to_py;
        let err = FopError::ParseError("unexpected token".to_string());
        let display_str = err.to_string();
        let err2 = FopError::ParseError("unexpected token".to_string());
        let py_err = fop_error_to_py(err2);
        // The PyErr message is the FopError Display string
        let py_msg = py_err.to_string();
        assert!(
            py_msg.contains("unexpected token"),
            "PyErr message must contain the FopError message, got: {}",
            py_msg
        );
        // Confirm Display string also contains it
        assert!(display_str.contains("unexpected token"));
    }

    #[test]
    fn test_fop_error_to_py_preserves_xml_error_message() {
        use super::fop_error_to_py;
        let err = FopError::XmlError("malformed XML".to_string());
        let expected = err.to_string();
        let err2 = FopError::XmlError("malformed XML".to_string());
        let py_err = fop_error_to_py(err2);
        let py_msg = py_err.to_string();
        assert!(
            py_msg.contains("malformed XML"),
            "PyErr must carry xml error message, got: {}",
            py_msg
        );
        assert!(expected.contains("malformed XML"));
    }

    #[test]
    fn test_fop_error_to_py_preserves_generic_message() {
        use super::fop_error_to_py;
        let err = FopError::Generic("custom generic msg".to_string());
        let py_err = fop_error_to_py(err);
        let py_msg = py_err.to_string();
        assert!(
            py_msg.contains("custom generic msg"),
            "PyErr must carry generic message, got: {}",
            py_msg
        );
    }

    #[test]
    fn test_fop_error_to_py_preserves_io_error_message() {
        use super::fop_error_to_py;
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = FopError::IoError(io_err);
        let py_err = fop_error_to_py(err);
        let py_msg = py_err.to_string();
        assert!(
            py_msg.contains("file not found"),
            "PyErr must carry io error message, got: {}",
            py_msg
        );
    }

    // ------------------------------------------------------------------
    // FopError Display — verify the strings that Python sees
    // ------------------------------------------------------------------

    #[test]
    fn test_fop_error_display_parse_error() {
        let err = FopError::ParseError("display check".to_string());
        let s = err.to_string();
        assert!(
            s.contains("display check"),
            "ParseError Display must contain message, got: {}",
            s
        );
        assert!(
            s.contains("Parse error"),
            "Must say 'Parse error', got: {}",
            s
        );
    }

    #[test]
    fn test_fop_error_display_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let err = FopError::IoError(io_err);
        let s = err.to_string();
        assert!(
            s.contains("not found"),
            "IoError Display must contain message, got: {}",
            s
        );
        assert!(s.contains("I/O error"), "Must say 'I/O error', got: {}", s);
    }

    #[test]
    fn test_fop_error_display_xml_error_with_location() {
        use fop_types::Location;
        let err = FopError::XmlErrorWithLocation {
            message: "close tag missing".to_string(),
            location: Location::new(10, 5),
            suggestion: None,
        };
        let s = err.to_string();
        assert!(
            s.contains("close tag missing"),
            "Must contain message, got: {}",
            s
        );
        assert!(s.contains("10"), "Must contain line number, got: {}", s);
    }

    #[test]
    fn test_fop_error_display_invalid_nesting() {
        let err = FopError::InvalidNesting {
            parent: "fo:block".to_string(),
            child: "fo:table".to_string(),
        };
        let s = err.to_string();
        assert!(
            s.contains("fo:block") && s.contains("fo:table"),
            "Must contain both parent and child names, got: {}",
            s
        );
    }

    // ------------------------------------------------------------------
    // From<io::Error> conversion
    // ------------------------------------------------------------------

    #[test]
    fn test_from_io_error_variant() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let fop_err: FopError = io_err.into();
        assert!(
            matches!(fop_err, FopError::IoError(_)),
            "Must be IoError variant"
        );
    }

    // ------------------------------------------------------------------
    // Direct PyRuntimeError creation (independent of fop_error_to_py)
    // ------------------------------------------------------------------

    #[test]
    fn test_py_runtime_error_direct_creation() {
        let err = PyRuntimeError::new_err("runtime failure");
        Python::attach(|py| {
            assert!(err.is_instance_of::<PyRuntimeError>(py));
        });
    }

    #[test]
    fn test_py_io_error_direct_creation() {
        let err = PyIOError::new_err("io failure");
        Python::attach(|py| {
            assert!(err.is_instance_of::<PyIOError>(py));
        });
    }

    #[test]
    fn test_py_value_error_direct_creation() {
        let err = PyValueError::new_err("value failure");
        Python::attach(|py| {
            assert!(err.is_instance_of::<PyValueError>(py));
        });
    }
}
