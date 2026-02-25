//! PDF compliance modes (PDF/A-1b and PDF/UA-1)
//!
//! Implements ISO 19005-1 (PDF/A-1b) and ISO 14289-1 (PDF/UA-1) compliance.

/// PDF compliance mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PdfCompliance {
    /// Standard PDF (no special compliance)
    #[default]
    Standard,
    /// PDF/A-1b — archival format (ISO 19005-1)
    PdfA1b,
    /// PDF/UA-1 — accessible format (ISO 14289-1)
    PdfUA1,
    /// Both PDF/A-1b and PDF/UA-1
    PdfA1bUA1,
}

impl PdfCompliance {
    /// Whether this mode requires PDF/A-1b compliance
    pub fn requires_pdfa(&self) -> bool {
        matches!(self, PdfCompliance::PdfA1b | PdfCompliance::PdfA1bUA1)
    }

    /// Whether this mode requires PDF/UA-1 compliance
    pub fn requires_pdfua(&self) -> bool {
        matches!(self, PdfCompliance::PdfUA1 | PdfCompliance::PdfA1bUA1)
    }
}

/// Minimal sRGB ICC profile (v2, 476 bytes).
///
/// This is a valid, conformant sRGB ICC profile suitable for use as
/// an OutputIntent in PDF/A-1b documents. It encodes the IEC 61966-2-1
/// sRGB colour space at a minimal size.
///
/// Layout (476 bytes total, declared in header bytes 0-3):
/// - Header:        128 bytes (offsets   0–127)
/// - Tag count:       4 bytes (offsets 128–131) = 9 tags
/// - Tag table:     108 bytes (offsets 132–239) = 9 × 12 bytes
/// - desc data:     100 bytes (offsets 240–339, 0x00F0–0x0153)
/// - cprt data:      42 bytes (offsets 340–381, 0x0154–0x017D)
/// - wtpt data:      20 bytes (offsets 382–401, 0x017E–0x0191)
/// - rXYZ data:      20 bytes (offsets 402–421, 0x0192–0x01A5)
/// - gXYZ data:      20 bytes (offsets 422–441, 0x01A6–0x01B9)
/// - bXYZ data:      20 bytes (offsets 442–461, 0x01BA–0x01CD)
/// - TRC data:       14 bytes (offsets 462–475, 0x01CE–0x01DB) shared by r/g/bTRC
pub const SRGB_ICC_PROFILE: &[u8] = &[
    // ── ICC profile header (128 bytes, offsets 0–127) ───────────────────────
    0x00, 0x00, 0x01, 0xDC, // [0]  profile size = 476 (0x01DC)
    0x00, 0x00, 0x00, 0x00, // [4]  CMM type (none)
    0x02, 0x10, 0x00, 0x00, // [8]  profile version 2.1.0
    0x6D, 0x6E, 0x74, 0x72, // [12] profile class: mntr (display device profile)
    0x52, 0x47, 0x42, 0x20, // [16] colour space: RGB
    0x58, 0x59, 0x5A, 0x20, // [20] PCS: XYZ
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // [24] date/time
    0x61, 0x63, 0x73, 0x70, // [36] file signature: acsp
    0x4D, 0x53, 0x46, 0x54, // [40] platform: MSFT
    0x00, 0x00, 0x00, 0x00, // [44] profile flags
    0x49, 0x45, 0x43, 0x20, // [48] device manufacturer: IEC
    0x73, 0x52, 0x47, 0x42, // [52] device model: sRGB
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // [56] device attributes
    0x00, 0x00, 0x00, 0x02, // [64] rendering intent: perceptual
    // [68] PCS illuminant (D50 in s15Fixed16)
    0x00, 0x00, 0xF6, 0xD6, // X = 0.9642
    0x00, 0x01, 0x00, 0x00, // Y = 1.0000
    0x00, 0x00, 0xD3, 0x2D, // Z = 0.8249
    0x48, 0x50, 0x20, 0x20, // [80] profile creator: HP
    // [84] MD5 profile ID (zeroed — optional for PDF/A)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // [100] reserved (28 bytes)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // ── Tag count (4 bytes, offset 128) ─────────────────────────────────────
    0x00, 0x00, 0x00, 0x09, // 9 tags
    // ── Tag table (9 × 12 = 108 bytes, offsets 132–239) ─────────────────────
    // Each entry: 4-byte signature, 4-byte offset, 4-byte size
    // desc  @ offset 0x00F0 (240), size 0x64 (100)
    0x64, 0x65, 0x73, 0x63, 0x00, 0x00, 0x00, 0xF0, 0x00, 0x00, 0x00, 0x64,
    // cprt  @ offset 0x0154 (340), size 0x2A (42)
    0x63, 0x70, 0x72, 0x74, 0x00, 0x00, 0x01, 0x54, 0x00, 0x00, 0x00, 0x2A,
    // wtpt  @ offset 0x017E (382), size 0x14 (20)
    0x77, 0x74, 0x70, 0x74, 0x00, 0x00, 0x01, 0x7E, 0x00, 0x00, 0x00, 0x14,
    // rXYZ  @ offset 0x0192 (402), size 0x14 (20)
    0x72, 0x58, 0x59, 0x5A, 0x00, 0x00, 0x01, 0x92, 0x00, 0x00, 0x00, 0x14,
    // gXYZ  @ offset 0x01A6 (422), size 0x14 (20)
    0x67, 0x58, 0x59, 0x5A, 0x00, 0x00, 0x01, 0xA6, 0x00, 0x00, 0x00, 0x14,
    // bXYZ  @ offset 0x01BA (442), size 0x14 (20)
    0x62, 0x58, 0x59, 0x5A, 0x00, 0x00, 0x01, 0xBA, 0x00, 0x00, 0x00, 0x14,
    // rTRC  @ offset 0x01CE (462), size 0x0E (14)  ← all three TRC share this
    0x72, 0x54, 0x52, 0x43, 0x00, 0x00, 0x01, 0xCE, 0x00, 0x00, 0x00, 0x0E,
    // gTRC  @ offset 0x01CE (462), size 0x0E (14)
    0x67, 0x54, 0x52, 0x43, 0x00, 0x00, 0x01, 0xCE, 0x00, 0x00, 0x00, 0x0E,
    // bTRC  @ offset 0x01CE (462), size 0x0E (14)
    0x62, 0x54, 0x52, 0x43, 0x00, 0x00, 0x01, 0xCE, 0x00, 0x00, 0x00, 0x0E,
    // ── desc tag data (offset 240 = 0x00F0, size 100 = 0x64) ────────────────
    0x64, 0x65, 0x73, 0x63, // type signature: desc
    0x00, 0x00, 0x00, 0x00, // reserved
    0x00, 0x00, 0x00, 0x13, // ASCII string length = 19
    0x73, 0x52, 0x47, 0x42, 0x20, 0x49, 0x45, 0x43, // "sRGB IEC"
    0x36, 0x31, 0x39, 0x36, 0x36, 0x2D, 0x32, 0x2D, // "61966-2-"
    0x31, 0x00, 0x00, // "1" + 2-byte null terminator
    // padding: type(4)+reserved(4)+len(4)+string17(17)+null2(2) = 31 used → 69 pad bytes needed
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00,
    // ── cprt tag data (offset 340 = 0x0154, size 42 = 0x2A) ─────────────────
    0x74, 0x65, 0x78, 0x74, // type signature: text
    0x00, 0x00, 0x00, 0x00, // reserved
    // "Copyright IEC http://www.iec.ch" + null (32 bytes) + 2 bytes padding = 34 bytes
    0x43, 0x6F, 0x70, 0x79, 0x72, 0x69, 0x67, 0x68, // "Copyrigh"
    0x74, 0x20, 0x49, 0x45, 0x43, 0x20, 0x68, 0x74, // "t IEC ht"
    0x74, 0x70, 0x3A, 0x2F, 0x2F, 0x77, 0x77, 0x77, // "tp://www"
    0x2E, 0x69, 0x65, 0x63, 0x2E, 0x63, 0x68, 0x00, // ".iec.ch\0"
    0x00, 0x00, // 2-byte pad to 42 total
    // ── wtpt tag data (offset 382 = 0x017E, size 20 = 0x14) ─────────────────
    // D50 white point in s15Fixed16: X=0.9642, Y=1.0000, Z=0.8249
    0x58, 0x59, 0x5A, 0x20, // type: XYZ
    0x00, 0x00, 0x00, 0x00, // reserved
    0x00, 0x00, 0xF6, 0xD6, // X = 0.9642
    0x00, 0x01, 0x00, 0x00, // Y = 1.0000
    0x00, 0x00, 0xD3, 0x2D, // Z = 0.8249
    // ── rXYZ tag data (offset 402 = 0x0192, size 20 = 0x14) ─────────────────
    // sRGB red primary
    0x58, 0x59, 0x5A, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x6E, 0xA2, // X = 0.4361
    0x00, 0x00, 0x38, 0xF2, // Y = 0.2225
    0x00, 0x00, 0x03, 0x90, // Z = 0.0139
    // ── gXYZ tag data (offset 422 = 0x01A6, size 20 = 0x14) ─────────────────
    // sRGB green primary
    0x58, 0x59, 0x5A, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x62, 0x99, // X = 0.3851
    0x00, 0x00, 0xB7, 0x85, // Y = 0.7169
    0x00, 0x00, 0x18, 0xDA, // Z = 0.0971
    // ── bXYZ tag data (offset 442 = 0x01BA, size 20 = 0x14) ─────────────────
    // sRGB blue primary
    0x58, 0x59, 0x5A, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x0E, // X = 0.0938
    0x00, 0x00, 0x0B, 0xA3, // Y = 0.0454
    0x00, 0x00, 0xB6, 0xCF, // Z = 0.7142
    // ── TRC tag data (offset 462 = 0x01CE, size 14 = 0x0E) ──────────────────
    // Shared by rTRC, gTRC, bTRC.  Single gamma value ≈ 2.2 (563/256).
    0x63, 0x75, 0x72, 0x76, // type: curv
    0x00, 0x00, 0x00, 0x00, // reserved
    0x00, 0x00, 0x00, 0x01, // count = 1 (single gamma entry)
    0x02,
    0x33, // gamma = 563/256 ≈ 2.20
          // ── END (total 476 bytes = 0x01DC) ──────────────────────────────────────
];

/// Generate XMP metadata stream for PDF/A compliance.
///
/// Returns the complete XMP packet as a UTF-8 string, including the required
/// Adobe XML namespace declarations and optional PDF/A and PDF/UA identifiers.
pub fn generate_xmp_metadata(
    title: Option<&str>,
    creator_tool: &str,
    compliance: PdfCompliance,
) -> String {
    let title_str = title.unwrap_or("Untitled");

    let pdfa_part = if compliance.requires_pdfa() {
        r#"  <rdf:Description rdf:about=""
     xmlns:pdfaid="http://www.aiim.org/pdfa/ns/id/">
   <pdfaid:part>1</pdfaid:part>
   <pdfaid:conformance>B</pdfaid:conformance>
  </rdf:Description>
"#
    } else {
        ""
    };

    let pdfua_part = if compliance.requires_pdfua() {
        r#"  <rdf:Description rdf:about=""
     xmlns:pdfuaid="http://www.aiim.org/pdfua/ns/id/">
   <pdfuaid:part>1</pdfuaid:part>
  </rdf:Description>
"#
    } else {
        ""
    };

    format!(
        "<?xpacket begin=\"\u{FEFF}\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?>\n\
<x:xmpmeta xmlns:x=\"adobe:ns:meta/\">\n \
<rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\n  \
<rdf:Description rdf:about=\"\"\n     \
xmlns:dc=\"http://purl.org/dc/elements/1.1/\"\n     \
xmlns:xmp=\"http://ns.adobe.com/xap/1.0/\">\n   \
<dc:title>\n    <rdf:Alt>\n     \
<rdf:li xml:lang=\"x-default\">{title}</rdf:li>\n    \
</rdf:Alt>\n   </dc:title>\n   \
<dc:format>application/pdf</dc:format>\n   \
<xmp:CreatorTool>{tool}</xmp:CreatorTool>\n  \
</rdf:Description>\n\
{pdfa}{pdfua}\
</rdf:RDF>\n\
</x:xmpmeta>\n\
<?xpacket end=\"w\"?>",
        title = title_str,
        tool = creator_tool,
        pdfa = pdfa_part,
        pdfua = pdfua_part,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_default() {
        let c = PdfCompliance::default();
        assert_eq!(c, PdfCompliance::Standard);
    }

    #[test]
    fn test_compliance_pdfa_flags() {
        assert!(PdfCompliance::PdfA1b.requires_pdfa());
        assert!(!PdfCompliance::PdfA1b.requires_pdfua());
        assert!(PdfCompliance::PdfA1bUA1.requires_pdfa());
        assert!(PdfCompliance::PdfA1bUA1.requires_pdfua());
        assert!(!PdfCompliance::Standard.requires_pdfa());
    }

    #[test]
    fn test_xmp_metadata_pdfa() {
        let xmp = generate_xmp_metadata(Some("Test Doc"), "fop-rs", PdfCompliance::PdfA1b);
        assert!(xmp.contains("pdfaid:part"));
        assert!(xmp.contains("<pdfaid:conformance>B</pdfaid:conformance>"));
        assert!(!xmp.contains("pdfuaid"));
    }

    #[test]
    fn test_xmp_metadata_pdfua() {
        let xmp = generate_xmp_metadata(None, "fop-rs", PdfCompliance::PdfUA1);
        assert!(!xmp.contains("pdfaid:part"));
        assert!(xmp.contains("pdfuaid:part"));
    }

    #[test]
    fn test_xmp_metadata_combined() {
        let xmp = generate_xmp_metadata(Some("Test"), "fop-rs", PdfCompliance::PdfA1bUA1);
        assert!(xmp.contains("pdfaid:part"));
        assert!(xmp.contains("pdfuaid:part"));
    }

    #[test]
    fn test_srgb_icc_profile_size() {
        // The profile header declares its total byte count in the first 4 bytes.
        // Verify that the declared size matches the actual compile-time array length.
        assert!(
            SRGB_ICC_PROFILE.len() >= 128,
            "ICC profile must be at least 128 bytes (header only)"
        );
        let declared = u32::from_be_bytes([
            SRGB_ICC_PROFILE[0],
            SRGB_ICC_PROFILE[1],
            SRGB_ICC_PROFILE[2],
            SRGB_ICC_PROFILE[3],
        ]) as usize;
        assert_eq!(
            declared,
            SRGB_ICC_PROFILE.len(),
            "ICC header declares {declared} bytes but array has {} bytes",
            SRGB_ICC_PROFILE.len()
        );
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    #[test]
    fn test_compliance_standard_requires_nothing() {
        let c = PdfCompliance::Standard;
        assert!(!c.requires_pdfa());
        assert!(!c.requires_pdfua());
    }

    #[test]
    fn test_compliance_pdfua_only() {
        let c = PdfCompliance::PdfUA1;
        assert!(!c.requires_pdfa());
        assert!(c.requires_pdfua());
    }

    #[test]
    fn test_compliance_pdfa_variant_name() {
        // Ensure all enum variants are distinct
        assert_ne!(PdfCompliance::Standard, PdfCompliance::PdfA1b);
        assert_ne!(PdfCompliance::PdfA1b, PdfCompliance::PdfUA1);
        assert_ne!(PdfCompliance::PdfUA1, PdfCompliance::PdfA1bUA1);
    }

    #[test]
    fn test_xmp_standard_contains_no_compliance_ids() {
        let xmp = generate_xmp_metadata(Some("Doc"), "fop-rs", PdfCompliance::Standard);
        assert!(!xmp.contains("pdfaid"));
        assert!(!xmp.contains("pdfuaid"));
    }

    #[test]
    fn test_xmp_metadata_contains_title() {
        let xmp = generate_xmp_metadata(Some("My Title"), "fop-rs", PdfCompliance::Standard);
        assert!(xmp.contains("My Title"));
    }

    #[test]
    fn test_xmp_metadata_contains_creator_tool() {
        let xmp = generate_xmp_metadata(None, "fop-render v1.0", PdfCompliance::Standard);
        assert!(xmp.contains("fop-render v1.0"));
    }

    #[test]
    fn test_xmp_metadata_no_title_uses_untitled() {
        let xmp = generate_xmp_metadata(None, "fop", PdfCompliance::Standard);
        assert!(xmp.contains("Untitled"));
    }

    #[test]
    fn test_xmp_metadata_starts_with_xpacket() {
        let xmp = generate_xmp_metadata(None, "fop", PdfCompliance::Standard);
        assert!(xmp.starts_with("<?xpacket"));
    }

    #[test]
    fn test_xmp_metadata_ends_with_xpacket() {
        let xmp = generate_xmp_metadata(None, "fop", PdfCompliance::Standard);
        assert!(xmp.ends_with("?>"));
    }

    #[test]
    fn test_srgb_icc_profile_starts_with_signature() {
        // ICC profile class for monitor (mntr) at offset 12: 0x6D 0x6E 0x74 0x72
        assert_eq!(
            &SRGB_ICC_PROFILE[12..16],
            &[0x6D, 0x6E, 0x74, 0x72],
            "ICC profile class should be 'mntr'"
        );
    }

    #[test]
    fn test_srgb_icc_profile_colour_space_rgb() {
        // Colour space at offset 16–19 should be 'RGB ' (0x52 0x47 0x42 0x20)
        assert_eq!(
            &SRGB_ICC_PROFILE[16..20],
            &[0x52, 0x47, 0x42, 0x20],
            "ICC colour space should be 'RGB '"
        );
    }

    #[test]
    fn test_srgb_icc_profile_pcs_xyz() {
        // PCS (Profile Connection Space) at offset 20–23 should be 'XYZ ' (0x58 0x59 0x5A 0x20)
        assert_eq!(
            &SRGB_ICC_PROFILE[20..24],
            &[0x58, 0x59, 0x5A, 0x20],
            "PCS should be 'XYZ '"
        );
    }

    #[test]
    fn test_compliance_copy_clone() {
        let c = PdfCompliance::PdfA1b;
        let c2 = c;
        assert_eq!(c, c2);
    }
}
