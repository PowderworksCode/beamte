# Agent field guide

Durable notes for anyone — human or agent — starting work in this repository.
Append what you learn; keep it to things that are true and not obvious from the
code.

## What this is

Beamte is a Rust library of test-quality rules: a parsed test file goes in,
findings come out. Every rule restates a post from the Google Testing Blog and
carries the citation, and every rule is written against treebank's node
vocabulary (`_loop`, `_branch`, `_invocation`, `_callable`) rather than against
one grammar's node names, which is what makes one rule work across languages.

It is a library on purpose. No CLI, no config, no output format, no file walker,
no exit codes — those are the host's, and straitjacket is the first host.
`notes/DESIGN.md` is authoritative: where it and the code disagree, one of them
is a bug, and which one is a decision to be recorded there. Several source files
cite it by section number, so renumbering it means grepping for
`notes/DESIGN.md`.

Today the catalogue has two rules — `test-logic` (`src/rules/test_logic.rs`)
and `env-read` (`src/rules/env_read.rs`) — and one grammar wired up for
development (Python). The scaffolding around them is the point; adding a rule
is meant to be the small part.

The two rules differ in *scope*, which every `Rule` now declares: `test-logic`
reads test bodies (`Scope::Tests`), `env-read` reads any source file
(`Scope::File`). A host deciding which files to hand a rule reads the scope
rather than guessing; `notes/DESIGN.md` §5.5 records why the catalogue widened.

## The two builds, and why `cargo test -p beamte` proves less than it looks

```sh
cargo test -p beamte     # the library alone: no parser, one dependency
cargo test --workspace   # the rules, against real parsed source
```

**`cargo test -p beamte` runs no rule against any source.** The rules are
exercised from `beamte-dev`, the harness crate beside the library, so a green
run of the library alone has not parsed a thing. Use `--workspace` and check
the count: it is 45, and 18 of those are the library's own.

This used to be sharper and worse. The harness was a `dev` feature on the
library, both integration tests opened with `#![cfg(feature = "dev")]`, and a
plain `cargo test` compiled them to nothing while reporting `ok` -- a green run
that had tested no rule at all. Splitting the crate removed the gate rather
than documenting it, which is why the count is the thing to check now.

`beamte-dev` carries native tree-sitter, treebank's Python grammar and the
`beamte` binary. It is never published; the library is. For iterating on a
rule:

```sh
cargo run -p beamte-dev -- check   some_test.py   # findings
cargo run -p beamte-dev -- explain some_test.py   # the tree, with roles
cargo run -p beamte-dev -- rules                  # the catalogue
```

`explain` is the one that matters: when a rule misfires the finding tells you
nothing and the tree tells you everything.

Note that the dev harness is *not* how a real host parses. A host loads treebank
wasm packs and implements `Node` over the `tb_*` ABI; `src/dev/mod.rs` links the
grammar natively instead. Same grammar, two paths, and that they agree is still
unproven — `notes/DESIGN.md` §10.1 wants the fixture that would settle it.

## Landmines

**`treebank-python` is a git dependency with no `rev` in `Cargo.toml`, pinned
only by `Cargo.lock`.** CI passes `--locked` everywhere, so the pin only moves
when someone runs `cargo update`, and `Cargo.lock` has to stay committed. It is
also why the crate is `publish = false`: treebank is not on crates.io yet, and a
git dependency blocks `cargo publish` even when it is optional and off by
default.

The part that surprises people: **cargo resolves the whole graph before it
applies features, so even `cargo build --no-default-features` clones that
repository.** An environment without access to
`github.com/PowderworksCode/treebank` fails at resolution, before compiling
anything, with a message about `treebank-python` that has nothing to do with the
feature you asked for. If your git config rewrites `https://github.com/` to SSH,
cargo will try SSH and fail on a missing agent key; that is a machine problem,
not a repository one.

**Bumping treebank can turn `tests/vocabulary.rs` red for a reason unrelated to
your change.** That file asserts `table.unknown_terms()` is empty — every term
treebank declares must map to a `Role` beamte knows. The failure mode it guards
is silent: an unrecognised term never matches, so a rule quietly stops firing
while every other test stays green. If treebank adds a term, add the `Role`.

**Two CI steps check claims that no compiler enforces**, both in the `library`
job of `.github/workflows/ci.yml`:

- with `--no-default-features`, `cargo tree` must list *zero* dependencies;
- with default features, the tree must contain no `wasmtime`, `wasmer`,
  `cranelift`, `ureq` or `tree-sitter`.

The second exists because treebank's own default features are `pack` and
`fetch`, which drag in a wasm engine and an HTTP stack — 238 crates against 14 —
and would break straitjacket's musl release build, which is the exact failure
beamte exists not to cause. The `default-features = false` on the `treebank`
dependency is load-bearing; do not drop it while "simplifying" the manifest.

**Adding any dependency to the default feature set will fail CI**, even a small
one. If a rule needs something, it belongs behind a feature.

## Where the reasoning lives

The per-grammar tier question is the subtlety most likely to bite a new rule.
A treebank term can be a real parse-table supertype in one grammar and absent in
another: Python does not thread `_control_flow` at all — `_loop` derives straight
from `_statement` — so a rule asking for `_control_flow` would silently never
fire there. That is why `test-logic` asks for `_loop` and `_branch` instead, and
`tests/vocabulary.rs` has a test pinning exactly that fact. Check what a grammar
actually threads before writing a rule against a term.

`src/manifest.rs` builds the role table from the grammar's own
`node-types.json` and `roles.json`, using treebank's types rather than a second
reading of the same files — a second reading is how two consumers come to
disagree about what derives from what. Supertype membership is resolved
transitively; reading one level of the subtype lists would miss
`while_statement` being a `_statement`.

Fixtures in `tests/test_logic.rs` are written DAMP — literal, self-contained,
with no loops or conditionals of their own. A fixture corpus for a rule that
bans logic in tests must not contain logic in tests.

## Fleet

`.github/workflows/fleet-lint.yml` is distributed by conf; edit it there, not
here, or the next fleet sync reports drift. Its `hawk` job pins Rust 1.98.0, runs
only when Rust changed, and is advisory. `.github/workflows/ci.yml` is this
repository's own and is the one to edit.
