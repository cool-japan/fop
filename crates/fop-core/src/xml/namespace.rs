//! Namespace definitions for XSL-FO

/// XSL-FO namespace URIs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Namespace {
    /// XSL-FO 1.1 namespace: <http://www.w3.org/1999/XSL/Format>
    Fo,
    /// FOP extensions namespace
    FoxExtensions,
    /// SVG namespace
    Svg,
    /// Other/unknown namespace
    Other,
}

impl Namespace {
    /// XSL-FO 1.1 namespace URI
    pub const FO_URI: &'static str = "http://www.w3.org/1999/XSL/Format";

    /// FOP extensions namespace URI
    pub const FOX_URI: &'static str = "http://xmlgraphics.apache.org/fop/extensions";

    /// SVG namespace URI
    pub const SVG_URI: &'static str = "http://www.w3.org/2000/svg";

    /// Parse a namespace URI
    pub fn from_uri(uri: &str) -> Self {
        match uri {
            Self::FO_URI => Namespace::Fo,
            Self::FOX_URI => Namespace::FoxExtensions,
            Self::SVG_URI => Namespace::Svg,
            _ => Namespace::Other,
        }
    }

    /// Get the namespace URI
    pub fn uri(self) -> &'static str {
        match self {
            Namespace::Fo => Self::FO_URI,
            Namespace::FoxExtensions => Self::FOX_URI,
            Namespace::Svg => Self::SVG_URI,
            Namespace::Other => "",
        }
    }

    /// Check if this is the FO namespace
    #[inline]
    pub fn is_fo(self) -> bool {
        matches!(self, Namespace::Fo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_parsing() {
        assert_eq!(
            Namespace::from_uri("http://www.w3.org/1999/XSL/Format"),
            Namespace::Fo
        );
        assert_eq!(
            Namespace::from_uri("http://xmlgraphics.apache.org/fop/extensions"),
            Namespace::FoxExtensions
        );
        assert_eq!(
            Namespace::from_uri("http://www.w3.org/2000/svg"),
            Namespace::Svg
        );
        assert_eq!(Namespace::from_uri("http://example.com"), Namespace::Other);
    }

    #[test]
    fn test_namespace_uri() {
        assert_eq!(Namespace::Fo.uri(), "http://www.w3.org/1999/XSL/Format");
        assert!(Namespace::Fo.is_fo());
        assert!(!Namespace::Svg.is_fo());
    }
}

// ===== ADDITIONAL TESTS =====
#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn test_namespace_fo_uri_constant() {
        assert_eq!(Namespace::FO_URI, "http://www.w3.org/1999/XSL/Format");
    }

    #[test]
    fn test_namespace_fox_uri_constant() {
        assert_eq!(
            Namespace::FOX_URI,
            "http://xmlgraphics.apache.org/fop/extensions"
        );
    }

    #[test]
    fn test_namespace_svg_uri_constant() {
        assert_eq!(Namespace::SVG_URI, "http://www.w3.org/2000/svg");
    }

    #[test]
    fn test_namespace_fox_is_not_fo() {
        assert!(!Namespace::FoxExtensions.is_fo());
    }

    #[test]
    fn test_namespace_other_is_not_fo() {
        assert!(!Namespace::Other.is_fo());
    }

    #[test]
    fn test_namespace_svg_is_not_fo() {
        assert!(!Namespace::Svg.is_fo());
    }

    #[test]
    fn test_namespace_fo_is_fo() {
        assert!(Namespace::Fo.is_fo());
    }

    #[test]
    fn test_namespace_from_empty_uri() {
        assert_eq!(Namespace::from_uri(""), Namespace::Other);
    }

    #[test]
    fn test_namespace_from_partial_uri() {
        assert_eq!(Namespace::from_uri("http://www.w3.org"), Namespace::Other);
    }

    #[test]
    fn test_namespace_equality() {
        assert_eq!(Namespace::Fo, Namespace::Fo);
        assert_eq!(Namespace::Svg, Namespace::Svg);
        assert_ne!(Namespace::Fo, Namespace::Svg);
        assert_ne!(Namespace::FoxExtensions, Namespace::Other);
    }

    #[test]
    fn test_namespace_svg_uri() {
        assert_eq!(Namespace::Svg.uri(), "http://www.w3.org/2000/svg");
    }

    #[test]
    fn test_namespace_fox_uri() {
        assert_eq!(
            Namespace::FoxExtensions.uri(),
            "http://xmlgraphics.apache.org/fop/extensions"
        );
    }

    #[test]
    fn test_namespace_other_uri_empty() {
        assert_eq!(Namespace::Other.uri(), "");
    }
}
