use std::path::Path;

use jacquard_toolkit_xtask::{checks, config};

fn fixture_root(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../fixtures/repos")
        .join(name)
        .canonicalize()
        .expect("fixture root")
}

fn fixture_config(repo_root: &Path) -> jacquard_toolkit_xtask::config::ToolkitConfig {
    config::load(&repo_root.join("toolkit.toml")).expect("fixture config")
}

#[test]
// long-block-exception: fixture assertions intentionally keep the failure
// surface in one test
fn fail_fixture_reports_expected_findings() {
    let repo_root = fixture_root("fail_repo");
    let cfg = fixture_config(&repo_root);

    let proc_macro = checks::proc_macro_scope::run(&repo_root, &cfg).unwrap();
    assert!(proc_macro.missing.contains("crates/traits/src/lib.rs"));

    let result_must_use = checks::result_must_use::run(&repo_root, &cfg).unwrap();
    assert!(result_must_use.entries.iter().any(|entry| entry.contains(
        "trait SampleTrait method compute returns Result without #[must_use]"
    )));

    let test_boundaries = checks::test_boundaries::run(&repo_root, &cfg).unwrap();
    assert!(test_boundaries.entries.iter().any(|entry| entry
        .contains("standalone unit-test source files under src/ are forbidden")));

    let docs_link = checks::docs_link_check::run(&repo_root, &cfg).unwrap();
    assert!(docs_link
        .entries
        .iter()
        .any(|entry| entry.contains("missing docs link")));

    let semantic = checks::docs_semantic_drift::run(&repo_root, &cfg).unwrap();
    assert!(semantic
        .entries
        .iter()
        .any(|entry| entry.contains("unresolved path")
            || entry.contains("unresolved symbol")));

    let workflows = checks::workflow_actions::run(&repo_root, &cfg).unwrap();
    assert!(workflows
        .entries
        .iter()
        .any(|entry| entry.contains("unresolved GitHub Action reference")));

    let escape_hatches = checks::lean_escape_hatches::run(&repo_root, &cfg).unwrap();
    assert!(escape_hatches
        .entries
        .iter()
        .any(|entry| entry.contains("lean escape hatch `sorry`")));

    let lean_architecture = checks::lean_architecture::run(&repo_root, &cfg).unwrap();
    assert!(lean_architecture.entries.iter().any(|entry| {
        entry.contains("placeholder contract `Prop := True`")
            || entry.contains("root facade imports debug/example/test modules")
    }));

    let text_formatting = checks::text_formatting::run(&repo_root, &cfg).unwrap();
    assert!(text_formatting
        .entries
        .iter()
        .any(|entry| entry.contains("forbidden emoji detected")));

    let docs_prose_quality = checks::docs_prose_quality::run(&repo_root, &cfg).unwrap();
    assert!(docs_prose_quality.entries.iter().any(|entry| {
        entry.contains("semicolon is not allowed")
            || entry.contains("code block must be followed by an explanatory paragraph")
            || entry.contains("explanatory text after code block must be prose")
    }));

    let workspace_hygiene = checks::workspace_hygiene::run(&repo_root, &cfg).unwrap();
    assert!(workspace_hygiene
        .entries
        .iter()
        .any(|entry| entry.contains("lonely mod.rs")));

    let workspace_layering = checks::workspace_layering::run(&repo_root, &cfg).unwrap();
    assert!(workspace_layering.entries.iter().any(|entry| {
        entry.contains("depends on higher-layer crate")
            || entry.contains("missing from `checks.workspace_layering.crate_layers`")
    }));

    let crate_root = checks::crate_root_policy::run(&repo_root, &cfg).unwrap();
    assert!(crate_root
        .entries
        .iter()
        .any(|entry| entry.contains("missing crate-root policy attribute")));

    let ignored = checks::ignored_result::run(&repo_root, &cfg).unwrap();
    assert!(ignored
        .entries
        .iter()
        .any(|entry| entry.contains("ignored result-like value")));

    let unsafe_boundary = checks::unsafe_boundary::run(&repo_root, &cfg).unwrap();
    assert!(unsafe_boundary
        .entries
        .iter()
        .any(|entry| entry.contains("unsafe")));

    let rust_architecture = checks::rust_architecture::run(&repo_root, &cfg).unwrap();
    assert!(rust_architecture.entries.iter().any(|entry| {
        entry.contains("raw `fixed::` usage is forbidden")
            || entry.contains("float-typed public config/schema field is forbidden")
            || entry.contains("FixedQ32 must not accept float-token decoding")
            || entry.contains("direct thread scheduling and timer calls are forbidden")
    }));

    let bool_param = checks::bool_param::run(&repo_root, &cfg).unwrap();
    assert!(bool_param
        .entries
        .iter()
        .any(|entry| entry.contains("bool parameter")));

    let must_use = checks::must_use_public_return::run(&repo_root, &cfg).unwrap();
    assert!(must_use
        .entries
        .iter()
        .any(|entry| entry.contains("meaningful value without #[must_use]")));

    let assert_shape = checks::assert_shape::run(&repo_root, &cfg).unwrap();
    assert!(assert_shape
        .entries
        .iter()
        .any(|entry| entry.contains("compound assert")));

    let drop_effects = checks::drop_side_effects::run(&repo_root, &cfg).unwrap();
    assert!(drop_effects
        .entries
        .iter()
        .any(|entry| entry.contains("Drop implementation")));

    let recursion = checks::recursion_guard::run(&repo_root, &cfg).unwrap();
    assert!(recursion.is_empty());

    let naming = checks::naming_units::run(&repo_root, &cfg).unwrap();
    assert!(naming
        .entries
        .iter()
        .any(|entry| entry.contains("time quantity") || entry.contains("bare `size`")));

    let limits = checks::limit_constant::run(&repo_root, &cfg).unwrap();
    assert!(limits
        .entries
        .iter()
        .any(|entry| entry.contains("named constant")));

    let widths = checks::public_type_width::run(&repo_root, &cfg).unwrap();
    assert!(widths
        .entries
        .iter()
        .any(|entry| entry.contains("banned public type")
            || entry.contains("banned parameter type")
            || entry.contains("banned return type")));

    let deps = checks::dependency_policy::run(&repo_root, &cfg).unwrap();
    assert!(deps
        .entries
        .iter()
        .any(|entry| entry.contains("default-features = false")
            || entry.contains("banned by toolkit dependency policy")));
}

#[test]
fn pass_fixture_reports_no_findings() {
    let repo_root = fixture_root("pass_repo");
    let cfg = fixture_config(&repo_root);

    assert!(checks::proc_macro_scope::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::result_must_use::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::test_boundaries::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::docs_link_check::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::docs_semantic_drift::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::workflow_actions::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::lean_escape_hatches::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::lean_architecture::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::text_formatting::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::docs_prose_quality::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::workspace_hygiene::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::workspace_layering::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::crate_root_policy::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::ignored_result::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::unsafe_boundary::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::rust_architecture::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::bool_param::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::must_use_public_return::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::assert_shape::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::drop_side_effects::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::recursion_guard::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::naming_units::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::limit_constant::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::public_type_width::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::dependency_policy::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
}
