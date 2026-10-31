# Mycelium integration

[Mycelium](https://github.com/SAIOS-SAMN/mycelium) is a separate public
project; Xylarium integrates against its tagged release `v0.2.0`. The
network it runs is the hypha. This document describes Xylarium's side of the
integration only. Mycelium's own design is documented in its repository.

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
  (`crates/xyl-cosmos/src/package.rs`): what its laws do not derive. On the
  network the package is a manifest of nuclei per atomic number: counts
  only, for `Z <= 1700` (the element table's `elements::TOP_Z`). Xylarium
  converts the counts into the universal core's units itself
  (`G = c = M_core = 1`); no mass crosses the boundary. Today the package
  declares the universal core's mass (`package::hole_mass`), holds the SI
  seam (`package::Si`) and names the founding's digest
  (`package::founding_digest`); founding from a manifest-shaped recipe is
  on the [roadmap](roadmap.md).
- **The genesis record.** `xylarium genesis` writes `genesis` in the home
  (format `xylarium genesis v2`): the seed, the founding digest, the laws'
  and life's digests, the package's core mass and spin, and the founding's
  census kind by kind (`Substrate::census`). The founding digest covers only
  the files the founding is derived from, so a change to a layer above the
  founding keeps the genesis.

## The Link seam

`xylarium::run::Link` (`crates/xylarium/src/run.rs`) is the one seam between
a universe in play and an uplink (paths abridged):

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

What it carries today:

- **Rings.** `ring` is called after every ring of the bell with mutable
  access to the universe. Here an uplink lists seats, sends percepts as
  seat records (`percept::perceive`, `lexicon::records`) and lands its
  minds' decisions (`Universe::decide`, `Universe::queue`); see
  [civilizations.md](civilizations.md#seats).
- **Checkpoints.** `saved` is called after each checkpoint is written to the
  home, so the uplink can ship what it records.
- **Clock height.** Before each checkpoint the universe commits its keepers'
  chain with `height()`, the last final clock height the uplink heard
  (`Universe::commit_clocks`); local play passes `None`. The checkpoint
  keeps each commit as a count and that height (`Checkpoint::commits`).

## Time on the network

- **Free on its own axis.** The universe advances one ring of its bell per
  loop of live play, every event inside the ring at its own count. A ring is
  one full bell, the pulse's round trip from the throat to the rim and back;
  an echo is two bells.
- **Internal chronometry** (`crates/xyl-kernel/src/chronometry.rs`). Time
  inside a universe is kept by keepers chained outward from the epicenter
  (the universal core): for each galaxy a hub (its core), a deep core, an
  inner mantle and an outer rim. Each keeper runs at the well rate
  `r / (r + M(<r))` of the keeper inside it, nested wells multiplying, with
  the pulse's light delay from the epicenter. A slower keeper's slip, its
  coherence and its whole-turn rungs are read in closed form.
- **The chain.** At each commit every keeper appends a BLAKE3 entry citing
  its own previous entry and the head of the keeper inside it, with its slip
  against that keeper kept as a residual. Only the epicenter's entry carries
  the network's clock height, so only the epicenter's entry reaches the
  network.
- **Pace measured, never declared.** `Chronometry::pace` reads rings per
  final-point advance from the first and last commits that heard a height.
  A faster machine computes more rings between two final points. The
  reading is `xylarium observe clock` ("the bridge").

## The algebra and pole states

- `crates/xyl-core/src/cga.rs` is the conformal geometric algebra
  `Cl(n+1, 1)` over exact rationals (`xyl_core::Q`), used at `n = 3`:
  `Cl(4, 1)`, the algebra the hypha's shared libraries use. Values that
  cross the boundary are exact and compared by their decoded values.
- The pole model ([EQUATIONS.md](EQUATIONS.md#z-q-and-r-the-pole-model),
  "Z, Q and R"): the Z pole at the origin `e_o`, the Q pole at infinity
  `e_inf`, and a carrier's pole state the point `X(t)` on the pole axis at
  `t = U / W` (`circuit::pole_point` in `circuit/poles.rs`; the axes in
  `circuit/axes.rs`). An echo carries a state by a dilator;
  `Substrate::loop_flux` composes the booked channels on each firmament
  kind's pole state and reads the residual (`circuit::PoleFlux`).
- Quantities read with logarithms, such as the jet's onset per kind
  (`circuit::onset_echoes`), stay inside the kernel and never cross the
  boundary.
- The coordinate (`crates/xyl-core/src/coordinate.rs`) holds a body's
  readings on four lanes `{S, G, T, H}`, each an exact share of the turn
  written as six sexagesimal places.

## Mu

Mu is not represented in the kernel. It flows only through the uplink:
whatever a universe spends or earns on the network is settled there, and
in-game costs (ley lines, tolls, folds) are kept in physical units.

## Next steps

1. **An uplink that posts ledger entries.** Pair each committed ring with a
   full final TUniversal point, a commitment to the universe's state and the
   previous entry's hash, signed by the domain's ancient house. These are
   the universe's chronometric anchors on the network.
2. **Keep the whole point.** `Checkpoint::commits` holds a count and a bare
   height, `(Q, Option<u64>)`. Store the full TUniversal point instead, so
   its lanes are kept.
3. **Name the coordinate lanes apart.** The kernel's lanes mean a body's
   phase on its anchor's ring (S), its holding as a share of its anchor's
   (G) and its own beat (T), with H not yet read. On the network the same
   letters are spatial, gauge, temporal and holonomic. Give the kernel's
   lanes their own names, or map them explicitly at the seam.
4. **Freshness and crossings.** When passage mouths are published to the
   network, carry a freshness in final-point advances, and apply crossings
   at an anchor so their reconciliation closes.
5. **A `xyl-hypha` crate.** A workspace crate pinned to Mycelium `v0.2.0`
   that implements `run::Link` over the SDK client session. Not built yet.
