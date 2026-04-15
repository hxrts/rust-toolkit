use std::{
    env,
    path::{Path, PathBuf},
    process,
};

use anyhow::{bail, Result};
use report::FindingSet;
use rust_toolkit_xtask::{checks, config, legacy, report};

const DEFAULT_CONFIG: &str = "toolkit/toolkit.toml";

fn toolkit_root() -> Result<PathBuf> {
    if let Some(root) = env::var_os("TOOLKIT_ROOT") {
        return canonicalize_existing(Path::new(&root));
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    canonicalize_existing(
        manifest_dir
            .parent()
            .ok_or_else(|| anyhow::anyhow!("xtask manifest has no parent directory"))?,
    )
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err:#}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        | Some("--help") | Some("-h") | Some("help") => {
            println!("toolkit-xtask: usage: check|parity|show-config|fmt|fmt-check|clippy|dylint");
            Ok(())
        },
        | Some("check") => run_check(&args[1..]),
        | Some("parity") => run_parity(&args[1..]),
        | Some("show-config") => show_config(&args[1..]),
        | Some("fmt") => run_fmt(&args[1..]),
        | Some("fmt-check") => run_fmt_check(&args[1..]),
        | Some("clippy") => run_clippy(&args[1..]),
        | Some("dylint") => run_dylint(&args[1..]),
        | Some(other) => bail!("toolkit-xtask: unknown command: {other}"),
        | None => {
            bail!(
                "toolkit-xtask: usage: check|parity|show-config|fmt|fmt-check|clippy|dylint"
            )
        },
    }
}

// long-block-exception: command dispatch keeps the check surface explicit at
// one callsite
fn run_check(args: &[String]) -> Result<()> {
    let (name, repo_root, config_path) = parse_check_args(args)?;
    let config = config::load(&config_path)?;
    let normalized_name = name.replace('_', "-");
    let findings = match normalized_name.as_str() {
        | "proc-macro-scope" => checks::proc_macro_scope::run(&repo_root, &config)?,
        | "crate-root-policy" => {
            let findings = checks::crate_root_policy::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "ignored-result" => {
            let findings = checks::ignored_result::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "unsafe-boundary" => {
            let findings = checks::unsafe_boundary::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "bool-param" => {
            let findings = checks::bool_param::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "must-use-public-return" => {
            let findings = checks::must_use_public_return::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "assert-shape" => {
            let findings = checks::assert_shape::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "drop-side-effects" => {
            let findings = checks::drop_side_effects::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "recursion-guard" => {
            let findings = checks::recursion_guard::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "naming-units" => {
            let findings = checks::naming_units::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "limit-constant" => {
            let findings = checks::limit_constant::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "public-type-width" => {
            let findings = checks::public_type_width::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "dependency-policy" => {
            let findings = checks::dependency_policy::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "result-must-use" => {
            let findings = checks::result_must_use::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "test-boundaries" => {
            let findings = checks::test_boundaries::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "lean-style" => {
            let findings = checks::lean_style::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "docs-link-check" => {
            let findings = checks::docs_link_check::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "docs-index" => {
            let findings = checks::docs_index::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "docs-semantic-drift" => {
            let findings = checks::docs_semantic_drift::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "ci-assurance-lanes" => {
            let findings = checks::ci_assurance_lanes::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "formal-claim-scope" => {
            let findings = checks::formal_claim_scope::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "parity-ledger" => {
            let findings = checks::parity_ledger::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "durable-boundaries" => {
            let findings = checks::durable_boundaries::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "protocol-machine-placeholders" => {
            let findings =
                checks::protocol_machine_placeholders::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "search-boundaries" => {
            let findings = checks::search_boundaries::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "viewer-tooling-boundaries" => {
            let findings = checks::viewer_tooling_boundaries::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "git-dependency-pins" => {
            let findings = checks::git_dependency_pins::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "workflow-actions" => {
            let findings = checks::workflow_actions::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "lean-escape-hatches" => {
            let findings = checks::lean_escape_hatches::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "text-formatting" => {
            let findings = checks::text_formatting::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "workspace-hygiene" => {
            let findings = checks::workspace_hygiene::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "unwrap-guard" => {
            let findings = checks::unwrap_guard::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "allow-attribute-guard" => {
            let findings = checks::allow_attribute_guard::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "doc-coverage" => {
            let findings = checks::doc_coverage::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "cloning-boundary" => {
            let findings = checks::cloning_boundary::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "fn-length" => {
            let findings = checks::fn_length::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | "annotation-scope" => {
            let findings = checks::annotation_scope::run(&repo_root, &config)?;
            print_flat_findings(name.as_str(), &findings);
            return if findings.is_empty() { Ok(()) } else { bail!("{name} failed") };
        },
        | _ => bail!("toolkit-xtask: unknown check: {name}"),
    };
    print_findings(name.as_str(), &findings);
    if findings.is_empty() {
        Ok(())
    } else {
        bail!("{name} failed")
    }
}

fn run_parity(args: &[String]) -> Result<()> {
    let (name, repo_root, config_path) = parse_check_args(args)?;
    let config = config::load(&config_path)?;
    match name.replace('_', "-").as_str() {
        | "proc-macro-scope" => {
            let toolkit = checks::proc_macro_scope::run(&repo_root, &config)?;
            let legacy = legacy::run_proc_macro_scope(&repo_root)?;
            if toolkit != legacy {
                eprintln!("toolkit findings:");
                print_findings("proc-macro-scope", &toolkit);
                eprintln!("legacy findings:");
                print_findings("proc-macro-scope", &legacy);
                bail!("toolkit parity mismatch for proc-macro-scope");
            }
            println!("proc-macro-scope parity OK");
            Ok(())
        },
        | "result-must-use" => parity_flat(
            "result-must-use",
            checks::result_must_use::run(&repo_root, &config)?,
            legacy::run_flat_check(&repo_root, "result-must-use")?,
        ),
        | "test-boundaries" => parity_flat(
            "test-boundaries",
            checks::test_boundaries::run(&repo_root, &config)?,
            legacy::run_flat_check(&repo_root, "test-boundaries")?,
        ),
        | "docs-link-check" => parity_flat(
            "docs-link-check",
            checks::docs_link_check::run(&repo_root, &config)?,
            legacy::run_flat_check(&repo_root, "docs-link-check")?,
        ),
        | "docs-semantic-drift" => parity_flat(
            "docs-semantic-drift",
            checks::docs_semantic_drift::run(&repo_root, &config)?,
            legacy::run_flat_check(&repo_root, "docs-semantic-drift")?,
        ),
        | _ => bail!("toolkit-xtask: unknown parity check: {name}"),
    }
}

fn show_config(args: &[String]) -> Result<()> {
    let (repo_root, config_path) = parse_shared_args(args)?;
    let config = config::load(&config_path)?;
    println!("repo_root = {}", repo_root.display());
    println!("config_path = {}", config_path.display());
    println!("crate_roots = {:?}", config.workspace.crate_roots);
    println!("include_crates = {:?}", config.workspace.include_crates);
    println!("exclude_crates = {:?}", config.workspace.exclude_crates);
    if let Some(bundle) = &config.bundles.rust_base {
        println!("bundles.rust_base.enabled = {}", bundle.enabled);
        println!("bundles.rust_base.rust_roots = {:?}", bundle.rust_roots);
        println!("bundles.rust_base.docs_roots = {:?}", bundle.docs_roots);
        println!(
            "bundles.rust_base.manifest_path = {:?}",
            bundle.manifest_path
        );
        println!(
            "bundles.rust_base.workflow_roots = {:?}",
            bundle.workflow_roots
        );
    }
    println!(
        "extra_check_keys = {:?}",
        config.checks.extra.keys().collect::<Vec<_>>()
    );
    Ok(())
}

fn run_fmt_check(args: &[String]) -> Result<()> {
    let (repo_root, _) = parse_shared_args(args)?;
    let toolkit_root = toolkit_root()?;
    let nix_path = format!("path:{}", toolkit_root.display());
    let status = process::Command::new("nix")
        .args([
            "develop",
            &nix_path,
            "--command",
            "bash",
            "-lc",
            "toolkit-cargo-fmt-nightly \"$REPO_ROOT\" --manifest-path \"$TOOLKIT_ROOT/xtask/Cargo.toml\" -- --check",
        ])
        .env("REPO_ROOT", &repo_root)
        .env("TOOLKIT_ROOT", &toolkit_root)
        .current_dir(&repo_root)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        bail!("toolkit fmt-check failed")
    }
}

fn run_fmt(args: &[String]) -> Result<()> {
    let (repo_root, _) = parse_shared_args(args)?;
    let toolkit_root = toolkit_root()?;
    let nix_path = format!("path:{}", toolkit_root.display());
    let status = process::Command::new("nix")
        .args([
            "develop",
            &nix_path,
            "--command",
            "bash",
            "-lc",
            "toolkit-cargo-fmt-nightly \"$REPO_ROOT\" --manifest-path \"$TOOLKIT_ROOT/xtask/Cargo.toml\"",
        ])
        .env("REPO_ROOT", &repo_root)
        .env("TOOLKIT_ROOT", &toolkit_root)
        .current_dir(&repo_root)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        bail!("toolkit fmt failed")
    }
}

fn run_clippy(args: &[String]) -> Result<()> {
    let (repo_root, _) = parse_shared_args(args)?;
    let toolkit_root = toolkit_root()?;
    let nix_path = format!("path:{}", toolkit_root.display());
    let status = process::Command::new("nix")
        .args([
            "develop",
            &nix_path,
            "--command",
            "bash",
            "-lc",
            "cd \"$TOOLKIT_ROOT/xtask\" && cargo clippy --all-targets -- -D warnings",
        ])
        .env("TOOLKIT_ROOT", &toolkit_root)
        .current_dir(&repo_root)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        bail!("toolkit clippy failed")
    }
}

fn run_dylint(args: &[String]) -> Result<()> {
    let (repo_root, _) = parse_shared_args(args)?;
    let toolkit_root = toolkit_root()?;
    let nix_path = format!("path:{}", toolkit_root.display());
    let status = process::Command::new("nix")
        .args([
            "develop",
            &nix_path,
            "--command",
            "bash",
            "-lc",
            "toolkit-install-dylint && cargo dylint --path \"$TOOLKIT_ROOT/lints/trait_must_use\" --manifest-path crates/traits/Cargo.toml -- --all-targets && cargo dylint --path \"$TOOLKIT_ROOT/lints/style_limits\" --manifest-path \"$TOOLKIT_ROOT/xtask/Cargo.toml\" -- --all-targets",
        ])
        .env("TOOLKIT_ROOT", &toolkit_root)
        .current_dir(&repo_root)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        bail!("toolkit dylint failed")
    }
}

fn parse_check_args(args: &[String]) -> Result<(String, PathBuf, PathBuf)> {
    let name = args.first().cloned().ok_or_else(|| {
        anyhow::anyhow!("toolkit-xtask: usage: <check|parity> <name> [--repo-root <path>] [--config <path>]")
    })?;
    let (repo_root, config_path) = parse_shared_args(&args[1..])?;
    Ok((name, repo_root, config_path))
}

fn parse_shared_args(args: &[String]) -> Result<(PathBuf, PathBuf)> {
    let mut repo_root = env::current_dir()?;
    let mut config_path = PathBuf::from(DEFAULT_CONFIG);
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            | "--repo-root" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    anyhow::anyhow!("toolkit-xtask: missing value for --repo-root")
                })?;
                repo_root = PathBuf::from(value);
            },
            | "--config" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    anyhow::anyhow!("toolkit-xtask: missing value for --config")
                })?;
                config_path = PathBuf::from(value);
            },
            | other => bail!("toolkit-xtask: unknown argument: {other}"),
        }
        index += 1;
    }
    let repo_root = canonicalize_existing(&repo_root)?;
    let config_path = if config_path.is_absolute() {
        config_path
    } else {
        repo_root.join(config_path)
    };
    Ok((repo_root, config_path))
}

fn canonicalize_existing(path: &Path) -> Result<PathBuf> {
    path.canonicalize()
        .map_err(|err| anyhow::anyhow!("canonicalizing {}: {err}", path.display()))
}

fn print_findings(name: &str, findings: &FindingSet) {
    if findings.stale.is_empty() && findings.missing.is_empty() {
        println!("{name}: OK");
        return;
    }
    if !findings.stale.is_empty() {
        println!("stale {name} findings:");
        for entry in &findings.stale {
            println!("  {entry}");
        }
    }
    if !findings.missing.is_empty() {
        println!("missing {name} findings:");
        for entry in &findings.missing {
            println!("  {entry}");
        }
    }
}

fn print_flat_findings(name: &str, findings: &report::FlatFindingSet) {
    if findings.entries.is_empty() {
        println!("{name}: OK");
        return;
    }
    for entry in &findings.entries {
        println!("{entry}");
    }
}

fn parity_flat(
    name: &str,
    toolkit: report::FlatFindingSet,
    legacy: report::FlatFindingSet,
) -> Result<()> {
    if toolkit != legacy {
        eprintln!("toolkit findings:");
        print_flat_findings(name, &toolkit);
        eprintln!("legacy findings:");
        print_flat_findings(name, &legacy);
        bail!("toolkit parity mismatch for {name}");
    }
    println!("{name} parity OK");
    Ok(())
}
