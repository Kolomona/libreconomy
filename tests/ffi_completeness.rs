//! FFI Completeness Test
//!
//! This test ensures that all public Rust API functions are properly exported to the C FFI layer
//! and that the generated C header is in sync with the FFI exports.
//!
//! This prevents situations where:
//! - A new public Rust function is added but not exported to FFI
//! - FFI exports exist but are missing from the generated header
//! - The bridge layer can't access needed functionality

use std::collections::{HashSet, HashMap};
use std::fs;
use std::path::Path;

/// Public Rust API functions that should be exported to FFI
/// Format: (module_path, function_name, ffi_function_name)
const EXPECTED_API_EXPORTS: &[(&str, &str, &str)] = &[
    // Agent creation functions
    ("agent::creation", "create_agent", "create_agent_default"),
    ("agent::creation", "create_agent_with_needs", "create_agent_with_needs"),
    ("agent::creation", "create_agent_with_wallet", "create_agent_with_wallet"),
    ("agent::creation", "create_agent_custom", "create_agent_full"),
    ("agent::creation", "remove_agent", "remove_agent"),
    
    // World management (FFI-specific)
    ("ffi", "create_world", "create_world"),
    ("ffi", "destroy_world", "destroy_world"),
    
    // Utility functions
    ("ffi", "get_agent_count", "get_agent_count"),
];

/// Core FFI functions that must always be present
const REQUIRED_FFI_FUNCTIONS: &[&str] = &[
    "create_world",
    "destroy_world",
    "create_agent_default",
    "create_agent_with_needs",
    "create_agent_with_wallet",
    "create_agent_full",
    "remove_agent",
    "get_agent_count",
];

#[test]
fn test_all_expected_functions_exported_in_ffi_module() {
    let ffi_source = fs::read_to_string("src/ffi/mod.rs")
        .expect("Failed to read src/ffi/mod.rs");
    
    let mut missing_exports = Vec::new();
    
    for (_module, _rust_name, ffi_name) in EXPECTED_API_EXPORTS {
        // Check if the FFI function is defined with #[no_mangle]
        // Accept both "pub extern "C" fn" and "pub unsafe extern "C" fn"
        let pattern1 = format!("pub extern \"C\" fn {}", ffi_name);
        let pattern2 = format!("pub unsafe extern \"C\" fn {}", ffi_name);
        if !ffi_source.contains(&pattern1) && !ffi_source.contains(&pattern2) {
            missing_exports.push(ffi_name.to_string());
        }
    }
    
    assert!(
        missing_exports.is_empty(),
        "Missing FFI exports in src/ffi/mod.rs:\n  - {}",
        missing_exports.join("\n  - ")
    );
}

#[test]
fn test_all_required_functions_in_generated_header() {
    let header_path = "libreconomy.h";
    
    if !Path::new(header_path).exists() {
        panic!(
            "libreconomy.h not found. Run 'cargo build' to generate it via cbindgen.\n\
             This test requires the C header to be generated first."
        );
    }
    
    let header_content = fs::read_to_string(header_path)
        .expect("Failed to read libreconomy.h");
    
    let mut missing_in_header = Vec::new();
    
    for function_name in REQUIRED_FFI_FUNCTIONS {
        // Look for function declaration in header
        // Format: return_type function_name(args);
        if !header_content.contains(function_name) {
            missing_in_header.push(function_name.to_string());
        }
    }
    
    assert!(
        missing_in_header.is_empty(),
        "Required FFI functions missing from libreconomy.h:\n  - {}\n\n\
         This suggests cbindgen is not picking up these exports.\n\
         Check that functions have #[no_mangle] and pub extern \"C\".",
        missing_in_header.join("\n  - ")
    );
}

#[test]
fn test_ffi_functions_match_rust_public_api() {
    // This test ensures we haven't forgotten to expose any public creation functions
    let creation_source = fs::read_to_string("src/agent/creation.rs")
        .expect("Failed to read src/agent/creation.rs");
    
    // Extract public function names from creation.rs
    let public_functions: Vec<&str> = creation_source
        .lines()
        .filter(|line| line.trim().starts_with("pub fn"))
        .filter_map(|line| {
            // Extract function name: "pub fn function_name(...)"
            line.split("pub fn ")
                .nth(1)
                .and_then(|s| s.split('(').next())
                .map(|s| s.trim())
        })
        .collect();
    
    // Map expected Rust functions to their FFI counterparts
    let expected_mapping: HashMap<&str, &str> = EXPECTED_API_EXPORTS
        .iter()
        .filter(|(module, _, _)| *module == "agent::creation")
        .map(|(_, rust_name, ffi_name)| (*rust_name, *ffi_name))
        .collect();
    
    let mut unmapped_functions = Vec::new();
    
    for func_name in &public_functions {
        // Skip test helper functions
        if func_name.starts_with("test_") || func_name.contains("_test_") {
            continue;
        }
        
        // Check if this function has an FFI export mapping
        if !expected_mapping.contains_key(func_name) {
            unmapped_functions.push(func_name.to_string());
        }
    }
    
    assert!(
        unmapped_functions.is_empty(),
        "Public Rust functions without FFI exports:\n  - {}\n\n\
         These functions are part of the public API but not exposed via FFI.\n\
         Add corresponding FFI exports in src/ffi/mod.rs and update EXPECTED_API_EXPORTS.",
        unmapped_functions.join("\n  - ")
    );
}

#[test]
fn test_bridge_cpp_uses_all_ffi_functions() {
    let bridge_path = "dist/godot4/bridge/libreconomy_bridge.cpp";
    
    if !Path::new(bridge_path).exists() {
        eprintln!(
            "Warning: {} not found. Skipping bridge completeness test.\n\
             Run './scripts/release.sh' to generate the bridge.",
            bridge_path
        );
        return;
    }
    
    let bridge_content = fs::read_to_string(bridge_path)
        .expect("Failed to read libreconomy_bridge.cpp");
    
    // Skip world lifecycle functions (create_world/destroy_world are in constructor/destructor)
    let functions_to_check: Vec<&str> = REQUIRED_FFI_FUNCTIONS
        .iter()
        .filter(|&f| *f != "create_world" && *f != "destroy_world")
        .copied()
        .collect();
    
    let mut unused_functions = Vec::new();
    
    for function_name in functions_to_check {
        // Check if function is called in bridge (with :: prefix or direct call)
        let patterns = [
            format!("::{}", function_name),
            format!("{}(", function_name),
        ];
        
        let is_used = patterns.iter().any(|pattern| bridge_content.contains(pattern));
        
        if !is_used {
            unused_functions.push(function_name.to_string());
        }
    }
    
    assert!(
        unused_functions.is_empty(),
        "FFI functions not used in libreconomy_bridge.cpp:\n  - {}\n\n\
         These functions are exported in the FFI but not exposed in the Godot bridge.\n\
         Add corresponding methods to the LibreWorld class.",
        unused_functions.join("\n  - ")
    );
}

#[test]
fn test_no_extra_ffi_exports() {
    // This test ensures we don't have FFI exports that aren't documented in EXPECTED_API_EXPORTS
    let ffi_source = fs::read_to_string("src/ffi/mod.rs")
        .expect("Failed to read src/ffi/mod.rs");
    
    // Extract all #[no_mangle] pub extern "C" fn declarations
    let mut declared_ffi_functions = HashSet::new();
    let lines: Vec<&str> = ffi_source.lines().collect();
    
    for i in 0..lines.len() {
        if lines[i].contains("#[no_mangle]") {
            // Next non-empty line should have the function declaration
            for j in (i + 1)..lines.len().min(i + 5) {
                let line = lines[j].trim();
                if line.starts_with("pub extern \"C\" fn") || line.starts_with("pub unsafe extern \"C\" fn") {
                    if let Some(func_name) = line
                        .split("fn ")
                        .nth(1)
                        .and_then(|s| s.split('(').next())
                    {
                        declared_ffi_functions.insert(func_name.trim().to_string());
                    }
                    break;
                }
            }
        }
    }
    
    // Get expected function names
    let expected_functions: HashSet<String> = EXPECTED_API_EXPORTS
        .iter()
        .map(|(_, _, ffi_name)| ffi_name.to_string())
        .collect();
    
    // Also allow libreconomy_version (utility function)
    let mut allowed_functions = expected_functions.clone();
    allowed_functions.insert("libreconomy_version".to_string());
    
    let undocumented: Vec<_> = declared_ffi_functions
        .iter()
        .filter(|f| !allowed_functions.contains(*f))
        .collect();
    
    assert!(
        undocumented.is_empty(),
        "Undocumented FFI exports in src/ffi/mod.rs:\n  - {}\n\n\
         These functions are exported but not listed in EXPECTED_API_EXPORTS.\n\
         Add them to the test's EXPECTED_API_EXPORTS constant.",
        undocumented.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n  - ")
    );
}
