//! fop-render-pdf — CLI tool to render PDF pages to PNG images
//!
//! Usage:
//!   fop-render-pdf input.pdf output.png         # render page 0
//!   fop-render-pdf input.pdf output.png --page 2
//!   fop-render-pdf input.pdf output.png --dpi 300
//!   fop-render-pdf input.pdf output-%d.png --all  # all pages

use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 || args.iter().any(|a| a == "--help" || a == "-h") {
        eprintln!("Usage: fop-render-pdf <input.pdf> <output.png> [OPTIONS]");
        eprintln!();
        eprintln!("Options:");
        eprintln!("  --page N     Page to render (0-indexed, default: 0)");
        eprintln!("  --dpi N      Output resolution in DPI (default: 150)");
        eprintln!("  --all        Render all pages (output must contain %d, e.g. page-%d.png)");
        process::exit(0);
    }

    let input = &args[1];
    let output = &args[2];

    let mut page: usize = 0;
    let mut dpi: f32 = 150.0;
    let mut all_pages = false;

    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--page" => {
                i += 1;
                page = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(0);
            }
            "--dpi" => {
                i += 1;
                dpi = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(150.0);
            }
            "--all" => {
                all_pages = true;
            }
            _ => {}
        }
        i += 1;
    }

    // Load PDF
    let pdf_data = match std::fs::read(input) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error reading {}: {}", input, e);
            process::exit(1);
        }
    };

    let renderer = match fop_pdf_renderer::PdfRenderer::from_bytes(&pdf_data) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error parsing PDF: {}", e);
            process::exit(1);
        }
    };

    let page_count = renderer.page_count();
    eprintln!("PDF has {} page(s)", page_count);

    if all_pages {
        // Render all pages: output must contain %d
        if !output.contains("%d") {
            eprintln!("Error: --all requires output path containing '%d' (e.g. page-%d.png)");
            process::exit(1);
        }
        for p in 0..page_count {
            let path = output.replace("%d", &p.to_string());
            eprintln!("Rendering page {} → {}", p, path);
            match renderer.save_as_png(p, &path, dpi) {
                Ok(()) => eprintln!("  Saved: {}", path),
                Err(e) => {
                    eprintln!("  Error rendering page {}: {}", p, e);
                    process::exit(1);
                }
            }
        }
    } else {
        // Render single page
        if page >= page_count {
            eprintln!(
                "Error: page {} out of range (document has {} pages)",
                page, page_count
            );
            process::exit(1);
        }
        eprintln!("Rendering page {} at {} DPI → {}", page, dpi, output);
        match renderer.save_as_png(page, output, dpi) {
            Ok(()) => eprintln!("Saved: {}", output),
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    }
}
