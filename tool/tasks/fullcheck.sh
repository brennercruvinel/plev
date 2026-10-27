#!/bin/sh
# the gate: every change passes all of it. one command per part so the
# first red names the failing gate. CI runs the same parts as separate jobs
# (.github/workflows/gatecheck.yml).
set -e

cd "$(dirname "$0")/../.."

# toolchain guard: rustup honors rust-toolchain.toml; a Homebrew rust ignores
# it and ships no wasm32 std, so the wasm leg is not checkable there.
if ! command -v rustup >/dev/null 2>&1; then
    echo "warning: rustup not found — a Homebrew rust ignores rust-toolchain.toml." >&2
    echo "         the pinned toolchain is not verified locally; run the gate via" >&2
    echo "         CI (.github/workflows/gatecheck.yml) or install rustup." >&2
fi

cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check

# wasm cross-platform guard. Skippable only when the std for the target is
# genuinely absent (Homebrew rust); with rustup the target comes from
# rust-toolchain.toml and this leg always runs.
# RUSTFLAGS as in CI (.github/workflows/gatecheck.yml sets -D warnings for
# every job): code only native builds read is dead on wasm32, and a warning
# there is red in CI, so it is red here too.
if command -v rustup >/dev/null 2>&1; then
    RUSTFLAGS="-D warnings" cargo check --target wasm32-unknown-unknown -p uigallery
elif [ -d "$(rustc --print sysroot)/lib/rustlib/wasm32-unknown-unknown" ]; then
    RUSTFLAGS="-D warnings" cargo check --target wasm32-unknown-unknown -p uigallery
else
    echo "warning: skipping the wasm32 check — rust-std for wasm32-unknown-unknown" >&2
    echo "         is not installed (Homebrew rust). the leg runs in CI." >&2
fi

# unions: rust/unions still holds the union of the workspace's features
# (.config/hakari.toml), and every member depends on it. the fix for a red
# here is the same command without the flag.
if command -v cargo-hakari >/dev/null 2>&1; then
    cargo hakari generate --diff
    cargo hakari manage-deps --dry-run
else
    echo "warning: skipping unions — cargo-hakari not installed (cargo install --locked cargo-hakari)." >&2
    echo "         the leg runs in CI." >&2
fi

# docs links: every link into the repository resolves (lychee, offline).
if command -v lychee >/dev/null 2>&1; then
    lychee --no-progress --config .devops/check/lychee.toml --offline .
else
    echo "warning: skipping the docs links — lychee not installed (brew install lychee)." >&2
    echo "         the leg runs in CI." >&2
fi

# urnadbapp leg. urnadbapp is excluded from the root workspace because its
# native backend path-depends on the sibling urna workspace; when that
# checkout is absent the leg skips with a warning and CI
# (.github/workflows/gatecheck.yml) is the verification.
#
# urnadbapp is its own cargo workspace. with CARGO_TARGET_DIR set (the
# agents' build dirs), both workspaces would write one target dir, and the
# engine, whose cdylib/staticlib crate types keep its file name free of a
# hash, would land in the same `libengine.rlib` built against another
# `accord` ("multiple different versions of crate `accord`"). the leg gets
# its own subdirectory then; without CARGO_TARGET_DIR it uses
# apps/urnadbapp/target.
if [ -f ../urna/rust/format/Cargo.toml ]; then
    (
        if [ -n "${CARGO_TARGET_DIR:-}" ]; then
            export CARGO_TARGET_DIR="$CARGO_TARGET_DIR/urnadbapp"
        fi
        cargo test --manifest-path apps/urnadbapp/Cargo.toml
        cargo clippy --manifest-path apps/urnadbapp/Cargo.toml --all-targets -- -D warnings
        cargo fmt --check --manifest-path apps/urnadbapp/Cargo.toml
        RUSTFLAGS="-D warnings" cargo check --manifest-path apps/urnadbapp/Cargo.toml --target wasm32-unknown-unknown
    )
else
    echo "warning: skipping urnadbapp — sibling urna checkout not found at ../urna." >&2
    echo "         the leg runs in CI (.github/workflows/gatecheck.yml)." >&2
fi
