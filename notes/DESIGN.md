# Beamte — design

Beamte is a library of test-quality rules. It takes a parsed test file and
returns findings about it: tests that cannot fail when the code breaks, tests
that fail when nothing broke, tests whose failure tells you nothing.

Three ideas define it:

1. **The rules are not opinions.** Every rule in this document is a
   restatement of a post on the Google Testing Blog, which across nineteen
   years published something close to a specification: here is a test, here
   is the structural property that makes it bad, here is the same test
   without that property. Every finding cites the post it was issued under.
2. **One rule, every language.** Rules are written against treebank's node
   vocabulary — `_loop`, `_branch`, `_invocation`, `_callable` — which is
   enforced in the parse table rather than maintained by hand, and means the
   same term is the same thing in nine grammars. A rule is written once.
3. **A library, not a tool.** Beamte parses nothing, reads no files, writes
   no output format and owns no configuration. It takes a tree and returns
   values. Straitjacket is the first host; it is a consumer on ordinary
   terms, not a parent.

**This document is authoritative.** Where an implementation and this file
disagree, one of them is a bug, and which one is a decision to be recorded
here.

## 1. What beamte is, and what it is not

Beamte is a Rust library crate. Its published surface is a set of rules, a
trait for walking trees, and a `Finding` type.

It does not have a CLI, a SARIF writer, a config file, a suppression syntax,
exit codes, a file walker, or a GitHub Action. Not as a simplification —
straitjacket already has all of those, they are host concerns, and
duplicating them would create two places to fix each of them. §6 draws the
line concern by concern.

It does not have a parser. §7 explains why that is a design decision rather
than an omission, and §8 covers the grammars that exist for development only.

## 2. Why this is possible now

The reason a portable test linter has never existed is that *"a loop"* was
never portable. Every grammar names it something else, so a rule written
against Python's tree must be rewritten against Rust's, then TypeScript's,
then Java's, and the fifth rewrite is where the project dies. The existing
tools show the shape of that failure: `eslint-plugin-jest` knows nothing
about pytest, `pytest-style` knows nothing about JUnit, and each
re-implements a fraction of the same list with no rule shared between them.

Treebank removes the constraint, for its own reasons. These are real
supertypes in the parse table, not a convention in a query file:

| tier | terms beamte uses |
|---|---|
| table | `_statement` `_expression` `_declaration` `_control_flow` `_branch` `_loop` `_jump` `_assignment` `_invocation` `_access` `_literal` |
| facet | `_callable` `_binding` `_scope` `_argument` `_parameter` `_string` `_comment` `_identifier` |

Two properties of treebank matter more than the list itself. `treebank roles`
enforces closed lists, total node coverage and declared containments, so the
vocabulary cannot rot silently. And `treebank rosetta` fails when a role is
threaded in one grammar and forgotten in another — which is the exact failure
that would otherwise make a beamte rule silently correct in Python and
silently wrong in Ruby. Beamte's cross-language claim rests on a gate that
already exists and already runs.

## 3. Provenance

The rule set was derived by reading the Google Testing Blog archive in full:
404 posts, 2007-01 through 2026-06, pulled through the Blogger feed rather
than the JS-rendered index. Roughly forty state a falsifiable rule about test
structure. Eighteen of those are checkable from a syntax tree, and are §5.

Some posts hand over the detector verbatim. *Keep Tests Focused* (2018-06-11)
says: "after asserting the output of one call to the system under test, the
test makes another call to the system under test." That is a tree walk, not a
sentiment. *Change-Detector Tests Considered Harmful* (2015-01-27) defines a
worthless test as "a transformation of the same information in the code under
test" — which, if true, is a computation, and §5.4 makes it one.

Full citations are in §12. Every rule carries its citation as data, so a host
can put the post in the finding.

## 4. The spine: fidelity, resilience, precision

A rule set without a spine becomes a pile of unrelated nags with no account
of why any given rule earns its place. *Effective Testing* (2014-05-07)
supplies the spine and does not name it as such: three properties every test
should maximise. Every other post in the archive is a tactic for one of them.

| property | the test | a violation means |
|---|---|---|
| **fidelity** | when the code is broken, the test fails | the test does not do its job at all |
| **resilience** | the test fails *only* when the code is broken | the test works, but bills you forever |
| **precision** | when it fails, you know where to look | the failure is not actionable |

Every rule declares the property it defends. **Beamte stops there.** A
property is a fact about a rule; a severity is a policy about a repository,
and policy belongs to whoever is running the scan. Straitjacket maps the
three onto its own `Severity`; a different consumer may map them differently.

The taxonomy earns its place regardless of who assigns severity, because it
answers the only question that matters when a finding appears: not "rule 47
says so", but "this test cannot fail when your code breaks".

## 5. The catalogue

Tiers are about what a rule needs to see, and they decide what a host can
afford to run on every keystroke versus in CI.

- **Tier 1 — shape.** One file, one tree, no resolution.
- **Tier 2 — semantic.** Needs the import graph and the code under test.
- **Tier 3 — empirical.** Needs the suite to actually run; beamte's half is
  the ranking and the mutation targets.

Signals below are stated in vocabulary terms. Where a threshold appears as
*n*, it is deliberately unset until §9 stage 04 measures it.

### 5.1 Tier 1 — shape

#### `test-logic` — precision
*Don't Put Logic in Tests, 2014-07-31*

A loop, conditional, or computed expected value inside a test body. Tests are
concrete input/output pairs; the moment a test computes its own expectation
it can reproduce the bug it exists to catch. Google's example expects
`baseUrl + "/u/0/photos"` and thereby asserts a URL with a doubled slash,
which the test would have made obvious had it stated the value.

Signal: a `(_loop)` or `(_branch)` under the test `(_callable)`, or an
arithmetic or concatenation operator in an assertion argument.

#### `no-assertion` — fidelity
*Effective Testing, 2014-05-07*

A test body with zero assertion invocations. Fidelity is zero by
construction: the test can only fail by throwing.

Signal: `count((_invocation) where is-assertion) == 0`.

#### `multi-scenario` — precision
*Keep Tests Focused, 2018-06-11*

One test exercising several scenarios, so a failure does not say which.
Google states the detector directly, and it is cheap.

Signal: an assertion, followed by a later sibling `(_invocation)` on the same
receiver as that assertion's subject.

#### `boolean-assertion` — precision
*Test Failures Should Be Actionable, 2024-05-06*

`assertTrue(a == b)`, `assert x.ok()`, `expect(a === b).toBe(true)`. The
failure message can only say *expected true, got false*, when the assertion
library was willing to say *expected 5, got 0* or *NOT_FOUND:
/path/to/metadata.bin*. Mechanically fixable, so the finding should carry the
rewrite.

Signal: assertion argument is a comparison operator, or is an `(_invocation)`
returning a status type.

#### `default-values` — fidelity
*Choosing Values for Robust Tests, 2026-06-04*

Every literal in the test is its type's default — `0`, `""`, `false`, `null`,
`None`, empty collection. Such a test passes against an implementation that
ignores its input entirely; Google's example is a `MyMap::insert` that never
stores the value and is green because the default is `0`. No shipping linter
appears to check this.

Signal: all `(_literal)` in the test body are type-defaults.

#### `repeated-argument` — fidelity
*Choosing Values for Robust Tests, 2026-06-04*

The same literal passed to two distinct parameters of one call, so the test
cannot detect the arguments being swapped or one being dropped.
`insert(1, 1)` passes against `insert(k, k)`.

Signal: literal `(_argument)` values of one `(_invocation)` contain duplicates
across distinct parameters.

#### `wide-assertion` — resilience
*Prefer Narrow Assertions in Unit Tests, 2024-04-04*

Full-equality assertion against a composite literal, which implicitly tests
every unrelated behaviour — so adding a column to a table breaks it. Budgeted
rather than banned: Google explicitly allows roughly one per suite for the
golden case, and beamte should report the *n+1*th, not the first.

Signal: assertion expected side is a composite `(_literal)` with more than *n*
fields; budget counted per file.

#### `literal-free-body` — precision
*Tests Too DRY? Make Them DAMP!, 2019-12-03*

A test body containing no literal at all — every value arrives from a fixture
field or helper. The sharpest available proxy for over-DRY tests: a test you
cannot verify by reading, because nothing it asserts is visible in it.

Signal: `count((_literal) directly in test body) == 0` and assertions > 0.

#### `cause-effect-distance` — precision
*Keep Cause and Effect Clear, 2017-01-31*

An assertion on state whose last mutation happened in a fixture or `setUp`,
far from the assertion. Google's example puts the cause 200 lines from the
effect. Reported as a distance so the threshold is tunable rather than moral.

Signal: line distance between the last `(_assignment)` to the subject and the
assertion exceeds *n*, or the assignment is in fixture scope.

#### `name-shape` — precision
*Writing Descriptive Test Names, 2014-10-16; Naming Unit Tests Responsibly,
2007-02-01*

A test name giving a scenario but no expected outcome
(`isUserLockedOut_invalidLogin`), or merely restating the method under test.
The name should read as a sentence about the class:
*locks out user after three invalid logins*.

Signal: name has no outcome clause, or name is equal to a production symbol
name.

#### `non-hermetic` — resilience
*Test Sizes, 2010-12-13; Avoiding Flakey Tests, 2008-04-17*

In a test declared small: sleeps, wall-clock reads, unseeded randomness,
absolute paths, literal hosts or ports. Google's size table makes this a
contract rather than a preference — a small test gets no network, no
filesystem, no sleeps.

Signal: `(_invocation)` of a known non-hermetic callee, or a `(_literal)`
matching a path, URL or port.

### 5.2 Tier 2 — semantic

Every rule in this tier needs to resolve the test to the code under test.
That resolution is not yet designed; see §10.2, which gates the whole tier.

#### `change-detector` — fidelity
*Change-Detector Tests Considered Harmful, 2015-01-27*

The flagship. Detailed in §5.4.

#### `verify-query` — resilience
*Only Verify State-Changing Method Calls, 2017-12-11*

Verifying that a non-state-changing method was called.
`verify(db).getPermissions(user)` asserts nothing about the world; it freezes
today's implementation and gives false confidence, because calling a method
is not the same as doing the right thing with its return value. Google
supplies the naming heuristic: get / is / has / find / read / list.

Signal: `verify(m).x()` where `x` matches a query prefix and its return value
is unused in the test.

Legitimate exception, named in the post: verifying a query call to prove an
RPC is cached. This is why suppression must exist and must carry a reason.

#### `over-specified-verify` — resilience
*Only Verify Relevant Method Arguments, 2018-06-26*

A verification pinning every argument to an exact literal when one of them is
the behaviour under test. Adding a field to a title bar then breaks every
test in the codebase that named it. The fix is argument matchers, and it is
suggestible.

Signal: verification arity >= 3, all arguments literal, no matcher present.

#### `mock-returns-mock` — fidelity
*Don't Overuse Mocks, 2013-05-28*

A stub whose return value is itself a mock: the code under test is reaching
through layers of object graph, and the test now encodes one assumption per
layer. Cheap to detect, high signal, and usually a design finding rather than
a test finding — which the message should say.

Signal: `when(m.x()).thenReturn(y)` where `y` is a mock. Report chain depth.

#### `foreign-mock` — fidelity
*Don't Mock Types You Don't Own, 2020-07-16*

Mocking a third-party type. The expectations hardcoded in the mock are a
guess about someone else's contract; when the library changes, the test keeps
passing and the product breaks. Needs one input: which module paths are
first-party.

Signal: mocked type resolves to an import outside the owned path set.

#### `hidden-relevant-value` — precision
*Include Only Relevant Details In Tests, 2023-10-30*

The value being asserted on never appears in the test — it is a default
buried in a helper, so the reader must leave the test to know whether it is
correct. The complement of `literal-free-body`, and the only rule that needs
real data flow.

Signal: the assertion's expected value traces to a helper default rather than
to an argument passed from the test body.

#### `handler-bypass` — fidelity
*Testing UI Logic? Follow the User!, 2020-10-26*

A UI test calling an event handler directly instead of dispatching the event.
Google's example is a Buy button that shipped disabled: the handler test
passed, because nothing ever clicked the button.

Signal: the test invokes a handler-named method on a component, with no
render or dispatch in the body.

### 5.3 Tier 3 — empirical

#### `survives-mutation` — fidelity
*Mutation Testing, 2021-04-12; Code Coverage Best Practices, 2020-08-07*

The adjudicator. Static analysis can only say a test *looks* worthless;
mutating the code it covers and watching it pass anyway proves it.

Mutation testing's problem has always been cost, and a static suspicion
ranking is exactly the prior that makes it affordable: mutate only the code
covered by tests that tiers 1 and 2 already flagged, rather than the whole
tree.

The split follows the same boundary as everything else. Beamte emits the
ranking and the spans worth mutating, because that is analysis over a tree.
Applying mutants and running a suite is not, and stays with the host.

Treebank's own `treebank mutate` carries the two lessons that make this
sound, and they transfer: mutate at token boundaries rather than byte offsets
(cutting inside an identifier mostly yields a different identifier, which
teaches nothing), and seed every run, because a fuzzer nobody can re-run is a
fuzzer whose findings cannot be confirmed.

### 5.4 `change-detector`, in detail

Google's 2015 post argues by analogy and then stops. A test that asserts the
source code line by line is obviously a checksum:

```
// Production                          // Test
def abs(i: Int)                        for (line in File(prod_source))
  return (i < 0) ? i * -1 : i            switch (line.number)
                                           1: assert line.content equals "def abs(i: Int)"
                                           2: assert line.content equals "  return (i < 0) ? i * -1 : i"
```

The claim is that this next test is the same thing — "a transformation of the
same information in the code under test":

```
// Production                          // Test
def process(w: Work)                   part1 = mock(FirstPart)
  firstPart.process(w)                 part2 = mock(SecondPart)
  secondPart.process(w)                Processor(part1, part2).process(w)
                                       verify_in_order
                                         was_called part1.process(w)
                                         was_called part2.process(w)
```

If it is a transformation, it is a computable one, and nobody computes it.

**The detector.** Parse the method under test; extract its ordered sequence of
`(_invocation)` nodes against injected collaborators. Parse the test; extract
its ordered sequence of verifications. If the two sequences are equal, the
test contains no information the production code does not already contain.
That is decidable, not a judgement call.

**Two corroborating signals**, required before firing, because sequence
equality alone will have false positives on genuinely interaction-shaped code:

1. every constructor argument of the system under test is a mock;
2. every assertion in the test is a verification, rather than a claim about a
   return value or about state.

Three of three, and the finding says what the post says: rewrite or delete.

**The finding carries evidence, not a verdict.** The correspondence is the
argument, so it should be visible: production call one against verification
one, call two against verification two. Straitjacket's `Finding` already
carries an `evidence: Vec<EvidenceStep>` and already renders it as a SARIF
code flow, so the shape needed here is the shape that exists.

This rule is the reason to build the library. The other seventeen are the
reason anyone adopts it.

## 6. The boundary

### 6.1 Concern by concern

Every entry in the straitjacket column is a file that exists today.

| concern | beamte | straitjacket |
|---|---|---|
| the rules | queries, analysis, thresholds | — |
| the test model | what marks a test, assertion, mock, fixture | — |
| vocabulary | via `treebank-core` | — |
| parsing | none — takes a tree | engine, packs, cache, `tb_*` ABI |
| file walk | — | `src/walk.rs` |
| configuration | plain constructor arguments | `straitjacket.toml`, `src/config.rs` |
| suppression | — | `src/suppression.rs` |
| severity | declares a property per rule | maps property to `Severity` |
| output | returns values | text, JSON, SARIF — `src/report.rs` |
| exit codes, `--no-fail` | — | `src/main.rs` |
| GitHub Action | — | `action.yml` |
| agent instructions | supplies each rule's sentence and citation | renders them — `src/instructions.rs` |

Straitjacket depends on beamte from crates.io, with no path dependency. This
is not decoration: straitjacket's field guide already forbids sibling path
deps, after the tool once could not build outside a full `powderworks/` tree.
Beamte must stand alone from the first commit, and being a real dependency of
a real consumer is what proves it does.

### 6.2 The surface

Staying this small is the design constraint that matters most.

```rust
// no io, no engine, no config file, no formatting

pub trait Node<'t> { /* kind, role, span, children, field */ }

pub struct Unit<'t, N> {
    /* a test file's tree and text; optionally the tree of the
       code under test, for the tier 2 rules */
}

pub fn inspect<'t, N: Node<'t>>(unit: &Unit<'t, N>, model: &TestModel) -> Vec<Finding>;
pub fn rules() -> &'static [Rule];   // id · property · sentence · citation

pub struct Finding { rule, property, span, message, help, evidence }
pub struct Citation { title, url, date }
```

`TestModel` is passed in rather than read from anywhere. It carries the
framework table of §9 stage 02, and taking it as an argument is what lets a
host extend it — a project with custom matchers or its own assertion wrappers
needs to say so, and that statement is configuration, which is the host's.

### 6.3 What the host must do

Four obligations sit with whoever runs the rules, and they decide whether
anyone keeps them switched on.

1. **Ratchet, don't gate.** Record current counts per rule as a baseline;
   fail only on an increase. Treebank already does exactly this per grammar
   with `lint_policy.toml`. A checker that demands a clean sweep on day one
   against an existing suite is removed on day two.
2. **Suppression states a reason**, with a bare marker itself a finding.
   Every rule here has exceptions Google names explicitly — one broad
   assertion per suite, verifying a query call to prove caching. Make
   declaring one cheap and hiding one impossible.
3. **Findings cite the post.** It converts an argument with a linter into a
   much shorter argument with Titus Winters, and makes the rule set auditable
   rather than personal.
4. **Render the rules as instructions.** Straitjacket's `instruction` hook is
   its best idea and matters more here: telling an agent the rules before it
   writes the tests is worth more than flagging them afterwards.

## 7. Substrate

### 7.1 Beamte does not own a parser

The tempting shape is for beamte to load its own grammar packs. It is the
wrong one, because straitjacket is about to have several structural rules
rather than one: the parked eyebrow rule needs HTML, beamte needs Python,
Rust and TypeScript now and more later. If every analysis library carried its
own engine, one scan would stand up two wasmer instances, two module caches,
and JIT the same pack twice in one process.

So the engine belongs to the host. Beamte takes a tree and returns findings:
it opens no files, resolves no packs, contacts no registry, and never links
wasmer. Its dependencies are `treebank-core` for the vocabulary, and a narrow
node-access trait that any treebank tree satisfies — native tree-sitter in
beamte's own tests, the `tb_*` ABI inside straitjacket.

Three consequences, all of them what make a library worth depending on. Its
test suite needs no wasm, no network and no toolchain, because its inputs are
trees. Its dependency footprint is small enough that taking it on is not a
decision. And it cannot break a consumer's musl release build, which is
exactly what linked tree-sitter did to the eyebrow rule.

### 7.2 What the host pays, once

Straitjacket's `notes/wasm_pack_plan.md` (2026-08-13) already took the
decision that structural rules parse with treebank wasm packs rather than
tree-sitter crates linked as C, and measured it. Those numbers were taken for
the eyebrow rule, on one machine; they are recorded here as the context that
makes the arrangement affordable, not as claims about beamte.

| measured | value | consequence here |
|---|---|---|
| parse vs native | 50–52 ms vs 32 ms, 1.6× | on a 696 KB TSX file; small files land at 0.7 ms |
| JIT per grammar | 90–160 ms, once per process | then 0.3 ms from a cranelift artifact keyed on (pack digest, wasmer version, target) |
| pack sizes | 0.67 / 0.94 / 1.75 MB | Python / Rust / TypeScript; fetched per language, never all at once |
| engine weight | 207 crates | the host's cost, paid once whether one grammar loads or nine |

Two things make even the 1.6× close to irrelevant for this workload. Beamte
only ever looks at test files, a small fraction of any repository. And the
prefilter is unusually strong: whether a file is a test is answerable from
its path and one substring, before any parse. Straitjacket measured its
equivalent heading prefilter skipping ~98% of files.

The linked-C alternative is not hypothetical. tree-sitter grammars are
generated C, and linking them broke both musl release targets in `cc-rs`
looking for a cross toolchain that straitjacket's `.cargo/config.toml` exists
to avoid needing. That is why the eyebrow rule is parked.

### 7.3 A missing grammar is "not read"

A language with no pack available must be reported as **not read**, visibly —
never silently clean. Same doctrine as treebank's own walk, where a sweep
over less than the requested tree must not be able to report success, and the
same shape the eyebrow rule already uses, holding the gap in one function.

Beamte's part of this is to make the gap expressible: a unit it was not given
a tree for is a reported condition, not an empty finding list.

## 8. Development

### 8.1 A CLI and real grammars, behind a feature nobody turns on

Rules cannot be written blind. Developing `multi-scenario` means running it
against a real file and seeing what the analysis saw, repeatedly, in a loop
measured in seconds. So beamte carries a small binary and real grammars for
exactly that, behind an optional `dev` feature, off by default, with the
binary marked `required-features = ["dev"]`.

This costs consumers nothing, and that is a fact about Cargo rather than a
hope: dev-dependencies do not propagate to dependents, and an unselected
optional feature is not compiled. Straitjacket's dependency tree, binary size
and musl cross-build are untouched by anything in this section — which is
precisely why the same grammars that could not live *in* straitjacket are
safe here. What broke the eyebrow rule was a real dependency; this is not
one.

The dev backend uses the **native** grammar crates, not packs: no engine, no
registry, no cache, no digest resolution inside a loop that has to be fast.
That choice pays twice, because the node trait then has two independent
implementations — native tree-sitter here, the `tb_*` ABI in straitjacket —
which is the cheapest available proof that the trait is an abstraction rather
than a mirror of whichever backend was written first.

### 8.2 The dev surface

| surface | purpose |
|---|---|
| `beamte check <file>` | run the rules, print findings plainly; the inner loop |
| `beamte explain <file>` | dump the test body with its roles annotated — what the analysis *saw*, not what it concluded. When a rule misfires the finding tells you nothing and the tree tells you everything. |
| `tests/fixtures/` | small, deliberately bad test files paired with expected findings |

Fixtures are written DAMP — literal, self-contained, no loops — because a
fixture corpus for a tool that bans logic in tests must not contain logic in
tests.

### 8.3 Gates

1. **Default-feature build.** CI builds the library with default features, so
   nothing dev-gated can quietly become reachable from the published API.
2. **Beamte clean on itself.** The match to straitjacket's existing
   "straitjacket clean on itself" gate, and not decorative: a test-quality
   checker whose own tests break its rules is not a credible instrument, and
   these rules are strict enough that the gate will find things.
3. **Native and pack agree.** See §10.1 — the one thing in this arrangement
   that must be verified rather than assumed.

## 9. Order of work

| stage | work |
|---|---|
| 01 | **The node interface and dev harness.** The narrow trait, its native tree-sitter implementation, `check`, `explain`, the fixture layout — all behind `--features dev`. Design the trait against the `tb_*` ABI's shape so straitjacket's implementation is thin, but do not import it. |
| 02 | **The test model.** One table: how each framework marks a test, assertion, mock and fixture — pytest `test_*`, JUnit `@Test`, Rust `#[test]`, Go `TestXxx`, jest `it()`. The only framework-specific layer in the design, and it doubles as the prefilter. Keeping it a data table rather than code is what stops this fragmenting the way the existing plugins did. |
| 03 | **The four free rules.** `test-logic`, `no-assertion`, `multi-scenario`, `boolean-assertion`. Pure shape, no resolution, near-zero false positives, written against the vocabulary rather than any one grammar. Already past what anything currently ships. |
| 04 | **Corpus calibration.** Run stage 03 across ~1,000 real packages via `treebank-corpus`, which already ranks and fetches an ecosystem's top packages with reproducible provenance. Every *n* in §5 gets a measured percentile; every rule gets a false-positive rate; rules firing on a fifth of all real tests get cut or demoted before anyone else sees them. |
| 05 | **Straitjacket takes the dependency.** Engine, pack resolution and cache, the trait over `tb_*`, property mapped to `Severity`, findings through the existing suppression, reporting and instruction machinery, the ratchet baseline. Shared with the eyebrow port. Worth doing at four rules rather than eighteen — it finds every place beamte quietly assumed something about its host. |
| 06 | **Resolution, then tier 2.** §10.2 first, then `change-detector` and the rest. The hard part, attempted with easy rules in production and a real consumer attached. |
| 07 | **Mutation adjudication.** The ranking and the mutation targets; the host applies and runs. |

## 10. Open questions

### 10.1 Do a native grammar crate and its wasm pack build the same tree?

The dev harness parses natively and the host parses through a pack. If those
ever disagree, beamte's entire suite is green about the wrong tree. This is
the only real risk in §8.

Treebank's CI already asserts that a pack is byte-reproducible, loads in a
real WASI runtime, and describes itself accurately; straitjacket's bench found
wasmi and wasmer producing byte-identical s-expressions of the same parse.
Native-crate against wasm-pack is a different claim from either, and needs its
own fixture: same file, both backends, compare the full s-expression.

### 10.2 How does a test find the code under test?

Every tier 2 rule needs it and none of them can start without it. Candidates,
probably in this order: a naming convention (`FooTest` → `Foo`,
`test_foo.py` → `foo.py`), the import graph of the test file, and an explicit
mapping supplied by the host. All three will be wrong somewhere, so the
design question is not which to pick but what beamte does when resolution
fails — and the answer should be the §7.3 answer: report it as unresolved,
never silently pass.

### 10.3 How much data flow does `hidden-relevant-value` need?

Full data flow is out of scope for a library this size. The likely bound is
intra-procedural plus one level into a helper, giving up visibly rather than
guessing. Needs to be settled before the rule is written, not during.

### 10.4 Repositories with several frameworks

The test model assumes a file can be attributed to one framework. Mixed repos
exist, and a wrong attribution produces confident nonsense rather than a
miss. Probably per-file detection with an explicit host override, but it is
unproven.

### 10.5 Assertion vocabularies are open-ended

Custom matchers and project-local assertion wrappers are common and invisible
to a fixed table. §6.2 takes `TestModel` as an argument partly for this, but
whether a project can practically describe its own wrappers, and whether an
undescribed wrapper degrades to `no-assertion` false positives, is not
settled. `no-assertion` may need to be conservative until stage 04 says
otherwise.

## 11. The name

A *Beamter* is a tenured civil servant: someone who checks paperwork against
a published regulation, impersonally, and is not empowered to be talked
round. The right neighbour for straitjacket — one is the institution's
restraint, the other its clerk.

It also settles questions of tone that would otherwise be relitigated per
rule. A finding is not advice and does not hedge. It cites the regulation it
was issued under, because these rules are not the author's opinion but a
published nineteen-year standard. And the clerk has no view on whether a
finding matters to you: it reports that the paperwork is not in order, and
what happens next is another office.

Which is the naming argument for §6 as well. Beamte issues findings; it does
not decide their consequence, hear appeals, or keep the register. Suppression
markers and ratchet baselines are straitjacket's, because those are rulings
on a finding rather than the finding itself.

## 12. References

All from the Google Testing Blog, `testing.googleblog.com`.

| date | post |
|---|---|
| 2007-02-01 | [Naming Unit Tests Responsibly](https://testing.googleblog.com/2007/02/tott-naming-unit-tests-responsibly.html) |
| 2008-04-17 | [Avoiding Flakey Tests](https://testing.googleblog.com/2008/04/tott-avoiding-flakey-tests.html) |
| 2010-12-13 | [Test Sizes](https://testing.googleblog.com/2010/12/test-sizes.html) |
| 2013-05-28 | [Don't Overuse Mocks](https://testing.googleblog.com/2013/05/testing-on-toilet-dont-overuse-mocks.html) |
| 2013-08-05 | [Test Behavior, Not Implementation](https://testing.googleblog.com/2013/08/testing-on-toilet-test-behavior-not.html) |
| 2014-05-07 | [Effective Testing](https://testing.googleblog.com/2014/05/testing-on-toilet-effective-testing.html) |
| 2014-07-31 | [Don't Put Logic in Tests](https://testing.googleblog.com/2014/07/testing-on-toilet-dont-put-logic-in.html) |
| 2014-10-16 | [Writing Descriptive Test Names](https://testing.googleblog.com/2014/10/testing-on-toilet-writing-descriptive.html) |
| 2015-01-14 | [Prefer Testing Public APIs Over Implementation-Detail Classes](https://testing.googleblog.com/2015/01/testing-on-toilet-prefer-testing-public.html) |
| 2015-01-27 | [Change-Detector Tests Considered Harmful](https://testing.googleblog.com/2015/01/testing-on-toilet-change-detector-tests.html) |
| 2017-01-31 | [Keep Cause and Effect Clear](https://testing.googleblog.com/2017/01/testing-on-toilet-keep-cause-and-effect.html) |
| 2017-12-11 | [Only Verify State-Changing Method Calls](https://testing.googleblog.com/2017/12/testing-on-toilet-only-verify-state.html) |
| 2018-02-20 | [Cleanly Create Test Data](https://testing.googleblog.com/2018/02/testing-on-toilet-cleanly-create-test.html) |
| 2018-06-11 | [Keep Tests Focused](https://testing.googleblog.com/2018/06/testing-on-toilet-keep-tests-focused.html) |
| 2018-06-26 | [Only Verify Relevant Method Arguments](https://testing.googleblog.com/2018/06/testing-on-toilet-only-verify-relevant.html) |
| 2019-12-03 | [Tests Too DRY? Make Them DAMP!](https://testing.googleblog.com/2019/12/testing-on-toilet-tests-too-dry-make.html) |
| 2020-07-16 | [Don't Mock Types You Don't Own](https://testing.googleblog.com/2020/07/testing-on-toilet-dont-mock-types-you.html) |
| 2020-08-07 | [Code Coverage Best Practices](https://testing.googleblog.com/2020/08/code-coverage-best-practices.html) |
| 2020-10-26 | [Testing UI Logic? Follow the User!](https://testing.googleblog.com/2020/10/testing-on-toilet-testing-ui-logic.html) |
| 2021-04-12 | [Mutation Testing](https://testing.googleblog.com/2021/04/mutation-testing.html) |
| 2023-10-30 | [Include Only Relevant Details In Tests](https://testing.googleblog.com/2023/10/include-only-relevant-details-in-tests.html) |
| 2024-02-27 | [Increase Test Fidelity By Avoiding Mocks](https://testing.googleblog.com/2024/02/increase-test-fidelity-by-avoiding-mocks.html) |
| 2024-04-04 | [Prefer Narrow Assertions in Unit Tests](https://testing.googleblog.com/2024/04/prefer-narrow-assertions-in-unit-tests.html) |
| 2024-04-18 | [How I Learned To Stop Writing Brittle Tests and Love Expressive APIs](https://testing.googleblog.com/2024/04/how-i-learned-to-stop-writing-brittle.html) |
| 2024-05-06 | [Test Failures Should Be Actionable](https://testing.googleblog.com/2024/05/test-failures-should-be-actionable.html) |
| 2024-10-15 | [SMURF: Beyond the Test Pyramid](https://testing.googleblog.com/2024/10/smurf-beyond-test-pyramid.html) |
| 2026-06-04 | [Choosing Values for Robust Tests](https://testing.googleblog.com/2026/06/choosing-values-for-robust-tests.html) |
