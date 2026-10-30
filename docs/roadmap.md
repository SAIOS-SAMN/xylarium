# Roadmap

Status: pre-alpha. The milestones below are in intended order. Each states
its goal, why it comes where it does, what has landed, what is next, and
the `xylarium observe` reading that shows it done. Changes are recorded in
[CHANGELOG.md](../CHANGELOG.md). To build, found and run a universe, see
the [README](../README.md#quick-start).

## Design process

Every change follows the steps in
[development.md, Extending a tier](development.md#extending-a-tier): derive
the law from the tier below, land one tier per change, guard it with
targeted tests on the shared test genesis, and read it once on a running
universe with `xylarium observe`.

## 1. Founding in the core's units (done)

Goal: a universe is founded from its package and its seed in the universal
core's units, `G = c = M_core = 1`, and every layer above reads the
founding.

Why this order: every tier above reads the founding, so its units come
first.

Landed:

- `package.rs`: what a universe is founded from that its laws do not derive
  (the core's mass, the SI seam, the founding digest). `disc.rs`: the star
  cut and the worlds on the star's manifold, built on the founding and never
  part of it.
- The founding digest: the founding cache is keyed by a digest of the
  founding's own files (`XYL_FOUNDING_DIGEST`), so a change to a world,
  disc, life or kernel law keeps the genesis.
- The genesis record v2: the seed, the founding digest, the package and the
  founding's census kind by kind (`Substrate::census`).
- The element table to `Z = 1700` (`elements::TOP_Z`).
- The firmament carries the universal state: the throat's held kinds, each
  in its founded proportion, are scaled so their mass is 1
  (`circuit::Substrate::found`).
- Galactic cores in the universal core's units: a galactic core's mass is
  `hole_mass x state` (`cores::founding_of`), and `Substrate::holds_a_star`
  reads the bead's state directly; neither divides by the mean bead state.

Done when: `xylarium observe cores` lists every galactic core's hole and
its mass ratio to the universal core, each core no heavier than the
universal core, and
`observe core` shows the substrate's books closing.

## 2. Worlds on the manifold chain (current)

Goal: every mass is an anchor whose manifold is the arc its own pull holds
against the tier above; bodies gather on an anchor's manifold where their
matter holds its own region against that anchor's twist, read at every
degree, with no seed draw and no partition.

Why this order: worlds are where peoples wake, so the game layers wait on
them.

Landed:

- Phase 1: the galaxy's stars are the lit share of its founding
  (`galaxy::stellar_mass`).
- Phase 2: worlds on the star's manifold (`disc::manifold_worlds`), read on
  the derived stellar mass; the disc partition is gone.
- Phase 3: every world anchors its own manifold; its moons gather there by
  the same law one tier down, against the star's tide at the world.
- Phase 5: a star is born along its ring's own star formation
  (`chem::Chemistry::birth_along`), so stars are born with their ring's
  metals.

Next:

1. The manifold's disc-mass scale: how much of the manifold's matter is the
   star's to keep, so that a metal-rich star's worlds settle as several
   bodies rather than one.
2. Phase 4, cross-ties: a relation between two bodies read once, in the
   lowest tier holding both, through the versor between their frames. Each
   manifold already holds its frame as an exact rotor. This first needs a
   rotor inverse and the angle read from the relating versor's action in
   `xyl_core::rotor`, a self-contained piece of exact algebra and a good
   first contribution. The versor then replaces the `acos` readings in
   `manifold::turn_between` and `chain::Place::arc`, and `manifold::gap` is
   read in the lowest common tier.

Done when: `xylarium observe worlds` shows metal-rich stars holding several
worlds, and `observe system S` reads each relation in the lowest tier
holding both ends.

## 3. Founding from the package manifest

Goal: a universe is founded from a manifest of nuclei per atomic number,
and every calibration carried over from measured values is replaced by a
quantity the founding derives.

Why this order: it builds on the founding's units (1) and feeds the
chemistry the worlds (2) are made of.

Next:

1. The firmament-to-galaxies guard `M >= m`: a galaxy is held only where
   its core is no heavier than the universal core. Today galaxies are
   selected by `Substrate::holds_a_star` alone.
2. The founding from a manifest-shaped recipe: nuclei per `Z`, counts only,
   converted to core units by the kernel ([network.md](network.md)).
3. The light-element founding: the lightest kinds read from the same
   founding as the firmament.
4. Chemistry from the census and the stars' own burning: enrichment as the
   hold a star's burn returns to its ring, in place of `chem::Rulers` and
   the calibration against measured abundances.
5. The planet layer in core units: worlds' masses, radii and fluxes read in
   the core's units, with `package::Si` used for display only.

Done when: `xylarium observe founding` and `observe chemistry` read from a
manifest-founded universe, and `scripts/calibration-anchors` counts no anchors in
the founding and chemistry.

## 4. Seats for minds

Goal: external minds, AI or human, hold seats by identity and act through
them with the same interfaces as the universe's own rule.

Why this order: seats need stable worlds and peoples (2) to act on.

Landed: seats for sovereign, steward, progenitor and ancient; typed
percepts for seats on a people (`percept::perceive`); the `Focus` and
`Handle` decisions (`Universe::decide`); seats taken by keyed minds as
recorded inputs (`Input::Take`), laid back on restart.

Next:

1. Seats keyed by their subject (realm key, people tag) rather than by
   index, so steward seats keep their identity across a restart.
2. A holder check in `Universe::decide`: a decision is accepted only from
   the seat's holder.
3. Percepts for steward seats, and decisions for the progenitor and ancient
   seats.
4. Seat commands on the `xylarium` CLI.
5. Players: taking a seat as a named player.

Done when: `xylarium observe house` shows a seat held by a mind across a
restart, and a decision from anyone but the holder is refused.

## 5. The xyl-hypha crate

Goal: a universe joins the hypha as a domain through a `xyl-hypha` crate
pinned to mycelium `v0.2.0`, implementing `run::Link` over an SDK client
session ([network.md](network.md)). Mu, the mycelium network's unit,
enters a universe only through an uplink, which is not part of this
repository.

Why this order: the uplink carries what seats (4) decide; mu carried with
work and passages through the uplink needs both.

Next:

1. The `xyl-hypha` crate and its client session.
2. Ledger entries: each committed ring paired with a full TUniversal point,
   a commitment to the universe's state and the previous entry's hash.
3. The whole point kept in `Checkpoint::commits`.
4. The kernel's coordinate lanes named apart from the network's.
5. Freshness for published passage mouths, and crossings applied at an
   anchor.

Done when: `xylarium observe clock` shows the bridge's pace read from
final points heard through the crate.

## 6. Game economies

Goal: the network of passages between peoples grows into economies
([civilizations.md](civilizations.md)).

Why this order: economies are carried over the worlds (2), seats (4) and
network (5) that come before.

Landed: peoples, numbers, knowledge, histories (annals, turnings, eminent
figures), drives, constellations, ley lines sited by the worth law, realms
and stewards, handling, discord, quarantine and folding; the toll law
quoted in energy (`transit::quote`).

Next:

1. Settlement beyond the home system: other stars' worlds over drives and
   ley lines.
2. Tolls that accrue as traffic is carried and booked against each line.
3. Trade: exchange between peoples, carrying goods, information and
   technology, emergent from the shape of each people's mind.
4. Exchange and its institutions: banks and currencies emerging from
   numbers, power and knowledge.
5. Conflict, arising where taking is worth more than trading;
   communications bounded by the fastest physical carrier.
6. Figures and peoples named by their own minds in place of the seed's
   placeholder syllables.

Done when: `xylarium observe ley` shows tolls booked per line and peoples
settled beyond their home systems.

## Engineering

- Widen the cluster id encoding (`compound.rs`: `Z_LIM = 1024`, counts below
  `N_LIM = 45`) so the whole firmament can form clusters.
- One anchor per universe rather than per process (`circuit::anchor`), so a
  process can found more than one universe.
- Enforce `clippy::indexing_slicing` across the simulation crates.

## Declared rulers

Constants declared rather than derived, by layer, each with what derives it:

- **Package** (`package.rs`): the universal core's mass, 4.3e6 suns
  (`hole_mass`), until the package supplies it; the SI seam (`Si`: the unit
  world, the sun, the AU, the kpc, c, G, the year and the physical
  constants), kept for display once each layer reads core units.
- **Home galaxy and chemistry** (`galaxy`, `chem`): the vertical orbit's
  scales and the chemistry's rulers (`chem::Rulers`: ring width, surface
  density, returns, loading, yields), to derive from the founding's own
  enrichment.
- **Systems and worlds** (`systems`, `worlds`, `bodies`, `turning`): the
  light's warmth `278 L / a` (`Spec::t_form`), the world scales and
  cosmic-ray reference (`worlds::crowd_r0`), the laboratory rock and iron
  (`bodies`), the rigidity bodies (`turning::rigidity`), the measured element
  rows Z 1-99 (`data/elements.tsv`; Z 100-150 are predicted rows, and past
  150 the table is generated by the periodic law) and the fixed volatile
  species; to derive from each world's own chemistry.
- **Life** (`life`): the viable people (500); to derive from each world's
  chemistry.
- **Growth** (`life`, `xyl-kernel growth::Rulers`): the mind-bearing body,
  10 kg (`m_mind_kg` in `life`, read as `growth::Rulers::m_mind`), and the
  unit metabolism it sets (19.1 W), the unit generation (14.2 yr), `eta_primary = 0.015`,
  `trophic = 0.1` and the flux ruler 340 W/m^2; to derive from each people's
  own chemistry.
- **Laws** (`laws`): each constant expressed as a ratio to a measured value;
  to be stated as the universe's own.
- **Civilizations** (`xyl-kernel`): the mass ratio 10, the 18 amu exhaust
  and the fission share (`propulsion`); the knowledge unit's thousand minds
  (`knowledge`); the trait thresholds (`cosmic.rs`); the memory threshold
  `e^-3` (`history::forgotten`); to derive from each people's own senses
  and economy.
- **Resolution** (choices, not laws): the galaxy sample (`SAMPLE_CAP`,
  65,536) and nucleus sample; 360 degrees of a star's life; the 64 bells
  searched (`field.rs`); ley lines per reading (`LINES_PER_READING`, 1024).
