//! Progress reporting for FOP CLI operations

use console::{style, Emoji};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::{Duration, Instant};

static CHECK_MARK: Emoji = Emoji("✓", "+");
#[allow(dead_code)]
static CROSS_MARK: Emoji = Emoji("✗", "x");
static SPARKLE: Emoji = Emoji("✨", "*");
static GEAR: Emoji = Emoji("⚙️ ", "");
static PAGE: Emoji = Emoji("📄", "");

/// Progress reporter for document processing
pub struct ProgressReporter {
    enabled: bool,
    spinner: Option<ProgressBar>,
    #[allow(dead_code)]
    start_time: Instant,
}

impl ProgressReporter {
    /// Create a new progress reporter
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            spinner: None,
            start_time: Instant::now(),
        }
    }

    /// Start a new phase of processing
    pub fn start_phase(&mut self, message: &str) {
        if !self.enabled {
            return;
        }

        if let Some(ref spinner) = self.spinner {
            spinner.finish_and_clear();
        }

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("Failed to set progress bar template"),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));

        self.spinner = Some(pb);
    }

    /// Update the current phase message
    pub fn update_message(&self, message: &str) {
        if let Some(ref spinner) = self.spinner {
            spinner.set_message(message.to_string());
        }
    }

    /// Finish the current phase successfully
    pub fn finish_phase(&mut self, message: &str, duration: Duration) {
        if !self.enabled {
            println!("{}", message);
            return;
        }

        if let Some(ref spinner) = self.spinner {
            spinner.finish_and_clear();
        }

        println!(
            "{} {} {}",
            style(CHECK_MARK).green().bold(),
            message,
            style(format!("({})", format_duration(duration))).dim()
        );

        self.spinner = None;
    }

    /// Finish the current phase with an error
    #[allow(dead_code)]
    pub fn finish_phase_error(&mut self, message: &str) {
        if !self.enabled {
            eprintln!("Error: {}", message);
            return;
        }

        if let Some(ref spinner) = self.spinner {
            spinner.finish_and_clear();
        }

        eprintln!("{} {}", style(CROSS_MARK).red().bold(), message);

        self.spinner = None;
    }

    /// Create a progress bar for a known number of items
    #[allow(dead_code)]
    pub fn create_bar(&self, total: u64, message: &str) -> Option<ProgressBar> {
        if !self.enabled {
            return None;
        }

        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg}\n[{bar:40.cyan/blue}] {percent}% ({pos}/{len} {unit})")
                .expect("Failed to set progress bar template")
                .progress_chars("=>-"),
        );
        pb.set_message(message.to_string());

        Some(pb)
    }

    /// Get elapsed time since reporter creation
    #[allow(dead_code)]
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Drop for ProgressReporter {
    fn drop(&mut self) {
        if let Some(ref spinner) = self.spinner {
            spinner.finish_and_clear();
        }
    }
}

/// Statistics tracker for document processing
#[derive(Debug, Default)]
pub struct ProcessingStats {
    pub nodes_parsed: usize,
    pub areas_created: usize,
    pub pages_generated: usize,
    pub warnings: usize,
    pub errors: usize,
    pub parse_duration: Duration,
    pub layout_duration: Duration,
    pub render_duration: Duration,
    pub total_duration: Duration,
    pub input_size: u64,
    pub output_size: u64,
}

impl ProcessingStats {
    /// Create a new statistics tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Display statistics in human-readable format
    pub fn display(&self) {
        println!("\n{} Statistics:", style("Processing").cyan().bold());
        println!();

        // Processing stages
        println!(
            "  {}Parsing:      {}",
            GEAR,
            format_duration(self.parse_duration)
        );
        println!(
            "  {}Layout:       {}",
            GEAR,
            format_duration(self.layout_duration)
        );
        println!(
            "  {}Rendering:    {}",
            GEAR,
            format_duration(self.render_duration)
        );
        println!(
            "  {}Total:        {}",
            SPARKLE,
            format_duration(self.total_duration)
        );
        println!();

        // Counts
        println!("  {}FO Nodes:     {}", style("•").dim(), self.nodes_parsed);
        println!("  {}Areas:        {}", style("•").dim(), self.areas_created);
        println!(
            "  {}Pages:        {}",
            PAGE,
            style(self.pages_generated).cyan().bold()
        );
        println!();

        // File sizes
        println!("  Input size:    {}", format_bytes(self.input_size));
        println!("  Output size:   {}", format_bytes(self.output_size));

        if self.input_size > 0 {
            let ratio = self.output_size as f64 / self.input_size as f64;
            println!("  Size ratio:    {:.2}x", ratio);
        }
        println!();

        // Issues
        if self.warnings > 0 || self.errors > 0 {
            if self.warnings > 0 {
                println!("  {}: {}", style("Warnings").yellow().bold(), self.warnings);
            }
            if self.errors > 0 {
                println!("  {}: {}", style("Errors").red().bold(), self.errors);
            }
            println!();
        }

        // Performance metrics
        if self.total_duration.as_secs_f64() > 0.0 {
            let pages_per_sec = self.pages_generated as f64 / self.total_duration.as_secs_f64();
            println!("  Throughput:    {:.2} pages/sec", pages_per_sec);
        }
    }

    /// Display statistics in JSON format
    pub fn display_json(&self) {
        let json = format!(
            r#"{{
  "parsing": {{
    "duration_ms": {},
    "nodes": {}
  }},
  "layout": {{
    "duration_ms": {},
    "areas": {}
  }},
  "rendering": {{
    "duration_ms": {},
    "pages": {}
  }},
  "total": {{
    "duration_ms": {},
    "input_bytes": {},
    "output_bytes": {},
    "warnings": {},
    "errors": {}
  }}
}}"#,
            self.parse_duration.as_millis(),
            self.nodes_parsed,
            self.layout_duration.as_millis(),
            self.areas_created,
            self.render_duration.as_millis(),
            self.pages_generated,
            self.total_duration.as_millis(),
            self.input_size,
            self.output_size,
            self.warnings,
            self.errors
        );
        println!("{}", json);
    }
}

/// Format a duration in human-readable form
fn format_duration(duration: Duration) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_millis(100)), "100ms");
        assert_eq!(format_duration(Duration::from_millis(1500)), "1.50s");
        assert_eq!(format_duration(Duration::from_millis(65000)), "1m 5s");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(100), "100 B");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1_572_864), "1.50 MB");
        assert_eq!(format_bytes(1_610_612_736), "1.50 GB");
    }

    #[test]
    fn test_stats_creation() {
        let stats = ProcessingStats::new();
        assert_eq!(stats.nodes_parsed, 0);
        assert_eq!(stats.pages_generated, 0);
    }
}
