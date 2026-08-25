//! Acceptance checks for the package and facade boundary.

const MANIFEST: &str = include_str!("../Cargo.toml");
const FACADE: &str = include_str!("../src/lib.rs");

#[test]
fn core_manifest_and_facade_use_one_locked_no_std_dependency() {
    let dependencies = MANIFEST
        .lines()
        .skip_while(|line| *line != "[dependencies]")
        .skip(1)
        .take_while(|line| !line.starts_with('['))
        .filter(|line| !line.is_empty());
    assert!(
        dependencies.eq(["bytebudget = { version = \"=0.0.1-rc.1\", default-features = false }"])
    );
    assert!(FACADE.contains("#![no_std]"));
    assert!(FACADE.contains("#![forbid(unsafe_code)]"));

    let bytebudget_exports = FACADE
        .lines()
        .filter(|line| line.starts_with("pub use bytebudget"));
    assert!(bytebudget_exports.eq(["pub use bytebudget::{ByteCount, Retained};"]));

    for module in ["entropy", "plan", "script", "time", "timeline", "trace"] {
        assert!(FACADE.contains(&format!("pub mod {module};")));
    }
    assert!(!FACADE.contains("pub mod retained;"));
}

#[test]
fn package_identity_is_explicit() {
    assert!(MANIFEST.contains("name = \"criticality\""));
    assert!(MANIFEST.contains("version = \"0.0.1-rc.3\""));
    assert!(MANIFEST.contains("rust-version = \"1.88\""));
}
