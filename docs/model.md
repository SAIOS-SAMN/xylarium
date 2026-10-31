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

Angles are held as exact turns where the rate is rational and are displayed
in degrees of 360. Phases about a throat are rounded to the coordinate grid
of 60^-6 of a turn (`circuit::turn`); drawn values are rounded to the 2^-40
grid.

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

### The cascade, shells and beads (`formation::founding_shells`, `circuit::Substrate::found`)

- The founding's mix is the share of holding of every founded kind
  (`formation::kinds`), converted to mass at `Z + Q` nucleons per nucleus
  (`circuit::nucleus_weights`).
- In the equitable cascade, each kind in Z order takes the share
  `s_Z = 1/2 held / (held + 1 + held)` of the unlocked face, where `held` is
  the mass share before it. `K = sum s_Z`, `H = 1 + held_final`, and the
  final share is `s_final = 1/2 held / (held + H)`. Kinds are stacked by the
  mass profile, heaviest first, and the stack is cut at the profile's local
  minima. Each kind is thrown to `R_Z = (K + s_i) / w_Z / H`, where `s_i` is
  the sum of its stack's shares and `w_Z` its nucleon mass, and spreads over
  `R (1 +/- s_final)`.
- In the filter, every lighter shell passes out through each heavier shell
  whose spread lies between the throat and the lighter shell's reach. Each
  passage grips `g = min(s D, T) / T`, where `s` is the two kinds' tightness
  ratio, `D = M / (M + 1 - contact)` and `T` the contrast of their deficits.
  The heavier shell gives `contact g m_i / (m_i + m_j)` of its matter, on
  the nucleus grid. Takes are scaled down where more is asked of a shell
  than it holds, and a shell's reach becomes the holding-weighted mean of
  its parts' release reaches. A kind is held by the throat where
  `R (1 + s_final) <= 1`.
- The shells that are pushed out link into runs where their spreads
  overlap. A run's superposed hold (each kind's mass over the deficit
  contrast of its spread) peaks at beads and is divided at its local minima;
  each kind's matter goes to the bead whose span contains the kind's reach.
  A bead's state is the mass of its recipe; its reach and phase are those of
  its most abundant kind (the phase is the torsion share turned Z times);
  its hold inside is `1 + 2 sum(inner beads' shares)`.
- The rim is the outer edge of the outermost shell, `max R (1 + s_final)`
  over all shells.

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
  (`circuit::vacuum_share`); read at the throat, it is the throat's push,
  `p = delta(H, H) = 1/2` (`circuit::throat_push`). A band swinging over
  `r (1 +/- p)` scales the pole coordinate by `rho = (1 + p) / (1 - p) = 3`
  per echo (`circuit::echo_dilation`).
- Reverberation (`Substrate::reverberation`) takes the firmament kinds
  in order of spin. For each kind the hold is
  `h = 1 + (firmament mass nearer) / unit`, the reach `r = spin / h`, the
  deficit `delta = h / (r + h)`, the share of its band past the throat
  `beyond = clamp((r (1 + delta) - h) / (2 delta r), 0, 1)`, and the leaving
  rate `beyond / bell` (per ring).
- In the jet, a kind's pole state sits at `t = open / hold` on the filament
  and its jetstream carries it out through the continuum at the throat's
  push and back through the vacuum at the kind's vacuum share `delta`
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
  of a kind that the jet throws is its pull `h / (r + h)` (`h` its state
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
  (`circuit::vacuum_share`), swung by the inbound echo; it moves `p_v w`
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
