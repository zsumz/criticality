//! Acceptance checks for the package and facade boundary.

const MANIFEST: &str = include_str!("../Cargo.toml");
const FACADE: &str = include_str!("../src/lib.rs");

#[test]
fn core_manifest_and_facade_preserve_initial_boundary() {
    let dependencies = MANIFEST
        .lines()
        .skip_while(|line| *line != "[dependencies]")
        .skip(1)
        .take_while(|line| !line.starts_with('['));
    assert!(dependencies.clone().all(str::is_empty));
    assert!(FACADE.contains("#![no_std]"));
    assert!(FACADE.contains("#![forbid(unsafe_code)]"));

    for module in [
        "entropy", "plan", "retained", "script", "time", "timeline", "trace",
    ] {
        assert!(FACADE.contains(&format!("pub mod {module};")));
    }
}

#[test]
fn package_identity_is_explicit() {
    assert!(MANIFEST.contains("name = \"criticality\""));
    assert!(MANIFEST.contains("version = \"0.0.1-rc.2\""));
    assert!(MANIFEST.contains("rust-version = \"1.88\""));
}
