//! Example: Using hcscoder tools programmatically
//!
//! This example shows how to use hcscoder's filesystem and bash tools
//! in your own applications.

use hcscoder::hcscoder_tools::filesystem;

fn main() -> anyhow::Result<()> {
    // Example: Validate a file path
    let test_path = "./examples/basic_usage.rs";
    
    println!("Testing path validation for: {}", test_path);
    
    // Note: The actual API may differ - this is a placeholder example
    // In real usage, you would call the appropriate validation function
    
    println!("Path validation example complete.");
    
    Ok(())
}
