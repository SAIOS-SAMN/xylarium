# Build-out

How the design becomes code. Every piece of work starts from a section of
the design, is planned as a milestone and an issue, is built on a branch,
reaches `main` through a pull request, and is done when a reading on a live
home witnesses it. The pull-request workflow itself is in
[CONTRIBUTING.md](CONTRIBUTING.md); the rules every change keeps are in
[AGENTS.md](AGENTS.md).

```text
design section -> milestone -> issue -> branch -> pull request -> witness
(docs/*.md)       (phase)     (done-when)  (#-name)  (Closes #N)   (observe)
```

## The chain

1. **Design section.** A law, interface or game rule is written first: its
   formula in [docs/model.md](docs/model.md) or
   [docs/civilizations.md](docs/civilizations.md), its row in
   [docs/EQUATIONS.md](docs/EQUATIONS.md), its interface in
   [docs/interfaces.md](docs/interfaces.md), its theorem in the register
   (model.md, Axioms and theorems). It is derived from the laws already
   built; an underived constant is a declared ruler, a law not built is a
   stand-in ([docs/assumptions.md](docs/assumptions.md)).
2. **Milestone.** Each milestone is a build-out phase named by what it
   delivers. Its description states its goal, its exit reading, its order
   of work and what it depends on. The phases and their governing docs are
   below.
3. **Issue.** Every issue names its design section, what exists (the
   sites), what to build, what it depends on (by issue number), and its
   done-when as a reading. A done issue is closed by the pull request that
   finishes it; a superseded one points to its successor.
4. **Branch.** Named for the issue (`264-seats-by-subject`), cut from
   `main`, holding the change with its tests and documentation.
5. **Pull request.** `Closes #N` or `Refs #N`; the checks run; for a change
   to a law, the same reading on the same seed before and after.
6. **Witness.** Done is a reading moved on a live home and checked for
   magnitude in its calibre, never a test alone. Tests guard invariants;
   the reading is the evidence.

## The phases

| Milestone | Delivers | Governing docs | Exit reading |
|---|---|---|---|
| [M3 Sound physics](https://github.com/SAIOS-SAMN/xylarium/milestone/2) | One calibre per share, no Earth rulers, time as coordinates | model.md (Calibre, Axioms and theorems), EQUATIONS.md (E032) | `observe universal-core`, `galaxy`, `tiers`, `proofs` in calibre; the seam in one ring (#280) |
| [v0.4.0](https://github.com/SAIOS-SAMN/xylarium/milestone/1) | The release at M3's exit | CHANGELOG.md | the release notes |
| [M4 The vortex universe](https://github.com/SAIOS-SAMN/xylarium/milestone/3) | Matter as vortices; pairs through the pivot; the founding as flow | model.md (The core as a paired throat, The ring), interfaces.md | every body reads its core, swirl, twist, writhe, linking; T5, T11, T12, T14 built |
| [M5 Physics as a kit](https://github.com/SAIOS-SAMN/xylarium/milestone/4) | A law changed by the package alone; two universes compared | network.md, model.md | a one-law package compared reading by reading |
| [M6 Realms](https://github.com/SAIOS-SAMN/xylarium/milestone/5) | Systems and constellations in every galaxy | model.md (worlds), roadmap.md section 2 | realms in more than one galaxy |
| [M7 Intelligence across the cosmos](https://github.com/SAIOS-SAMN/xylarium/milestone/6) | The game: seats, decisions, economies, minds | civilizations.md, network.md | minds seated in realms of more than one galaxy; play from the terminal |

M7's seats track (#264 to #267) depends only on M3's worlds standing in
their system's scale and runs alongside M4 to M6. Its economies track
(#271 to #277) reads the worlds M6 makes lawful. The issue graph is in each
issue's "Depends on".

## The gates

A pull request merges only when all of these pass, run from the
repository root:

| Gate | Command |
|---|---|
| Formatting | `cargo fmt --all --check` |
| Lints, floats denied | `cargo clippy --workspace --all-targets -- -D warnings` |
| Documentation builds | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` |
| Docs resolve to code; stand-ins and theorems agree | `scripts/check-docs` (runs `check-stand-ins` and `check-theorems`) |
| Style | `scripts/check-style` |
| Tests of the changed modules, once in the dev profile | `cargo test -p <crate> <module>::` |

A change to a law also updates, in the same pull request, the module
header, EQUATIONS.md, model.md or civilizations.md, the roadmap, the
glossary for a new term, and the CHANGELOG's Unreleased section; a theorem
it builds updates its register row. An issue labelled `needs-decision`
waits for the operator; one labelled `cross-project` waits for the other
node's agreement.

## The board

The [project board](https://github.com/orgs/SAIOS-SAMN/projects/1) holds
every open issue. Status is Todo, In Progress, Blocked (waiting on an issue
or a decision named in its body) or Done. Area is the crate; Doc is the
design section that governs the item; Seam is `none`, `mycelium` or
`netstratum` where the item crosses a node.

## The seams

| Seam | xylarium | Other side |
|---|---|---|
| Time: the front's arrival `tau(r) = r + H(<r)`, one ring at the equator | #245, #280 | netstratum #207 |
| The uplink: domain, manifest, ledger, the house's relay | network.md; #270, #108 | xylarium-uplink; mycelium |
| The core tube | network.md | netstratum #109 |
| The chronometric library and its telemetry: one form per quantity of time | [chronometry.md](docs/chronometry.md); #299 to #303 | the uplink's clock; netstratum's seam (`delta tau`) |
| Checkpoint segments as the ledger: one object, kept locally and posted | [checkpoints.md](docs/checkpoints.md); #285, #287 | xylarium-uplink #14; mycelium 78.154 |

## Labels

The labels are listed in [CONTRIBUTING.md](CONTRIBUTING.md#development-workflow);
`game` marks grand-strategy play (M7's tracks) and `time` the
time-as-coordinates work.
