//! PDF validation and quality checks
//!
//! Validates generated PDFs for correctness and quality.

use std::collections::HashSet;

/// PDF validator for checking structural integrity and quality
pub struct PdfValidator {
    /// Strict mode (fail on warnings)
    strict: bool,
}

/// Validation result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    /// PDF is valid with no issues
    Valid,
    /// PDF is valid but has warnings
    Warning(Vec<String>),
    /// PDF is invalid with errors
    Error(Vec<String>),
}

impl ValidationResult {
    /// Check if validation passed (valid or warning only)
    pub fn is_ok(&self) -> bool {
        matches!(self, ValidationResult::Valid | ValidationResult::Warning(_))
    }

    /// Check if there are errors
    pub fn has_errors(&self) -> bool {
        matches!(self, ValidationResult::Error(_))
    }

    /// Get all issues (warnings or errors)
    pub fn issues(&self) -> Vec<String> {
        match self {
            ValidationResult::Valid => Vec::new(),
            ValidationResult::Warning(warnings) => warnings.clone(),
            ValidationResult::Error(errors) => errors.clone(),
        }
    }
}

impl PdfValidator {
    /// Create a new PDF validator
    pub fn new() -> Self {
        Self { strict: false }
    }

    /// Create a new strict PDF validator (warnings are treated as errors)
    pub fn new_strict() -> Self {
        Self { strict: true }
    }

    /// Validate a PDF document
    ///
    /// # Arguments
    /// * `pdf_bytes` - Raw PDF file bytes
    ///
    /// # Returns
    /// ValidationResult indicating success, warnings, or errors
    pub fn validate_pdf(&self, pdf_bytes: &[u8]) -> ValidationResult {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Check minimum size
        if pdf_bytes.len() < 20 {
            errors.push("PDF file is too small (minimum 20 bytes)".to_string());
            return ValidationResult::Error(errors);
        }

        // Check PDF header
        if let Err(e) = self.check_header(pdf_bytes) {
            errors.push(e);
        }

        // Check EOF marker
        if let Err(e) = self.check_eof(pdf_bytes) {
            errors.push(e);
        }

        // Find xref offset
        let xref_offset = match self.find_xref_offset(pdf_bytes) {
            Ok(offset) => offset,
            Err(e) => {
                errors.push(e);
                return ValidationResult::Error(errors);
            }
        };

        // Validate cross-reference table
        match self.validate_xref(pdf_bytes, xref_offset) {
            Ok(warns) => warnings.extend(warns),
            Err(e) => errors.push(e),
        }

        // Validate trailer
        match self.validate_trailer(pdf_bytes, xref_offset) {
            Ok(warns) => warnings.extend(warns),
            Err(e) => errors.push(e),
        }

        // Validate object structure
        match self.validate_objects(pdf_bytes) {
            Ok(warns) => warnings.extend(warns),
            Err(e) => errors.push(e),
        }

        // Check catalog
        match self.check_catalog(pdf_bytes) {
            Ok(warns) => warnings.extend(warns),
            Err(e) => errors.push(e),
        }

        // Check pages
        match self.check_pages(pdf_bytes) {
            Ok(warns) => warnings.extend(warns),
            Err(e) => errors.push(e),
        }

        // Check file size reasonableness
        if let Some(warning) = self.check_size(pdf_bytes) {
            warnings.push(warning);
        }

        // Return result
        if !errors.is_empty() {
            ValidationResult::Error(errors)
        } else if !warnings.is_empty() {
            if self.strict {
                ValidationResult::Error(warnings)
            } else {
                ValidationResult::Warning(warnings)
            }
        } else {
            ValidationResult::Valid
        }
    }

    /// Check PDF header for correct version
    fn check_header(&self, pdf_bytes: &[u8]) -> Result<(), String> {
        if !pdf_bytes.starts_with(b"%PDF-") {
            return Err("PDF header missing or invalid (expected %PDF-)".to_string());
        }

        // Extract version
        if pdf_bytes.len() < 8 {
            return Err("PDF header truncated".to_string());
        }

        let header_line = match find_line_end(pdf_bytes, 0) {
            Some(end) => &pdf_bytes[0..end],
            None => return Err("PDF header line incomplete".to_string()),
        };

        let header_str = String::from_utf8_lossy(header_line);

        // Check for valid version numbers
        if !header_str.starts_with("%PDF-1.") && !header_str.starts_with("%PDF-2.") {
            return Err(format!("Unsupported PDF version: {}", header_str));
        }

        Ok(())
    }

    /// Check EOF marker
    fn check_eof(&self, pdf_bytes: &[u8]) -> Result<(), String> {
        // Find %%EOF from the end (allowing trailing whitespace)
        let trimmed = trim_end_whitespace(pdf_bytes);

        if !trimmed.ends_with(b"%%EOF") {
            return Err("PDF file missing %%EOF marker".to_string());
        }

        Ok(())
    }

    /// Find startxref offset value
    fn find_xref_offset(&self, pdf_bytes: &[u8]) -> Result<usize, String> {
        // Search for "startxref" from the end
        let content = String::from_utf8_lossy(pdf_bytes);

        if let Some(pos) = content.rfind("startxref") {
            // Read the number after startxref
            let after_keyword = &content[pos + 9..];

            // Find the first line with a number
            for line in after_keyword.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    if let Ok(offset) = trimmed.parse::<usize>() {
                        return Ok(offset);
                    }
                }
            }

            return Err("startxref value not found or invalid".to_string());
        }

        Err("startxref keyword not found".to_string())
    }

    /// Validate cross-reference table
    fn validate_xref(&self, pdf_bytes: &[u8], xref_offset: usize) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();

        if xref_offset >= pdf_bytes.len() {
            return Err("xref offset points beyond file end".to_string());
        }

        let xref_section = &pdf_bytes[xref_offset..];
        let xref_str = String::from_utf8_lossy(xref_section);

        // Check for "xref" keyword (with optional leading whitespace/newline)
        let trimmed_xref = xref_str.trim_start();
        if !trimmed_xref.starts_with("xref") {
            return Err(format!(
                "xref table missing 'xref' keyword (found: {:?})",
                &xref_str.chars().take(20).collect::<String>()
            ));
        }

        // Parse xref header (object number and count)
        let mut lines = xref_str.lines();
        let _ = lines.next(); // Skip "xref"

        if let Some(header_line) = lines.next() {
            let parts: Vec<&str> = header_line.split_whitespace().collect();
            if parts.len() != 2 {
                return Err("xref subsection header invalid".to_string());
            }

            // Parse object count
            let count = parts[1]
                .parse::<usize>()
                .map_err(|_| "xref object count invalid".to_string())?;

            // Validate xref entries (each should be 20 bytes: "nnnnnnnnnn ggggg x \n")
            let mut entry_count = 0;
            for line in lines {
                if line.trim().starts_with("trailer") {
                    break;
                }

                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                // Each entry should be: offset(10) space gen(5) space flag(1)
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() != 3 {
                    warnings.push(format!("xref entry malformed: {}", trimmed));
                    continue;
                }

                // Validate offset (10 digits)
                if parts[0].len() != 10 {
                    warnings.push(format!("xref offset not 10 digits: {}", parts[0]));
                }

                // Validate generation (5 digits)
                if parts[1].len() != 5 {
                    warnings.push(format!("xref generation not 5 digits: {}", parts[1]));
                }

                // Validate flag (n or f)
                if parts[2] != "n" && parts[2] != "f" {
                    warnings.push(format!(
                        "xref flag invalid (expected 'n' or 'f'): {}",
                        parts[2]
                    ));
                }

                entry_count += 1;
            }

            // Check if entry count matches declared count
            if entry_count != count {
                warnings.push(format!(
                    "xref entry count mismatch (declared: {}, found: {})",
                    count, entry_count
                ));
            }
        } else {
            return Err("xref subsection header missing".to_string());
        }

        Ok(warnings)
    }

    /// Validate trailer dictionary
    fn validate_trailer(
        &self,
        pdf_bytes: &[u8],
        xref_offset: usize,
    ) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();

        let xref_section = &pdf_bytes[xref_offset..];
        let xref_str = String::from_utf8_lossy(xref_section);

        // Find trailer keyword
        if let Some(trailer_pos) = xref_str.find("trailer") {
            let trailer_section = &xref_str[trailer_pos..];

            // Check for required entries
            if !trailer_section.contains("/Size") {
                return Err("trailer missing required /Size entry".to_string());
            }

            if !trailer_section.contains("/Root") {
                return Err("trailer missing required /Root entry".to_string());
            }

            // Optional but recommended
            if !trailer_section.contains("/Info") {
                warnings.push("trailer missing /Info dictionary (metadata)".to_string());
            }

            // Check for valid dictionary format
            if !trailer_section.contains("<<") || !trailer_section.contains(">>") {
                return Err("trailer dictionary malformed".to_string());
            }
        } else {
            return Err("trailer keyword not found".to_string());
        }

        Ok(warnings)
    }

    /// Validate object structure
    fn validate_objects(&self, pdf_bytes: &[u8]) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();
        let content = String::from_utf8_lossy(pdf_bytes);

        // Find all obj...endobj pairs
        let obj_starts: Vec<_> = content.match_indices(" obj\n").collect();
        let obj_ends: Vec<_> = content.match_indices("\nendobj\n").collect();

        if obj_starts.len() != obj_ends.len() {
            return Err(format!(
                "Mismatched obj/endobj pairs (obj: {}, endobj: {})",
                obj_starts.len(),
                obj_ends.len()
            ));
        }

        // Validate each object has proper ID format
        for (pos, _) in &obj_starts {
            // Look backward for object ID (format: "n g obj")
            let before = &content[..=*pos];
            if let Some(line_start) = before.rfind('\n') {
                let obj_line = &before[line_start + 1..=*pos];
                let parts: Vec<&str> = obj_line.split_whitespace().collect();

                if parts.len() < 3 {
                    warnings.push(format!("Object header malformed near offset {}", pos));
                    continue;
                }

                // Validate object number
                if parts[0].parse::<u32>().is_err() {
                    warnings.push(format!("Invalid object number: {}", parts[0]));
                }

                // Validate generation number
                if parts[1].parse::<u16>().is_err() {
                    warnings.push(format!("Invalid generation number: {}", parts[1]));
                }
            }
        }

        // Check for stream objects
        let stream_count = content.matches("\nstream\n").count();
        let endstream_count = content.matches("\nendstream\n").count();

        if stream_count != endstream_count {
            warnings.push(format!(
                "Mismatched stream/endstream pairs (stream: {}, endstream: {})",
                stream_count, endstream_count
            ));
        }

        Ok(warnings)
    }

    /// Validate catalog dictionary
    fn check_catalog(&self, pdf_bytes: &[u8]) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();
        let content = String::from_utf8_lossy(pdf_bytes);

        // Find catalog object (should be referenced in trailer as /Root)
        if !content.contains("/Type /Catalog") {
            return Err("Catalog object (/Type /Catalog) not found".to_string());
        }

        // Check for required entries in catalog
        if !content.contains("/Pages") {
            return Err("Catalog missing required /Pages entry".to_string());
        }

        // Optional but useful entries
        if !content.contains("/Outlines") {
            warnings.push("Catalog missing /Outlines (bookmarks not present)".to_string());
        }

        Ok(warnings)
    }

    /// Validate page tree structure
    fn check_pages(&self, pdf_bytes: &[u8]) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();
        let content = String::from_utf8_lossy(pdf_bytes);

        // Find pages object
        if !content.contains("/Type /Pages") {
            return Err("Pages object (/Type /Pages) not found".to_string());
        }

        // Check for required entries
        if !content.contains("/Kids") {
            return Err("Pages object missing required /Kids array".to_string());
        }

        if !content.contains("/Count") {
            return Err("Pages object missing required /Count entry".to_string());
        }

        // Find individual page objects
        let page_count = content.matches("/Type /Page\n").count();

        if page_count == 0 {
            warnings.push("No page objects found in document".to_string());
        }

        // Validate page objects have required entries
        let page_positions: Vec<_> = content.match_indices("/Type /Page\n").collect();

        for (pos, _) in page_positions {
            // Find the object containing this page
            let before = &content[..pos];
            if let Some(obj_start) = before.rfind(" obj\n") {
                let after = &content[pos..];
                if let Some(obj_end) = after.find("\nendobj\n") {
                    let page_obj = &content[obj_start..pos + obj_end];

                    // Check required page entries
                    if !page_obj.contains("/Parent") {
                        warnings.push("Page object missing /Parent reference".to_string());
                    }

                    if !page_obj.contains("/MediaBox") && !page_obj.contains("/CropBox") {
                        warnings.push("Page object missing /MediaBox or /CropBox".to_string());
                    }

                    if !page_obj.contains("/Resources") {
                        warnings.push("Page object missing /Resources dictionary".to_string());
                    }
                }
            }
        }

        Ok(warnings)
    }

    /// Check if resources are properly referenced
    #[allow(dead_code)]
    fn check_resources(&self, pdf_bytes: &[u8]) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();
        let content = String::from_utf8_lossy(pdf_bytes);

        // Check for font resources
        let font_refs: Vec<_> = content.match_indices("/Font").collect();
        let type1_fonts = content.matches("/Type1").count();
        let truetype_fonts = content.matches("/TrueType").count();

        if font_refs.is_empty() {
            warnings.push("No font resources defined".to_string());
        } else if type1_fonts == 0 && truetype_fonts == 0 {
            warnings.push("Font resources defined but no font types found".to_string());
        }

        // Check for XObject resources (images, etc.)
        if content.contains("/XObject") {
            // Validate XObject dictionary exists
            if !content.contains("/Type /XObject") {
                warnings.push("XObject referenced but no XObject definitions found".to_string());
            }
        }

        Ok(warnings)
    }

    /// Validate stream dictionaries
    #[allow(dead_code)]
    fn validate_stream(&self, pdf_bytes: &[u8]) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();
        let content = String::from_utf8_lossy(pdf_bytes);

        // Find all stream objects
        let stream_positions: Vec<_> = content.match_indices("\nstream\n").collect();

        for (pos, _) in stream_positions {
            // Look backward for the stream dictionary
            let before = &content[..pos];

            // Should have a /Length entry
            if let Some(dict_start) = before.rfind("<<") {
                let dict_section = &before[dict_start..];

                if !dict_section.contains("/Length") {
                    warnings.push("Stream dictionary missing /Length entry".to_string());
                }
            } else {
                warnings.push("Stream missing dictionary".to_string());
            }
        }

        Ok(warnings)
    }

    /// Validate object references
    #[allow(dead_code)]
    fn validate_object(&self, pdf_bytes: &[u8]) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();
        let content = String::from_utf8_lossy(pdf_bytes);

        // Collect all defined object IDs
        let mut defined_objects = HashSet::new();

        for line in content.lines() {
            if line.trim().ends_with(" obj") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let Ok(obj_num) = parts[0].parse::<u32>() {
                        defined_objects.insert(obj_num);
                    }
                }
            }
        }

        // Find all object references (format: "n 0 R")
        // Simple pattern matching without regex
        for line in content.lines() {
            for word_group in line.split_whitespace().collect::<Vec<_>>().windows(3) {
                if word_group.len() == 3 && word_group[1] == "0" && word_group[2] == "R" {
                    if let Ok(obj_num) = word_group[0].parse::<u32>() {
                        if !defined_objects.contains(&obj_num) && obj_num > 0 {
                            warnings
                                .push(format!("Reference to undefined object: {} 0 R", obj_num));
                        }
                    }
                }
            }
        }

        Ok(warnings)
    }

    /// Check file size is reasonable
    fn check_size(&self, pdf_bytes: &[u8]) -> Option<String> {
        const MAX_REASONABLE_SIZE: usize = 100 * 1024 * 1024; // 100 MB
        const MIN_REASONABLE_SIZE: usize = 100; // 100 bytes

        let size = pdf_bytes.len();

        if size < MIN_REASONABLE_SIZE {
            Some(format!(
                "PDF file is very small ({} bytes), may be incomplete",
                size
            ))
        } else if size > MAX_REASONABLE_SIZE {
            Some(format!(
                "PDF file is very large ({} MB), consider optimization",
                size / (1024 * 1024)
            ))
        } else {
            None
        }
    }
}

impl Default for PdfValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Find the end of a line (LF or CRLF)
fn find_line_end(data: &[u8], start: usize) -> Option<usize> {
    for (i, &byte) in data.iter().enumerate().skip(start) {
        if byte == b'\n' {
            return Some(i);
        }
    }
    None
}

/// Trim trailing whitespace from byte slice
fn trim_end_whitespace(data: &[u8]) -> &[u8] {
    let mut end = data.len();

    while end > 0 && matches!(data[end - 1], b' ' | b'\t' | b'\n' | b'\r') {
        end -= 1;
    }

    &data[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = PdfValidator::new();
        assert!(!validator.strict);

        let strict_validator = PdfValidator::new_strict();
        assert!(strict_validator.strict);
    }

    #[test]
    fn test_validation_result() {
        let valid = ValidationResult::Valid;
        assert!(valid.is_ok());
        assert!(!valid.has_errors());

        let warning = ValidationResult::Warning(vec!["test warning".to_string()]);
        assert!(warning.is_ok());
        assert!(!warning.has_errors());

        let error = ValidationResult::Error(vec!["test error".to_string()]);
        assert!(!error.is_ok());
        assert!(error.has_errors());
    }

    #[test]
    fn test_empty_pdf() {
        let validator = PdfValidator::new();
        let result = validator.validate_pdf(b"");
        assert!(result.has_errors());
    }

    #[test]
    fn test_minimal_pdf() {
        let validator = PdfValidator::new();

        // Too small
        let result = validator.validate_pdf(b"%PDF-1.4");
        assert!(result.has_errors());
    }

    #[test]
    fn test_invalid_header() {
        let validator = PdfValidator::new();
        let invalid_pdf = b"INVALID HEADER\n%%EOF\n";
        let result = validator.validate_pdf(invalid_pdf);
        assert!(result.has_errors());
    }

    #[test]
    fn test_missing_eof() {
        let validator = PdfValidator::new();
        let pdf = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog >>\nendobj\n";
        let result = validator.validate_pdf(pdf);
        assert!(result.has_errors());
    }

    #[test]
    fn test_basic_valid_pdf() {
        let validator = PdfValidator::new();

        // Create a minimal but valid PDF structure
        let pdf = b"%PDF-1.4\n\
            1 0 obj\n\
            << /Type /Catalog /Pages 2 0 R >>\n\
            endobj\n\
            2 0 obj\n\
            << /Type /Pages /Kids [3 0 R] /Count 1 >>\n\
            endobj\n\
            3 0 obj\n\
            << /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>\n\
            endobj\n\
            4 0 obj\n\
            << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\n\
            endobj\n\
            5 0 obj\n\
            << /Length 44 >>\n\
            stream\n\
            BT\n/F1 12 Tf\n100 700 Td\n(Hello World) Tj\nET\n\
            endstream\n\
            endobj\n\
            xref\n\
            0 6\n\
            0000000000 65535 f \n\
            0000000009 00000 n \n\
            0000000058 00000 n \n\
            0000000115 00000 n \n\
            0000000261 00000 n \n\
            0000000339 00000 n \n\
            trailer\n\
            << /Size 6 /Root 1 0 R >>\n\
            startxref\n\
            404\n\
            %%EOF\n";

        let result = validator.validate_pdf(pdf);

        // Debug print issues if validation fails
        if !result.is_ok() {
            eprintln!("Validation result: {:?}", result);
            for issue in result.issues() {
                eprintln!("  - {}", issue);
            }
        }

        assert!(result.is_ok());
    }

    #[test]
    fn test_find_line_end() {
        let data = b"Hello\nWorld";
        assert_eq!(find_line_end(data, 0), Some(5));

        let no_newline = b"Hello";
        assert_eq!(find_line_end(no_newline, 0), None);
    }

    #[test]
    fn test_trim_end_whitespace() {
        let data = b"Hello  \n\r\t";
        let trimmed = trim_end_whitespace(data);
        assert_eq!(trimmed, b"Hello");

        let no_whitespace = b"Hello";
        let trimmed2 = trim_end_whitespace(no_whitespace);
        assert_eq!(trimmed2, b"Hello");
    }

    #[test]
    fn test_check_header() {
        let validator = PdfValidator::new();

        // Valid headers
        assert!(validator.check_header(b"%PDF-1.4\n").is_ok());
        assert!(validator.check_header(b"%PDF-1.7\n").is_ok());
        assert!(validator.check_header(b"%PDF-2.0\n").is_ok());

        // Invalid headers
        assert!(validator.check_header(b"PDF-1.4\n").is_err());
        assert!(validator.check_header(b"%PDF-\n").is_err());
    }

    #[test]
    fn test_check_size() {
        let validator = PdfValidator::new();

        // Too small
        let small = vec![0u8; 50];
        assert!(validator.check_size(&small).is_some());

        // Normal size
        let normal = vec![0u8; 1024];
        assert!(validator.check_size(&normal).is_none());

        // Very large (would test but don't want to allocate 100MB in test)
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    // ── ValidationResult API ─────────────────────────────────────────────────

    #[test]
    fn test_valid_is_ok() {
        assert!(ValidationResult::Valid.is_ok());
    }

    #[test]
    fn test_valid_has_no_errors() {
        assert!(!ValidationResult::Valid.has_errors());
    }

    #[test]
    fn test_valid_issues_empty() {
        assert!(ValidationResult::Valid.issues().is_empty());
    }

    #[test]
    fn test_warning_is_ok() {
        let w = ValidationResult::Warning(vec!["w1".to_string()]);
        assert!(w.is_ok());
    }

    #[test]
    fn test_warning_has_no_errors() {
        let w = ValidationResult::Warning(vec!["w1".to_string()]);
        assert!(!w.has_errors());
    }

    #[test]
    fn test_warning_issues_returns_messages() {
        let w = ValidationResult::Warning(vec!["a".to_string(), "b".to_string()]);
        let issues = w.issues();
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0], "a");
        assert_eq!(issues[1], "b");
    }

    #[test]
    fn test_error_not_ok() {
        let e = ValidationResult::Error(vec!["bad".to_string()]);
        assert!(!e.is_ok());
    }

    #[test]
    fn test_error_has_errors() {
        let e = ValidationResult::Error(vec!["bad".to_string()]);
        assert!(e.has_errors());
    }

    #[test]
    fn test_error_issues_returns_messages() {
        let e = ValidationResult::Error(vec!["e1".to_string(), "e2".to_string()]);
        let issues = e.issues();
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0], "e1");
    }

    // ── PdfValidator creation ────────────────────────────────────────────────

    #[test]
    fn test_default_is_non_strict() {
        let v = PdfValidator::default();
        assert!(!v.strict);
    }

    #[test]
    fn test_new_strict_is_strict() {
        let v = PdfValidator::new_strict();
        assert!(v.strict);
    }

    // ── Header validation ────────────────────────────────────────────────────

    #[test]
    fn test_pdf_1_0_header_unsupported() {
        // %PDF-1.0 is technically valid but our check passes it since it
        // starts with "%PDF-1."
        let v = PdfValidator::new();
        assert!(v.check_header(b"%PDF-1.0\n").is_ok());
    }

    #[test]
    fn test_pdf_1_7_header_valid() {
        let v = PdfValidator::new();
        assert!(v.check_header(b"%PDF-1.7\n").is_ok());
    }

    #[test]
    fn test_pdf_2_0_header_valid() {
        let v = PdfValidator::new();
        assert!(v.check_header(b"%PDF-2.0\n").is_ok());
    }

    #[test]
    fn test_missing_percent_header_invalid() {
        let v = PdfValidator::new();
        assert!(v.check_header(b"PDF-1.4\n").is_err());
    }

    #[test]
    fn test_garbage_header_invalid() {
        let v = PdfValidator::new();
        assert!(v.check_header(b"garbage\n").is_err());
    }

    // ── EOF validation ───────────────────────────────────────────────────────

    #[test]
    fn test_check_eof_present() {
        let v = PdfValidator::new();
        // check_eof only looks at the bytes passed to it
        let data = b"...content...%%EOF";
        assert!(v.check_eof(data).is_ok());
    }

    #[test]
    fn test_check_eof_missing() {
        let v = PdfValidator::new();
        let data = b"...content...no-eof-marker";
        assert!(v.check_eof(data).is_err());
    }

    #[test]
    fn test_check_eof_with_trailing_whitespace() {
        let v = PdfValidator::new();
        let data = b"%%EOF\n\r\n";
        assert!(v.check_eof(data).is_ok());
    }

    // ── Size validation ──────────────────────────────────────────────────────

    #[test]
    fn test_tiny_pdf_triggers_size_warning() {
        let v = PdfValidator::new();
        let tiny = vec![0u8; 10];
        assert!(v.check_size(&tiny).is_some());
    }

    #[test]
    fn test_reasonable_size_no_warning() {
        let v = PdfValidator::new();
        let normal = vec![0u8; 4096];
        assert!(v.check_size(&normal).is_none());
    }

    // ── Full validation — well-formed minimal PDF ────────────────────────────

    /// Build a minimal well-formed PDF bytes string for validation tests.
    ///
    /// We build each object as a separate `Vec<u8>` so that Rust string
    /// continuation (`\` at end of line) does NOT add whitespace and the
    /// byte offsets we store in the xref table are exact.
    fn minimal_valid_pdf() -> Vec<u8> {
        let obj1: &[u8] = b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n";
        let obj2: &[u8] = b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n";
        let obj3: &[u8] = b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> >>\nendobj\n";
        let obj4: &[u8] =
            b"4 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n";

        let header: &[u8] = b"%PDF-1.4\n";
        let o1_off = header.len(); // 9
        let o2_off = o1_off + obj1.len(); // 58
        let o3_off = o2_off + obj2.len(); // 115
        let o4_off = o3_off + obj3.len(); // 225
        let xref_off = o4_off + obj4.len(); // 295

        let xref = format!(
            "xref\n             0 5\n             {o0:010} 65535 f \n             {o1:010} 00000 n \n             {o2:010} 00000 n \n             {o3:010} 00000 n \n             {o4:010} 00000 n \n             trailer\n             << /Size 5 /Root 1 0 R >>\n             startxref\n             {xref_off}\n             %%EOF\n",
            o0 = 0,
            o1 = o1_off,
            o2 = o2_off,
            o3 = o3_off,
            o4 = o4_off,
            xref_off = xref_off,
        );

        let mut pdf = Vec::new();
        pdf.extend_from_slice(header);
        pdf.extend_from_slice(obj1);
        pdf.extend_from_slice(obj2);
        pdf.extend_from_slice(obj3);
        pdf.extend_from_slice(obj4);
        pdf.extend_from_slice(xref.as_bytes());
        pdf
    }

    #[test]
    fn test_valid_pdf_passes_validation() {
        let v = PdfValidator::new();
        let result = v.validate_pdf(&minimal_valid_pdf());
        assert!(result.is_ok(), "validation failed: {:?}", result);
    }

    #[test]
    fn test_invalid_pdf_too_short() {
        let v = PdfValidator::new();
        let result = v.validate_pdf(b"%PDF-1");
        assert!(result.has_errors());
    }

    #[test]
    fn test_invalid_pdf_no_header() {
        let v = PdfValidator::new();
        let result = v.validate_pdf(b"JUNK JUNK JUNK JUNK JUNK\n%%EOF\n");
        assert!(result.has_errors());
    }

    #[test]
    fn test_invalid_pdf_no_eof() {
        let v = PdfValidator::new();
        let result = v.validate_pdf(b"%PDF-1.4\nsome content without eof marker");
        assert!(result.has_errors());
    }

    // ── Strict mode ──────────────────────────────────────────────────────────

    #[test]
    fn test_strict_mode_treats_warnings_as_errors() {
        // A PDF that is structurally OK but triggers a warning (no /Info dict).
        // In strict mode warnings become errors.
        let v_strict = PdfValidator::new_strict();
        let pdf = minimal_valid_pdf(); // The minimal PDF has no /Info → warning
        let result = v_strict.validate_pdf(&pdf);
        // Strict mode: warning(s) → error
        // The minimal PDF will produce at least the /Info warning plus
        // the /Outlines warning → should be Error in strict mode.
        assert!(
            result.has_errors(),
            "strict mode should have errors (warnings promoted): {:?}",
            result
        );
    }

    #[test]
    fn test_non_strict_warnings_not_errors() {
        let v = PdfValidator::new();
        let pdf = minimal_valid_pdf();
        let result = v.validate_pdf(&pdf);
        // Non-strict: warnings are ok, not errors
        assert!(result.is_ok(), "non-strict should be ok: {:?}", result);
    }

    // ── trim_end_whitespace helper ───────────────────────────────────────────

    #[test]
    fn test_trim_end_whitespace_empty() {
        assert_eq!(trim_end_whitespace(b""), b"");
    }

    #[test]
    fn test_trim_end_whitespace_only_spaces() {
        assert_eq!(trim_end_whitespace(b"   "), b"");
    }

    #[test]
    fn test_trim_end_whitespace_preserves_content() {
        assert_eq!(trim_end_whitespace(b"abc\n\r"), b"abc");
    }

    // ── find_line_end helper ─────────────────────────────────────────────────

    #[test]
    fn test_find_line_end_at_start() {
        assert_eq!(find_line_end(b"\nrest", 0), Some(0));
    }

    #[test]
    fn test_find_line_end_mid_string() {
        assert_eq!(find_line_end(b"ab\ncd", 0), Some(2));
    }

    #[test]
    fn test_find_line_end_none_without_newline() {
        assert_eq!(find_line_end(b"abcdef", 0), None);
    }

    #[test]
    fn test_find_line_end_with_offset() {
        // Skip past first newline
        assert_eq!(find_line_end(b"a\nb\nc", 2), Some(3));
    }
}
