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
  (`systems`); the stars' burn (`stars`); and life and minds (`life`,
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
- The bell (`circuit::anchor`, `Anchor::bell_gyr`) is the round-trip time
  of light to the home galaxy's rim, `bell = 2 rim_galaxy / c`, where
  `rim_galaxy` is the founding's rim read in the galaxy tier's unit. The
  bell is the unit of time.
- A ring of time is one bell; counts of time (cooling, first light, a
  star's burn and life) are whole numbers of rings. An echo is two bells,
  one ring for each face (near and far) of the paired throat
  (`circuit::echo_gyr`). A one-way crossing to reach `r` takes
  `(r / rim) bell / 2`, a fraction of a ring.
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
- The SI bridge is `package.rs`. `package::hole_mass` (4.3e6 solar masses)
  is the only conversion from core units to SI: it fixes `r_g` in metres and
  thereby every reading displayed in kpc, Gyr, AU, solar masses or world
  masses. The SI constants used at seams are in `package::Si`. Laws stated
  in core units do not depend on the bridge; the chemistry and the world
  states, which read SI-calibrated constants, do depend on it. Next: the
  core's mass is to be supplied by the package (roadmap, Founding from the
  package manifest).

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

### The Big Bang (`bang.rs`)

- The parent's Kerr spin is the only value drawn from the seed: a uniform
  rational strictly inside `(0, 1)` on the 2^-40 grid (`BigBang::of`,
  `clamp_spin`). The founding depends on the spin alone, so the substrate is
  founded first and the parent bounce is read in the substrate's enclosure:
  `Bounce::of(1, spin, radius_rg, 0)`.
- The torsion share (`torsion_share`) is the share that the outbound jet
  carries through the throat. At the throat reach `H = 2M` the deficit of
  one face against the other is `delta(H, H) = 1/2`; the spin `J` is a
  further share that the throat does not hold, so the held face is
  `H / (2H + J)` and the jet carries `a = (H + J) / (2H + J)`. The share is
  1/2 at zero spin and 3/5 at extremal spin. The inverse is
  `J = H (2a - 1) / (1 - a)` (`spin_of_share`).
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

- In the capture chain (`elements::founded`), nuclei are built one capture
  at a time from hydrogen, `count_Z = floor(count_(Z-1) a h_Z)`, where
  `h_Z = (3Z - u_Z) / 3Z` is the closed share of the new nucleus and `u_Z`
  is its number of unpaired windings (`atoms::unpaired`). Counts are whole
  nuclei out of `2^b` hydrogen nuclei, where `b = founding_bits(a)` is the
  smallest number of bits that carries one whole nucleus of the vessel's
  sealing kind through the floored chain. A kind whose count floors to zero
  is absent from the founding. Next: the shares of the light kinds are to be
  read from the same founding as the firmament (roadmap, Founding from the
  package manifest).
- The table runs from Z = 1 to `TOP_Z` = 1700. `data/elements.tsv` holds Z 1
  to 99 as measured rows and Z 100 to 150 as predicted rows, each predicted
  from its homologue by the periodic law. Z 151 to 1700 are generated with
  systematic names, the mass number of the valley of stability
  (`elements::valley_a`) and uranium as the homologue for the material
  columns. The founding replaces the abundance column with the chain's
  counts; mass numbers, phase temperatures, densities and geochemical
  classes remain catalogue values. Next: those columns are to be derived
  from the atom (roadmap, Founding from the package manifest, chemistry from
  the census).
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
- Reverberation (`Substrate::reverberation_with`) takes the firmament kinds
  in order of spin. For each kind the hold is
  `h = 1 + (firmament mass nearer) / unit`, the reach `r = spin / h`, the
  deficit `delta = h / (r + h)`, the share of its band past the throat
  `beyond = clamp((r (1 + delta) - h) / (2 delta r), 0, 1)`, and the leaving
  rate `beyond / bell`.
- In the jet, a kind's pole state sits at `t = open / hold` on the filament
  and reaches the throat after `ln(1 / t) / ln rho` echoes, which is its
  onset (`circuit::onset_echoes`); a sealed kind has no onset. The kind then
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
  state is its mass in the universal core's units, so the mass of its core
  reads `hole_suns = hole_mass x state`. A bead is a galaxy where the lit
  share of that mass is at least one solar mass,
  `lit x hole_mass x state >= 1`. Each galaxy circulates about the throat at
  `4 delta` turns per echo, with `delta = inside / (reach + inside)`. Its
  plane is set by the repulsion pivots one tier up (the node by the golden
  turn, the tilt by the plastic turn's share of the S^360 measure), and its
  place at `t` is its plane's rotor applied to its phase
  (`Shape::place_at`). The stars of another galaxy are the home galaxy's
  stars read through that galaxy's length scale `2 m / (1 + m)`, where `m`
  is its hole mass ratio (`Shape::scale`, placeholder).

## 5. The galaxy

- Structure (`galaxy::Structure`): the home galaxy is made of the universal
  core's founded matter, so its structure is the founding's beads re-read in
  the galaxy tier's unit. Its rings are the beads, with their reaches and
  spans in galaxy units and their states as their holding. The core's reach
  is one galaxy unit, where `1 / (r + 1)` is one half (`galaxy::core_kpc`).
  The nucleus is the innermost ring; the reference ring is the ring within
  which half the holding lies (`galaxy::r0`); crowding is a ring's state per
  span over the reference ring's state per span (`galaxy::density`).
- Mass: `stellar_mass = Lighting::lit x hole_mass` (`galaxy::stellar_mass`).
  The mass inside `r` is `hole_mass (1 + rings within r / all rings)`
  (`galaxy::inside_mass`), which the turning law reads (`turning.rs`,
  `omega = c delta(<r) / r`).
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
- Reach unit: a star's reach about the core is measured in galaxy rings,
  `r_kpc / galaxy_kpc` (`stars::reach_rings`, `Anchor::galaxy_kpc`), the
  same unit in which its piece was lit.
- Placement (`galaxy::star`): sample stars stand for equal shares of the
  stellar mass and are placed along the rings' cumulative holding by
  irrational pivots (golden, silver, plastic and `sqrt 3 - 1` turns), which
  is the repulsion law. Disc stars orbit in the plane; core and nucleus
  stars orbit on tilted planes. The orbit's rate is the turning law at the
  star's reach. The initial mass function (`galaxy::imf`), the binary share
  and the scales of the vertical orbit (`galaxy::vertical`) are declared
  rulers. Next: these are to be derived from the piece's infall (roadmap,
  Declared rulers).
- Births (`galaxy::birth_gyr`, `chem::Chemistry::birth_along`): a star's
  ring first lights at `max(cools_in, 1)` bells, and no star stands in a
  ring that never lights. The star's birth is then placed on the ring's star
  formation history: it is the time by which the share `u` (the star's draw)
  of the stars that the ring forms between first light and the horizon
  (30 Gyr) have formed. The ring's `sfr` is integrated term by term,
  `c p! / k^(p+1) (1 - e^(-kt) sum_(j<=p) (kt)^j / j!)`
  (`c t^(p+1) / (p + 1)` where `k = 0`), and the time is found by 48
  halvings.
- Chemistry (`chem::Model`): each disc ring (30 rings of 0.5 kpc) is a sum
  of closed forms `c t^m e^(-lambda t)`. Gas falls in on the timescale of
  the ring's first light toward the structure's crowding profile, and stars
  form at `efficiency x Omega / 2 pi`. Prompt and slow returns, the
  fountain's landing map and the core's jets each feed every ring once, and
  each nucleosynthetic channel makes its elements after its delay. Yields
  are scaled per channel by `chem::Calibration` against the gas at the
  reference ring and the solar-birth reference (`Rulers::ism_today_dex`,
  `Rulers::reference_metallicity`, `Rulers::today_gyr`, `Rulers::v_flat`,
  `Rulers::sigma_r0`); hydrogen and helium come from the founding's counts
  (`chem::Light`). Status: partial. Next: the yields and ring scales are to
  be derived from the founding (roadmap, Founding from the package manifest,
  chemistry from the census).

## 6. Stars and systems

- The burn (`stars.rs`): a star at reach `r` absorbs the ring that the bell
  brings there, `1 / (pi (r + 1))` per bell, plus the vessel's drift
  `bell q(t) / (r + 1)`, and takes its deficit share of that absorption,
  `delta_star`, read on the solar rim (`stars::delta_star`,
  `package::Si::star_radius_m`). Its light is that burn transduced,
  `L = M / (r + 1) (1 + pi bell q(t))` in solar units
  (`systems::luminosity`, `stars::light`). Its main-sequence life is
  `(1 - hold(He) / (4 hold(H))) / burn` bells, one quarter over the burn
  (`stars::life`); its giant phase ends at that life times
  `transduced_He / transduced_H` (`stars::ends`); its remnant keeps
  `1 - transduced_He` of its hold (`stars::remnant_share`). The radiation's
  swing within a bell is `(pi / 2) |cos(2 pi t / echo)|` (`stars::flicker`).
- The star's cut (`disc::star_cut`): the infall of the star's lit piece
  collapses carrying the galaxy's shear across its extent,
  `h w / (R (R + w))`, as its turning. It is heated by the pulse's turning
  per echo at the star and pushed by the star's burn per echo as an
  over-pressure `p0 = 1 + burn`. Each kind is carried to
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
