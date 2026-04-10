# Consuming The Toolkit

Repositories that adopt the toolkit should keep their own entrypoints and
repository-specific policy, but consume the toolkit through a flake input.

## Ownership Split

- toolkit repo:
  generic checks, reusable lints, fixture harnesses, config parsing, proc
  macros / effect support, and tooling shells
- consuming repo:
  local `justfile`, CI wiring, hooks, and a repo-owned `policy/` directory

## Recommended Layout

```text
repo/
  flake.nix
  flake.lock
  justfile
  .githooks/
  .github/
  policy/
    README.md
    toolkit.toml
    checks/
    lints/
    fixtures/
    docs/
```

The consuming repository keeps ownership of:

- `justfile`
- CI workflow commands
- pre-commit hooks
- the repo `flake.nix` and `flake.lock`
- `policy/`

Those entrypoints call into the toolkit commands exposed from the repo's own
default dev shell and pass the local config, usually
`--config policy/toolkit.toml`.

## Developer Workflow

1. Add the toolkit as a flake input and pin it in `flake.lock`.
2. Keep repo entrypoints local.
3. Add `policy/toolkit.toml`.
4. Add repo-local `policy/checks/`, `policy/lints/`, `policy/fixtures/`, and
   policy docs only as needed.
5. Export `TOOLKIT_ROOT` to the toolkit input path from the repo shell.
6. Add the toolkit command packages to the repo shell.
7. Run toolkit commands from `just`, CI, and hooks through the repo shell.
7. Add domain-specific rules only under `policy/`.

## Toolkit Command Surface

Inside the toolkit Nix shell, the reusable command surface is:

- `toolkit-xtask`
  Runs the generic `xtask` runner from the pinned toolkit checkout.
- `toolkit-fmt`
  Runs nightly `cargo fmt`, using either `--config <rustfmt.toml>` or the
  toolkit repo's own `rustfmt.toml`.
- `toolkit-install-dylint`
  Installs `cargo-dylint` and `dylint-link`, then links the pinned nightly
  toolchain name used by toolkit lint runs.
- `toolkit-dylint`
  Runs either a toolkit-owned lint (`--toolkit-lint <name>`) or a
  consumer-owned lint (`--lint-path <path>`) with the toolkit nightly setup.

Toolkit commands assume the consuming repo has added those packages to its own
shell and exported `TOOLKIT_ROOT` to the toolkit input path.

## Recommended Flake Wiring

The consuming repo should usually wire the toolkit in its own `flake.nix`:

```nix
inputs = {
  toolkit = {
    url = "github:hxrts/rust-toolkit";
    inputs.nixpkgs.follows = "nixpkgs";
    inputs.rust-overlay.follows = "rust-overlay";
    inputs.flake-utils.follows = "flake-utils";
  };
};
```

and then add the toolkit packages plus:

```nix
shellHook = ''
  export TOOLKIT_ROOT="${toolkit}"
'';
```

That keeps toolkit versioning repo-owned through the repo's flake lock while
making the toolkit command surface directly usable from `just`, CI, and hooks.

## Rule Placement

- if a rule is generic and only the scope is repo-specific, configure it in
  `policy/toolkit.toml`
- if a rule depends on repo-specific architecture concepts, keep it under
  `policy/`
- if a repo-local rule later proves reusable, move it into the toolkit and
  delete the local copy

## Practical Command Shape

The consuming repo should usually call toolkit commands directly from its own
shell:

```bash
nix develop --command toolkit-xtask check <name> --repo-root . --config policy/toolkit.toml
nix develop --command toolkit-install-dylint
nix develop --command toolkit-dylint --repo-root . --toolkit-lint trait_purity --all -- --all-targets
nix develop --command toolkit-dylint --repo-root . --lint-path ./policy/lints/model_policy --all -- --all-targets
nix develop --command toolkit-fmt --all -- --check
```

Repo-specific policy commands remain repo-owned. For example, a consuming repo
can still run its own policy runner directly:

```bash
cargo run --manifest-path policy/xtask/Cargo.toml -- check <name>
```

## Repo-Local Dylint Requirements

If the consuming repo adds its own `policy/lints/*` crates, those lint crates
still need the normal Dylint linker setup. The recommended shape is:

```text
policy/
  lints/
    .cargo/
      config.toml
```

with:

```toml
[target.'cfg(not(target_os = "windows"))']
linker = "toolkit-dylint-link"
```

That linker wrapper is provided by the toolkit Nix shell and is required so
`cargo dylint` produces the `@toolchain` library filename that Dylint expects.
