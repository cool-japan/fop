//! Glyph outline rendering helpers
//!
//! Bridges ttf_parser's outline callbacks to tiny_skia path construction,
//! and resolves Standard-14 font substitutes from the local system.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// OutlinePathBuilder
// ---------------------------------------------------------------------------

/// Adapts ttf_parser's OutlineBuilder callbacks to a tiny_skia PathBuilder.
/// Paths are emitted in font design units; the caller applies a Transform
/// that scales from design units to screen pixels.
pub struct OutlinePathBuilder {
    pub builder: tiny_skia::PathBuilder,
}

impl ttf_parser::OutlineBuilder for OutlinePathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.builder.move_to(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.builder.line_to(x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.builder.quad_to(x1, y1, x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.builder.cubic_to(x1, y1, x2, y2, x, y);
    }
    fn close(&mut self) {
        self.builder.close();
    }
}

// ---------------------------------------------------------------------------
// Standard-14 font substitute resolution
// ---------------------------------------------------------------------------

/// Ordered substitute family names for each Standard-14 PDF font name.
fn standard14_substitutes(base_font: &str) -> &'static [&'static str] {
    match base_font {
        "Helvetica" | "Helvetica-Oblique" => &[
            "Arial",
            "Liberation Sans",
            "DejaVu Sans",
            "Nimbus Sans",
            "FreeSans",
        ],
        "Helvetica-Bold" | "Helvetica-BoldOblique" => &[
            "Arial Bold",
            "Liberation Sans Bold",
            "DejaVu Sans Bold",
            "Nimbus Sans Bold",
        ],
        "Times-Roman" | "Times-Italic" => &[
            "Times New Roman",
            "Liberation Serif",
            "DejaVu Serif",
            "Nimbus Roman",
            "FreeSerif",
        ],
        "Times-Bold" | "Times-BoldItalic" => &[
            "Times New Roman Bold",
            "Liberation Serif Bold",
            "DejaVu Serif Bold",
        ],
        "Courier" | "Courier-Oblique" => &[
            "Courier New",
            "Liberation Mono",
            "DejaVu Sans Mono",
            "Nimbus Mono",
            "FreeMono",
        ],
        "Courier-Bold" | "Courier-BoldOblique" => &[
            "Courier New Bold",
            "Liberation Mono Bold",
            "DejaVu Sans Mono Bold",
        ],
        "Symbol" => &["Symbol", "OpenSymbol", "Standard Symbols PS"],
        "ZapfDingbats" => &["Dingbats", "ZapfDingbats", "D050000L"],
        _ => &[],
    }
}

/// System font directories to scan, per platform.
fn system_font_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    #[cfg(target_os = "macos")]
    {
        dirs.push(PathBuf::from("/Library/Fonts"));
        dirs.push(PathBuf::from("/System/Library/Fonts"));
        if let Some(home) = std::env::var_os("HOME") {
            dirs.push(PathBuf::from(home).join("Library/Fonts"));
        }
    }

    #[cfg(target_os = "windows")]
    {
        dirs.push(PathBuf::from(r"C:\Windows\Fonts"));
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            dirs.push(
                PathBuf::from(local_app_data)
                    .join("Microsoft")
                    .join("Windows")
                    .join("Fonts"),
            );
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        // Linux / other Unix
        dirs.push(PathBuf::from("/usr/share/fonts"));
        dirs.push(PathBuf::from("/usr/local/share/fonts"));
        if let Some(home) = std::env::var_os("HOME") {
            let home_path = PathBuf::from(home);
            dirs.push(home_path.join(".fonts"));
            dirs.push(home_path.join(".local/share/fonts"));
        }
    }

    dirs
}

/// Walk a directory recursively and collect `(family_name_lowercase, (path, ttc_index))` pairs.
fn scan_font_dir(dir: &std::path::Path, out: &mut HashMap<String, (PathBuf, u32)>) {
    let read_dir = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_font_dir(&path, out);
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !matches!(ext.as_str(), "ttf" | "otf" | "ttc") {
            continue;
        }
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(_) => continue,
        };
        // Try up to 4 TTC face indices
        for ttc_index in 0u32..4 {
            let face = match ttf_parser::Face::parse(&bytes, ttc_index) {
                Ok(f) => f,
                Err(_) => break,
            };
            // Extract family name (name ID 1)
            for name_record in face.names() {
                if name_record.name_id == ttf_parser::name_id::FAMILY {
                    if let Some(family) = name_record.to_string() {
                        let key = family.to_lowercase();
                        out.entry(key).or_insert_with(|| (path.clone(), ttc_index));
                    }
                }
            }
        }
    }
}

/// Global system font scan result: `family_name_lowercase` → `(path, ttc_index)`.
static SYSTEM_FONT_SCAN: OnceLock<HashMap<String, (PathBuf, u32)>> = OnceLock::new();

fn get_system_font_scan() -> &'static HashMap<String, (PathBuf, u32)> {
    SYSTEM_FONT_SCAN.get_or_init(|| {
        let mut map: HashMap<String, (PathBuf, u32)> = HashMap::new();
        for dir in system_font_dirs() {
            scan_font_dir(&dir, &mut map);
        }
        map
    })
}

type ResolvedFontMap = HashMap<String, Option<(PathBuf, u32)>>;

/// Per-base-font resolved result cache: `base_font` → `Option<(PathBuf, ttc_index)>`.
static RESOLVED_CACHE: OnceLock<std::sync::Mutex<ResolvedFontMap>> = OnceLock::new();

fn get_resolved_cache() -> &'static std::sync::Mutex<ResolvedFontMap> {
    RESOLVED_CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

/// Returns `(path, ttc_index)` for a substitute font for the given Standard-14
/// base font name, or `None` if no substitute can be found on this system.
/// Results are cached after the first call.
pub fn resolve_standard14_substitute(base_font: &str) -> Option<(PathBuf, u32)> {
    // Check per-base-font cache first
    {
        let cache = get_resolved_cache();
        if let Ok(guard) = cache.lock() {
            if let Some(cached) = guard.get(base_font) {
                return cached.clone();
            }
        }
    }

    // Not yet cached — resolve against the system scan
    let scan = get_system_font_scan();
    let substitutes = standard14_substitutes(base_font);

    let result = substitutes.iter().find_map(|&family| {
        let key = family.to_lowercase();
        scan.get(&key).cloned()
    });

    // Store in per-base-font cache
    {
        let cache = get_resolved_cache();
        if let Ok(mut guard) = cache.lock() {
            guard.insert(base_font.to_string(), result.clone());
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outline_builder_records_move_and_line() {
        let mut b = OutlinePathBuilder {
            builder: tiny_skia::PathBuilder::new(),
        };
        ttf_parser::OutlineBuilder::move_to(&mut b, 0.0, 0.0);
        ttf_parser::OutlineBuilder::line_to(&mut b, 100.0, 0.0);
        ttf_parser::OutlineBuilder::line_to(&mut b, 100.0, 100.0);
        ttf_parser::OutlineBuilder::close(&mut b);
        // finish() returns Some if path is non-empty
        assert!(b.builder.finish().is_some());
    }

    #[test]
    fn test_resolve_unknown_basefont_returns_none() {
        assert!(resolve_standard14_substitute("NotARealFont-9999").is_none());
    }

    #[test]
    fn test_resolve_helvetica_consistent() {
        // Two calls return the same result (cached)
        let r1 = resolve_standard14_substitute("Helvetica");
        let r2 = resolve_standard14_substitute("Helvetica");
        assert_eq!(r1.is_some(), r2.is_some());
        if let (Some(p1), Some(p2)) = (&r1, &r2) {
            assert_eq!(p1.0, p2.0);
        }
    }
}
