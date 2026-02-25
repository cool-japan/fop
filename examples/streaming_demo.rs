//! Demonstration of streaming layout and rendering for large documents
//!
//! This example shows how to use the streaming API to process large documents
//! efficiently with bounded memory usage.

use fop_core::{FoArena, FoNode, FoNodeData, PropertyList, PropertyValue};
use fop_layout::layout::{StreamingConfig, StreamingLayoutEngine};
use fop_render::pdf::StreamingPdfRenderer;
use fop_types::Length;
use std::fs::File;
use std::io::BufWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Optional: env_logger::init();

    println!("=== Streaming Layout and Rendering Demo ===\n");

    // Example 1: Basic streaming with default configuration
    basic_streaming_example()?;

    // Example 2: Custom configuration for very large documents
    custom_config_example()?;

    // Example 3: Comparison of memory usage between modes
    memory_comparison_example()?;

    println!("\n=== All examples completed successfully ===");
    Ok(())
}

/// Basic streaming example with 50 pages
fn basic_streaming_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Basic Streaming Example (50 pages)");

    // Create FO tree with 50 pages
    let fo_tree = create_fo_tree(50);

    // Create streaming layout engine
    let layout_engine = StreamingLayoutEngine::new();

    // Create streaming PDF renderer
    let mut pdf_renderer = StreamingPdfRenderer::new();
    pdf_renderer.set_title("Streaming Demo - 50 Pages".to_string());
    pdf_renderer.set_author("FOP Streaming Engine".to_string());

    // Process pages one at a time
    println!("   Processing pages...");
    for (index, page_result) in layout_engine.layout_streaming(&fo_tree).enumerate() {
        let page = page_result?;
        pdf_renderer.add_page(&page)?;

        if (index + 1) % 10 == 0 {
            println!("   - Processed {} pages", index + 1);
        }
        // page is dropped here, freeing memory
    }

    println!("   Total pages: {}", pdf_renderer.page_count());

    // Write to file
    let output_path = "/tmp/streaming_basic.pdf";
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);
    pdf_renderer.write_to(&mut writer)?;

    println!("   Output written to: {}\n", output_path);
    Ok(())
}

/// Example with custom configuration for very large documents
fn custom_config_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. Custom Configuration Example (1000 pages)");

    // Create FO tree with 1000 pages (simulating a large document)
    let fo_tree = create_fo_tree(1000);

    // Create streaming layout engine with small buffer
    let config = StreamingConfig {
        max_memory_pages: 5, // Only buffer 5 pages
        page_width: Length::from_mm(210.0),
        page_height: Length::from_mm(297.0),
    };
    let layout_engine = StreamingLayoutEngine::with_config(config);

    // Create streaming PDF renderer
    let mut pdf_renderer = StreamingPdfRenderer::new();
    pdf_renderer.set_title("Large Document - 1000 Pages".to_string());

    // Process pages
    println!("   Processing large document...");
    let mut page_count = 0;
    for page_result in layout_engine.layout_streaming(&fo_tree) {
        let page = page_result?;
        pdf_renderer.add_page(&page)?;
        page_count += 1;

        if page_count % 100 == 0 {
            println!("   - Processed {} pages", page_count);
        }
    }

    println!("   Total pages: {}", pdf_renderer.page_count());

    // Write to file
    let output_path = "/tmp/streaming_large.pdf";
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);
    pdf_renderer.write_to(&mut writer)?;

    println!("   Output written to: {}\n", output_path);
    Ok(())
}

/// Compare memory usage between normal and streaming mode
fn memory_comparison_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. Memory Comparison Example");

    let page_counts = vec![10, 50, 100];

    for &count in &page_counts {
        println!("   Document size: {} pages", count);

        // Estimate memory for normal mode: O(n) where n = total pages
        // Each page has approximately 1 page area + 1 region + 1 block + 1 text = ~4 areas
        // Each area ~500 bytes (geometry, traits, content)
        let normal_memory_kb = (count * 4 * 500) / 1024;
        println!("     - Normal mode (estimated): ~{} KB", normal_memory_kb);

        // Estimate memory for streaming mode: O(k) where k = buffer size
        // With buffer size of 10 pages
        let streaming_memory_kb = (10 * 4 * 500) / 1024;
        println!("     - Streaming mode (estimated): ~{} KB", streaming_memory_kb);

        let savings_percent =
            ((normal_memory_kb - streaming_memory_kb) as f64 / normal_memory_kb as f64) * 100.0;

        if count > 10 {
            println!("     - Memory savings: {:.1}%", savings_percent);
        } else {
            println!("     - Memory savings: minimal (document too small)");
        }
        println!();
    }

    // Extrapolate to very large document
    println!("   Document size: 5000 pages");
    let normal_memory_mb = (5000 * 4 * 500) / (1024 * 1024);
    let streaming_memory_mb = (10 * 4 * 500) / (1024 * 1024);
    println!("     - Normal mode (estimated): ~{} MB", normal_memory_mb);
    println!("     - Streaming mode (estimated): <{} MB", streaming_memory_mb.max(1));
    println!("     - Memory savings: ~{}x reduction\n", normal_memory_mb / streaming_memory_mb.max(1));

    Ok(())
}

/// Create an FO tree with the specified number of pages
fn create_fo_tree(page_count: usize) -> FoArena<'static> {
    let mut fo_tree = FoArena::new();

    // Create root
    let root = fo_tree.add_node(FoNode::new(FoNodeData::Root));

    // Create layout-master-set
    let master_set = fo_tree.add_node(FoNode::new(FoNodeData::LayoutMasterSet));
    fo_tree.append_child(root, master_set).expect("bench/example: should succeed");

    // Create page sequences (one per page)
    for i in 0..page_count {
        let mut properties = PropertyList::new();
        properties.set(
            fop_core::PropertyId::BackgroundColor,
            PropertyValue::Color(fop_types::Color::WHITE),
        );

        let page_seq = fo_tree.add_node(FoNode::new(FoNodeData::PageSequence {
            master_reference: format!("page-{}", i),
            properties,
        }));
        fo_tree.append_child(root, page_seq).expect("bench/example: should succeed");

        // Create flow
        let flow = fo_tree.add_node(FoNode::new(FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: PropertyList::new(),
        }));
        fo_tree.append_child(page_seq, flow).expect("bench/example: should succeed");

        // Add a block with content
        let mut block_props = PropertyList::new();
        block_props.set(
            fop_core::PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(12.0)),
        );

        let block = fo_tree.add_node(FoNode::new(FoNodeData::Block {
            properties: block_props,
        }));
        fo_tree.append_child(flow, block).expect("bench/example: should succeed");

        // Add text content
        let text = fo_tree.add_node(FoNode::new(FoNodeData::Text(format!(
            "This is page {} of the streaming demo document.",
            i + 1
        ))));
        fo_tree.append_child(block, text).expect("bench/example: should succeed");
    }

    fo_tree
}
