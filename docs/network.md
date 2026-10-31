# Mycelium integration

[Mycelium](https://github.com/SAIOS-SAMN/mycelium) is a separate public project;
Xylarium integrates against its tagged release `v0.2.0`. The network Mycelium
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
- **Clock height.** Before each checkpoint the universe commits its keepers'
  chain with `height()`, the last final clock height the uplink heard
  (`Universe::commit_clocks`); local play passes `None`. The checkpoint
  keeps each commit as a count and that height (`Checkpoint::commits`).

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
  its coherence and its whole-turn rungs are read in closed form.
- **The chain.** At each commit every keeper appends a BLAKE3 entry citing the
  previous entry of the same keeper and the head of the keeper inside it, with
  its slip against that keeper kept as a residual. Only the epicenter's entry
  carries the network's clock height, so only the epicenter's entry reaches the
  network.
- **Measured pace.** The kernel does not configure a ratio between rings and
  network time. `Chronometry::pace` computes rings per final-point advance from
  the first and last commits that heard a height; a faster machine computes more
  rings between two final points. Until two commits have heard different heights
  it returns `Pace::Unlinked`. `xylarium observe clock` prints the pace.

## The algebra and pole states

- `crates/xyl-core/src/cga.rs` is the conformal geometric algebra
  `Cl(n+1, 1)` over exact rationals (`xyl_core::Q`), used at `n = 3`:
  `Cl(4, 1)`, the algebra the hypha's shared libraries use. Values that
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
- The coordinate (`crates/xyl-core/src/coordinate.rs`) holds a body's
  readings on four lanes `{S, G, T, H}`, each an exact share of the turn
  written as six sexagesimal places.

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
3. **Name the coordinate lanes apart.** The kernel's lanes mean a body's
   phase on the ring of its anchor (S), its holding as a share of the
   holding of its anchor (G) and the beat of the body (T); H is not yet
   read. On the network the same letters mean spatial, gauge, temporal and
   holonomic. The kernel's lanes are to receive distinct names, or the two
   sets are to be mapped explicitly at the seam.
4. **Freshness and crossings.** When passage mouths are published to the
   network, each mouth is to carry a freshness in final-point advances, and
   crossings are to be applied at an anchor so that their reconciliation
   closes.
5. **A `xyl-hypha` crate.** A workspace crate pinned to Mycelium `v0.2.0`
   that implements `run::Link` over the SDK client session is planned.
