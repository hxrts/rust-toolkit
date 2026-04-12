use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{bail, Context, Result};

#[derive(Debug, Clone)]
pub struct ToolkitConfig {
    pub workspace: WorkspaceConfig,
    pub checks: ChecksConfig,
    pub bundles: BundlesConfig,
}

#[derive(Debug, Clone, Default)]
pub struct BundlesConfig {
    pub rust_base: Option<RustBaseBundle>,
}

#[derive(Debug, Clone)]
pub struct RustBaseBundle {
    pub enabled: bool,
    pub rust_roots: Vec<String>,
    pub docs_roots: Vec<String>,
    pub manifest_path: String,
    pub workflow_roots: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
    pub crate_roots: Vec<String>,
    pub include_crates: Vec<String>,
    pub exclude_crates: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct ChecksConfig {
    pub proc_macro_scope: Option<ProcMacroScopeConfig>,
    pub result_must_use: Option<ResultMustUseConfig>,
    pub test_boundaries: Option<TestBoundariesConfig>,
    pub lean_style: Option<LeanStyleConfig>,
    pub lean_escape_hatches: Option<LeanEscapeHatchesConfig>,
    pub docs_link_check: Option<DocsLinkCheckConfig>,
    pub docs_semantic_drift: Option<DocsSemanticDriftConfig>,
    pub workflow_actions: Option<WorkflowActionsConfig>,
    pub text_formatting: Option<TextFormattingConfig>,
    pub workspace_hygiene: Option<WorkspaceHygieneConfig>,
    pub crate_root_policy: Option<CrateRootPolicyConfig>,
    pub ignored_result: Option<IgnoredResultConfig>,
    pub unsafe_boundary: Option<UnsafeBoundaryConfig>,
    pub bool_param: Option<BoolParamConfig>,
    pub must_use_public_return: Option<MustUsePublicReturnConfig>,
    pub assert_shape: Option<AssertShapeConfig>,
    pub drop_side_effects: Option<DropSideEffectsConfig>,
    pub recursion_guard: Option<RecursionGuardConfig>,
    pub naming_units: Option<NamingUnitsConfig>,
    pub limit_constant: Option<LimitConstantConfig>,
    pub public_type_width: Option<PublicTypeWidthConfig>,
    pub dependency_policy: Option<DependencyPolicyConfig>,
    pub extra: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone)]
pub struct ProcMacroScopeConfig {
    pub enabled: bool,
    pub include_paths: Vec<String>,
    pub required_markers: Vec<String>,
    pub exclude_files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResultMustUseConfig {
    pub enabled: bool,
    pub include_paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TestBoundariesConfig {
    pub enabled: bool,
    pub scan_roots: Vec<String>,
    pub exclude_prefixes: Vec<String>,
    pub exclude_path_parts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DocsLinkCheckConfig {
    pub enabled: bool,
    pub docs_roots: Vec<String>,
    pub scratch_dir_prefix: String,
}

#[derive(Debug, Clone)]
pub struct LeanStyleConfig {
    pub enabled: bool,
    pub include_paths: Vec<String>,
    pub exclude_path_parts: Vec<String>,
    pub non_trivial_file_lines: usize,
    pub section_header_min_lines: usize,
    pub max_file_lines: usize,
    pub max_decl_lines_target: usize,
    pub max_decl_lines_hard_limit: usize,
    pub enforce_target_decl_lines: bool,
    pub require_problem_statement: bool,
    pub enforce_top_of_file_structure: bool,
    pub require_section_headers: bool,
    pub require_over_limit_comment: bool,
    pub require_explanatory_comment_for_long_blocks: bool,
    pub require_public_theorem_lemma_docstrings: bool,
    pub forbid_sorry: bool,
    pub banned_imports: Vec<String>,
    pub banned_import_exemptions: Vec<String>,
    pub require_todo_for_sorry: bool,
    pub todo_comment_markers: Vec<String>,
    pub over_limit_comment_markers: Vec<String>,
    pub file_exemptions: Vec<LeanStyleFileExemption>,
    pub declaration_exemptions: Vec<LeanStyleDeclarationExemption>,
}

#[derive(Debug, Clone)]
pub struct LeanStyleFileExemption {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct LeanStyleDeclarationExemption {
    pub path: String,
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct LeanEscapeHatchesConfig {
    pub enabled: bool,
    pub include_paths: Vec<String>,
    pub exclude_path_parts: Vec<String>,
    pub kind_thresholds: BTreeMap<String, usize>,
    pub file_exemptions: Vec<LeanEscapeHatchFileExemption>,
}

#[derive(Debug, Clone)]
pub struct LeanEscapeHatchFileExemption {
    pub path: String,
    pub kinds: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct DocsSemanticDriftConfig {
    pub enabled: bool,
    pub docs_roots: Vec<String>,
    pub manifest_path: String,
    pub planned_crates: Vec<String>,
    pub file_exemptions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TextFormattingConfig {
    pub enabled: bool,
    pub include_paths: Vec<String>,
    pub exclude_path_parts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceHygieneConfig {
    pub enabled: bool,
    pub include_paths: Vec<String>,
    pub exclude_path_parts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CrateRootPolicyConfig {
    pub enabled: bool,
    pub required_attributes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct IgnoredResultConfig {
    pub enabled: bool,
    pub include_paths: Vec<String>,
    pub allowed_comment_markers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UnsafeBoundaryConfig {
    pub enabled: bool,
    pub include_paths: Vec<String>,
    pub exclude_path_parts: Vec<String>,
    pub allowed_path_parts: Vec<String>,
    pub required_comment_markers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BoolParamConfig {
    pub enabled: bool,
    pub include_paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MustUsePublicReturnConfig {
    pub enabled: bool,
    pub include_paths: Vec<String>,
    pub exclude_path_parts: Vec<String>,
    pub allowed_return_type_prefixes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AssertShapeConfig {
    pub enabled: bool,
    pub include_paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DropSideEffectsConfig {
    pub enabled: bool,
    pub include_paths: Vec<String>,
    pub allow_comment_marker: String,
}

#[derive(Debug, Clone)]
pub struct RecursionGuardConfig {
    pub enabled: bool,
    pub include_paths: Vec<String>,
    pub exclude_path_parts: Vec<String>,
    pub allow_comment_marker: String,
}

#[derive(Debug, Clone)]
pub struct NamingUnitsConfig {
    pub enabled: bool,
    pub include_paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LimitConstantConfig {
    pub enabled: bool,
    pub include_paths: Vec<String>,
    pub allow_comment_marker: String,
}

#[derive(Debug, Clone)]
pub struct PublicTypeWidthConfig {
    pub enabled: bool,
    pub include_paths: Vec<String>,
    pub exclude_path_parts: Vec<String>,
    pub banned_types: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DependencyPolicyConfig {
    pub enabled: bool,
    pub manifest_roots: Vec<String>,
    pub require_default_features_false: Vec<String>,
    pub banned_dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WorkflowActionsConfig {
    pub enabled: bool,
    pub workflow_roots: Vec<String>,
    pub pin_comment_markers: Vec<String>,
}

pub fn load(path: &Path) -> Result<ToolkitConfig> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("reading toolkit config {}", path.display()))?;
    let value: toml::Value = contents
        .parse()
        .with_context(|| format!("parsing toolkit config {}", path.display()))?;
    parse_toolkit_config(&value)
}

fn parse_toolkit_config(value: &toml::Value) -> Result<ToolkitConfig> {
    let table = expect_table(value, "root")?;
    let workspace =
        parse_workspace_config(table.get("workspace").ok_or_else(|| {
            anyhow::anyhow!("toolkit config missing [workspace] section")
        })?)?;
    let mut checks = match table.get("checks") {
        | Some(value) => parse_checks_config(value)?,
        | None => ChecksConfig::default(),
    };
    let bundles = match table.get("bundles") {
        | Some(value) => parse_bundles_config(value)?,
        | None => BundlesConfig::default(),
    };
    if let Some(bundle) = &bundles.rust_base {
        apply_rust_base_bundle(&mut checks, bundle);
    }
    Ok(ToolkitConfig { workspace, checks, bundles })
}

fn parse_workspace_config(value: &toml::Value) -> Result<WorkspaceConfig> {
    let table = expect_table(value, "workspace")?;
    Ok(WorkspaceConfig {
        crate_roots: required_string_list(table, "crate_roots")?,
        include_crates: optional_string_list(table, "include_crates")?,
        exclude_crates: optional_string_list(table, "exclude_crates")?,
    })
}

// long-block-exception: config parsing stays explicit so keys map one-to-one to
// check surfaces
fn parse_checks_config(value: &toml::Value) -> Result<ChecksConfig> {
    let table = expect_table(value, "checks")?;
    let proc_macro_scope = table
        .get("proc_macro_scope")
        .map(parse_proc_macro_scope_config)
        .transpose()?;
    let result_must_use = table
        .get("result_must_use")
        .map(parse_result_must_use_config)
        .transpose()?;
    let test_boundaries = table
        .get("test_boundaries")
        .map(parse_test_boundaries_config)
        .transpose()?;
    let lean_style = table
        .get("lean_style")
        .map(parse_lean_style_config)
        .transpose()?;
    let lean_escape_hatches = table
        .get("lean_escape_hatches")
        .map(parse_lean_escape_hatches_config)
        .transpose()?;
    let docs_link_check = table
        .get("docs_link_check")
        .map(parse_docs_link_check_config)
        .transpose()?;
    let docs_semantic_drift = table
        .get("docs_semantic_drift")
        .map(parse_docs_semantic_drift_config)
        .transpose()?;
    let workflow_actions = table
        .get("workflow_actions")
        .map(parse_workflow_actions_config)
        .transpose()?;
    let text_formatting = table
        .get("text_formatting")
        .map(parse_text_formatting_config)
        .transpose()?;
    let workspace_hygiene = table
        .get("workspace_hygiene")
        .map(parse_workspace_hygiene_config)
        .transpose()?;
    let crate_root_policy = table
        .get("crate_root_policy")
        .map(parse_crate_root_policy_config)
        .transpose()?;
    let ignored_result = table
        .get("ignored_result")
        .map(parse_ignored_result_config)
        .transpose()?;
    let unsafe_boundary = table
        .get("unsafe_boundary")
        .map(parse_unsafe_boundary_config)
        .transpose()?;
    let bool_param = table
        .get("bool_param")
        .map(parse_bool_param_config)
        .transpose()?;
    let must_use_public_return = table
        .get("must_use_public_return")
        .map(parse_must_use_public_return_config)
        .transpose()?;
    let assert_shape = table
        .get("assert_shape")
        .map(parse_assert_shape_config)
        .transpose()?;
    let drop_side_effects = table
        .get("drop_side_effects")
        .map(parse_drop_side_effects_config)
        .transpose()?;
    let recursion_guard = table
        .get("recursion_guard")
        .map(parse_recursion_guard_config)
        .transpose()?;
    let naming_units = table
        .get("naming_units")
        .map(parse_naming_units_config)
        .transpose()?;
    let limit_constant = table
        .get("limit_constant")
        .map(parse_limit_constant_config)
        .transpose()?;
    let public_type_width = table
        .get("public_type_width")
        .map(parse_public_type_width_config)
        .transpose()?;
    let dependency_policy = table
        .get("dependency_policy")
        .map(parse_dependency_policy_config)
        .transpose()?;

    let mut extra = BTreeMap::new();
    for (key, value) in table {
        if matches!(
            key.as_str(),
            "proc_macro_scope"
                | "result_must_use"
                | "test_boundaries"
                | "lean_style"
                | "lean_escape_hatches"
                | "docs_link_check"
                | "docs_semantic_drift"
                | "workflow_actions"
                | "text_formatting"
                | "workspace_hygiene"
                | "crate_root_policy"
                | "ignored_result"
                | "unsafe_boundary"
                | "bool_param"
                | "must_use_public_return"
                | "assert_shape"
                | "drop_side_effects"
                | "recursion_guard"
                | "naming_units"
                | "limit_constant"
                | "public_type_width"
                | "dependency_policy"
        ) {
            continue;
        }
        extra.insert(key.clone(), value.clone());
    }

    Ok(ChecksConfig {
        proc_macro_scope,
        result_must_use,
        test_boundaries,
        lean_style,
        lean_escape_hatches,
        docs_link_check,
        docs_semantic_drift,
        workflow_actions,
        text_formatting,
        workspace_hygiene,
        crate_root_policy,
        ignored_result,
        unsafe_boundary,
        bool_param,
        must_use_public_return,
        assert_shape,
        drop_side_effects,
        recursion_guard,
        naming_units,
        limit_constant,
        public_type_width,
        dependency_policy,
        extra,
    })
}

fn parse_proc_macro_scope_config(value: &toml::Value) -> Result<ProcMacroScopeConfig> {
    let table = expect_table(value, "checks.proc_macro_scope")?;
    Ok(ProcMacroScopeConfig {
        enabled: required_bool(table, "enabled")?,
        include_paths: required_string_list(table, "include_paths")?,
        required_markers: required_string_list(table, "required_markers")?,
        exclude_files: optional_string_list(table, "exclude_files")?,
    })
}

fn parse_result_must_use_config(value: &toml::Value) -> Result<ResultMustUseConfig> {
    let table = expect_table(value, "checks.result_must_use")?;
    Ok(ResultMustUseConfig {
        enabled: required_bool(table, "enabled")?,
        include_paths: required_string_list(table, "include_paths")?,
    })
}

fn parse_test_boundaries_config(value: &toml::Value) -> Result<TestBoundariesConfig> {
    let table = expect_table(value, "checks.test_boundaries")?;
    Ok(TestBoundariesConfig {
        enabled: required_bool(table, "enabled")?,
        scan_roots: required_string_list(table, "scan_roots")?,
        exclude_prefixes: optional_string_list(table, "exclude_prefixes")?,
        exclude_path_parts: optional_string_list(table, "exclude_path_parts")?,
    })
}

fn parse_lean_style_config(value: &toml::Value) -> Result<LeanStyleConfig> {
    let table = expect_table(value, "checks.lean_style")?;
    Ok(LeanStyleConfig {
        enabled: required_bool(table, "enabled")?,
        include_paths: required_string_list(table, "include_paths")?,
        exclude_path_parts: optional_string_list(table, "exclude_path_parts")?,
        non_trivial_file_lines: required_usize(table, "non_trivial_file_lines")?,
        section_header_min_lines: required_usize(
            table,
            "section_header_min_lines",
        )?,
        max_file_lines: required_usize(table, "max_file_lines")?,
        max_decl_lines_target: required_usize(table, "max_decl_lines_target")?,
        max_decl_lines_hard_limit: required_usize(
            table,
            "max_decl_lines_hard_limit",
        )?,
        enforce_target_decl_lines: required_bool(
            table,
            "enforce_target_decl_lines",
        )?,
        require_problem_statement: required_bool(
            table,
            "require_problem_statement",
        )?,
        enforce_top_of_file_structure: required_bool(
            table,
            "enforce_top_of_file_structure",
        )?,
        require_section_headers: required_bool(table, "require_section_headers")?,
        require_over_limit_comment: required_bool(
            table,
            "require_over_limit_comment",
        )?,
        require_explanatory_comment_for_long_blocks: required_bool(
            table,
            "require_explanatory_comment_for_long_blocks",
        )?,
        require_public_theorem_lemma_docstrings: required_bool(
            table,
            "require_public_theorem_lemma_docstrings",
        )?,
        forbid_sorry: required_bool(table, "forbid_sorry")?,
        banned_imports: optional_string_list(table, "banned_imports")?,
        banned_import_exemptions: optional_string_list(
            table,
            "banned_import_exemptions",
        )?,
        require_todo_for_sorry: required_bool(table, "require_todo_for_sorry")?,
        todo_comment_markers: required_string_list(table, "todo_comment_markers")?,
        over_limit_comment_markers: required_string_list(
            table,
            "over_limit_comment_markers",
        )?,
        file_exemptions: optional_table_array(table, "file_exemptions")?
            .into_iter()
            .map(parse_lean_style_file_exemption)
            .collect::<Result<Vec<_>>>()?,
        declaration_exemptions: optional_table_array(
            table,
            "declaration_exemptions",
        )?
        .into_iter()
        .map(parse_lean_style_declaration_exemption)
        .collect::<Result<Vec<_>>>()?,
    })
}

fn parse_lean_style_file_exemption(
    table: &toml::map::Map<String, toml::Value>,
) -> Result<LeanStyleFileExemption> {
    Ok(LeanStyleFileExemption {
        path: required_string(table, "path")?,
        reason: required_string(table, "reason")?,
    })
}

fn parse_lean_style_declaration_exemption(
    table: &toml::map::Map<String, toml::Value>,
) -> Result<LeanStyleDeclarationExemption> {
    Ok(LeanStyleDeclarationExemption {
        path: required_string(table, "path")?,
        name: required_string(table, "name")?,
        reason: required_string(table, "reason")?,
    })
}

fn parse_lean_escape_hatches_config(
    value: &toml::Value,
) -> Result<LeanEscapeHatchesConfig> {
    let table = expect_table(value, "checks.lean_escape_hatches")?;
    Ok(LeanEscapeHatchesConfig {
        enabled: required_bool(table, "enabled")?,
        include_paths: required_string_list(table, "include_paths")?,
        exclude_path_parts: optional_string_list(table, "exclude_path_parts")?,
        kind_thresholds: optional_usize_map(table, "kind_thresholds")?,
        file_exemptions: optional_table_array(table, "file_exemptions")?
            .into_iter()
            .map(parse_lean_escape_hatch_file_exemption)
            .collect::<Result<Vec<_>>>()?,
    })
}

fn parse_lean_escape_hatch_file_exemption(
    table: &toml::map::Map<String, toml::Value>,
) -> Result<LeanEscapeHatchFileExemption> {
    Ok(LeanEscapeHatchFileExemption {
        path: required_string(table, "path")?,
        kinds: optional_string_list(table, "kinds")?,
        reason: required_string(table, "reason")?,
    })
}

fn parse_docs_link_check_config(value: &toml::Value) -> Result<DocsLinkCheckConfig> {
    let table = expect_table(value, "checks.docs_link_check")?;
    Ok(DocsLinkCheckConfig {
        enabled: required_bool(table, "enabled")?,
        docs_roots: required_string_list(table, "docs_roots")?,
        scratch_dir_prefix: required_string(table, "scratch_dir_prefix")?,
    })
}

fn parse_docs_semantic_drift_config(
    value: &toml::Value,
) -> Result<DocsSemanticDriftConfig> {
    let table = expect_table(value, "checks.docs_semantic_drift")?;
    Ok(DocsSemanticDriftConfig {
        enabled: required_bool(table, "enabled")?,
        docs_roots: required_string_list(table, "docs_roots")?,
        manifest_path: required_string(table, "manifest_path")?,
        planned_crates: optional_string_list(table, "planned_crates")?,
        file_exemptions: optional_string_list(table, "file_exemptions")?,
    })
}

fn parse_text_formatting_config(
    value: &toml::Value,
) -> Result<TextFormattingConfig> {
    let table = expect_table(value, "checks.text_formatting")?;
    Ok(TextFormattingConfig {
        enabled: required_bool(table, "enabled")?,
        include_paths: required_string_list(table, "include_paths")?,
        exclude_path_parts: optional_string_list(table, "exclude_path_parts")?,
    })
}

fn parse_workspace_hygiene_config(
    value: &toml::Value,
) -> Result<WorkspaceHygieneConfig> {
    let table = expect_table(value, "checks.workspace_hygiene")?;
    Ok(WorkspaceHygieneConfig {
        enabled: required_bool(table, "enabled")?,
        include_paths: required_string_list(table, "include_paths")?,
        exclude_path_parts: optional_string_list(table, "exclude_path_parts")?,
    })
}

fn parse_crate_root_policy_config(
    value: &toml::Value,
) -> Result<CrateRootPolicyConfig> {
    let table = expect_table(value, "checks.crate_root_policy")?;
    Ok(CrateRootPolicyConfig {
        enabled: required_bool(table, "enabled")?,
        required_attributes: required_string_list(table, "required_attributes")?,
    })
}

fn parse_ignored_result_config(value: &toml::Value) -> Result<IgnoredResultConfig> {
    let table = expect_table(value, "checks.ignored_result")?;
    Ok(IgnoredResultConfig {
        enabled: required_bool(table, "enabled")?,
        include_paths: required_string_list(table, "include_paths")?,
        allowed_comment_markers: optional_string_list(
            table,
            "allowed_comment_markers",
        )?,
    })
}

fn parse_unsafe_boundary_config(value: &toml::Value) -> Result<UnsafeBoundaryConfig> {
    let table = expect_table(value, "checks.unsafe_boundary")?;
    Ok(UnsafeBoundaryConfig {
        enabled: required_bool(table, "enabled")?,
        include_paths: required_string_list(table, "include_paths")?,
        exclude_path_parts: optional_string_list(table, "exclude_path_parts")?,
        allowed_path_parts: optional_string_list(table, "allowed_path_parts")?,
        required_comment_markers: required_string_list(
            table,
            "required_comment_markers",
        )?,
    })
}

fn parse_bool_param_config(value: &toml::Value) -> Result<BoolParamConfig> {
    let table = expect_table(value, "checks.bool_param")?;
    Ok(BoolParamConfig {
        enabled: required_bool(table, "enabled")?,
        include_paths: required_string_list(table, "include_paths")?,
    })
}

fn parse_must_use_public_return_config(
    value: &toml::Value,
) -> Result<MustUsePublicReturnConfig> {
    let table = expect_table(value, "checks.must_use_public_return")?;
    Ok(MustUsePublicReturnConfig {
        enabled: required_bool(table, "enabled")?,
        include_paths: required_string_list(table, "include_paths")?,
        exclude_path_parts: optional_string_list(table, "exclude_path_parts")?,
        allowed_return_type_prefixes: optional_string_list(
            table,
            "allowed_return_type_prefixes",
        )?,
    })
}

fn parse_assert_shape_config(value: &toml::Value) -> Result<AssertShapeConfig> {
    let table = expect_table(value, "checks.assert_shape")?;
    Ok(AssertShapeConfig {
        enabled: required_bool(table, "enabled")?,
        include_paths: required_string_list(table, "include_paths")?,
    })
}

fn parse_drop_side_effects_config(
    value: &toml::Value,
) -> Result<DropSideEffectsConfig> {
    let table = expect_table(value, "checks.drop_side_effects")?;
    Ok(DropSideEffectsConfig {
        enabled: required_bool(table, "enabled")?,
        include_paths: required_string_list(table, "include_paths")?,
        allow_comment_marker: required_string(table, "allow_comment_marker")?,
    })
}

fn parse_recursion_guard_config(value: &toml::Value) -> Result<RecursionGuardConfig> {
    let table = expect_table(value, "checks.recursion_guard")?;
    Ok(RecursionGuardConfig {
        enabled: required_bool(table, "enabled")?,
        include_paths: required_string_list(table, "include_paths")?,
        exclude_path_parts: optional_string_list(table, "exclude_path_parts")?,
        allow_comment_marker: required_string(table, "allow_comment_marker")?,
    })
}

fn parse_naming_units_config(value: &toml::Value) -> Result<NamingUnitsConfig> {
    let table = expect_table(value, "checks.naming_units")?;
    Ok(NamingUnitsConfig {
        enabled: required_bool(table, "enabled")?,
        include_paths: required_string_list(table, "include_paths")?,
    })
}

fn parse_limit_constant_config(value: &toml::Value) -> Result<LimitConstantConfig> {
    let table = expect_table(value, "checks.limit_constant")?;
    Ok(LimitConstantConfig {
        enabled: required_bool(table, "enabled")?,
        include_paths: required_string_list(table, "include_paths")?,
        allow_comment_marker: required_string(table, "allow_comment_marker")?,
    })
}

fn parse_public_type_width_config(
    value: &toml::Value,
) -> Result<PublicTypeWidthConfig> {
    let table = expect_table(value, "checks.public_type_width")?;
    Ok(PublicTypeWidthConfig {
        enabled: required_bool(table, "enabled")?,
        include_paths: required_string_list(table, "include_paths")?,
        exclude_path_parts: optional_string_list(table, "exclude_path_parts")?,
        banned_types: required_string_list(table, "banned_types")?,
    })
}

fn parse_dependency_policy_config(
    value: &toml::Value,
) -> Result<DependencyPolicyConfig> {
    let table = expect_table(value, "checks.dependency_policy")?;
    Ok(DependencyPolicyConfig {
        enabled: required_bool(table, "enabled")?,
        manifest_roots: required_string_list(table, "manifest_roots")?,
        require_default_features_false: optional_string_list(
            table,
            "require_default_features_false",
        )?,
        banned_dependencies: optional_string_list(table, "banned_dependencies")?,
    })
}

fn parse_bundles_config(value: &toml::Value) -> Result<BundlesConfig> {
    let table = expect_table(value, "bundles")?;
    let rust_base = table
        .get("rust_base")
        .map(parse_rust_base_bundle)
        .transpose()?;
    Ok(BundlesConfig { rust_base })
}

fn parse_rust_base_bundle(value: &toml::Value) -> Result<RustBaseBundle> {
    let table = expect_table(value, "bundles.rust_base")?;
    Ok(RustBaseBundle {
        enabled: required_bool(table, "enabled")?,
        rust_roots: required_string_list(table, "rust_roots")?,
        docs_roots: optional_string_list(table, "docs_roots")?,
        manifest_path: match table.get("manifest_path") {
            | Some(v) => v
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| anyhow::anyhow!("manifest_path must be a string"))?,
            | None => "Cargo.toml".to_string(),
        },
        workflow_roots: match table.get("workflow_roots") {
            | Some(v) => string_list(v, "workflow_roots")?,
            | None => vec![".github/workflows".to_string()],
        },
    })
}

// long-block-exception: bundle application fills in each check independently;
// explicit [checks.*] sections always take precedence over bundle defaults
fn apply_rust_base_bundle(checks: &mut ChecksConfig, bundle: &RustBaseBundle) {
    if !bundle.enabled {
        return;
    }
    let rust = &bundle.rust_roots;
    let docs = &bundle.docs_roots;
    let manifest = bundle.manifest_path.clone();
    let workflows = &bundle.workflow_roots;
    if checks.proc_macro_scope.is_none() {
        checks.proc_macro_scope = Some(ProcMacroScopeConfig {
            enabled: true,
            include_paths: rust.clone(),
            required_markers: vec![],
            exclude_files: vec![],
        });
    }
    if checks.result_must_use.is_none() {
        checks.result_must_use = Some(ResultMustUseConfig {
            enabled: true,
            include_paths: rust.clone(),
        });
    }
    if checks.test_boundaries.is_none() {
        checks.test_boundaries = Some(TestBoundariesConfig {
            enabled: true,
            scan_roots: rust.clone(),
            exclude_prefixes: vec![],
            exclude_path_parts: vec![
                "/tests/".to_string(),
                "/benches/".to_string(),
                "/examples/".to_string(),
                "/target/".to_string(),
            ],
        });
    }
    if !docs.is_empty() {
        if checks.docs_link_check.is_none() {
            checks.docs_link_check = Some(DocsLinkCheckConfig {
                enabled: true,
                docs_roots: docs.clone(),
                scratch_dir_prefix: "work/".to_string(),
            });
        }
        if checks.docs_semantic_drift.is_none() {
            checks.docs_semantic_drift = Some(DocsSemanticDriftConfig {
                enabled: true,
                docs_roots: docs.clone(),
                manifest_path: manifest.clone(),
                planned_crates: vec![],
                file_exemptions: vec![],
            });
        }
        if checks.text_formatting.is_none() {
            checks.text_formatting = Some(TextFormattingConfig {
                enabled: true,
                include_paths: docs.clone(),
                exclude_path_parts: vec![],
            });
        }
    }
    if checks.workspace_hygiene.is_none() {
        checks.workspace_hygiene = Some(WorkspaceHygieneConfig {
            enabled: true,
            include_paths: rust.clone(),
            exclude_path_parts: vec![],
        });
    }
    if checks.crate_root_policy.is_none() {
        checks.crate_root_policy = Some(CrateRootPolicyConfig {
            enabled: true,
            required_attributes: vec![],
        });
    }
    if checks.ignored_result.is_none() {
        checks.ignored_result = Some(IgnoredResultConfig {
            enabled: true,
            include_paths: rust.clone(),
            allowed_comment_markers: vec!["allow-ignored-result:".to_string()],
        });
    }
    if checks.unsafe_boundary.is_none() {
        checks.unsafe_boundary = Some(UnsafeBoundaryConfig {
            enabled: true,
            include_paths: rust.clone(),
            exclude_path_parts: vec![],
            allowed_path_parts: vec![],
            required_comment_markers: vec!["Safety:".to_string()],
        });
    }
    if checks.bool_param.is_none() {
        checks.bool_param = Some(BoolParamConfig {
            enabled: true,
            include_paths: rust.clone(),
        });
    }
    if checks.must_use_public_return.is_none() {
        checks.must_use_public_return = Some(MustUsePublicReturnConfig {
            enabled: true,
            include_paths: rust.clone(),
            exclude_path_parts: vec![],
            allowed_return_type_prefixes: vec![],
        });
    }
    if checks.assert_shape.is_none() {
        checks.assert_shape = Some(AssertShapeConfig {
            enabled: true,
            include_paths: rust.clone(),
        });
    }
    if checks.drop_side_effects.is_none() {
        checks.drop_side_effects = Some(DropSideEffectsConfig {
            enabled: true,
            include_paths: rust.clone(),
            allow_comment_marker: "drop-side-effects-exception:".to_string(),
        });
    }
    if checks.recursion_guard.is_none() {
        checks.recursion_guard = Some(RecursionGuardConfig {
            enabled: true,
            include_paths: rust.clone(),
            exclude_path_parts: vec![],
            allow_comment_marker: "recursion-exception:".to_string(),
        });
    }
    if checks.naming_units.is_none() {
        checks.naming_units = Some(NamingUnitsConfig {
            enabled: true,
            include_paths: rust.clone(),
        });
    }
    if checks.limit_constant.is_none() {
        checks.limit_constant = Some(LimitConstantConfig {
            enabled: true,
            include_paths: rust.clone(),
            allow_comment_marker: "limit-constant-exception:".to_string(),
        });
    }
    if checks.public_type_width.is_none() {
        checks.public_type_width = Some(PublicTypeWidthConfig {
            enabled: true,
            include_paths: rust.clone(),
            exclude_path_parts: vec![],
            banned_types: vec![],
        });
    }
    if checks.dependency_policy.is_none() {
        checks.dependency_policy = Some(DependencyPolicyConfig {
            enabled: true,
            manifest_roots: rust.clone(),
            require_default_features_false: vec![],
            banned_dependencies: vec![],
        });
    }
    if checks.workflow_actions.is_none() {
        checks.workflow_actions = Some(WorkflowActionsConfig {
            enabled: true,
            workflow_roots: workflows.clone(),
            pin_comment_markers: vec!["pin".to_string()],
        });
    }
}

fn parse_workflow_actions_config(value: &toml::Value) -> Result<WorkflowActionsConfig> {
    let table = expect_table(value, "checks.workflow_actions")?;
    Ok(WorkflowActionsConfig {
        enabled: required_bool(table, "enabled")?,
        workflow_roots: match table.get("workflow_roots") {
            | Some(value) => string_list(value, "workflow_roots")?,
            | None => vec![".github/workflows".to_string()],
        },
        pin_comment_markers: match table.get("pin_comment_markers") {
            | Some(value) => string_list(value, "pin_comment_markers")?,
            | None => vec!["pin".to_string()],
        },
    })
}

fn expect_table<'a>(
    value: &'a toml::Value,
    context: &str,
) -> Result<&'a toml::map::Map<String, toml::Value>> {
    value
        .as_table()
        .ok_or_else(|| anyhow::anyhow!("{context} must be a TOML table"))
}

fn required_bool(
    table: &toml::map::Map<String, toml::Value>,
    key: &str,
) -> Result<bool> {
    table
        .get(key)
        .and_then(toml::Value::as_bool)
        .ok_or_else(|| anyhow::anyhow!("missing or invalid boolean `{key}`"))
}

fn required_string(
    table: &toml::map::Map<String, toml::Value>,
    key: &str,
) -> Result<String> {
    table
        .get(key)
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| anyhow::anyhow!("missing or invalid string `{key}`"))
}

fn required_usize(
    table: &toml::map::Map<String, toml::Value>,
    key: &str,
) -> Result<usize> {
    let Some(raw) = table.get(key).and_then(toml::Value::as_integer) else {
        bail!("missing or invalid integer `{key}`");
    };
    usize::try_from(raw).map_err(|_| anyhow::anyhow!("`{key}` must be >= 0"))
}

fn required_string_list(
    table: &toml::map::Map<String, toml::Value>,
    key: &str,
) -> Result<Vec<String>> {
    let Some(value) = table.get(key) else {
        bail!("missing array `{key}`");
    };
    string_list(value, key)
}

fn optional_string_list(
    table: &toml::map::Map<String, toml::Value>,
    key: &str,
) -> Result<Vec<String>> {
    match table.get(key) {
        | Some(value) => string_list(value, key),
        | None => Ok(Vec::new()),
    }
}

fn optional_table_array<'a>(
    table: &'a toml::map::Map<String, toml::Value>,
    key: &str,
) -> Result<Vec<&'a toml::map::Map<String, toml::Value>>> {
    let Some(value) = table.get(key) else {
        return Ok(Vec::new());
    };
    let Some(items) = value.as_array() else {
        bail!("`{key}` must be an array");
    };
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        out.push(expect_table(item, key)?);
    }
    Ok(out)
}

fn optional_usize_map(
    table: &toml::map::Map<String, toml::Value>,
    key: &str,
) -> Result<BTreeMap<String, usize>> {
    let Some(value) = table.get(key) else {
        return Ok(BTreeMap::new());
    };
    let map = expect_table(value, key)?;
    let mut out = BTreeMap::new();
    for (kind, value) in map {
        let Some(raw) = value.as_integer() else {
            bail!("`{key}` values must be integers");
        };
        let threshold = usize::try_from(raw)
            .map_err(|_| anyhow::anyhow!("`{key}` values must be >= 0"))?;
        out.insert(kind.clone(), threshold);
    }
    Ok(out)
}

fn string_list(value: &toml::Value, key: &str) -> Result<Vec<String>> {
    let Some(items) = value.as_array() else {
        bail!("`{key}` must be an array");
    };
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let Some(value) = item.as_str() else {
            bail!("`{key}` must contain only strings");
        };
        out.push(value.to_owned());
    }
    Ok(out)
}
