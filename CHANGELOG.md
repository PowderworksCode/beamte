# Changelog

Notable changes to the `beamte` crate. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the crate
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The `beamte-dev` crate is not published and is not versioned here.

## [Unreleased]

## [0.2.0] - 2026-08-30

### Added

- `env-read`, the first rule with something to say about every file rather
  than only test files: code that reads the process environment where nothing
  declares it — `std::env::var`, `os.environ`, `process.env`, `ENV[…]`,
  `System.getenv`, `getenv` — is an input no signature admits to, and no
  small test of that code can stay hermetic (*Test Sizes*, 2010-12-13).
  Where the environment *may* be read is the host's configuration, exactly as
  severity is. Bash is deliberately not covered: `$VAR` is the language's own
  variable model, and `env_read::covers` lets a host report the gap rather
  than a clean file.
- `Scope` on every `Rule`, saying what the rule reads: `Tests` for the
  catalogue as it was, `File` for `env-read`. A host that runs "all rules"
  over test files alone needs to know which kind it is holding, and this is
  how it knows.

### Changed

- **Breaking**: `Rule` gained the public `scope` field, so a consumer
  matching the struct exhaustively has a new field to name.

## [0.1.0] - 2026-08-30

First release. The library was already in use through a git dependency; this
is the point at which a consumer can name a version instead.

### Added

- `inspect` and `inspect_with`, one entry point a host calls with a parsed
  unit, a test model and a rule selection (`All`, `Only`, `Except`). A host
  adds no rule dispatch of its own and gains every rule added here.
- `rule(name)`, for validating a rule name out of a host's configuration file
  before a typo silently turns a rule off.
- `test-logic`, which flags a loop or a conditional in a test body, carrying
  the citation as well as the fix.
- Test models for the ten languages treebank serves a pack for, recognising a
  test by name, by attribute, by an invocation taking a body, or by a syntax
  kind of its own -- the four shapes languages actually use.
- `manifests` (on by default), which reads a grammar's `node-types.json` and
  `roles.json` into a role table using treebank's own types, so two hosts
  cannot come to disagree about what derives from what.

### Changed

- The development harness moved out of the library into a `beamte-dev` crate.
  As an optional `dev` feature it could not survive publishing: it links
  treebank's native Python grammar, which carries `publish = false` on purpose,
  and an optional dependency still has to name a version cargo can resolve.
  Build it with `cargo run -p beamte-dev` rather than
  `cargo run --features dev`.
