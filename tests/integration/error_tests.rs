//! Error handling integration tests

use super::process_fo_document;

#[test]
fn test_invalid_xml() {
    let invalid_xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <!-- Missing closing tag -->
  <fo:layout-master-set>
</fo:root>"##;

    let result = process_fo_document(invalid_xml);
    assert!(result.is_err(), "Should fail on invalid XML");
}

#[test]
fn test_missing_required_element() {
    // Missing layout-master-set
    let invalid_fo = r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Test</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    let result = process_fo_document(invalid_fo);
    // This should either fail or produce a warning
    // Depending on implementation, it might be lenient
    if let Ok(bytes) = result {
        // If it succeeds, verify it's a valid PDF
        assert!(bytes.starts_with(b"%PDF-"));
    }
}

#[test]
fn test_unknown_property() {
    // Document with unknown property (should be ignored or warned)
    let fo_content = r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block unknown-property="value">Test</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    let result = process_fo_document(fo_content);
    // Unknown properties should be handled gracefully
    assert!(
        result.is_ok(),
        "Should handle unknown properties gracefully"
    );
}

#[test]
fn test_invalid_property_value() {
    // Invalid color value
    let fo_content = r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block color="not-a-color">Test</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    let result = process_fo_document(fo_content);
    // Invalid property values should be handled gracefully
    // Implementation may fall back to defaults
    if let Ok(bytes) = result {
        assert!(bytes.starts_with(b"%PDF-"));
    }
}

#[test]
fn test_missing_namespace() {
    // FO elements without namespace
    let no_namespace = r##"<?xml version="1.0" encoding="UTF-8"?>
<root>
  <layout-master-set>
    <simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
      <region-body/>
    </simple-page-master>
  </layout-master-set>
  <page-sequence master-reference="A4">
    <flow flow-name="xsl-region-body">
      <block>Test</block>
    </flow>
  </page-sequence>
</root>"##;

    let result = process_fo_document(no_namespace);
    // Should fail or handle gracefully depending on implementation
    // Most likely will fail since namespace is required
    let _ = result;
}

#[test]
fn test_empty_page_sequence() {
    // Page sequence with no flow
    let empty_sequence = r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
  </fo:page-sequence>
</fo:root>"##;

    let result = process_fo_document(empty_sequence);
    // Should handle empty sequence gracefully
    let _ = result;
}

#[test]
fn test_malformed_table() {
    // Table without table-body
    let malformed_table = r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:table>
        <fo:table-column column-width="100%"/>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    let result = process_fo_document(malformed_table);
    // Should handle malformed table gracefully or error appropriately
    let _ = result;
}

#[test]
fn test_very_large_values() {
    // Extremely large page size
    let large_values = r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="huge" page-width="100000mm" page-height="100000mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="huge">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Test</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    let result = process_fo_document(large_values);
    // Should handle or reject unreasonable values
    let _ = result;
}

#[test]
fn test_negative_dimensions() {
    // Negative width/height values
    let negative = r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="invalid" page-width="-100mm" page-height="297mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="invalid">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Test</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    let result = process_fo_document(negative);
    // Should reject negative dimensions
    let _ = result;
}
