# Mycelium integration

[Mycelium](https://github.com/SAIOS-SAMN/mycelium) is a separate public project.
Xylarium integrates against its tagged releases, the stable contract the
projects on the network pin to (its `mycelium-sdk` crate, the wire version
and act table, and the shared physics library's law ids). The uplink is
pinned to `v0.3.0`; `v0.4.0-rc.1` (wire `0x0200`, law version 5) is tagged
and the move to it is open
([#82](https://github.com/SAIOS-SAMN/xylarium/issues/82)). The
network Mycelium
runs is called the [hypha](glossary.md). This document describes Xylarium's side
of the integration only. The design of Mycelium is documented in the Mycelium
repository.

No crate in this workspace depends on Mycelium, and the uplink that connects
a universe to the hypha is not part of this repository. Without an uplink,
`xylarium run` plays a universe locally.

## How a kernel plugs in

- **A client session.** The uplink opens an SDK client session with the
  hypha and runs the universe's live play beside it (`run::run_with` with a
  `run::Link`, below).
- **The universe as a domain.** One universe, founded from one package and
  one seed, is one domain. Keys on the seat surface are hashed under the
  domain (`lexicon::people_key`), so a mind sees peoples only as keys.
- **The founding package.** A universe is founded from a package
  (`crates/xyl-cosmos/src/package.rs`), which holds the quantities that the
  laws do not derive. On the network the package is a manifest of nuclei
  per atomic number. It holds counts only, for `Z <= 1700` (the element
  table's `elements::TOP_Z`). Xylarium converts the counts into the units
  of the universal core (`G = c = M_core = 1`); no mass crosses the
  boundary. Today the package
  declares the universal core's mass (`package::hole_mass`), holds the SI
  seam (`package::Si`) and names the founding's digest
  (`package::founding_digest`); founding from a manifest-shaped recipe is
  on the [roadmap](roadmap.md).
- **The genesis record.** `xylarium genesis` writes `genesis` in the home
  (format `xylarium genesis v2`). The record holds the seed, the founding
  digest, the digests of the laws and of life, the package's core mass and spin,
  and the founding's census kind by kind (`Substrate::census`). The founding
  digest covers only the files the founding is derived from, so a change to a
  layer above the founding keeps the genesis.

## The uplink

The uplink is the one program that links Mycelium. It is built on
`mycelium-sdk` and on this workspace's crates, and lives outside this
repository.

- **Where.** A separate git repository, `xylarium-uplink`, kept beside the
  kernel checkout. It is private and not yet published or hosted remotely;
  it is to be published with Mycelium's shared libraries.
- **What.** One crate, `xyl-uplink`: the ancients' house on a hypha, the
  universal clock, the upward ledger, the endowment and the domain
  commands (`xyl-uplink run` for live play with the house as a
  `run::Link`, `xyl-uplink found-universe` to found a domain).
- **Dependencies.** `mycelium-sdk` pinned by git tag (`v0.3.0`), and
  `xyl-core`, `xyl-cosmos`, `xyl-kernel` and `xylarium` by relative path to
  the kernel checkout.
- **Status against v0.4.0-rc.1.** Not built: the release changes the wire
  (`0x0200`) and the law version (5), so the uplink cannot speak to a
  v0.4.0 hypha until it is moved (#82). The SDK's own conformance suite,
  run from this host with the kernel profile against Mycelium's test mesh,
  conforms in process and on the network (2026-11-10). That run signs
  with a scratch test voice: it is not a xylarium signing.
- **Signing the Concordance.** A kernel signs a clause by presenting its
  own reconstruction of each fixture under its key. Xylarium has signed
  none: it implements LAW-COORDINATE at v1 only, and has no FORMS-PASSAGE
  v4, FORMS-COUPLING or FORMS-TURNING reading. FORMS-PASSAGE v4 is a
  network clause, which a hosted domain's crossings need, so it is read
  first (#136). Enabling or vetoing the kernel clauses is the operator's
  decision.
- **Standards.** The uplink is to follow the same layout and process as
  this repository and Mycelium: a hosted repository, the shared commit
  identity, dependencies on the kernel by git rather than by path,
  AGENTS.md, CONTRIBUTING.md, a CHANGELOG, and the same checks
  ([#175](https://github.com/SAIOS-SAMN/xylarium/issues/175)).

## The Link seam

`xylarium::run::Link` (`crates/xylarium/src/run.rs`) is the only interface
between a universe in play and an uplink (paths abridged):

```rust
pub trait Link {
    /// After each ring, with the universe's bells and their relays.
    fn ring(&mut self, u: &mut Universe<'_>, bells: &[Node], relays: &[Relay]);
    /// At each checkpoint, after it is written.
    fn saved(&mut self, home: &Path, u: &Universe<'_>) -> Result<(), String>;
    /// The last final clock height heard.
    fn height(&self) -> Option<u64>;
}
```

The seam carries the following today.

- **Rings.** `ring` is called after every ring of the bell with mutable
  access to the universe. Here an uplink lists seats, sends percepts as
  seat records (`percept::perceive`, `lexicon::records`) and lands its
  minds' decisions (`Universe::decide`, `Universe::queue`); see
  [civilizations.md](civilizations.md#seats).
- **Checkpoints.** `saved` is called after each checkpoint is written to the
  home, so the uplink can ship what it records.
- **Clock points.** Before each checkpoint the universe commits its
  keepers' chain with `point()`, the last final TUniversal point the uplink
  heard, whole: its height, its lanes S G T H and K
  (`xyl_kernel::chronometry::Point`, `Universe::commit_clocks`); local play
  passes `None`. `origin()` gives the point heard with the universe's
  origin receipt, which anchors ring 0 (`Universe::anchor_clock`). The
  checkpoint keeps each commit as a count and its point
  (`Checkpoint::commits`) and the anchor (`Checkpoint::anchor`).

## A mind on both faces

A mind joined through the network holds an ancient seat
([civilizations.md](civilizations.md#minds-in-the-ancient-seats); #94).
Its cognition is the mycelium client's. Inside the universe it acts as the
house through `run::Link`: percepts out after every ring, decisions in as
recorded inputs. On the network its key, mu and receipts stand on the
hypha, held by the private uplink's session.

- The kernel never trusts the mind: every act is an input under the
  kernel's laws, and replaying the inputs re-derives the history.
- The hypha never thinks for the mind: it reconstructs what the mind claims
  and admits it only on reaching the same exact object.
- The uplink is where the two faces meet. It confirms a seat's holder
  against the mind's key, pairs each committed ring with its TUniversal
  point (Next steps, item 1), and settles mu; the kernel holds no mu.
- Both faces name a body by one coordinate (LAW-COORDINATE v1), so a
  percept and a ledger entry can refer to the same body without either
  reading the other's state.
- **Joined and embedded minds** (#108). A mind joins today as a separate
  process: the hypha (`mycelium-server`), the uplink's live play with the
  house (`xyl-uplink run`), and the mind (`mycelium-client --seat
  <domain> <seat-ids>`), which takes ancient seats, reads its percepts,
  calls `FOCUS` and answers the court's tasks. Planned: the uplink hosts
  minds in the uplink's process, the client's cognition loaded as a library, one
  per ancient seat. Each still takes its seat, reads its percepts and
  calls its seat's functions through the house's seat-surface path and
  signs under the mind's key, so the kernel sees the same recorded inputs as
  from a joined mind and the court reconstructs what it claims; the kernel
  takes no mycelium dependency.
- **What the uplink fixed first** (#107). A percept's axis coordinate
  `u` (Mycelium 78.156) is the universe's exact count, whole rings and the
  ring's phase, never floored to a ring. A mind's focus is four exact
  shares (`knowledge::Focus`), and the seat listing states the call's four
  arguments. The frame's lane byte is a vestige: Mycelium writes S on every
  act and nothing reads it, an act's lanes being the modes its content
  moves, so the uplink writes S until Mycelium retires the byte.

## Time on the network

- **Local time.** The universe advances one [ring](glossary.md) per loop
  of live play, independently of the network's clock, and applies every
  event inside the ring at the count of that event. A ring is one full
  [bell](glossary.md), the pulse's round trip from the throat to the rim and
  back; an [echo](glossary.md) is two bells.
- **Internal chronometry** (`crates/xyl-kernel/src/chronometry.rs`). Time
  inside a universe is kept by keepers chained outward from the epicenter
  (the universal core): for each galaxy a hub (its core), a deep core, an
  inner mantle and an outer rim. Each keeper runs at the well rate
  `r / (r + M(<r))` of the keeper inside it, nested wells multiplying, with
  the pulse's light delay from the epicenter. The slip of a slower keeper,
  its coherence and its rungs of 360 degrees are read in closed form.
- **The chain.** At each commit every keeper appends a BLAKE3 entry citing the
  previous entry of the same keeper and the head of the keeper inside it, with
  its slip against that keeper kept as a residual. Only the epicenter's entry
  carries the network's clock height, so only the epicenter's entry reaches the
  network.
- **Ring 0 on TUniversal.** The universe's ring 0 is anchored at the final
  point heard with its origin receipt (`Chronometry::anchor`), so every
  count of the bell has a place on the network: the anchor plus its rings.
  The present is read on the T lane as whole rings and the ring's phase in
  degrees (`Universe::t_coordinate`, written by `coordinate::beat`);
  `xylarium observe clock` prints the anchor.
- **Dilation per segment.** The binding to the network is on the height
  `h` (MU-CHRONOMETRY 3.3). The kernel does not configure a ratio between
  rings and network time: `Chronometry::dilations` reads, for each segment
  between consecutive commits that heard a point under one origin with the
  height advanced, the rings over the heights there, exact. It is never one
  ratio for the whole chain and never computed across a change of origin;
  a faster machine computes more rings between two heights, and the
  segment shows it. `xylarium observe clock` prints the last segments.

## The algebra and pole states

- `crates/xyl-core/src/cga.rs` is the geometric algebra `Cl(p, q)` over
  exact rationals (`xyl_core::Q`), the signature carried by the type
  `Mv<NEG>` so that two signatures never compose (Concordance 2.10). Its
  conformal model `Cl(n+1, 1)` is used at `n = 3`: `Cl(4, 1)`, the algebra
  the hypha's shared libraries use. The four passage fixtures of
  Concordance 2.11 hold in `Cl(3, 2)`. Values that
  cross the boundary are exact and compared by their decoded values.
- The pole model ([EQUATIONS.md](EQUATIONS.md#z-q-and-r-the-pole-model),
  "Z, Q and R"): the Z pole is at the origin `e_o`, the Q pole at infinity
  `e_inf`, and a carrier's pole state is the point `X(t)` on the pole axis at
  `t = U / W` (`circuit::pole_point` in `circuit/poles.rs`; the axes in
  `circuit/axes.rs`). An echo carries a state by a dilator;
  `Substrate::loop_flux` composes the booked channels on each firmament
  kind's pole state and reads the residual (`circuit::PoleFlux`).
- Quantities computed with logarithms, such as the jet's onset per kind
  (`circuit::onset_echoes`), stay inside the kernel and do not cross the
  boundary.
- The coordinate (`crates/xyl-core/src/coordinate.rs`) reads LAW-COORDINATE
  v1 (Mycelium proposal 78.173). Its lanes carry the network's meanings:
  S spatial (a body's holding share is read here by its dilation), G
  gauge (planned: pairing and the transductions), T temporal (the count in
  rings), H holonomic (a body's phase on its anchor's ring is one plane's
  turning). A lane is an exact rational of either sign; its sector at depth
  `d` is `[m 60^-d, (m + 1) 60^-d)`; `Reading::within` reads Below, Within
  or Above against another's sector and refuses two lanes by name. The
  kernel reads the law's fixtures 2, 3 and 4 as its tests; fixtures 1, 5
  and 6 (the origin, the ordered key and neighbour ranges) wait on bodies
  being addressed by coordinate (roadmap item 40).

## Mu

Mu is not represented in the kernel. It flows only through the uplink:
whatever a universe spends or earns on the network is settled there, and
in-game costs (ley lines, tolls, folds) are kept in physical units.

## Next steps

1. **An uplink that posts ledger entries.** The uplink is to pair each
   committed ring with a full final TUniversal point, a commitment to the
   universe's state and the hash of the previous entry, signed by the
   domain's ancient [house](glossary.md). These entries anchor the
   universe's time on the network.
2. **Keep the whole point.** `Checkpoint::commits` holds a count and a bare
   height, `(Q, Option<u64>)`. It is to store the full TUniversal point
   instead, so that the lanes of the point are kept.
3. **The coordinate's origin and ordered key.** The lanes carry the
   network's meanings (done). The origin, the ordered key and neighbour
   ranges of LAW-COORDINATE v1 (its fixtures 1, 5 and 6) are to come with
   bodies addressed by coordinate rather than by sample index (roadmap
   item 40).
4. **Freshness and crossings.** When passage mouths are published to the
   network, each mouth is to carry a freshness in final-point advances, and
   crossings are to be applied at an anchor so that their reconciliation
   closes.
5. **The uplink on v0.4.0.** Move the uplink's pin to `mycelium-sdk`
   v0.4.0 and pass the kernel profile's conformance read from the uplink
   itself ([#82](https://github.com/SAIOS-SAMN/xylarium/issues/82)); the
   uplink implements `run::Link` over the SDK session outside this
   workspace (#21, closed).
