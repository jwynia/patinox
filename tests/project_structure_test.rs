// Integration test to verify project structure is set up correctly
// This test validates that the basic workspace structure works
use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn test_workspace_structure_exists() {
    // Test that essential project files exist
    assert!(
        Path::new("Cargo.toml").exists(),
        "Root Cargo.toml must exist"
    );
    assert!(
        Path::new("src/lib.rs").exists(),
        "Core library file must exist"
    );
    assert!(Path::new(".gitignore").exists(), ".gitignore must exist");
    assert!(
        Path::new("rust-toolchain.toml").exists(),
        "Rust toolchain config must exist"
    );
    assert!(Path::new("README.md").exists(), "README must exist");
}

#[test]
fn test_workspace_cargo_toml_structure() {
    let content = fs::read_to_string("Cargo.toml").expect("Should be able to read root Cargo.toml");

    // Verify workspace configuration
    assert!(
        content.contains("[workspace]"),
        "Must be configured as workspace"
    );
    assert!(
        content.contains("members = ["),
        "Must have workspace members"
    );
    assert!(
        content.contains("resolver = \"2\""),
        "Must use Cargo resolver v2"
    );
    
    // Verify workspace dependencies section exists
    assert!(
        content.contains("[workspace.dependencies]"),
        "Must have workspace dependencies section for version management"
    );
    
    // Verify workspace package metadata
    assert!(
        content.contains("[workspace.package]"),
        "Must have workspace package section for shared metadata"
    );
    assert!(
        content.contains("edition = \"2021\""),
        "Must specify Rust 2021 edition in workspace"
    );
}

#[test]
fn test_core_crate_cargo_toml_structure() {
    let content = fs::read_to_string("Cargo.toml").expect("Should be able to read Cargo.toml");

    // Verify basic package metadata
    assert!(content.contains("[package]"), "Must have package section");
    assert!(
        content.contains("name = \"patinox\""),
        "Must have correct package name"
    );
    assert!(
        content.contains("edition.workspace = true"),
        "Must inherit edition from workspace"
    );
    assert!(
        content.contains("version.workspace = true"), 
        "Must inherit version from workspace"
    );
    
    // Verify essential dependencies are present
    assert!(
        content.contains("thiserror.workspace = true"),
        "Must include thiserror for error handling"
    );
    assert!(
        content.contains("anyhow.workspace = true"),
        "Must include anyhow for error context"
    );
    
    // Verify package description exists
    assert!(
        content.contains("description = "),
        "Must have package description"
    );
}

#[test]
fn test_project_compiles() {
    // Test that the project compiles without errors
    let output = Command::new("cargo")
        .args(["check", "--workspace"])
        .output()
        .expect("Failed to execute cargo check");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("Cargo check failed:\n{}", stderr);
    }
}

#[test]
fn test_basic_library_structure() {
    let lib_content = fs::read_to_string("src/lib.rs").expect("Should be able to read src/lib.rs");

    // Verify basic lib.rs structure
    assert!(
        lib_content.contains("//!"),
        "Must have crate-level documentation"
    );
    assert!(!lib_content.is_empty(), "lib.rs should not be empty");
}

#[test]
fn test_development_dependencies_present() {
    let content = fs::read_to_string("Cargo.toml").expect("Should be able to read Cargo.toml");

    // Verify key development dependencies are configured
    assert!(
        content.contains("[dev-dependencies]"),
        "Must have dev-dependencies section"
    );
    
    // Verify essential testing dependencies are present
    assert!(
        content.contains("proptest"),
        "Must include proptest for property-based testing"
    );
    assert!(
        content.contains("criterion"),
        "Must include criterion for benchmarking"
    );
    assert!(
        content.contains("tokio-test"),
        "Must include tokio-test for async testing"
    );
}

#[test]
fn test_rust_toolchain_pinned() {
    let toolchain_content = fs::read_to_string("rust-toolchain.toml")
        .expect("Should be able to read rust-toolchain.toml");

    // Verify toolchain is pinned to stable
    assert!(
        toolchain_content.contains("channel = \"stable\""),
        "Must pin to stable toolchain"
    );
}

#[test]
fn test_gitignore_covers_rust_basics() {
    let gitignore_content =
        fs::read_to_string(".gitignore").expect("Should be able to read .gitignore");

    // Verify standard Rust ignores
    assert!(
        gitignore_content.contains("target/"),
        "Must ignore target directory"
    );
    assert!(
        gitignore_content.contains("Cargo.lock") || gitignore_content.contains("*.lock"),
        "Should handle Cargo.lock appropriately"
    );
}

#[test]
fn test_readme_has_basic_content() {
    let readme_content = fs::read_to_string("README.md").expect("Should be able to read README.md");

    // Verify README has basic project info
    assert!(
        readme_content.to_lowercase().contains("patinox"),
        "README must mention project name"
    );
    assert!(
        readme_content.contains("cargo"),
        "README must include build instructions"
    );
}

#[cfg(feature = "ci-tests")]
#[test]
fn test_github_actions_workflow_exists() {
    assert!(
        Path::new(".github/workflows/ci.yml").exists(),
        "CI workflow must exist"
    );
}

#[cfg(feature = "ci-tests")]
#[test]
fn test_ci_workflow_structure() {
    let ci_content =
        fs::read_to_string(".github/workflows/ci.yml").expect("Should be able to read CI workflow");

    // Verify basic CI structure
    assert!(ci_content.contains("on:"), "Must have trigger conditions");
    assert!(ci_content.contains("cargo test"), "Must run tests");
    assert!(
        ci_content.contains("cargo check") || ci_content.contains("cargo build"),
        "Must check/build code"
    );
}
