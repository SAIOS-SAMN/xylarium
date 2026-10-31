# Concepts

A first read for new contributors: what Xylarium computes, in what order,
and the few ideas everything else builds on. Each term is introduced once
here in ordinary words and linked to its precise entry in the
[glossary](glossary.md). The full model is in [model.md](model.md) and
[civilizations.md](civilizations.md); every formula is in
[EQUATIONS.md](EQUATIONS.md).

Xylarium's physics is the game's own model. It borrows the names of real
physics (black holes, elements, orbits) and their general behaviour, but it is a
self-consistent rule set rather than a simulation of the real universe.

## The pipeline

A universe is computed in layers. Each layer reads only the layers beneath
it, and nothing is placed by hand: every galaxy, star and world is the
result of a rule applied to the layer below.

```
  seed
   |
   v
  Founding        universal core, elements, firmament, galactic cores
   |              (xyl-cosmos: bang, bounce, elements, atoms, circuit)
   v
  Galaxies        rings of gas, star formation, stars on orbits
   |              (galaxy, chem, stars)
   v
  Systems         each star's disc, its worlds and their moons
   |              (disc, systems, manifold)
   v
  Worlds          air, seas, temperature, radiation over the star's life
   |              (bodies, worlds, hydro)
   v
  Life and minds  when, if ever, a world's life produces minds
   |              (life, minds)
   v
  Play            peoples, knowledge, growth, travel, seats, histories
                  (xyl-kernel)
```

The first three layers are computed at genesis and cached; the rest are
computed lazily, when an event or a reading needs them.

## Exact numbers

Every value the simulation keeps is an exact fraction (`Q`). A value that
is not a fraction, such as pi or a square root, is kept as a pair of
fractions proven to bracket it (`Encl`). Heavy intermediate calculations use
a fixed-size binary fraction (`Dy`) and are rounded to a fraction before
they are stored. Floating point is denied by the workspace's Clippy lints.

The result is that the same seed produces the same universe on every
machine and thread count, and that conservation can be checked exactly:
the parts of a quantity add up to the whole with no rounding error. Tests
and readings check these balances (the glossary calls them
[books](glossary.md)).

## Units: the core is 1

All physics is measured against one object, the universal core: the
central black hole the universe is founded from. Its mass, the speed of
light and the gravitational constant are all 1 (`G = c = M_core = 1`), so
the unit of length is the core's gravitational radius, `r_g`. Kilograms,
metres and years appear only when a value is displayed
(`package::Si`).

Distances span a huge range, so space is divided into
[tiers](glossary.md) (universe, cluster, galaxy, system, planet), each with
its own unit: a power of sixty of `r_g` (`xyl_core::units::Tier`). A law
works in the unit of its tier.

## The deficit share

One formula recurs across the model: the share of influence a mass `M`
holds at distance `R`,

```
delta(R, M) = M / (R + M)
```

It is 0 far away, approaches 1 close in, and never needs a square root or a
power. Orbital speeds, the region a planet's gravity controls, where worlds
form and how far a core's push reaches are all written with it
([deficit](glossary.md)).

## Time: the bell and the ring

The universe's clock is the [bell](glossary.md): the time light takes to
cross from the core to the edge of the home galaxy and back. One full bell
is a [ring](glossary.md), and every count of time in the engine is a whole
number of rings. Live play (`xylarium run`) advances one ring per loop.

## The founding

The founding is what a seed determines before any galaxy exists
([model.md, section 3](model.md#3-the-bounce-and-the-founding)):

1. **The parent's spin.** The seed draws one number, the spin of the black
   hole the universe bounces from. Everything else follows from it.
2. **Elements.** Nuclei are built one capture at a time from hydrogen; how
   many of each element survive is set by how completely its electron
   shells close (`elements::founded`). The element table runs to Z = 1700.
3. **The firmament.** A band of heavy elements stays bound at the core
   ([firmament](glossary.md)); its total mass is scaled to 1, the core's
   own.
4. **Beads and galactic cores.** The matter thrown out settles in shells
   that pinch into [beads](glossary.md); a bead heavy enough to light a
   star becomes a galaxy, and its mass sets that galaxy's core.

Atoms in the model are described by three numbers: protons (Z), neutrons
(Q) and the state of the electrons (R). Chemistry reads how many electron
slots an atom fills and leaves open; gravity reads only its mass, Z + Q.
The details are in [roadmap.md, Mathematics](roadmap.md#mathematics).

## Galaxies, stars and worlds

The home galaxy surrounds the universal core. Its rings are the founding's
beads, read at the galaxy tier; stars form along each ring with the
elements that ring has accumulated by the time of their birth
(`chem::Chemistry::birth_along`).

Every body controls a region around it, its [manifold](glossary.md): the
arc of its orbit its own gravity holds against the body it orbits. Worlds
form on a star's manifold. Reading outward degree by degree, matter
accumulates until it controls a region against the star's tide, and a world
forms there (`disc::manifold_worlds`). Moons form on each world's manifold
by the same rule. Matter that never controls a region is a
[belt](glossary.md). Nearby worlds whose orbits disturb each other enough
merge (`systems::settle_about`).

## Life and minds

On a world whose chemistry can copy molecules, life advances up a ladder of
[rungs](glossary.md) (`life::Ladder`), each a step that stores more
information with fewer copying errors. The forecast of minds
(`minds::schedule`) computes, in closed form, when a world's life would
reach the rung that holds a mind. At that time, if the world can still feed
a viable population, a [people](glossary.md) is created.

## Play

The kernel (`crates/xyl-kernel`) runs the universe as an event queue: it
jumps from one event to the next (a people appearing, a ley line opening, a
recorded input) and computes nothing in between. At each step it
[carries](glossary.md) every people forward: population, knowledge,
housing, travel and history (`civ::Civilizations::carry`).

- **Peoples** grow toward what their world can house, accumulate knowledge
  in domains (biology, materials, energy, information), and move when their
  world fails.
- **Travel** is orbital within a system, along arcs within a constellation
  of stars, and over [ley lines](glossary.md) between constellations.
- **Seats** are roles (sovereign, steward, progenitor, ancient) that make
  decisions. Each is filled by a built-in rule until an external agent takes
  it through `run::Link`. Every decision is a recorded input, held in the
  checkpoint and applied again on restart.
- **Annals** record what happened: emergence, settlement, exodus, falls.

## Where to go next

| To | Read |
|---|---|
| build, run and read a universe | [README, Quick start](../README.md#quick-start) |
| make a first change | [CONTRIBUTING.md](../CONTRIBUTING.md), [development.md, Extending a tier](development.md#extending-a-tier) |
| understand a module | its `//!` header, then [architecture.md](architecture.md) |
| look up a term | [glossary.md](glossary.md) |
| find a formula | [EQUATIONS.md](EQUATIONS.md) |
| see what is planned | [roadmap.md](roadmap.md) |
