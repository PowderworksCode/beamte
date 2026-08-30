#!/usr/bin/env bash
# Stand up a fresh beamte checkout: hooks, then both halves of the gate -- the
# library as a consumer takes it, and the dev harness rules are written against.
# Safe to re-run; every step is idempotent.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

# A clone runs no hooks until it is pointed at them: core.hooksPath is per-clone
# configuration, so nothing a checkout carries can set it for you.
git config core.hooksPath .githooks
if [ ! -d .githooks ]; then
    echo "note: .githooks is fleet-managed and not synced here yet; git will"
    echo "      start using it the moment ordnung writes it."
fi

if ! command -v cargo >/dev/null; then
    echo "error: cargo is not on PATH; install Rust from https://rustup.rs" >&2
    exit 1
fi

# Both configurations, in the order CI checks them. The no-default-features
# build comes first because it is the claim the crate makes about itself: a
# library with no dependencies and no parser. Building only the dev harness
# would never notice something dev-gated becoming reachable from the API.
echo "== library, no default features"
cargo build --locked --no-default-features

echo "== library, default features"
cargo build --locked
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings

# treebank-python is a git dependency, so this is the step that needs network
# and repository access the first time.
echo "== dev harness"
cargo fmt --check
cargo test --locked --features dev
cargo clippy --locked --all-targets --features dev -- -D warnings

echo
echo "ready. the harness, against a real file:"
echo "  cargo run --features dev -- check   some_test.py   # findings"
echo "  cargo run --features dev -- explain some_test.py   # the tree, with roles"
echo "  cargo run --features dev -- rules                  # the catalogue"
