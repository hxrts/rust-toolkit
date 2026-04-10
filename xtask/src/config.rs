use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{bail, Context, Result};

#[derive(Debug, Clone)]
pub struct ToolkitConfig {
    pub workspace: WorkspaceConfig,
    pub checks: ChecksConfig,
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
    pub docs_link_check: Option<DocsLinkCheckConfig>,
    pub docs_semantic_drift: Option<DocsSemanticDriftConfig>,
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
pub struct DocsSemanticDriftConfig {
    pub enabled: bool,
    pub docs_roots: Vec<String>,
    pub manifest_path: String,
    pub planned_crates: Vec<String>,
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
    pub banned_types: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DependencyPolicyConfig {
    pub enabled: bool,
    pub manifest_roots: Vec<String>,
    pub require_default_features_false: Vec<String>,
    pub banned_dependencies: Vec<String>,
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
    let checks = match table.get("checks") {
        | Some(value) => parse_checks_config(value)?,
        | None => ChecksConfig::default(),
    };
    Ok(ToolkitConfig { workspace, checks })
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
    let docs_link_check = table
        .get("docs_link_check")
        .map(parse_docs_link_check_config)
        .transpose()?;
    let docs_semantic_drift = table
        .get("docs_semantic_drift")
        .map(parse_docs_semantic_drift_config)
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
                | "docs_link_check"
                | "docs_semantic_drift"
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
        docs_link_check,
        docs_semantic_drift,
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
