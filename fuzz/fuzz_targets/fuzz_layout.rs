#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz the full pipeline: XML -> FO tree -> layout
    let _ = fop_layout::LayoutEngine::new().layout_fo(data);
});
