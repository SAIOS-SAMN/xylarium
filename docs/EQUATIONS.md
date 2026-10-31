# Equations

This document lists Xylarium's physics equations, formula by formula, in the
order of [model.md](model.md), which explains how each follows from the one
before. Terms are defined in [glossary.md](glossary.md).
Each entry gives the formula, its code location and its status. The status
is one of three values:

- Built: the formula is derived from the laws above it and read by the
  engine.
- Partial: the formula is derived, but it rests on a declared input or is
  read only by the observer.
- Placeholder: the formula is a declared constant or calibration that stands
  until a law derives it; placeholders are listed in
  [roadmap.md](roadmap.md), Declared rulers.

Module paths are relative to `crates/xyl-cosmos/src` unless prefixed with
another crate. Laws are composed from shares (ratios of the form
`M / (R + M)`) and angles, not from squares or cubes of a quantity.

## 1. Numbers

- `Q`, the exact rational (`xyl-core q.rs`), holds all held state. Status:
  built.
- `Encl`, the certified enclosure `lo <= x <= hi` over `Q`
  (`xyl-core certified.rs`), holds transcendental values with proven bounds.
  Status: built.
- `Dy`, the dyadic `m 2^e` with a 62-bit mantissa rounded toward zero
  (`xyl-core dyadic.rs`), holds readings at seams; they are converted to `Q`
  on a grid before being held. Status: built.
- Phase grid (`circuit::turn`): a turn is rounded to `60^-6`. Status: built.

## 2. Units and the master frame

- Base units (`xyl-core units.rs`): `G = c = M_core = 1`, and
  `r_g = G M_core / c^2` (`Units::of`). Status: built.
- Tier unit (`Tier::in_rg`): the unit of a tier is `60^k r_g`, with `k` = 10
  for the universe, 7 for the cluster, 5 for the galaxy, 1 for the system and
  -2 for the planet; between tiers,
  `dilation(from, to) = 60^(k_from - k_to)` (`manifold::dilation`). Status:
  built.
- Bell (`circuit::anchor::of`): `bell = 2 rim 60^5 r_g / c`, with the rim
  read in the galaxy tier. Status: built.
- Ring and echo (`circuit::bell`, `circuit::echo`): one ring is one
  bell, `echo = 2 bell`, and a one-way crossing to reach `r` takes
  `(r / rim) bell / 2`. Status: built.
- Ocean speed (`Substrate::c_ocean`): `c_ocean = rim / (bell / 2) = 3600 c`.
  Status: built.
- Universe radius (`circuit::anchor::of`): the radius is `rim / pi`, given
  in kpc and in `r_g`. Status: built.
- First stars (`circuit::anchor::of`): the first stars form at
  `max(cools_in, 1) bell`. Status: built.
- SI bridge (`package::hole_mass`): `M_core = 4.3e6` solar masses, which fixes
  `r_g` in metres for display in kpc, Gyr, AU and solar or world masses; the SI
  constants are in `package::Si`. Status: placeholder; the next step is the
  core's mass from the package (roadmap, Rules from the founding and the
  galactic cores).

## 3. The bounce and the founding

- Deficit law (`circuit::deficit`, `circuit::vacuum_share`,
  `turning::deficit_share`): `delta(R, M) = M / (R + M)`, and the received
  share is `1 - delta`. Status: built.
- Horizon reach (`bounce::horizon_reach`): `delta_h = 1/3`, so
  `R_h = M (1 - delta_h) / delta_h = 2M`. Status: built.
- Curved reach (`bounce::curved_reach`): the reach is `rho sin(r / rho)` on
  the S^360 enclosure of radius `rho`. Status: built.
- Horizon sphere (`bounce::horizon`): `sigma = sphere_through(C, P)`, where
  `P` is the crossing on isoclinic plane 1 at reach `rho sin(R_h / rho)` and
  the pulse's phase (`bounce::crossing`). Status: built.
- Spin bivector (`bounce::spin_bivector`): `K = a sum_(k=1..5) B_k`. Status:
  built.
- White hole (`bounce::Bounce::of`, `xyl-core cga::Source`):
  `N = P ^ (P . K)` is null, fixes `P` and keeps `sigma`; the far face is the
  antipode of `P` under the planes' half turn (`circuit::plane_antipole`).
  Status: built.
- Spin draw (`bang::BigBang::of`): the spin is drawn uniformly on the 2^-40
  grid, strictly inside `(0, 1)` (`bang::clamp_spin`). Status: built.
- Torsion share (`bang::torsion_share`): `a = (H + J) / (2H + J)` with
  `H = 2`, so `a = 1/2` at `J = 0` and `a = 3/5` at `J = 1`. The inverse
  (`bang::spin_of_share`) is `J = H (2a - 1) / (1 - a)`. Status: built.
- Admission (`Substrate::least_share`, `cores::substrate_for`): the founding
  requires `a >= (r_far / rim) (1 + A_in_peak)`; otherwise the universe is
  refounded at `spin_of_share(least_share)`, with at most three foundings.
  The margin is `a - least_share` (`Substrate::margin`). Status: built.
- Cascade readings (`bang::BigBang::of`): the lumpiness is
  `sum max(state - mean, 0) / sum state`, the held share is
  `firmament mass / holding`, the assembly time is `Lighting::cools_in` and
  the efficiency is `Lighting::lit`. Status: built.
- Closed share (`elements::closed_share`): `h_Z = (3Z - u_Z) / 3Z`. Status:
  built.
- Capture chain (`elements::founded`): `count_1 = 2^b` and
  `count_Z = floor(count_(Z-1) a h_Z)`. Status: built.
- Founding bits (`elements::founding_bits`): `b` is the least value with
  `chain(2^b) >= 1` at the vessel's sealing kind (Z 1092). Status: built.
- Element table (`elements::table`): Z 1 to 99 are measured rows, Z 100 to
  150 are predicted rows (`data/elements.tsv`), and Z 151 to `TOP_Z` = 1700
  are generated (`elements::systematic_row`); the mass number lies on the
  valley `Z = A / (1.98 + 0.334 e(A / 100))` (`elements::valley_a`). Status:
  the counts are built; the catalogue columns (mass, melt, boil, density,
  condensation, class) are placeholders, and the next step derives them from
  the atom.
- Orientations on S^11 (`atoms::orientations`): a band has
  `C(l + 11, 11) - C(l + 9, 11)` orientations, and its room is twice that.
  Status: built.
- Fill (`atoms::atom`): each electron goes to the tightest open winding, with
  tightness `(Z - inside) / n` and winding reach `(n + 1)(l + 1)`. Status:
  built.
- Seals (`atoms::Atom::sealed`): the sealed kinds are Z 2, 28, 208 and 1092.
  Status: built.
- Vessel (`atoms::vessel`): the upper edge is the last seal, and the lower
  edge is the greatest `z < upper` with `u(z - 1) + 1 - u(z) < 0` (Z 568 on
  the current table). Status: built.
- Cascade shares (`formation::founding_shells`):
  `s_Z = 1/2 held / (held + 1 + held)`, cumulative by Z on mass;
  `K = sum s_Z`; `H = 1 + held_final`; and
  `s_final = 1/2 held_final / (held_final + H)`. Status: built.
- Shell reach (`formation::founding_shells`): `R_Z = (K + s_i) / w_Z / H`,
  with `s_i` the stack's shares and `w_Z = Z + Q`; the spread is
  `R (1 +/- s_final)`, and a shell is held where `R (1 + s_final) <= 1`.
  Status: built.
- Filter (`formation::founding_shells`): the contact is `1/2` for a whole
  passage and `1/2` of the share crossed for a partial one;
  `D = M / (M + 1 - contact)`, `T = |d_j - d_i| / (d_j + d_i)` and
  `g = min(s D, T) / T`. The filter takes `contact g m_i / (m_i + m_j)` of
  `m_j`, scaled by held over asked, and the new reach is the
  holding-weighted mean of the release reaches. Status: built.
- Kind density in a run (`Substrate::found`): the density is `mass / t`,
  with `t = |delta(lo, 1) - delta(hi, 1)| / (delta(lo, 1) + delta(hi, 1))`;
  beads lie at the local maxima of the superposed hold and are parted at its
  minima. Status: built.
- Bead (`Substrate::found`): the state is the recipe mass; the reach and
  phase are those of the most abundant kind, with phase `turn(a Z)`; and the
  inside is `1 + 2 sum(inner states / pushed-out mass)`. Status: built.
- Rim (`Substrate::found`): the rim is `max R (1 + s_final)` over every
  shell. Status: built.

## 4. The firmament and the core

- Firmament (`Substrate::found`): the firmament holds the held kinds inside
  the vessel, with spin equal to reach on the 2^-64 grid and phase
  `turn(a Z)`; amounts are scaled by `1 / sum mass_of(z, amount)` so that the
  firmament's mass is 1. Status: built.
- Mass of an amount (`circuit::mass_of`): `amount (Z + Q) / hold_Z`.
  Status: built.
- Held share (`Substrate::held_share`): the held share is
  `firmament mass / (firmament mass + sum bead masses)`. Status: built.
- Throat push (`circuit::throat_push`): `p = delta(H, H) = 1/2`. Status:
  built.
- Echo dilation (`circuit::echo_dilation`): `rho = (1 + p) / (1 - p) = 3`.
  Status: built.
- Reverberation (`Substrate::reverberation`): in order of spin,
  `h = 1 + inside / unit`, `r = spin / h`, `delta = h / (r + h)`,
  `beyond = clamp((r (1 + delta) - h) / (2 delta r), 0, 1)` and
  `rate = beyond / bell`. Status: built.
- Jet onset (`circuit::onset_echoes`): the onset falls after
  `ln(1 / t) / ln rho` echoes, with `t = open / hold`; a sealed kind has no
  onset, and the onset is zero where `t >= 1`. Status: built.
- Outflow (`Substrate::outflow`): `m(t) = m0 (1 - exp(-rate (t - onset)))`
  for `t > onset`. Status: built.
- Outflow rate (`Substrate::rate`):
  `q(t) = sum m0 (1 + e_z) rate exp(-rate (t - onset))`, with
  `e_z = open_z / hold_z` (`formation::emission`). Status: built.
- Inbound echo (`Substrate::inbound`):
  `A_in(t) = cos(2 pi t / echo) + echo q(t)`. Status: built.
- Inbound peak (`Substrate::inbound_peak`): the peak is
  `1 + echo max_j q(onset_j)`. Status: built.
- Inbound mean (`Substrate::inbound_mean`): the mean is
  `echo (sin(2 pi t / echo) / 2 pi + Q(t)) / t`, with `Q` the state out by
  `t`. Status: built.
- Push at reach (`Substrate::push_at`):
  `p(R, t) = h / (R + h) (1 + A_in(t))`. Status: built.
- Band past the throat at `t` (`Substrate::beyond_at`): the share is
  `beyond` with the deficit replaced by `p(r, t)`. Status: built.
- Pulse phase (`Substrate::pulse`): the phase is `frac((t / echo) / t_u)`,
  with `t_u = floor(2 (rim + 2)) + 1` echoes. Status: built.
- Jetstream (`Substrate::loop_flux`, `circuit::pole_flux`): on a kind of hold
  `w` and open frames `u`, the filament channel moves
  `(rho - 1) u w / (w + rho u)` frames from Z to Q, the vacuum channel moves
  `p(r, t) w` frames from Z to Q (none where `p >= 1`), and the echo channel
  moves `R u` frames from Q to Z. Each channel is a dilator applied to the
  state left by the previous channel; the residual is the observed dilation
  over the channels' product and equals 1 where the loop closes. Status:
  partial (read by the observer).
- Waveform (`Substrate::amplitude`):
  `A(r, t) = [cos(2 pi (t - tau) / echo) / echo + q(t - tau)] / (r + 1)`,
  with `tau = (r / rim) bell / 2`. Status: built.
- Pull and delay (`Substrate::ocean_shares`): the pull is `h / (r + h)` with
  `h = state / mean state` (the home galaxy has 1 at reach 0); a galaxy's
  share is its pull over the sum, and its delay is `tau`. Status: built.
- Coherence (`circuit::coherence`): `1 - 2 min(d, 1 - d)`, with `d` the
  phase difference in turns. Status: built.
- Landing (`Substrate::landing`): a galaxy's share is its pull times its
  coherence over the sum, or the pull alone where every coherence is zero.
  Status: built.
- Intake and composition (`Substrate::intakes`, `Substrate::composition`):
  the intake is the landing share times `outflow(t - tau)`, and the
  composition is the recipe plus the intake. Status: built.
- Books (`Substrate::books`): mass is conserved as held + left = founded and
  taken + in flight = left. Status: built.
- Push on a galaxy (`Substrate::push`):
  `l(t) = R inside + d [Q(t - tau) + sin(2 pi (t - tau) / echo) / 2 pi] /
  ((R + 1) H)` and `reach = l / inside`; the recession is `dl/dt / inside`,
  and the shift is `z = recession / c_ocean`. Status: built.
- Tide (`Substrate::tide`): `A(R - x, t) - A(R + x, t)`. Status: built.
- Galaxy selection (`Substrate::holds_a_star`): a bead is a galaxy where
  `lit x hole_mass x state >= 1` (solar masses). Status: built.
- Galactic core mass (`cores::founding_of`): `hole_suns = hole_mass x state`.
  Status: built.
- Circulation (`cores::founding_of`): the circulation is `4 delta / echo`
  turns per Gyr, with `delta = inside / (reach + inside)`. Status: built.
- Galaxy plane (`cores::founding_of`): the node lies at
  `index x golden turn`, and the tilt holds `index x plastic turn` of the
  S^360 measure (`galaxy::node_of`, `galaxy::tilt_of`). Status: built.
- Galaxy scale (`cores::Shape::scale`): `2 m / (1 + m)`, with
  `m = hole_suns / hole_mass`. Status: placeholder.

### Z, Q and R (the pole model)

- Weights (`circuit::nucleus_weights`, `circuit::mix_weights`): `Z` is the
  proton count and `Q` the neutron count (the mass number less the charge,
  from the element table; a cluster sums its parts), and a kind weighs
  `Z + Q` nucleons. Status: built.
- Electron phase state (`circuit::electron_phase`):
  `R = kappa (1 - 2 frac(p))` in `[-kappa, kappa]`, with `p` the bounce's
  pulse in turns and `kappa` the electrons' coherence on S^11 (`fibres`,
  `2 G / ends` for the charge's electrons); `R` is zero for a kind with no
  charge. Status: built.
- Pole point (`circuit::pole_point`, `circuit/poles.rs`): `X(t)` lies on the
  pole axis at `t = open / hold`, with the Z pole at the origin `e_o`, the Q
  pole at the point at infinity `e_inf` and the throat at `t = 1`; the
  inversion is `X(t) -> X(1 / t)` (`circuit::axis_antipole`). Status: built.
- Axes (`circuit/axes.rs`): the pulse has one fixed axis, the pole axis
  (base axis 1), and five isoclinic planes on base axes 2-3, 4-5, 6-7, 8-9
  and 10-11 (`circuit::plane_blade`). The planes' half turn
  (`circuit::plane_antipole`) sends a point to its antipode, and together
  with the axis's inversion it forms the pulse's antipole
  (`circuit::antipole`). The antinode share of harmonic `q` is 0 for even `q`
  and `1/q` for odd `q` (`circuit::antinode`). Status: built.

## 5. The galaxy

- Structure (`galaxy::Structure`): the rings are the beads sorted by reach,
  with edges `span x galaxy_kpc` and per-kpc holding `state / width`; the
  core reach is one galaxy unit, the nucleus is the first ring's outer edge,
  and the reference ring is where the cumulative holding crosses one half.
  Status: built.
- Crowding (`galaxy::density`): `lambda_ring / lambda_reference`. Status:
  built.
- Stellar mass (`galaxy::stellar_mass`): `Lighting::lit x hole_mass`.
  Status: built.
- Mass inside (`galaxy::inside_mass`):
  `hole_mass (1 + rings within r / all rings)`, with a ring counted by the
  share of its span inside `r`. Status: built.
- Turning law (`turning.rs`): `omega = c delta(<r) / r`, with
  `delta(<r) = M(<r) / (r + M(<r))` in `r_g`. Status: built.
- Piece of gas (`formation::cooling`): `lambda = (state / unit) / W`; the
  agitation is `a0 = (p - delta) / p` with `delta = h / (R + h)` and
  `h = 1 + held share`; `e = sum s_z e_z` and `mu = lambda e`; `n = 1`
  beyond one galaxy ring and `n = 2` otherwise. Status: built.
- Gathering (`formation::gathered`): `m(w) = (lambda + mu) W share_n(theta)`,
  with `theta = w / (2 (w + R))` turns (`xyl-core measure::share_turns`).
  Status: built.
- Grip and shear: the grip is `m / (w + m)`, and the shear is
  `h w / (R (R + w))`. Status: built.
- Vacuum halving (`formation::vacuum_halving`): `k_v` is the least whole
  `k >= ln 2 / ln((R + h) / R)`. Status: built.
- Heat (`formation::collapse`): the heat is
  `min(1, ring x swing x grip(W) / shed)`, with `ring = 1 / (pi (R + 1))` and
  `shed = e + ln 2 / k_v`. Status: built.
- Spare (`formation::collapse`): the spare is `grip(w) - shear(w) - heat` on
  `w = W / 2^k` for `k < 48`. Status: built.
- Cooling (`formation::collapse`): the gas halves until `a <= max spare`;
  `cools_in` is the least ring `n` with
  `floor(n / P_e) + floor(n / k_v) >= halvings`, where `P_e = ceil(1 / e)`;
  the infall is the largest `w` with `spare(w) >= a`, and the number of
  clouds is `max(1, floor(W / infall))`. Status: built.
- Lighting (`formation::lights`, `formation::lighting`): a piece lights where
  clouds > 0 and `cooled + heat < 1/2`; `cools_in` is the least over lit
  pieces, and `lit` is the lit holding over all holding. Status: built.
- Reach in galaxy rings (`stars::reach_rings`): `r_kpc / galaxy_kpc`.
  Status: built.
- First light (`galaxy::birth_gyr`): the first light of the piece at the
  star's reach falls at `max(cools_in, 1) bell`; there is none where the
  piece never lights. Status: built.
- Birth along the ring (`chem::Chemistry::birth_along`): the birth time is
  the `t` with `F(t) = F(t0) + u (F(top) - F(t0))`, where `top` is the 30 Gyr
  horizon, found by 48 halvings. `F` is the ring's `sfr` integrated term by
  term, `c p! / k^(p+1) (1 - e^(-kt) sum_(j<=p) (kt)^j / j!)`, or
  `c t^(p+1) / (p + 1)` at `k = 0` (`chem::formed_by`). Status: built.
- Star formation per ring (`chem::Model::of`): the rate is
  `efficiency x Omega / 2 pi` with `Omega = v_flat / r`; the gas is a sum of
  `c t^m e^(-lambda t)` terms, with infall on the ring's first-light
  timescale toward `galaxy::density x sigma_r0`. Status: partial (rulers
  `v_flat`, `sigma_r0`, `prompt`, `slow`, `loading` and the ring width).
- Yields (`chem::Calibration`): per-channel scales are set from
  `Rulers::ism_today_dex` at the reference ring at `Rulers::today_gyr`;
  `relative` is 1 in the solar-birth reference gas; hydrogen and helium follow
  from the founding's counts with `Rulers::dy_dz` and
  `Rulers::reference_metallicity` (`chem::Light`). Status: placeholder; the next
  step is yields from the founding (roadmap, Rules from the founding and the
  galactic cores).
- Placement (`galaxy::star`): the reach is where the rings' cumulative
  holding reaches the silver-turn pivot; the disc phase follows the golden
  turn, the ball planes follow the plastic and golden turns, and the rate
  follows the turning law. Status: built.
- Initial mass function (`galaxy::imf`):
  `m = lo + mb x / (1 - x)` with `x = u (hi - lo) / (hi - lo + mb)`,
  `mb = 1/2 x 2d / (1 + d)` clamped to `[1.01 lo, 0.99 hi]`, `lo = 0.08` and
  `hi = 60`. Status: placeholder.
- Vertical orbit (`galaxy::vertical`, `galaxy::layer_kpc`,
  `galaxy::vertical_frequency`): the vertical orbit uses declared scales.
  Status: placeholder.

## 6. Stars and systems

- Star's rim deficit (`stars::delta_star`): `r_g,sun / (R_sun + r_g,sun)`.
  Status: placeholder (solar rim).
- Ring per bell (`stars::ring_per_bell`): `1 / (pi (r + 1))`. Status: built.
- Burn per bell (`stars::burn_per_bell`): `delta_star / (pi (r + 1))`.
  Status: built.
- Hydrogen step (`stars::transduced_hydrogen`):
  `1 - hold(He) / (4 hold(H)) = 1/4`. Status: built.
- Helium step (`stars::transduced_helium`):
  `h + (1 - h)(1 - hold(C) / (3 hold(He)))`. Status: built.
- Life (`stars::life`): a star lives `(1/4) / burn` bells. The giant end
  (`stars::ends`) falls at `life x transduced_He / transduced_H`, and the
  remnant share (`stars::remnant_share`) is `1 - transduced_He`. Status:
  built.
- Light (`systems::luminosity`, `stars::light`):
  `L = M / (r + 1) (1 + pi bell q(t))`. Status: built.
- Flicker (`stars::flicker`): `(pi / 2) |cos(2 pi t / echo)|`. Status:
  built.
- Star's cut (`disc::star_cut`, `formation::hold_at_heat`): the spin is
  `h w / (R (R + w))`, the heat is `2 ring_per_bell(r)` and
  `p0 = 1 + 2 burn_per_bell(r)`; each kind is carried to
  `l = p p0 (e / e_mean)(w_mean / w) + spin` and is held where
  `h* / (l / h* + h*) >= p`, at the largest fixed point `h*` of the held
  hold. Status: built.
- Disc agitation (`disc::star_cut`):
  `a_disc = min(1, turning x swing x grip / shed)`, with
  `grip = (1 - star) / (2 - star)` and `shed = e_rest + ln 2 / k_v`. Status:
  built.
- Star's manifold (`galaxy::disc_frame`): the gathering is
  `min((mass / stellar_mass) / (cloud / infall), infall)`, the reach is
  `manifold::reach(r, m / M(<r)) = 2 pi r m / (M + m)`, and the star share
  is `min(gathering / reach, 1)`. Status: built.
- Worlds on the manifold (`disc::manifold_worlds`): degree `k` of 360 lies
  at `x = s + (k + 1)(1 - s) / 360`, and each degree adds
  `(1 - a)(1 - s) / 360`; a world forms where `m / (s + m) >= tide x`, with
  its reach at the gathered matter's mean, and the belts hold
  `1 - s - sum worlds`. Status: built.
- Solids (`systems::system`): the solids are
  `M_star / s x Z_rel x Rulers::reference_metallicity` (world masses); a
  region's mass is `holding x solids x enrichment`; regions inside the rock
  line `min Tc(Fe, Si, Mg)` are skipped, and the phase is
  `k x golden turn`. Status: partial (`reference_metallicity`).
- Formation temperature (`systems::Spec::t_form`): `278 L / a`. Status:
  placeholder.
- Envelope (`systems::system`): a world gathers an envelope when its core
  holds at least 10 world masses, by a declared gathering law. Status:
  placeholder.
- Settling rate (`systems::settle_about`): `omega(A) = [M / (A + M)] / A`
  in `r_g`. Status: built.
- Kick (`systems::settle_about`): `eps = [m / (r + m)] / r / |dw|` and
  `N = age |dw|`; `q` is the least continued-fraction denominator with
  `|s - p/q| N q <= 1`, `c = |1 - 2 frac(q psi_0)|`, and the swing is
  `eps (floor(N / q) c + N mod q)`. Status: built.
- Merge (`systems::settle_about`): the merge age is the first age with
  `swing x inner / gap >= 1`, found by 48 halvings to the 30 Gyr horizon;
  merges by the settled age (3 Gyr) combine mass, reach and phase
  mass-weighted, and later ages are recorded on the lighter body
  (`World::merged_at`). Status: built.
- Orbit (`bodies::orbits`): `e0 = swing`; the tilt is `swing / (1 + swing)`
  of a quarter turn, and the node and periapsis take the settled phase.
  Status: built.
- Moon manifold (`systems::moons`): the reach is `2 pi a m / (M + m)` in the
  world's radii from 5; the budget is
  `min(a_disc x (reach / a_out) x solids, belt pool)`; the world share is
  `m / (m + budget)`; the tide is `s / (x + s)` with `x = a / a_out`; moons
  follow the degree law with no heat and settle about the world. Status:
  built.
- Moon forced eccentricity (`systems::moons`): the eccentricity is the
  neighbours' mass over the world's mass, over the distance from the
  nearest first-order resonance, and is at most `1/10`. Status: built.
- Belt (`systems::belts`): the belt is the remaining pool beyond the
  outermost world out to `a_out`; the comet cloud is a declared share of the
  giants' mass. Status: partial.
- Resolution guard (`systems::exp`): `exp(x)` is enclosed by
  `[0, 2^-bits]` for `x <= -(bits + 1)`. Status: built.
- Angle between bodies (`manifold::turn_between`, `chain::Place::arc`):
  `acos(a . b)`. Status: partial; the next step is the versor between frames
  (roadmap, Worlds on the manifold chain).
- Disc-mass scale of the manifold: this scale is not yet built; it is the
  next step (roadmap, Worlds on the manifold chain).
- Degree of a star's life (`worlds::star_degree_gyr`):
  `max(life / 360, bell)`. Status: built.
- Orbital speed (`turning.rs`): `v = c delta(<r)`, which is less than `c` at
  every reach since `delta < 1`. Status: built.

## 7. Life and minds

- Fidelity (`life::fidelity`): `eps0 = exp(-d E_weak / kT)`. Status: built.
- Rung (`life::ladder`): `bits_s = b / eps^(s+1)`, capped at `I_MIND`, with
  at most 12 rungs. Status: built.
- Selection (`life::bits_per_generation`): selection gains
  `min(1, b excess / ln pop)` bits per generation, with `excess = 1/2`.
  Status: built.
- Climb (`life::climb`): the rung time is `(e^(mu m) - 1) / mu` under
  impacts at rate `mu`, and sterilisation falls at `1 / hazard`. Status:
  built.
- Strikes (`life::strikes_per_gyr`):
  `debris x orbits per Gyr x R / (2 pi a)`. Status: built.
- Viability (`life::viable`): a viable people has 500 bodies. Status:
  placeholder.
- Forecast (`minds::schedule`): a mind appears at the last rung where the
  world then feeds a viable people; the first age is one degree of the
  star's life, and the horizon is 30 Gyr (`life::horizon_gyr`). Status:
  built.
- Ancients (`minds::ancients`): the ancients are core-born minds within
  `minds::ancient_window` (the nucleus's first light) of the core's first
  core-born mind. Status: built.
