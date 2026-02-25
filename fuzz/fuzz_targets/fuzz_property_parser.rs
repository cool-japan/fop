#![no_main]

use libfuzzer_sys::fuzz_target;
use fop_core::{Length, Color};
use std::str;

fuzz_target!(|data: &[u8]| {
    // Try to parse as UTF-8 string
    if let Ok(s) = str::from_utf8(data) {
        if s.len() > 1000 {
            return;
        }

        // Try parsing as length
        let _ = s.parse::<f64>().ok().map(Length::from_pt);

        // Try parsing as color
        if s.starts_with('#') {
            let _ = Color::from_hex(s);
        }
    }
});
