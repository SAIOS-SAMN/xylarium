# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Until 1.0.0, minor versions may change public APIs and the model's laws.

## Unreleased

### Changed

- Minimum supported Rust is 1.85, the version the locked dependencies
  require.
- README restructured; the roadmap opens with the mathematics (number
  systems, algebra and geometry, and the nucleon model: the 3Z span, the
  3Q capacity and the polarity R).

## 0.3.0 - 2026-10-30

First public release.

Every layer, from the founding to the kernel, now reads the universal
core's units (`G = c = M_core = 1`), and worlds form on the manifold chain.

### Added

- `xyl_cosmos::package` (`package.rs`): what a universe is founded from that
  its laws do not derive - the universal core's mass (`hole_mass`), the SI
  seam (`Si`) and the founding digest (`founding_digest`) - moved from
  `galaxy.rs` and `bodies.rs`.
- `xyl_cosmos::disc` (`disc.rs`): a star's cut of its piece's infall
  (`star_cut`) and the worlds on its manifold (`manifold_worlds`), moved
  from `formation.rs`. Built on the founding and never part of it.
- The founding digest (`XYL_FOUNDING_DIGEST`, `crates/xyl-cosmos/build.rs`):
  a digest of the founding's own files only (the Big Bang, the bounce, the
  elements and atoms, the cascade and lighting, the circuit, the package and
  the mathematics). The founding cache is keyed by it and the torsion share,
  so a change to a layer built on the founding keeps the cached founding.
- The genesis record v2 (`xylarium genesis`): it names the founding digest
  and the package, and carries the founding's census kind by kind
  (`Substrate::census`), with the held share, beads, galaxies, lit share and
  stellar mass.
- One test genesis (`xyl_cosmos::genesis::universe`): seed 7 founded once
  into one home under `target/genesis`; every test reads it.
- `chem::Chemistry::birth_along` and `Chemistry::cached`.
- `Anchor::galaxy_kpc`: the galaxy tier's ring in kiloparsecs.
- The worlds reading counts the stars without worlds by cause.
- Colonies: a sailing people's reach (open worlds of its system) and
  daughter peoples founded by settlers; an exodus goes to an unoccupied
  world.
- History: turnings from exodus, settlement, joining the network,
  quarantine, mastery and folding name eminent figures whose memory fades at
  the people's forgetting rate; held in checkpoints.
- Realms and stewards: realms form from ley-connected constellations, each
  with a steward seat that sets line handling (open, tolled, guarded,
  closed); passage reads handling for the passer.
- Space folding: peoples without lawful passage fold space when a passage
  is worth it at their own drive; folded energy is a breach share that
  raises discord toward quarantine.
- Exoduses, settlements and steward decisions are recorded inputs, laid back
  on restart.

### Changed

- Worlds form on the star's manifold (`disc::manifold_worlds`): the star's
  manifold is the arc its pull holds about the core, the star holds its
  gathering, and a world forms where its gathered matter's deficit share
  holds against the core's twist, read at every degree. This replaces the
  disc partition `formation::disc`.
- The galaxy's stellar mass is the lit share of its founding
  (`galaxy::stellar_mass`: `Lighting::lit` times the core's mass), in place
  of the galaxy holding what its core holds.
- Every world anchors its own manifold, and its moons gather there by the
  same law one tier down, against the star's tide at the world. The fixed
  moon budget (1e-4 of a giant) and moons for giants only are gone.
- A star is born along its ring's own star formation from first light
  (`chem::Chemistry::birth_along`), so stars carry their ring's metals at
  birth; previously every star was born at its ring's first light.
- A certified exponential below the resolution is enclosed by
  `[0, 2^-bits]` for `x <= -(bits + 1)` (`systems.rs`), exact and immediate.
- Settling counts each mass once: a pair that meets by the settled age
  merges; a pair that meets later stands as two bodies, with the meeting
  recorded on the lighter.
- The galaxy structure and the founding grain are cached only once the
  universe is founded.
- One reach per tier: every reach inside the home galaxy is read in galaxy
  rings (`stars::reach_rings`, `Star::reach_rings`); a star's reach is its
  own and is no longer summed with its galaxy's reach about the universal
  core.
- The ring of time is one full bell, the pulse's round trip from the throat
  to the rim and back; an echo is two rings.
- The rim is the outermost founding shell's outer edge,
  `reach x (1 + s_final)`, and the held share has one definition, by mass
  (`Substrate::held_share`), which `BigBang::of` reads.
- The element table extends to `Z = 1700` (`elements::TOP_Z`). The vessel's
  upper edge is the last seal of S^11 within the table (Z 1092). The
  founding's hydrogen count is derived as the fewest bits that found the
  sealing kind whole (`elements::founding_bits`), no longer a package input.
- The bell is derived from the founding: light's round trip to the home
  galaxy's rim, `2 rim_galaxy / c` (`circuit::anchor`), replacing two 15 kpc
  disc crossings. The pulse rides the filament across the universe within
  the same bell (3600 c, the ladder step between the galaxy and cluster
  tiers). The universe's curvature radius is `rim / pi`, the home galaxy's
  rim is the founding's rim read in the galaxy tier's unit, and galaxies sit
  at their founded reach.
- The substrate's push is the vacuum's share, `H / (R + H)` read at the
  reach pushed (`circuit::vacuum_share`), one half only at the throat
  (`circuit::throat_push`). The inbound echo (`Substrate::inbound`) swings
  every push as `1 + A_in(t)` (`Substrate::push_at`) and heats the home
  galaxy's gas.
- The home galaxy's gas is the founding's beads one tier down
  (`formation::lighting`): the disc's pieces on a circle and the bulge's in
  a ball (`measure::share_turns`); an infall gathers its own angle of a
  piece and grips it at its own rim, `m / (w + m)`; the pulse's heat is the
  ring the bell brings at the piece; a piece that never lights forms no
  stars, and nothing stands in for it.
- A star's cut of its piece's infall is read on the chain
  (`galaxy::disc_frame`, `disc::star_cut`): the disc's agitation is the
  pulse's heat on the disc, and the core-star tie is the galaxy's tide at
  the pinch. The Sun's disc mass, the 360 parts in `ln a`, `Spec::limits`
  and the stirred inner belt are gone.
- The jetstream is read: `Substrate::loop_flux` composes the filament's
  dilation, the vacuum's share and the electrons' phase state as dilators on
  each firmament kind's pole state (`circuit::PoleFlux`) and reads the
  residual.
- The parent's spin is admitted only where its torsion share holds the
  firmament whole against the pulse (`Substrate::least_share`,
  `bang::spin_of_share`); the genesis record states the margin.
- The Big Bang draws nothing but the spin. The lumpiness, the held share,
  the assembly and the star efficiency are read off the cascade; the
  chemistry is calibrated on the universe's own history
  (`chem::Calibration::of`); the Big Bang `Kind`s and the `--kind` option
  are gone; the home defaults to `var/universe-7`.
- Stars are fuelled by the pulse (`stars`): the burn per bell is the ring
  the bell brings at the star's reach times its own deficit at its rim; its
  light is that burn; its life and its giant's end are its hydrogen and
  helium steps' holds over the burn (`stars::life`, `stars::ends`); every
  ended star's remnant is a bell of the network (`stars::remnant_share`).
- Radiation and temperature are distinct readings: a world's flux is the
  bell's mean of its star's burn, the swing within a bell is the light's own
  reading (`stars::flicker`), and temperatures are the media's agitation.
- Minds wake by the chemistry's expectation, in closed form: rungs take
  their means, impacts stretch them to `(e^(mu m) - 1) / mu`, and the
  sterilising pulse falls at its hazard's expected time; the forecast's
  default sight is the census of the sample.
- Planetary settling is carried as ages: each pair's merge age is the first
  age its swing spans the gap (`World::merged_at`); starting eccentricities,
  tilts, nodes and periapses come from the settle's own swing and phase.
- The jet's onset per kind is the filament's dilation count to the throat
  (`circuit::onset_gyr`); the landing shares each kind by pull times phase
  coherence (`Substrate::landing`); the horizon's source carries the five
  isoclinic planes' spin (`bounce::spin_bivector`); the galactic jet's
  episode is the galaxy's own echo (`centre::galaxy_episode_myr`).
- The home galaxy's structure is read from the founding one tier down
  (`galaxy::Structure`); the Milky Way disc, bulge and nucleus rulers and the
  26,000-star sample are gone.
- The bounce's share is derived from the parent's spin by torsion,
  `a = (H + J) / (2H + J)` (`bang::torsion_share`), so every seed founds its
  own capture chain, rim and bell.
- The bounce's horizon is read in its S^360 enclosure
  (`bounce::curved_reach`) at the deficit share `delta_h = 1/3`
  (`bounce::horizon_reach`), and the sphere is built through the crossing
  point (`cga::sphere_through`). The founding precedes the Big Bang, is
  founded once per process (`cores::substrate_for`) and is cached in the
  home directory as `founding-*`.
- Civilization layer reduced to its emergent core in `civ`: one housing law
  (`civ::housing`), numbers as the logistic toward housing scaled by the
  radiation environment, history (`civ::history`), settlers when crowded
  past the logistic's inflection.
- Knowledge is solved in closed form (`knowledge::advance`), replacing
  backward-Euler steps.
- The unit body's generation is a named ruler,
  `growth::Rulers::generation_yr` (14.2 yr).
- Turnings carry one lowercase act set: `exodus`, `settlement`, `network`,
  `quarantine`, `biology`, `information`, `fold`.
- Whether space folds is the universe's own: `Scales::ftl` carries
  `Laws::ftl`, and `Network::fold` reads it from the galaxy's scales.
- The lexicon's solvent ids are derived from `worlds::Species::ALL` plus the
  melt and the plasma (`lexicon::solvents`).
- `civ::housing::housing` takes no appetite.
- Seats: one sovereign seat per people, and an ancient and a progenitor
  seat per ancient people.
- The firmament carries the universal state: the throat's held kinds, each
  in its founded proportion, are scaled so their mass is 1
  (`circuit::Substrate::found`).
- A galactic core's mass is `hole_mass x state`, its bead's state read in
  the universal core's units (`cores::founding_of`), and
  `Substrate::holds_a_star` reads the state directly; previously both divided
  by the mean bead state.

### Removed

- Tests that pinned measured values (the Sun's, water's, Titan's and the
  Earth's) or the model's own output; tests now guard laws: books that
  close, exact identities, orderings and monotonicities.
- The Planck draw centres and the Big Bang `Kind`s; the Chiappini infall
  fit (`galaxy::infall_fit`); the stellar rulers `10 Gyr x M / L`, `L = M`,
  the faint-young-Sun slope and the 20-sun hole cut; the main-sequence
  brightening in `worlds`; the chemistry's reference history
  (the chemistry's reference parameters).
- The disc partition `formation::disc`, replaced by
  `disc::manifold_worlds`.
- The trade-route optimiser in `transit` (`Offer`, `Market`, `Leg`, `Route`,
  `circuits`, `departures`, `spreads`, `supplies`) and the toll law's
  `around`; the fuel-choice machinery in `propulsion`; `Knowledge::learn`,
  `transit::binding`, `Cosmic::from_core`, `Chronometry::rim_heads`,
  `Seats::sovereigns` and the measure's radial shares, all uncalled.
- The 15 kpc disc edge as the bell's length, the Hubble length and the
  closure draw, the 0.3 Gyr first stars, the core's drawn spin
  (`hole_spin`), and the `XYL_FOUNDING_BITS` environment input.
- Earlier civilization machinery: human life stages and disease
  (`lifespan`), demographic transition and pronatal policy (`fertility`),
  four seasons and food reserves (`sustenance`), the policy fork
  (`demography`), central and federal banks, markets and backed currencies
  (`banks`, `exchange`, `backing`, `decide`), sedimentation and mining depth
  (`archaeology`), and captains' offices.
- Measured human data in knowledge; the r and K fertility ceilings (a
  people's natural rate is one e-fold per its own generation).
- Carbon as every chemistry's backbone (monomers are built from the
  solvent's own molecules) and fixed extinction rates (strikes come from
  each system's own debris).
- `primordial.rs` and `crust.rs`.
- `seats::council_size` and `seats::On::Ship`.

## 0.2.0 - 2026-10-28

Initial release.

### Added

- Deterministic universe generation from a seed: a bounce cosmology, element
  synthesis by successive capture, the founding cascade into a held
  firmament and galaxy-forming beads.
- The substrate: the core, the inter-galactic medium and each galaxy
  expressed as closed-form functions of time, with outflow, intake, force
  (drift and swing) and tide.
- Galaxies distributed over the universe's sphere and circulating about the
  throat; stars on double-cone manifolds; places as bearings and distances as
  arcs.
- Planetary systems formed from the matter available, nested manifolds for
  planets and moons, and world state read per degree of the star's life.
- Life and the forecast of minds; peoples woken where their world feeds a
  viable population.
- One civilization carry for live play and readings: knowledge, the fall or
  exodus of a people, food, fertility, lifespan and gauges, with housing from
  natural production and energy-driven synthesis.
- Propulsion, constellations, ley lines sited by discovery and bounded
  optimisation with passage energy derived from the builder's drive, and
  quarantine by discord.
- Seats for external agents, applied as input events.
- Economy libraries (banks, markets, backed currencies, the decision rule),
  not yet integrated into the carry.
- `xyl_core::cga`, a conformal geometric algebra over exact rationals.
- `xylarium` binary with `genesis`, `run` and `observe`, and the
  `xylarium::run::Link` hook for network uplinks.
- Documentation: README, architecture, model, civilizations, network
  protocol, glossary, development guide and roadmap.
- Continuous integration: formatting, clippy with warnings denied, tests,
  documentation with warnings denied, and line coverage with
  `cargo llvm-cov`.
