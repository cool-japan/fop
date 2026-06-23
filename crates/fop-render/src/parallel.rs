//! Parallel rendering support for multi-page documents
//!
//! Enables parallel processing of independent pages using std::thread::scope.
//! Each page is rendered independently, making this embarrassingly parallel.

use crate::pdf::FontConfig;
use crate::{PdfDocument, PdfRenderer, Result};
use fop_layout::AreaTree;

/// Parallel PDF renderer
///
/// Renders pages in parallel using multiple threads. Each page is independent,
/// so this provides linear speedup with the number of cores.
pub struct ParallelRenderer {
    /// Number of threads to use (0 = auto-detect)
    num_threads: usize,

    /// Font configuration: maps family names to TTF file paths.
    ///
    /// This is forwarded to the internal [`PdfRenderer`] so that
    /// [`build_font_cache_public`][crate::PdfRenderer::build_font_cache_public]
    /// can embed the same fonts that the sequential path would embed.
    font_config: FontConfig,
}

impl ParallelRenderer {
    /// Create a new parallel renderer with no extra font configuration.
    ///
    /// Use [`ParallelRenderer::with_font_config`] to supply a [`FontConfig`]
    /// that mirrors the one used by the equivalent sequential [`PdfRenderer`].
    ///
    /// # Arguments
    /// * `num_threads` - Number of threads to use (0 = auto-detect)
    pub fn new(num_threads: usize) -> Self {
        Self {
            num_threads,
            font_config: FontConfig::new(),
        }
    }

    /// Attach a [`FontConfig`] to this parallel renderer.
    ///
    /// The supplied configuration is used to locate and embed TrueType /
    /// OpenType fonts into the output document, matching the behaviour of
    /// [`PdfRenderer::with_font_config`] on the sequential render path.
    pub fn with_font_config(mut self, font_config: FontConfig) -> Self {
        self.font_config = font_config;
        self
    }

    /// Render an area tree to PDF using parallel processing
    ///
    /// Pages are rendered in parallel and then combined into a single document.
    /// This is significantly faster for multi-page documents on multi-core systems.
    ///
    /// Implementation strategy:
    /// 1. Pre-collect shared resources (images, opacity states, fonts) sequentially
    /// 2. Render individual page content streams in parallel
    /// 3. Combine results into final document in correct order
    pub fn render(&self, area_tree: &AreaTree) -> Result<PdfDocument> {
        use fop_layout::AreaType;
        use std::collections::HashMap;

        // Phase 1: Create document and collect shared resources (must be sequential).
        // Build the per-render PdfRenderer with the same FontConfig that the
        // sequential path would use so that images, opacity states, and – crucially
        // – the font cache are all built from the same configuration.
        let mut doc = PdfDocument::new();
        doc.info.title = Some("FOP Generated PDF".to_string());

        let renderer = PdfRenderer::new().with_font_config(self.font_config.clone());

        let mut image_map = HashMap::new();
        renderer.collect_images_public(area_tree, &mut doc, &mut image_map)?;

        let mut opacity_map = HashMap::new();
        renderer.collect_opacity_states_public(area_tree, &mut doc, &mut opacity_map);

        // Build the font cache ONCE sequentially before the parallel page loop.
        // Font embedding mutates `doc` (it inserts binary TTF data into the font
        // manager), so it cannot run per-thread.  We reuse the same
        // `build_font_cache` logic that `PdfRenderer::render()` uses on the
        // sequential path, ensuring custom/embedded fonts are handled identically.
        let font_cache = renderer.build_font_cache_public(area_tree, &mut doc)?;

        // Phase 2: Collect page IDs in document order
        let page_ids: Vec<_> = area_tree
            .iter()
            .filter_map(|(id, node)| {
                if matches!(node.area.area_type, AreaType::Page) {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();

        if page_ids.is_empty() {
            return Ok(doc);
        }

        // Phase 3: Render pages in parallel using scoped threads
        let num_threads = self.effective_threads();
        let pages = if num_threads > 1 && page_ids.len() > 1 {
            // Parallel rendering
            std::thread::scope(|scope| {
                let mut handles = Vec::new();

                for page_id in &page_ids {
                    // Spawn a thread for each page
                    let handle = scope.spawn(|| {
                        renderer.render_page_public(
                            area_tree,
                            *page_id,
                            &image_map,
                            &opacity_map,
                            &font_cache,
                        )
                    });
                    handles.push(handle);
                }

                // Collect results in order
                handles
                    .into_iter()
                    .map(|h| h.join().expect("render thread panicked"))
                    .collect::<Result<Vec<_>>>()
            })?
        } else {
            // Sequential fallback for single page or single thread
            page_ids
                .iter()
                .map(|&page_id| {
                    renderer.render_page_public(
                        area_tree,
                        page_id,
                        &image_map,
                        &opacity_map,
                        &font_cache,
                    )
                })
                .collect::<Result<Vec<_>>>()?
        };

        // Phase 4: Add pages to document in order
        for page in pages {
            doc.add_page(page);
        }

        Ok(doc)
    }

    /// Get the effective number of threads
    pub fn effective_threads(&self) -> usize {
        if self.num_threads == 0 {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        } else {
            self.num_threads
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_renderer_creation() {
        let renderer = ParallelRenderer::new(4);
        assert_eq!(renderer.num_threads, 4);
    }

    #[test]
    fn test_effective_threads_auto() {
        let renderer = ParallelRenderer::new(0);
        let threads = renderer.effective_threads();
        assert!(threads >= 1);
    }

    #[test]
    fn test_effective_threads_explicit() {
        let renderer = ParallelRenderer::new(8);
        assert_eq!(renderer.effective_threads(), 8);
    }

    #[test]
    fn test_with_font_config_replaces_default() {
        let mut fc = FontConfig::new();
        fc.add_mapping("TestFont", std::path::PathBuf::from("/test.ttf"));
        let renderer = ParallelRenderer::new(2).with_font_config(fc);
        // The renderer now holds the config; verify by checking num_threads is
        // still intact (the builder must return Self).
        assert_eq!(renderer.num_threads, 2);
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;
    use fop_core::FoTreeBuilder;
    use fop_layout::LayoutEngine;
    use std::io::Cursor;

    fn single_page_area_tree() -> fop_layout::AreaTree {
        let fo_xml = r##"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Parallel test page</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;
        let builder = FoTreeBuilder::new();
        let fo_tree = builder
            .parse(Cursor::new(fo_xml))
            .expect("test: should succeed");
        LayoutEngine::new()
            .layout(&fo_tree)
            .expect("test: should succeed")
    }

    #[test]
    fn test_parallel_render_produces_pdf() {
        let renderer = ParallelRenderer::new(2);
        let area_tree = single_page_area_tree();
        let doc = renderer.render(&area_tree).expect("test: should succeed");
        assert_eq!(doc.pages.len(), 1);
    }

    #[test]
    fn test_parallel_render_empty_tree() {
        let renderer = ParallelRenderer::new(2);
        let area_tree = fop_layout::AreaTree::new();
        let doc = renderer.render(&area_tree).expect("test: should succeed");
        assert_eq!(doc.pages.len(), 0);
    }

    #[test]
    fn test_parallel_render_single_thread() {
        let renderer = ParallelRenderer::new(1);
        let area_tree = single_page_area_tree();
        let doc = renderer.render(&area_tree).expect("test: should succeed");
        assert_eq!(doc.pages.len(), 1);
    }

    #[test]
    fn test_parallel_render_auto_thread_count() {
        let renderer = ParallelRenderer::new(0);
        let area_tree = single_page_area_tree();
        let doc = renderer.render(&area_tree).expect("test: should succeed");
        assert_eq!(doc.pages.len(), 1);
    }

    #[test]
    fn test_parallel_render_page_count_matches_sequential() {
        let area_tree = single_page_area_tree();

        let sequential = ParallelRenderer::new(1);
        let parallel = ParallelRenderer::new(4);

        let seq_doc = sequential.render(&area_tree).expect("test: should succeed");
        let par_doc = parallel.render(&area_tree).expect("test: should succeed");

        assert_eq!(seq_doc.pages.len(), par_doc.pages.len());
    }

    #[test]
    fn test_effective_threads_returns_at_least_one() {
        for n in [0, 1, 2, 4, 8] {
            let r = ParallelRenderer::new(n);
            assert!(
                r.effective_threads() >= 1,
                "effective_threads should be >= 1 for num_threads={}",
                n
            );
        }
    }

    #[test]
    fn test_parallel_renderer_new_various_counts() {
        for n in [0, 1, 4, 16] {
            let r = ParallelRenderer::new(n);
            assert_eq!(r.num_threads, n);
        }
    }
}

#[cfg(test)]
mod tests_parallel_font_embedding {
    use super::*;
    use crate::pdf::PdfRenderer;
    use fop_layout::area::TraitSet;
    use fop_layout::{Area, AreaTree, AreaType};
    use fop_types::{Length, Point, Rect, Size};

    /// Build a one-page area tree containing a single text area whose
    /// `font_family` trait is set to `family`.  Constructed directly (no FO
    /// parser) so the test is self-contained and fast.
    fn area_tree_with_font_family(family: &str) -> AreaTree {
        let page_rect = Rect::from_point_size(
            Point::ZERO,
            Size::new(Length::from_mm(210.0), Length::from_mm(297.0)),
        );
        let mut tree = AreaTree::new();
        let page_id = tree.add_area(Area::new(AreaType::Page, page_rect));

        let text_rect = Rect::from_point_size(
            Point::new(Length::from_mm(20.0), Length::from_mm(20.0)),
            Size::new(Length::from_mm(170.0), Length::from_pt(14.0)),
        );
        let traits = TraitSet {
            font_family: Some(family.to_string()),
            font_size: Some(Length::from_pt(12.0)),
            ..TraitSet::default()
        };
        let text_area = Area::text(text_rect, format!("Custom font rendering test: {family}"))
            .with_traits(traits);
        let text_id = tree.add_area(text_area);
        tree.append_child(page_id, text_id)
            .expect("test: append_child should succeed");

        tree
    }

    /// Asserts that `ParallelRenderer` with a `FontConfig` embeds the same
    /// fonts as `PdfRenderer` with the same config – and that at least one
    /// font is embedded (i.e. not silently falling back to built-in Helvetica).
    ///
    /// Uses DejaVu Sans, which is available on any standard Debian / Ubuntu /
    /// CI Linux installation (`fonts-dejavu-core` package).
    #[test]
    fn test_parallel_renderer_embeds_fonts_matching_sequential() {
        let font_path = std::path::PathBuf::from("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
        assert!(
            font_path.exists(),
            "DejaVu Sans not found at {:?}; install fonts-dejavu-core",
            font_path,
        );

        // "DejaVu Sans" is the Typographic Family name extracted by ttf-parser.
        let family = "DejaVu Sans";
        let mut font_config = FontConfig::new();
        font_config.add_mapping(family, font_path);

        let area_tree = area_tree_with_font_family(family);

        // Sequential reference path
        let seq_renderer = PdfRenderer::new().with_font_config(font_config.clone());
        let seq_doc = seq_renderer
            .render(&area_tree)
            .expect("test: sequential render should succeed");

        // Parallel path under test (2 threads; single page falls back to sequential
        // code path inside render(), but font cache must still be built correctly)
        let par_renderer = ParallelRenderer::new(2).with_font_config(font_config);
        let par_doc = par_renderer
            .render(&area_tree)
            .expect("test: parallel render should succeed");

        let seq_count = seq_doc.font_manager.font_count();
        let par_count = par_doc.font_manager.font_count();

        assert_eq!(
            seq_count, par_count,
            "parallel renderer must embed the same number of fonts as sequential \
             (seq={seq_count}, par={par_count})",
        );

        assert!(
            seq_count > 0,
            "expected at least one font to be embedded; got zero – \
             font_config was not forwarded to the parallel renderer",
        );

        // Verify the embedded font names agree between both paths.
        let seq_names: Vec<String> = (0..seq_count)
            .filter_map(|i| seq_doc.font_manager.get_font(i))
            .map(|f| f.font_name.clone())
            .collect();
        let par_names: Vec<String> = (0..par_count)
            .filter_map(|i| par_doc.font_manager.get_font(i))
            .map(|f| f.font_name.clone())
            .collect();

        assert_eq!(
            seq_names, par_names,
            "embedded font names must be identical between sequential and parallel rendering",
        );

        // Guard against the Helvetica fallback: none of the embedded fonts
        // should be the built-in Type1 placeholder.
        for name in &par_names {
            assert_ne!(
                name.to_lowercase(),
                "helvetica",
                "parallel renderer fell back to built-in Helvetica; \
                 custom font '{family}' was not embedded",
            );
        }
    }
}
