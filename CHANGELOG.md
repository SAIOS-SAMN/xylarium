# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html). Until 1.0.0, minor versions
may change public APIs and the model's laws.

## Unreleased

### Added

- The shared-laws ledger (docs/EQUATIONS.md section 8): the laws proposed
  for the core shell of mycelium's `forms`, with proposed ids (#33).

- The jetstream for every holder (`circuit::jet_map`,
  `circuit::throat_crossing`): out through the continuum, back through the
  vacuum; the firmament's onsets take the vacuum leg (the founding
  changes: onsets come earlier) and a star's collective crosses the throat
  by the same law. `stars::timeline` runs fusion steps until the crossing
  and fission steps from it; `stars::remnant` is what the steps that ran
  did not transduce.
- One law per formula: `bounce::torsion_share(mass, spin)`,
  `circuit::dilation`, `circuit::echoes_to_throat` (the duplicates in
  `origin` and `stars` are removed).
- Stellar transduction by polarity (`stars::Sequence`): a star's held
  collective's pole point `t0 = sum n u / sum n w`, dilated per echo by
  `(1 + delta) / (1 - delta)`, selects fusion before the throat and fission
  after it; each stage steps the held kind of greatest release, its
  products join the mixture, and no kind, product or order is set. Field
  `Transduction` states: unlit, step, remnant. Star classes name the held
  body that does not ignite and the fission star.
- The word "burn" is replaced by "transduction" throughout
  (`stars::transduced_per_bell`, `field::Transduction`,
  `field::transduced_share`).
- `Q::ratio` is the constant-ratio constructor; it returns zero on a zero
  denominator.
- `AGENTS.md` and `docs/concepts.md` are added, and the glossary is
  rewritten as a lexicon with code paths.
- The examples `found_and_observe` and `minimal_link`
  (`crates/xylarium/examples`) are added, with `# Examples` sections on
  `run::genesis`, `run::run`, `run::run_with` and `observe::observe`.
- `scripts/check-docs` and `scripts/check-style` are added and are run
  locally with the other checks (CONTRIBUTING.md, Verification).

### Changed

- Roles: MAINTAINERS.md names the coordinator (mycelium's maintainer
  session) for contract review; CONTRIBUTING's workflow sends
  `cross-project` changes to it before merge.
- A home without a checkpoint is read at the first mind's dawn, where
  genesis writes its first checkpoint, in place of 14 Gyr (#3).
- The minimum supported Rust version is 1.85, the version the locked
  dependencies require.
- Code style: Modules over about 800 lines are split into directory modules
  (`observe`, `transit`, `cosmic`, `formation`, `galaxy`, `chem`, `worlds`,
  `systems`, `life`, `bodies`, `minds`) with public paths unchanged. Long
  functions are split into named steps, and lines are at most 100 columns.
  Private one-letter numeric wrappers are replaced by the shared
  constructors, and inline comments state reasons and formulas.
- `run::Options` derives `Clone` and `Debug`.
- The README is restructured. The roadmap opens with the mathematics:
  number systems, algebra and geometry, and the nucleon model (the 3Z span,
  the 3Q capacity and the polarity R).
- The Markdown documentation is edited to complete sentences in technical
  prose.
- The turning law is evaluated in core units, `omega = delta bell / r`
  radians per ring (`turning::turning`, `turning::turning_dy`), in place of
  `turning_kpc` and `turning_kpc_dy`; the SI forms remain as boundaries for
  the system and body layers.
- The galaxy's structure, star reaches and orbits are in galaxy rings and
  core masses (`galaxy::core_reach`, `galaxy::nucleus_reach`,
  `DiscFrame::reach`); kpc and suns appear at display and at the marked
  boundaries `galaxy::kpc_of_rings` and `galaxy::suns_of_core_masses`.
- The vertical orbit is derived from the turning law, the crowding and the
  ring structure (`galaxy::layer`, `galaxy::vertical_frequency`), in place
  of the declared 75.2 rad/Gyr, 20 km/s, 10 Gyr and 0.3 kpc.
- Star masses are in core masses: a piece's star mass is the share of its
  infall the star tier's cut holds (`galaxy::piece_star_mass`), and the
  initial mass function (`galaxy::imf`) draws shares of it by the deficit
  form with the ring's crowding as break, in place of the bounds 0.08 and 60
  suns and the break scale. `Star::mass`, `Star::companion`,
  `Star::stands_for` and the mass argument of `galaxy::disc_frame` are in
  core masses.
- A bead is a galaxy where its lit holding is at least one star's mass on
  the chain (`galaxy::first_star_mass`) and its core is no heavier than the
  universal core (`Substrate::holds_a_star`, `Substrate::galaxies`), in
  place of one sun. This refounds the test genesis once.
- Star light is the power the burn transduces, in core masses per ring
  (`stars::power`, `stars::light`); `systems::luminosity` is removed, and
  the system layer reads suns through the boundary `stars::suns_of_power`.
- Stars burn by polarity (`stars::Sequence`, `stars/sequence.rs`): each
  cloud's dark cut records its held mixture (`disc::StarHold::held`), and the
  star's sequence of fusion and fission steps emerges from that mixture at
  the cut's heat, each stage the held kind of greatest release, with the
  throat after `n* = ln(1 / t0) / ln rho_s` echoes (`stars::throat_echoes`).
  `stars::life`, `stars::ends`, `stars::burned` and `stars::remnant_share`
  take the star's sequence (`galaxy::Star::sequence`,
  `galaxy::piece_sequence`, `galaxy::first_star_sequence`);
  `field::Burning` is `Unlit`, `Step(i)` or `Remnant`; `field::burning` and
  `field::burn_out_share` read the sequence, and `field::absorb` takes the
  step's burn-out share. `galaxy::Class::name` names held bodies that do not
  ignite, fission stars and giants from the sequence. `life::lifetime`,
  `life::first_age` and `worlds::star_degree_gyr` take the star's sequence,
  carried in `life::Home::sequence` and `worlds::Setting::sequence`. A held
  body that does not ignite forms no remnant node (`pulse::nodes_among`).

### Removed

- The fixed hydrogen-helium-carbon chain: `stars::transduced_hydrogen`,
  `stars::transduced_helium` and the measured binding energies of
  `field::burn_out_share` (`binding_mev`, `nucleon_mev`, `weak_mev`).
- Hosted CI (`.github/workflows/ci.yml`) and its README badge are removed;
  the checks are run locally.

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
  from `formation.rs`. It reads the founding and is not part of the
  founding digest.
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
- Exoduses, settlements and steward decisions are recorded inputs, applied again
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
  stars, and no substitute is used.
- A star's cut of its piece's infall is read on the chain
  (`galaxy::disc_frame`, `disc::star_cut`): the disc's agitation is the
  pulse's heat on the disc, and the core-star tie is the galaxy's tide at
  the pinch. The Sun's disc mass, the 360 parts in `ln a`, `Spec::limits`
  and the stirred inner belt are gone.
- The jetstream reading: `Substrate::loop_flux` composes the filament's
  dilation, the vacuum's share and the electrons' phase state as dilators on
  each firmament kind's pole state (`circuit::PoleFlux`) and reads the
  residual.
- The parent's spin is admitted only where its torsion share holds the
  firmament whole against the pulse (`Substrate::least_share`,
  `bang::spin_of_share`); the genesis record states the margin.
- The Big Bang's only random draw is the parent's spin. The lumpiness, the held
  share, the assembly and the star efficiency are read off the cascade; the
  chemistry is calibrated on the universe's own history
  (`chem::Calibration::of`); the Big Bang `Kind`s and the `--kind` option are
  gone; the home defaults to `var/universe-7`.
- Stars are fuelled by the pulse (`stars`): the burn per bell is the ring
  the bell brings at the star's reach times its own deficit at its rim; its
  light is that burn; its life and its giant's end are its hydrogen and
  helium steps' holds over the burn (`stars::life`, `stars::ends`); every
  ended star's remnant is a bell of the network (`stars::remnant_share`).
- Radiation and temperature are distinct readings: a world's flux is the
  bell's mean of its star's burn, the swing within a bell is the light's own
  reading (`stars::flicker`), and temperatures are the media's agitation.
- The forecast of minds uses expected values in closed form: rungs take
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
- Civilization layer consolidated in `civ`: one housing law
  (`civ::housing`), numbers as the logistic toward housing scaled by the
  radiation environment, history (`civ::history`), settlers when crowded
  past the logistic's inflection.
- Knowledge is solved in closed form (`knowledge::advance`), replacing
  backward-Euler steps.
- The unit body's generation is a named ruler,
  `growth::Rulers::generation_yr` (14.2 yr).
- Turnings carry one lowercase act set: `exodus`, `settlement`, `network`,
  `quarantine`, `biology`, `information`, `fold`.
- Whether space can be folded is a law of the universe: `Scales::ftl` carries
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
  brightening in `worlds`; the chemistry's reference history and its
  parameters.
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
- Life and the forecast of minds; peoples created where their world feeds a
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
