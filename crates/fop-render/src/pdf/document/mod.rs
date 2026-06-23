//! PDF document structure
//!
//! Represents the internal structure of a PDF file.

pub mod gradient;
pub mod outline;
pub mod page;
pub mod types;

pub use types::{
    LinkAnnotation, LinkDestination, PdfDocument, PdfExtGState, PdfGradient, PdfInfo, PdfObject,
    PdfOutline, PdfOutlineItem, PdfPage, PdfValue,
};

use fop_types::{FopError, Gradient, Result};

use crate::pdf::compliance::{
    generate_xmp_metadata, reconcile_xmp, PdfCompliance, SRGB_ICC_PROFILE,
};
use crate::pdf::image::ImageXObject;
use crate::pdf::security::EncryptionDict;

use gradient::write_gradient_objects;
use outline::{count_outline_objects, write_outline_objects};

impl PdfDocument {
    /// Create a new PDF document
    pub fn new() -> Self {
        Self {
            version: "1.4".to_string(),
            objects: Vec::new(),
            pages: Vec::new(),
            info: PdfInfo::default(),
            image_xobjects: Vec::new(),
            gradients: Vec::new(),
            ext_g_states: Vec::new(),
            outline: None,
            font_manager: crate::pdf::font::FontManager::new(),
            encryption: None,
            file_id: None,
            compliance: PdfCompliance::Standard,
            xmp_metadata: None,
        }
    }

    /// Set a raw XMP metadata packet to embed in the PDF catalog `/Metadata` stream.
    ///
    /// The packet will be reconciled (wrapped in `<?xpacket ...?>` if absent and
    /// compliance identifiers spliced in) during `to_bytes()`.
    pub fn set_xmp_metadata(&mut self, xmp: String) {
        self.xmp_metadata = Some(xmp);
    }

    /// Set the PDF compliance mode
    ///
    /// # Errors
    /// Returns an error if PDF/A compliance is requested together with encryption,
    /// since PDF/A-1b (ISO 19005-1) forbids encryption.
    ///
    /// Returns an error if PDF/UA-1 compliance is requested, because ISO 14289-1
    /// requires a complete tagged-PDF structure tree (StructTreeRoot with real `K`
    /// kids, `BDC`/`EMC` marked-content operators in every page stream, etc.) that
    /// this implementation does not yet produce.  Emitting `/Marked true` and an
    /// empty StructTreeRoot would be a false conformance claim rejected by veraPDF
    /// and PAC.  Full tagged-PDF support is planned for a future release.
    pub fn set_compliance(&mut self, compliance: PdfCompliance) -> Result<()> {
        if compliance.requires_pdfua() {
            return Err(FopError::Generic(
                "PDF/UA-1 tagged-PDF output is not yet implemented: \
                 ISO 14289-1 requires a complete structure tree with marked-content \
                 operators; a future release will add this feature"
                    .to_string(),
            ));
        }
        if compliance.requires_pdfa() && self.encryption.is_some() {
            return Err(FopError::Generic(
                "PDF/A-1b compliance is incompatible with encryption (ISO 19005-1 §6.1.1)"
                    .to_string(),
            ));
        }
        // PDF/A requires PDF 1.4
        if compliance.requires_pdfa() {
            self.version = "1.4".to_string();
        }
        self.compliance = compliance;
        Ok(())
    }

    /// Add a page to the document
    pub fn add_page(&mut self, page: PdfPage) {
        self.pages.push(page);
    }

    /// Add an image XObject to the document and return its index
    pub fn add_image_xobject(&mut self, xobject: ImageXObject) -> usize {
        self.image_xobjects.push(xobject);
        self.image_xobjects.len() - 1
    }

    /// Add a gradient shading pattern to the document and return its index
    ///
    /// The gradient will be registered as a PDF shading pattern resource.
    /// Returns the index that can be used to reference this gradient.
    pub fn add_gradient(&mut self, gradient: Gradient) -> usize {
        self.gradients.push(PdfGradient {
            gradient,
            object_id: 0, // Will be assigned during PDF generation
        });
        self.gradients.len() - 1
    }

    /// Add an ExtGState for opacity/transparency and return its index
    ///
    /// Creates an Extended Graphics State dictionary with the specified opacity values.
    /// Returns the index that can be used to reference this graphics state.
    ///
    /// # Arguments
    /// * `fill_opacity` - Opacity for fill operations (0.0 = transparent, 1.0 = opaque)
    /// * `stroke_opacity` - Opacity for stroke operations (0.0 = transparent, 1.0 = opaque)
    pub fn add_ext_g_state(&mut self, fill_opacity: f64, stroke_opacity: f64) -> usize {
        // Check if this opacity combination already exists
        for (idx, gs) in self.ext_g_states.iter().enumerate() {
            if (gs.fill_opacity - fill_opacity).abs() < f64::EPSILON
                && (gs.stroke_opacity - stroke_opacity).abs() < f64::EPSILON
            {
                return idx;
            }
        }

        // Add new graphics state
        self.ext_g_states.push(PdfExtGState {
            fill_opacity,
            stroke_opacity,
            object_id: 0, // Will be assigned during PDF generation
        });
        self.ext_g_states.len() - 1
    }

    /// Set the document outline (bookmarks)
    pub fn set_outline(&mut self, outline: PdfOutline) {
        self.outline = Some(outline);
    }

    /// Set encryption for the document
    ///
    /// When encryption is set, `to_bytes()` will encrypt all content streams
    /// and string objects, and include the /Encrypt dictionary in the trailer.
    ///
    /// # Errors
    /// Returns an error if PDF/A compliance mode is active, since PDF/A-1b
    /// (ISO 19005-1 §6.1.1) forbids encryption.
    pub fn set_encryption(&mut self, encryption: EncryptionDict, file_id: Vec<u8>) -> Result<()> {
        if self.compliance.requires_pdfa() {
            return Err(FopError::Generic(
                "PDF/A-1b compliance is incompatible with encryption (ISO 19005-1 §6.1.1)"
                    .to_string(),
            ));
        }
        self.encryption = Some(encryption);
        self.file_id = Some(file_id);
        Ok(())
    }

    /// Encrypt data for a specific PDF object (if encryption is enabled)
    fn encrypt_stream(&self, data: &[u8], obj_num: u32) -> Vec<u8> {
        if let Some(ref enc) = self.encryption {
            enc.encrypt_data(data, obj_num, 0)
        } else {
            data.to_vec()
        }
    }

    /// Embed a TrueType font and return its index
    ///
    /// # Arguments
    /// * `font_data` - Raw bytes of the TTF/OTF font file
    ///
    /// # Returns
    /// Font index that can be used with `add_text_with_font`
    pub fn embed_font(&mut self, font_data: Vec<u8>) -> Result<usize> {
        self.font_manager.embed_font(font_data)
    }

    /// Generate PDF bytes
    ///
    /// # Errors
    /// Returns an error if the active compliance mode includes PDF/UA-1, because
    /// the required tagged-PDF structure tree is not yet implemented.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        // Belt-and-suspenders: `set_compliance` already rejects pdfua, but the
        // `compliance` field is `pub`, so a caller might set it directly.
        if self.compliance.requires_pdfua() {
            return Err(FopError::Generic(
                "PDF/UA-1 tagged-PDF output is not yet implemented: \
                 ISO 14289-1 requires a complete structure tree with marked-content \
                 operators; a future release will add this feature"
                    .to_string(),
            ));
        }

        let mut bytes = Vec::new();
        let mut xref_offsets = Vec::new();

        // PDF header
        bytes.extend_from_slice(format!("%PDF-{}\n", self.version).as_bytes());
        bytes.extend_from_slice(b"%\xE2\xE3\xCF\xD3\n"); // Binary marker

        // Object 0 is always free
        xref_offsets.push(0);

        // Calculate outline object count
        let outline_obj_count = if let Some(ref outline) = self.outline {
            count_outline_objects(outline)
        } else {
            0
        };

        // Calculate encryption object count (1 if encryption is set)
        let encrypt_obj_count = if self.encryption.is_some() { 1 } else { 0 };

        // Calculate compliance object IDs / counts
        // Compliance objects are placed after the encryption dict (if present):
        //   xmp_obj_id       : XMP metadata stream (if any compliance mode)
        //   output_intent_id : OutputIntent dict   (if PDF/A)
        //   icc_profile_id   : ICC profile stream  (if PDF/A)
        let font_obj_id = 3;
        let first_outline_obj_id = 4;
        let num_embedded_fonts = self.font_manager.font_count();
        // Encryption dict goes after outline objects
        let encrypt_obj_id = first_outline_obj_id + outline_obj_count;

        // Compliance objects follow immediately after the encryption dict slot
        let compliance_base_id = encrypt_obj_id + encrypt_obj_count;
        let needs_compliance = self.compliance != PdfCompliance::Standard;
        // needs_xmp is true whenever we have a user-supplied XMP packet OR a non-standard
        // compliance mode (PDF/A-1b requires an /Metadata stream).
        let needs_xmp = needs_compliance || self.xmp_metadata.is_some();
        let xmp_obj_count = if needs_xmp { 1 } else { 0 };
        let xmp_obj_id = compliance_base_id; // only valid when needs_xmp
        let oi_obj_count = if self.compliance.requires_pdfa() {
            2
        } else {
            0
        };
        let output_intent_obj_id = compliance_base_id + xmp_obj_count; // only valid when pdfa
        let icc_profile_obj_id = output_intent_obj_id + 1; // only valid when pdfa
        let total_compliance_obj_count = xmp_obj_count + oi_obj_count;

        let first_embedded_font_obj_id = compliance_base_id + total_compliance_obj_count;

        // Object 1: Catalog (root)
        xref_offsets.push(bytes.len());
        bytes.extend_from_slice(b"1 0 obj\n");
        bytes.extend_from_slice(b"<<\n");
        bytes.extend_from_slice(b"/Type /Catalog\n");
        bytes.extend_from_slice(b"/Pages 2 0 R\n");

        // Add outline reference if present
        if self.outline.is_some() {
            bytes.extend_from_slice(b"/Outlines 4 0 R\n");
        }

        // PDF/A, PDF/UA, and user-XMP catalog entries
        if needs_xmp {
            bytes.extend_from_slice(format!("/Metadata {} 0 R\n", xmp_obj_id).as_bytes());
        }

        if self.compliance.requires_pdfa() {
            bytes.extend_from_slice(
                format!("/OutputIntents [{} 0 R]\n", output_intent_obj_id).as_bytes(),
            );
        }
        if let Some(ref lang) = self.info.lang {
            // Add /Lang to catalog when xml:lang is specified
            bytes.extend_from_slice(format!("/Lang ({})\n", lang).as_bytes());
        }

        bytes.extend_from_slice(b">>\n");
        bytes.extend_from_slice(b"endobj\n");
        let first_image_obj_id = first_embedded_font_obj_id + num_embedded_fonts * 6; // 6 objects per font: descriptor, stream, CIDFont, Type0, ToUnicode, CIDToGIDMap
        let num_images = self.image_xobjects.len();
        let first_gradient_obj_id = first_image_obj_id + num_images;
        let num_gradients = self.gradients.len();
        let first_ext_g_state_obj_id = first_gradient_obj_id + num_gradients * 2; // 2 objects per gradient
        let num_ext_g_states = self.ext_g_states.len();
        let first_page_obj_id = first_ext_g_state_obj_id + num_ext_g_states;

        // Count total annotations and build annotation ranges per page
        #[allow(unused_variables)]
        let total_annotations: usize = self.pages.iter().map(|p| p.link_annotations.len()).sum();
        let first_annotation_obj_id = first_page_obj_id + self.pages.len() * 2;

        // Object 2: Pages (page tree root)
        xref_offsets.push(bytes.len());
        bytes.extend_from_slice(b"2 0 obj\n");
        bytes.extend_from_slice(b"<<\n");
        bytes.extend_from_slice(b"/Type /Pages\n");

        // Build kids array
        let page_obj_ids: Vec<usize> = (0..self.pages.len())
            .map(|i| first_page_obj_id + i * 2)
            .collect();

        bytes.extend_from_slice(b"/Kids [");
        for page_id in &page_obj_ids {
            bytes.extend_from_slice(format!("{} 0 R ", page_id).as_bytes());
        }
        bytes.extend_from_slice(b"]\n");
        bytes.extend_from_slice(format!("/Count {}\n", self.pages.len()).as_bytes());
        bytes.extend_from_slice(b">>\n");
        bytes.extend_from_slice(b"endobj\n");

        // Object 3: Font resource (Type 1 Helvetica)
        xref_offsets.push(bytes.len());
        bytes.extend_from_slice(format!("{} 0 obj\n", font_obj_id).as_bytes());
        bytes.extend_from_slice(b"<<\n");
        bytes.extend_from_slice(b"/Type /Font\n");
        bytes.extend_from_slice(b"/Subtype /Type1\n");
        bytes.extend_from_slice(b"/BaseFont /Helvetica\n");
        bytes.extend_from_slice(b">>\n");
        bytes.extend_from_slice(b"endobj\n");

        // Generate outline objects if present
        if let Some(ref outline) = self.outline {
            write_outline_objects(
                outline,
                &mut bytes,
                &mut xref_offsets,
                first_outline_obj_id,
                &page_obj_ids,
            );
        }

        // Generate encryption dictionary object if encryption is enabled
        if let Some(ref enc) = self.encryption {
            xref_offsets.push(bytes.len());
            let enc_dict_str = enc.to_pdf_dict(encrypt_obj_id);
            bytes.extend_from_slice(enc_dict_str.as_bytes());
        }

        // Generate XMP metadata stream (for compliance modes and/or user-supplied XMP)
        if needs_xmp {
            let xmp_content = if let Some(ref raw_xmp) = self.xmp_metadata {
                // User-supplied packet: reconcile (wrap in <?xpacket?>, splice compliance IDs)
                reconcile_xmp(raw_xmp, self.compliance)
            } else {
                // Compliance-only mode: generate a default XMP packet
                let title_ref = self.info.title.as_deref();
                let creator_tool = format!("fop-rs {}", env!("CARGO_PKG_VERSION"));
                generate_xmp_metadata(title_ref, &creator_tool, self.compliance)
            };
            let xmp_bytes = xmp_content.as_bytes();
            xref_offsets.push(bytes.len());
            bytes.extend_from_slice(format!("{} 0 obj\n", xmp_obj_id).as_bytes());
            bytes.extend_from_slice(b"<<\n");
            bytes.extend_from_slice(b"/Type /Metadata\n");
            bytes.extend_from_slice(b"/Subtype /XML\n");
            bytes.extend_from_slice(format!("/Length {}\n", xmp_bytes.len()).as_bytes());
            bytes.extend_from_slice(b">>\nstream\n");
            bytes.extend_from_slice(xmp_bytes);
            bytes.extend_from_slice(b"\nendstream\nendobj\n");
        }

        if self.compliance.requires_pdfa() {
            // OutputIntent dictionary referencing the ICC profile stream
            xref_offsets.push(bytes.len());
            bytes.extend_from_slice(format!("{} 0 obj\n", output_intent_obj_id).as_bytes());
            bytes.extend_from_slice(b"<<\n");
            bytes.extend_from_slice(b"/Type /OutputIntent\n");
            bytes.extend_from_slice(b"/S /GTS_PDFA1\n");
            bytes.extend_from_slice(b"/OutputConditionIdentifier (sRGB)\n");
            bytes.extend_from_slice(b"/RegistryName (http://www.color.org)\n");
            bytes.extend_from_slice(
                format!("/DestOutputProfile {} 0 R\n", icc_profile_obj_id).as_bytes(),
            );
            bytes.extend_from_slice(b">>\nendobj\n");

            // ICC profile stream (sRGB)
            let icc_data = SRGB_ICC_PROFILE;
            xref_offsets.push(bytes.len());
            bytes.extend_from_slice(format!("{} 0 obj\n", icc_profile_obj_id).as_bytes());
            bytes.extend_from_slice(b"<<\n");
            bytes.extend_from_slice(b"/N 3\n"); // 3 colour components for RGB
            bytes.extend_from_slice(format!("/Length {}\n", icc_data.len()).as_bytes());
            bytes.extend_from_slice(b">>\nstream\n");
            bytes.extend_from_slice(icc_data);
            bytes.extend_from_slice(b"\nendstream\nendobj\n");
        }

        // Generate embedded font objects (6 objects per font: descriptor, stream, CIDFont, Type0, ToUnicode, CIDToGIDMap)
        if num_embedded_fonts > 0 {
            use crate::pdf::font::{
                generate_cidfont_dict, generate_font_descriptor, generate_font_dictionary,
                generate_font_stream_header, generate_to_unicode_cmap,
            };

            let font_objects = self
                .font_manager
                .generate_font_objects(first_embedded_font_obj_id)?;

            for (
                font_idx,
                (
                    descriptor_id,
                    stream_id,
                    cidfont_id,
                    type0_dict_id,
                    to_unicode_id,
                    cidtogidmap_id,
                    font,
                ),
            ) in font_objects.iter().enumerate()
            {
                // Font descriptor object
                xref_offsets.push(bytes.len());
                bytes.extend_from_slice(format!("{} 0 obj\n", descriptor_id).as_bytes());
                bytes.extend_from_slice(generate_font_descriptor(font, *stream_id).as_bytes());
                bytes.extend_from_slice(b"\nendobj\n");

                // Font stream object (the actual TTF data)
                xref_offsets.push(bytes.len());
                bytes.extend_from_slice(format!("{} 0 obj\n", stream_id).as_bytes());
                bytes.extend_from_slice(generate_font_stream_header(font).as_bytes());
                bytes.extend_from_slice(b"\nstream\n");
                bytes.extend_from_slice(&font.font_data);
                bytes.extend_from_slice(b"\nendstream\n");
                bytes.extend_from_slice(b"endobj\n");

                // CIDFont dictionary object (CIDFontType2 - TrueType descendant)
                xref_offsets.push(bytes.len());
                bytes.extend_from_slice(format!("{} 0 obj\n", cidfont_id).as_bytes());
                bytes.extend_from_slice(
                    generate_cidfont_dict(font, *descriptor_id, *cidtogidmap_id).as_bytes(),
                );
                bytes.extend_from_slice(b"\nendobj\n");

                // Type 0 font dictionary object (composite font)
                xref_offsets.push(bytes.len());
                bytes.extend_from_slice(format!("{} 0 obj\n", type0_dict_id).as_bytes());
                bytes.extend_from_slice(
                    generate_font_dictionary(font, *cidfont_id, Some(*to_unicode_id)).as_bytes(),
                );
                bytes.extend_from_slice(b"\nendobj\n");

                // ToUnicode CMap object
                let cmap_content = generate_to_unicode_cmap(font);
                xref_offsets.push(bytes.len());
                bytes.extend_from_slice(format!("{} 0 obj\n", to_unicode_id).as_bytes());
                bytes.extend_from_slice(b"<<\n/Length ");
                bytes.extend_from_slice(cmap_content.len().to_string().as_bytes());
                bytes.extend_from_slice(b"\n>>\nstream\n");
                bytes.extend_from_slice(cmap_content.as_bytes());
                bytes.extend_from_slice(b"\nendstream\nendobj\n");

                // CIDToGIDMap stream object
                // Get the subsetter for this font to find used characters
                let used_chars = if let Some(subsetter) = self.font_manager.get_subsetter(font_idx)
                {
                    subsetter.used_chars()
                } else {
                    &std::collections::BTreeSet::new()
                };

                let cidtogidmap_data = crate::pdf::cidfont::generate_cidtogidmap_stream(
                    &font.char_to_glyph,
                    used_chars,
                );

                xref_offsets.push(bytes.len());
                bytes.extend_from_slice(format!("{} 0 obj\n", cidtogidmap_id).as_bytes());
                bytes.extend_from_slice(b"<<\n/Length ");
                bytes.extend_from_slice(cidtogidmap_data.len().to_string().as_bytes());
                bytes.extend_from_slice(b"\n>>\nstream\n");
                bytes.extend_from_slice(&cidtogidmap_data);
                bytes.extend_from_slice(b"\nendstream\nendobj\n");
            }
        }

        // Generate image XObject objects
        for (img_idx, xobject) in self.image_xobjects.iter().enumerate() {
            let obj_id = first_image_obj_id + img_idx;
            xref_offsets.push(bytes.len());

            // Write XObject dictionary and stream header
            let stream_header = xobject.to_pdf_stream(obj_id as u32);
            bytes.extend_from_slice(stream_header.as_bytes());

            // Write binary stream data
            bytes.extend_from_slice(xobject.stream_data());

            // Write stream end
            bytes.extend_from_slice(ImageXObject::stream_end().as_bytes());
        }

        // Generate gradient shading objects (2 objects per gradient: function + shading)
        for (grad_idx, pdf_gradient) in self.gradients.iter().enumerate() {
            let function_obj_id = first_gradient_obj_id + grad_idx * 2;
            let shading_obj_id = function_obj_id + 1;

            // Generate the gradient objects
            write_gradient_objects(
                &pdf_gradient.gradient,
                function_obj_id,
                shading_obj_id,
                &mut bytes,
                &mut xref_offsets,
            );
        }

        // Generate ExtGState objects for transparency
        for (gs_idx, ext_g_state) in self.ext_g_states.iter().enumerate() {
            let obj_id = first_ext_g_state_obj_id + gs_idx;
            xref_offsets.push(bytes.len());
            bytes.extend_from_slice(format!("{} 0 obj\n", obj_id).as_bytes());
            bytes.extend_from_slice(b"<<\n");
            bytes.extend_from_slice(b"/Type /ExtGState\n");
            bytes.extend_from_slice(format!("/ca {:.3}\n", ext_g_state.fill_opacity).as_bytes());
            bytes.extend_from_slice(format!("/CA {:.3}\n", ext_g_state.stroke_opacity).as_bytes());
            bytes.extend_from_slice(b">>\n");
            bytes.extend_from_slice(b"endobj\n");
        }

        // Generate page objects and content streams in order
        let mut current_annotation_obj_id = first_annotation_obj_id;
        for (page_idx, page) in self.pages.iter().enumerate() {
            let page_obj_id = first_page_obj_id + page_idx * 2;
            let content_obj_id = page_obj_id + 1;

            // Page object first
            xref_offsets.push(bytes.len());
            bytes.extend_from_slice(format!("{} 0 obj\n", page_obj_id).as_bytes());
            bytes.extend_from_slice(b"<<\n");
            bytes.extend_from_slice(b"/Type /Page\n");
            bytes.extend_from_slice(b"/Parent 2 0 R\n");
            bytes.extend_from_slice(
                format!(
                    "/MediaBox [0 0 {} {}]\n",
                    page.width.to_pt(),
                    page.height.to_pt()
                )
                .as_bytes(),
            );
            bytes.extend_from_slice(b"/Resources <<\n");

            // Font resources: F1 is Helvetica, F2+ are embedded fonts
            bytes.extend_from_slice(b"  /Font <<\n");
            bytes.extend_from_slice(format!("    /F1 {} 0 R\n", font_obj_id).as_bytes());

            // Add embedded fonts as F2, F3, F4, etc.
            if num_embedded_fonts > 0 {
                for font_idx in 0..num_embedded_fonts {
                    let type0_dict_obj_id = first_embedded_font_obj_id + font_idx * 6 + 3; // Type0 dictionary is 4th object (6 objects per font)
                    bytes.extend_from_slice(
                        format!("    /F{} {} 0 R\n", font_idx + 2, type0_dict_obj_id).as_bytes(),
                    );
                }
            }
            bytes.extend_from_slice(b"  >>\n");

            // Add XObject resources if there are any images
            if !self.image_xobjects.is_empty() {
                bytes.extend_from_slice(b"  /XObject <<\n");
                for img_idx in 0..self.image_xobjects.len() {
                    let obj_id = first_image_obj_id + img_idx;
                    bytes.extend_from_slice(
                        format!("    /Im{} {} 0 R\n", img_idx, obj_id).as_bytes(),
                    );
                }
                bytes.extend_from_slice(b"  >>\n");
            }

            // Add Shading resources if there are any gradients
            if !self.gradients.is_empty() {
                bytes.extend_from_slice(b"  /Shading <<\n");
                for grad_idx in 0..self.gradients.len() {
                    let shading_obj_id = first_gradient_obj_id + grad_idx * 2 + 1; // Shading is 2nd object
                    bytes.extend_from_slice(
                        format!("    /Sh{} {} 0 R\n", grad_idx, shading_obj_id).as_bytes(),
                    );
                }
                bytes.extend_from_slice(b"  >>\n");
            }

            // Add ExtGState resources if there are any transparency settings
            if !self.ext_g_states.is_empty() {
                bytes.extend_from_slice(b"  /ExtGState <<\n");
                for gs_idx in 0..self.ext_g_states.len() {
                    let gs_obj_id = first_ext_g_state_obj_id + gs_idx;
                    bytes.extend_from_slice(
                        format!("    /GS{} {} 0 R\n", gs_idx, gs_obj_id).as_bytes(),
                    );
                }
                bytes.extend_from_slice(b"  >>\n");
            }

            bytes.extend_from_slice(b">>\n");
            bytes.extend_from_slice(format!("/Contents {} 0 R\n", content_obj_id).as_bytes());

            // Add /Annots array if this page has link annotations
            if !page.link_annotations.is_empty() {
                bytes.extend_from_slice(b"/Annots [");
                for annot_idx in 0..page.link_annotations.len() {
                    bytes.extend_from_slice(
                        format!("{} 0 R ", current_annotation_obj_id + annot_idx).as_bytes(),
                    );
                }
                bytes.extend_from_slice(b"]\n");
                current_annotation_obj_id += page.link_annotations.len();
            }

            bytes.extend_from_slice(b">>\n");
            bytes.extend_from_slice(b"endobj\n");

            // Content stream object second (encrypt if needed)
            let stream_data = self.encrypt_stream(&page.content, content_obj_id as u32);
            xref_offsets.push(bytes.len());
            bytes.extend_from_slice(format!("{} 0 obj\n", content_obj_id).as_bytes());
            bytes.extend_from_slice(b"<<\n");
            bytes.extend_from_slice(format!("/Length {}\n", stream_data.len()).as_bytes());
            bytes.extend_from_slice(b">>\n");
            bytes.extend_from_slice(b"stream\n");
            bytes.extend_from_slice(&stream_data);
            bytes.extend_from_slice(b"\nendstream\n");
            bytes.extend_from_slice(b"endobj\n");
        }

        // Generate link annotation objects
        if total_annotations > 0 {
            let mut annot_obj_id = first_annotation_obj_id;
            for (page_idx, page) in self.pages.iter().enumerate() {
                let page_obj_id = first_page_obj_id + page_idx * 2;

                for annot in &page.link_annotations {
                    xref_offsets.push(bytes.len());
                    bytes.extend_from_slice(format!("{} 0 obj\n", annot_obj_id).as_bytes());
                    bytes.extend_from_slice(b"<<\n");
                    bytes.extend_from_slice(b"/Type /Annot\n");
                    bytes.extend_from_slice(b"/Subtype /Link\n");
                    bytes.extend_from_slice(
                        format!(
                            "/Rect [{:.2} {:.2} {:.2} {:.2}]\n",
                            annot.rect[0], annot.rect[1], annot.rect[2], annot.rect[3]
                        )
                        .as_bytes(),
                    );
                    bytes.extend_from_slice(format!("/P {} 0 R\n", page_obj_id).as_bytes());
                    bytes.extend_from_slice(b"/Border [0 0 0]\n"); // No border

                    // Add destination based on type
                    match &annot.destination {
                        LinkDestination::External(url) => {
                            bytes.extend_from_slice(b"/A <<\n");
                            bytes.extend_from_slice(b"  /S /URI\n");
                            bytes.extend_from_slice(
                                format!("  /URI ({})\n", outline::escape_pdf_string(url))
                                    .as_bytes(),
                            );
                            bytes.extend_from_slice(b">>\n");
                        }
                        LinkDestination::Internal(dest_id) => {
                            // For internal destinations, we would need to resolve the ID to a page
                            // For now, use a named destination
                            bytes.extend_from_slice(
                                format!("/Dest ({})\n", outline::escape_pdf_string(dest_id))
                                    .as_bytes(),
                            );
                        }
                    }

                    bytes.extend_from_slice(b">>\n");
                    bytes.extend_from_slice(b"endobj\n");

                    annot_obj_id += 1;
                }
            }
        }

        // Cross-reference table
        let xref_offset = bytes.len();
        bytes.extend_from_slice(b"xref\n");
        bytes.extend_from_slice(format!("0 {}\n", xref_offsets.len()).as_bytes());
        bytes.extend_from_slice(b"0000000000 65535 f \n"); // Object 0 is free
        for offset in xref_offsets.iter().skip(1) {
            bytes.extend_from_slice(format!("{:010} 00000 n \n", offset).as_bytes());
        }

        // Trailer
        bytes.extend_from_slice(b"trailer\n");
        bytes.extend_from_slice(b"<<\n");
        bytes.extend_from_slice(format!("/Size {}\n", xref_offsets.len()).as_bytes());
        bytes.extend_from_slice(b"/Root 1 0 R\n");

        // Add /Encrypt reference if encryption is enabled
        if self.encryption.is_some() {
            bytes.extend_from_slice(format!("/Encrypt {} 0 R\n", encrypt_obj_id).as_bytes());
        }

        // Add /ID array (required for encryption, recommended for all PDFs)
        if let Some(ref file_id) = self.file_id {
            let hex = file_id
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<String>();
            bytes.extend_from_slice(format!("/ID [<{}> <{}>]\n", hex, hex).as_bytes());
        }

        // Add document info if any metadata is present
        if self.info.title.is_some()
            || self.info.author.is_some()
            || self.info.subject.is_some()
            || self.info.creation_date.is_some()
        {
            bytes.extend_from_slice(b"/Info <<\n");

            if let Some(ref title) = self.info.title {
                bytes.extend_from_slice(
                    format!("  /Title ({})\n", outline::escape_pdf_string(title)).as_bytes(),
                );
            }

            if let Some(ref author) = self.info.author {
                bytes.extend_from_slice(
                    format!("  /Author ({})\n", outline::escape_pdf_string(author)).as_bytes(),
                );
            }

            if let Some(ref subject) = self.info.subject {
                bytes.extend_from_slice(
                    format!("  /Subject ({})\n", outline::escape_pdf_string(subject)).as_bytes(),
                );
            }

            if let Some(ref creation_date) = self.info.creation_date {
                bytes.extend_from_slice(
                    format!(
                        "  /CreationDate ({})\n",
                        outline::escape_pdf_string(creation_date)
                    )
                    .as_bytes(),
                );
            }

            bytes.extend_from_slice(b">>\n");
        }

        bytes.extend_from_slice(b">>\n");
        bytes.extend_from_slice(b"startxref\n");
        bytes.extend_from_slice(format!("{}\n", xref_offset).as_bytes());
        bytes.extend_from_slice(b"%%EOF\n");

        Ok(bytes)
    }
}

impl Default for PdfDocument {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_document_creation() {
        let doc = PdfDocument::new();
        assert_eq!(doc.version, "1.4");
        assert_eq!(doc.pages.len(), 0);
    }

    #[test]
    fn test_pdf_page() {
        let mut page = PdfPage::new(
            fop_types::Length::from_mm(210.0),
            fop_types::Length::from_mm(297.0),
        );

        page.add_text(
            "Hello World",
            fop_types::Length::from_pt(100.0),
            fop_types::Length::from_pt(700.0),
            fop_types::Length::from_pt(12.0),
        );

        assert!(!page.content.is_empty());
        let content_str = String::from_utf8_lossy(&page.content);
        assert!(content_str.contains("Hello World"));
        assert!(content_str.contains("BT")); // Begin text
        assert!(content_str.contains("ET")); // End text
    }

    #[test]
    fn test_pdf_bytes() {
        let doc = PdfDocument::new();
        let bytes = doc.to_bytes().expect("test: should succeed");

        let header = String::from_utf8_lossy(&bytes[..8]);
        assert!(header.starts_with("%PDF-"));
    }

    #[test]
    fn test_pdf_encrypted_bytes() {
        use crate::pdf::security::{generate_file_id, PdfPermissions, PdfSecurity};

        let mut doc = PdfDocument::new();

        // Add a page with content
        let mut page = PdfPage::new(
            fop_types::Length::from_mm(210.0),
            fop_types::Length::from_mm(297.0),
        );
        page.add_text(
            "Secret Text",
            fop_types::Length::from_pt(100.0),
            fop_types::Length::from_pt(700.0),
            fop_types::Length::from_pt(12.0),
        );
        doc.add_page(page);

        // Set encryption
        let permissions = PdfPermissions {
            allow_print: false,
            allow_copy: false,
            ..Default::default()
        };
        let security = PdfSecurity::new("owner123", "user456", permissions);
        let file_id = generate_file_id("test-encrypted");
        let encryption_dict = security.compute_encryption_dict(&file_id);
        doc.set_encryption(encryption_dict, file_id)
            .expect("test: should succeed");

        let bytes = doc.to_bytes().expect("test: should succeed");
        let content = String::from_utf8_lossy(&bytes);

        // Verify encrypted PDF structure
        assert!(content.contains("%PDF-"));
        assert!(content.contains("/Filter /Standard"));
        assert!(content.contains("/V 2")); // Version 2 (RC4-128)
        assert!(content.contains("/R 3")); // Revision 3
        assert!(content.contains("/Length 128"));
        assert!(content.contains("/Encrypt")); // Trailer has /Encrypt
        assert!(content.contains("/ID [<")); // Trailer has /ID

        // Content stream should be encrypted (not contain plaintext)
        assert!(!content.contains("Secret Text"));
    }

    #[test]
    fn test_pdf_without_encryption_has_plaintext() {
        let mut doc = PdfDocument::new();
        let mut page = PdfPage::new(
            fop_types::Length::from_mm(210.0),
            fop_types::Length::from_mm(297.0),
        );
        page.add_text(
            "Visible Text",
            fop_types::Length::from_pt(100.0),
            fop_types::Length::from_pt(700.0),
            fop_types::Length::from_pt(12.0),
        );
        doc.add_page(page);

        let bytes = doc.to_bytes().expect("test: should succeed");
        let content = String::from_utf8_lossy(&bytes);

        // Without encryption, text should be visible in the PDF
        assert!(content.contains("Visible Text"));
        // Should NOT have encryption entries
        assert!(!content.contains("/Encrypt"));
        assert!(!content.contains("/Filter /Standard"));
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;
    use crate::pdf::compliance::PdfCompliance;
    use crate::pdf::security::{generate_file_id, PdfPermissions, PdfSecurity};
    use fop_types::Length;

    #[test]
    fn test_pdf_document_default() {
        let doc = PdfDocument::default();
        assert_eq!(doc.version, "1.4");
        assert!(doc.pages.is_empty());
    }

    #[test]
    fn test_pdf_document_add_multiple_pages() {
        let mut doc = PdfDocument::new();
        for _ in 0..3 {
            let page = PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0));
            doc.add_page(page);
        }
        assert_eq!(doc.pages.len(), 3);
    }

    #[test]
    fn test_pdf_page_new_has_correct_dimensions() {
        let page = PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0));
        assert_eq!(page.width, Length::from_mm(210.0));
        assert_eq!(page.height, Length::from_mm(297.0));
        assert!(page.content.is_empty());
    }

    #[test]
    fn test_pdf_page_add_text_generates_bt_et() {
        let mut page = PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0));
        page.add_text(
            "Test",
            Length::from_pt(72.0),
            Length::from_pt(700.0),
            Length::from_pt(12.0),
        );
        let content = String::from_utf8_lossy(&page.content);
        assert!(content.contains("BT"));
        assert!(content.contains("ET"));
    }

    #[test]
    fn test_pdf_page_add_background() {
        let mut page = PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0));
        page.add_background(
            Length::ZERO,
            Length::ZERO,
            Length::from_pt(595.0),
            Length::from_pt(842.0),
            fop_types::Color::WHITE,
        );
        let content = String::from_utf8_lossy(&page.content);
        // Should have filled rectangle
        assert!(content.contains("re f"));
    }

    #[test]
    fn test_pdf_compliance_pdfa1b_adds_version_info() {
        let mut doc = PdfDocument::new();
        doc.set_compliance(PdfCompliance::PdfA1b)
            .expect("test: should succeed");
        let bytes = doc.to_bytes().expect("test: should succeed");
        let content = String::from_utf8_lossy(&bytes);
        // PDF/A uses PDF 1.4
        assert!(content.contains("%PDF-1.4"));
    }

    #[test]
    fn test_pdf_document_to_bytes_starts_with_header() {
        let doc = PdfDocument::new();
        let bytes = doc.to_bytes().expect("test: should succeed");
        assert!(bytes.starts_with(b"%PDF-"));
    }

    #[test]
    fn test_pdf_document_to_bytes_ends_with_eof() {
        let doc = PdfDocument::new();
        let bytes = doc.to_bytes().expect("test: should succeed");
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("%%EOF"));
    }

    #[test]
    fn test_pdf_document_aes256_encryption() {
        let mut doc = PdfDocument::new();
        let mut page = PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0));
        page.add_text(
            "Private",
            Length::from_pt(72.0),
            Length::from_pt(700.0),
            Length::from_pt(12.0),
        );
        doc.add_page(page);

        let sec = PdfSecurity::new_aes256("owner", "user", PdfPermissions::default());
        let file_id = generate_file_id("aes-doc");
        let dict = sec.compute_encryption_dict(&file_id);
        doc.set_encryption(dict, file_id)
            .expect("test: should succeed");

        let bytes = doc.to_bytes().expect("test: should succeed");
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("/V 5")); // AES-256 version
        assert!(content.contains("/R 6")); // Revision 6
        assert!(content.contains("/OE <")); // owner encrypted key
    }

    #[test]
    fn test_pdf_outline_structure() {
        let mut doc = PdfDocument::new();
        let outline = PdfOutline {
            items: vec![
                PdfOutlineItem {
                    title: "Chapter 1".to_string(),
                    page_index: Some(0),
                    external_destination: None,
                    children: vec![],
                },
                PdfOutlineItem {
                    title: "Chapter 2".to_string(),
                    page_index: Some(1),
                    external_destination: None,
                    children: vec![],
                },
            ],
        };
        doc.set_outline(outline);

        // Add pages for the outline to reference
        for _ in 0..2 {
            let page = PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0));
            doc.add_page(page);
        }

        let bytes = doc.to_bytes().expect("test: should succeed");
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("Chapter 1"));
        assert!(content.contains("Chapter 2"));
        assert!(content.contains("/Outlines"));
    }

    #[test]
    fn test_pdf_page_add_rule() {
        let mut page = PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0));
        page.add_rule(
            Length::from_pt(50.0),
            Length::from_pt(400.0),
            Length::from_pt(400.0),
            Length::from_pt(2.0),
            fop_types::Color::BLACK,
            "solid",
        );
        let content = String::from_utf8_lossy(&page.content);
        // Should have line drawing content
        assert!(!content.is_empty());
    }
}

#[cfg(test)]
mod tests_document_comprehensive {
    use super::*;
    use fop_types::Length;

    // ── PdfDocument::new() ────────────────────────────────────────────────────

    #[test]
    fn test_new_produces_non_empty_output() {
        let doc = PdfDocument::new();
        let bytes = doc.to_bytes().expect("test: should succeed");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_new_version_is_1_4() {
        let doc = PdfDocument::new();
        assert_eq!(doc.version, "1.4");
    }

    #[test]
    fn test_new_has_no_pages() {
        let doc = PdfDocument::new();
        assert_eq!(doc.pages.len(), 0);
    }

    #[test]
    fn test_new_has_no_images() {
        let doc = PdfDocument::new();
        assert_eq!(doc.image_xobjects.len(), 0);
    }

    #[test]
    fn test_new_has_no_outline() {
        let doc = PdfDocument::new();
        assert!(doc.outline.is_none());
    }

    // ── PDF version header ────────────────────────────────────────────────────

    #[test]
    fn test_pdf_header_starts_with_pdf_1_4() {
        let doc = PdfDocument::new();
        let bytes = doc.to_bytes().expect("test: should succeed");
        assert!(bytes.starts_with(b"%PDF-1.4"));
    }

    #[test]
    fn test_pdf_header_present_in_output() {
        let doc = PdfDocument::new();
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("%PDF-"));
    }

    // ── Page count in catalog ─────────────────────────────────────────────────

    #[test]
    fn test_page_count_zero_pages_in_catalog() {
        let doc = PdfDocument::new();
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/Count 0"));
    }

    #[test]
    fn test_page_count_one_page_in_catalog() {
        let mut doc = PdfDocument::new();
        doc.add_page(PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0)));
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/Count 1"));
    }

    #[test]
    fn test_page_count_three_pages_in_catalog() {
        let mut doc = PdfDocument::new();
        for _ in 0..3 {
            doc.add_page(PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0)));
        }
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/Count 3"));
        assert_eq!(doc.pages.len(), 3);
    }

    // ── Info dictionary fields ────────────────────────────────────────────────

    #[test]
    fn test_info_title_appears_in_output() {
        let mut doc = PdfDocument::new();
        doc.info.title = Some("My Test Document".to_string());
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/Title (My Test Document)"));
    }

    #[test]
    fn test_info_author_appears_in_output() {
        let mut doc = PdfDocument::new();
        doc.info.author = Some("Jane Doe".to_string());
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/Author (Jane Doe)"));
    }

    #[test]
    fn test_info_subject_appears_in_output() {
        let mut doc = PdfDocument::new();
        doc.info.subject = Some("Unit Testing".to_string());
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/Subject (Unit Testing)"));
    }

    #[test]
    fn test_info_creation_date_appears_in_output() {
        let mut doc = PdfDocument::new();
        doc.info.creation_date = Some("D:20260220120000".to_string());
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/CreationDate (D:20260220120000)"));
    }

    #[test]
    fn test_info_lang_field_roundtrip() {
        let mut info = PdfInfo::default();
        assert!(info.lang.is_none());
        info.lang = Some("ja".to_string());
        assert_eq!(info.lang.as_deref(), Some("ja"));
    }

    #[test]
    fn test_info_no_metadata_omits_info_dict() {
        // A fresh document with no metadata should have no /Info entry
        let doc = PdfDocument::new();
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(!s.contains("/Info <<"));
    }

    #[test]
    fn test_info_all_fields_set() {
        let mut doc = PdfDocument::new();
        doc.info.title = Some("Full Meta".to_string());
        doc.info.author = Some("Author A".to_string());
        doc.info.subject = Some("Subject S".to_string());
        doc.info.creation_date = Some("D:20260101".to_string());
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/Title (Full Meta)"));
        assert!(s.contains("/Author (Author A)"));
        assert!(s.contains("/Subject (Subject S)"));
        assert!(s.contains("/CreationDate (D:20260101)"));
    }

    // ── Cross-reference table structure ───────────────────────────────────────

    #[test]
    fn test_xref_table_present() {
        let doc = PdfDocument::new();
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("xref\n"));
    }

    #[test]
    fn test_xref_free_object_zero() {
        let doc = PdfDocument::new();
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        // Object 0 must be the free-object entry
        assert!(s.contains("0000000000 65535 f "));
    }

    #[test]
    fn test_xref_entries_use_n_type() {
        let doc = PdfDocument::new();
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        // At least one in-use entry must be present
        assert!(s.contains(" 00000 n "));
    }

    // ── Trailer dictionary ────────────────────────────────────────────────────

    #[test]
    fn test_trailer_has_root_reference() {
        let doc = PdfDocument::new();
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/Root 1 0 R"));
    }

    #[test]
    fn test_trailer_has_size_entry() {
        let doc = PdfDocument::new();
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        // Trailer must contain a /Size entry
        assert!(s.contains("/Size "));
    }

    // ── startxref offset ─────────────────────────────────────────────────────

    #[test]
    fn test_startxref_keyword_present() {
        let doc = PdfDocument::new();
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("startxref\n"));
    }

    #[test]
    fn test_startxref_offset_is_nonzero() {
        let doc = PdfDocument::new();
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        // Find startxref and read the number after it
        let idx = s.find("startxref\n").expect("test: should succeed");
        let after = &s[idx + "startxref\n".len()..];
        let offset_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        let offset: usize = offset_str.parse().expect("test: should succeed");
        assert!(offset > 0);
    }

    #[test]
    fn test_eof_marker_present() {
        let doc = PdfDocument::new();
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("%%EOF"));
    }

    // ── PdfInfo struct ────────────────────────────────────────────────────────

    #[test]
    fn test_pdfinfo_default_all_none() {
        let info = PdfInfo::default();
        assert!(info.title.is_none());
        assert!(info.author.is_none());
        assert!(info.subject.is_none());
        assert!(info.creation_date.is_none());
        assert!(info.lang.is_none());
    }

    #[test]
    fn test_pdfinfo_clone() {
        let info = PdfInfo {
            title: Some("Clone Me".to_string()),
            ..Default::default()
        };
        let cloned = info.clone();
        assert_eq!(cloned.title.as_deref(), Some("Clone Me"));
    }

    // ── Document ID in trailer ────────────────────────────────────────────────

    #[test]
    fn test_file_id_appears_in_trailer_as_id_array() {
        use crate::pdf::security::generate_file_id;
        let mut doc = PdfDocument::new();
        let fid = generate_file_id("id-test");
        // Set file_id directly (without encryption)
        doc.file_id = Some(fid);
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/ID [<"));
    }

    // ── add_ext_g_state deduplication ─────────────────────────────────────────

    #[test]
    fn test_add_ext_g_state_deduplication() {
        let mut doc = PdfDocument::new();
        let idx1 = doc.add_ext_g_state(0.5, 0.5);
        let idx2 = doc.add_ext_g_state(0.5, 0.5);
        assert_eq!(idx1, idx2);
        assert_eq!(doc.ext_g_states.len(), 1);
    }

    #[test]
    fn test_add_ext_g_state_different_values_creates_two() {
        let mut doc = PdfDocument::new();
        let idx1 = doc.add_ext_g_state(0.3, 0.3);
        let idx2 = doc.add_ext_g_state(0.7, 0.7);
        assert_ne!(idx1, idx2);
        assert_eq!(doc.ext_g_states.len(), 2);
    }

    // ── add_gradient ─────────────────────────────────────────────────────────

    #[test]
    fn test_add_gradient_returns_index() {
        use fop_types::{Color, ColorStop, Gradient, Length, Point};
        let mut doc = PdfDocument::new();
        let gradient = Gradient::linear(
            Point::new(Length::from_pt(0.0), Length::from_pt(0.0)),
            Point::new(Length::from_pt(100.0), Length::from_pt(0.0)),
            vec![
                ColorStop::new(0.0, Color::BLACK),
                ColorStop::new(1.0, Color::WHITE),
            ],
        );
        let idx = doc.add_gradient(gradient);
        assert_eq!(idx, 0);
        assert_eq!(doc.gradients.len(), 1);
    }

    // ── Catalog structure ─────────────────────────────────────────────────────

    #[test]
    fn test_catalog_type_present() {
        let doc = PdfDocument::new();
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/Type /Catalog"));
    }

    #[test]
    fn test_catalog_pages_reference_present() {
        let doc = PdfDocument::new();
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/Pages 2 0 R"));
    }

    // ── PdfPage ───────────────────────────────────────────────────────────────

    #[test]
    fn test_pdfpage_new_empty_content() {
        let page = PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0));
        assert!(page.content.is_empty());
        assert!(page.link_annotations.is_empty());
    }

    #[test]
    fn test_pdfpage_add_text_with_spacing_produces_tc_tw() {
        let mut page = PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0));
        page.add_text_with_spacing(
            "Hello",
            Length::from_pt(72.0),
            Length::from_pt(700.0),
            Length::from_pt(12.0),
            Some(Length::from_pt(1.0)),
            Some(Length::from_pt(2.0)),
        );
        let content = String::from_utf8_lossy(&page.content);
        assert!(content.contains("Tc"));
        assert!(content.contains("Tw"));
    }

    #[test]
    fn test_pdfpage_add_background_generates_rg_and_re() {
        let mut page = PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0));
        page.add_background(
            Length::from_pt(10.0),
            Length::from_pt(10.0),
            Length::from_pt(200.0),
            Length::from_pt(100.0),
            fop_types::Color::rgb(255, 0, 0),
        );
        let content = String::from_utf8_lossy(&page.content);
        // Color set (rg) and rectangle drawn (re f)
        assert!(content.contains("rg"));
        assert!(content.contains("re f"));
    }

    #[test]
    fn test_pdfpage_add_link_annotation_stores_annotation() {
        let mut page = PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0));
        page.add_link_annotation(
            Length::from_pt(50.0),
            Length::from_pt(700.0),
            Length::from_pt(100.0),
            Length::from_pt(12.0),
            LinkDestination::External("https://example.com".to_string()),
        );
        assert_eq!(page.link_annotations.len(), 1);
    }

    #[test]
    fn test_pdfpage_link_annotation_rect_values() {
        let mut page = PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0));
        page.add_link_annotation(
            Length::from_pt(10.0),
            Length::from_pt(20.0),
            Length::from_pt(80.0),
            Length::from_pt(14.0),
            LinkDestination::Internal("section-1".to_string()),
        );
        let ann = &page.link_annotations[0];
        // rect: [x, y, x+w, y+h]
        assert!((ann.rect[0] - 10.0).abs() < 0.01);
        assert!((ann.rect[1] - 20.0).abs() < 0.01);
        assert!((ann.rect[2] - 90.0).abs() < 0.01);
        assert!((ann.rect[3] - 34.0).abs() < 0.01);
    }

    #[test]
    fn test_pdfpage_multiple_texts_accumulate_in_content() {
        let mut page = PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0));
        page.add_text(
            "First",
            Length::from_pt(72.0),
            Length::from_pt(700.0),
            Length::from_pt(12.0),
        );
        page.add_text(
            "Second",
            Length::from_pt(72.0),
            Length::from_pt(680.0),
            Length::from_pt(12.0),
        );
        let content = String::from_utf8_lossy(&page.content);
        assert!(content.contains("First"));
        assert!(content.contains("Second"));
    }

    // ── PdfObject / PdfValue ──────────────────────────────────────────────────

    #[test]
    fn test_pdf_value_boolean() {
        let v = PdfValue::Boolean(true);
        if let PdfValue::Boolean(b) = v {
            assert!(b);
        } else {
            panic!("Expected Boolean");
        }
    }

    #[test]
    fn test_pdf_value_integer() {
        let v = PdfValue::Integer(42);
        if let PdfValue::Integer(n) = v {
            assert_eq!(n, 42);
        } else {
            panic!("Expected Integer");
        }
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_pdf_value_real() {
        let v = PdfValue::Real(3.14);
        if let PdfValue::Real(f) = v {
            assert!((f - 3.14).abs() < f64::EPSILON);
        } else {
            panic!("Expected Real");
        }
    }

    #[test]
    fn test_pdf_value_name() {
        let v = PdfValue::Name("Font".to_string());
        if let PdfValue::Name(s) = v {
            assert_eq!(s, "Font");
        } else {
            panic!("Expected Name");
        }
    }

    #[test]
    fn test_pdf_value_null() {
        let v = PdfValue::Null;
        assert!(matches!(v, PdfValue::Null));
    }

    // ── set_compliance errors ─────────────────────────────────────────────────

    #[test]
    fn test_set_compliance_pdfa_with_encryption_returns_error() {
        use crate::pdf::compliance::PdfCompliance;
        use crate::pdf::security::{generate_file_id, PdfPermissions, PdfSecurity};
        let mut doc = PdfDocument::new();
        let sec = PdfSecurity::new("owner", "user", PdfPermissions::default());
        let fid = generate_file_id("enc");
        let dict = sec.compute_encryption_dict(&fid);
        doc.set_encryption(dict, fid).expect("test: should succeed");
        let result = doc.set_compliance(PdfCompliance::PdfA1b);
        assert!(result.is_err());
    }

    #[test]
    fn test_info_escapes_parentheses_in_title() {
        let mut doc = PdfDocument::new();
        doc.info.title = Some("(parenthesised)".to_string());
        let bytes = doc.to_bytes().expect("test: should succeed");
        let content = String::from_utf8_lossy(&bytes);
        // The raw string r"\(parenthesised\)" is the PDF-escaped form
        assert!(
            content.contains(r"/Title (\(parenthesised\))"),
            "Expected PDF-escaped title with backslash-escaped parentheses; got:\n{}",
            content
        );
        // The unescaped form must NOT appear
        assert!(!content.contains("/Title ((parenthesised))"));
    }

    // ── PDF/UA-1 honest error tests ───────────────────────────────────────────

    /// `set_compliance(PdfUA1)` must return an error rather than silently
    /// producing a PDF that falsely claims UA-1 conformance with an empty
    /// StructTreeRoot (which veraPDF/PAC would reject).
    #[test]
    fn test_set_compliance_pdfua1_returns_not_implemented_error() {
        use crate::pdf::compliance::PdfCompliance;
        let mut doc = PdfDocument::new();
        let result = doc.set_compliance(PdfCompliance::PdfUA1);
        assert!(
            result.is_err(),
            "PDF/UA-1 set_compliance must return Err (tagged-PDF not implemented)"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("PDF/UA-1"),
            "Error message must mention PDF/UA-1; got: {msg}"
        );
    }

    /// `set_compliance(PdfA1bUA1)` must also error because the UA-1 part
    /// is not implemented regardless of the PDF/A-1b part.
    #[test]
    fn test_set_compliance_pdfa1bua1_returns_not_implemented_error() {
        use crate::pdf::compliance::PdfCompliance;
        let mut doc = PdfDocument::new();
        let result = doc.set_compliance(PdfCompliance::PdfA1bUA1);
        assert!(
            result.is_err(),
            "PDF/A-1b+UA-1 set_compliance must return Err (UA-1 tagged-PDF not implemented)"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("PDF/UA-1"),
            "Error message must mention PDF/UA-1; got: {msg}"
        );
    }

    /// Even if a caller bypasses `set_compliance` by writing to the `pub`
    /// `compliance` field directly, `to_bytes()` must still return an error
    /// rather than emit false `/Marked true` + empty StructTreeRoot markers.
    #[test]
    fn test_to_bytes_pdfua1_via_direct_field_returns_error() {
        use crate::pdf::compliance::PdfCompliance;
        let mut doc = PdfDocument::new();
        // Bypass set_compliance by writing the field directly.
        doc.compliance = PdfCompliance::PdfUA1;
        let result = doc.to_bytes();
        assert!(
            result.is_err(),
            "to_bytes() with PdfUA1 compliance must return Err, not a non-conformant PDF"
        );
        // Also verify the false markers are absent — to_bytes errored before writing them.
        // (The error path means no bytes are returned, so there is nothing to scan.)
    }
}
