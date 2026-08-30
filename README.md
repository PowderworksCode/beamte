# beamte

Test-quality rules over treebank trees. Trees in, findings out.

Beamte takes a parsed test file and returns findings about it: tests that
cannot fail when the code breaks, tests that fail when nothing broke, tests
whose failure tells you nothing. Every rule restates a post from the Google
Testing Blog, and every finding cites the post it was issued under.

Every rule declares its scope. Most read test bodies; `env-read` reads any
source file, flagging code that reads the process environment where nothing
declares it — the input no signature admits to, and the reason a small test
above it can never be hermetic.

It is a library. It parses nothing, reads no files, writes no output format
and owns no configuration — those are the host's, and
[straitjacket](https://github.com/PowderworksCode/straitjacket) is the first
host.

**[`notes/DESIGN.md`](notes/DESIGN.md) is the authoritative document** — the rule
catalogue and its provenance, the fidelity/resilience/precision spine, the
boundary between this library and its host, the substrate it assumes, and the
open questions. Start there.

## Development

`scripts/dev.sh` points git at the committed hooks and runs both halves of the
gate in the order CI checks them, starting with the no-default-features build,
which is the claim the crate makes about itself.

```sh
scripts/dev.sh

cargo test                 # the library, with no parser and no dependencies
cargo test --workspace     # the rules, against real parsed source
```

`dev` adds a parser and a `beamte` binary so rules can be developed against
real files. It is off by default and never reaches a consumer. It parses with
treebank's own grammar and reads roles out of the manifests that grammar ships,
so a rule is exercised against treebank's answers rather than a table written
here.

treebank is not on crates.io yet, so it is taken as a git dependency. That
needs no credentials — the repository is public — but it does mean beamte
cannot be published until treebank is, which is why the crate is
`publish = false` for now.

```sh
cargo run -p beamte-dev -- check   some_test.py   # findings
cargo run -p beamte-dev -- explain some_test.py   # the tree, with roles
cargo run -p beamte-dev -- rules                  # the catalogue
```

`explain` is the one that matters: when a rule misfires, the finding tells you
nothing and the tree tells you everything.
