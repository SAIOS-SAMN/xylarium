# Stand-ins: the register of assumptions

A stand-in is a value, an outcome or a branch the code takes where the law
that decides it is not built: an imposed outcome, a draw, a borrowed
constant, a fallback, or another body's or tier's quantity. Each one is
listed here with an ID. The code carries the ID where it uses the stand-in,
as a comment `Stand-in S###` at the site, so a reader and a new consumer see
it at the call. `scripts/check-stand-ins`, run by `scripts/check-docs`,
keeps this register and the markers in agreement in both directions.

The register is the history of the remediation. A row stays here when its
law lands, marked remediated with the PR that closed it, and the marker
leaves the code with that PR.

## Why this register exists

Audit of 2026-11-11 ([#189](https://github.com/SAIOS-SAMN/xylarium/issues/189)).
- **The finding:** about 115 stand-ins were read as law. Nearly all came
  with the first commit, `e11a9ec` (xylarium 0.3.0), the import of the
  prototype calibrated to the Earth, its star and its galaxy, already wired
  into consumers.
- **The status lived only in prose,** in three documents that disagreed
  (EQUATIONS.md, model.md and the roadmap's declared rulers). The code
  carried no mark.
- **Every later pass kept the values** and changed their syntax (`a7fdafd`,
  `6bd8958`, `7700802`, `c5d9e4c`), so a stand-in's history read as
  reviewed.
- **New readers repeated the pattern:** #187 read the star's compactness for
  its remnant (#188).

The rule (AGENTS.md):
- A law that is not built is refused or registered here.
- A reader never takes the nearest quantity of another body, tier or event.
- A new consumer of a stand-in cites its ID.
- A refactor keeps the marker.

## Classes

| Class | Meaning | Remedy at the root |
|---|---|---|
| A | An imposed outcome: a fate or category no law decides | [#190](https://github.com/SAIOS-SAMN/xylarium/issues/190), per site |
| B | A draw standing for a law | [#191](https://github.com/SAIOS-SAMN/xylarium/issues/191) |
| C | A borrowed constant or threshold (the prototype's Earth, Sun and Jupiter calibrations) | [#25](https://github.com/SAIOS-SAMN/xylarium/issues/25), [#13](https://github.com/SAIOS-SAMN/xylarium/issues/13), [#14](https://github.com/SAIOS-SAMN/xylarium/issues/14), [#18](https://github.com/SAIOS-SAMN/xylarium/issues/18) |
| D | A fabricated fallback: a missing value read as a law value | [#179](https://github.com/SAIOS-SAMN/xylarium/issues/179) (rule 11, tiers C-F) |
| E | The nearest quantity of another body, tier or event | [#190](https://github.com/SAIOS-SAMN/xylarium/issues/190) |
| F | The doc's status (placeholder, next, planned) not carried by the code | [#189](https://github.com/SAIOS-SAMN/xylarium/issues/189), this register |
| G | Borrowed jargon whose outside meaning carries an assumption | [#135](https://github.com/SAIOS-SAMN/xylarium/issues/135) |
| H | A boolean or an endpoint blend where a share is meant | [#192](https://github.com/SAIOS-SAMN/xylarium/issues/192) |
| U | A ruler inline and undeclared (class C rows read by laws) | [#193](https://github.com/SAIOS-SAMN/xylarium/issues/193) |

## The register

Origin `0.3.0` is `e11a9ec`, the first commit. Paths are relative to the
repository root.

| ID | Class | Site | Assumed | Law it stands for | Consumers | Origin | Issue | State |
|---|---|---|---|---|---|---|---|---|
| S001 | A,E | `crates/xyl-cosmos/src/pulse.rs` nodes_among | Every igniting sample star becomes a black-hole bell at the end of its first step | A remnant is a hole only where its compactness reaches 1/2, formed when every step ends (model.md star types, Next) | field::Bell, xyl-kernel field, chronometry | 0.3.0 | [#188](https://github.com/SAIOS-SAMN/xylarium/issues/188) | open |
| S002 | A,E | `crates/xyl-cosmos/src/pulse.rs` nodes_among | A sample star's whole represented population becomes one hole of mass stands_for x remnant at its place | A body's hold, not a population's, makes one hole | Bell pitch, light and strain; pulse::hear | 0.3.0 | [#188](https://github.com/SAIOS-SAMN/xylarium/issues/188) | open |
| S003 | E | `crates/xyl-cosmos/src/field.rs` stellar_tilt | The star's compactness stands for its remnant's held face | The remnant's face (the throat's equator 1/2 or the horizon 1/3) | Bell::axis | 7313e1b | [#188](https://github.com/SAIOS-SAMN/xylarium/issues/188) | open |
| S004 | D | `crates/xyl-cosmos/src/field.rs` stellar_axis | An unread tilt stands upright | Unread is refused, not upright | Bell::axis, received | 7313e1b | [#179](https://github.com/SAIOS-SAMN/xylarium/issues/179) | open |
| S005 | G | `crates/xyl-cosmos/src/pulse.rs` Node::place | "Progenitor star" (astrophysics) for the star a remnant collapsed from; collides with seats::Role::Progenitor | "The star it collapsed from" | field docs | 0.3.0 | [#188](https://github.com/SAIOS-SAMN/xylarium/issues/188) | open |
| S006 | B,C | `crates/xyl-cosmos/src/galaxy/star.rs` star | Binary when u_bin < 1/3; companion (0.1 + 0.9 u) of the mass | From the cloud's split (model.md star types, Placeholder) | Star::companion | 0.3.0 | [#191](https://github.com/SAIOS-SAMN/xylarium/issues/191) | open |
| S007 | B | `crates/xyl-cosmos/src/galaxy/star.rs` imf | A star's mass is a draw u_mass over the piece's top mass | Each star from its cloud's cut (roadmap: the mass draw to be deleted) | compactness, life, stands_for, systems, field | 0.3.0 | [#191](https://github.com/SAIOS-SAMN/xylarium/issues/191) | open |
| S008 | B | `crates/xyl-cosmos/src/galaxy/star.rs` placement | Radius, phase, tilt and node from irrational turns of the sample index (phyllotaxis) | Placement by the repulsion law (roadmap item 40, to be replaced) | Orbit, Star::place, pulse, field | 0.3.0 | [#36](https://github.com/SAIOS-SAMN/xylarium/issues/36) | open |
| S009 | B | `crates/xyl-cosmos/src/galaxy/vertical.rs` vertical | Vertical speed drawn through an exponential, vz = -sigma ln(1 - u), and a drawn phase | From the star's formation | Star::exposure, life | 0.3.0 | [#191](https://github.com/SAIOS-SAMN/xylarium/issues/191) | open |
| S010 | A,E | `crates/xyl-cosmos/src/galaxy/star.rs` star | Every other galaxy is the home galaxy's homologue: home stars at scaled lengths and masses | Each galaxy read from the bead it formed from (roadmap items 6, 33) | pulse nodes, bells, systems | 0.3.0 | [#36](https://github.com/SAIOS-SAMN/xylarium/issues/36) | open |
| S011 | C,F | `crates/xyl-cosmos/src/cores.rs` Shape::scale | Galaxy length scale 2m / (1 + m) with M_a the home mass | Placeholder (EQUATIONS galaxy scale); M_a undeclared | every non-home star's orbit and clock | 0.3.0 | [#7](https://github.com/SAIOS-SAMN/xylarium/issues/7) | open |
| S012 | A,E | `crates/xyl-cosmos/src/cores.rs` founding_of | Every galaxy is born at the home galaxy's first-star time | Each bead lights by its lighting (Lighting::cools_in_at) | pulse core nodes, relays, bells | 0.3.0 | [#190](https://github.com/SAIOS-SAMN/xylarium/issues/190) | open |
| S013 | B | `crates/xyl-cosmos/src/cores.rs` founding_of | A galaxy's plane from index x plastic and golden turns | Placement by the repulsion law (roadmap item 40) | Shape planes, place_at, pulse, field, transit | 0.3.0 | [#36](https://github.com/SAIOS-SAMN/xylarium/issues/36) | open |
| S014 | D | `crates/xyl-cosmos/src/circuit/anchor.rs` of | First stars at ring 1 where the home gas never lights | Refused where the gas never lights | every galaxy's born | 0.3.0 | [#179](https://github.com/SAIOS-SAMN/xylarium/issues/179) | open |
| S015 | D,A | `crates/xyl-cosmos/src/cores.rs` Weights::factor | H and He weights fixed at 1; a metal missing from a bead's mix reads 0 | From the bead's own mix | each star's composition | 0.3.0 | [#190](https://github.com/SAIOS-SAMN/xylarium/issues/190) | open |
| S016 | C,G | `crates/xyl-cosmos/src/field.rs` hole_light_per_star_mass | Hole light = mass x 32 000 x horizon share x efficiency (an Eddington-fed flow) | Not derived; undeclared | Bell::light, energy_at, kernel books | 0.3.0 | [#193](https://github.com/SAIOS-SAMN/xylarium/issues/193) | open |
| S017 | C | `crates/xyl-cosmos/src/centre.rs` horizon_share, efficiency | horizon_share 1/10 and efficiency 1/10, rulers named in code only | Undeclared | hole light, Core::capture | 0.3.0 | [#193](https://github.com/SAIOS-SAMN/xylarium/issues/193) | open |
| S018 | C | `crates/xyl-cosmos/src/field.rs` star_year | The field's energy unit is the Sun's luminosity times a Julian year (2.28002) | SI only at the display seam | energy_at, kernel books | 0.3.0 | [#25](https://github.com/SAIOS-SAMN/xylarium/issues/25) | open |
| S019 | C | `crates/xyl-cosmos/src/field.rs` star_ring | A ring's energy reads Si::seconds_per_ring inside the law | SI only at the display seam | energy_at, kernel field | 0.3.0 | [#25](https://github.com/SAIOS-SAMN/xylarium/issues/25) | open |
| S020 | B | `crates/xyl-cosmos/src/field.rs` drawn_spin | Each stellar hole's spin is a fresh draw | Only the parent's spin is drawn; T11 a_g = a_p M_p / H_g | Bell::spin, received, kernel absorb | 0.3.0 | [#191](https://github.com/SAIOS-SAMN/xylarium/issues/191) | open |
| S021 | E | `crates/xyl-cosmos/src/field.rs` Bell::of | Every galactic core's bell spins about z at the parent's drawn spin | The core's plane and spin (T11, core_spin) | kernel field | 0.3.0 | [#190](https://github.com/SAIOS-SAMN/xylarium/issues/190) | open |
| S022 | A | `crates/xyl-cosmos/src/field.rs` pitch_kpc | The hole's pitch scales as extent(M / M_core) in place of a cube root | Not derived | Bell::period, received, energy_at | 0.3.0 | [#190](https://github.com/SAIOS-SAMN/xylarium/issues/190) | open |
| S023 | D | `crates/xyl-cosmos/src/field.rs` Bell::energy_at | Distance floored at 1 AU (energy) and 1e-9 kpc (strain); energy falls as a^2 / 4 d^2 while hear falls as M / d | One distance law, no floor | kernel energy and shear | 0.3.0 | [#179](https://github.com/SAIOS-SAMN/xylarium/issues/179) | open |
| S024 | F | `crates/xyl-cosmos/src/field.rs` Anchor::on_orbit | Every world anchor sits on the first axis [a, 0, 0] | A listed defect (model.md, the rotor, the place and the chain) | kernel absorb shear | 0.3.0 | [#90](https://github.com/SAIOS-SAMN/xylarium/issues/90) | open |
| S025 | A,F | `crates/xyl-cosmos/src/pulse.rs` hear | Intensity I = M / d in place of M / d^2; every bell summed as one flat phasor | A listed defect, to be replaced | xyl-kernel percept, run | 0.3.0 | [#20](https://github.com/SAIOS-SAMN/xylarium/issues/20) | open |
| S026 | F | `crates/xyl-cosmos/src/pulse.rs` relays | Relay continuity cos(2 pi detour / lambda), every hole relaying in phase | Next: under review (EQUATIONS) | hear, run | 0.3.0 | [#20](https://github.com/SAIOS-SAMN/xylarium/issues/20) | open |
| S027 | E,F | `crates/xyl-cosmos/src/stars/mod.rs` ring_at | Every star's received ring is read about the universal core's hold at its reach about its galactic core | Until the galaxy tier's medium is carried (roadmap item 44) | life, ends, power, light, class, nodes, systems | c965fbb | [#113](https://github.com/SAIOS-SAMN/xylarium/issues/113) | open |
| S028 | E | `crates/xyl-cosmos/src/galaxy/disc_manifold.rs` first_star_compactness | The founding's reference star supplies compactness and sequence for readings naming no star | Each reading names its body | worlds unit, field | ebe75ad | [#190](https://github.com/SAIOS-SAMN/xylarium/issues/190) | open |
| S029 | C,E | `crates/xyl-cosmos/src/circuit/weights.rs` mass_number | Nucleon count A = round(Earth's standard atomic weight) | Each nucleus from the founding; the tsv ruler covers worlds only | neutrons_of, mix_weights, stellar_tilt, fusion products | e0109a6 | [#17](https://github.com/SAIOS-SAMN/xylarium/issues/17) | open |
| S030 | C | `crates/xyl-cosmos/src/elements.rs` build_table | Rows without a mass take A from the valley-of-stability fit | Not derived, undeclared | mass_number above Z 99 | 0.3.0 | [#9](https://github.com/SAIOS-SAMN/xylarium/issues/9) | open |
| S031 | D | `crates/xyl-cosmos/src/galaxy/star.rs` Star::exposure | Exposure 1 for bulge and nucleus stars | Not derived | life living | 0.3.0 | [#190](https://github.com/SAIOS-SAMN/xylarium/issues/190) | open |
| S032 | G | `crates/xyl-cosmos/src/galaxy/vertical.rs` vertical; `crates/xyl-cosmos/src/galaxy/star.rs` | "The crowded layer is where comet clouds are perturbed and supernovae occur"; brown and red dwarf names | No supernova or comet perturbation exists; types are not set | life's reading of exposure | 0.3.0 | [#135](https://github.com/SAIOS-SAMN/xylarium/issues/135) | open |
| S033 | D | `crates/xyl-cosmos/src/galaxy/vertical.rs` heating_share | Cooling time falls back to the star's birth; lambda0 and holdings floored at 1e-9 | Refused where unread | vertical, density | 0.3.0 | [#179](https://github.com/SAIOS-SAMN/xylarium/issues/179) | open |
| S034 | F | `crates/xyl-core/src/rotor.rs` angle_between | One unnamed plane's angle as the distance under every arc | Planned: replaced by the pair reading (EQUATIONS, #118) | chain::Place::arc, pulse relays, field delay, kernel transit | 42cfb27 | [#118](https://github.com/SAIOS-SAMN/xylarium/issues/118) | open |
| S035 | F | `crates/xyl-cosmos/src/elements.rs` closed_share | The closed share W / 3Z built in place of the capacity face | Planned 2P / cap (#158, #172) | founded, the whole firmament | 0.3.0 | [#158](https://github.com/SAIOS-SAMN/xylarium/issues/158) | open |
| S036 | F | `crates/xyl-cosmos/src/circuit/core.rs` Substrate::held_within; `crates/xyl-cosmos/src/galaxy/mod.rs` | Two interim models of the hold inside a reach (held_within and galaxy::inside_mass) | One model (#116) | cores, disc_manifold, galaxy star, vertical, chronometry | 0.3.0 | [#116](https://github.com/SAIOS-SAMN/xylarium/issues/116) | open |
| S037 | F | `crates/xyl-cosmos/src/galaxy/star.rs` star (Orbit); `crates/xyl-cosmos/src/chain.rs` | galaxy::Orbit circles and chain::Place points as the carriers | Planned: the vortex (#121) | every crate | 0.3.0 | [#121](https://github.com/SAIOS-SAMN/xylarium/issues/121) | open |
| S038 | B | `crates/xyl-cosmos/src/bodies/orbit.rs` orbits | A world's birth spin is (0.1 + 0.4 u) of break-up | Not derived | lock, day, life's lock | 0.3.0 | [#191](https://github.com/SAIOS-SAMN/xylarium/issues/191) | open |
| S039 | D | `crates/xyl-cosmos/src/bodies/orbit.rs` orbits | No world_tilt (no elements) reads cos(obliquity) = 1, upright | Unread is refused | polar_share, Body.tilt, seasonal | 7313e1b | [#179](https://github.com/SAIOS-SAMN/xylarium/issues/179) | open |
| S040 | D | `crates/xyl-cosmos/src/bodies/orbit.rs` orbits | A world with no neighbour gets an eccentricity cap of 9/10 | e0 = swing (EQUATIONS section 6) | tides, manifold | 0.3.0 | [#193](https://github.com/SAIOS-SAMN/xylarium/issues/193) | open |
| S041 | C | `crates/xyl-cosmos/src/bodies/mod.rs` tidal_q | Tidal Q = 100 and I / MR^2 = 0.33 (Earth's) | Lab constants only are declared | lock_gyr, circularise_gyr | 0.3.0 | [#25](https://github.com/SAIOS-SAMN/xylarium/issues/25) | open |
| S042 | C | `crates/xyl-cosmos/src/systems/mod.rs` World::icy | A world is icy where ice > 1/5 | Not derived, undeclared | rigidity, lock, tides | 0.3.0 | [#193](https://github.com/SAIOS-SAMN/xylarium/issues/193) | open |
| S043 | A | `crates/xyl-cosmos/src/worlds/system.rs` settings | Every moon is locked from birth, upright, with tau_e = 0 | lock_gyr about its giant | moon heartbeat, season, polar share | 0.3.0 | [#190](https://github.com/SAIOS-SAMN/xylarium/issues/190) | open |
| S044 | B | `crates/xyl-cosmos/src/manifold.rs` manifolds | A world's mean anomaly at the star's birth is a uniform draw | The settled phase exists | stand, bearing, gap, kernel transit | 0.3.0 | [#191](https://github.com/SAIOS-SAMN/xylarium/issues/191) | open |
| S045 | A | `crates/xyl-cosmos/src/manifold.rs` manifolds | Every moon sits on a captured 2:1 chain, rate = innermost / 2^k | No resonance-capture law is built | moon positions, gaps, voyages | 0.3.0 | [#39](https://github.com/SAIOS-SAMN/xylarium/issues/39) | open |
| S046 | D | `crates/xyl-cosmos/src/manifold.rs` manifolds | A moon's plane is the identity rotor | Not derived | moon bearings | 0.3.0 | [#90](https://github.com/SAIOS-SAMN/xylarium/issues/90) | open |
| S047 | E | `crates/xyl-cosmos/src/manifold.rs` manifolds | A refused plane skips the planet counter, so later planets read their predecessor's orbit (latent) | Each world on its orbit | m0, bearings | 0.3.0 | [#190](https://github.com/SAIOS-SAMN/xylarium/issues/190) | open |
| S048 | C | `crates/xyl-cosmos/src/worlds/setting.rs` star_t | Star surface temperature 5772 L / R (the Sun's), radius floor 0.08 | Not derived | photolysis, media light, life energy | 0.3.0 | [#25](https://github.com/SAIOS-SAMN/xylarium/issues/25) | open |
| S049 | D | `crates/xyl-cosmos/src/worlds/setting.rs` build | No path reads cos(obliquity) = 0.7071 (45 degrees), inside the unit world's 288 K calibration | Unread is refused | polar_share, climate | 0.3.0 | [#179](https://github.com/SAIOS-SAMN/xylarium/issues/179) | open |
| S050 | C | `crates/xyl-cosmos/src/worlds/setting.rs` build; `crates/xyl-cosmos/src/worlds/mod.rs` | Exosphere 4 x T_eq, x 3/2 without a field | Not derived | jeans_held, the atmosphere | 0.3.0 | [#14](https://github.com/SAIOS-SAMN/xylarium/issues/14) | open |
| S051 | C | `crates/xyl-cosmos/src/worlds/climate.rs` Climate::model_mixed | Albedo 0.3 (Earth's), +0.4 at full ice; first guess at 0.3 | World heat, placeholder (EQUATIONS) | every climate solve | 0.3.0 | [#14](https://github.com/SAIOS-SAMN/xylarium/issues/14) | open |
| S052 | A | `crates/xyl-cosmos/src/worlds/setting.rs` build | A giant's state is a blank at 1000 bar and albedo 0.3 | No giant atmosphere law | giant media | 0.3.0 | [#14](https://github.com/SAIOS-SAMN/xylarium/issues/14) | open |
| S053 | C | `crates/xyl-cosmos/src/worlds/setting.rs` build | Surface light attenuated by exp(-p / 10 bar) | Not derived | media energy, ladders | 0.3.0 | [#14](https://github.com/SAIOS-SAMN/xylarium/issues/14) | open |
| S054 | C | `crates/xyl-cosmos/src/worlds/setting.rs` photolyse; `crates/xyl-cosmos/src/worlds/mod.rs` | A fixed photolysis list calibrated on the Sun and Titan | Not derived | atmosphere composition | 0.3.0 | [#15](https://github.com/SAIOS-SAMN/xylarium/issues/15) | open |
| S055 | F | `crates/xyl-cosmos/src/worlds/mod.rs` constants | greenhouse_tau 31/5, tuned so the unit world reads 288 K | Placeholder: world heat replaces it (EQUATIONS) | every surface temperature, media, life | 0.3.0 | [#14](https://github.com/SAIOS-SAMN/xylarium/issues/14) | open |
| S056 | C | `crates/xyl-cosmos/src/worlds/mod.rs` constants | Runaway limit 282 / 340 (Earth) | Placeholder (EQUATIONS world heat) | climate solve | 0.3.0 | [#14](https://github.com/SAIOS-SAMN/xylarium/issues/14) | open |
| S057 | C | `crates/xyl-cosmos/src/worlds/star.rs` activity | Stellar activity 1 + 9 exp(-k age), floor 1/20, a solar fit | Not derived | escape, wind dose | 0.3.0 | [#25](https://github.com/SAIOS-SAMN/xylarium/issues/25) | open |
| S058 | C | `crates/xyl-cosmos/src/hydro.rs` radiolysis | Earth's magnetosphere and air: standoff 10.1 R, cutoffs 6, 1.7 and 4 GV, columns 1033 and 185 g/cm2, fluxes, 3 % radiolysis | Only the unit world is a ruler | media energy, ladder | 0.3.0 | [#25](https://github.com/SAIOS-SAMN/xylarium/issues/25) | open |
| S059 | C,G | `crates/xyl-cosmos/src/worlds/media.rs` region | Regions by Earth-biochemistry windows: above 450 K mineral, below 230 K cryogenic, CH4/N2/NH3 cryogenic | Life from any elements; not derived | best, chemistries, minds' region | 0.3.0 | [#18](https://github.com/SAIOS-SAMN/xylarium/issues/18) | open |
| S060 | C | `crates/xyl-cosmos/src/worlds/media.rs` solvents | Giant plasma from 10 world masses; energy scaled on Jupiter (5.4 / 340, 318, 11); 1e4 K; melt mid 2000 K | Not derived | plasmic media, foragers | 0.3.0 | [#18](https://github.com/SAIOS-SAMN/xylarium/issues/18) | open |
| S061 | C | `crates/xyl-cosmos/src/worlds/media.rs` solvents | A silicate melt solvent above 1400 K | Not derived | mineral media | 0.3.0 | [#18](https://github.com/SAIOS-SAMN/xylarium/issues/18) | open |
| S062 | C | `crates/xyl-cosmos/src/worlds/media.rs` solvents | A subsurface ocean sits at its triple point + 5 K | Not derived | fidelity, pace | 0.3.0 | [#18](https://github.com/SAIOS-SAMN/xylarium/issues/18) | open |
| S063 | D | `crates/xyl-cosmos/src/worlds/media.rs` solvents; `crates/xyl-cosmos/src/worlds/mod.rs` | No interior flux reads Earth's 0.087 W/m2 | Unread is refused | subsurface oceans | 0.3.0 | [#179](https://github.com/SAIOS-SAMN/xylarium/issues/179) | open |
| S064 | C | `crates/xyl-cosmos/src/worlds/media.rs` richness | Richness counts C, N, O and metal against 4 (a CHON chemistry) | Life from any elements | best, ladder | 0.3.0 | [#18](https://github.com/SAIOS-SAMN/xylarium/issues/18) | open |
| S065 | C | `crates/xyl-cosmos/src/worlds/media.rs` plasma_field_ev | Plasma field energy = Bohr magneton x field x 1e-4 T | Not derived | plasmic ladder | 0.3.0 | [#18](https://github.com/SAIOS-SAMN/xylarium/issues/18) | open |
| S066 | C | `crates/xyl-cosmos/src/systems/mod.rs` settled_gyr | Systems read at a fixed settled age of 3 Gyr | Each body at its epoch | merges, exposure, Body::of | 0.3.0 | [#190](https://github.com/SAIOS-SAMN/xylarium/issues/190) | open |
| S067 | C | `crates/xyl-cosmos/src/systems/mod.rs` planet | Comet delivery 1e-4 x env x z with fixed Solar-System ratios | Not derived | ices, atmospheres | 0.3.0 | [#15](https://github.com/SAIOS-SAMN/xylarium/issues/15) | open |
| S068 | C | `crates/xyl-cosmos/src/systems/mod.rs` planet | Envelope from 10 world masses, x 5.6, period / 25; giant radius 11(1 - e^(-M/30)); ice radius 0.4 | Placeholder: a declared gathering law (EQUATIONS), undeclared | giant, moons, media, life | 0.3.0 | [#193](https://github.com/SAIOS-SAMN/xylarium/issues/193) | open |
| S069 | C,F | `crates/xyl-cosmos/src/systems/moons.rs` belts | Comet cloud 5 % of a tenth of the giants' mass, condensed at 30x the outermost reach | Declared share (partial), not in the list | debris, strikes, climb | 0.3.0 | [#193](https://github.com/SAIOS-SAMN/xylarium/issues/193) | open |
| S070 | C | `crates/xyl-cosmos/src/systems/gas.rs` carbon_as | Carbon split CO 1/2, CO2 1/5, CH4 3/10; nitrogen NH3 and N2 1/2 each | Not derived | ices | 0.3.0 | [#15](https://github.com/SAIOS-SAMN/xylarium/issues/15) | open |
| S071 | C | `crates/xyl-cosmos/src/chem/calibration.rs` Calibration::build; `crates/xyl-cosmos/src/chem/mod.rs` | The Sun born at 9.2 Gyr, today 13.8 Gyr, v_flat 220 km/s | Earth-centred; to be removed (phase 5) | isotopes kept, metals, every system | 0.3.0 | [#25](https://github.com/SAIOS-SAMN/xylarium/issues/25) | open |
| S072 | C,B | `crates/xyl-cosmos/src/chem/chemistry.rs` Chemistry::lump | Mergers 2e-5 per sun over 300 pc and 0.1 Gyr; a Poisson count drawn | Not derived, inline | r-process, heat sources | 0.3.0 | [#193](https://github.com/SAIOS-SAMN/xylarium/issues/193) | open |
| S073 | C | `crates/xyl-cosmos/src/life/body.rs` growing | Growing share over 9 latitude bands and 48 days, half the yearly mean | An Earth calendar | foragers, viable, minds | 0.3.0 | [#18](https://github.com/SAIOS-SAMN/xylarium/issues/18) | open |
| S074 | C,G | `crates/xyl-cosmos/src/life/body.rs` foragers | r-strategy where the star is below 1/2 sun (the red-dwarf flare convention) | From the world's variability | foragers | 0.3.0 | [#192](https://github.com/SAIOS-SAMN/xylarium/issues/192) | open |
| S075 | C,G | `crates/xyl-cosmos/src/life/body.rs` foragers | r/K fertility ceilings 2.5 and 1.2 set body mass | Undeclared | foragers | 0.3.0 | [#193](https://github.com/SAIOS-SAMN/xylarium/issues/193) | open |
| S076 | C,A | `crates/xyl-cosmos/src/life/body.rs` foragers | A locked world grows half the year; a giant's gravity is radius / 4; 50 000 foragers per unit area | Undeclared | foragers, minds | 0.3.0 | [#193](https://github.com/SAIOS-SAMN/xylarium/issues/193) | open |
| S077 | E,D | `crates/xyl-cosmos/src/life/body.rs` Body::of | Tilt and lock read once at the 3 Gyr settled age; no path reads unlocked and upright | The body at the mind's epoch | foragers in minds | 0.3.0 | [#190](https://github.com/SAIOS-SAMN/xylarium/issues/190) | open |
| S078 | C,A | `crates/xyl-cosmos/src/life/living.rs` chemistries | A lit medium's window closes at 0.9 of the star's life ("giant branch") | The star's later steps (built) | climb horizon, star_died | 0.3.0 | [#193](https://github.com/SAIOS-SAMN/xylarium/issues/193) | open |
| S079 | A | `crates/xyl-cosmos/src/life/living.rs` lineage; `crates/xyl-cosmos/src/minds/mod.rs` | The sterilising pulse falls at its expected time, 1 / hazard (a mean as an event) | An event found by xyl_core::event | minds, curve | 0.3.0 | [#41](https://github.com/SAIOS-SAMN/xylarium/issues/41) | open |
| S080 | C | `crates/xyl-cosmos/src/life/mod.rs` constants | i_mind 1e9, body_per_monomer 1e3, sterilise_at_r0 1/10, discrimination 1/2 ("open rulers" in code) | Undeclared | ladder, hazard, kernel knowledge | 0.3.0 | [#193](https://github.com/SAIOS-SAMN/xylarium/issues/193) | open |
| S081 | A | `crates/xyl-cosmos/src/life/ladder.rs` dormant_rate | A dormant plasma pattern is lost at 1 per second | Not derived | transfer survival | 0.3.0 | [#18](https://github.com/SAIOS-SAMN/xylarium/issues/18) | open |
| S082 | E,A | `crates/xyl-cosmos/src/minds/mod.rs` expect | The forecast climbs without arrivals and picks the earliest complete chemistry, unlike lineage | One model with the minds | Schedule.curve | 0.3.0 | [#43](https://github.com/SAIOS-SAMN/xylarium/issues/43) | open |
| S083 | A | `crates/xyl-kernel/src/transit/voyage.rs` best_spec | The best drive is always annihilation at c^2 per kg, so v_e = c | The people's chemistry and capture; antimatter is manufactured | building, drives, observe | 0.3.0 | [#192](https://github.com/SAIOS-SAMN/xylarium/issues/192) | open |
| S084 | A,E | `crates/xyl-kernel/src/transit/building.rs` builders | Ancient builders use best_spec, so passage energy m c (c - v_e) = 0: every lawful line is free | The builder's own exhaust | line cost, worth, breach | 0.3.0 | [#192](https://github.com/SAIOS-SAMN/xylarium/issues/192) | open |
| S085 | B,H | `crates/xyl-kernel/src/transit/voyage.rs` blended | The drive blends chemical to annihilation by energy mastery; kind flips at 1/2 | Not derived | discord reach, folds, exodus | 0.3.0 | [#192](https://github.com/SAIOS-SAMN/xylarium/issues/192) | open |
| S086 | E | `crates/xyl-kernel/src/transit/building.rs` builders | Builders' spare power is the world's starlight and heat less the bodies' use, with no capture share | growth.rs: a people commands only its bodies' power | builder gate, due time | 0.3.0 | [#192](https://github.com/SAIOS-SAMN/xylarium/issues/192) | open |
| S087 | C,H | `crates/xyl-kernel/src/cosmic/emergence.rs` Cosmic::arise | r or K as a boolean; r where the tilt is seasonal or star_mass < 1/2 sun | A share from the world's variability | size, sociality, growth, generation | 0.3.0 | [#192](https://github.com/SAIOS-SAMN/xylarium/issues/192) | open |
| S088 | B | `crates/xyl-kernel/src/cosmic/emergence.rs` select | Sociality is a draw u + 3/10 (r) + 1/5 (low energy) - 1/4 | Not derived | Shape, collective, thought, forgetting | 0.3.0 | [#191](https://github.com/SAIOS-SAMN/xylarium/issues/191) | open |
| S089 | A,H | `crates/xyl-kernel/src/cosmic/emergence.rs` select | Mind shape by cutoffs (superorganism above 3/4 social and r; distributed if plasma or dark and slow) | Not derived | collective weight, eminent naming | 0.3.0 | [#192](https://github.com/SAIOS-SAMN/xylarium/issues/192) | open |
| S090 | C | `crates/xyl-kernel/src/cosmic/emergence.rs` select | The leading sense by cutoffs, heat above 450 K | Declared trait thresholds (roadmap) | sense weights, lived thought | 0.3.0 | [#193](https://github.com/SAIOS-SAMN/xylarium/issues/193) | open |
| S091 | C | `crates/xyl-kernel/src/knowledge.rs` sense_weights | A per-sense domain weight table (a game table) | Declared in the header, not in the list | affinity, directed thought | 0.3.0 | [#193](https://github.com/SAIOS-SAMN/xylarium/issues/193) | open |
| S092 | C | `crates/xyl-kernel/src/knowledge.rs` knowledge_unit | U = 10 KU, lived share 1/20, curiosity 1/4, pooled keep 1/2 | Only the knowledge unit's thousand minds is listed | mastery, effort, forgetting | 0.3.0 | [#193](https://github.com/SAIOS-SAMN/xylarium/issues/193) | open |
| S093 | C,G | `crates/xyl-kernel/src/knowledge.rs` profile | Profile weights (3/2, 3/4, 1/2, 3/5, 2/5, 1/10, 3/10); aquatic as a flag ("without fire") | Undeclared | collective | 0.3.0 | [#193](https://github.com/SAIOS-SAMN/xylarium/issues/193) | open |
| S094 | E | `crates/xyl-kernel/src/knowledge.rs` Thinkers::of | Life expectancy set equal to one generation | Not derived | forgetting, stocks, figures | 0.3.0 | [#190](https://github.com/SAIOS-SAMN/xylarium/issues/190) | open |
| S095 | D,F | `crates/xyl-kernel/src/knowledge.rs` Needs::of | Births, crust, power and trade pulls 0; works 0; impedance 1; fluid 0 | Read zero until their layers exist (civilizations.md) | free thought, effort | 0.3.0 | [#179](https://github.com/SAIOS-SAMN/xylarium/issues/179) | open |
| S096 | C | `crates/xyl-kernel/src/growth.rs` food_w | A locked world's growing share is 1/2 | Not derived | food, housing, waking | 0.3.0 | [#193](https://github.com/SAIOS-SAMN/xylarium/issues/193) | open |
| S097 | F | `crates/xyl-kernel/src/growth.rs` power | Starlight = flux x 340 W/m2 x surface | Placeholder (EQUATIONS world heat) | housing, builders | 0.3.0 | [#14](https://github.com/SAIOS-SAMN/xylarium/issues/14) | open |
| S098 | F | `crates/xyl-kernel/src/cosmic/emergence.rs` Cosmic::arise; `crates/xyl-cosmos/src/life/mod.rs` | A viable people is 500 bodies | Placeholder (EQUATIONS viability) | waking, reach, settling | 0.3.0 | [#18](https://github.com/SAIOS-SAMN/xylarium/issues/18) | open |
| S099 | C,H | `crates/xyl-kernel/src/civ/mod.rs` stand_of | A people falls below 1 body (waking needs 500); failing is a boolean | A share | fall, exodus, seats | 0.3.0 | [#192](https://github.com/SAIOS-SAMN/xylarium/issues/192) | open |
| S100 | C | `crates/xyl-kernel/src/civ/mod.rs` stand_of | Reach within four orbital places (an index window) | Not derived | exodus targets | 0.3.0 | [#192](https://github.com/SAIOS-SAMN/xylarium/issues/192) | open |
| S101 | B | `crates/xyl-kernel/src/civ/mod.rs` expand | Emigrants are half the target's housing, capped at half the parent | Not derived | daughters, settle | 0.3.0 | [#191](https://github.com/SAIOS-SAMN/xylarium/issues/191) | open |
| S102 | D,E | `crates/xyl-kernel/src/cosmic/emergence.rs` Cosmic::settle | A daughter's capacity = its emigrants; growth copied from the parent, region only where the world has none | The daughter's world | thinkers, crowding | 0.3.0 | [#190](https://github.com/SAIOS-SAMN/xylarium/issues/190) | open |
| S103 | D | `crates/xyl-kernel/src/civ/mod.rs` carry | Environment 1 where no reading exists | Refused | housing | a7fdafd | [#179](https://github.com/SAIOS-SAMN/xylarium/issues/179) | open |
| S104 | C,E | `crates/xyl-kernel/src/field.rs` radiation | Radiation = all energy absorbed per year over a, floored at 1e-6 AU | Through the receiver's deficit (model.md, dispersal) | environment, housing | 0.3.0 | [#190](https://github.com/SAIOS-SAMN/xylarium/issues/190) | open |
| S105 | D | `crates/xyl-kernel/src/field.rs` environment | The first nonzero radiation becomes the people's adapted home | Not derived | environment, housing | 0.3.0 | [#179](https://github.com/SAIOS-SAMN/xylarium/issues/179) | open |
| S106 | A | `crates/xyl-kernel/src/universe.rs` discord_parts | Ancient peoples are skipped from discord | Not derived | quarantine, passage | 0.3.0 | [#192](https://github.com/SAIOS-SAMN/xylarium/issues/192) | open |
| S107 | H | `crates/xyl-kernel/src/transit/discord.rs` Network::holds_mouth; `crates/xyl-kernel/src/universe.rs` | J (holds a ley mouth) is 0 or 1 and switches discord off | A share | discord, quarantine, folds | 0.3.0 | [#192](https://github.com/SAIOS-SAMN/xylarium/issues/192) | open |
| S108 | D | `crates/xyl-kernel/src/universe.rs` discord_parts | An unknown z reads 1, so discord is 0 | Refused | discord, quarantine | 0.3.0 | [#179](https://github.com/SAIOS-SAMN/xylarium/issues/179) | open |
| S109 | B,H | `crates/xyl-kernel/src/transit/discord.rs` Network::fold | A fold once per generation at a fixed rate; worth and the ftl switch as booleans | Not derived | breach, discord | 0.3.0 | [#192](https://github.com/SAIOS-SAMN/xylarium/issues/192) | open |
| S110 | H | `crates/xyl-kernel/src/transit/building.rs` in_tune_stars | "In tune" is discord < 1/2 and "closed" is quarantine >= 1/2 | Shares, not cuts | progenitor targets, quarantine | 0.3.0 | [#192](https://github.com/SAIOS-SAMN/xylarium/issues/192) | open |
| S111 | D | `crates/xyl-kernel/src/transit/building.rs` begin | An unknown mouth reach reads 1 kpc in the tithe and 0 in the crossing time | Refused | line cost, passage years | 0.3.0 | [#179](https://github.com/SAIOS-SAMN/xylarium/issues/179) | open |
| S112 | C,A | `crates/xyl-kernel/src/transit/tolls.rs` quote; `crates/xyl-kernel/src/transit/building.rs` | The toll is the midpoint of floor and ceiling; new lines default to Tolled | Not derived (tolls partial) | quote, carries | 0.3.0 | [#192](https://github.com/SAIOS-SAMN/xylarium/issues/192) | open |
| S113 | E,C | `crates/xylarium/src/observe/network.rs` passages; `crates/xylarium/src/observe/galaxy.rs` | observe reports passages at best_spec; the galaxy probe defaults to the Earth's reach 8/15 | Read the people's drive; no Earth default | observe output | 0.3.0 | [#192](https://github.com/SAIOS-SAMN/xylarium/issues/192) | open |
