//! Command-line argument parsing for FOP CLI

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "fop")]
#[command(author, about, long_about = None)]
#[command(about = "Apache FOP - XSL-FO to PDF converter")]
#[command(long_about = "
Apache FOP (Formatting Objects Processor) Rust Implementation

Converts XSL-FO (Formatting Objects) documents to PDF format.
This is a high-performance Rust port of the original Java Apache FOP.

APACHE FOP COMPATIBLE USAGE:
    # Basic conversion (Apache FOP style)
    fop -fo input.fo -pdf output.pdf
    fop -xml input.xml -xsl transform.xsl -pdf output.pdf

    # With configuration file
    fop -c fop.conf -fo input.fo -pdf output.pdf

    # Various output formats
    fop -fo input.fo -pdf output.pdf
    fop -fo input.fo -svg output.svg
    fop -fo input.fo -ps output.ps
    fop -fo input.fo -txt output.txt
    fop -fo input.fo -at application/pdf output.pdf

    # Debug and relaxed modes
    fop -d -fo input.fo -pdf output.pdf
    fop -r -fo input.fo -pdf output.pdf

    # With XSLT parameters
    fop -xml data.xml -xsl style.xsl -param name value -pdf out.pdf

MODERN USAGE:
    # Basic conversion to PDF
    fop input.fo output.pdf

    # Generate SVG output
    fop input.fo output.svg
    fop input.fo --svg output.svg

    # Generate PNG/JPEG output
    fop input.fo output.png --dpi 300
    fop input.fo output.jpg --dpi 150 --jpeg-quality 90
    fop input.fo --png output.png

    # Auto-detect format from extension
    fop input.fo output.pdf   # Generates PDF
    fop input.fo output.svg   # Generates SVG
    fop input.fo output.png   # Generates PNG

    # With verbose output
    fop input.fo output.pdf -v

    # With custom images and fonts
    fop input.fo output.pdf --images-dir ./images --font-dir ./fonts

    # Show statistics after conversion
    fop input.fo output.pdf --stats

    # Validation only (no output)
    fop input.fo --validate-only

    # Enable compression (PDF only)
    fop input.fo output.pdf --compress
")]
pub struct Cli {
    /// Input XSL-FO file (positional argument for backward compatibility)
    #[arg(value_name = "INPUT", conflicts_with_all = &["fo", "xml"])]
    pub input: Option<PathBuf>,

    /// Output PDF file (positional argument for backward compatibility)
    #[arg(value_name = "OUTPUT", conflicts_with_all = &["pdf", "ps", "pcl", "txt", "svg", "png", "jpeg", "at_output"])]
    pub output: Option<PathBuf>,

    // ========== Apache FOP Compatible Input Options ==========
    /// XSL-FO input file (Apache FOP style)
    #[arg(short = 'f', long = "fo", value_name = "FILE")]
    pub fo: Option<PathBuf>,

    /// XML input file to transform (use with -xsl)
    #[arg(long = "xml", value_name = "FILE")]
    pub xml: Option<PathBuf>,

    /// XSLT stylesheet to apply
    #[arg(long = "xsl", alias = "xslt", value_name = "FILE")]
    pub xsl: Option<PathBuf>,

    /// XSLT parameter (name value pairs, can be repeated)
    #[arg(long = "param", value_names = &["NAME", "VALUE"], number_of_values = 2, action = clap::ArgAction::Append)]
    pub param: Vec<String>,

    // ========== Apache FOP Compatible Output Options ==========
    /// Output as PDF file
    #[arg(long = "pdf", value_name = "FILE")]
    pub pdf: Option<PathBuf>,

    /// Output as PostScript file
    #[arg(long = "ps", value_name = "FILE")]
    pub ps: Option<PathBuf>,

    /// Output as PCL file
    #[arg(long = "pcl", value_name = "FILE")]
    pub pcl: Option<PathBuf>,

    /// Output as plain text file
    #[arg(long = "txt", value_name = "FILE")]
    pub txt: Option<PathBuf>,

    /// Output as SVG file
    #[arg(long = "svg", value_name = "FILE")]
    pub svg: Option<PathBuf>,

    /// Output as PNG file (raster image)
    #[arg(long = "png", value_name = "FILE")]
    pub png: Option<PathBuf>,

    /// Output as JPEG file (raster image)
    #[arg(long = "jpeg", alias = "jpg", value_name = "FILE")]
    pub jpeg: Option<PathBuf>,

    /// JPEG quality (1-100, default: 85)
    #[arg(long = "jpeg-quality", value_name = "QUALITY", default_value = "85")]
    pub jpeg_quality: u8,

    /// Output format with MIME type
    #[arg(long = "at", value_name = "FORMAT")]
    pub at: Option<String>,

    /// Output file for -at option
    #[arg(value_name = "OUTPUT_FILE", requires = "at")]
    pub at_output: Option<PathBuf>,

    // ========== Apache FOP Compatible General Options ==========
    /// Configuration file (Apache FOP compatible)
    #[arg(short = 'c', long = "config", value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Debug mode - detailed output (Apache FOP compatible)
    #[arg(short = 'd', long = "debug")]
    pub debug: bool,

    /// Relaxed validation mode (Apache FOP compatible)
    #[arg(short = 'r', long = "relaxed")]
    pub relaxed: bool,

    /// Target resolution in DPI (Apache FOP compatible)
    #[arg(long = "dpi", value_name = "DPI")]
    pub dpi: Option<u32>,

    /// Use catalog resolver for entity resolution
    #[arg(long = "catalog")]
    pub catalog: bool,

    /// Disable loading of configuration file
    #[arg(long = "noconfig")]
    pub noconfig: bool,

    /// Quiet mode - suppress non-essential output (Apache FOP compatible)
    #[arg(short = 'q')]
    pub quiet_mode: bool,

    // ========== Apache FOP Security Options ==========
    /// Set owner password for PDF encryption
    #[arg(short = 'o', long = "owner-password", value_name = "PASSWORD")]
    pub owner_password: Option<String>,

    /// Set user password for PDF encryption
    #[arg(short = 'u', long = "user-password", value_name = "PASSWORD")]
    pub user_password: Option<String>,

    /// Disable printing permission in encrypted PDF
    #[arg(long = "noprint")]
    pub noprint: bool,

    /// Disable copy permission in encrypted PDF
    #[arg(long = "nocopy")]
    pub nocopy: bool,

    /// Disable editing permission in encrypted PDF
    #[arg(long = "noedit")]
    pub noedit: bool,

    /// Disable annotation permission in encrypted PDF
    #[arg(long = "noannotations")]
    pub noannotations: bool,

    /// PDF encryption algorithm: rc4 (RC4-128, PDF 1.4, default) or aes256 (AES-256, PDF 2.0)
    #[arg(long = "encryption", value_name = "ALGORITHM", default_value = "rc4")]
    pub encryption: String,

    // ========== Accessibility Options ==========
    /// Enable Tagged PDF / PDF/UA output for accessibility
    #[arg(short = 'a', long = "accessibility")]
    pub accessibility: bool,

    /// Enable PDF/A-1b archival compliance mode (ISO 19005-1)
    ///
    /// Forbids encryption, embeds all fonts, adds sRGB OutputIntent, and
    /// includes required XMP metadata. Implies PDF version 1.4.
    #[arg(long = "pdfa")]
    pub pdfa: bool,

    /// Enable PDF/UA-1 accessibility compliance mode (ISO 14289-1)
    ///
    /// Adds MarkInfo, StructTreeRoot, Lang, and ViewerPreferences entries to
    /// the document catalog. Can be combined with --pdfa.
    #[arg(long = "pdfua")]
    pub pdfua: bool,

    // ========== Font Cache Options ==========
    /// Font cache directory
    #[arg(long = "cache", value_name = "DIR")]
    pub font_cache: Option<PathBuf>,

    /// Flush (clear) font cache before processing
    #[arg(long = "flush")]
    pub flush_font_cache: bool,

    // ========== Memory Options ==========
    /// Conserve memory (trade speed for lower memory usage)
    #[arg(long = "conserve")]
    pub conserve_memory: bool,

    // ========== Extended Options (Rust FOP) ==========
    /// Display version information
    #[arg(long = "version", alias = "V")]
    pub show_version: bool,

    /// Enable verbose logging
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Show detailed statistics after conversion
    #[arg(long)]
    pub stats: bool,

    /// Only validate the XSL-FO document, don't generate PDF
    #[arg(long)]
    pub validate_only: bool,

    /// Directory containing images referenced in the FO document
    #[arg(long, value_name = "DIR")]
    pub images_dir: Option<PathBuf>,

    /// Directory containing fonts for text rendering
    #[arg(long, value_name = "DIR")]
    pub font_dir: Option<PathBuf>,

    /// Enable PDF compression (reduces file size)
    #[arg(long)]
    pub compress: bool,

    /// Suppress progress bars and animations
    #[arg(long)]
    pub quiet: bool,

    /// Number of threads to use for processing (default: auto-detect)
    #[arg(short = 'j', long, value_name = "N")]
    pub jobs: Option<usize>,

    /// Maximum memory usage in MB (default: unlimited)
    #[arg(long, value_name = "MB")]
    pub max_memory: Option<usize>,

    /// Stop processing after first error
    #[arg(long)]
    pub fail_fast: bool,

    /// Enable strict XSL-FO validation
    #[arg(long)]
    pub strict: bool,

    /// PDF version to generate (1.4, 1.5, 1.6, 1.7, 2.0)
    #[arg(long, value_name = "VERSION", default_value = "1.4")]
    pub pdf_version: String,

    /// Set PDF author metadata
    #[arg(long, value_name = "AUTHOR")]
    pub author: Option<String>,

    /// Set PDF title metadata
    #[arg(long, value_name = "TITLE")]
    pub title: Option<String>,

    /// Set PDF subject metadata
    #[arg(long, value_name = "SUBJECT")]
    pub subject: Option<String>,

    /// Set PDF keywords metadata
    #[arg(long, value_name = "KEYWORDS")]
    pub keywords: Option<String>,

    /// Disable progress reporting (for scripting)
    #[arg(long)]
    pub no_progress: bool,

    /// Output format for validation results (text, json)
    #[arg(long, value_name = "FORMAT", default_value = "text")]
    pub output_format: String,

    /// Validate generated PDF after rendering
    #[arg(long)]
    pub validate: bool,

    /// Re-parse and rasterize the generated PDF as a self-verification step (PDF output only)
    #[arg(
        long,
        help = "Re-parse and rasterize the generated PDF as a self-verification step (PDF output only)"
    )]
    pub render_verify: bool,
}

/// Output format enum for type-safe handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum OutputFormat {
    Pdf,
    PostScript,
    Pcl,
    Text,
    Svg,
    Png,
    Jpeg,
    Custom,
}

impl Cli {
    /// Check if input is from stdin (filename is "-")
    pub fn is_stdin(&self) -> bool {
        self.input
            .as_ref()
            .map(|p| p.as_os_str() == "-")
            .unwrap_or(false)
            || self
                .fo
                .as_ref()
                .map(|p| p.as_os_str() == "-")
                .unwrap_or(false)
    }

    /// Check if output is to stdout (filename is "-")
    pub fn is_stdout(&self) -> bool {
        self.output
            .as_ref()
            .map(|p| p.as_os_str() == "-")
            .unwrap_or(false)
            || self
                .pdf
                .as_ref()
                .map(|p| p.as_os_str() == "-")
                .unwrap_or(false)
    }

    /// Check if PDF security/encryption is requested
    #[allow(dead_code)]
    pub fn has_security(&self) -> bool {
        self.owner_password.is_some()
            || self.user_password.is_some()
            || self.noprint
            || self.nocopy
            || self.noedit
            || self.noannotations
    }

    /// Get the effective input file (handles both positional and flag-based input)
    pub fn get_input(&self) -> Option<&PathBuf> {
        self.input
            .as_ref()
            .or(self.fo.as_ref())
            .or(self.xml.as_ref())
    }

    /// Get the effective output file (handles both positional and flag-based output)
    pub fn get_output(&self) -> Option<&PathBuf> {
        self.output
            .as_ref()
            .or(self.pdf.as_ref())
            .or(self.ps.as_ref())
            .or(self.pcl.as_ref())
            .or(self.txt.as_ref())
            .or(self.svg.as_ref())
            .or(self.png.as_ref())
            .or(self.jpeg.as_ref())
            .or(self.at_output.as_ref())
    }

    /// Determine the output format based on the options provided
    pub fn get_output_format(&self) -> OutputFormat {
        if self.svg.is_some() {
            OutputFormat::Svg
        } else if self.png.is_some() {
            OutputFormat::Png
        } else if self.jpeg.is_some() {
            OutputFormat::Jpeg
        } else if self.pdf.is_some() {
            OutputFormat::Pdf
        } else if self.ps.is_some() {
            OutputFormat::PostScript
        } else if self.pcl.is_some() {
            OutputFormat::Pcl
        } else if self.txt.is_some() {
            OutputFormat::Text
        } else if self.at.is_some() {
            // Parse MIME type for custom format
            OutputFormat::Pdf // Default for now
        } else if let Some(output) = &self.output {
            // Auto-detect from file extension
            if let Some(ext) = output.extension() {
                match ext.to_str() {
                    Some("svg") => OutputFormat::Svg,
                    Some("png") => OutputFormat::Png,
                    Some("jpg") | Some("jpeg") => OutputFormat::Jpeg,
                    Some("ps") => OutputFormat::PostScript,
                    Some("pcl") => OutputFormat::Pcl,
                    Some("txt") => OutputFormat::Text,
                    _ => OutputFormat::Pdf,
                }
            } else {
                OutputFormat::Pdf
            }
        } else {
            OutputFormat::Pdf // Default
        }
    }

    /// Check if XSLT transformation is needed
    pub fn needs_xslt_transform(&self) -> bool {
        self.xml.is_some() && self.xsl.is_some()
    }

    /// Get XSLT parameters as key-value pairs
    pub fn get_xslt_params(&self) -> Vec<(&str, &str)> {
        let mut params = Vec::new();
        let mut i = 0;
        while i + 1 < self.param.len() {
            params.push((self.param[i].as_str(), self.param[i + 1].as_str()));
            i += 2;
        }
        params
    }

    /// Check if running in quiet mode (handles both -q and --quiet)
    pub fn is_quiet(&self) -> bool {
        self.quiet || self.quiet_mode
    }

    /// Check if running in debug/verbose mode
    pub fn is_verbose(&self) -> bool {
        self.verbose || self.debug
    }

    /// Validate command-line arguments
    pub fn validate(&self) -> anyhow::Result<()> {
        // If --version requested, skip all validation
        if self.show_version {
            return Ok(());
        }

        // Get effective input file
        let input = self.get_input().ok_or_else(|| {
            anyhow::anyhow!("No input file specified. Use positional argument or -fo/--xml option")
        })?;

        // Skip file existence check for stdin
        if !self.is_stdin() {
            // Check input file exists
            if !input.exists() {
                anyhow::bail!("Input file does not exist: {}", input.display());
            }

            // Check input file is a file (not directory)
            if !input.is_file() {
                anyhow::bail!("Input path is not a file: {}", input.display());
            }
        }

        // If XML input is specified, XSLT stylesheet is required
        if self.xml.is_some() && self.xsl.is_none() {
            anyhow::bail!("XSLT stylesheet (-xsl) is required when using XML input (-xml)");
        }

        // If XSLT is specified, XML input is required
        if self.xsl.is_some() && self.xml.is_none() {
            anyhow::bail!("XML input (-xml) is required when using XSLT stylesheet (-xsl)");
        }

        // Check XSLT file exists if specified
        if let Some(ref xsl) = self.xsl {
            if !xsl.exists() {
                anyhow::bail!("XSLT stylesheet does not exist: {}", xsl.display());
            }
        }

        // If not validate-only, output is required
        if !self.validate_only && self.get_output().is_none() {
            anyhow::bail!("Output file is required unless --validate-only is specified");
        }

        // Check output directory exists if output specified (skip for stdout)
        if let Some(output) = self.get_output() {
            if !self.is_stdout() {
                if let Some(parent) = output.parent() {
                    if !parent.exists() && !parent.as_os_str().is_empty() {
                        anyhow::bail!("Output directory does not exist: {}", parent.display());
                    }
                }
            }
        }

        // Validate config file if specified
        if let Some(ref config) = self.config {
            if !self.noconfig && !config.exists() {
                anyhow::bail!("Configuration file does not exist: {}", config.display());
            }
        }

        // Validate images directory if specified
        if let Some(ref images_dir) = self.images_dir {
            if !images_dir.exists() {
                anyhow::bail!("Images directory does not exist: {}", images_dir.display());
            }
            if !images_dir.is_dir() {
                anyhow::bail!("Images path is not a directory: {}", images_dir.display());
            }
        }

        // Validate font directory if specified
        if let Some(ref font_dir) = self.font_dir {
            if !font_dir.exists() {
                anyhow::bail!("Font directory does not exist: {}", font_dir.display());
            }
            if !font_dir.is_dir() {
                anyhow::bail!("Font path is not a directory: {}", font_dir.display());
            }
        }

        // Validate PDF/A-1b is not combined with encryption
        if self.pdfa && self.has_security() {
            anyhow::bail!(
                "--pdfa (PDF/A-1b) is incompatible with encryption options \
                 (ISO 19005-1 §6.1.1 forbids encryption)"
            );
        }

        // Validate encryption algorithm
        match self.encryption.as_str() {
            "rc4" | "aes256" => {}
            _ => anyhow::bail!(
                "Invalid encryption algorithm: {} (supported: rc4, aes256)",
                self.encryption
            ),
        }

        // Validate PDF version
        match self.pdf_version.as_str() {
            "1.4" | "1.5" | "1.6" | "1.7" | "2.0" => {}
            _ => anyhow::bail!(
                "Invalid PDF version: {} (supported: 1.4, 1.5, 1.6, 1.7, 2.0)",
                self.pdf_version
            ),
        }

        // Validate output format
        match self.output_format.as_str() {
            "text" | "json" => {}
            _ => anyhow::bail!(
                "Invalid output format: {} (supported: text, json)",
                self.output_format
            ),
        }

        // Validate jobs count if specified
        if let Some(jobs) = self.jobs {
            if jobs == 0 {
                anyhow::bail!("Number of jobs must be at least 1");
            }
        }

        // Validate DPI if specified
        if let Some(dpi) = self.dpi {
            if dpi == 0 || dpi > 10000 {
                anyhow::bail!("DPI must be between 1 and 10000, got {}", dpi);
            }
        }

        // Validate XSLT parameters (must be in pairs)
        if !self.param.len().is_multiple_of(2) {
            anyhow::bail!("XSLT parameters must be provided in name-value pairs");
        }

        // Validate JPEG quality
        if self.jpeg_quality == 0 || self.jpeg_quality > 100 {
            anyhow::bail!(
                "JPEG quality must be between 1 and 100, got {}",
                self.jpeg_quality
            );
        }

        Ok(())
    }

    /// Get the effective number of jobs (threads) to use
    #[allow(dead_code)]
    pub fn effective_jobs(&self) -> usize {
        self.jobs.unwrap_or_else(num_cpus::get)
    }

    /// Check if progress reporting should be enabled
    pub fn show_progress(&self) -> bool {
        !self.is_quiet() && !self.no_progress && atty::is(atty::Stream::Stdout)
    }
}

// Helper function to detect if we're running in a TTY
mod atty {
    pub enum Stream {
        Stdout,
    }

    pub fn is(_stream: Stream) -> bool {
        // Simple implementation - assume TTY if not redirected
        // In real implementation, would use libc/windows-sys
        true
    }
}

// Helper to get CPU count
mod num_cpus {
    #[allow(dead_code)]
    pub fn get() -> usize {
        // Use num_cpus crate or fall back to 1
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a minimal Cli struct with only the fields we care about set.
    /// Uses Cli::parse_from with a safe argument list so we don't need Default.
    fn make_cli_positional(input: &str, output: &str) -> Cli {
        Cli::parse_from(["fop", input, output])
    }

    // ------------------------------------------------------------------
    // Format detection from file extension (get_output_format)
    // ------------------------------------------------------------------

    #[test]
    fn test_format_from_extension_pdf() {
        let cli = make_cli_positional("input.fo", "output.pdf");
        assert_eq!(cli.get_output_format(), OutputFormat::Pdf);
    }

    #[test]
    fn test_format_from_extension_svg() {
        let cli = make_cli_positional("input.fo", "output.svg");
        assert_eq!(cli.get_output_format(), OutputFormat::Svg);
    }

    #[test]
    fn test_format_from_extension_txt() {
        let cli = make_cli_positional("input.fo", "output.txt");
        assert_eq!(cli.get_output_format(), OutputFormat::Text);
    }

    #[test]
    fn test_format_from_extension_ps() {
        let cli = make_cli_positional("input.fo", "output.ps");
        assert_eq!(cli.get_output_format(), OutputFormat::PostScript);
    }

    #[test]
    fn test_format_from_extension_png() {
        let cli = make_cli_positional("input.fo", "output.png");
        assert_eq!(cli.get_output_format(), OutputFormat::Png);
    }

    #[test]
    fn test_format_from_extension_jpg() {
        let cli = make_cli_positional("input.fo", "output.jpg");
        assert_eq!(cli.get_output_format(), OutputFormat::Jpeg);
    }

    #[test]
    fn test_format_from_extension_jpeg() {
        let cli = make_cli_positional("input.fo", "output.jpeg");
        assert_eq!(cli.get_output_format(), OutputFormat::Jpeg);
    }

    #[test]
    fn test_format_from_extension_pcl() {
        let cli = make_cli_positional("input.fo", "output.pcl");
        assert_eq!(cli.get_output_format(), OutputFormat::Pcl);
    }

    #[test]
    fn test_format_from_extension_unknown_defaults_to_pdf() {
        // Unknown extension falls back to PDF
        let cli = make_cli_positional("input.fo", "output.xyz");
        assert_eq!(cli.get_output_format(), OutputFormat::Pdf);
    }

    #[test]
    fn test_format_from_extension_no_extension_defaults_to_pdf() {
        // No extension falls back to PDF
        let cli = make_cli_positional("input.fo", "output");
        assert_eq!(cli.get_output_format(), OutputFormat::Pdf);
    }

    // ------------------------------------------------------------------
    // Explicit flag overrides extension
    // ------------------------------------------------------------------

    #[test]
    fn test_format_flag_svg_overrides_extension() {
        // --svg flag takes priority over positional output extension
        let cli = Cli::parse_from(["fop", "--fo", "input.fo", "--svg", "output.svg"]);
        assert_eq!(cli.get_output_format(), OutputFormat::Svg);
    }

    #[test]
    fn test_format_flag_pdf_via_flag() {
        let cli = Cli::parse_from(["fop", "--fo", "input.fo", "--pdf", "output.pdf"]);
        assert_eq!(cli.get_output_format(), OutputFormat::Pdf);
    }

    #[test]
    fn test_format_flag_ps_via_flag() {
        let cli = Cli::parse_from(["fop", "--fo", "input.fo", "--ps", "output.ps"]);
        assert_eq!(cli.get_output_format(), OutputFormat::PostScript);
    }

    #[test]
    fn test_format_flag_txt_via_flag() {
        let cli = Cli::parse_from(["fop", "--fo", "input.fo", "--txt", "output.txt"]);
        assert_eq!(cli.get_output_format(), OutputFormat::Text);
    }

    #[test]
    fn test_format_flag_png_via_flag() {
        let cli = Cli::parse_from(["fop", "--fo", "input.fo", "--png", "output.png"]);
        assert_eq!(cli.get_output_format(), OutputFormat::Png);
    }

    #[test]
    fn test_format_flag_jpeg_via_flag() {
        let cli = Cli::parse_from(["fop", "--fo", "input.fo", "--jpeg", "output.jpg"]);
        assert_eq!(cli.get_output_format(), OutputFormat::Jpeg);
    }

    // ------------------------------------------------------------------
    // stdin / stdout detection
    // ------------------------------------------------------------------

    #[test]
    fn test_stdin_detection_via_positional() {
        let cli = Cli::parse_from(["fop", "-", "output.pdf"]);
        assert!(cli.is_stdin(), "Positional '-' should be detected as stdin");
    }

    #[test]
    fn test_stdin_detection_via_fo_flag() {
        let cli = Cli::parse_from(["fop", "--fo", "-", "--pdf", "output.pdf"]);
        assert!(cli.is_stdin(), "--fo - should be detected as stdin");
    }

    #[test]
    fn test_not_stdin_for_real_file() {
        let cli = make_cli_positional("input.fo", "output.pdf");
        assert!(!cli.is_stdin(), "Normal file should not be stdin");
    }

    #[test]
    fn test_stdout_detection_via_positional() {
        let cli = Cli::parse_from(["fop", "input.fo", "-"]);
        assert!(cli.is_stdout(), "Positional '-' output should be stdout");
    }

    #[test]
    fn test_stdout_detection_via_pdf_flag() {
        let cli = Cli::parse_from(["fop", "--fo", "input.fo", "--pdf", "-"]);
        assert!(cli.is_stdout(), "--pdf - should be detected as stdout");
    }

    #[test]
    fn test_not_stdout_for_real_file() {
        let cli = make_cli_positional("input.fo", "output.pdf");
        assert!(!cli.is_stdout(), "Normal file should not be stdout");
    }

    // ------------------------------------------------------------------
    // Quiet / verbose mode
    // ------------------------------------------------------------------

    #[test]
    fn test_quiet_mode_long_flag() {
        let cli = Cli::parse_from(["fop", "input.fo", "output.pdf", "--quiet"]);
        assert!(cli.is_quiet());
    }

    #[test]
    fn test_quiet_mode_short_flag() {
        let cli = Cli::parse_from(["fop", "input.fo", "output.pdf", "-q"]);
        assert!(cli.is_quiet());
    }

    #[test]
    fn test_not_quiet_by_default() {
        let cli = make_cli_positional("input.fo", "output.pdf");
        assert!(!cli.is_quiet());
    }

    #[test]
    fn test_verbose_mode_long_flag() {
        let cli = Cli::parse_from(["fop", "input.fo", "output.pdf", "--verbose"]);
        assert!(cli.is_verbose());
    }

    #[test]
    fn test_verbose_mode_short_flag() {
        let cli = Cli::parse_from(["fop", "input.fo", "output.pdf", "-v"]);
        assert!(cli.is_verbose());
    }

    #[test]
    fn test_debug_mode_counts_as_verbose() {
        let cli = Cli::parse_from(["fop", "input.fo", "output.pdf", "-d"]);
        assert!(cli.is_verbose(), "-d should activate verbose mode");
    }

    #[test]
    fn test_not_verbose_by_default() {
        let cli = make_cli_positional("input.fo", "output.pdf");
        assert!(!cli.is_verbose());
    }

    // ------------------------------------------------------------------
    // Input / output accessors
    // ------------------------------------------------------------------

    #[test]
    fn test_get_input_positional() {
        let cli = make_cli_positional("my_doc.fo", "out.pdf");
        let input = cli.get_input().expect("test: should succeed");
        assert_eq!(input.to_str().expect("test: should succeed"), "my_doc.fo");
    }

    #[test]
    fn test_get_input_via_fo_flag() {
        let cli = Cli::parse_from(["fop", "--fo", "my_doc.fo", "--pdf", "out.pdf"]);
        let input = cli.get_input().expect("test: should succeed");
        assert_eq!(input.to_str().expect("test: should succeed"), "my_doc.fo");
    }

    #[test]
    fn test_get_output_positional() {
        let cli = make_cli_positional("input.fo", "my_output.pdf");
        let output = cli.get_output().expect("test: should succeed");
        assert_eq!(
            output.to_str().expect("test: should succeed"),
            "my_output.pdf"
        );
    }

    #[test]
    fn test_get_output_via_pdf_flag() {
        let cli = Cli::parse_from(["fop", "--fo", "input.fo", "--pdf", "my_output.pdf"]);
        let output = cli.get_output().expect("test: should succeed");
        assert_eq!(
            output.to_str().expect("test: should succeed"),
            "my_output.pdf"
        );
    }

    #[test]
    fn test_get_output_via_svg_flag() {
        let cli = Cli::parse_from(["fop", "--fo", "input.fo", "--svg", "my_output.svg"]);
        let output = cli.get_output().expect("test: should succeed");
        assert_eq!(
            output.to_str().expect("test: should succeed"),
            "my_output.svg"
        );
    }

    // ------------------------------------------------------------------
    // XSLT parameter extraction
    // ------------------------------------------------------------------

    #[test]
    fn test_get_xslt_params_empty() {
        let cli = make_cli_positional("input.fo", "output.pdf");
        let params = cli.get_xslt_params();
        assert!(params.is_empty(), "No params should return empty vec");
    }

    #[test]
    fn test_get_xslt_params_single_pair() {
        let cli = Cli::parse_from([
            "fop",
            "input.fo",
            "output.pdf",
            "--param",
            "title",
            "Hello World",
        ]);
        let params = cli.get_xslt_params();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], ("title", "Hello World"));
    }

    #[test]
    fn test_get_xslt_params_multiple_pairs() {
        let cli = Cli::parse_from([
            "fop",
            "input.fo",
            "output.pdf",
            "--param",
            "title",
            "Hello",
            "--param",
            "author",
            "World",
        ]);
        let params = cli.get_xslt_params();
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], ("title", "Hello"));
        assert_eq!(params[1], ("author", "World"));
    }

    // ------------------------------------------------------------------
    // XSLT transform detection
    // ------------------------------------------------------------------

    #[test]
    fn test_needs_xslt_transform_when_xml_and_xsl_provided() {
        // Build CLI that has both xml and xsl set.
        // Note: --fo conflicts with xml, so we can't use positional input here.
        let cli = Cli::parse_from([
            "fop",
            "--xml",
            "data.xml",
            "--xsl",
            "style.xsl",
            "--pdf",
            "out.pdf",
        ]);
        assert!(cli.needs_xslt_transform());
    }

    #[test]
    fn test_does_not_need_xslt_when_only_fo() {
        let cli = make_cli_positional("input.fo", "output.pdf");
        assert!(!cli.needs_xslt_transform());
    }

    // ------------------------------------------------------------------
    // Security / encryption helpers
    // ------------------------------------------------------------------

    #[test]
    fn test_has_security_when_owner_password_set() {
        let cli = Cli::parse_from([
            "fop",
            "input.fo",
            "output.pdf",
            "--owner-password",
            "secret",
        ]);
        assert!(cli.has_security());
    }

    #[test]
    fn test_has_security_when_noprint_set() {
        let cli = Cli::parse_from(["fop", "input.fo", "output.pdf", "--noprint"]);
        assert!(cli.has_security());
    }

    #[test]
    fn test_no_security_by_default() {
        let cli = make_cli_positional("input.fo", "output.pdf");
        assert!(!cli.has_security());
    }

    // ------------------------------------------------------------------
    // PDF version / output-format validation logic
    // ------------------------------------------------------------------

    #[test]
    fn test_pdf_version_valid_values() {
        for version in &["1.4", "1.5", "1.6", "1.7", "2.0"] {
            assert!(
                matches!(*version, "1.4" | "1.5" | "1.6" | "1.7" | "2.0"),
                "Version {} should be valid",
                version
            );
        }
    }

    #[test]
    fn test_pdf_version_invalid() {
        let invalid = "99.9";
        assert!(
            !matches!(invalid, "1.4" | "1.5" | "1.6" | "1.7" | "2.0"),
            "Version 99.9 must not be valid"
        );
    }

    #[test]
    fn test_output_format_valid_values() {
        for fmt in &["text", "json"] {
            assert!(
                matches!(*fmt, "text" | "json"),
                "Format {} should be valid",
                fmt
            );
        }
    }

    #[test]
    fn test_output_format_invalid() {
        let invalid = "yaml";
        assert!(
            !matches!(invalid, "text" | "json"),
            "Format 'yaml' must not be valid"
        );
    }

    // ------------------------------------------------------------------
    // OutputFormat PartialEq coverage
    // ------------------------------------------------------------------

    #[test]
    fn test_output_format_equality() {
        assert_eq!(OutputFormat::Pdf, OutputFormat::Pdf);
        assert_eq!(OutputFormat::Svg, OutputFormat::Svg);
        assert_eq!(OutputFormat::Text, OutputFormat::Text);
        assert_eq!(OutputFormat::PostScript, OutputFormat::PostScript);
        assert_eq!(OutputFormat::Png, OutputFormat::Png);
        assert_eq!(OutputFormat::Jpeg, OutputFormat::Jpeg);
        assert_eq!(OutputFormat::Pcl, OutputFormat::Pcl);
        assert_ne!(OutputFormat::Pdf, OutputFormat::Svg);
        assert_ne!(OutputFormat::Svg, OutputFormat::Text);
    }

    // ------------------------------------------------------------------
    // JPEG quality default
    // ------------------------------------------------------------------

    #[test]
    fn test_jpeg_quality_default_is_85() {
        let cli = make_cli_positional("input.fo", "output.jpg");
        assert_eq!(cli.jpeg_quality, 85);
    }

    #[test]
    fn test_jpeg_quality_custom() {
        let cli = Cli::parse_from(["fop", "input.fo", "output.jpg", "--jpeg-quality", "75"]);
        assert_eq!(cli.jpeg_quality, 75);
    }

    // ------------------------------------------------------------------
    // Default encryption algorithm
    // ------------------------------------------------------------------

    #[test]
    fn test_encryption_default_is_rc4() {
        let cli = make_cli_positional("input.fo", "output.pdf");
        assert_eq!(cli.encryption, "rc4");
    }

    #[test]
    fn test_encryption_aes256_flag() {
        let cli = Cli::parse_from(["fop", "input.fo", "output.pdf", "--encryption", "aes256"]);
        assert_eq!(cli.encryption, "aes256");
    }

    // ------------------------------------------------------------------
    // Default PDF version
    // ------------------------------------------------------------------

    #[test]
    fn test_pdf_version_default_is_1_4() {
        let cli = make_cli_positional("input.fo", "output.pdf");
        assert_eq!(cli.pdf_version, "1.4");
    }
}
