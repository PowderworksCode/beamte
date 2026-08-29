# beamte

Test-quality rules over treebank trees. Trees in, findings out.

Beamte takes a parsed test file and returns findings about it: tests that
cannot fail when the code breaks, tests that fail when nothing broke, tests
whose failure tells you nothing. Every rule restates a post from the Google
Testing Blog, and every finding cites the post it was issued under.

It is a library. It parses nothing, reads no files, writes no output format
and owns no configuration — those are the host's, and
[straitjacket](https://github.com/PowderworksCode/straitjacket) is the first
host.

**[`DESIGN.md`](DESIGN.md) is the authoritative document** — the rule
catalogue and its provenance, the fidelity/resilience/precision spine, the
boundary between this library and its host, the substrate it assumes, and the
open questions. Start there.

## Development

```sh
cargo test                 # the library, with no parser and no dependencies
cargo test --features dev  # the rules, against real parsed source
```

`dev` adds a parser and a `beamte` binary so rules can be developed against
real files. It is off by default and never reaches a consumer.

```sh
cargo run --features dev -- check   some_test.py   # findings
cargo run --features dev -- explain some_test.py   # the tree, with roles
cargo run --features dev -- rules                  # the catalogue
```

`explain` is the one that matters: when a rule misfires, the finding tells you
nothing and the tree tells you everything.
