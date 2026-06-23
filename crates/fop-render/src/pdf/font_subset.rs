//! Real TrueType/OpenType font subsetting for PDF embedding.
//!
//! This module turns a full font into a strictly smaller, still-valid font that
//! contains only the glyphs actually used in the document (plus `.notdef` and
//! any composite-glyph dependencies). It is the engine behind the embedded
//! subset that [`crate::pdf::font::FontManager`] writes into the PDF.
//!
//! ## Why a dedicated crate instead of hand-rolled table surgery
//!
//! Correct subsetting requires rebuilding `glyf`/`loca`, recomputing every table
//! checksum *and* the `head.checkSumAdjustment`, fixing `head.indexToLocFormat`,
//! and chasing composite-glyph component references — any mistake silently
//! corrupts the font. We therefore delegate the binary surgery to the pure-Rust
//! [`subsetter`] crate (the same one Typst uses), which is well tested across
//! millions of real-world fonts.
//!
//! ## Glyph-ID remapping and PDF consistency
//!
//! `subsetter` 0.2.x **renumbers** the retained glyphs into a new, contiguous
//! range: the glyphs are assigned IDs `0, 1, 2, …` in the order they are added
//! to the [`GlyphRemapper`]. `.notdef` is always glyph 0. This matters because
//! the PDF embeds the font as a `CIDFontType2` descendant of a `Type0`
//! `Identity-H` font, where a `CIDToGIDMap` stream maps each CID to a glyph ID
//! *in the embedded bytes*. If the glyph IDs change, that map (and the
//! `char -> glyph` table it is generated from, and the `cmap`) must refer to the
//! **new** glyph space, or the renderer will draw the wrong glyphs.
//!
//! Composite glyphs add a subtlety: a compound glyph stores the IDs of its
//! component glyphs inside its outline data. `subsetter` discovers those
//! components automatically (it clones the supplied remapper and runs an
//! internal `glyf` closure pass, see its `subset` implementation) and rewrites
//! the component references to the new IDs. Because the closure pass only ever
//! *appends* freshly-discovered components, the new ID of every glyph we
//! explicitly requested is exactly [`GlyphRemapper::get`] — so we can read the
//! old → new mapping straight back out of the remapper we built.
//!
//! [`subsetter`]: https://docs.rs/subsetter
//! [`GlyphRemapper`]: subsetter::GlyphRemapper
//! [`GlyphRemapper::get`]: subsetter::GlyphRemapper::get

use fop_types::{FopError, Result};
use std::collections::{BTreeSet, HashMap};
use subsetter::GlyphRemapper;

/// The product of subsetting a font.
pub(crate) struct SubsetFont {
    /// The subsetted font bytes. For a small `used_glyphs` set this is orders of
    /// magnitude smaller than the original font.
    pub(crate) data: Vec<u8>,

    /// Mapping from each *originally-requested* glyph ID to its new (remapped)
    /// glyph ID in [`SubsetFont::data`].
    ///
    /// Only the glyphs passed in `used_glyphs` appear here (`.notdef` included);
    /// composite components that the subsetter pulled in internally are *not*
    /// listed, because no CID ever addresses them directly — they are reached
    /// only through the composite glyphs that reference them.
    pub(crate) gid_map: HashMap<u16, u16>,
}

/// Build a real subset of a TrueType/OpenType font.
///
/// The returned font contains `.notdef`, every glyph in `used_glyphs`, and
/// (transitively) every component glyph any of those composite glyphs reference.
/// All retained glyphs are renumbered into a new contiguous range; the returned
/// [`SubsetFont::gid_map`] records the original → new glyph-ID mapping for the
/// requested glyphs so the caller can keep the PDF's `CIDToGIDMap`, `cmap`, and
/// `W` width array pointing at the right glyphs.
///
/// `font_data` must be the same single-face font that the rest of the embedding
/// pipeline parses (face index 0).
///
/// # Errors
///
/// Returns [`FopError::Generic`] if `subsetter` cannot process the font (e.g. an
/// unknown font kind, a malformed table, or an unimplemented feature). The error
/// is surfaced rather than silently falling back to the full font, so that a
/// genuine subsetting failure is never mistaken for a successful subset.
pub(crate) fn subset_font(
    font_data: &[u8],
    used_glyphs: &BTreeSet<ttf_parser::GlyphId>,
) -> Result<SubsetFont> {
    // `GlyphRemapper::new()` seeds the subset with `.notdef` (old 0 -> new 0).
    let mut remapper = GlyphRemapper::new();

    // Register every requested glyph. Iterating the `BTreeSet` yields ascending
    // glyph IDs, which makes the remapping order — and therefore the produced
    // bytes — deterministic. The subsetter appends any composite components it
    // discovers *after* these, so it never disturbs the IDs assigned here.
    for glyph in used_glyphs.iter() {
        remapper.remap(glyph.0);
    }

    // `index` is 0: the embedding pipeline always parses face 0 and never embeds
    // straight from a `.ttc`/`.otc` collection.
    let data = subsetter::subset(font_data, 0, &remapper)
        .map_err(|e| FopError::Generic(format!("font subsetting failed: {e}")))?;

    // Read back the new ID of each requested glyph. `get` is authoritative here
    // precisely because the internal closure pass only appends new components.
    let mut gid_map = HashMap::with_capacity(used_glyphs.len() + 1);
    gid_map.insert(0u16, 0u16); // `.notdef`, guaranteed present.
    for glyph in used_glyphs.iter() {
        if let Some(new_gid) = remapper.get(glyph.0) {
            gid_map.insert(glyph.0, new_gid);
        }
    }

    Ok(SubsetFont { data, gid_map })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ttf_parser::{Face, GlyphId, OutlineBuilder};

    /// A standard DejaVu Sans install path on Debian / Ubuntu / CI Linux
    /// (`fonts-dejavu-core`). The repository's other font tests rely on the same
    /// file (see `crate::parallel` tests).
    const DEJAVU_SANS: &str = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf";

    /// Counts the number of outline drawing operations for a glyph. A non-zero
    /// count means the glyph has a real, drawable outline; an empty glyph (such
    /// as `.notdef` in DejaVu, or a space) yields zero.
    #[derive(Default)]
    struct OutlineCounter {
        ops: usize,
    }

    impl OutlineBuilder for OutlineCounter {
        fn move_to(&mut self, _x: f32, _y: f32) {
            self.ops += 1;
        }
        fn line_to(&mut self, _x: f32, _y: f32) {
            self.ops += 1;
        }
        fn quad_to(&mut self, _x1: f32, _y1: f32, _x: f32, _y: f32) {
            self.ops += 1;
        }
        fn curve_to(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _x: f32, _y: f32) {
            self.ops += 1;
        }
        fn close(&mut self) {}
    }

    fn load_dejavu() -> Vec<u8> {
        assert!(
            std::path::Path::new(DEJAVU_SANS).exists(),
            "DejaVu Sans not found at {DEJAVU_SANS:?}; install fonts-dejavu-core",
        );
        std::fs::read(DEJAVU_SANS).expect("test: read DejaVu Sans")
    }

    /// (a) The subset of a tiny glyph set is strictly smaller than the original.
    #[test]
    fn subset_is_strictly_smaller_than_original() {
        let original = load_dejavu();
        let face = Face::parse(&original, 0).expect("test: parse DejaVu");

        let mut used = BTreeSet::new();
        used.insert(GlyphId(0));
        for c in ['H', 'e', 'l', 'o'] {
            used.insert(face.glyph_index(c).expect("test: glyph present"));
        }

        let subset = subset_font(&original, &used).expect("test: subset");

        assert!(
            subset.data.len() < original.len(),
            "subset ({} bytes) must be smaller than the original ({} bytes)",
            subset.data.len(),
            original.len(),
        );
        // A handful of glyphs out of a ~750 KB font should shrink dramatically;
        // guard against an accidental near-passthrough regression.
        assert!(
            subset.data.len() * 10 < original.len(),
            "subset ({} bytes) is suspiciously close to the original ({} bytes) — \
             did subsetting silently degrade to a passthrough?",
            subset.data.len(),
            original.len(),
        );
    }

    /// (b) The subset still parses and carries real outlines for the used glyphs,
    /// and composite glyphs keep their components (outline op-count is preserved).
    #[test]
    fn subset_parses_and_preserves_outlines_including_composites() {
        let original = load_dejavu();
        let face_full = Face::parse(&original, 0).expect("test: parse DejaVu");

        // 'H','i' are simple glyphs. The accented letters are composite glyphs in
        // DejaVu Sans (a base letter plus a combining accent component), which
        // exercises the subsetter's composite-closure + component-remap path.
        let chars = ['H', 'i', 'é', 'ñ', 'ü', 'à'];
        let mut used = BTreeSet::new();
        used.insert(GlyphId(0));
        let mut requested: Vec<(char, GlyphId)> = Vec::new();
        for &c in &chars {
            let gid = face_full.glyph_index(c).expect("test: glyph present");
            used.insert(gid);
            requested.push((c, gid));
        }

        let subset = subset_font(&original, &used).expect("test: subset");

        // Still a valid, parseable font (requirement (b)).
        let face_sub = Face::parse(&subset.data, 0).expect("test: subset must parse");

        for (c, old_gid) in requested {
            let new_gid = *subset
                .gid_map
                .get(&old_gid.0)
                .unwrap_or_else(|| panic!("test: no remap for {c:?}"));

            // Outline in the subset (new glyph space).
            let mut sub_outline = OutlineCounter::default();
            let sub_bbox = face_sub.outline_glyph(GlyphId(new_gid), &mut sub_outline);

            // Outline in the original (old glyph space).
            let mut full_outline = OutlineCounter::default();
            let full_bbox = face_full.outline_glyph(old_gid, &mut full_outline);

            assert!(
                sub_bbox.is_some() && sub_outline.ops > 0,
                "subset glyph for {c:?} (new gid {new_gid}) has no drawable outline",
            );
            // If a composite component had been dropped, the subset glyph would
            // have fewer drawing ops (a missing accent) or none at all. Requiring
            // an exact match proves the closure pass and component remap worked.
            assert_eq!(
                sub_outline.ops, full_outline.ops,
                "outline op-count mismatch for {c:?}: subset={} original={} \
                 (composite component lost during subsetting?)",
                sub_outline.ops, full_outline.ops,
            );
            assert_eq!(
                sub_bbox, full_bbox,
                "outline bounding box mismatch for {c:?} between subset and original",
            );
        }
    }

    /// `.notdef` is always retained and mapped to new glyph 0.
    #[test]
    fn notdef_is_always_glyph_zero() {
        let original = load_dejavu();
        let face = Face::parse(&original, 0).expect("test: parse DejaVu");
        let mut used = BTreeSet::new();
        used.insert(GlyphId(0));
        used.insert(face.glyph_index('A').expect("test: glyph present"));

        let subset = subset_font(&original, &used).expect("test: subset");
        assert_eq!(
            subset.gid_map.get(&0),
            Some(&0),
            ".notdef must remain glyph 0"
        );
    }
}
