# The physical model

This document explains Xylarium's physics: the master frame, the founding,
and how each tier follows from the one before it. Every formula, with its
code location and status, is listed in [EQUATIONS.md](EQUATIONS.md); terms
are defined in [glossary.md](glossary.md); declared constants are listed in
[roadmap.md](roadmap.md), Declared rulers.

Status words used throughout:

- built: derived from the laws above it and read by the engine;
- partial: derived, but resting on a declared input or read only by the
  observer;
- placeholder: a declared constant or calibration standing until a law
  derives it.

Module paths are relative to `crates/xyl-cosmos/src` unless prefixed with
another crate (`xyl-core`, `xyl-kernel`).

## 1. Numbers

Three number types carry every law (`xyl-core`):

- `Q` (`q.rs`): exact rationals in lowest terms, promoted to big integers
  only when a value outgrows `i64`. Division is checked (`Q::checked_div`).
  Held state is `Q`: element counts, firmament and bead amounts, reaches,
  spans, phases, the anchor, star masses and births, world masses.
- `Encl` (`certified.rs`): a closed interval of `Q` proven to contain a
  transcendental value (pi, exp, ln, sin, cos). Used wherever a decision or
  a held value needs a proven bound: pi in the anchor and the bounce's
  curved reach, the island abundances, galaxy orbits and places
  (`chain::Place`), manifold caps and shares (`manifold`), the chemistry's
  closed forms (`xyl-core motion.rs` `ExpSum`).
- `Dy` (`dyadic.rs`): dyadic rationals `m 2^e` with a 62-bit mantissa,
  rounded toward zero in integer arithmetic, identical on every machine.
  Used for heavy readings at seams: the substrate's clock, reverberation,
  outflow, waveform and push (`circuit::core`); the settling (`systems`);
  the stars' burn (`stars`); life and minds (`life`, `minds`). A `Dy` value
  is converted to `Q` (often on a 2^-40 or 2^-64 grid) before it is held.

Angles are held as exact turns where the rate is rational and displayed in
degrees of 360. Phases about a throat are rounded to the coordinate grid of
60^-6 of a turn (`circuit::turn`); drawn values to the 2^-40 grid.

## 2. Units and the master frame

- The base is the universal core's own scale: `G = c = M_core = 1`. The core
  holds the universal state, 1. The unit of length is the core's
  gravitational radius `r_g = G M_core / c^2` (`xyl-core units.rs`).
- Tiers. Each tier reads in its own unit, an exact power of sixty of `r_g`
  (`xyl-core units.rs` `Tier::in_rg`): universe `60^10`, cluster `60^7`,
  galaxy `60^5`, system `60`, planet `60^-2`. Moving between tiers
  multiplies by a power of sixty (`manifold::dilation`), exact. The
  substrate's reaches (core rings) are in cluster units; the home galaxy's
  rings and every star's reach are in galaxy units.
- The bell (`circuit::anchor`, `Anchor::bell_gyr`): light's round trip to
  the home galaxy's rim, `bell = 2 rim_galaxy / c`, where `rim_galaxy` is the
  founding's rim read in the galaxy tier's unit. It is the unit of time.
- A ring of time is one full bell: every count of time (cooling, first
  light, a star's burn and life) is in whole rings. An echo is two bells,
  the near face's ring and the far face's: the paired throat
  (`circuit::echo_gyr`). A one-way crossing to reach `r` is travel within a
  ring, `(r / rim) bell / 2`, not a ring of its own.
- The pulse along the filament crosses the universe's rim in the same half
  bell, so the ocean's speed is `c_ocean = rim / (bell / 2) = 3600 c`, the
  ladder step between the galaxy and cluster tiers (`Substrate::c_ocean`).
- The anchor (`circuit::anchor::of`) fixes, from the founding alone: the rim;
  one cluster unit and one galaxy unit in kpc; the bell; the home galaxy's
  rim (`rim x galaxy_kpc`); the universe's curvature radius, `rim / pi` (each
  enclosed sphere's rim is half a great circle), in kpc and in `r_g`; the
  first stars, `max(cools_in, 1)` bells; the parent's admitted spin, its
  torsion share and the firmament's margin. The first founding in a process
  sets the anchor.
- The SI bridge (`package.rs`). `package::hole_mass` (4.3e6 solar masses) is
  the one bridge from core units to SI: it fixes `r_g` in metres and so
  every reading displayed in kpc, Gyr, AU, solar or world masses. The SI
  constants used at seams are in `package::Si`. Laws stated in core units do
  not depend on the bridge; the chemistry and the world states, which still
  read SI-calibrated constants, do. Next: the core's mass supplied by the
  package (roadmap, Founding from the package manifest).

## 3. The bounce and the founding

### The bounce (`bounce.rs`)

A universe begins as the white-hole face of a parent's black hole, in the
conformal algebra Cl(4,1) over `Q`, exact.

- Horizon reach: the hole holds the deficit share `delta = M / (R + M)` at
  reach `R`; the horizon is where the received share `1 - delta` is twice
  the held share, `delta_h = 1/3`, so `R_h = M (1 - delta_h) / delta_h = 2M`
  (`horizon_reach`).
- The reach is taken along the S^360 enclosure the core sits in, of
  curvature radius `rho`: `rho sin(R_h / rho)` (`curved_reach`).
- The crossing point lies on the first of the pulse's five isoclinic planes
  at the pulse's phase (`crossing`); the pole axis is left to the filament.
  The horizon is the conformal sphere about the centre through it
  (`horizon`, `xyl-core cga::sphere_through`).
- The spin is the bivector `K = a sum_k B_k` over the five planes' blades
  (`spin_bivector`). The white hole is the null generator `N = P ^ (P . K)`
  at the crossing point (`xyl-core cga::Source`), verified null, fixing `P`
  and keeping the horizon. The throat's far face is the crossing point's
  antipode under the planes' half turn (`circuit::plane_antipole`).
- `Bounce::of` refuses a spin outside `(0, 1)`: a spinless hole does not
  bounce.

### The Big Bang (`bang.rs`)

- The parent's Kerr spin is the seed's one draw: a uniform rational strictly
  inside `(0, 1)` on the 2^-40 grid (`BigBang::of`, `clamp_spin`). The
  founding depends on the spin alone, so the substrate is founded first and
  the parent bounce is read in its enclosure: `Bounce::of(1, spin,
  radius_rg, 0)`.
- Torsion share (`torsion_share`): the throat `H = 2M` holds one face
  against the other, `delta(H, H) = 1/2`; the spin `J` is a further share it
  cannot hold, so the held face is `H / (2H + J)` and the jet carries
  `a = (H + J) / (2H + J)`: one half spinless, three fifths extremal. The
  inverse is `J = H (2a - 1) / (1 - a)` (`spin_of_share`).
- Admission (`cores::substrate_for`): the torsion share must hold the
  firmament whole against the pulse, `a >= r_far / rim (1 + A_in_peak)`
  (`Substrate::least_share`). Where it does not, the spin is raised to the
  spin of that share and the universe is refounded, at most three times;
  the margin `a - least_share` is held on the anchor.
- The cascade's readings: lumpiness, the beads' states above the mean over
  every bead's state; held share, the firmament's mass over everything
  founded; assembly, the bells until the home galaxy's gas first lights;
  efficiency, the lit share of that gas (`formation::Lighting`). Nothing
  else is drawn.

### The elements (`elements.rs`, `atoms.rs`)

- Capture chain (`elements::founded`): nuclei are built one capture at a
  time from hydrogen, `count_Z = floor(count_(Z-1) a h_Z)`, with
  `h_Z = (3Z - u_Z) / 3Z` the new nucleus's closed share and `u_Z` its
  unpaired windings (`atoms::unpaired`). Counts are whole nuclei out of
  `2^b` hydrogen, `b = founding_bits(a)` the fewest bits that carry one
  whole nucleus of the vessel's sealing kind through the floored chain. A
  kind with fewer than one nucleus does not exist. Next: the light kinds'
  shares, read from the same founding as the firmament (roadmap, Founding
  from the package manifest).
- The table runs Z = 1 to `TOP_Z` = 1700. `data/elements.tsv` holds Z 1 to 99
  as measured rows and Z 100 to 150 as predicted rows (each from its
  homologue by the periodic law); Z 151 to 1700 are generated with
  systematic names, the valley of stability's mass number
  (`elements::valley_a`) and uranium as the homologue for material columns.
  The founding replaces the abundance column with the chain's counts; mass
  numbers, phase temperatures, densities and geochemical classes remain
  catalogue values. Next: derive those columns from the atom (roadmap,
  Founding from the package manifest, chemistry from the census).
- The atom (`atoms::atom`) is computed on S^11: a shape-`l` subshell holds
  `2 (C(l + 11, 11) - C(l + 9, 11))` electrons (orientations 1, 12, 77,
  352, ...), so shells hold 2, 26, 180, 884 and the seals (every band and
  every shape of the outermost shell full) fall at Z 2, 28, 208 and 1092.
  Electrons fill the tightest open winding, tightness `(Z - inside) / n`.
- The vessel (`atoms::vessel`): from the heaviest seam below the last seal
  (a kind whose single-nucleon peel releases, `u(z - 1) + 1 - u(z) < 0`) to
  that seal. Both edges are computed; on the current table the band is
  Z 568 to Z 1092.

### The cascade, shells and beads (`formation::founding_shells`, `circuit::Substrate::found`)

- The founding's mix is every founded kind's share of holding
  (`formation::kinds`), converted to mass (`Z + Q` nucleons per nucleus,
  `circuit::nucleus_weights`).
- Equitable cascade: in Z order each kind takes the share
  `s_Z = 1/2 held / (held + 1 + held)` of the unlocked face, `held` the mass
  share before it. `K = sum s_Z`, `H = 1 + held_final`, and the final share
  `s_final = 1/2 held / (held + H)`. Kinds are stacked by the mass profile,
  heaviest first, cut at its local minima; each kind is thrown to
  `R_Z = (K + s_i) / w_Z / H` (`s_i` its stack's shares, `w_Z` its nucleon
  mass) and spreads `R (1 +/- s_final)`.
- The filter: every lighter shell passes out through each heavier shell
  whose spread lies between the throat and its reach. Each passage grips
  `g = min(s D, T) / T` (`s` the two kinds' tightness ratio,
  `D = M / (M + 1 - contact)`, `T` the contrast of their deficits) and the
  heavier gives `contact g m_i / (m_i + m_j)` of its matter, on the nucleus
  grid; takes are scaled where a shell is asked more than it holds; a
  shell's reach becomes the holding-weighted mean of its parts' release
  reaches. A kind is held by the throat where `R (1 + s_final) <= 1`.
- Beads: the pushed-out shells link into runs where spreads overlap. A run's
  superposed hold (each kind's mass over its spread's deficit contrast)
  peaks at beads and parts at its local minima; each kind's matter goes to
  the bead whose span contains its reach. A bead's state is its recipe's
  mass; its reach and phase are its most abundant kind's (the phase is the
  torsion share turned Z times); its hold inside is
  `1 + 2 sum(inner beads' shares)`.
- The rim is the outermost shell's outer edge, `max R (1 + s_final)` over
  every shell.

## 4. The firmament and the core

- The firmament (`circuit::Substrate::found`, `circuit::Kind`): the held
  kinds inside the vessel band, each at its reach (its spin about the
  throat) and its capture phase. Their amounts are scaled, each in its
  founded proportion, so that their mass is the universal state, 1: the
  firmament carries the core's 1. The held share is then
  `1 / (1 + unit)`, `unit` the beads' summed states (`held_share`).
- The deficit law is the grammar of every hold: `delta(R, M) = M / (R + M)`,
  received share `1 - delta`. The vacuum's share at reach `R` about a hold
  `H` is that deficit, the push the pulse delivers there
  (`circuit::vacuum_share`); read at the throat itself it is the throat's
  push, `p = delta(H, H) = 1/2` (`circuit::throat_push`). A band swinging
  `r (1 +/- p)` dilates the pole coordinate by `rho = (1 + p) / (1 - p) = 3`
  per echo (`circuit::echo_dilation`).
- Reverberation (`Substrate::reverberation_with`): firmament kinds in order
  of spin; each one's hold `h = 1 + (firmament mass nearer) / unit`, reach
  `r = spin / h`, deficit `delta = h / (r + h)`, share of its band past the
  throat `beyond = clamp((r (1 + delta) - h) / (2 delta r), 0, 1)`, and
  leaving rate `beyond / bell`.
- The jet: a kind's pole state sits at `t = open / hold` on the filament and
  reaches the throat after `ln(1 / t) / ln rho` echoes, its onset
  (`circuit::onset_echoes`); a sealed kind never does. It then empties as
  `m(t) = m0 (1 - exp(-rate (t - onset)))` (`Substrate::outflow`).
- The inbound echo (`Substrate::inbound`): the waveform at the throat per
  echo, `A_in(t) = cos(2 pi t / echo) + echo q(t)`, with `q(t)` the vessel's
  outflow rate in whole state (`Substrate::rate`). Its peak is
  `1 + echo max q(onset)` (`inbound_peak`), its mean over the first `t` the
  drift plus the ring's rung (`inbound_mean`). The push at `R` and `t` is
  `h / (R + h) (1 + A_in(t))` (`push_at`).
- The jetstream (`Substrate::loop_flux`, `circuit::pole_flux`): over one
  echo three channels compose on each firmament kind's pole state as
  dilators: the filament's dilation at the throat, the vacuum's swung share
  of the hold, and the electrons' phase state `R` on the open frames
  (`circuit::electron_phase`, `R = kappa (1 - 2 frac(p))`). The residual
  dilator against the kind's own state is the torsion the echo leaves; the
  loop closes where it is one. There is one jetstream per held kind. Status:
  partial (read by the observer).
- The ocean: the waveform at reach `r`,
  `A(r, t) = [cos(2 pi (t - tau) / echo) / echo + q(t - tau)] / (r + 1)`,
  `tau = (r / rim) bell / 2` (`Substrate::amplitude`). Each galaxy's share of
  a kind the jet throws is its pull `h / (r + h)` (`h` its state over the
  mean) times its coherence with the kind's phase, `1 - 2 min(d, 1 - d)`,
  over the sum (`Substrate::landing`). Its composition at `t` is its recipe
  plus its intake (`composition`); the books close by mass:
  held + left = founded, taken + in flight = left (`books`).
- The push on a galaxy (`Substrate::push`): push over inertia on its spin
  about the throat, in closed form,
  `l(t) = l0 + d [Q(t - tau) + sin(2 pi (t - tau) / echo) / 2 pi] / ((R + 1) H)`;
  recession is `dl/dt / inside`, the shift `z = v / c_ocean`, the tide
  across extent `x` is `A(R - x) - A(R + x)`.
- Galactic cores (`cores::founding_of`, `Substrate::holds_a_star`). A bead's
  state is its mass in the universal core's units, so its core's mass reads
  `hole_suns = hole_mass x state`. A bead is a galaxy where the lit share of
  that mass is at least one sun, `lit x hole_mass x state >= 1`. Each
  galaxy circulates about the throat at `4 delta` turns per echo,
  `delta = inside / (reach + inside)`; its plane is set by the repulsion
  pivots one tier up (node by the golden turn, tilt by the plastic turn's
  share of the S^360 measure) and its place at `t` is its plane's rotor
  applied to its phase (`Shape::place_at`). Another galaxy's stars are the
  home galaxy's read through its length scale `2 m / (1 + m)`, `m` its hole
  mass ratio (`Shape::scale`, placeholder).

## 5. The galaxy

- Structure (`galaxy::Structure`): the home galaxy is the core's own matter,
  so it is the founding read one tier down. Its rings are the beads, their
  reaches and spans in galaxy units, their holding their states. The core's
  reach is one galaxy unit, where `1 / (r + 1)` is one half
  (`galaxy::core_kpc`); the nucleus is the innermost ring; the reference
  ring is where half the holding lies within (`galaxy::r0`); crowding is a
  ring's state per span over the reference ring's (`galaxy::density`).
- Mass: `stellar_mass = Lighting::lit x hole_mass` (`galaxy::stellar_mass`);
  the mass inside `r` is `hole_mass (1 + rings within r / all rings)`
  (`galaxy::inside_mass`), read by the turning law (`turning.rs`,
  `omega = c delta(<r) / r`).
- Lighting (`formation::cooling`, `formation::collapse`,
  `formation::lighting`). Each bead is a piece of gas: holding per galaxy
  ring `lambda = (state / unit) / W`; agitation `(p - delta) / p` with
  `delta = h / (R + h)`, `h = 1 + held share`; emission
  `e = sum s_z open_z / hold_z`, magnetism `mu = lambda e`. Outside the
  core's reach the gas is a circle (`n = 1`), within it a ball (`n = 2`). An
  infall of extent `w` gathers `m(w) = (lambda + mu) W share_n(theta)`,
  `theta = w / (w + R)` of the half turn (`xyl-core measure::share_turns`),
  and grips it as `m / (w + m)`. The pulse heats the piece by
  `ring x swing x grip(W) / shed` (capped at 1), `ring = 1 / (pi (R + 1))`,
  `swing = 1 + A_in` at the first echo's mean, `shed = e + ln 2 / k_v`. On a
  ladder of halvings of `W` the spare is `grip - H w / (R (R + w)) - heat`;
  the piece halves its agitation until it is under the best spare, at
  `floor(n / P_e) + floor(n / k_v)` halvings by ring `n`
  (`P_e = ceil(1 / e)`, `k_v` the vacuum's halving), and fragments at the
  largest extent that holds. It lights where its agitation at ignition is
  under the throat's push. The first stars are the first lit piece's ring
  count; the lit share is the star efficiency.
- One reach per tier: a star's reach about the core is in galaxy rings,
  `r_kpc / galaxy_kpc` (`stars::reach_rings`, `Anchor::galaxy_kpc`), the
  same unit its piece was lit in.
- Placement (`galaxy::star`): sample stars stand for equal shares of the
  stellar mass and are placed along the rings' cumulative holding by
  irrational pivots (golden, silver, plastic and `sqrt 3 - 1` turns), the
  repulsion law; disc stars orbit in the plane, core and nucleus stars on
  tilted planes. The orbit's rate is the turning law at its reach. The
  initial mass function (`galaxy::imf`), the binary share and the vertical
  orbit's scales (`galaxy::vertical`) are declared rulers. Next: derive
  them from the piece's infall (roadmap, Declared rulers).
- Births (`galaxy::birth_gyr`, `chem::Chemistry::birth_along`): a star's ring
  first lights at `max(cools_in, 1)` bells; no star stands in a ring that
  never lights. Its birth is then placed on the ring's own star formation:
  the time by which the share `u` (its draw) of the ring's stars formed from
  first light to the horizon (30 Gyr) have formed, the ring's `sfr`
  integrated term by term, `c p! / k^(p+1) (1 - e^(-kt) sum_(j<=p) (kt)^j / j!)`
  (`c t^(p+1) / (p + 1)` where `k = 0`), and found by 48 halvings.
- Chemistry (`chem::Model`): each disc ring (30 rings of 0.5 kpc) is a sum of
  closed forms `c t^m e^(-lambda t)`. Gas falls in on the ring's own
  first-light timescale toward the structure's crowding profile; stars form
  at `efficiency x Omega / 2 pi`; prompt and slow returns, the fountain's
  landing map and the core's jets feed each ring once; each nucleosynthetic
  channel makes its elements after its delay. Yields are scaled per channel
  by `chem::Calibration` against the gas at the reference ring and the
  solar-birth reference (`Rulers::ism_today_dex`, `Rulers::reference_metallicity`,
  `Rulers::today_gyr`, `Rulers::v_flat`, `Rulers::sigma_r0`); hydrogen and
  helium come from the founding's own counts (`chem::Light`). Status:
  partial. Next: yields and ring scales from the founding (roadmap,
  Founding from the package manifest, chemistry from the census).

## 6. Stars and systems

- The burn (`stars.rs`): a star at reach `r` absorbs the ring the bell
  brings there, `1 / (pi (r + 1))` per bell, plus the vessel's drift
  `bell q(t) / (r + 1)`, and takes its own deficit of it, `delta_star`, read
  on the solar rim (`stars::delta_star`, `package::Si::star_radius_m`). Its light is
  that burn transduced, `L = M / (r + 1) (1 + pi bell q(t))` in solar units
  (`systems::luminosity`, `stars::light`); its main-sequence life is
  `(1 - hold(He) / (4 hold(H))) / burn` bells, one quarter over the burn
  (`stars::life`); its giant ends at that life times
  `transduced_He / transduced_H` (`stars::ends`); its remnant keeps
  `1 - transduced_He` of its hold (`stars::remnant_share`). The radiation's
  swing within a bell is `(pi / 2) |cos(2 pi t / echo)|`
  (`stars::flicker`).
- The star's cut (`disc::star_cut`): the infall of its lit piece collapses
  carrying the galaxy's shear across its extent, `h w / (R (R + w))`, as its
  turning, heated by the pulse's turning per echo at the star and pushed by
  its burn per echo as over-pressure `p0 = 1 + burn`. Each kind is carried
  to `l = p p0 (e / e_mean)(w_mean / w) + add` and held where
  `h / (l / h + h) >= p` on the held hold (`formation::hold_at_heat`). The
  disc's agitation is `turning x swing x grip / shed`, with
  `grip = (1 - star) / (2 - star)` and `shed = e_rest + ln 2 / k_v`.
- The star's manifold (`galaxy::disc_frame`): its gathering extent is its
  mass in holding units (`mass / stellar_mass`) over the infall's holding
  per ring, at most the infall; its manifold reach is the arc of its circle
  about the core its pull holds, `manifold::reach(r, m / M(<r)) =
  2 pi r m / (M + m)`; the star's share is gathering over reach, at most 1.
- Worlds on the manifold (`disc::manifold_worlds`): in the manifold's units
  the star holds the inner share `s`; the rest is read over 360 degrees out
  to the reach, each degree adding `(1 - a) (1 - s) / 360` of matter. A world
  forms where the matter gathered since the last world holds its region
  against the core's twist, `m / (s + m) >= tide x`, `x` the degree's outer
  edge; its reach is the gathered matter's mean. There is no seed and no
  partition. What never holds a region is belt; the books close at
  `1 - s`.
- The system (`systems::system`): the worlds' outer edge is the manifold
  reach in AU; the infall's solids are the star's mass over its share times
  the gas's metal mass fraction (relative metals times `Rulers::reference_metallicity`); a
  region inside the rock line (the lowest condensation temperature of Fe,
  Si, Mg) holds no solids. Each region's mass is its holding times the
  solids, enriched past the water line; its phase is its index times the
  golden turn. A core of ten world masses or more takes an envelope by the
  declared gathering law.
- Settling (`systems::settle_about`): about an anchor of mass `M` (core
  units), each body turns at `omega(A) = [M / (A + M)] / A`. Neighbours meet
  `N = age |dw|` times; each meeting kicks by
  `eps = [m / (r + m)] / r / |dw|`; the kicks repeat on the continued
  fraction period `q` of the phase step and cancel to
  `c = |1 - 2 frac(q psi_0)|` per period, so the greatest swing is
  `eps (floor(N / q) c + N mod q)`. A pair merges at the first age its swing
  spans its gap (48 halvings out to the horizon). Merges by the settled age
  (3 Gyr) combine the pair, mass-weighted; later meetings are recorded on
  the lighter body as its merge age (`World::merged_at`). A body's starting
  eccentricity is its swing and its tilt `swing / (1 + swing)` of a quarter
  turn (`bodies::orbits`).
- Moons (`systems::moons`): every world anchors its own manifold, reach
  `2 pi a m / (M + m)` out from five of its radii. Its budget is the disc's
  agitation times its width on the star's manifold times the infall's
  solids, at most what the belts still hold; the world keeps
  `m / (m + budget)`; moons gather by the same degree law against the star's
  tide at the world, `s / (x + s)`, then settle about the world.
- Belts (`systems::belts`): the ungathered remainder, less the moons, lies
  beyond the outermost world out to the manifold's reach; giants fling a
  comet cloud by a declared share.
- Exponentials below the working resolution are enclosed by
  `[0, 2^-bits]` rather than computed (`systems::exp`).
- Angles between bodies: `manifold::turn_between` and `chain::Place::arc`
  read the turn between two directions as the arccosine of their dot
  product. Next: read it from the versor between the two bodies' frames,
  which needs `Rotor`'s inverse and the versor's angle (roadmap, Worlds on
  the manifold chain, cross-ties).
- Next: the manifold's disc-mass scale. At real metallicity a star's
  manifold holds solids of the star's own order and the settling merges its
  worlds into one heavy world per star (roadmap, Worlds on the manifold chain).
- World states (`worlds::build`): air, seas, ices, surface temperature and
  pressure, interior heat and field, built at an age from the setting, in
  world masses and radii, AU and kelvin through `package::Si`; the disc's
  formation temperature is `278 L / a` (`systems::Spec::t_form`). Worlds in
  play are read in degrees of their star's life, its lifetime over 360,
  floored at one bell (`worlds::star_degree_gyr`).

## 7. Life and minds

- Ladder (`life::ladder`): a medium's bare chemistry miscopies with
  `eps0 = exp(-d E_weak / kT)` (`life::fidelity`). Rung `s` holds
  `b / eps^(s+1)` bits, `eps` including the decay a generation suffers,
  capped at a mind's blueprint `I_MIND`; at most 12 rungs. Each rung's mean
  wait is the bits to write over what selection writes per generation
  (`life::bits_per_generation`), in the population the medium's power feeds.
  Monomers are built from the solvent's own molecules; no solvent list is
  fixed.
- Climb (`life::climb`): each rung takes its mean; impacts at rate `mu` (the
  system's own debris swept once an orbit, `life::strikes_per_gyr`) make a
  rung of mean `m` complete in `(e^(mu m) - 1) / mu`; a sterilising pulse
  falls at `1 / hazard`. No draw enters.
- Forecast (`minds::schedule`): for every world a sight opens, a mind wakes
  at its last rung if its world at that epoch feeds a viable people of 500
  bodies (`life::viable`). The first age read is one degree of the star's
  life; the horizon is 30 Gyr.
- Ancients (`minds::ancients`): minds born within a galactic core's reach
  that wake within one core assembly time (the nucleus's first light,
  `minds::ancient_window`) of that core's first core-born mind.
