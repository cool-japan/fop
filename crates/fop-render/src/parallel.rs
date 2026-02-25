//! Parallel rendering support for multi-page documents
//!
//! Enables parallel processing of independent pages using std::thread::scope.
//! Each page is rendered independently, making this embarrassingly parallel.

use crate::{PdfDocument, PdfRenderer, Result};
use fop_layout::AreaTree;

/// Parallel PDF renderer
///
/// Renders pages in parallel using multiple threads. Each page is independent,
/// so this provides linear speedup with the number of cores.
pub struct ParallelRenderer {
    /// Number of threads to use (0 = auto-detect)
    num_threads: usize,
}

impl ParallelRenderer {
    /// Create a new parallel renderer
    ///
    /// # Arguments
    /// * `num_threads` - Number of threads to use (0 = auto-detect)
    pub fn new(num_threads: usize) -> Self {
        Self { num_threads }
    }

    /// Render an area tree to PDF using parallel processing
    ///
    /// Pages are rendered in parallel and then combined into a single document.
    /// This is significantly faster for multi-page documents on multi-core systems.
    ///
    /// Implementation strategy:
    /// 1. Pre-collect shared resources (images, opacity states) sequentially
    /// 2. Render individual page content streams in parallel
    /// 3. Combine results into final document in correct order
    pub fn render(&self, area_tree: &AreaTree) -> Result<PdfDocument> {
        use fop_layout::AreaType;
        use std::collections::HashMap;

        // Phase 1: Create document and collect shared resources (must be sequential)
        let mut doc = PdfDocument::new();
        doc.info.title = Some("FOP Generated PDF".to_string());

        let mut image_map = HashMap::new();
        let renderer = PdfRenderer::new();
        renderer.collect_images_public(area_tree, &mut doc, &mut image_map)?;

        let mut opacity_map = HashMap::new();
        renderer.collect_opacity_states_public(area_tree, &mut doc, &mut opacity_map);

        // Build font cache (empty for parallel renderer – no font config attached)
        let font_cache: HashMap<String, usize> = HashMap::new();

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
