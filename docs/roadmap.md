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

Angles are exact turns, displayed in degrees of 360. Phases about a throat
lie on the `60^-6` coordinate grid, and drawn values on the `2^-40` grid.

### Algebra and geometry

- Conformal geometric algebra `Cl(n+1, 1)` over `Q` (`xyl_core::cga`)
  provides points, versors and dilators. The nucleon poles described below
  are its origin and its point at infinity.
- Rotors (`xyl_core::rotor`) are rational quaternions, the even part of
  `Cl(3)`. Every manifold holds its frame as an exact rotor.
- Sphere measure (`xyl_core::measure`) gives the share of an n-sphere
  within an angle of an axis, on which caps and shares on a manifold are
  based.
- Harmonics on S^11 (`fibres`, `atoms`): degree `l` has
  `d(l) = C(l + 11, 11) - C(l + 9, 11)` directions (1, 12, 77, 352, ...).
  Shells and seals follow by exact binomial sums.
- Motion (`xyl_core::motion`, `xyl_core::event`): every change is a
  one-parameter motion with an exact phase at `t`. Boosts are modes
  `amp x base^(rate t)` with exact rapidity, and crossings are solved with
  certified bounds.
- Units (`xyl_core::units`) are `G = c = M_core = 1`, with the length unit
  `r_g` and one power of sixty per tier of the anchor chain.

### Nucleons: 3Z, 3Q and the polarity R

A kind is a nuclide `(Z, Q)`, where `Z` counts its protons and `Q` its
neutrons. Its mass is `Z + Q` nucleons, which gravity and inertia read
(`circuit::nucleus_weights`, `formation::mass_weight`). Chemistry, the
founding and the light read the frames instead.

- **3Z, the span.** Each proton spans three frames, split between the Z pole
  (held, coherence) and the Q pole (open, decoherence). The hold is
  `w = 3Z - u`, `u` the open (unpaired) frames (`compound::hold`,
  `compound::open`). Pairing moves frames from the Q pole to the Z pole with
  the span fixed. The closed share `h_Z = (3Z - u_Z) / 3Z` drives the capture
  chain (`elements::closed_share`). Amounts move in whole nuclei of `w`
  frames, so fusion conserves 3Z (`circuit::grain::nuclei_of`), and the
  stars' burn produces light with the 3Z hold conserved.
- **3Q, the capacity.** 3Q is three times the number of states in every
  shell the atom reaches: `cap = sum_n sum_(l < n) 2 d(l)` and
  `3Q = 3 cap`. The balance `R_Q = 6Z - 3Q` and its share
  `rho_Q = (2Z - cap) / cap` mark the seals where `rho_Q = +1`, at Z 2, 28,
  208 and 1092 (`xylarium observe seals`).
- **R, the polarity.** The electron phase state
  `R = kappa (1 - 2 frac(p))` lies in `[-kappa, kappa]`. Its sign is the
  hemisphere of the bounce's pulse `p` (in turns), and its magnitude is the
  electrons' coherence on S^11, `kappa = 2 G / ends` (`fibres`). It is zero
  for a kind with no charge (`circuit::electron_phase`).
- **The pole point.** A carrier's pole state is the conformal point `X(t)`
  at `t = u / w`, open over held. The Z pole is the origin `e_o`, the Q
  pole is the point at infinity `e_inf`, and the throat is at `t = 1`. The
  vacuum's inversion `X(t) -> X(1 / t)` swaps the poles. An echo's flux is
  the dilator `D(rho)`, toward Z where `rho < 1` and toward Q where
  `rho > 1`, and the books close where the composed channel versors equal
  the observed one (`circuit/poles.rs`).

Next, in order:

1. A rotor inverse and the versor angle are to be added to
   `xyl_core::rotor`, so that relations are read through the versor between
   frames (milestone 2, phase 4) rather than by `acos` in
   `manifold::turn_between` and `chain::Place::arc`.
2. The atom's catalogue columns (mass number, phase temperatures, density)
   are to be derived from 3Z, 3Q and R rather than read from
   `data/elements.tsv` (milestone 3, chemistry from the census).

## 1. Founding in the core's units (done)

Goal: A universe is founded from its package and its seed in the universal
core's units, `G = c = M_core = 1`, and every layer above reads the
founding.

Why this order: Every tier above reads the founding, so its units come
first.

Landed:

- `package.rs` holds the inputs of a founding that the laws do not derive:
  the core's mass, the SI seam and the founding digest. `disc.rs` holds the
  star cut and the worlds on the star's manifold, which are built on the
  founding and are not part of it.
- The founding cache is keyed by a digest of the founding files
  (`XYL_FOUNDING_DIGEST`), so a change to a world, disc, life or kernel law
  does not invalidate the cached founding.
- The genesis record v2 holds the seed, the founding digest, the package
  and the founding's census kind by kind (`Substrate::census`).
- The element table extends to `Z = 1700` (`elements::TOP_Z`).
- The firmament carries the universal state: the throat's held kinds, each
  in its founded proportion, are scaled so that their mass is 1
  (`circuit::Substrate::found`).
- Galactic cores are held in the universal core's units. A galactic core's
  mass is `hole_mass x state` (`cores::founding_of`), and
  `Substrate::holds_a_star` reads the bead's state directly; neither
  divides by the mean bead state.

Done when: `xylarium observe cores` lists every galactic core's hole and
its mass ratio to the universal core, with each core no heavier than the
universal core, and `observe core` shows the substrate's books closing.

## 2. Worlds on the manifold chain

Goal: Every mass is an anchor whose manifold is the arc that the anchor's
pull holds against the tier above. Bodies gather on an anchor's manifold
where a body's matter holds a region against that anchor's twist, read at
every degree, with no seed draw and no partition.

Why this order: Peoples are created on worlds (`Cosmic::wake` reads each
world at a mind's epoch), so the civilization and game layers depend on
the world layer.

Landed:

- Phase 1: The galaxy's stars are the lit share of its founding
  (`galaxy::stellar_mass`).
- Phase 2: Worlds form on the star's manifold (`disc::manifold_worlds`),
  read on the derived stellar mass. The disc partition has been removed.
- Phase 3: Every world anchors a manifold. Its moons gather there by the
  same law one tier down, against the star's tide at the world.
- Phase 5: A star is born along the star formation of its ring
  (`chem::Chemistry::birth_along`), so stars are born with their ring's
  metals.

Next:

1. The manifold's disc-mass scale: The scale is the share of the
   manifold's matter that the star keeps, so that a metal-rich star's
   worlds settle as several bodies rather than one.
2. Phase 4, cross-ties: A relation between two bodies is read once, in the
   lowest tier holding both, through the versor between their frames. Each
   manifold already holds its frame as an exact rotor. This first needs a
   rotor inverse and the angle read from the relating versor's action in
   `xyl_core::rotor`, a self-contained piece of exact algebra suitable as a
   first contribution. The versor then replaces the `acos` readings in
   `manifold::turn_between` and `chain::Place::arc`, and `manifold::gap` is
   read in the lowest common tier.

Done when: `xylarium observe worlds` shows metal-rich stars holding several
worlds, and `observe system S` reads each relation in the lowest tier
holding both ends.

## 3. Rules from the founding and the galactic cores (current)

Goal: Every rule above the founding is a quantity derived from the
universal core's founding and the galactic cores. No law reads a measured
property of the Earth, the Sun or the Milky Way. SI units appear only at
display, through `package::Si`, and the universal physical constants (c,
G, k_B, h) are the only measured values the laws read.

Why this order: The planet layer, the chemistry and the civilization
rulers were ported from a prototype that ran in Earth masses, astronomical
units, solar masses and Earth years, with the core's mass set to the
measured 4.3e6 solar masses. Each phase below removes one of those units
or calibrations and states the founding quantity that replaces it. Phases
are ordered by dependency: time and length first, because every later
phase is written in them; then mass; then light, which needs mass and
length; then the chemistry, geochemistry and life rulers that read light.

Measure: `scripts/calibration-anchors` counts the remaining anchors in
code by category (441 at the start of the milestone). Each phase drives
its categories to zero and is checked with one targeted reading on the
cached test genesis.

Landed:

- Phase 0, builds: the founding and forecast digests hash code only
  (`crates/xyl-cosmos/build.rs`), so documentation edits reuse the cached
  founding, and the anchor counter reads code only by category.

Next:

1. Phase 1, time from the bell. The engine's unit of time is the ring (one
   bell), not the Gyr, which is 10^9 Earth years (`Si::yr_s`). The bell in
   core time is exact from the founding, `bell = 2 rim 60^5` in units of
   `r_g / c` (`circuit::anchor::Anchor`). A time is a position on the
   temporal coordinate (`xyl_core::coordinate`, the T lane): whole rings
   plus the phase within the ring, written in degrees, minutes and seconds
   of arc. A second is therefore a second of arc of phase, and local time
   between orbital bodies is their relative phase: a world's year is one
   turn of its orbit and its day one turn of its spin, each read against
   the ring of its anchor. SI seconds exist only in the SI seam
   (`package::Si`), which converts universal constants given in SI and
   displays Gyr. The conversion runs bottom-up: founding and substrate,
   galaxy and stars, systems and worlds, the kernel, then the display. The
   calibration epochs 13.8 Gyr (`chem::Rulers::today`) and 4.57 Gyr
   (`bodies/isotopes`) leave with the calibrations they belong to, in
   phases 5 and 6. Converted so far: the founding and substrate, then the
   galaxy tier (`galaxy`, `stars`, `chem`, `centre`, `pulse`, `field`,
   `turning`, `manifold`), whose rulers declared in Gyr are converted once
   where they are declared (`turning::span_in_rings`,
   `turning::rate_per_ring`).
2. Phase 2, length from the tiers. Orbits in astronomical units (`a_au`,
   `Si::au`) and radii in Earth radii (`Si::world_radius_m`) are replaced
   by the system tier's unit (`60 r_g`) and the planet tier's unit
   (`r_g / 60^2`) from `xyl_core::units::Tier`, with conversions between
   tiers by `manifold::dilation`.
3. Phase 3, mass down the founding's chain. The solar and Earth mass units
   (`Si::star_mass_kg`, `Si::world_mass_kg`) and the core's mass in suns
   (`package::hole_mass`) are replaced by shares of the chain: the
   universal core is 1; a galactic core is its bead's state
   (`circuit::Bead`); a star is the share of its piece's infall it holds
   (`disc::star_cut`), which replaces the initial mass function's bounds of
   0.08 and 60 suns (`galaxy::imf`); a world is its share of the star's
   manifold (`disc::manifold_worlds`); a moon is its share of the world's.
   A bead is a galaxy where its lit holding is at least one star's mass on
   that chain, in place of "at least one sun" (`Substrate::holds_a_star`),
   and only where its core is no heavier than the universal core
   (`M >= m`).
4. Phase 4, light and temperature from the burn. A star's luminosity is the
   hold its burn transduces per bell (`stars::burn_per_bell`), in core
   units. The flux at a world is that light times the world's received
   share at its reach (`circuit::vacuum_share`, the deficit law), and its
   temperature follows from the flux through the radiation law with the
   universal constants. This replaces the Earth calibrations 278 K
   (`Spec::t_form`, `worlds::t_unit`), 340 W/m^2 (`life`, `growth`,
   `hydro`, `worlds`) and 288 K (`lexicon`, `percept`), and the greenhouse
   fit `greenhouse_tau` is replaced by the atmosphere's clustering and
   containment.
5. Phase 5, chemistry from the census. The solar reference metallicity
   (`chem::Rulers::reference_metallicity`, 0.0142), the Milky Way gas
   pattern (`ism_today_dex`) and the flat rotation speed (`v_flat`,
   220 km/s) are replaced by the founding's census
   (`circuit::Substrate::census`), the home galaxy's enrichment along each
   ring (`chem::Chemistry::birth_along`) and the turning law
   `v = c delta(<r)` (`turning`). The light kinds are read from the same
   founding as the firmament.
6. Phase 6, geochemistry from the atoms. The bulk-silicate partition
   (`systems::MANTLE_KEPT`, McDonough 2003), the measured heat-producing
   isotopes (`bodies/isotopes`) and the unit world (`worlds/unit.rs`) are
   replaced by each element's hold and open frames (`compound::hold`,
   `compound::open`) against iron's, and by the isotopes of each star's
   birth mixture. `worlds/unit.rs` is deleted.
7. Phase 7, life and peoples from their worlds. The mind-bearing body of
   10 kg, its 19.1 W metabolism and 14.2-year generation, the viable people
   of 500, the 18 amu exhaust and the mass ratio of 10 (`life`,
   `xyl-kernel growth`, `propulsion`) are derived from each world's
   gravity, temperature and chemistry.
8. Phase 8, the package at the seam. `package::hole_mass` is supplied by
   the package and read only for display; the founding is read from a
   manifest of nuclei per `Z` ([network.md](network.md)); `package::Si`
   keeps only the universal constants and the display conversions.

Done when: `scripts/calibration-anchors` reports zero in every category,
and `xylarium observe worlds` and `observe chemistry` print quantities in
the founding's units with SI shown as a display conversion.

## 4. Seats for minds

Goal: External agents, AI or human, hold seats by identity and act through
them with the same interfaces as the built-in rule (`Holder::Universe`).

Why this order: A seat's subject is a people or a realm, and both are
derived from the worlds (2), so seat identity is only stable once worlds
are.

Landed: Seats exist for sovereign, steward, progenitor and ancient. Seats
on a people receive typed percepts (`percept::perceive`). The `Focus` and
`Handle` decisions are applied by `Universe::decide`. Seats taken by keyed
minds are recorded inputs (`Input::Take`) and are applied again on
restart.

Next:

1. Seats are to be keyed by their subject (realm key, people tag) rather
   than by index, so that steward seats keep their identity across a
   restart.
2. A holder check in `Universe::decide`: A decision is accepted only from
   the seat's holder.
3. Steward seats are to receive percepts, and the progenitor and ancient
   seats are to make decisions.
4. The `xylarium` CLI is to gain seat commands.
5. Players: A seat can be taken as a named player.

Done when: `xylarium observe house` shows a seat held by a mind across a
restart, and a decision from anyone but the holder is refused.

## 5. The xyl-hypha crate

Goal: A universe joins the hypha as a domain through a `xyl-hypha` crate
pinned to Mycelium `v0.2.0`, implementing `run::Link` over an SDK client
session ([network.md](network.md)). Mu, the Mycelium network's unit,
enters a universe only through an uplink, which is not part of this
repository.

Why this order: The crate transmits seat decisions (4) to the network, so
seats come first; mu accounting for work and passages needs both the seats
and the uplink session.

Next:

1. The `xyl-hypha` crate and its client session are to be written.
2. Ledger entries: Each committed ring is paired with a full TUniversal
   point, a commitment to the universe's state and the previous entry's
   hash.
3. The whole point is to be kept in `Checkpoint::commits`.
4. The kernel's coordinate lanes are to be named apart from the network's.
5. Published passage mouths are to carry freshness, and crossings are to be
   applied at an anchor.

Done when: `xylarium observe clock` shows the pace in rings per final point,
read from final points received through the crate.

## 6. Game economies

Goal: Economies are built on the network of passages between peoples
([civilizations.md](civilizations.md)).

Why this order: Trade and tolls read the worlds (2) and are directed by
seat decisions (4), and any exchange in mu passes through the uplink (5).

Landed: Peoples, numbers, knowledge, histories (annals, turnings, eminent
figures), drives, constellations, ley lines sited by the worth law, realms
and stewards, handling, discord, quarantine and folding are implemented.
The toll law is quoted in energy (`transit::quote`).

Next:

1. Settlement beyond the home system: Peoples settle other stars' worlds
   over drives and ley lines.
2. Tolls: Tolls accrue as traffic is carried and are booked against each
   line.
3. Trade: Peoples exchange goods, information and technology, with the
   exchange derived from each people's mind.
4. Exchange and its institutions: Banks and currencies are derived from
   numbers, power and knowledge.
5. Conflict: Conflict arises where taking is worth more than trading, and
   communications are bounded by the fastest physical carrier.
6. Names: Seated minds name figures and peoples in place of the seed's
   placeholder syllables.

Done when: `xylarium observe ley` shows tolls booked per line and peoples
settled beyond their home systems.

## Engineering

- The cluster id encoding (`compound.rs`: `Z_LIM = 1024`, counts below
  `N_LIM = 45`) is to be widened so that the whole firmament can form
  clusters.
- The anchor is to be held per universe rather than per process
  (`circuit::anchor`), so that a process can found more than one universe.
- `clippy::indexing_slicing` is to be enforced across the simulation
  crates.

## Declared rulers

The constants below are declared rather than derived. They are listed by
layer, each with the quantity that is to derive it.

- **Package** (`package.rs`): The universal core's mass, 4.3e6 suns
  (`hole_mass`), is declared until the package supplies it. The SI seam
  (`Si`: the unit world, the sun, the AU, the kpc, c, G, the year and the
  physical constants) is kept for display once each layer reads core units.
- **Home galaxy and chemistry** (`galaxy`, `chem`): The vertical orbit's
  scales and the chemistry's rulers (`chem::Rulers`: ring width, surface
  density, returns, loading, yields) are to be derived from the enrichment
  that the founding produces.
- **Systems and worlds** (`systems`, `worlds`, `bodies`, `turning`): The
  light's warmth `278 L / a` (`Spec::t_form`), the world scales and
  cosmic-ray reference (`worlds::crowd_r0`), the laboratory rock and iron
  (`bodies`), the rigidity bodies (`turning::rigidity`), the measured
  element rows Z 1-99 (`data/elements.tsv`) and the fixed volatile species
  are to be derived from each world's chemistry. In the element table,
  Z 100-150 are predicted rows, and rows past Z 150 are generated by the
  periodic law.
- **Life** (`life`): The viable people (500) is to be derived from each
  world's chemistry.
- **Growth** (`life`, `xyl-kernel growth::Rulers`): The mind-bearing body
  of 10 kg (`m_mind_kg` in `life`, read as `growth::Rulers::m_mind`), the
  unit metabolism it sets (19.1 W), the unit generation (14.2 yr),
  `eta_primary = 0.015`, `trophic = 0.1` and the flux ruler 340 W/m^2 are
  to be derived from each people's chemistry.
- **Laws** (`laws`): Each constant is expressed as a ratio to a measured
  value and is to be stated in core units.
- **Civilizations** (`xyl-kernel`): The mass ratio 10, the 18 amu exhaust
  and the fission share (`propulsion`), the knowledge unit's thousand minds
  (`knowledge`), the trait thresholds (`cosmic/emergence.rs`) and the
  memory threshold `e^-3` (`history::forgotten`) are to be derived from
  each people's senses and economy.
- **Resolution** (choices, not laws): The galaxy sample (`SAMPLE_CAP`,
  65,536), the nucleus sample, the 360 degrees of a star's life, the 64
  sample stars searched for bells (`SAMPLE` in xyl-kernel `field.rs`) and
  the ley lines per reading (`LINES_PER_READING`, 1024) are resolution
  choices.
