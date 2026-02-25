//! Font configuration and system font discovery
//!
//! Provides a mapping from font family names to TTF/OTF file paths,
//! and utilities to discover fonts installed on the system.

use std::collections::HashMap;
use std::path::PathBuf;

/// Maps font family names to font file paths.
///
/// Font names are stored in lowercase for case-insensitive lookup.
#[derive(Debug, Default)]
pub struct FontConfig {
    /// Map from font family name (lowercase) to file path
    mappings: HashMap<String, PathBuf>,
}

impl FontConfig {
    /// Create an empty `FontConfig` with no font mappings.
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// Add a font mapping: `name` (case-insensitive) → `path`.
    pub fn add_mapping(&mut self, name: &str, path: PathBuf) {
        self.mappings.insert(name.to_lowercase(), path);
    }

    /// Look up a font file path by family name (case-insensitive).
    ///
    /// Returns `None` if no mapping exists for `family`.
    pub fn find_font(&self, family: &str) -> Option<&PathBuf> {
        self.mappings.get(&family.to_lowercase())
    }

    /// Build a `FontConfig` by scanning standard system font directories
    /// and registering every TTF/OTF file found there.
    ///
    /// Font names are extracted from the font file metadata using `ttf-parser`.
    /// If a file cannot be parsed its name is derived from the file stem instead.
    pub fn with_system_fonts() -> Self {
        let mut config = Self::new();

        for dir in system_font_dirs() {
            if dir.is_dir() {
                scan_font_dir(&dir, &mut config);
            }
        }

        config
    }

    /// Return an iterator over all registered (name, path) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &PathBuf)> {
        self.mappings.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Return the number of registered font mappings.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    /// Return `true` if no font mappings are registered.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }
}

/// Return the platform-specific directories to search for installed fonts.
fn system_font_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    #[cfg(target_os = "linux")]
    {
        dirs.push(PathBuf::from("/usr/share/fonts"));
        dirs.push(PathBuf::from("/usr/local/share/fonts"));
        if let Some(home) = home_dir() {
            dirs.push(home.join(".fonts"));
            dirs.push(home.join(".local/share/fonts"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        dirs.push(PathBuf::from("/Library/Fonts"));
        dirs.push(PathBuf::from("/System/Library/Fonts"));
        if let Some(home) = home_dir() {
            dirs.push(home.join("Library/Fonts"));
        }
    }

    #[cfg(target_os = "windows")]
    {
        dirs.push(PathBuf::from(r"C:\Windows\Fonts"));
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            let mut p = PathBuf::from(local_app_data);
            p.push("Microsoft");
            p.push("Windows");
            p.push("Fonts");
            dirs.push(p);
        }
    }

    // Fallback: on unknown platforms return an empty list.
    dirs
}

/// Obtain the current user's home directory from the `HOME` / `USERPROFILE`
/// environment variable.
fn home_dir() -> Option<PathBuf> {
    // Try HOME first (Unix), then USERPROFILE (Windows)
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

/// Recursively scan `dir` for TTF/OTF files and register them in `config`.
fn scan_font_dir(dir: &std::path::Path, config: &mut FontConfig) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            // Recurse into sub-directories (e.g. /usr/share/fonts/truetype/…)
            scan_font_dir(&path, config);
            continue;
        }

        // Only handle TrueType / OpenType files
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        if !matches!(ext.as_deref(), Some("ttf") | Some("otf")) {
            continue;
        }

        register_font_file(&path, config);
    }
}

/// Try to read the PostScript / family name from `path` using `ttf-parser`
/// and register it in `config`. Falls back to the file stem on parse errors.
fn register_font_file(path: &std::path::Path, config: &mut FontConfig) {
    // Read the font data
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(_) => return,
    };

    // Try to parse the font and extract the preferred family name
    let name = extract_font_family_name(&data).or_else(|| {
        // Fall back to file stem (e.g. "NotoSansCJK-Regular")
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    });

    if let Some(family) = name {
        config.add_mapping(&family, path.to_path_buf());
    }
}

/// Extract the preferred font family name from TTF/OTF data.
///
/// Tries, in order:
/// 1. Name ID 16 – Typographic / Preferred Family
/// 2. Name ID 1  – Font Family
/// 3. Name ID 6  – PostScript name
///
/// Returns `None` if none of these can be found or decoded.
pub fn extract_font_family_name(font_data: &[u8]) -> Option<String> {
    use ttf_parser::name_id;

    let face = ttf_parser::Face::parse(font_data, 0).ok()?;

    // Priority: Preferred Family → Family → PostScript name
    let preferred_ids = [
        name_id::TYPOGRAPHIC_FAMILY, // 16
        name_id::FAMILY,             // 1
        name_id::POST_SCRIPT_NAME,   // 6
    ];

    for &id in &preferred_ids {
        if let Some(name) = face
            .names()
            .into_iter()
            .find(|n| n.name_id == id)
            .and_then(|n| n.to_string())
        {
            return Some(name);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_config_new_is_empty() {
        let cfg = FontConfig::new();
        assert!(cfg.is_empty());
        assert_eq!(cfg.len(), 0);
    }

    #[test]
    fn test_add_and_find_mapping() {
        let mut cfg = FontConfig::new();
        cfg.add_mapping("Noto Sans", PathBuf::from("/usr/share/fonts/NotoSans.ttf"));
        assert_eq!(cfg.len(), 1);

        // Case-insensitive lookup
        assert!(cfg.find_font("noto sans").is_some());
        assert!(cfg.find_font("Noto Sans").is_some());
        assert!(cfg.find_font("NOTO SANS").is_some());
    }

    #[test]
    fn test_find_missing_font_returns_none() {
        let cfg = FontConfig::new();
        assert!(cfg.find_font("NonExistentFont").is_none());
    }

    #[test]
    fn test_add_mapping_overwrites_existing() {
        let mut cfg = FontConfig::new();
        cfg.add_mapping("Arial", PathBuf::from("/path/a.ttf"));
        cfg.add_mapping("Arial", PathBuf::from("/path/b.ttf"));
        // The second mapping should overwrite the first
        assert_eq!(cfg.len(), 1);
        assert_eq!(cfg.find_font("arial"), Some(&PathBuf::from("/path/b.ttf")));
    }

    #[test]
    fn test_with_system_fonts_does_not_panic() {
        // Just verify it can be called without panicking even if no fonts are present
        let _cfg = FontConfig::with_system_fonts();
    }

    #[test]
    fn test_iter() {
        let mut cfg = FontConfig::new();
        cfg.add_mapping("FontA", PathBuf::from("/a.ttf"));
        cfg.add_mapping("FontB", PathBuf::from("/b.ttf"));

        let names: Vec<&str> = cfg.iter().map(|(n, _)| n).collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"fonta"));
        assert!(names.contains(&"fontb"));
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    #[test]
    fn test_font_config_multiple_fonts() {
        let mut cfg = FontConfig::new();
        cfg.add_mapping("Font A", PathBuf::from("/a.ttf"));
        cfg.add_mapping("Font B", PathBuf::from("/b.ttf"));
        cfg.add_mapping("Font C", PathBuf::from("/c.ttf"));
        assert_eq!(cfg.len(), 3);
        assert!(!cfg.is_empty());
    }

    #[test]
    fn test_font_config_lookup_is_case_insensitive() {
        let mut cfg = FontConfig::new();
        cfg.add_mapping("Arial Bold", PathBuf::from("/arial-bold.ttf"));
        assert!(cfg.find_font("arial bold").is_some());
        assert!(cfg.find_font("ARIAL BOLD").is_some());
        assert!(cfg.find_font("Arial Bold").is_some());
        assert!(cfg.find_font("ArIaL bOlD").is_some());
    }

    #[test]
    fn test_font_config_path_is_preserved() {
        let mut cfg = FontConfig::new();
        let path = PathBuf::from("/usr/share/fonts/truetype/NotoSans.ttf");
        cfg.add_mapping("Noto Sans", path.clone());
        assert_eq!(cfg.find_font("noto sans"), Some(&path));
    }

    #[test]
    fn test_font_config_iter_count() {
        let mut cfg = FontConfig::new();
        cfg.add_mapping("F1", PathBuf::from("/f1.ttf"));
        cfg.add_mapping("F2", PathBuf::from("/f2.ttf"));
        let count = cfg.iter().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_extract_font_family_name_invalid_data() {
        let bad_data = b"not a font";
        let result = extract_font_family_name(bad_data);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_font_family_name_empty_data() {
        let result = extract_font_family_name(b"");
        assert!(result.is_none());
    }

    #[test]
    fn test_font_config_default_is_empty() {
        let cfg = FontConfig::default();
        assert!(cfg.is_empty());
    }
}
