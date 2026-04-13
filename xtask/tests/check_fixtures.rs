use std::path::Path;

use rust_toolkit_xtask::{checks, config};

fn fixture_root(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../fixtures/repos")
        .join(name)
        .canonicalize()
        .expect("fixture root")
}

fn fixture_config(repo_root: &Path) -> rust_toolkit_xtask::config::ToolkitConfig {
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

    let docs_index = checks::docs_index::run(&repo_root, &cfg).unwrap();
    assert!(docs_index
        .entries
        .iter()
        .any(|entry| entry.contains("does not match H1 title")));

    let semantic = checks::docs_semantic_drift::run(&repo_root, &cfg).unwrap();
    assert!(semantic
        .entries
        .iter()
        .any(|entry| entry.contains("unresolved path")
            || entry.contains("unresolved symbol")));

    let formal_claim = checks::formal_claim_scope::run(&repo_root, &cfg).unwrap();
    assert!(formal_claim
        .entries
        .iter()
        .any(|entry| entry.contains("missing required text")
            || entry.contains("forbidden text present")));

    let parity_ledger = checks::parity_ledger::run(&repo_root, &cfg).unwrap();
    assert!(parity_ledger
        .entries
        .iter()
        .any(|entry| entry.contains("Deviation Registry")
            || entry.contains("| ID | Status | Owner | Revisit | Summary |")));

    let durable = checks::durable_boundaries::run(&repo_root, &cfg).unwrap();
    assert!(durable
        .entries
        .iter()
        .any(|entry| entry.contains("PersistedDurabilityArtifact")
            || entry.contains("typed durable artifacts")));

    let search = checks::search_boundaries::run(&repo_root, &cfg).unwrap();
    assert!(search
        .entries
        .iter()
        .any(|entry| entry.contains("dioxus")
            || entry.contains("missing required pattern")));

    let viewer = checks::viewer_tooling_boundaries::run(&repo_root, &cfg).unwrap();
    assert!(viewer
        .entries
        .iter()
        .any(|entry| entry.contains("web_sys")
            || entry.contains("missing required pattern")));

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

    let text_formatting = checks::text_formatting::run(&repo_root, &cfg).unwrap();
    assert!(text_formatting
        .entries
        .iter()
        .any(|entry| entry.contains("forbidden emoji detected")));

    let workspace_hygiene = checks::workspace_hygiene::run(&repo_root, &cfg).unwrap();
    assert!(workspace_hygiene
        .entries
        .iter()
        .any(|entry| entry.contains("lonely mod.rs")));

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

    let unwrap = checks::unwrap_guard::run(&repo_root, &cfg).unwrap();
    assert!(unwrap
        .entries
        .iter()
        .any(|entry| entry.contains("unwrap") && entry.contains("rationale comment")));

    let allow_attr = checks::allow_attribute_guard::run(&repo_root, &cfg).unwrap();
    assert!(
        allow_attr
            .entries
            .iter()
            .any(|entry| entry.contains("#[allow(")
                && entry.contains("rationale comment"))
    );

    let doc_cov = checks::doc_coverage::run(&repo_root, &cfg).unwrap();
    assert!(doc_cov
        .entries
        .iter()
        .any(|entry| entry.contains("missing a doc comment")));

    let cloning = checks::cloning_boundary::run(&repo_root, &cfg).unwrap();
    assert!(cloning
        .entries
        .iter()
        .any(|entry| entry.contains("cloning trait")
            && entry.contains("rationale comment")));

    let fn_len = checks::fn_length::run(&repo_root, &cfg).unwrap();
    assert!(
        fn_len
            .entries
            .iter()
            .any(|entry| entry.contains("overly_long_function")
                && entry.contains("lines"))
    );
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
    assert!(checks::docs_index::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::docs_semantic_drift::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::formal_claim_scope::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::parity_ledger::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::durable_boundaries::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::search_boundaries::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::viewer_tooling_boundaries::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::workflow_actions::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::lean_escape_hatches::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::text_formatting::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::workspace_hygiene::run(&repo_root, &cfg)
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
    assert!(checks::unwrap_guard::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::allow_attribute_guard::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::doc_coverage::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::cloning_boundary::run(&repo_root, &cfg)
        .unwrap()
        .is_empty());
    assert!(checks::fn_length::run(&repo_root, &cfg).unwrap().is_empty());
}
