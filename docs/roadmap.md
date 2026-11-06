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

Angles are exact degrees on a named plane, read in sectors of base 60 and written in degrees
and their places (`xyl_core::coordinate`, LAW-COORDINATE v1). Phases about a throat are exact; drawn values
carry the `2^-40` resolution of the draw.

### Algebra and geometry

- Geometric algebra `Cl(p, q)` over `Q` (`xyl_core::cga`), the signature
  carried by the multivector's type; its conformal model `Cl(n+1, 1)`
  provides points, versors and dilators. The nucleon poles described below
  are its origin and its point at infinity.
- Rotors (`xyl_core::rotor`) are rational quaternions, the even part of
  `Cl(3)`. Every manifold holds its frame as an exact rotor, and a rotor
  turns a vector by the quaternion action (`Rotor::apply`), never by a
  matrix.
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
  stars' transduction produces light with the 3Z hold conserved.
- **3Q, the capacity.** 3Q is three times the number of states in every
  shell the atom reaches: `cap = sum_n sum_(l < n) 2 d(l)` and
  `3Q = 3 cap`. The balance `R_Q = 6Z - 3Q` and its share
  `rho_Q = (2Z - cap) / cap` mark the seals where `rho_Q = +1`, at Z 2, 28,
  208 and 1092 (`xylarium observe seals`).
- **R, the polarity.** The electron phase state
  `R = kappa (1 - 2 frac(p))` lies in `[-kappa, kappa]`. Its sign is the
  hemisphere of the bounce's pulse `p` (a share of 360 degrees on the
  isoclinic planes), and its magnitude is the
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

1. Done ([#26](https://github.com/SAIOS-SAMN/xylarium/issues/26)): the
   rotor's inverse (`Rotor::inverse`) and the certified angle between two
   directions (`rotor::angle_between`) replace the `acos` readings in
   `manifold::gap` and `chain::Place::arc`.
2. The atom's catalogue columns (mass number, phase temperatures, density)
   are to be derived from 3Z, 3Q and R rather than read from
   `data/elements.tsv` (milestone 3, chemistry from the census).

## Open items

The items identified so far, each a GitHub issue (linked) with the labels
shown (labels as in the shared standard: `defect`, `proposal`, `track`,
`docs`, `law`; `needs-decision`, `cross-project`, `good first issue`).
Work on an item goes through a branch and a pull request that cites it.
Items are listed by area in dependency order; an item's milestone is given
where it belongs to one.

### Verification

1. **Readings on `main` against `v0.3.0`**
   ([#2](https://github.com/SAIOS-SAMN/xylarium/issues/2), `defect`, blocker).
   Since `v0.3.0` the founding, the stars, time, the jetstream and the chemistry
   changed with unit tests only and no reading of a running universe. Found seed
   7 with both and compare `observe universe`, `observe worlds 200` and `observe
   galaxy`: stars placed on their manifolds and worlds per star. Where `main`
   has regressed, find the commit by the same readings and fix forward. Every
   law change from now on shows these readings before and after.
2. **The fresh-home default carries to 14 Gyr**
   ([#3](https://github.com/SAIOS-SAMN/xylarium/issues/3), `defect`). Done in
   [#28](https://github.com/SAIOS-SAMN/xylarium/pull/28): a home without a
   checkpoint is read at the first mind's dawn, where genesis writes its first
   checkpoint.

26. **Readings take their time argument in rings**
    ([#29](https://github.com/SAIOS-SAMN/xylarium/issues/29), `defect`). Done:
    `observe universal-core [R]` and the R of `people N [R]` and
    `sync [K] [R]` are rings of the bell, with Gyr shown only as a display
    conversion.

### Foundations (milestone 3, layer 0)

28. **One angle unit** ([#40](https://github.com/SAIOS-SAMN/xylarium/issues/40), `defect`). Done for the coordinate
    (LAW-COORDINATE v1: exact signed lanes in sectors of base 60, the T lane
    written by `xyl_core::coordinate::beat`); left, the angle APIs still in
    radians (`dyadic`, `turning`, the phases of `galaxy::Orbit` and `cores`;
    item 40), and the code that holds angles as shares of a whole turn,
    whose comments change with it: every turn on a named plane in degrees,
    with its return read from the deficit at its reach
    ([model.md](model.md#turns-axis-degrees-and-the-return)).
29. **The flow law and the dilation in `xyl-core`** ([#42](https://github.com/SAIOS-SAMN/xylarium/issues/42), `defect`,
    `law`). Done: `D(p)`, the flow law and the deficit share in
    `xyl_core::shares`, and the squared share removed; left, the rotors'
    composition and the field's twist rotor, which go with the frame carrier
    (item 40).
30. **Duplicate laws** ([#43](https://github.com/SAIOS-SAMN/xylarium/issues/43), `defect`). Done: one deficit function, one
    meeting law (`formation::founding_shells`), one certified exponential,
    ln 2 and pi. Left: the ocean's own pulls beside `Substrate::pulls`, the
    moons' placement beside `manifold`, and the two rotation carriers
    (item 40).
31. **Rule breaks** ([#44](https://github.com/SAIOS-SAMN/xylarium/issues/44), `defect`). Hole spin from `Stream::derive`,
    refusal instead of zero, `survival`, coherence in meetings.

### Founding (milestone 3)

32. **The founding as flow** (depends on item 38; axioms and theorems
    in [model.md](model.md#axioms-and-theorems-of-the-founding); the state
    at the pinch per layer and circuit
    ([model.md](model.md#the-state-at-the-pinch-the-fold-and-its-circuits));
    the bounce at a pinched throat
    ([model.md](model.md#the-bounce-at-a-pinched-throat)); the core as a
    paired throat whose spin is a sheared orbit turned by what each face
    releases, read in place of the reach-based admission
    ([model.md](model.md#the-core-as-a-paired-throat),
    [#86](https://github.com/SAIOS-SAMN/xylarium/issues/86)); the galactic
    cores' multiplicity is still to be derived (#79): the gate gives one
    chain; the firmament is the
    closed kinds, proved in EQUATIONS.md, Throat holding; [#35](https://github.com/SAIOS-SAMN/xylarium/issues/35), `track`, `law`). Parcels leave their
    throat at onset and move by the reach rate
    ([model.md](model.md#the-founding-as-flow)); angular state in degrees with
    the swing as torsion and twist; perturbation on frames; meetings by
    contrast times coherence; concentrations contract into throats of their
    own. Resolves items 3, 4 and 5 below.
33. **Genesis structure** ([#36](https://github.com/SAIOS-SAMN/xylarium/issues/36), `defect`). The bead's triple use,
    homologue galaxies, stars by index and the forecast of minds are replaced
    by tiers formed from the flow.
34. **The echo moves position** ([#37](https://github.com/SAIOS-SAMN/xylarium/issues/37), `defect`, `law`). The echo acts on
    angular state and frames only. Done: the reach-scaled admission
    (`least_share`) is removed with the paired throat (#86). Left:
    `push_at` and `beyond_at` banding reaches by the swing, `Push::swing`,
    and the pulse carrying world positions in `field`.
35. **Parcels carry counts** ([#38](https://github.com/SAIOS-SAMN/xylarium/issues/38), `defect`, `law`). Counts of nuclei
    with pole state and heat; chemistry is what a parcel closes into.
36. **Fixed tables and chains** ([#39](https://github.com/SAIOS-SAMN/xylarium/issues/39), `defect`). Each replaced by what
    the parcel closes into, as its layer is rebuilt.
37. **Closed forms, not steps** ([#41](https://github.com/SAIOS-SAMN/xylarium/issues/41), `defect`). Each loop over time or
    sampled age replaced by a closed form or an event as its module is
    rebuilt.
38. **The ring as the throat orbit**
    ([#60](https://github.com/SAIOS-SAMN/xylarium/issues/60), `law`). A ring
    is the reading of the core's sheared elliptic orbit at the throat (the
    axis dilation by its exponent, each plane's phase in Q/Z, never one
    scalar); the echo's twist `n = a / (4 + a)` moves Q up and Z down about the equator and is
    held by the dipole as torsion, `Q / Z = 1 + a / 2`
    ([model.md](model.md#the-ring-the-throat-orbit-and-the-echos-torsion)).
    Done: each throat's echo is its sheared return at that throat's
    torsion share, `bell / (1 - s)` on the near face and `bell / s` on the far
    (`circuit::echo_at`), the universal core's founding at the founded
    share and every tier after its bounce at its released spin's. Left:
    the ring replacing the bell as the unit of time, and the ringing as a
    per-plane phase rather than one cosine over a period (item 40).
    Replaces the bell as light's round trip to the rim, which the founding
    as flow cannot define. With it, the equator: the inflow `D(-s)` by
    continuity, viscosity `t / (1 + t)` reverting across the throat, the
    positron as the reverted phase and the pair share `s (+) (-(1 - s))`
    ([model.md](model.md#the-equator-inflow-viscosity-and-pairs)); the
    antipole group, the shear the dipole reconciles, `p_pair = n (+) n/3`,
    and the pinch where the folds meet
    ([model.md](model.md#the-antipoles-the-shear-and-the-pinch)); what a
    pinch carries, the next throat's spin `a_p M_p / H_g` (Kerr) and the stretching
    pressure on its states
    ([model.md](model.md#what-a-pinch-carries-the-next-throat-and-its-pressure)).


3. **Elements attached to the filament's released pressure**
   ([#4](https://github.com/SAIOS-SAMN/xylarium/issues/4), `proposal`, `law`).
   The cascade assigns each kind one reach and exchanges between stacks on the way out
   (`formation::founding_shells`). In its place: the
   bounce releases the torsion share's pressure along the filament, falling with
   reach as the vacuum's share; each kind attaches to it by its open frames and
   is held back by its held frames, and settles where the two balance, as a
   distribution over reach. Shells and bands become concentrations read from
   those distributions. Sealed kinds do not attach and stay as the firmament.
4. **Recycling through the cores**
   ([#5](https://github.com/SAIOS-SAMN/xylarium/issues/5), `proposal`, `law`).
   Matter flows one way: the firmament leaves the core, stars lock gas into
   remnants, and nothing returns. The jetstream's vacuum leg is to carry matter
   back: what galaxies and stars no longer hold (remnants, the passed share at
   each throat crossing, released gas) returns to its galactic core by the
   vacuum's share at its reach, and the galactic cores to the universal core.
   The books become one cycle across the universe: held, in flight outward, held
   downstream and in flight inward together equal founded.

40. **The frame, the place and the chain**
    ([#90](https://github.com/SAIOS-SAMN/xylarium/issues/90), `defect`,
    `law`; [model.md](model.md#the-frame-the-place-and-the-chain)). One
    frame carrier over the axis plane and five planes in `xyl_core::cga`,
    composed along the anchor chain, replacing `xyl_core::rotor::Rotor` and
    the three coordinates of `chain::Place`; tier maps by
    `manifold::dilation` only; separation and containment by caps in
    degrees; shear, stretch, torque and twist on every link; exact degrees on named planes.
    Its shape is settled by T18 (founded matter turns its five planes
    together): an axis-plane phase, one isoclinic phase and the turning
    between planes that carries a body into its parent; that turning is
    to be derived from the fold's writhe where a link forms. The H lane
    follows FORMS-COORDINATE v3 (Mycelium): each component keyed by its
    generator (the plane named by its two axes, a turning between planes by
    the axes it mixes), held in degrees as an exact rational with a rate in
    degrees per bell advanced in closed form, `K` counted per component, and
    the turning between planes stored as a separate component; composition
    through the generators awaits the turning law (Netstratum docs/117).
43. **Scalar collapses**
    ([#107](https://github.com/SAIOS-SAMN/xylarium/issues/107), `defect`,
    `law`;
    [model.md](model.md#scalar-collapses-the-register-and-the-order-of-opening)).
    The register of 50 sites where a structured quantity is one number, a
    graded reading a boolean, or a choice a scalar, and the order of
    opening: the bond and stripping as graded shares; candidates held
    together; booleans to shares where they gate existence; relations held
    by tier; lanes and planes kept whole; enclosures kept.
42. **Chemistry from the elements**
    ([#103](https://github.com/SAIOS-SAMN/xylarium/issues/103), `law`,
    `track`; [model.md](model.md#chemistry-from-the-elements); with items 7,
    14 and 16). Chemistry and biology emerge from the founding's physics at
    the atomic tier, a bond being a paired throat read by its separation and
    holding its turning as remanence; nothing is named in advance. The vents,
    in order: received turning per plane in place of a temperature; closure
    held as a superposition, never chosen by release; a bond network's state
    as its sheared orbit, the minds naming what it reads; nuclides as
    attractor pairs; living matter on the synaptic chain; no Earth unit in a
    law. It replaces the Milky-Way calibration, the solids and frozen-matter
    tables, the six catalogued volatiles (whose formulas disagree with
    `compound::closing`), the Earth isotopes, and the tabled bond, cohesion
    and drive constants.
41. **The automorphic, synaptic chain**
    ([#97](https://github.com/SAIOS-SAMN/xylarium/issues/97), `law`,
    `proposal`; depends on items 39 and 40;
    [model.md](model.md#the-automorphic-synaptic-chain)). One
    attractor-pair map at every tier, commuting with the tier's dilation
    (at the law: `xyl_core::shares`, invariant under every dilation, the
    tier ladder one instance); a link's
    weight its stored turning between planes, its memory the remanence,
    strengthened by a passage less separated than its hold and weakened by
    one more separated, conserved between passages (T19); the pulse carried link
    to link; the council's convergence on the same links (#94).
39. **Phase weakening and phase strengthening**
    ([#84](https://github.com/SAIOS-SAMN/xylarium/issues/84), `proposal`,
    `law`; depends on items 32 and 34;
    [model.md](model.md#phase-weakening-and-phase-strengthening)). A
    link's remanence is derived (T19): built from passages by their
    separation, conserved between them, cleared at its coercivity.
    Remanence of the pole state: the part of a drive's displacement held
    when the drive falls to its mean, and its coercivity; the inverse
    driven by mass at distance from a core (a star's own included), by a
    throat with an Einstein-Cartan bounce, or by core fusion, as antipoles.
    Builds on `circuit::push_at`, `circuit::jet_map`,
    `Substrate::loop_flux`, `circuit::electron_phase`,
    `xyl_core::shares::deficit` and `stars::fusion_step`; needs the held
    residual as a closed form of the drive's history.

44. **The ocean as a carry** ([#113](https://github.com/SAIOS-SAMN/xylarium/issues/113), `law`;
    [model.md](model.md#the-held-medium-the-ocean-as-a-carry)). What reaches
    a place is what the ocean's medium carries there, never a push. Done:
    the held medium inside a reach (`Substrate::held_within`,
    `Substrate::medium`: the core's 1, the beads nearer, less what the jet
    carried past), read by the waveform, a bead's ring hold, a galaxy's
    circulation and push, and a piece's star cut; the ocean's motion
    (`Substrate::carry`): outward `s (+) (-(1 - s))` per echo from the
    echo's faces, `360 delta` degrees a traversal on the isoclinic planes
    from the torsion. Next, in order: (1) the star's received quantity as a
    carry, a star's life read about the galaxy tier's own medium
    (`stars::ring_per_bell` still takes `h = 1` there), the hold inside the
    star's orbit (item 46, first), and graded
    stripping (T20, held on `graded-stripping`) re-derived on it; (2) the
    reverberation's interior hold, `1 + (firmament nearer) / unit`, put on
    the same footing as `H(r)` (it divides a mass by the beads' total); (3)
    viscosity `nu(t) = t / (1 + t)` built, replacing the laboratory
    reference in `bodies::interior`; (4) the pairs the outward carry bears
    (`p_pair`), the ocean's particles.

### Galaxy (milestone 3, layer B)

5. **Fluid bands** ([#6](https://github.com/SAIOS-SAMN/xylarium/issues/6),
   `proposal`). The galaxy's bands (today "rings", the founding's beads read at
   the galaxy tier) have fixed edges, no flow between them and hard cuts between
   neighbours. Matter is to circulate along the shear of neighbouring orbits,
   bands to blend where the founding's shells overlap, and edges to move with
   their holding. The word "ring" is kept for the bell's unit of time.
   First step: item 46.
46. **The hold inside an orbit**
    ([#116](https://github.com/SAIOS-SAMN/xylarium/issues/116), `law`,
    `defect`, `needs-decision`;
    [model.md](model.md#the-hold-inside-an-orbit)). A body is caught in an
    orbit, never a band: `H(r) = 1 + sum m_b chi_b(r)`, each nearer orbit
    counted by the share of its swing within `r` (the reverberation's swing
    law, eccentricity the deficit about the hold inside it). Replaces the
    step in `Substrate::held_within` and, at the galaxy tier, the bands of
    `galaxy::Structure` and `inside_mass`, so one model reads both tiers
    (#43). Needs decision: the galaxy tier's divisor (the bands' holding
    taken as the core's 1; the beads total 0.4008 core masses). Unblocks
    item 44 step 1.
6. **Each galaxy's own structure**
   ([#7](https://github.com/SAIOS-SAMN/xylarium/issues/7), `proposal`). Every
   other galaxy is the home galaxy scaled by its hole mass
   (`cores::Shape::scale`); each is to be read from the bead it formed from.
7. **Chemistry from the founding and the stars**
   ([#8](https://github.com/SAIOS-SAMN/xylarium/issues/8), `track`, branch
   `chem-from-stars`). Replaces the Milky Way chemical-evolution model. Before
   merge: systems born before their band's first throat crossing come out empty,
   and each band's return uses one representative star rather than every star
   born along it.

### Stars (milestone 3, layer B)

8. **No iron peak** ([#9](https://github.com/SAIOS-SAMN/xylarium/issues/9),
   `needs-decision`). The atoms' held frames per nucleon fall across the whole
   table, so every fusion step releases hold and the sequence runs to superheavy
   kinds.
9. **Many-body fusion steps**
   ([#10](https://github.com/SAIOS-SAMN/xylarium/issues/10), `needs-decision`).
   Where the table holds no kind at twice a kind's nucleon count, the least `k`
   can be eight or ten nuclei in one step, while bonding on S^11 is pairwise.
10. **The transduction's multi-axial path**
    ([#11](https://github.com/SAIOS-SAMN/xylarium/issues/11), `proposal`). The
    pole point moves by the jetstream's axial map; torsion in the five isoclinic
    planes (`bounce::spin_bivector`) is specified, not built.
11. **Remaining star types**
    ([#12](https://github.com/SAIOS-SAMN/xylarium/issues/12), `proposal`). The
    remnant's own cut (compact, denser and throat-compact remnants), the
    ignition condition of a held body that does not ignite, the boundary of a
    supergiant, and the binary companion's share (`0.1 + 0.9 u`, imposed).

### Worlds (milestone 3, layer C)

12. **The temperature law**
    ([#13](https://github.com/SAIOS-SAMN/xylarium/issues/13), `needs-decision`).
    A world's heat is the held share of its star's light; its temperature is to
    follow either as held energy per particle, `k_B T = held / particles` (a
    first estimate gives far too cold a world), or from a radiation balance with
    the universal constants (a fourth root at a law's root).
13. **Climate from the held light**
    ([#14](https://github.com/SAIOS-SAMN/xylarium/issues/14), `track`). Replace
    278 K (`Spec::t_form`, `worlds::t_unit`), 340 W/m^2, 288 K and the fitted
    `greenhouse_tau`; the greenhouse from nested held shares.
14. **Condensates from the disc's mixture**
    ([#15](https://github.com/SAIOS-SAMN/xylarium/issues/15), `track`). Replace
    the fixed ice species and the rock and metal mineralogy (`systems::ICES`) by
    the clusters the disc's atoms bond at the heat a world holds.
15. **World units** ([#16](https://github.com/SAIOS-SAMN/xylarium/issues/16),
    `track`). Orbits in the system tier, radii in the planet tier, masses as
    shares of the star's manifold, the day and year as degrees of spin and orbit on their planes; SI only through
    `package::Si` for universal constants.
16. **Geochemistry from the atoms**
    ([#17](https://github.com/SAIOS-SAMN/xylarium/issues/17), `track`). Replace
    the bulk-silicate partition (`systems::MANTLE_KEPT`) and the measured
    heat-producing isotopes; delete `worlds/unit.rs`.

### Life and peoples (milestone 3, layers D and E)

17. **Life rulers from worlds**
    ([#18](https://github.com/SAIOS-SAMN/xylarium/issues/18), `track`). The
    mind-bearing body (10 kg), its metabolism (19.1 W) and generation (14.2
    years), and the viable people (500).
18. **Peoples and drives**
    ([#19](https://github.com/SAIOS-SAMN/xylarium/issues/19), `track`). Peoples'
    times in rings; drives from each people's chemistry in place of the 18 amu
    exhaust and the mass ratio of 10.

### Network

19. **The pulse front**
    ([#20](https://github.com/SAIOS-SAMN/xylarium/issues/20), `cross-project`,
    `needs-decision`). Netstratum's wavefront derivation (its docs/111): the
    held medium's delay `r + H(<r)`, containment until the first exit, escape by
    bonded collections, and stars and remnants as surfaces.
20. **Mycelium v0.3.0**
    ([#21](https://github.com/SAIOS-SAMN/xylarium/issues/21), `cross-project`).
    Pin mycelium v0.3.0 by git tag when it is cut: the `mycelium-sdk` crate
    (Voice, Session, tasks and solves, the rule seam, seats, wallet and leases,
    readings) is the client surface, the shared physics library `forms` carries
    law ids and versions that EQUATIONS rows declare, the wire states its
    version and act table, and a client conformance read checks that a client
    speaks the same Anastome. The `xyl-hypha` crate of milestone 5 starts on it.
27. **Shared laws in the core shell**
    ([#33](https://github.com/SAIOS-SAMN/xylarium/issues/33), `cross-project`,
    `needs-decision`). Concordance sections 1.14 and 2.10: the laws listed in
    [EQUATIONS.md, Shared laws](EQUATIONS.md#8-shared-laws) land in mycelium's
    `forms` one law per pull request, written by the coordinator from
    Xylarium's forms, refusals and fixtures; a law is universal only by a
    Concordance amendment, otherwise a named form of Xylarium's shell.
    Xylarium then calls the forms in `Q` and deletes its copies. Also owed:
    signing the Concordance (section 0) at the v0.3.0 pin, and the amendment
    of sections 1.8 and 8.5 stating each universe's unit at founding
    (proposed on #33).

### Project

21. **Adopt the open-source standard**
    ([#22](https://github.com/SAIOS-SAMN/xylarium/issues/22), `track`,
    `needs-decision`). MAINTAINERS.md, ROADMAP.md and GLOSSARY.md at the root,
    `scripts/check`, labels and templates, these items as issues, and a
    protected `main` once confirmed (`/opt/handoff/OPEN-SOURCE-STANDARD.md`).
22. **Conduct and security**
    ([#23](https://github.com/SAIOS-SAMN/xylarium/issues/23), `needs-decision`).
    The content of CODE_OF_CONDUCT.md and SECURITY.md.
23. **Release v0.4.0** ([#24](https://github.com/SAIOS-SAMN/xylarium/issues/24),
    `track`). Cut only from a commit whose readings show stars placed and worlds
    forming.
24. **Earth anchors** ([#25](https://github.com/SAIOS-SAMN/xylarium/issues/25),
    `track`). `scripts/calibration-anchors` counts the remaining Earth, Sun and
    Milky Way anchors in code (722 on `main`).

### Numerics

25. **Rotor inverse and the versor angle**
    ([#26](https://github.com/SAIOS-SAMN/xylarium/issues/26), `proposal`, `good
    first issue`). Done: `Rotor::inverse` and the certified
    `rotor::angle_between` replace the `acos` readings in
    `manifold::gap` and `chain::Place::arc`.
45. **Ring keeps one meaning**
    ([#115](https://github.com/SAIOS-SAMN/xylarium/issues/115), `defect`).
    "Ring" is the harmonic of the bell, echo and pulse only. Done: the
    length units formerly read as "core rings" and "galaxy rings" are the
    cluster unit (`60^7 r_g`) and the galaxy unit (`60^5 r_g`) in docs,
    comments, labels and identifiers (`galaxy::units_of_kpc`,
    `galaxy::kpc_of_units`, `galaxy::Star::reach_units`), with no formula
    changed. The band sense goes with item 46.

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

Done when: `xylarium observe galactic-core N` reads each galactic core's
hole and its mass ratio to the universal core, with each core no heavier
than the universal core, and `observe universal-core` shows the substrate's
books closing.

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
   rotor inverse and the certified angle in `xyl_core::rotor` (done, #26);
   `manifold::gap` is next to be read in the lowest common tier.

Done when: `xylarium observe worlds` shows metal-rich stars holding several
worlds, and `observe system S` reads each relation in the lowest tier
holding both ends.

## 3. Rules from the founding and the galactic cores (current)

Goal: Every rule above the founding is a quantity derived from the
universal core's founding and the galactic cores. No law reads a measured
property of the Earth, its star or its galaxy. SI units appear only at
display, through `package::Si`, and the universal physical constants (c,
G, k_B, h) are the only measured values the laws read.

Why this order: The layers above the founding were ported from a prototype that
ran in units calibrated to the Earth, its star and its galaxy: kiloparsecs,
stellar masses, Earth masses, astronomical units and Earth years. Every one of
those units passes through the same Earth root: the kiloparsec through `r_g` in
metres, which comes from the core's mass set to a measured value of a real
galaxy's central black hole (`package::hole_mass`), and the year through the
Earth's orbit (`Si::yr_s`). Changing a unit without deriving the law behind it
keeps the Earth value under a new name, so each layer is converted whole,
bottom-up: its lengths to its tier's unit (`xyl_core::units::Tier`), its masses
to shares of the founding's chain in core masses, its times to rings of the
bell, and every law that read a measured value is derived from the founding and
the galactic cores in the same change. A law with no derivation yet is not
carried over in a new unit; it is listed under the layer as open.

Rules for every layer:

- Units: tier lengths, core masses (the universal core is 1), rings. A
  time is a position on the temporal coordinate (`xyl_core::coordinate`,
  the T lane): whole rings plus a phase in degrees, minutes and seconds of
  arc; local time between orbital bodies is their relative phase.
- No conversion of an Earth-valued constant: no `turning::span_in_rings`
  or `circuit::rings_of_gyr` of a value declared per Gyr, no `Si::` in a law. SI
  appears only in `package::Si` for display and for the universal
  constants (c, G, k_B, h).
- Verification by the founding's own invariants (conservation balances,
  shares summing to one, ordering), not by agreement with earlier readings.

Measure: `scripts/calibration-anchors` counts the remaining anchors in
code by category.

Landed:

- Builds: the founding and forecast digests hash code only
  (`crates/xyl-cosmos/build.rs`), so documentation edits reuse the cached
  founding; the anchor counter reads code only, by category.
- Layer A, founding and substrate: the bell is exact in core time
  (`Anchor::bell = 2 rim 60^5`, units of `r_g / c`); first stars, onsets,
  rates, outflow, intake, landing, waveform and push are in rings. SI
  seconds enter only through `Si::seconds_per_ring`, used by the display
  and the temporary boundary helpers `circuit::rings_of_gyr` and
  `gyr_of_rings`, which are removed when the last layer is converted.
- Layer B, part 1, the turning law, the galaxy's structure and star orbits, and
  the vertical orbit: the turning law is `omega = delta(<r) bell / r` radians
  per ring in core units (`turning::turning`, `turning_dy`); the structure,
  `inside_mass`, `stellar_mass`, star reaches and orbit radii are in galaxy
  rings and core masses; the vertical orbit is derived (`nu = omega 2d / (1 +
  d)`, `h = r m / (M(<r) + m)`, `s = t / (1 + t)`, `t` the age over the ring's
  cooling time, `sigma = h nu s 2d / (1 + d)`). The layers above read kpc and
  the prototype's stellar-mass unit through the marked boundaries
  `galaxy::kpc_of_units`, `galaxy::units_of_kpc` and
  `galaxy::suns_of_core_masses` / `core_masses_of_suns`; the SI forms of the
  turning law (`turning_si`, read in rad/s by `Si::rad_s`) remain as
  boundaries for the system and body layers.
- Layer B, part 2, star masses and star light: a piece's star mass is the share
  of its infall the star tier's cut holds, `m_p = s c` core masses
  (`galaxy::piece_star_mass`); the initial mass function draws a share `x = b u
  / (1 + b - u)` of it, the inverse of the deficit form `x / (x + b)` with `b =
  d / (1 + d)` from the ring's crowding (`galaxy::imf`), in place of the
  measured initial mass function's bounds. A bead is a galaxy where `lit x state
  >= m_p` of the first piece to light and `state <= 1`
  (`Substrate::holds_a_star`, `Substrate::galaxies`). Star light is the power
  the transduction transduces, `P = m transduced_per_bell(r) / bell` core masses
  per ring (`stars::power`, `stars::light`); the life is the first step over the
  transduction, in rings. The system layer reads the prototype's stellar-mass
  unit through the boundaries `galaxy::suns_of_core_masses` and
  `stars::suns_of_power`.
- Layer B part 3, star compactness from the atoms: each star's deficit at its
  rim, `delta = m / (R 60 + m)`, with `R` the rim of its cloud's cut
  (`disc::rim_share`, `galaxy::Star::compactness`). The cut is read at collapse
  (dark), ignition (the transduced share is `delta0` of the received turning)
  and settled (the cut at that transduction's over-pressure). Transduction,
  power, light, life and the giant's end take the star's compactness; the
  measured stellar radius and the rim share built on it are deleted.
- Layer B, stellar transduction by polarity: each cloud's dark cut records its
  held mixture (`disc::StarHold::held`); the star's sequence emerges from it at
  the cut's heat (`stars::Sequence`): the pole point `t0 = sum n u / sum n w`,
  the throat after `ln(1 / t0) / ln rho_s` echoes, fusion steps to the kind at
  `k A` and fission steps to two kinds of equal total, each stage the held kind
  of greatest release, re-read from the updated mixture. Life, step ends,
  remnant, `field::Transduction` (`Unlit`, `Step`, `Remnant`) and the
  transduction-out share read the sequence; the fixed hydrogen and helium
  shares and the measured binding energies are deleted.

- One law per formula: the torsion share `(H + J) / (2H + J)` for every
  throat (`bounce::torsion_share`), the dilation `(1 + p) / (1 - p)` for
  every holder (`shares::dilation_dy`), and the throat crossing of the echo
  map `t -> a t + b` in closed form (`circuit::throat_crossing`).
- The jetstream for every holder: out through the continuum, back through
  the vacuum (`circuit::jet_map`). The firmament's onsets and each star's
  collective cross the throat by the same law; a star's timeline runs its
  fusion steps until the crossing and its fission steps from it
  (`stars::timeline`, `stars::remnant`).
Next:

0. The founding as flow, in this order; each step is one pull request with
   its readings (item 32, [model.md](model.md#the-founding-as-flow)):
   1. Foundations in `xyl-core` (items 28-31): degrees with the ring count,
      `D(p)` and the flow law, one deficit share.
   2. The parcel and the reach rate: release at onset, the reach law in
      closed form between events, the throat's books
      `H_0(t) = 1 - sum outflow`; the fixed reaches (`formation::founding_shells`,
      `s_final`, `runs_of`, `bead_run`, `place_beads`) deleted.
   3. Angular state and meetings: phase and plane in degrees, swing as
      torsion and twist, meetings by contrast times coherence.
   4. Concentrations: galactic cores where flowing parcels contract; each
      galaxy's own rings from its core's outflow; homologues and the bead's
      triple use deleted.
   5. Stars from clouds in a galaxy's flow; `galaxy::star`'s placement by
      index and the mass draw deleted.
   6. Systems at their formation event, worlds from the disc's parcel, life
      read from a formed world; the forecast of minds (`minds::schedule`)
      replaced by events, and `xyl-kernel`'s reads of it moved to them.
   Layers B to F below are rebuilt on this flow rather than on the beads.
1. Layer B, the galaxy tier (`galaxy`, `stars`, `chem`, `pulse`, `field`,
   `turning`, `centre`, `manifold`). The interim conversions of Gyr-valued
   rulers made in this layer are replaced:
   - Stellar transduction by polarity, timing: the fission steps start at the
     throat, `n*` echoes after ignition, rather than at the end of the fusion
     steps' cumulative transduced share.
   - Chemistry from the founding: enrichment is the hold each star's
     transduction returns to its ring, from the census
     (`circuit::Substrate::census`) and the births along each ring
     (`chem::Chemistry::birth_along`), in place of the Milky Way model's rulers,
     the reference metallicity, the gas pattern `ism_today_dex`, the flat speed
     `v_flat` and the calibration epoch `Rulers::today`.
   - The pulse, the field's energy unit and the galactic-core episodes in
     core units.
2. Layer C, systems and worlds (`disc`, `systems`, `bodies`, `worlds`,
   `hydro`): orbits in the system tier's unit and radii in the planet
   tier's; world masses as shares of the star's manifold; a world's day and
   year as degrees of its spin and orbit, each on its plane, per ring of its star; the world's heat as
   the deficit share it holds of its star's light at its reach, its
   temperature from that held share and its re-emission as that heat turned
   outward (model.md, Radiation: irradiation held, transduced and passed
   on), in place of 278 K, 340 W/m^2, 288 K and `greenhouse_tau`;
   the mantle and the heat-producing isotopes from each element's hold and
   open frames and the star's birth mixture, in place of the bulk-silicate
   Earth tables and the 4.57 Gyr epoch; `worlds/unit.rs` deleted.
3. Layer D, life and minds (`life`, `minds`): rung waits and strike rates
   in rings; the mind-bearing body, its metabolism and generation, and the
   viable people derived from the world's gravity, temperature and
   chemistry, in place of 10 kg, 19.1 W, 14.2 years and 500.
4. Layer E, the kernel (`xyl-kernel`): peoples' times in rings, drives from
   each people's chemistry in place of the 18 amu exhaust and the mass
   ratio of 10, and the remaining boundary helpers removed.
5. Layer F, the package at the seam: `package::hole_mass` supplied by the
   package and read only for display; the founding read from a manifest of
   nuclei per `Z` ([network.md](network.md)); `package::Si` keeps only the
   universal constants and the display conversions.

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
6. The council's decisions as the convergence of its seats' readings, a
   decision that does not converge held
   ([#94](https://github.com/SAIOS-SAMN/xylarium/issues/94);
   [civilizations.md](civilizations.md#minds-in-the-ancient-seats)).
7. Consciousness passed from the ancient seats to the progenitor, steward
   and sovereign seats as a derived share (#94; needs item 3).
8. The house's relay: mu settled by the uplink for the home's survival
   resources, with in-game costs physical (#94; network.md, Next steps,
   item 1).
9. Deliberation between houses bound by the delays of their percepts
   (#94).
10. Done: the uplink's exact T reading and a mind's focus as four exact
    shares (`knowledge::Focus`); the frame's lane byte stays S until
    Mycelium retires it
    ([#108](https://github.com/SAIOS-SAMN/xylarium/issues/108), #107).
11. Embedded minds: the uplink hosts the client's cognition, one per
    ancient seat, through the house's seat-surface path, linked to the
    hypha under each mind's key (#108;
    [network.md](network.md#a-mind-on-both-faces)).
12. The council's convergence act on the embedded and joined minds (#94).

Done when: `xylarium observe house` shows a seat held by a mind across a
restart, and a decision from anyone but the holder is refused.

## 5. The xyl-hypha crate

Goal: A universe joins the hypha as a domain through a `xyl-hypha` crate
pinned to Mycelium `v0.3.0` by git tag, implementing `run::Link` over a
`mycelium-sdk` session ([network.md](network.md)). Mu, the Mycelium network's unit,
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
4. The coordinate's origin, ordered key and neighbour ranges
   (LAW-COORDINATE v1, fixtures 1, 5 and 6) are to come with bodies
   addressed by coordinate.
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

- **Package** (`package.rs`): The universal core's mass, (declared, a measured
  value of a real galaxy's central black hole) (`hole_mass`), is declared until
  the package supplies it. The SI seam (`Si`: the unit world, the reference
  star, the AU, the kpc, c, G, the year and the physical constants) is kept for
  display once each layer reads core units.
- **Units** (`xyl_core::units::Tier`): the tier ladder, each tier's unit
  `60^k r_g` (the universe 60^10, the galaxy 60^5, a system 60, a planet
  60^-2), is declared. Every law is invariant under any dilation between
  tiers (`shares::tests::deficit_is_invariant_under_dilation`), so the
  ladder's base is a reading convention, to be replaced by each tier's own
  dilation as the chain derives it (item 41).
- **Founding** (`bounce`): The horizon's held share `delta_h = 1/3`
  (`horizon_reach`, so `H = 2M`), the reach where the received share is twice
  the held share, is to be derived from the bounce's geometry; until then the
  shared torsion law takes `H` as an argument.
- **Home galaxy and chemistry** (`galaxy`, `chem`): The chemistry's rulers
  (`chem::Rulers`: ring width, surface density, returns, loading, yields) are to
  be derived from the enrichment that the founding produces.
- **Systems and worlds** (`systems`, `worlds`, `bodies`, `turning`): The
  light's warmth `278 L / a` (`Spec::t_form`), the world scales and
  cosmic-ray reference (`worlds::crowd_r0`), the unit world's age of 4.57
  Gyr at which its isotopes and its star's activity are measured
  (`bodies::unit_age_gyr`), the laboratory rock and iron
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
  to be derived from each people's chemistry. The flux ruler is still
  written twice (`life` and `growth::Rulers::flux_w`) until `life` exposes
  it; the r/K rule and body size of waking (`cosmic::select`) still repeat
  the rule inside `life::foragers`, which no longer gates waking, until
  `life` exposes them.
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
