# Roadmap

Xylarium is pre-alpha. This roadmap opens with the mathematics the laws are
built on, then lists the milestones in intended order. Each milestone states
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

## Mathematics

Xylarium holds no floating-point state: `clippy::float_arithmetic` is denied
across the workspace. The number systems, the algebra and the nucleon model
below carry every law; [EQUATIONS.md](EQUATIONS.md) gives each formula with
its source location.

### Number systems

| System | Role | Where |
|---|---|---|
| Integers (ZZ) | Everything counted: protons and neutrons, whole nuclei, frames, S^11 harmonics, the seal counts, the sexagesimal places of a coordinate, seeds and stream states | `u32` / `i64` indices; counts beyond a machine word as `Q` with denominator 1 (`num-bigint`); `atoms`, `fibres`, `circuit::grain`, `xyl_core::coordinate`, `xyl_core::rng` |
| Rationals (QQ) | All held state: element counts, firmament and bead amounts, reaches, spans, phases, the anchor, star and world masses | `xyl_core::Q`, lowest terms, `i64` on the stack and promoted to big integers on overflow; division is checked |
| Reals (RR) | Never held directly. A real value enters as a proven rational enclosure `lo <= x <= hi` (pi, roots, exp, ln, sin, cos), or as a dyadic reading `m 2^e` at a seam, rounded toward zero and gridded back to `Q` before it is held | `xyl_core::certified::Encl`; `xyl_core::dyadic::Dy` |

Angles are exact turns, displayed in degrees of 360; phases about a throat
sit on the `60^-6` coordinate grid, drawn values on the `2^-40` grid.

### Algebra and geometry

- Conformal geometric algebra `Cl(n+1, 1)` over `Q` (`xyl_core::cga`):
  points, versors and dilators; the nucleon poles below are its origin and
  its point at infinity.
- Rotors (`xyl_core::rotor`): rational quaternions, the even part of
  `Cl(3)`; every manifold holds its frame as an exact rotor.
- Sphere measure (`xyl_core::measure`): the share of an n-sphere within an
  angle of an axis, the basis of caps and shares on a manifold.
- Harmonics on S^11 (`fibres`, `atoms`): degree `l` has
  `d(l) = C(l + 11, 11) - C(l + 9, 11)` directions (1, 12, 77, 352, ...);
  shells and seals follow by exact binomial sums.
- Motion (`xyl_core::motion`, `xyl_core::event`): every change is a
  one-parameter motion with an exact phase at `t`; boosts are modes
  `amp x base^(rate t)` with exact rapidity, and crossings are solved with
  certified bounds.
- Units (`xyl_core::units`): `G = c = M_core = 1`, the length unit `r_g`,
  and one power of sixty per tier of the anchor chain.

### Nucleons: 3Z, 3Q and the polarity R

A kind is a nuclide `(Z, Q)`: `Z` its protons, `Q` its neutrons. Its mass
is `Z + Q` nucleons, which gravity and inertia read
(`circuit::nucleus_weights`, `formation::mass_weight`). Chemistry, the
founding and the light read the frames instead:

- **3Z, the span.** Each proton spans three frames, split between the Z pole
  (held, coherence) and the Q pole (open, decoherence). The hold is
  `w = 3Z - u`, `u` the open (unpaired) frames (`compound::hold`,
  `compound::open`). Pairing moves frames from the Q pole to the Z pole with
  the span fixed. The closed share `h_Z = (3Z - u_Z) / 3Z` drives the capture
  chain (`elements::closed_share`); amounts move in whole nuclei of `w`
  frames, so fusion conserves 3Z (`circuit::grain::nuclei_of`), and the
  stars' burn transduces light with the 3Z hold conserved.
- **3Q, the capacity.** Three times the room of every shape of every turning
  the atom reaches: `cap = sum_n sum_(l < n) 2 d(l)`, `3Q = 3 cap`. The
  balance `R_Q = 6Z - 3Q` and its share `rho_Q = (2Z - cap) / cap` mark the
  seals where `rho_Q = +1`, at Z 2, 28, 208 and 1092
  (`xylarium observe seals`).
- **R, the polarity.** The electron phase state
  `R = kappa (1 - 2 frac(p))` lies in `[-kappa, kappa]`: its sign is the
  hemisphere of the bounce's pulse `p` (in turns), its magnitude the
  electrons' coherence on S^11, `kappa = 2 G / ends` (`fibres`), and it is
  zero for a kind with no charge (`circuit::electron_phase`).
- **The pole point.** A carrier's pole state is the conformal point `X(t)`
  at `t = u / w`, open over held: the Z pole is the origin `e_o`, the Q pole
  the point at infinity `e_inf`, the throat `t = 1`. The vacuum's inversion
  `X(t) -> X(1 / t)` swaps the poles; an echo's flux is the dilator `D(rho)`,
  toward Z where `rho < 1` and toward Q where `rho > 1`, and the books close
  where the composed channel versors equal the observed one
  (`circuit/poles.rs`).

Next, in order:

1. Rotor inverse and the versor angle in `xyl_core::rotor`, so relations are
   read through the versor between frames (milestone 2, phase 4) rather
   than by `acos` in `manifold::turn_between` and `chain::Place::arc`.
2. The atom's catalogue columns (mass number, phase temperatures, density)
   derived from 3Z, 3Q and R rather than read from `data/elements.tsv`
   (milestone 3, chemistry from the census).

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
  disc, life or kernel law does not invalidate the cached founding.
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

Why this order: peoples are created on worlds (`Cosmic::wake` reads each
world at a mind's epoch), so the civilization and game layers depend on
the world layer.

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
manifest-founded universe, and `scripts/calibration-anchors` counts no anchors
in the founding and chemistry.

## 4. Seats for minds

Goal: external agents, AI or human, hold seats by identity and act through
them with the same interfaces as the built-in rule (`Holder::Universe`).

Why this order: a seat's subject is a people or a realm, and both are
derived from the worlds (2), so seat identity is only stable once worlds
are.

Landed: seats for sovereign, steward, progenitor and ancient; typed
percepts for seats on a people (`percept::perceive`); the `Focus` and
`Handle` decisions (`Universe::decide`); seats taken by keyed minds as
recorded inputs (`Input::Take`), applied again on restart.

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
pinned to Mycelium `v0.2.0`, implementing `run::Link` over an SDK client
session ([network.md](network.md)). Mu, the Mycelium network's unit,
enters a universe only through an uplink, which is not part of this
repository.

Why this order: the crate transmits seat decisions (4) to the network, so
seats come first; mu accounting for work and passages needs both the seats
and the uplink session.

Next:

1. The `xyl-hypha` crate and its client session.
2. Ledger entries: each committed ring paired with a full TUniversal point,
   a commitment to the universe's state and the previous entry's hash.
3. The whole point kept in `Checkpoint::commits`.
4. The kernel's coordinate lanes named apart from the network's.
5. Freshness for published passage mouths, and crossings applied at an
   anchor.

Done when: `xylarium observe clock` shows the pace in rings per final point,
read from final points received through the crate.

## 6. Game economies

Goal: economies built on the network of passages between peoples
([civilizations.md](civilizations.md)).

Why this order: trade and tolls read the worlds (2) and are directed by
seat decisions (4), and any exchange in mu passes through the uplink (5).

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
6. Figures and peoples named by seated minds in place of the seed's
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
- **Growth** (`life`, `xyl-kernel growth::Rulers`): the mind-bearing body, 10 kg
  (`m_mind_kg` in `life`, read as `growth::Rulers::m_mind`), and the unit
  metabolism it sets (19.1 W), the unit generation (14.2 yr), `eta_primary =
  0.015`, `trophic = 0.1` and the flux ruler 340 W/m^2; to derive from each
  people's own chemistry.
- **Laws** (`laws`): each constant expressed as a ratio to a measured value;
  to be stated in the universe's own units.
- **Civilizations** (`xyl-kernel`): the mass ratio 10, the 18 amu exhaust
  and the fission share (`propulsion`); the knowledge unit's thousand minds
  (`knowledge`); the trait thresholds (`cosmic.rs`); the memory threshold
  `e^-3` (`history::forgotten`); to derive from each people's own senses
  and economy.
- **Resolution** (choices, not laws): the galaxy sample (`SAMPLE_CAP`,
  65,536) and nucleus sample; 360 degrees of a star's life; the 64 bells
  searched (`field.rs`); ley lines per reading (`LINES_PER_READING`, 1024).
