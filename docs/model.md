# The physical model

This document describes Xylarium's physics: the master frame, the founding,
and the derivation of each tier from the tier before it. Every formula, with
its code location and status, is listed in [EQUATIONS.md](EQUATIONS.md);
terms are defined in [glossary.md](glossary.md); declared constants are
listed in [roadmap.md](roadmap.md), Declared rulers.

The following status words are used throughout:

- Built: the quantity is derived from the laws above it and read by the
  engine.
- Partial: the quantity is derived, but it rests on a declared input or is
  read only by the observer.
- Placeholder: the quantity is a declared constant or calibration that
  stands until a law derives it.

Module paths are relative to `crates/xyl-cosmos/src` unless they are
prefixed with another crate (`xyl-core`, `xyl-kernel`).

## 1. Numbers

Three number types, defined in `xyl-core`, carry every law:

- `Q` (`q.rs`) is an exact rational in lowest terms, promoted to big
  integers only when a value exceeds the range of `i64`. Division is checked
  (`Q::checked_div`). Held state is `Q`: element counts, firmament and bead
  amounts, reaches, spans, phases, the anchor, star masses and births, and
  world masses.
- `Encl` (`certified.rs`) is a closed interval of `Q` proven to contain a
  transcendental value (pi, exp, ln, sin, cos). It is used wherever a
  decision or a held value requires a proven bound: pi in the anchor and in
  the bounce's curved reach, the island abundances, galaxy orbits and places
  (`chain::Place`), manifold caps and shares (`manifold`), and the closed
  forms of the chemistry (`xyl-core motion.rs` `ExpSum`).
- `Dy` (`dyadic.rs`) is a dyadic rational `m 2^e` with a 62-bit mantissa,
  rounded toward zero in integer arithmetic and therefore identical on every
  machine. It is used for heavy readings at seams: the substrate's clock,
  reverberation, outflow, waveform and push (`circuit::core`); the settling
  (`systems`); the stars' transduction (`stars`); and life and minds (`life`,
  `minds`). A `Dy` value is converted to `Q`, often on a 2^-40 or 2^-64
  grid, before it is held.

Angles are held as exact turns where the rate is rational and are read in
degrees: the coordinate (`xyl_core::coordinate`) writes a value as degrees,
minutes, seconds and three finer sexagesimal places of arc, with the whole
turns kept (on the T lane, the rings). Phases about a throat are exact: a
kind's capture phase is the torsion share turned Z times, whose denominator
divides the share's, so it never grows. Only drawn values carry a grid,
the 2^-40 resolution of the draw.

## 2. Units and the master frame

- Base units are set by the universal core: `G = c = M_core = 1`, so the
  core's mass, called the universal state, is 1. The unit of length is the
  core's gravitational radius `r_g = G M_core / c^2` (`xyl-core units.rs`).
- Each tier (see [glossary.md](glossary.md)) has a length unit that is an
  exact power of sixty of `r_g` (`xyl-core units.rs` `Tier::in_rg`):
  universe `60^10`, cluster `60^7`, galaxy `60^5`, system `60` and planet
  `60^-2`. A conversion between tiers multiplies by a power of sixty
  (`manifold::dilation`) and is exact. The substrate's reaches (core rings)
  are in cluster units; the home galaxy's rings and every star's reach are
  in galaxy units.
- The bell (`circuit::anchor`, `Anchor::bell`) is the round-trip time
  of light to the home galaxy's rim, `bell = 2 rim 60^5` in core time
  (`r_g / c`), with the founding's rim read in the galaxy tier's unit. The
  bell is the unit of time: every time is a count of rings and every rate
  is per ring. A time is a position on the temporal coordinate:
  whole rings plus the phase within the ring, its seconds being seconds of
  arc of that phase. SI seconds enter only at the SI seam
  (`package::Si::seconds_per_ring`, `bell r_g / c`); `circuit::rings_of_gyr`
  and `gyr_of_rings` convert at the boundaries of layers still counted in
  Gyr and at display.
- A ring of time is one bell; counts of time (cooling, first light, a star's
  transduction and life) are whole numbers of rings. An echo is two bells, one
  ring for each face (near and far) of the paired throat (`circuit::echo`). A
  one-way crossing to reach `r` takes `(r / rim) bell / 2`, a fraction of a
  ring.
- The pulse along the filament (the pole axis; see
  [glossary.md](glossary.md)) crosses the universe's rim in the same half
  bell, so the speed in the ocean (the medium between galaxies) is
  `c_ocean = rim / (bell / 2) = 3600 c`, the ratio of the cluster tier's
  unit to the galaxy tier's unit (`Substrate::c_ocean`).
- The anchor (`circuit::anchor::of`) is computed from the founding and fixes
  the following values: the rim; one cluster unit and one galaxy unit in
  kpc; the bell; the home galaxy's rim (`rim x galaxy_kpc`); the universe's
  curvature radius, `rim / pi` (each enclosed sphere's rim is half a great
  circle), in kpc and in `r_g`; the first stars, at `max(cools_in, 1)`
  bells; and the parent's admitted spin, its torsion share and the
  firmament's margin. The anchor is stored once per process
  (`circuit::anchor::set`, a `OnceLock`), so the first founding in a
  process sets it.
- The SI bridge is `package.rs`. `package::hole_mass` (the core's mass in SI
  units, declared by the package) is the only conversion from core units to SI:
  it fixes `r_g` in metres and thereby every reading displayed in kpc, Gyr, AU
  or kilograms. The SI constants used at seams are in `package::Si`. Laws stated
  in core units do not depend on the bridge; the chemistry and the world states,
  which read SI-calibrated constants, do depend on it. Next: the core's mass is
  to be supplied by the package (roadmap, Rules from the founding and the
  galactic cores).

## 3. The bounce and the founding

### The bounce (`bounce.rs`)

A universe begins as the white-hole face of a parent's black hole, computed
exactly in the conformal algebra `Cl(12, 1)` over `Q` (eleven base axes: the
pole axis and the five isoclinic planes).

- Horizon reach: the hole holds the deficit share `delta = M / (R + M)` at
  reach `R`. The horizon is where the received share `1 - delta` is twice
  the held share, `delta_h = 1/3`, so `R_h = M (1 - delta_h) / delta_h = 2M`
  (`horizon_reach`).
- The reach is taken along the S^360 enclosure that contains the core, of
  curvature radius `rho`: `rho sin(R_h / rho)` (`curved_reach`).
- The crossing point lies on the first of the pulse's five isoclinic planes,
  at the pulse's phase (`crossing`); the pole axis is reserved for the
  filament. The horizon is the conformal sphere about the centre through the
  crossing point (`horizon`, `xyl-core cga::sphere_through`).
- The spin is the bivector `K = a sum_k B_k` over the blades of the five
  planes (`spin_bivector`). The white hole is the null generator
  `N = P ^ (P . K)` at the crossing point (`xyl-core cga::Source`); it is
  verified to be null, fixes `P` and preserves the horizon. The throat's far
  face is the crossing point's antipode under the planes' half turn
  (`circuit::plane_antipole`).
- `Bounce::of` returns `None` for a spin outside the open interval `(0, 1)`
  and for a non-positive mass or enclosure, so a spinless hole yields no
  bounce.

### The origin (`origin.rs`)

- The parent's Kerr spin is the only value drawn from the seed: a uniform
  rational strictly inside `(0, 1)` on the 2^-40 grid (`Origin::of`,
  `clamp_spin`). The founding depends on the spin alone, so the substrate is
  founded first and the parent bounce is read in the substrate's enclosure:
  `Bounce::of(1, spin, radius_rg, 0)`.
- The torsion share (`bounce::torsion_share`, one law for every throat) is the
  share that the outbound jet carries through the throat. At the throat reach `H
  = 2M` the deficit of one face against the other is `delta(H, H) = 1/2`; the
  spin `J` is a further share that the throat does not hold, so the held face is
  `H / (2H + J)` and the jet carries `a = (H + J) / (2H + J)`. The share is 1/2
  at zero spin and 3/5 at extremal spin. The inverse is `J = H (2a - 1) / (1 -
  a)` (`spin_of_share`).
- Admission (`cores::substrate_for`) requires the torsion share to be at
  least the furthest held kind's reach over the rim, scaled by the peak of
  the inbound echo, `a >= r_far / rim (1 + A_in_peak)`
  (`Substrate::least_share`). Where the condition fails, the spin is raised
  to the spin of that share and the universe is founded again, at most three
  times; the margin `a - least_share` is held on the anchor.
- The cascade yields four readings. Lumpiness is the beads' states above the
  mean over the sum of all bead states. Held share is the firmament's mass
  over all founded mass. Assembly is the number of bells until the home
  galaxy's gas first lights. Efficiency is the lit share of that gas
  (`formation::Lighting`). These readings are computed from the founding; no
  further value is drawn from the seed.

### The elements (`elements.rs`, `atoms.rs`)

- In the capture chain (`elements::founded`), nuclei are built one capture at a
  time from hydrogen, `count_Z = floor(count_(Z-1) a h_Z)`, where `h_Z = (3Z -
  u_Z) / 3Z` is the closed share of the new nucleus and `u_Z` is its number of
  unpaired windings (`atoms::unpaired`). Counts are whole nuclei out of `2^b`
  hydrogen nuclei, where `b = founding_bits(a)` is the smallest number of bits
  that carries one whole nucleus of the vessel's sealing kind through the
  floored chain. A kind whose count floors to zero is absent from the founding.
  Next: the shares of the light kinds are to be read from the same founding as
  the firmament (roadmap, Rules from the founding and the galactic cores).
- The table runs from Z = 1 to `TOP_Z` = 1700. `data/elements.tsv` holds Z 1 to
  99 as measured rows and Z 100 to 150 as predicted rows, each predicted from
  its homologue by the periodic law. Z 151 to 1700 are generated with systematic
  names, the mass number of the valley of stability (`elements::valley_a`) and
  uranium as the homologue for the material columns. The founding replaces the
  abundance column with the chain's counts; mass numbers, phase temperatures,
  densities and geochemical classes remain catalogue values. Next: those columns
  are to be derived from the atom (roadmap, Rules from the founding and the
  galactic cores, chemistry from the census).
- The atom (`atoms::atom`) is computed on S^11. A subshell of shape `l`
  holds `2 (C(l + 11, 11) - C(l + 9, 11))` electrons (orientations 1, 12,
  77, 352, ...), so shells hold 2, 26, 180 and 884 electrons, and the seals
  (every band and every shape of the outermost shell full; see
  [glossary.md](glossary.md)) fall at Z 2, 28, 208 and 1092. Electrons fill
  the tightest open winding, where tightness is `(Z - inside) / n`.
- The vessel (`atoms::vessel`) is the band of kinds that the throat holds.
  It extends from the heaviest seam below the last seal (a kind whose
  single-nucleon peel releases, `u(z - 1) + 1 - u(z) < 0`) to that seal.
  Both edges are computed; on the current table the band is Z 568 to
  Z 1092.

### The founding as flow

Matter is not placed. At the bounce every founded kind is held at the
universal throat, the universal state 1, and each leaves it as a parcel
carried by the jetstream. The laws below decide where each parcel goes, how
it changes, and where parcels gather.

**The parcel.** A parcel carries the counts of nuclei of each kind (atoms
and clusters, `compound`), each kind's pole state (held frames `w`, open
frames `u` and the electron phase `R`), its heat (the held share of the
irradiation it has received), its hold `m` (a reading of its counts on the
founding, in core masses), its reach `r` from its throat (in `r_g`) and its
angular state (its phase and plane in degrees on the S^360). Counts and
frames are conserved through every change; mass is read from the counts,
never carried.

**Release.** Kind `k` leaves the throat at its onset, the echo at which its
pole state crosses the throat under the jet map (`circuit::onset`,
`circuit::throat_crossing`), at the throat's outflow rate
`m0 (1 - exp(-rate (t - onset)))`, from the throat's horizon reach
`R_h(H_0) = 2 H_0`. What has left is no longer held:
`H_0(t) = 1 - sum_k outflow_k(t)`. A kind released over an interval is a
filament: the locus of the parcels released along it.

**What the throat holds.** A kind with no open frames has no pole state,
so the jet never acts on it and the throat holds it. An open kind's pole
point `t = U / W` is its position on the axis plane, and its net axis
weight `A` (Weights on the six planes) is its moment there; the flow law
adds their hyperbolic angles, so the kind dilates by `D(s) D(n)^A` per
echo on the continuum. It is held where that is at most one (with the
vacuum leg, at most `1 - 2 delta`), that is `A <= -log_D(n) D(s)`; for seed
7, `A <= -8` exactly, which holds 461 open kinds (Z 65 to 124, 279 to 338,
422 to 762) beside the 20 closed ones, with no change to the held mass at
twelve digits (they are sparse). Every other open kind reaches the throat
and leaves.
The firmament is therefore the closed kinds (for seed 7, 20 kinds with
43.5% of the founded mass, from Z 2 to the seal at Z 1092), not the
vessel's band; the present `beyond_share` test, which holds the vessel by
its assigned reaches, goes with the cascade. Heavier closed kinds exist
(22 between Z 1093 and Z 1700) but are not founded, because the founding's
budget stops at the vessel's seal.

**Reach.** Two pushes act on a parcel at reach `r` with the hold `H` inside
it (the throat's remaining state plus every parcel nearer the throat):

- the continuum, outward: the deficit of the inner hold at the parcel's
  reach, `p_c = H / (r + H)`;
- the vacuum, inward: the parcel's own deficit at its reach,
  `p_v = m / (r + m)`.

They oppose, so they compose by the flow law
`x (+) y = (x + y) / (1 + x y)` as `p_net = p_c (-) p_v`:

```text
p_net = (H - m) / (r + H + m)
D(p_net) = D(p_c) / D(p_v) = (r + R_h(H)) / (r + R_h(m))
```

with `D(p) = (1 + p) / (1 - p)` the dilation and `R_h(M) = 2M` the
horizon reach. The reach changes at the rate of the dilation's excess, in
echoes `tau`:

```text
dr/dtau = r (D(p_net) - 1) = [R_h(H) - R_h(m)] r / (r + R_h(m))
```

A parcel expands where `H > m`, rests where `H = m`, and returns toward its
throat where `m > H`; the return is the recycling of matter through the
cores. Far out (`r >> R_h(m)`) it coasts at `R_h(H) - R_h(m)` per echo; the
rim is the reach of the outermost parcel at `t`, not a boundary. With `H`
and `m` fixed the law is separable and exact:

```text
tau - tau0 = [ (r - r0) + R_h(m) ln(r / r0) ] / (R_h(H) - R_h(m))
```

and `r(tau)` is its monotone inverse, found by `xyl_core::event`. The law is
a rate, as every motion in `xyl_core::motion` is; between events `H`
changes only through the throat's closed-form outflow, because the matter
inside a reach acts as one hold.

**Angular state.** A parcel's phase about its throat advances by the turning
law, `dphi/dt = delta(r, H) bell / r`, in degrees. The echo's swing is
torsion and twist: it turns the parcel's phase and plane by the torsion
share of the throat's spin, and never moves its reach. Parcels interact by
the degrees between them, the coherence `c = 1 - 2d` (`circuit::coherence`,
`d` the shorter way round as a share of a turn, so `c` is in `[0, 1]`): 1 in
phase, 0 opposed. Opposed parcels do not exchange.

**Perturbation.** The part of the echo a parcel's elements cannot pass, its
held share `m / (r + m)`, reverberates in its frames: the vacuum channel of
the parcel's loop (`circuit::Substrate::loop_flux`, swung by the inbound
echo) moves frames between held and open, and so sets its heat. The same
net push `p_net` dilates the parcel's pole state per echo. Chemistry is what
a parcel's kinds close into at the parcel's heat (`compound::closing`,
`atoms::closes`), and a parcel whose pole state reaches the throat
(`t = 1`) begins to transduce (`stars::Sequence`).

**Meetings.** Where two parcels reach the same `r`, they exchange matter by
the contrast of their deficits `T` times their coherence, with `sum m l`
conserved (the passage of `formation::cascade`). Meetings and onsets are the
events of the founding; between them every parcel's path is the closed form
above.

**Concentrations and tiers.** A concentration is a reading: where parcels
share reach and degrees and their gathered hold grips more than its
agitation, the shear and the echo's heat (the collapse test of
`formation::gas`), they contract (`m > H`). A contracting concentration is a
new throat, with a hold, an outflow and parcels of that throat: a galactic core,
then a cloud, then a star. Each tier is a body contained in its parent; it
exists only from its formation event, and what it takes its parent no
longer holds.

## 4. The firmament and the core

- The firmament (`circuit::Substrate::found`, `circuit::Kind`) consists of
  the held kinds inside the vessel band, each at its reach (its spin about
  the throat) and its capture phase. Their amounts are scaled, preserving
  their founded proportions, so that their total mass is the universal
  state, 1, which is the core's mass. The held share is then
  `1 / (1 + unit)`, where `unit` is the sum of the beads' states
  (`held_share`).
- The deficit law gives the share that a mass `M` holds at reach `R`, and
  every hold in the model takes its form: `delta(R, M) = M / (R + M)`, with
  received share `1 - delta`. The vacuum's share at reach `R` about a hold
  `H` is that deficit, the push that the pulse delivers there
  (`xyl_core::shares::deficit_dy`); read at the throat, it is the throat's push,
  `p = delta(H, H) = 1/2` (`circuit::throat_push`). The jet carries the
  throat's torsion share `s = (H + J) / (2H + J)` at each crossing
  (`bounce::torsion_share`), so the continuum dilates the pole coordinate
  by `rho = D(s) = (1 + s) / (1 - s)` per echo (`circuit::jet_dilation`):
  3 for a spinless throat, rising to 4 at the extremal spin.
- Reverberation (`Substrate::reverberation`) takes the firmament kinds
  in order of spin. For each kind the hold is
  `h = 1 + (firmament mass nearer) / unit`, the reach `r = spin / h`, the
  deficit `delta = h / (r + h)`, the share of its band past the throat
  `beyond = clamp((r (1 + delta) - h) / (2 delta r), 0, 1)`, and the leaving
  rate `beyond / bell` (per ring).
- In the jet, a kind's pole state sits at `t = open / hold` on the filament
  and its jetstream carries it out through the continuum at the torsion
  share `s` and back through the vacuum at the kind's vacuum share `delta`
  (`circuit::jet_map`); it reaches the throat after
  `ln((1 - t*) / (t0 - t*)) / ln a` echoes, which is its onset
  (`circuit::onset_echoes`, `circuit::throat_crossing`); a sealed kind has
  no pole state and no onset. The kind then
  empties as `m(t) = m0 (1 - exp(-rate (t - onset)))` (`Substrate::outflow`).
- The inbound echo (`Substrate::inbound`) is the waveform at the throat per
  echo, `A_in(t) = cos(2 pi t / echo) + echo q(t)`, where `q(t)` is the
  vessel's outflow rate in whole state (`Substrate::rate`). Its peak is
  `1 + echo max q(onset)` (`inbound_peak`), and its mean over the first `t`
  is the drift plus the ring's rung (`inbound_mean`). The push at `R` and
  `t` is `h / (R + h) (1 + A_in(t))` (`push_at`).
- In the jetstream (`Substrate::loop_flux`, `circuit::pole_flux`), three
  channels compose as dilators on each firmament kind's pole state over one
  echo: the filament's dilation at the throat, the vacuum's swung share of
  the hold, and the electrons' phase state `R` on the open frames
  (`circuit::electron_phase`, `R = kappa (1 - 2 frac(p))`). The residual is
  the observed dilation over the product of the channels, which is the
  torsion that the echo leaves on the kind. The loop closes, conserving the
  kind's 3Z hold, where the residual is 1. There is one jetstream per held
  kind. Status: partial (read by the observer).
- The ocean is described by the waveform at reach `r`,
  `A(r, t) = [cos(2 pi (t - tau) / echo) / echo + q(t - tau)] / (r + 1)`,
  with `tau = (r / rim) bell / 2` (`Substrate::amplitude`). A galaxy's share
  of a kind that the jet carries is its pull `h / (r + h)` (`h` its state
  over the mean) times its coherence with the kind's phase,
  `1 - 2 min(d, 1 - d)`, divided by the sum over all galaxies
  (`Substrate::landing`). A galaxy's composition at `t` is its recipe plus
  its intake (`composition`). Mass is conserved, and `books` checks it:
  held + left = founded, and taken + in flight = left.
- The push on a galaxy (`Substrate::push`) is push over inertia on the
  galaxy's spin about the throat, in closed form,
  `l(t) = l0 + d [Q(t - tau) + sin(2 pi (t - tau) / echo) / 2 pi] /
  ((R + 1) H)`.
  Recession is `dl/dt / inside`, the shift is `z = v / c_ocean`, and the
  tide across an extent `x` is `A(R - x) - A(R + x)`.
- Galactic cores (`cores::founding_of`, `Substrate::holds_a_star`): a bead's
  state is its mass in the universal core's units, so the mass of its core reads
  `hole_suns = hole_mass x state`. A bead is a galaxy where the lit share of
  that mass is at least the mass of one star on the chain, `lit x state >= m_p`
  in core masses, with `m_p` the star mass of the piece that lights first
  (`galaxy::first_star_mass`), and where its core is no heavier than the
  universal core, `state <= 1` (`Substrate::galaxies`): a heavier core would
  hold the universal core rather than circle it. Each galaxy circulates about
  the throat at `4 delta` turns per echo, with `delta = inside / (reach +
  inside)`. Its plane is set by the repulsion pivots one tier up (the node by
  the golden turn, the tilt by the plastic turn's share of the S^360 measure),
  and its place at `t` is its plane's rotor applied to its phase
  (`Shape::place_at`). The stars of another galaxy are the home galaxy's stars
  read through that galaxy's length scale `2 m / (1 + m)`, where `m` is its hole
  mass ratio (`Shape::scale`, placeholder).

### Perturbations, throats and reverberation

Every core is one face of a throat: the universal core's white face is
paired with the parent's black face, and each galactic core is one face of
a throat between its galaxy and the enclosure beyond (`bounce::Bounce`,
`centre`). A core's bounce is a perturbation
of its enclosure, and the construction is spherical:

- **The perturbation is a sphere.** A bounce emits one perturbation that
  grows from the core (S^0) across the enclosure that contains it, the
  sphere of curvature radius `rho` on which every reach is taken
  (`bounce::curved_reach`, S^360 at the universe's scale). Its front at a
  given time is the sphere of points at equal light-time from the core.
  There is no oscillator: the bell is the time the front takes to the home
  galaxy's rim and back (`Anchor::bell`, `2 rim 60^5` in `r_g / c`).
- **Throats separate the cores.** Between two cores the perturbation meets
  a throat, the sphere through the crossing at the one-third deficit,
  `R_h = 2M` (`bounce::horizon`). A throat's two faces are a near face and
  a far face, so a perturbation that reaches a throat continues on the
  far face.
- **Reverberation is the perturbation held between faces.** One ring is
  the front's round trip on the near face; the echo is two rings, the near
  face's ring and the far face's return (`circuit::echo`). What the throat
  holds of each pass swings with it: each firmament kind's band swings over
  `r (1 +/- p)` about the throat with `p = 1/2`, and the share carried past
  the throat leaves as the jet (`Substrate::reverberation`,
  `circuit::onset_echoes`). The inbound echo at the throat,
  `A_in(t) = cos(2 pi t / echo) + echo q(t)`, is the reverberation read at
  the core (`Substrate::inbound`).
- **The throat is the equator.** The deficit law splits every exchange
  into a held share `delta = M / (R + M)` and a passed share `1 - delta`.
  At the throat itself `delta(H, H) = 1/2` (`circuit::throat_push`): the
  held and passed shares are equal, the fixed point of the inversion
  `delta -> 1 - delta`. The same inversion on the pole axis,
  `X(t) -> X(1 / t)`, swaps the Z pole (held) and the Q pole (open) and
  fixes the throat at `t = 1` (`circuit::axis_antipole`).
- **Arrival at a reach.** The front's arrival at reach `r` is
  `(r / rim) bell / 2` after the bounce, and its amplitude falls with the
  vacuum's share about the universal state, `1 / (r + 1)`
  (`Substrate::amplitude`). In the galaxy tier the front is carried
  between holes by the relay network (`pulse::relays`): every galactic core
  and stellar remnant passes it on continuous in phase. Next: the relay is
  under review against a single front echoed only at surfaces (cores,
  the firmament, the rim) with the held medium's delay `r + H(<r)`, shared
  with Netstratum (roadmap, milestone 3).

### The ring: the throat orbit and the echo's torsion

Planned ([#60](https://github.com/SAIOS-SAMN/xylarium/issues/60)); it replaces the bell as light's round trip to the
rim. Units: `G = c = M = 1`, reaches in `r_g`; counts are in Z and turns in
Q/Z (exact shares of a turn, never decimals); an irrational value (pi, a
cosine at a general turn) is read only as a certified enclosure for
display, never stored.

- **The orbit.** About a hold `M`, a body at reach `r` moves at the deficit
  share of light's speed, `v = delta = M / (r + M)` (the turning law), and
  the reverberation's swing `r (1 +/- delta)` is its radial excursion. The
  orbit is an ellipse with semi-major axis `r` and eccentricity
  `e = delta`, periapsis `r_p = r (1 - delta)` on the held (Z) side and
  apoapsis `r_a = r (1 + delta)` on the open (Q) side. Its apsis ratio is
  the dilation: `r_a / r_p = (1 + delta) / (1 - delta) = D(delta) =
  1 + R_h(M) / r`.
- **The shear.** The orbit does not close. Each traversal advances its
  periapsis by `delta` of a turn (`turning::twist_per_turn`, `360 delta`
  degrees), and the spin moves the orbit across the five isoclinic planes
  (`bounce::spin_bivector`): every direction turns on a great circle of
  S^9 in the ten dimensions the planes span, and the shear carries the
  orbit from one circle to the next.
- **Resonance.** An orbit returns exactly after `q` traversals, `q` the
  denominator of its advance in lowest terms. The reaches `r_k = (k - 1) M`
  advance `1 / k` of a turn and return after `k`: the throat (`k = 2`,
  apsis ratio 3, the throat push's dilation), the horizon (`k = 3`, apsis
  ratio 2), and so on; the inversion `r -> M^2 / r` maps each to
  `M / (k - 1)` on the far face, the throat to itself.
- **The ring.** A ring is the reading of the core's orbit at the throat
  (`r = M`, `delta = 1/2`), never one scalar. At a count `t` (exact in Q) it
  is

  ```text
  R(t) = ( the axis:    D(n)^t, held by its exponent t
           the planes:  phi_i(t) = t theta_i mod 1, i = 1..5, each in Q/Z
           the returns: the whole traversals k in Z and each plane's order )
  ```

  each plane with a rate `theta_i` of that plane, the hold's deficit share there
  (all equal to `s` while the spin is isoclinic), and the readings are never
  summed. Every stored value is rational: a duration in `r_g / c` or in
  seconds (one traversal lasts `2 pi M / s` in `r_g / c`) is a display
  conversion through a certified enclosure of pi, never a quantity. An epoch
  is a position in this reading, nested through the tiers (each tier read on the
  throat orbit of that tier), not a count of a scalar unit; a reading moves to any count
  in closed form, and the count live play advances per loop is read against
  the network's height per segment, never fixed. With no spin the planes
  advance half a turn per traversal and return after two: the echo is the
  throat orbit's return.
- **The local ring.** At reach `r` about a hold `M`, the local ring is one
  traversal of the orbit at `r`, and every law step at that reach (a turn,
  the vacuum's halving of an agitation, a cooling) counts local
  traversals. Two orbits' traversals compare exactly, since pi cancels in
  the ratio of their angular rates: one local traversal at `r` lasts

  ```text
  (r / delta(r)) (s / M) = s r (r + M) / M^2   core traversals
  ```

  The engine counts core traversals; each tier reads the orbits at the reaches
  within it and converts by this ratio. The turning law is then one turn per
  local traversal, `(delta(r) / r) (M / s)` turns per core traversal; no
  light-time and no SI enters a law.
- **The twist and the dipole.** With spin `a`, the throat carries the
  torsion share `s = (2 + a) / (4 + a)` on the open side and holds `1 - s`
  on the held side. Each echo twists the open pole Q up and the held pole Z
  down about the equator `1/2` by half the twist `n`:

  ```text
  n = 2s - 1 = a / (4 + a)        the twist per echo, in Q/Z
  Q = (1 + n) / 2,  Z = (1 - n) / 2
  Q / Z = (1 + n) / (1 - n) = D(n) = 1 + a / 2
  ```

  Q moves as the antipole of Z (the inversion `t -> 1 / t` exchanges
  them), and the dipole keeps the twist as torsion: `Q / Z` depends on the
  spin alone and does not drift into a phase. The twist is the echo's
  residual, held per echo, never summed into the count.
- **Influence.** The derivatives of the readings in the spin and in the
  reach are exact:

  ```text
  dn/da = 4 / (4 + a)^2,   dQ/da = 2 / (4 + a)^2 = -dZ/da,   d(Q/Z)/da = 1/2
  dD(n)/dn = 2 / (1 - n)^2
  d delta/dr = -M / (r + M)^2,   dD(delta)/dr = -2M / r^2
  ```

  The dipole's ratio is linear in the spin; the twist's influence falls as
  `(4 + a)^-2`; an orbit's eccentricity and apsis ratio fall with reach.

### The equator: inflow, viscosity and pairs

Derived ([#60](https://github.com/SAIOS-SAMN/xylarium/issues/60)); not yet built. Theorems are composed by the flow law
before anything is added, and reduced to its axioms: the deficit share,
the flow law, the conservation of counts and the inversion.

- **The equatorial inflow.** The jet stretches the medium along the pole
  by `D(s) = (1 + s) / (1 - s) = 3 + a` per echo. Since `s (+) (-s) = 0`,
  `D(s) D(-s) = 1`: the stretch composed with its opposed push is the
  identity, which is continuity. The equatorial span therefore shrinks by
  `D(-s)`, and the equator draws in the share `1 - 1 / D(s) =
  (2 + a) / (3 + a)` per echo. Read from the equator, the dipole is
  `Z / Q = (1 - n) / (1 + n) = 1 / D(n)`.
- **The swirl.** The turning law's circulation, `Gamma(r) = 2 pi r v =
  2 pi M r / (r + M)`, is the passed share: full outside the core,
  suppressed inside it, the core's hold the throat. It is the swirl of a
  stretched vortex in deficit form.
- **Viscosity.** A medium is viscous where its frames grip their
  neighbours and fluid where they do not: closed kinds and bonds heat has
  broken slide. In the pole coordinate `t = U / W` the viscosity is the
  open share, `nu(t) = t / (1 + t)`, the deficit form `delta(t, 1)`; heat
  enters through `compound::open_hot`. The inversion reverts its polarity:
  `nu(1 / t) = 1 / (1 + t) = 1 - nu(t)`, so what grips on the fusion side
  (`t < 1`) slides on the fission side (`t > 1`), and at the throat
  (`t = 1`) both faces read one half.
- **Positrons.** The electrons' phase state is `R = kappa (1 - 2x)`,
  `x = frac(p)` in Q/Z (`circuit::weights::electron_phase`). The reversion
  runs the echo backwards, `x -> -x`, and `R(-x) = -R(x)`: the positron is
  the electron's phase read through the reversion, with the same
  coherence and the opposite sign. A pair arises at the echo's turn
  (`x = 1/2`, `R = 0`) and separates, the electron toward the held face
  and the positron toward the open face; meeting in phase, `R + (-R) = 0`
  returns to the dipole.
- **The pair share.** The echo's near face carries the open share `s` out
  and its far face returns the held share `1 - s` through the reversion,
  opposed. Composed:

  ```text
  p_pair = s (+) (-(1 - s)) = 4n / (3 + n^2) = a (4 + a) / (12 + 6a + a^2)
  D(p_pair) = D(s) D(-(1 - s)) = (3 + a)(2 + a) / (6 + a)
  ```

  `p_pair` is the share of the electrons' phase the far face does not
  return; it separates as `p_pair / 2` electrons on the held face and
  `p_pair / 2` positrons on the open face per echo. A spinless throat
  makes none (`1/2 (+) (-1/2) = 0`); the extremal spin makes `5 / 19`.

### Weights on the six planes

Derived ([#60](https://github.com/SAIOS-SAMN/xylarium/issues/60)); the counts are built and tested
(`atoms::tests::a_bands_harmonics_split_by_weight_on_the_six_planes`), the
rest is not yet built. The algebra's twelve base axes are six planes: the
axis plane (axes 0 and 1), where the Z-Q stretch acts, and the five
isoclinic planes (axes 2 to 11), which the spin turns together; together
they are C^6, where the bands' Hopf fibres lie. Turning each plane on its
own splits a band's harmonics by their weight `m = (m0, m1, ..., m5)` in
Z^6, each plane's reading kept, never summed into one number.

- **Multiplicity.** A degree-`l` harmonic of weight `m` occurs
  `C(q + 4, 4)` times, `q = (l - sum |mi|) / 2` (none where it is not a
  whole number). By axis weight alone a band holds `C(l - |m0| + 9, 9)`
  harmonics at `m0`; both distributions are symmetric under `m0 -> -m0`
  and under the twist charge `c = m1 + ... + m5 -> -c`.
- **Flow.** One bivector generates a state's motion: a dilation in the
  axis plane (hyperbolic; the dipole's `D(n)`, the flow law being the
  addition of its hyperbolic angle) and the spin's turn `s` in the five
  planes. The parts lie in orthogonal planes and commute, so a state's
  reading at a count `t` is a closed form: on the axis the dilation
  `D(n)^(m0 t)`, exact in Q at whole `t`; on the planes the phase
  `t s (m1 + ... + m5) mod 1`, exact in Q/Z. The state moves on a flat line
  of the six-torus inside S^11, curved by the embedding; in each plane a
  circle stretched by the axis's dilation is an ellipse with apsis ratio
  `D`; each plane's rate is the hold's deficit share there.
- **Filling within a band.** The axis's dilation holds Z-side states
  (`m0 < 0`) tighter and stretches Q-side states (`m0 > 0`), so a band's
  electrons take the Z side first, singly before pairing. A band less than
  half full keeps its unpaired electrons on the Z side; one more than half
  full has paired the Z side and keeps them on the Q side. The atom's rest
  fill, which bands hold how many electrons, is unchanged.
- **Axis polarity.** A kind's net axis weight is the sum of `m0` over its
  unpaired electrons: Z side (negative), Q side (positive) or neutral. Of
  the founded kinds Z 1 to 1092, 535 are Z side, 517 Q side and 40 neutral
  (the 20 closed kinds and 20 open ones of net weight zero); for example
  carbon, nitrogen, oxygen and silicon are Z side (-1), iron is Q side
  (+1), hydrogen is neutral (its s-state has `m = 0`).
- **Open.** At rest every kind's net twist charge is zero (the `c`
  distribution is symmetric), so the twist's direction needs the spin to
  break the symmetry. The jet map moves hydrogen first, from
  `t = U / W = 1/2`, while its only state has weight zero: the pole
  coordinate counts frames (`3Z` per kind) and the weights count electron
  states (`Z`), and the two are not yet one account. The planes share one
  rate while the spin is isoclinic.

### The antipoles, the shear and the pinch

Derived ([#60](https://github.com/SAIOS-SAMN/xylarium/issues/60)); not yet built. Each check below is exact.

- **The antipole group.** A state of weight `m` has three images: its
  axis antipode `(-m0, m1, ..., m5)` (the throat's inversion, Z and Q
  exchanged), its plane antipode `(m0, -m1, ..., -m5)` (the phase reversal
  `x -> -x` that gives the positron, twist charge `c -> -c`), and both,
  `-m`. All four keep `sum |mi|` and so the multiplicity.
- **The flip keeps the states.** The twist unwinds by the plane antipode:
  a state moves from `c` to `-c` within its axis level. For every band to
  `l = 18`, every `(m0, c)` and its image `(m0, -c)` hold the same number
  of harmonics, every axis level keeps `C(l - |m0| + 9, 9)`, and the
  filling reads the levels only, so each band keeps its electrons and
  pairing and every founded kind keeps its open frames and axis polarity.
- **The shear on the antipode.** A state and its antipode lie on opposite
  faces, which advance `s` and `1 - s` per traversal, so the pair does not
  cancel: on the planes it drifts `c n` of a turn per traversal, and on the
  axis it dilates by `D(s)^m0 D(1 - s)^-m0 = D(p_pair)^m0`.
- **The dipole's reconciliation.** The pair's push splits exactly,
  `p_pair = n (+) (n / 3)`, `D(p_pair) = D(n) D(n / 3)`, per state
  `D(p_pair)^m0 = D(n)^m0 D(n / 3)^m0`. The dipole reconciles `n`, the
  twist it holds as torsion; the residual `n / 3 = a / (3 (4 + a))`, with
  `D(n / 3) = (6 + 2a) / (6 + a)`, is the shear it cannot hold, and it
  flips into writhe: the fold's rotation.
- **Fold forward and back.** The residual folds the near face forward,
  `+n / 3` of a turn per echo in the spin's direction, and the antipode
  back, `-n / 3`. Per echo the faces cancel, so `Lk = Tw + Wr` is
  conserved: the twist moves `+n / 2` and `-n / 2`, the writhe `+n / 3`
  and `-n / 3`.
- **The pinch.** The two folds meet where `N n / 3 = -N n / 3 (mod 1)`,
  first at the antipodal point, half a turn: after
  `N* = 3 / (2n) = 3 (4 + a) / (2a)` echoes, where the folds' relative
  writhe first reaches a whole turn and the linking number may change by
  one. The count is exact in Q; the fold's axis dilation there,
  `D(n / 3)^N*`, is held by its exponent, never as a value. A spinless
  core has no residual and never pinches.

### What a pinch carries: the next throat and its pressure

Derived ([#60](https://github.com/SAIOS-SAMN/xylarium/issues/60), [#35](https://github.com/SAIOS-SAMN/xylarium/issues/35)); not yet built; each identity checked
exactly.

- **The spin.** Degrees of turning are conserved (axiom 11 below): a
  throat of spin `a` and hold `M` turns `J = 360 a M^2` degrees, and matter
  leaving it carries `j = 360 a_p M_p` degrees of turning per unit hold.
  A pinched loop of hold `H_g` closes with `j H_g` degrees of turning, and
  by the Kerr relation its spin is `a_g = j / (360 H_g) = a_p M_p / H_g`.
  The degrees per unit hold are therefore the same at every tier, the
  universal core's `360 a_0 M_0` (for seed 7, `124 deg 47' 11"` and
  `5728280873/8589934592"`). A cohort closes only where its turning does
  not exceed a whole turn per unit hold, `j <= 360 H_g` (`a_g <= 1`); a
  lighter one stays in the flow as medium, and a heavy one makes a slowly
  spinning throat. The new throat's twist and pinch follow
  from its spin, `n_g = a_g / (4 + a_g)` and `3 (4 + a_g) / (2 a_g)`
  echoes; a spinless throat makes no pinch and no tier.
- **The place.** The pinch's coordinates (The antipoles, the shear and the
  pinch): half a turn, after `3 (4 + a) / (2a)` echoes.
- **The hold.** The matter the flow has in the fold when it closes, summed
  from the parcels' counts: a reading of the founding as flow, not a
  formula. Until the flow is built, the galactic cores' present mass
  ratios stand in for it.
- **Pressure on the states.** A throat's spin stretches its axis by
  `D(n) = 1 + a / 2` per echo, and a state of axis weight `m0` takes the
  stretch `m0` times, composed by the flow law: its push along the axis is
  `p(m0) = (D(n)^m0 - 1) / (D(n)^m0 + 1)`, exact at every whole `m0`
  (`p(1) = n`, `p(-1) = -n`, `p(2) = 2n / (1 + n^2)`). Q-side states are
  pushed out, Z-side states pressed in; a kind of net axis weight `A` feels
  `p(A)`, and is held where it balances the jet,
  `D(s) D(n)^A <= 1 - 2 delta`. Each throat holds an orbit, a stretch
  `D(n_k)` and a held set of that throat; the pressure follows each
  throat's spin.

### Axioms and theorems of the founding

Status: derived ([#35](https://github.com/SAIOS-SAMN/xylarium/issues/35), [#60](https://github.com/SAIOS-SAMN/xylarium/issues/60)); the sections above give each derivation.

Axioms:

1. **Exactness.** Counts are in Z, turns in Q/Z, everything else exact in
   Q; an irrational is a certified enclosure, never stored; no rational is
   rounded.
2. **Deficit.** A hold `M` at reach `r` keeps `M / (r + M)` of what reaches
   it.
3. **Flow.** Pushes compose as `x (+) y = (x + y) / (1 + x y)`;
   `D(p) = (1 + p) / (1 - p)`.
4. **Conservation.** Nuclei and frames are conserved; mass is a reading of
   the counts.
5. **Inversion.** `t -> 1 / t` exchanges the Z and Q poles; `x -> -x`
   reverses a phase.
6. **Horizon.** `R_h(M) = 2M`, the held share one third.
7. **Torsion.** `s = (R_h + J) / (2 R_h + J)`, `J = a M^2`.
8. **Six planes.** R^12 = C^6: the axis plane and five isoclinic planes,
   which the spin turns together; a state is a harmonic on S^11 with a
   weight `m` in Z^6.
9. **Frames.** Each proton spans three frames: open `U` (the unpaired
   electrons), held `W = 3Z - U`; the pole state `t = U / W`.
10. **Shear.** Every cycle is a sheared orbit read in closed form; nothing
    repeats closed and nothing is stepped.
11. **Degrees of turning.** The degrees a body turns, its hold times its
    degrees per unit hold, are conserved; the linking number's
    conservation is their topological reading.

Theorems:

| | Statement | From |
|---|---|---|
| T1 | `D(x (+) y) = D(x) D(y)`; continuity `s (+) (-s) = 0`, equatorial inflow `(2 + a) / (3 + a)` | 3, 4 |
| T2 | `D(s) = 3 + a`; twist `n = a / (4 + a)`; dipole `Q / Z = D(n) = 1 + a / 2` | 3, 7 |
| T3 | orbit about `M`: speed and eccentricity `delta`, apsis ratio `1 + 2M / r`; resonances `(k - 1) M` returning after `k` | 2, 6 |
| T4 | sheared return `k / (1 - theta)`; the echo the throat orbit's return | 10 |
| T5 | local ring `s r (r + M) / M^2` core traversals | 10 |
| T6 | multiplicity `C(q + 4, 4)`; axis level `C(l - |m0| + 9, 9)` | 8 |
| T7 | filling Z side first; axis polarity `A` | 8, 3 |
| T8 | held where closed, or where `D(s) D(n)^A <= 1 - 2 delta` | 3, 9 |
| T9 | pair share `p_pair = s (+) -(1 - s) = n (+) n / 3`; positron `R -> -R` | 3, 5 |
| T10 | the flip keeps every state; residual `n / 3`; `Lk` conserved; pinch after `3 (4 + a) / (2a)` echoes at half a turn | 5, 8 |
| T11 | pinched throat's spin `a_g = a_p M_p / H_g` (Kerr, with the parent's degrees of turning per unit hold, `360 a_p M_p`); a cohort with `a_g > 1` does not close and stays as medium | 7, 11 |
| T12 | stretching pressure `(D(n)^m0 - 1) / (D(n)^m0 + 1)` | 3, T2 |
| T13 | a pinch encloses the whole cohort released since the last, in every direction | 8, T10 |
| T14 | enclosed matter, from the reach its layer has at the pinch (The state at the pinch); in plane `j` it is pushed out at `J_j / (6 H_j + J_j)` per echo, `H_j` the cohort's hold in the plane and `J_j = J / 5` (none on the axis plane) | 2, 3, 7 |
| T15 | a whole nucleus feels its planes composed, `D_k = D(p_plane)^w_k D(n)^A`, `w_k` its share of turning in the isoclinic planes; held where `D_k <= 1`, else it leaves after `ln 2 / ln D_k` echoes | 3, T14 |

Reading (seed 7, the gate of roadmap item 32, a screen with float
logarithms and exact held sets): the universal core holds 0.4352 of the
founded mass and its first pinched throat 0.4569; what that throat
releases (0.108) and the core's three later cohorts (1.7 x 10^-18) are too
light to close (`a_g > 1`) and stay in the flow as medium. One galactic
core, not a population: under these laws multiplicity does not come from
pinching, and the medium is where it is to be derived.

### The state at the pinch: the fold and its circuits

Derived ([#35](https://github.com/SAIOS-SAMN/xylarium/issues/35)); not yet built.

- **The fold has thickness.** It is not a crease in a sheet but a roll of
  the cohort's matter, layer by layer and on every plane: each layer folds
  at the residual of the deficit at its reach, so the fold shears through its
  thickness; counts are conserved, so what the inner layers lose in span
  the medium carries by continuity, and the roll's interior circulates.
  A fold cannot bend tighter than its matter's grip allows, so its least
  radius is set by its viscosity, not by a crease.
- **The neutral surface.** A layer `l` of the cohort, at reach `r_l` with
  hold `m_l` and grip `nu_l`, is dilated by `r_l / r_0` about the neutral
  reach `r_0`, defined by the layers' strains, each weighted by its
  resistance, composing to the identity:

  ```text
  prod_l (r_l / r_0)^(m_l nu_l) = 1    <=>    sum_l m_l nu_l ln(r_l / r_0) = 0
  ```

  No share is fixed: `r_0` comes from the cohort's layers, held by this relation
  (generally irrational, never as a value). Inside it the layers are
  compressed toward the Z pole, outside it stretched toward the Q pole: the
  dipole across the fold's thickness. A cohort with no grip
  (`sum m_l nu_l = 0`) has no neutral surface: it does not bend but flows.
- **The circuits.** Each quantity a layer carries turns on a circuit of
  that quantity, with the axes and the pivot of that circuit, and is read at the pinch
  plane by plane, never summed:

  | Circuit | Axes | Pivot | Read at the pinch |
  |---|---|---|---|
  | Pole state | each plane's open and held frames | the throat, `t = 1` | the reach on the axis plane, `ln r = (N* - n_k) ln(D(s) D(n)^A)` since the kind's onset `n_k` |
  | Heat | each plane's deficit | the balance of held and emitted | held irradiation net of emission, composed over the echoes since onset |
  | Twist | each plane's weight | the antipode on each plane | the share of antipodal pairs flipped, `1 - (1 - n / 3)^N` |
  | Place | each plane's phase and anomaly | the periapsis | phase and anomaly per plane, and returns made |
  | Pressure | the axis plane, per state | the dipole's Z and Q sides | `D(n)^(A N)` |
  | Pairs | each plane's electron phase | the echo's turn | `D(p_pair)^N` |
  | Fold | each layer's writhe | the meeting of the folds | each layer's strain `r / r_0` about the neutral surface |

  The enclosed matter starts at the reach its layer has when the fold
  closes (this replaces T14's starting reach on the throat orbit).
- **Reading** (seed 7, a screen with float logarithms; the relations are
  exact): the first pinch encloses 605 layers, 0.5648 of the founded mass;
  the neutral surface lies at `ln r_0 = 17.966` on the axis plane, set by
  the gripping layers (their grip-weighted hold 0.0096 of the founded
  mass); 0.1783 of the founded mass lies inside it, compressed, and 0.3865
  outside, stretched; 0.4569 has no grip and flows. Per layer the reach
  runs from `ln r = 0.22` to `255`, the flipped share from 0.065 to 0.395,
  and the phase on the isoclinic planes over the whole turn.

### The bounce at a pinched throat

Derived ([#78](https://github.com/SAIOS-SAMN/xylarium/issues/78)); not yet built.

- **The far face.** A pinched throat bounces: its matter passes to its far
  face, which is both antipoles at once (`bounce::Bounce::of`,
  `circuit::axes`). The axis antipole, the inversion `t -> 1 / t`, carries a
  layer at pole state `t` relative to the parent to `1 / t` in the new
  throat, and turns every state's axis weight `m0 -> -m0`, so the kind's
  axis polarity becomes `-A`; the plane antipole turns its phases half a
  turn on the five planes.
- **The new throat's jet.** From there the layer climbs back to the new
  throat by that throat's own jet, `t -> a t + b` with
  `a = D(s_g) D(n_g)^(-A) / (1 - delta)` and `b = delta / (1 - delta)`,
  `delta = m / (2 H_g + m)` the kind's own deficit at the new throat's
  horizon; it is held there by pressure where `D(s_g) D(n_g)^(-A) <= 1 - 2 delta`,
  and a layer carried far out by the parent arrives deep and climbs late.
  The new throat's pinches enclose what it releases, and a pinch's cohort
  closes where `a = j' / H <= 1`.
- **Reading** (seed 7, a screen with float logarithms): eleven throats in
  a chain, the universal core alone holding more than `10^-6` of the
  founded mass. The inversion spreads the tail, layers carried far out,
  over several pinches of the new throat, but those cohorts are too light
  to close and stay as trace medium; the bulk, at mass-weighted
  `ln r = 19.8`, climbs back in about 16 echoes, within the new throat's
  first pinch interval of about 24, so each tier passes it on whole. The
  ratio of the bulk's climb to the pinch interval, about 0.67, decides it:
  below one a throat's matter goes into a single pinch, so these laws give
  one chain, not a population (#79).

### Radiation: irradiation held, transduced and passed on

Light, heat and their dispersal are one exchange read through the deficit
law, the same at a nucleus, a star and a world:

- **Irradiation.** A body at reach `r` receives the perturbation's ring,
  `1 / (pi (r + 1))` per bell, plus the vessel's drift `bell q(t) / (r + 1)`
  (`stars::ring_per_bell`, `stars::transduced_per_bell`).
- **Held and passed.** The body holds its deficit share of what it
  receives at the body's rim, `delta = M / (R + M)`; the remainder,
  `1 - delta`, is passed: it continues past the body to what lies beyond.
  Received equals held plus passed, so the exchange conserves energy.
- **Heat.** The held share enters the body's hold, on its Z pole. A body
  that does not transduce keeps it as warmth.
- **Light.** A transducing body converts its held share and returns it
  outward. For a star the held share per bell is its transduction,
  `delta / (pi (r + 1))`; its power is the hold the transduction converts per
  ring, `P = m transduced_per_bell(r) / bell` in core masses per ring (`c = 1`,
  `stars::power`), and its light is that power with the vessel's drift,
  `P (1 + pi bell q(t))` (`stars::light`). For a nucleus the radiated part
  is its open frames over its weight, the Q pole's share of its frames
  (`formation::emission`).
- **The inversion.** Emission is irradiation turned outward: what a body
  held becomes what it gives to the bodies beyond it, so at each exchange
  the held share of one body is the received share of the next. The
  inversion `delta -> 1 - delta`, fixed at the throat's `delta = 1/2`, and
  the pole axis's `X(t) -> X(1 / t)`, which swaps the Z (held) and Q (open)
  poles, are the two readings of the same exchange.
- **Dispersal.** The emitted light spreads on the emitter's sphere and is
  received at reach `r` through the receiver's deficit there; about the
  universal state the waveform falls as `1 / (r + 1)`
  (`Substrate::amplitude`).
- **At a world (next).** A world receives its star's light at its reach and
  holds its deficit share of it; that held share is its heat, from which
  its temperature follows, and what it re-emits is that heat turned
  outward. This replaces the Earth calibrations of the climate (278 K,
  340 W/m^2, 288 K and `greenhouse_tau`) in layer C of roadmap milestone 3.

## 5. The galaxy

- Structure (`galaxy::Structure`): the home galaxy is made of the universal
  core's founded matter, so its structure is the founding's beads re-read in
  the galaxy tier's unit. Its rings are the beads, with their reaches and
  spans in galaxy rings (60^5 r_g) and their states as their holding. The
  core's reach is one galaxy ring, where `1 / (r + 1)` is one half
  (`galaxy::core_reach`).
  The nucleus is the innermost ring; the reference ring is the ring within
  which half the holding lies (`galaxy::r0`); crowding is a ring's state per
  span over the reference ring's state per span (`galaxy::density`).
- Mass, in core masses (the universal core is 1): the stellar mass is the
  lit share, `stellar_mass = Lighting::lit` (`galaxy::stellar_mass`). The
  mass inside `r` is `1 + rings within r / all rings`
  (`galaxy::inside_mass`), which the turning law reads (`turning::turning`,
  `omega = delta(<r) bell / r` radians per ring, `r` in r_g, `bell` in
  r_g / c).
- Lighting (`formation::cooling`, `formation::collapse`,
  `formation::lighting`): each bead is a piece of gas. Its holding per
  galaxy ring is `lambda = (state / unit) / W`; its agitation is
  `(p - delta) / p` with `delta = h / (R + h)` and `h = 1 + held share`; its
  emission is `e = sum s_z open_z / hold_z`, and its magnetism is
  `mu = lambda e`. Outside the core's reach the gas is a circle (`n = 1`);
  within it, a ball (`n = 2`). An infall of extent `w` gathers
  `m(w) = (lambda + mu) W share_n(theta)`, with `theta = w / (w + R)` of a
  half turn (`xyl-core measure::share_turns`), and grips it as
  `m / (w + m)`. The pulse heats the piece by `ring x swing x grip(W) / shed`
  (capped at 1), where `ring = 1 / (pi (R + 1))`, `swing = 1 + A_in` at the
  first echo's mean, and `shed = e + ln 2 / k_v`. On a ladder of halvings of
  `W` the spare is `grip - H w / (R (R + w)) - heat`. The piece halves its
  agitation until the agitation is under the best spare, at
  `floor(n / P_e) + floor(n / k_v)` halvings by ring `n`
  (`P_e = ceil(1 / e)`, `k_v` the vacuum's halving), and fragments at the
  largest extent that holds. The piece lights where its agitation at
  ignition is under the throat's push. The first stars are the ring count of
  the first lit piece; the lit share is the star efficiency.
- Reach unit: a star's reach about the core (`Star::r`, `Orbit::radius`) is
  measured in galaxy rings, the same unit in which its piece was lit; kpc
  appear only at display and at the marked boundaries of the layers not yet
  converted (`galaxy::kpc_of_rings`).
- Placement (`galaxy::star`): sample stars stand for equal shares of the
  stellar mass and are placed along the rings' cumulative holding by
  irrational pivots (golden, silver, plastic and `sqrt 3 - 1` turns), which
  is the repulsion law. Disc stars orbit in the plane; core and nucleus
  stars orbit on tilted planes. The orbit's rate is the turning law at the
  star's reach, in radians per ring. The binary share is a declared ruler.
- Star masses (`galaxy::piece_star_mass`, `galaxy::imf`): a piece's star
  mass is the share of its infall the star tier's cut holds, `m_p = s c`
  core masses, with `s` the cut's held share (`disc::star_hold`) and `c` the
  infall's holding, a share of the galaxy's rings, which hold one core mass.
  It is the most one star of the piece holds. The star's draw `u` spreads the
  piece's stars over shares `x` of it by the deficit form
  `F(x) = x / (x + b)`, with the break `b = d / (1 + d)` the deficit share of
  the ring's crowding: `x = b u / (1 + b - u)`, `m = x m_p`. A crowded ring's
  stars are heavier. A homologue's stars are cut from the same pieces; the
  stellar mass a sample star stands for is the population's mass in core
  masses over the sample, scaled by the hole ratio.
- Vertical orbit (`galaxy::vertical`): a disc star swings through the plane at
  the frequency `nu = omega 2d / (1 + d)`, the turning rate held by the crowding
  share (`galaxy::vertical_frequency`). The crowded layer's half-thickness is
  the deficit share of the ring's mass against the mass inside it, taken of the
  reach, `h = r m / (M(<r) + m)` (`galaxy::layer`). The heating share is `s = t
  / (1 + t)`, with `t` the star's age over its ring's cooling time (the rings
  until the ring's gas first lights). The speed scale is `sigma = h nu s 2d / (1
  + d)`, the speed an exponential draw `vz = -sigma ln(1 - u)`, the amplitude
  `vz / nu`, and the exposure the share of the swing inside `|z| < h`.
- Births (`galaxy::birth`, `chem::Chemistry::birth_along`): a star's
  ring first lights at `max(cools_in, 1)` rings, and no star stands in a
  ring that never lights. The star's birth is then placed on the ring's star
  formation history: it is the time by which the share `u` (the star's draw)
  of the stars that the ring forms between first light and the horizon
  (30 Gyr) have formed. The ring's `sfr` is integrated term by term,
  `c p! / k^(p+1) (1 - e^(-kt) sum_(j<=p) (kt)^j / j!)`
  (`c t^(p+1) / (p + 1)` where `k = 0`), and the time is found by 48
  halvings.
- Chemistry (`chem::Model`): each disc ring (30 rings of 0.5 kpc) is a sum of
  closed forms `c t^m e^(-lambda t)`, with `t` in rings; the timescales measured
  in Gyr are converted to rings where they are declared. Gas falls in on the
  timescale of the ring's first light toward the structure's crowding profile,
  and stars form at `efficiency x Omega / 2 pi`. Prompt and slow returns, the
  fountain's landing map and the core's jets each feed every ring once, and each
  nucleosynthetic channel makes its elements after its delay. Yields are scaled
  per channel by `chem::Calibration` against the gas at the reference ring and
  the calibration's reference birth gas (`Rulers::ism_today_dex`,
  `Rulers::reference_metallicity`, `Rulers::today`, `Rulers::v_flat`,
  `Rulers::sigma_r0`); hydrogen and helium come from the founding's counts
  (`chem::Light`). Status: partial. Next: the yields and ring scales are to be
  derived from the founding (roadmap, Rules from the founding and the galactic
  cores, chemistry from the census).

## 6. Stars and systems

- The transduction (`stars`): a star at reach `r` absorbs the ring that the bell
  brings there, `1 / (pi (r + 1))` per bell, plus the vessel's drift `bell q(t)
  / (r + 1)`, and holds its compactness `delta` of that absorption: its deficit
  at its rim, `delta = m / (R 60 + m)` (`disc::rim_share`), with `m` the star's
  mass in core masses and `R` the rim of the held core its cloud's cut gives, in
  system-tier units (`galaxy::piece_rim`, `galaxy::Star::compactness`). The rim
  comes from the atoms: the cut carries each kind to a spin from its open frames
  and its mass per nucleus and holds it where its deficit meets the push
  (`formation::hold_at_heat`). The cut is read in three states. At collapse the
  dark cut (no over-pressure) gives the held share `s0` and rim `R0`, so the
  collapsed star's compactness is `delta0 = s0 c / (R0 60 + s0 c)`. At ignition
  the star holds `delta0` of the turning it receives per echo; that is its
  transduction, which adds the over-pressure `p0 = 1 + transduced`. Settled, the
  settled cut gives the star's held share and rim (`disc::star_hold`). Every
  star of a cloud shares the rim, so a heavier star is more compact, transduces
  a larger share and lives fewer rings. Its power is the hold that transduction
  transduces per ring, `P = m transduced_per_bell(r) / bell` in core masses per
  ring (`c = 1`; `stars::power`), and its light that power times the drift
  factor `1 + pi bell q(t)` (`stars::light`). The system and world layers still
  read the luminosity in the prototype's display unit through the boundary
  `stars::suns_of_power`, removed when layer C is converted. Each step of its
  transduction sequence (Polarity, fusion and fission; `galaxy::Star::sequence`)
  lasts its transduced share over the transduction: its main-sequence life is
  the first step's (`stars::life`), the later steps end one after another
  (`stars::ends`), and its remnant keeps the hold no step transduced
  (`stars::remnant`). The radiation's swing within a bell is `(pi / 2)
  |cos(2 pi t / echo)|` (`stars::flicker`).
- The star's cut (`disc::star_hold`): the infall of the star's lit piece
  collapses carrying the galaxy's shear across its extent,
  `h w / (R (R + w))`, as its turning. It is heated by the pulse's turning
  per echo at the star and pushed by the star's transduction per echo as an
  over-pressure `p0 = 1 + transduced`. Each kind is carried to
  `l = p p0 (e / e_mean)(w_mean / w) + add` and is held where
  `h / (l / h + h) >= p` on the held hold (`formation::hold_at_heat`). The
  disc's agitation is `turning x swing x grip / shed`, with
  `grip = (1 - star) / (2 - star)` and `shed = e_rest + ln 2 / k_v`.
- The star's manifold (`galaxy::disc_frame`): the star's gathering extent is
  its mass in holding units (`mass / stellar_mass`) over the infall's
  holding per ring, at most the infall. Its manifold reach is the arc of its
  circle about the core that its pull holds,
  `manifold::reach(r, m / M(<r)) = 2 pi r m / (M + m)`. The star's share is
  the gathering extent over the reach, at most 1.
- Worlds on the manifold (`disc::manifold_worlds`): in the manifold's units
  the star holds the inner share `s`. The remainder is read over 360 degrees
  out to the reach, each degree adding `(1 - a) (1 - s) / 360` of matter. A
  world forms where the matter gathered since the previous world holds its
  region against the core's twist, `m / (s + m) >= tide x`, where `x` is the
  degree's outer edge; the world's reach is the mean reach of the gathered
  matter. No random draw and no preset partition enter. Matter that never
  holds a region is belt; worlds plus belt sum to `1 - s`.
- The system (`systems::system`): the outer edge of the worlds is the
  manifold reach in AU. The infall's solids are the star's mass over its
  share times the gas's metal mass fraction (relative metals times
  `Rulers::reference_metallicity`). A region inside the rock line (the
  lowest condensation temperature of Fe, Si and Mg) holds no solids. Each
  region's mass is its holding times the solids, enriched beyond the water
  line; its phase is its index times the golden turn. A core of ten world
  masses or more acquires an envelope by the declared gathering law.
- Settling (`systems::settle_about`): about an anchor of mass `M` in core
  units, each body turns at `omega(A) = [M / (A + M)] / A`. Neighbours meet
  `N = age |dw|` times, and each meeting kicks by
  `eps = [m / (r + m)] / r / |dw|`. The kicks repeat on the continued
  fraction period `q` of the phase step and cancel to
  `c = |1 - 2 frac(q psi_0)|` per period, so the greatest swing is
  `eps (floor(N / q) c + N mod q)`. A pair merges at the first age at which
  its swing spans its gap, found by 48 halvings out to the horizon. Merges
  before the settled age (3 Gyr) combine the pair, mass-weighted; later
  meetings are recorded on the lighter body as its merge age
  (`World::merged_at`). A body's starting eccentricity is its swing, and its
  tilt is `swing / (1 + swing)` of a quarter turn (`bodies::orbits`).
- Moons (`systems::moons`): each world anchors a manifold of reach
  `2 pi a m / (M + m)`, measured outward from five of the world's radii. The
  manifold's budget is the disc's agitation times the world's width on the
  star's manifold times the infall's solids, at most what the belts still
  hold; the world keeps `m / (m + budget)`. Moons gather by the same degree
  law against the star's tide at the world, `s / (x + s)`, and then settle
  about the world.
- Belts (`systems::belts`): the ungathered remainder, less the moons, lies
  beyond the outermost world out to the manifold's reach. Giants fling a
  comet cloud by a declared share.
- Exponentials below the working resolution are enclosed by `[0, 2^-bits]`
  rather than computed (`systems::exp`).
- Angles between bodies: `manifold::turn_between` and `chain::Place::arc`
  read the turn between two directions as the arccosine of their dot
  product. Next: the turn is to be read from the versor between the two
  bodies' frames, which requires the inverse of `Rotor` and the versor's
  angle (roadmap, Worlds on the manifold chain, cross-ties).
- Next: the manifold's disc-mass scale, the share of the manifold's matter
  that the star keeps, is to be revised. With the current scale, the
  settling merges the worlds of a metal-rich star into one heavy world; the
  revised scale is to let them settle as several bodies (roadmap, Worlds on
  the manifold chain).
- World states (`worlds::build`): air, seas, ices, surface temperature and
  pressure, interior heat and magnetic field are built at an age from the
  setting, in world masses and radii, AU and kelvin through `package::Si`.
  The disc's formation temperature is `278 L / a` (`systems::Spec::t_form`).
  Worlds in play are read in degrees of their star's life, the star's
  lifetime over 360, floored at one bell (`worlds::star_degree_gyr`).

### Polarity, fusion and fission

A star transduces by the polarity of the collective it holds: the inverse of
magnetism between its bonding nucleons over the quantum states its
irradiation opens. Every quantity is read from the atoms:

- **Held and open frames under irradiation.** A kind `z` irradiated at heat
  `q` (the turning the star receives per echo, in hydrogen units) keeps
  `u_z(q)` open frames (`atoms::unpaired_hot`), its magnetism, the Q pole;
  its held frames are `w_z(q) = 3Z - u_z(q)`, its bonding, the Z pole
  (`compound::hold`).
- **The collective's pole point.** Over the held mixture, with `n_z` the
  count of nuclei of each kind (its share of the hold over `w_z`),
  `t = sum n_z u_z(q) / sum n_z w_z(q)` (`circuit::pole_point`). The star's
  polarity is the inversion `1 / t`: bonding over irradiated states.
- **Irradiation carries the pole point.** A held kind's open share is at
  most `Z` of its `3Z` frames, so `t <= 1/2` at ignition: every collective
  starts on the Z side. Each echo carries the pole point outward by the
  residual of its loop through the continuum and the vacuum, `rho_net`
  (next subsection), and after `n` echoes since ignition
  `t(n) = t0 rho_net^n`.
- **Direction.** While `t < 1` the collective lies on the Z side of the
  throat and bonding combines nuclei: fusion. The pole point reaches the
  throat, `t = 1`, after `n* = ln(1 / t0) / ln rho_s` echoes, in closed
  form; there the collective holds as much as it passes and takes no step.
  Past it, `t > 1`, the collective lies on the Q side and irradiation
  splits nuclei: fission. Both `n*` and the transduction's duration scale as
  `1 / delta`, so a star's fusion era and its fuel end on one timescale.
- **A fusion step.** `k` nuclei of kind `z` become one nucleus of the kind `z'`
  whose nucleon count is `A_z' = k A_z` (`A` the element table's mass number),
  for the smallest `k >= 2` for which such a kind exists and the step releases
  hold. The share transduced is the hold released, `1 - w_z'(q) / (k w_z(q))`.
  For example, for a mixture of hydrogen alone the rule finds four hydrogens to
  the kind of four nucleons; the products found depend on the element table and
  the mixture, not on a list.
- **A fission step.** One nucleus of kind `z` becomes two nuclei of kinds
  `z1` and `z2` with `A_z1 + A_z2 = A_z`, the split nearest to equal halves
  that releases hold. The share transduced is `1 - (w_z1(q) + w_z2(q)) /
  w_z(q)`.
- **The sequence.** No kind, product or order is set: the sequence emerges from
  the star's held mixture (`disc::StarHold::held`, the dark cut's held kinds).
  At each stage the held kind whose step has the greatest release steps (ties by
  the lesser nucleon count), transducing its release times its share of the
  hold; its products join the held mixture, and the next stage is read again
  from the updated mixture, until no held kind releases under the current
  direction. Fusion stages come first; after the throat the fission stages
  follow, read the same way, a split's products sharing the passed hold in
  proportion to their held frames. Each step's duration is its transduced share
  over the star's transduction per bell (`stars::transduced_per_bell`); the
  first step is the main sequence, later steps the giant phases, and the hold no
  step transduced is the remnant (`stars::Sequence::of`, `stars::fusion_step`,
  `stars::fission_step`, `stars::pole_point`, `circuit::echoes_to_throat`).
  Status: built.
- **A held body that does not ignite.** Where no step of the held mixture
  releases hold under its polarity, the body holds its matter and transduces
  nothing.

Open, read from the emergent sequences (cold hydrogen gives four
hydrogens to the kind of four nucleons, three of those to the kind of
twelve, two to the kind of twenty-four, then eight to `Z 77` and ten to
`Z 752`; 296 kinds of the table take no step in either direction):

- The held frames per nucleon fall with the nucleon count across the whole
  table, so every fusion step releases hold and the sequence runs to
  superheavy kinds; the atoms give no peak at which fusion stops releasing.
- Where the table holds no kind at `2 A_z`, the rule's least `k` can be
  large (eight or ten nuclei in one step), while the atoms' bonding on
  S^11 is pairwise (`fibres`); a pairwise rule alone would stop the
  hydrogen step, since the table holds one nucleon count per kind.

Grain: the steps move whole nuclei and conserve the nucleon count, so the
founding's grain is kept (`circuit::grain::nuclei_of`).

### The transduction's path: the continuum and the vacuum

The collective's state moves on the founding's loop, the jetstream, not
along one axis (`circuit::jetstream`, the law `Substrate::loop_flux` reads
for each firmament kind). Each echo carries the state out through the
continuum and back through the vacuum:

- **Continuum (outflow).** The filament dilates the state by the push the
  holder holds, `rho = (1 + p) / (1 - p)` (`circuit::dilation`): `p = 1/2`
  for a firmament kind at the throat, the compactness `delta` for a star's
  collective. It moves `(rho - 1) u w / (w + rho u)` frames from the Z pole
  to the Q pole.
- **Vacuum (inflow).** The return is through the vacuum's share at the
  holder's reach about the universal state, `p_v = H / (R + H)`
  (`xyl_core::shares::deficit_dy`), swung by the inbound echo; it moves `p_v w`
  frames from Z to Q.
- **Electron phase.** The electrons' phase state `R = kappa (1 - 2 frac(p))`
  moves `R u` frames from Q to Z (`circuit::electron_phase`); over a whole
  echo its mean is zero, so the echo's mean motion is the continuum's and
  the vacuum's.
- **Why the loop does not close.** Composed in one medium, an outbound
  dilation and its return through the throat's antipole cancel: dilation
  about the origin commutes with rotation in the five planes, a point on
  the pole axis is fixed by that rotation, and conjugation by the antipole
  inverts the dilation, so a full echo would be the rotation alone. The
  continuum and the vacuum are different media with different pushes, so
  the return does not undo the outflow. The residual dilation per echo,
  `rho_net`, is the torsion the echo leaves on the holder: the product of
  the channels' dilations.
- **The star's legs.** Both legs carry the push the star holds, its
  compactness `delta`: the continuum dilates by `(1 + delta) / (1 - delta)`
  and the vacuum's share at the star's rim about the star's mass is `delta`,
  as a firmament kind's vacuum share is read at its reach about the
  kind's hold. One echo is then the map `t -> a t + b` with
  `a = rho / (1 - delta)` and `b = delta / (1 - delta)` (`circuit::jet_map`).
- **Transduction.** Iterated, `t_n = t* + (t0 - t*) a^n` with the fixed
  point `t* = b / (1 - a)`, so the collective reaches the throat after
  `ln((1 - t*) / (t0 - t*)) / ln a` echoes, in closed form
  (`circuit::throat_crossing`); a collective with nothing open has no pole
  state and, as a sealed kind, no crossing. The star's
  timeline runs the fusion steps until that crossing, cutting a step still
  running there, and the fission steps from it (`stars::timeline`); the
  remnant is the hold the steps that ran did not transduce
  (`stars::remnant`). Status: built, for the firmament's kinds
  and for stars alike.

### Star types

Star types are not set. A star's class is the state its formulas give at a time
(`galaxy::Star::class`): its transduction step at its age
(`field::transduction`: unlit, a step of its sequence, or remnant), its
compactness `delta = m / (R 60 + m)` (`galaxy::Star::compactness`), its mass as
a share of its cloud's star mass, and its life in rings against the universe's
age (`stars::life`). Two stars are of one class where these agree. The known
types below are expectations, not limits: a known name is attached where its
condition holds (`galaxy::Class::name`), and every other combination is reported
by its quantities, so the universe can produce classes no observation has named.

`xylarium observe star-types [N] [R]` counts the classes over each
galaxy's first `N` sample stars at the universe's age, then probes the home
galaxy's main-sequence stars at `R` of its rim (default 8/15) and prints,
beside them, the Earth's star's age, life and compactness as reference
values for comparison only. The probe is one reading among others; no law
reads the reference values and none is tuned to them.

| Type | Condition in the model | Status |
|---|---|---|
| Collapsing cloud (protostar) | A lit piece's infall held by the dark cut, before ignition: held share `s0`, rim `R0`, compactness `delta0` (`disc::star_hold`). | Partial: computed as the collapse state of each cut; not yet a stage of a star's timeline. |
| Main-sequence star | The first step of the star's sequence (Polarity, fusion and fission): age below that step's transduced share over the transduction. | Built: the first step of the sequence its held mixture gives. |
| Mass ordering of the main sequence (the classes O to M) | Within a cloud every star shares the rim, so a heavier star is more compact, transduces a larger share and lives fewer rings. | Built for mass, compactness and life. The surface temperature that the classes are named by needs a world-independent temperature of a star from its held share (layer C, radiation). |
| Long-lived low-mass star (red dwarf) | A star whose life in rings exceeds the universe's current count, so it has not left the main sequence since any star was born. | Built as a reading of `stars::life` against the universe's count. |
| Held body that does not ignite (brown dwarf) | No step of the held mixture releases hold under its polarity (Polarity, fusion and fission). | Built: `field::Transduction::Unlit` where the sequence has no step. |
| Giant | The later steps of the star's sequence: age past the first step's end and before the last step's. | Built: the later fusion steps of the sequence. |
| Fission star | A star whose held collective lies on the Q side of the throat (`t > 1`), so its steps split nuclei. No observed type is known to match; the model allows it. | Built: a fission step of the sequence, after the throat; the step's start is read from the cumulative step ends, not yet from `n*`. |
| Supergiant | A giant of a star in the upper share of its cloud's masses. | Next: needs a derived boundary in the cloud's mass shares. |
| Remnant | Every step of the sequence has ended: the star keeps the hold no step transduced (`stars::remnant`) and becomes a hole in the pulse network (`pulse::nodes`); a held body that does not ignite forms none. | Built. |
| Compact remnant (white dwarf), denser remnant (neutron star) | A remnant whose held core's rim, from the remnant's cut of its remaining mixture, gives compactness below one half; the two separated by where the remnant's held kinds lose their open frames. | Next: needs that cut of the remnant's mixture. |
| Remnant at the throat (black hole) | A remnant whose compactness reaches `delta = 1/2`, the throat's equator: it holds as much as it passes and becomes a core with a throat. | Next: follows from the remnant's rim; whether such a remnant is an echo surface is under review with Netstratum. |
| Binary companion | A second star drawn with a share of the primary's mass. | Placeholder: the share `0.1 + 0.9 u` is imposed; it is to come from the cloud's split. |
| Variable star | The light's swing within a bell, `(pi / 2) |cos(2 pi t / echo)|` (`stars::flicker`). | Partial: a reading of the light only. |

## 7. Life and minds

- Ladder (`life::ladder`): the bare chemistry of a medium miscopies with
  `eps0 = exp(-d E_weak / kT)` (`life::fidelity`). Rung `s` holds
  `b / eps^(s+1)` bits, where `eps` includes the decay a generation suffers,
  capped at a mind's blueprint `I_MIND`; there are at most 12 rungs. Each
  rung's mean wait is the bits to write over the bits that selection writes
  per generation (`life::bits_per_generation`), in the population that the
  medium's power feeds. The solvent is the world's liquid or supercritical
  volatile of any species, a silicate melt or a giant's deep plasma
  (`worlds::Solvent`); monomers are built from the solvent's molecules.
- Climb (`life::climb`): each rung takes its mean wait. Impacts at rate `mu`
  (the system's debris swept once per orbit, `life::strikes_per_gyr`) make a
  rung of mean `m` complete in `(e^(mu m) - 1) / mu`; a sterilising pulse
  falls at `1 / hazard`. No random draw is used.
- Forecast (`minds::schedule`): for every world that an observer's sight
  opens (`minds::Sight`), a mind is scheduled at the world's last rung if
  the world at that epoch feeds a viable people of 500 bodies
  (`life::viable`). The first age read is one degree of the star's life;
  the horizon is 30 Gyr.
- Ancients (`minds::ancients`) are minds born within a galactic core's reach
  that arise within one core assembly time (the nucleus's first light,
  `minds::ancient_window`) of that core's first core-born mind.
