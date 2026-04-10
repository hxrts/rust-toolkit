{
  description = "Toolkit nightly tooling shell for formatter and dylint validation";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchainNightly = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "rustc-dev"
            "llvm-tools-preview"
            "rustfmt"
          ];
        };

        cargoWrapper = pkgs.writeShellScriptBin "cargo" ''
          set -euo pipefail
          if [ -z "''${RUSTUP_TOOLCHAIN:-}" ]; then
            host="$(rustc -vV | awk '/^host: / { print $2 }')"
            export RUSTUP_TOOLCHAIN="toolkit-nightly-''${host}"
          fi
          exec "$HOME/.cargo/bin/cargo" "$@"
        '';

        installDylint = pkgs.writeShellScriptBin "toolkit-install-dylint" ''
          set -euo pipefail
          dylint_repo="''${XDG_CACHE_HOME:-$HOME/.cache}/toolkit/dylint"
          dylint_rev="4bd91ce7729b74c7ee5664bbb588f7baf30b4a09"
          mkdir -p "$(dirname "$dylint_repo")"
          if [ ! -d "$dylint_repo/.git" ]; then
            git clone https://github.com/trailofbits/dylint.git "$dylint_repo"
          fi
          git -C "$dylint_repo" fetch --tags origin
          git -C "$dylint_repo" checkout --force "$dylint_rev"
          ${rustToolchainNightly}/bin/cargo install --locked --force --path "$dylint_repo/cargo-dylint"
          ${rustToolchainNightly}/bin/cargo install --locked --force --path "$dylint_repo/dylint-link"
          host="$(rustc -vV | awk '/^host: / { print $2 }')"
          toolchain_name="toolkit-nightly-''${host}"
          toolchain_root="$(dirname "$(dirname "$(command -v rustc)")")"
          rustup toolchain remove "$toolchain_name" >/dev/null 2>&1 || true
          rustup toolchain link "$toolchain_name" "$toolchain_root"
          if [ -d "$PWD/toolkit/lints" ]; then
            (cd "$PWD/toolkit/lints" && rustup override set "$toolchain_name" >/dev/null)
          fi
        '';

        dylintLinkWrapper = pkgs.writeShellScriptBin "toolkit-dylint-link" ''
          set -euo pipefail
          if [ -z "''${RUSTUP_TOOLCHAIN:-}" ]; then
            host="$(rustc -vV | awk '/^host: / { print $2 }')"
            export RUSTUP_TOOLCHAIN="toolkit-nightly-''${host}"
          fi
          exec dylint-link "$@"
        '';

        cargoFmtNightly = pkgs.writeShellScriptBin "toolkit-cargo-fmt-nightly" ''
          set -euo pipefail
          repo_root="''${1:-$PWD}"
          if [ "$#" -gt 0 ]; then
            shift
          fi
          export RUSTFMT_CONFIG_PATH="$repo_root/toolkit/rustfmt.toml"
          exec ${rustToolchainNightly}/bin/cargo fmt "$@"
        '';
      in
      {
        devShells.default = pkgs.mkShell {
          packages =
            with pkgs;
            [
              cargoWrapper
              rustToolchainNightly
              installDylint
              dylintLinkWrapper
              cargoFmtNightly
              git
              just
              ripgrep
              perl
              pkg-config
              openssl
              zlib
            ]
            ++ lib.optionals stdenv.isDarwin [
              libiconv
            ];

          shellHook = ''
            echo "Toolkit nightly environment"
            echo "Rust: $(rustc --version)"
            echo "Run 'toolkit-install-dylint' once in this shell if cargo-dylint is not installed."
          '';
        };
      }
    );
}
