//! Tests for hot-reload functionality.

use super::*;

#[test]
fn test_shader_dir_exists() {
    let dir = shader_dir();
    assert!(dir.exists(), "shaders/ directory should exist at {:?}", dir);
    assert!(dir.is_dir());
}

#[test]
fn test_shader_source_loads_all() {
    let shaders = [
        "quad.wgsl",
        "text.wgsl",
        "rect_sdf.wgsl",
        "composite.wgsl",
        "blur.wgsl",
        "shadow.wgsl",
    ];
    for name in &shaders {
        let src = shader_source(name);
        assert!(!src.is_empty(), "Shader {} should not be empty", name);
        assert!(
            src.contains("fn"),
            "Shader {} should contain at least one fn",
            name
        );
    }
}

#[test]
fn test_poll_changes_empty_initially() {
    let watcher = ShaderWatcher::new(&shader_dir()).expect("watcher should start");
    assert!(
        watcher.poll_changes().is_none(),
        "No changes should be pending initially"
    );
}

#[test]
fn test_fallback_for_unknown_shader() {
    let src = fallback_shader("nonexistent.wgsl");
    assert!(src.is_empty());
}

#[test]
fn test_narrate_override_empty() {
    assert!(narrate_override("nonexistent.rs", 1).is_none());
}

#[test]
fn test_update_and_check_override() {
    let blocks = vec![(5, r#"div bg "blue" w 100 h 50"#.to_string())];
    update_narrate_overrides("test_file.rs", blocks);

    let el = narrate_override("test_file.rs", 5);
    assert!(el.is_some());

    // Clean up
    update_narrate_overrides("test_file.rs", vec![]);
}

#[test]
fn test_override_replaces_on_update() {
    let blocks1 = vec![(10, r#"div bg "red" w 50 h 50"#.to_string())];
    update_narrate_overrides("replace_test.rs", blocks1);
    assert!(narrate_override("replace_test.rs", 10).is_some());

    let blocks2 = vec![(10, r#"div bg "blue" w 100 h 100"#.to_string())];
    update_narrate_overrides("replace_test.rs", blocks2);
    assert!(narrate_override("replace_test.rs", 10).is_some());

    // Clean up
    update_narrate_overrides("replace_test.rs", vec![]);
}

#[test]
fn test_src_and_examples_dirs_exist() {
    let root = project_root();
    assert!(root.join("src").exists());
    assert!(root.join("examples").exists());
}

#[test]
fn test_path_matching_file_macro_vs_watcher() {
    // file!() returns the path relative to CARGO_MANIFEST_DIR
    let file_macro_path = file!();
    // Watcher delivers absolute paths. Simulate what process_narrate_file does:
    let root = project_root();
    let absolute = root.join(file_macro_path);
    let rel = absolute
        .strip_prefix(&root)
        .unwrap()
        .to_string_lossy()
        .to_string();
    assert_eq!(
        rel, file_macro_path,
        "strip_prefix(project_root) should produce the same string as file!()"
    );
}

#[test]
fn test_path_matching_roundtrip_override() {
    // Simulate: watcher detects change in this file, extracts block,
    // then narrate_resolve calls narrate_override with file!() + line
    let file_macro_path = file!();
    let line = 9999_u32;
    let dsl = r#"div bg "green""#.to_string();

    // Watcher path: absolute -> strip_prefix -> relative key
    let root = project_root();
    let abs_path = root.join(file_macro_path);
    let rel = abs_path
        .strip_prefix(&root)
        .unwrap()
        .to_string_lossy()
        .to_string();

    // Store via watcher-derived key
    update_narrate_overrides(&rel, vec![(line, dsl)]);

    // Lookup via file!()-derived key (what narrate_resolve does)
    let result = narrate_override(file_macro_path, line);
    assert!(
        result.is_some(),
        "Override lookup with file!() key must find the watcher-stored entry"
    );

    // Clean up
    update_narrate_overrides(&rel, vec![]);
}
