#![no_main]

use libfuzzer_sys::fuzz_target;
use fop_core::FoTreeBuilder;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // Skip obviously invalid inputs
    if data.len() < 10 || data.len() > 1_000_000 {
        return;
    }

    // Try to parse the data as XML
    let builder = FoTreeBuilder::new();
    let cursor = Cursor::new(data);

    // We don't care if it fails, just that it doesn't panic
    let _ = builder.parse(cursor);
});
