//! Apache FOP Command-Line Interface
//!
//! This is the main entry point for the FOP CLI tool that converts
//! XSL-FO documents to PDF format.

mod cli;
mod progress;

use anyhow::{Context, Result};
use clap::Parser;
use cli::Cli;
use console::style;
use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::{EncryptionAlgorithm, PdfPermissions, PdfSecurity};
use fop_render::{
    ParallelRenderer, PdfCompliance, PdfRenderer, PdfValidator, PsRenderer, RasterFormat,
    RasterRenderer, SvgRenderer, TextRenderer,
};
use log::{debug, error, info};
use progress::{ProcessingStats, ProgressReporter};
use std::fs;
use std::io::{self, Cursor, Read as _, Write as _};
use std::path::Path;
use std::process::Command;
use std::time::Instant;

fn main() {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Initialize logging based on verbosity
    init_logging(&cli);

    // Run the application and handle errors
    if let Err(err) = run(cli) {
        error!("Error: {}", err);

        // Print error chain
        if let Some(cause) = err.source() {
            eprintln!("\nCaused by:");
            for (i, e) in std::iter::successors(Some(cause), |e| e.source()).enumerate() {
                eprintln!("  {}: {}", i, e);
            }
        }

        std::process::exit(1);
    }
}

/// Initialize logging system
fn init_logging(cli: &Cli) {
    let log_level = if cli.is_verbose() { "debug" } else { "info" };

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .format_timestamp(None)
        .format_module_path(false)
        .init();
}

/// Main application logic
fn run(cli: Cli) -> Result<()> {
    let start_time = Instant::now();

    // Validate arguments
    cli.validate().context("Invalid command-line arguments")?;

    // Handle --version flag
    if cli.show_version {
        print_version_info();
        return Ok(());
    }

    // Print banner
    if !cli.is_quiet() {
        print_banner();
    }

    // Get effective input file
    let input = cli
        .get_input()
        .ok_or_else(|| anyhow::anyhow!("No input file specified"))?;

    // Create progress reporter (disable for stdout output to avoid mixing)
    let show_progress = cli.show_progress() && !cli.is_stdout();
    let mut progress = ProgressReporter::new(show_progress);
    let mut stats = ProcessingStats::new();

    // If XSLT transformation is needed, perform it first
    let input_data = if cli.needs_xslt_transform() {
        perform_xslt_transform(&cli, &mut progress, &mut stats)?
    } else if cli.is_stdin() {
        // Read from stdin
        progress.start_phase("Reading from stdin...");
        let phase_start = Instant::now();

        let mut data = Vec::new();
        io::stdin()
            .read_to_end(&mut data)
            .context("Failed to read from stdin")?;

        stats.input_size = data.len() as u64;
        debug!("Read {} bytes from stdin", data.len());
        progress.finish_phase(
            &format!("Read from stdin ({} bytes)", data.len()),
            phase_start.elapsed(),
        );

        data
    } else {
        // Phase 1: Read input file
        progress.start_phase("Reading input file...");
        let phase_start = Instant::now();

        let data = fs::read(input)
            .with_context(|| format!("Failed to read input file: {}", input.display()))?;

        stats.input_size = data.len() as u64;

        debug!("Read {} bytes from {}", data.len(), input.display());
        progress.finish_phase(
            &format!("Read input file: {}", input.display()),
            phase_start.elapsed(),
        );

        data
    };

    // Phase 2: Parse XSL-FO document
    progress.start_phase("Parsing XSL-FO document...");
    let phase_start = Instant::now();

    let builder = FoTreeBuilder::new();
    let cursor = Cursor::new(&input_data);

    let arena = builder
        .parse(cursor)
        .context("Failed to parse XSL-FO document")?;

    stats.nodes_parsed = arena.len();
    stats.parse_duration = phase_start.elapsed();

    info!("Parsed {} FO nodes", stats.nodes_parsed);
    progress.finish_phase(
        &format!("Parsed FO tree ({} nodes)", stats.nodes_parsed),
        stats.parse_duration,
    );

    // If validation-only mode, stop here
    if cli.validate_only {
        if !cli.is_quiet() {
            println!("\n{} Document is valid!", style("✓").green().bold());
        }

        if cli.stats {
            stats.total_duration = start_time.elapsed();
            stats.display();
        }

        return Ok(());
    }

    // Phase 3: Layout processing
    progress.start_phase("Processing layout...");
    let phase_start = Instant::now();

    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&arena).context("Failed to process layout")?;

    stats.areas_created = area_tree.len();
    stats.layout_duration = phase_start.elapsed();

    info!("Created {} areas", stats.areas_created);
    progress.finish_phase(
        &format!("Layout complete ({} areas)", stats.areas_created),
        stats.layout_duration,
    );

    // Phase 4: Rendering
    let output = cli
        .get_output()
        .context("Output path not specified; pass an output file path")?;
    let output_format = cli.get_output_format();

    let final_bytes = match output_format {
        cli::OutputFormat::Svg => {
            progress.start_phase("Rendering SVG...");
            let phase_start = Instant::now();

            let renderer = SvgRenderer::new();
            let svg_content = renderer
                .render_to_svg(&area_tree)
                .context("Failed to render SVG")?;

            // Count pages in area tree
            let page_count = area_tree
                .iter()
                .filter(|(_, node)| matches!(node.area.area_type, fop_layout::AreaType::Page))
                .count();
            stats.pages_generated = page_count;

            info!("Generated {} pages", stats.pages_generated);

            let svg_bytes = svg_content.into_bytes();
            stats.output_size = svg_bytes.len() as u64;
            stats.render_duration = phase_start.elapsed();

            progress.finish_phase(
                &format!("SVG rendered ({} pages)", stats.pages_generated),
                stats.render_duration,
            );

            svg_bytes
        }
        cli::OutputFormat::Png => {
            progress.start_phase("Rendering PNG...");
            let phase_start = Instant::now();

            let dpi = cli.dpi.unwrap_or(96);
            let renderer = RasterRenderer::new(dpi);
            let pages = renderer
                .render_to_raster(&area_tree, RasterFormat::Png)
                .context("Failed to render PNG")?;

            stats.pages_generated = pages.len();
            info!("Generated {} page(s)", stats.pages_generated);

            // For multi-page documents, save each page separately
            if pages.len() > 1 {
                let output_path = output;
                let base_name = output_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output");
                let parent = output_path
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new("."));

                for (i, page_data) in pages.iter().enumerate() {
                    let page_path = parent.join(format!("{}-page{}.png", base_name, i + 1));
                    fs::write(&page_path, page_data)
                        .with_context(|| format!("Failed to write page {}", i + 1))?;
                    info!("Saved page {} to {}", i + 1, page_path.display());
                }

                stats.output_size = pages.iter().map(|p| p.len() as u64).sum();
                stats.render_duration = phase_start.elapsed();

                progress.finish_phase(
                    &format!("PNG rendered ({} pages)", stats.pages_generated),
                    stats.render_duration,
                );

                // Return the first page for final stats
                pages.into_iter().next().unwrap_or_default()
            } else {
                let page_data = pages.into_iter().next().unwrap_or_default();
                stats.output_size = page_data.len() as u64;
                stats.render_duration = phase_start.elapsed();

                progress.finish_phase(
                    &format!("PNG rendered ({} pages)", stats.pages_generated),
                    stats.render_duration,
                );

                page_data
            }
        }
        cli::OutputFormat::Jpeg => {
            progress.start_phase("Rendering JPEG...");
            let phase_start = Instant::now();

            let dpi = cli.dpi.unwrap_or(96);
            let quality = cli.jpeg_quality;
            let renderer = RasterRenderer::new(dpi);
            let pages = renderer
                .render_to_raster(&area_tree, RasterFormat::Jpeg { quality })
                .context("Failed to render JPEG")?;

            stats.pages_generated = pages.len();
            info!("Generated {} page(s)", stats.pages_generated);

            // For multi-page documents, save each page separately
            if pages.len() > 1 {
                let output_path = output;
                let base_name = output_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output");
                let parent = output_path
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new("."));

                for (i, page_data) in pages.iter().enumerate() {
                    let page_path = parent.join(format!("{}-page{}.jpg", base_name, i + 1));
                    fs::write(&page_path, page_data)
                        .with_context(|| format!("Failed to write page {}", i + 1))?;
                    info!("Saved page {} to {}", i + 1, page_path.display());
                }

                stats.output_size = pages.iter().map(|p| p.len() as u64).sum();
                stats.render_duration = phase_start.elapsed();

                progress.finish_phase(
                    &format!("JPEG rendered ({} pages)", stats.pages_generated),
                    stats.render_duration,
                );

                // Return the first page for final stats
                pages.into_iter().next().unwrap_or_default()
            } else {
                let page_data = pages.into_iter().next().unwrap_or_default();
                stats.output_size = page_data.len() as u64;
                stats.render_duration = phase_start.elapsed();

                progress.finish_phase(
                    &format!("JPEG rendered ({} pages)", stats.pages_generated),
                    stats.render_duration,
                );

                page_data
            }
        }
        cli::OutputFormat::PostScript => {
            progress.start_phase("Rendering PostScript...");
            let phase_start = Instant::now();

            let renderer = PsRenderer::new();
            let ps_content = renderer
                .render_to_ps(&area_tree)
                .context("Failed to render PostScript")?;

            // Count pages in area tree
            let page_count = area_tree
                .iter()
                .filter(|(_, node)| matches!(node.area.area_type, fop_layout::AreaType::Page))
                .count();
            stats.pages_generated = page_count;

            info!("Generated {} pages", stats.pages_generated);

            let ps_bytes = ps_content.into_bytes();
            stats.output_size = ps_bytes.len() as u64;
            stats.render_duration = phase_start.elapsed();

            progress.finish_phase(
                &format!("PostScript rendered ({} pages)", stats.pages_generated),
                stats.render_duration,
            );

            ps_bytes
        }
        cli::OutputFormat::Text => {
            progress.start_phase("Rendering text...");
            let phase_start = Instant::now();

            let renderer = TextRenderer::new();
            let text_content = renderer
                .render_to_text(&area_tree)
                .context("Failed to render text")?;

            // Count pages in area tree
            let page_count = area_tree
                .iter()
                .filter(|(_, node)| matches!(node.area.area_type, fop_layout::AreaType::Page))
                .count();
            stats.pages_generated = page_count;

            info!("Generated {} pages", stats.pages_generated);

            let text_bytes = text_content.into_bytes();
            stats.output_size = text_bytes.len() as u64;
            stats.render_duration = phase_start.elapsed();

            progress.finish_phase(
                &format!("Text rendered ({} pages)", stats.pages_generated),
                stats.render_duration,
            );

            text_bytes
        }
        _ => {
            // Default to PDF rendering
            progress.start_phase("Rendering PDF...");
            let phase_start = Instant::now();

            // Use parallel renderer if --jobs is specified and > 1
            let mut pdf_doc = if let Some(jobs) = cli.jobs {
                if jobs > 1 && area_tree.len() > 1 {
                    info!("Using parallel renderer with {} threads", jobs);
                    let renderer = ParallelRenderer::new(jobs);
                    renderer
                        .render(&area_tree)
                        .context("Failed to render PDF (parallel)")?
                } else {
                    let renderer = PdfRenderer::new();
                    renderer
                        .render(&area_tree)
                        .context("Failed to render PDF")?
                }
            } else {
                let renderer = PdfRenderer::new();
                renderer
                    .render(&area_tree)
                    .context("Failed to render PDF")?
            };

            stats.pages_generated = pdf_doc.pages.len();

            info!("Generated {} pages", stats.pages_generated);
            progress.update_message(&format!(
                "Rendering PDF ({} pages)...",
                stats.pages_generated
            ));

            // Apply compliance mode (PDF/A-1b and/or PDF/UA-1) if requested
            let compliance = match (cli.pdfa, cli.pdfua) {
                (true, true) => Some(PdfCompliance::PdfA1bUA1),
                (true, false) => Some(PdfCompliance::PdfA1b),
                (false, true) => Some(PdfCompliance::PdfUA1),
                (false, false) => None,
            };
            if let Some(c) = compliance {
                pdf_doc
                    .set_compliance(c)
                    .context("Failed to apply PDF compliance mode")?;
                info!("PDF compliance mode: {:?}", c);
            }

            // Apply PDF encryption before serialization if security options are set
            if cli.has_security() {
                let permissions = PdfPermissions {
                    allow_print: !cli.noprint,
                    allow_copy: !cli.nocopy,
                    allow_modify: !cli.noedit,
                    allow_annotations: !cli.noannotations,
                    ..Default::default()
                };

                let owner_pwd = cli.owner_password.as_deref().unwrap_or("");
                let user_pwd = cli.user_password.as_deref().unwrap_or("");

                // Select encryption algorithm from CLI flag
                let security = match cli.encryption.as_str() {
                    "aes256" => PdfSecurity::new_aes256(owner_pwd, user_pwd, permissions),
                    _ => {
                        // Default: rc4
                        PdfSecurity::new(owner_pwd, user_pwd, permissions)
                    }
                };

                let algo_name = match security.algorithm {
                    EncryptionAlgorithm::Aes256 => "AES-256",
                    EncryptionAlgorithm::Rc4128 => "RC4-128",
                };

                let file_id = fop_render::pdf::security::generate_file_id(&format!(
                    "fop-{}-{}",
                    env!("CARGO_PKG_VERSION"),
                    stats.pages_generated
                ));

                let encryption_dict = security.compute_encryption_dict(&file_id);

                info!(
                    "PDF encryption applied ({}, permissions: P={})",
                    algo_name, encryption_dict.p_value
                );

                pdf_doc
                    .set_encryption(encryption_dict, file_id)
                    .context("Failed to apply PDF encryption")?;
            }

            // Serialize PDF
            progress.update_message("Serializing PDF...");

            let pdf_bytes = pdf_doc.to_bytes().context("Failed to serialize PDF")?;

            stats.output_size = pdf_bytes.len() as u64;
            stats.render_duration = phase_start.elapsed();

            progress.finish_phase(
                &format!("PDF rendered ({} pages)", stats.pages_generated),
                stats.render_duration,
            );

            // Handle other output formats
            match output_format {
                cli::OutputFormat::Pdf => pdf_bytes,
                cli::OutputFormat::Pcl => {
                    if !cli.is_quiet() {
                        eprintln!(
                            "Warning: PCL output not yet implemented, generating PDF instead"
                        );
                    }
                    pdf_bytes
                }
                cli::OutputFormat::Custom => pdf_bytes,
                cli::OutputFormat::Svg
                | cli::OutputFormat::Text
                | cli::OutputFormat::PostScript
                | cli::OutputFormat::Png
                | cli::OutputFormat::Jpeg => unreachable!(),
            }
        }
    };

    // Phase 5: Write output
    if cli.is_stdout() {
        progress.start_phase("Writing to stdout...");
        let phase_start = Instant::now();

        io::stdout()
            .write_all(&final_bytes)
            .context("Failed to write to stdout")?;
        io::stdout().flush().context("Failed to flush stdout")?;

        progress.finish_phase("Written to stdout", phase_start.elapsed());
    } else {
        progress.start_phase("Writing output file...");
        let phase_start = Instant::now();

        fs::write(output, &final_bytes)
            .with_context(|| format!("Failed to write output file: {}", output.display()))?;

        progress.finish_phase(
            &format!("Saved to {}", output.display()),
            phase_start.elapsed(),
        );
    }

    // Phase 6: Validate PDF if requested
    if cli.validate && matches!(output_format, cli::OutputFormat::Pdf) {
        progress.start_phase("Validating PDF...");
        let phase_start = Instant::now();

        let validator = if cli.strict {
            PdfValidator::new_strict()
        } else {
            PdfValidator::new()
        };

        let validation_result = validator.validate_pdf(&final_bytes);

        progress.finish_phase("PDF validation complete", phase_start.elapsed());

        // Display validation results
        if !cli.is_quiet() {
            match &validation_result {
                fop_render::ValidationResult::Valid => {
                    println!("\n{} PDF validation passed!", style("✓").green().bold());
                }
                fop_render::ValidationResult::Warning(warnings) => {
                    println!(
                        "\n{} PDF validation passed with warnings:",
                        style("⚠").yellow().bold()
                    );
                    for warning in warnings {
                        println!("  • {}", style(warning).yellow());
                    }
                }
                fop_render::ValidationResult::Error(errors) => {
                    println!("\n{} PDF validation failed:", style("✗").red().bold());
                    for error in errors {
                        eprintln!("  • {}", style(error).red());
                    }
                }
            }
        }

        // JSON output format
        if cli.output_format == "json" {
            let json_result = match &validation_result {
                fop_render::ValidationResult::Valid => {
                    serde_json::json!({
                        "status": "valid",
                        "issues": []
                    })
                }
                fop_render::ValidationResult::Warning(warnings) => {
                    serde_json::json!({
                        "status": "warning",
                        "issues": warnings
                    })
                }
                fop_render::ValidationResult::Error(errors) => {
                    serde_json::json!({
                        "status": "error",
                        "issues": errors
                    })
                }
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&json_result)
                    .context("Failed to serialize validation result to JSON")?
            );
        }

        // Exit with error code if validation failed
        if validation_result.has_errors() || (cli.strict && !validation_result.is_ok()) {
            std::process::exit(1);
        }
    }

    // Phase 7: Render-verify — re-parse and rasterize the generated PDF as a self-check
    if cli.render_verify && matches!(output_format, cli::OutputFormat::Pdf) {
        progress.start_phase("Render-verifying PDF...");
        let phase_start = Instant::now();

        let dpi = cli.dpi.map(|d| d as f32).unwrap_or(72.0_f32);

        match fop_pdf_renderer::PdfRenderer::from_bytes(&final_bytes) {
            Ok(renderer) => {
                let page_count = renderer.page_count();
                if !cli.is_quiet() {
                    eprintln!("Render-verify: {} page(s)", page_count);
                }
                for i in 0..page_count {
                    match renderer.render_page(i, dpi) {
                        Ok(page) => {
                            if !cli.is_quiet() {
                                eprintln!("  Page {}: {}x{} px", i + 1, page.width, page.height);
                            }
                        }
                        Err(e) => {
                            eprintln!("render-verify: page {} rasterization failed: {}", i, e);
                            std::process::exit(1);
                        }
                    }
                }
                progress.finish_phase(
                    &format!("Render-verify complete ({} pages)", page_count),
                    phase_start.elapsed(),
                );
            }
            Err(e) => {
                eprintln!("render-verify: failed to parse generated PDF: {}", e);
                std::process::exit(1);
            }
        }
    }
    // Non-PDF: silently no-op

    // Final summary
    stats.total_duration = start_time.elapsed();

    if !cli.is_quiet() && !cli.is_stdout() {
        print_success_summary(&cli, &stats, output);
    }

    // Display statistics if requested
    if cli.stats {
        if cli.output_format == "json" {
            stats.display_json();
        } else {
            stats.display();
        }
    }

    Ok(())
}

/// Print detailed version information
fn print_version_info() {
    println!("Apache FOP (Formatting Objects Processor) Rust Implementation");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Edition: Rust 2021");
    println!();
    println!("Supported output formats:");
    println!("  PDF       (Portable Document Format)");
    println!("  SVG       (Scalable Vector Graphics)");
    println!("  PS        (PostScript)");
    println!("  PNG       (Portable Network Graphics)");
    println!("  JPEG      (Joint Photographic Experts Group)");
    println!("  TXT       (Plain Text)");
    println!();
    println!("Features:");
    println!("  - XSL-FO 1.1 compliance (29 elements, 294 properties)");
    println!("  - Knuth-Plass line breaking algorithm");
    println!("  - i18n support (16+ languages, automatic font fallback)");
    println!("  - PDF encryption (RC4-128, AES-256)");
    println!("  - Font embedding (Type 0 composite fonts, CIDFontType2)");
}

/// Print application banner
fn print_banner() {
    println!("{}", style("Apache FOP").cyan().bold());
    println!("{}", style("Rust Implementation").dim());
    println!();
}

/// Transform XML using XSLT stylesheet via xsltproc command
fn transform_with_xsltproc(
    xml_file: &Path,
    xsl_file: &Path,
    params: &[(&str, &str)],
) -> Result<Vec<u8>> {
    debug!("Using xsltproc for XSLT transformation");

    let mut cmd = Command::new("xsltproc");

    // Add parameters
    for (name, value) in params {
        cmd.arg("--stringparam");
        cmd.arg(name);
        cmd.arg(value);
    }

    // Add stylesheet and input file
    cmd.arg(xsl_file);
    cmd.arg(xml_file);

    // Execute the command
    let output = cmd
        .output()
        .with_context(|| "Failed to execute xsltproc. Please ensure xsltproc is installed.")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("xsltproc failed: {}", stderr);
    }

    Ok(output.stdout)
}

/// Perform XSLT transformation
fn perform_xslt_transform(
    cli: &Cli,
    progress: &mut ProgressReporter,
    stats: &mut ProcessingStats,
) -> Result<Vec<u8>> {
    progress.start_phase("Performing XSLT transformation...");
    let phase_start = Instant::now();

    let xml_file = cli
        .xml
        .as_ref()
        .context("XML input file not specified; use --xml <file>")?;
    let xsl_file = cli
        .xsl
        .as_ref()
        .context("XSL stylesheet not specified; use --xsl <file>")?;

    debug!("XML input: {}", xml_file.display());
    debug!("XSLT stylesheet: {}", xsl_file.display());

    // Get the XML file size for stats
    let xml_metadata = fs::metadata(xml_file)
        .with_context(|| format!("Failed to read XML file metadata: {}", xml_file.display()))?;
    stats.input_size = xml_metadata.len();

    // Validate XSLT file is readable
    fs::metadata(xsl_file)
        .with_context(|| format!("Failed to read XSLT file: {}", xsl_file.display()))?;

    // Log XSLT parameters
    let params = cli.get_xslt_params();
    if !params.is_empty() {
        debug!("XSLT parameters:");
        for (name, value) in &params {
            debug!("  {} = {}", name, value);
        }
    }

    // Perform XSLT transformation using xsltproc
    let transformed_data = transform_with_xsltproc(xml_file, xsl_file, &params)
        .with_context(|| "Failed to perform XSLT transformation")?;

    progress.finish_phase("XSLT transformation complete", phase_start.elapsed());

    Ok(transformed_data)
}

/// Print success summary
fn print_success_summary(cli: &Cli, stats: &ProcessingStats, output: &Path) {
    println!();
    println!(
        "{} {}",
        style("✓").green().bold(),
        style("Success!").green().bold()
    );
    println!();

    let input = cli
        .get_input()
        .expect("invariant: input path must be set before print_success_summary is called");
    println!("  Input:   {}", input.display());
    println!("  Output:  {}", output.display());
    println!(
        "  Size:    {} → {}",
        format_bytes(stats.input_size),
        format_bytes(stats.output_size)
    );
    println!("  Pages:   {}", stats.pages_generated);
    println!("  Time:    {}", format_duration(stats.total_duration));

    println!();
}

/// Format bytes in human-readable form
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    }
}

/// Format duration in human-readable form
fn format_duration(duration: std::time::Duration) -> String {
    let millis = duration.as_millis();
    if millis < 1000 {
        format!("{}ms", millis)
    } else if millis < 60_000 {
        format!("{:.2}s", duration.as_secs_f64())
    } else {
        let mins = millis / 60_000;
        let secs = (millis % 60_000) / 1000;
        format!("{}m {}s", mins, secs)
    }
}

/// Process an XSL-FO document string and return rendered output bytes.
///
/// This is a thin wrapper around the parse → layout → render pipeline that
/// makes the individual pipeline stages testable without invoking the full CLI.
pub fn process_document(fo_xml: &str, format: cli::OutputFormat) -> Result<Vec<u8>> {
    use fop_render::{PsRenderer, SvgRenderer, TextRenderer};
    use std::io::Cursor;

    let builder = FoTreeBuilder::new();
    let cursor = Cursor::new(fo_xml.as_bytes());
    let arena = builder.parse(cursor).context("Failed to parse XSL-FO")?;

    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&arena).context("Failed to layout")?;

    let bytes = match format {
        cli::OutputFormat::Svg => {
            let renderer = SvgRenderer::new();
            renderer
                .render_to_svg(&area_tree)
                .context("Failed to render SVG")?
                .into_bytes()
        }
        cli::OutputFormat::PostScript => {
            let renderer = PsRenderer::new();
            renderer
                .render_to_ps(&area_tree)
                .context("Failed to render PostScript")?
                .into_bytes()
        }
        cli::OutputFormat::Text => {
            let renderer = TextRenderer::new();
            renderer
                .render_to_text(&area_tree)
                .context("Failed to render text")?
                .into_bytes()
        }
        _ => {
            // Default: PDF
            let renderer = fop_render::PdfRenderer::new();
            let pdf_doc = renderer
                .render_with_fo(&area_tree, &arena)
                .context("Failed to render PDF")?;
            pdf_doc.to_bytes().context("Failed to serialize PDF")?
        }
    };

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // Utility function tests
    // ------------------------------------------------------------------

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2048), "2.00 KB");
        assert_eq!(format_bytes(1_572_864), "1.50 MB");
    }

    #[test]
    fn test_format_bytes_gb() {
        assert_eq!(format_bytes(1_073_741_824), "1.00 GB");
    }

    #[test]
    fn test_format_bytes_zero() {
        assert_eq!(format_bytes(0), "0 B");
    }

    #[test]
    fn test_format_duration() {
        use std::time::Duration;

        assert_eq!(format_duration(Duration::from_millis(500)), "500ms");
        assert_eq!(format_duration(Duration::from_millis(2500)), "2.50s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
    }

    #[test]
    fn test_format_duration_exact_seconds() {
        use std::time::Duration;
        assert_eq!(format_duration(Duration::from_secs(1)), "1.00s");
        assert_eq!(format_duration(Duration::from_millis(999)), "999ms");
    }

    #[test]
    fn test_format_duration_long() {
        use std::time::Duration;
        // 2 minutes 5 seconds
        assert_eq!(format_duration(Duration::from_secs(125)), "2m 5s");
    }

    // ------------------------------------------------------------------
    // process_document: PDF output
    // ------------------------------------------------------------------

    const MINIMAL_FO: &str = r#"<?xml version="1.0"?>
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
      <fo:block>Hello, FOP pipeline!</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    #[test]
    fn test_process_document_pdf() {
        let result = process_document(MINIMAL_FO, cli::OutputFormat::Pdf);
        assert!(result.is_ok(), "PDF render failed: {:?}", result.err());
        let bytes = result.expect("test: should succeed");
        assert!(!bytes.is_empty(), "PDF output must not be empty");
        assert!(
            bytes.starts_with(b"%PDF-"),
            "Output must start with PDF header"
        );
    }

    #[test]
    fn test_process_document_svg() {
        let result = process_document(MINIMAL_FO, cli::OutputFormat::Svg);
        assert!(result.is_ok(), "SVG render failed: {:?}", result.err());
        let bytes = result.expect("test: should succeed");
        assert!(!bytes.is_empty(), "SVG output must not be empty");
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            text.contains("<svg"),
            "SVG output must contain <svg element"
        );
    }

    #[test]
    fn test_process_document_text() {
        let result = process_document(MINIMAL_FO, cli::OutputFormat::Text);
        assert!(result.is_ok(), "Text render failed: {:?}", result.err());
        let bytes = result.expect("test: should succeed");
        assert!(!bytes.is_empty(), "Text output must not be empty");
        // The text output should contain our content
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            text.contains("Hello"),
            "Text output should contain document text"
        );
    }

    #[test]
    fn test_process_document_postscript() {
        let result = process_document(MINIMAL_FO, cli::OutputFormat::PostScript);
        assert!(
            result.is_ok(),
            "PostScript render failed: {:?}",
            result.err()
        );
        let bytes = result.expect("test: should succeed");
        assert!(!bytes.is_empty(), "PostScript output must not be empty");
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            text.contains("%!PS-Adobe"),
            "PostScript must start with PS header"
        );
    }

    #[test]
    fn test_process_document_invalid_xml() {
        let invalid_fo = "this is not XML at all <<<<";
        let result = process_document(invalid_fo, cli::OutputFormat::Pdf);
        assert!(result.is_err(), "Invalid XML should return an error");
    }

    #[test]
    fn test_process_document_multipage_pdf() {
        let fo = r#"<?xml version="1.0"?>
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
      <fo:block>Page one content.</fo:block>
      <fo:block break-before="page">Page two content.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;
        let result = process_document(fo, cli::OutputFormat::Pdf);
        assert!(result.is_ok(), "Multi-page PDF failed: {:?}", result.err());
        let bytes = result.expect("test: should succeed");
        assert!(bytes.starts_with(b"%PDF-"), "Must be valid PDF");
    }

    // ------------------------------------------------------------------
    // Pipe-mode: stdin → stdout simulation (process_document is the testable core)
    // ------------------------------------------------------------------

    #[test]
    fn test_pipe_mode_pdf_bytes_non_empty() {
        // Simulates: echo FO | fop - - (stdin → stdout)
        // We test this by directly exercising process_document since
        // actual stdin/stdout piping is covered by CLI integration tests.
        let result = process_document(MINIMAL_FO, cli::OutputFormat::Pdf);
        assert!(result.is_ok());
        let bytes = result.expect("test: should succeed");
        // Verify it looks like a real PDF (has xref and trailer)
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            text.contains("xref") || text.contains("%%EOF"),
            "PDF must have xref/EOF marker"
        );
    }
}
