# Equations

The master equations of Xylarium's physics, formula by formula, in the order
of [model.md](model.md), which explains how each follows from the one before.
Each entry gives the formula, its code location and its status:

- built: derived from the laws above it and read by the engine;
- partial: derived, but resting on a declared input or read only by the
  observer;
- placeholder: a declared constant or calibration standing until a law
  derives it (listed in [roadmap.md](roadmap.md), Declared rulers).

Module paths are relative to `crates/xyl-cosmos/src` unless prefixed with
another crate. Laws are linear in shares and angles; no square or cube
stands at the root of a law.

## 1. Numbers

- `Q`, exact rational (`xyl-core q.rs`): held state. Built.
- `Encl`, certified enclosure `lo <= x <= hi` over `Q`
  (`xyl-core certified.rs`): transcendental values with proven bounds.
  Built.
- `Dy`, dyadic `m 2^e`, 62-bit mantissa, rounded toward zero
  (`xyl-core dyadic.rs`): readings at seams, converted to `Q` on a grid
  before being held. Built.
- Phase grid: a turn rounded to `60^-6` (`circuit::turn`). Built.

## 2. Units and the master frame

- Base units (`xyl-core units.rs`): `G = c = M_core = 1`;
  `r_g = G M_core / c^2` (`Units::of`). Built.
- Tier unit (`Tier::in_rg`): `60^k r_g`, `k` = 10 universe, 7 cluster,
  5 galaxy, 1 system, -2 planet; between tiers
  `dilation(from, to) = 60^(k_from - k_to)` (`manifold::dilation`). Built.
- Bell (`circuit::anchor::of`): `bell = 2 rim 60^5 r_g / c`, the rim read in
  the galaxy tier. Built.
- Ring and echo (`circuit::bell_gyr`, `circuit::echo_gyr`): one ring is one
  bell; `echo = 2 bell`; a one-way crossing to reach `r` takes
  `(r / rim) bell / 2`. Built.
- Ocean speed (`Substrate::c_ocean`): `c_ocean = rim / (bell / 2) = 3600 c`.
  Built.
- Universe radius (`circuit::anchor::of`): `rim / pi` (in kpc and in `r_g`).
  Built.
- First stars (`circuit::anchor::of`): `max(cools_in, 1) bell`. Built.
- SI bridge (`package::hole_mass`): `M_core = 4.3e6` solar masses, fixing
  `r_g` in metres for display in kpc, Gyr, AU and solar or world masses;
  SI constants in `package::Si`. Placeholder; next: the core's mass from
  the package (roadmap, Founding from the package manifest).

## 3. The bounce and the founding

- Deficit law (`circuit::deficit`, `circuit::vacuum_share`,
  `turning::deficit_share`): `delta(R, M) = M / (R + M)`; received share
  `1 - delta`. Built.
- Horizon reach (`bounce::horizon_reach`): `delta_h = 1/3`,
  `R_h = M (1 - delta_h) / delta_h = 2M`. Built.
- Curved reach (`bounce::curved_reach`): `rho sin(r / rho)` on the S^360
  enclosure of radius `rho`. Built.
- Horizon sphere (`bounce::horizon`): `sigma = sphere_through(C, P)`, `P` the
  crossing on isoclinic plane 1 at reach `rho sin(R_h / rho)` and the
  pulse's phase (`bounce::crossing`). Built.
- Spin bivector (`bounce::spin_bivector`): `K = a sum_(k=1..5) B_k`. Built.
- White hole (`bounce::Bounce::of`, `xyl-core cga::Source`):
  `N = P ^ (P . K)`, null, fixing `P`, keeping `sigma`; far face the
  antipode of `P` under the planes' half turn (`circuit::plane_antipole`).
  Built.
- Spin draw (`bang::BigBang::of`): uniform on the 2^-40 grid, strictly
  inside `(0, 1)` (`bang::clamp_spin`). Built.
- Torsion share (`bang::torsion_share`): `a = (H + J) / (2H + J)`, `H = 2`;
  `a = 1/2` at `J = 0`, `3/5` at `J = 1`. Inverse (`bang::spin_of_share`):
  `J = H (2a - 1) / (1 - a)`. Built.
- Admission (`Substrate::least_share`, `cores::substrate_for`):
  `a >= (r_far / rim) (1 + A_in_peak)`; otherwise refound at
  `spin_of_share(least_share)`, at most three foundings; margin
  `a - least_share` (`Substrate::margin`). Built.
- Cascade readings (`bang::BigBang::of`): lumpiness
  `sum max(state - mean, 0) / sum state`; held share
  `firmament mass / holding`; assembly `Lighting::cools_in`; efficiency
  `Lighting::lit`. Built.
- Closed share (`elements::closed_share`): `h_Z = (3Z - u_Z) / 3Z`. Built.
- Capture chain (`elements::founded`):
  `count_1 = 2^b`, `count_Z = floor(count_(Z-1) a h_Z)`. Built.
- Founding bits (`elements::founding_bits`): the least `b` with
  `chain(2^b) >= 1` at the vessel's sealing kind (Z 1092). Built.
- Element table (`elements::table`): Z 1 to 99 measured rows, Z 100 to 150
  predicted rows (`data/elements.tsv`), Z 151 to `TOP_Z` = 1700 generated
  (`elements::systematic_row`); mass number on the valley
  `Z = A / (1.98 + 0.334 e(A / 100))` (`elements::valley_a`). Counts
  built; catalogue columns (mass, melt, boil, density, condensation,
  class) placeholder; next: from the atom.
- Orientations on S^11 (`atoms::orientations`):
  `C(l + 11, 11) - C(l + 9, 11)`; band room twice that. Built.
- Fill (`atoms::atom`): each electron to the tightest open winding,
  tightness `(Z - inside) / n`, winding reach `(n + 1)(l + 1)`. Built.
- Seals (`atoms::Atom::sealed`): Z 2, 28, 208, 1092. Built.
- Vessel (`atoms::vessel`): upper edge the last seal; lower edge
  `max z < upper` with `u(z - 1) + 1 - u(z) < 0` (Z 568 on the current
  table). Built.
- Cascade shares (`formation::founding_shells`):
  `s_Z = 1/2 held / (held + 1 + held)` cumulative by Z on mass;
  `K = sum s_Z`; `H = 1 + held_final`;
  `s_final = 1/2 held_final / (held_final + H)`. Built.
- Shell reach (`formation::founding_shells`): `R_Z = (K + s_i) / w_Z / H`,
  `s_i` the stack's shares, `w_Z = Z + Q`; spread `R (1 +/- s_final)`;
  held where `R (1 + s_final) <= 1`. Built.
- Filter (`formation::founding_shells`): contact `1/2` for a whole passage,
  `1/2` of the share crossed for a partial one;
  `D = M / (M + 1 - contact)`; `T = |d_j - d_i| / (d_j + d_i)`;
  `g = min(s D, T) / T`; take `contact g m_i / (m_i + m_j)` of `m_j`, scaled
  by held over asked; new reach the holding-weighted mean of release
  reaches. Built.
- Kind density in a run (`Substrate::found`):
  `mass / t`, `t = |delta(lo, 1) - delta(hi, 1)| / (delta(lo, 1) + delta(hi, 1))`;
  beads at the local maxima of the superposed hold, parted at its minima.
  Built.
- Bead (`Substrate::found`): state = recipe mass; reach and phase of the most
  abundant kind, phase `turn(a Z)`; inside `1 + 2 sum(inner states / pushed-out mass)`.
  Built.
- Rim (`Substrate::found`): `max R (1 + s_final)` over every shell. Built.

## 4. The firmament and the core

- Firmament (`Substrate::found`): held kinds inside the vessel, spin = reach
  on the 2^-64 grid, phase `turn(a Z)`; amounts scaled by
  `1 / sum mass_of(z, amount)` so the firmament's mass is 1. Built.
- Mass of an amount (`circuit::mass_of`): `amount (Z + Q) / hold_Z`. Built.
- Held share (`Substrate::held_share`): `firmament mass / (firmament mass +
  sum bead masses)`. Built.
- Throat push (`circuit::throat_push`): `p = delta(H, H) = 1/2`. Built.
- Echo dilation (`circuit::echo_dilation`): `rho = (1 + p) / (1 - p) = 3`.
  Built.
- Reverberation (`Substrate::reverberation_with`): in order of spin,
  `h = 1 + inside / unit`, `r = spin / h`, `delta = h / (r + h)`,
  `beyond = clamp((r (1 + delta) - h) / (2 delta r), 0, 1)`,
  `rate = beyond / bell`. Built.
- Jet onset (`circuit::onset_echoes`): `ln(1 / t) / ln rho` echoes,
  `t = open / hold`; none for a sealed kind; zero where `t >= 1`. Built.
- Outflow (`Substrate::outflow`): `m(t) = m0 (1 - exp(-rate (t - onset)))`
  for `t > onset`. Built.
- Outflow rate (`Substrate::rate`):
  `q(t) = sum m0 (1 + e_z) rate exp(-rate (t - onset))`,
  `e_z = open_z / hold_z` (`formation::emission`). Built.
- Inbound echo (`Substrate::inbound`): `A_in(t) = cos(2 pi t / echo) + echo q(t)`.
  Built.
- Inbound peak (`Substrate::inbound_peak`): `1 + echo max_j q(onset_j)`.
  Built.
- Inbound mean (`Substrate::inbound_mean`):
  `echo (sin(2 pi t / echo) / 2 pi + Q(t)) / t`, `Q` the state out by `t`.
  Built.
- Push at reach (`Substrate::push_at`): `p(R, t) = h / (R + h) (1 + A_in(t))`.
  Built.
- Band past the throat at `t` (`Substrate::beyond_at`): `beyond` with the
  deficit replaced by `p(r, t)`. Built.
- Pulse phase (`Substrate::pulse`): `frac((t / echo) / t_u)`,
  `t_u = floor(2 (rim + 2)) + 1` echoes. Built.
- Jetstream (`Substrate::loop_flux`, `circuit::pole_flux`): channels on a
  kind of hold `w` and open frames `u`: filament
  `(rho - 1) u w / (w + rho u)` frames Z to Q; vacuum `p(r, t) w` frames
  Z to Q (none where `p >= 1`); echo `R u` frames Q to Z. Each is a dilator
  on the state it finds; residual = observed dilation over the channels'
  product, one where the loop closes. Partial (read by the observer).
- Waveform (`Substrate::amplitude`):
  `A(r, t) = [cos(2 pi (t - tau) / echo) / echo + q(t - tau)] / (r + 1)`,
  `tau = (r / rim) bell / 2`. Built.
- Pull and delay (`Substrate::ocean_shares`): pull `h / (r + h)`,
  `h = state / mean state` (home galaxy 1 at reach 0); share = pull over the
  sum; delay `tau`. Built.
- Coherence (`circuit::coherence`): `1 - 2 min(d, 1 - d)`, `d` the phase
  difference in turns. Built.
- Landing (`Substrate::landing`): share = pull x coherence over the sum
  (pull alone where every coherence is zero). Built.
- Intake and composition (`Substrate::intakes`, `Substrate::composition`):
  intake = landing share x `outflow(t - tau)`; composition = recipe + intake.
  Built.
- Books (`Substrate::books`): held + left = founded; taken + in flight =
  left. Built.
- Push on a galaxy (`Substrate::push`):
  `l(t) = R inside + d [Q(t - tau) + sin(2 pi (t - tau) / echo) / 2 pi] / ((R + 1) H)`,
  `reach = l / inside`; recession `dl/dt / inside`; shift
  `z = recession / c_ocean`. Built.
- Tide (`Substrate::tide`): `A(R - x, t) - A(R + x, t)`. Built.
- Galaxy selection (`Substrate::holds_a_star`):
  `lit x hole_mass x state >= 1` (solar masses). Built.
- Galactic core mass (`cores::founding_of`): `hole_suns = hole_mass x state`.
  Built.
- Circulation (`cores::founding_of`): `4 delta / echo` turns per Gyr,
  `delta = inside / (reach + inside)`. Built.
- Galaxy plane (`cores::founding_of`): node at `index x golden turn`, tilt
  holding `index x plastic turn` of the S^360 measure (`galaxy::node_of`,
  `galaxy::tilt_of`). Built.
- Galaxy scale (`cores::Shape::scale`): `2 m / (1 + m)`,
  `m = hole_suns / hole_mass`. Placeholder.

### Z, Q and R (the pole model)

- Weights (`circuit::nucleus_weights`, `circuit::mix_weights`): `Z` the
  protons, `Q` the neutrons (mass number less charge, from the element
  table; a cluster sums its parts); a kind weighs `Z + Q` nucleons. Built.
- Electron phase state (`circuit::electron_phase`):
  `R = kappa (1 - 2 frac(p))` in `[-kappa, kappa]`, `p` the bounce's pulse
  in turns, `kappa` the electrons' coherence on S^11 (`fibres`, `2 G / ends`
  for the charge's electrons); zero for a kind with no charge. Built.
- Pole point (`circuit::pole_point`, `circuit/poles.rs`): `X(t)` on the pole
  axis at `t = open / hold`; Z pole the origin `e_o`, Q pole the point at
  infinity `e_inf`, throat at `t = 1`; inversion `X(t) -> X(1 / t)`
  (`circuit::axis_antipole`). Built.
- Axes (`circuit/axes.rs`): the pulse has one fixed axis, the pole axis
  (base axis 1), and five isoclinic planes on base axes 2-3, 4-5, 6-7, 8-9
  and 10-11 (`circuit::plane_blade`); the planes' half turn
  (`circuit::plane_antipole`) sends a point to its antipode, and with the
  axis's inversion forms the pulse's antipole (`circuit::antipole`).
  Antinode share of harmonic `q`: 0 for even `q`, `1/q` for odd `q`
  (`circuit::antinode`). Built.

## 5. The galaxy

- Structure (`galaxy::Structure`): rings = beads sorted by reach, edges
  `span x galaxy_kpc`, per-kpc holding `state / width`; core reach
  one galaxy unit; nucleus the first ring's outer edge; reference ring where
  the cumulative holding crosses one half. Built.
- Crowding (`galaxy::density`): `lambda_ring / lambda_reference`. Built.
- Stellar mass (`galaxy::stellar_mass`): `Lighting::lit x hole_mass`. Built.
- Mass inside (`galaxy::inside_mass`):
  `hole_mass (1 + rings within r / all rings)`, a ring counted by the share
  of its span inside. Built.
- Turning law (`turning.rs`): `omega = c delta(<r) / r`,
  `delta(<r) = M(<r) / (r + M(<r))` in `r_g`. Built.
- Piece of gas (`formation::cooling`): `lambda = (state / unit) / W`;
  agitation `a0 = (p - delta) / p`, `delta = h / (R + h)`, `h = 1 + held share`;
  `e = sum s_z e_z`; `mu = lambda e`; `n = 1` beyond one galaxy ring,
  else 2. Built.
- Gathering (`formation::gathered`): `m(w) = (lambda + mu) W share_n(theta)`,
  `theta = w / (2 (w + R))` turns (`xyl-core measure::share_turns`). Built.
- Grip: `m / (w + m)`. Shear: `h w / (R (R + w))`. Built.
- Vacuum halving (`formation::vacuum_halving`): least whole
  `k >= ln 2 / ln((R + h) / R)`. Built.
- Heat (`formation::collapse`):
  `min(1, ring x swing x grip(W) / shed)`, `ring = 1 / (pi (R + 1))`,
  `shed = e + ln 2 / k_v`. Built.
- Spare (`formation::collapse`): `grip(w) - shear(w) - heat` on
  `w = W / 2^k`, `k < 48`. Built.
- Cooling (`formation::collapse`): halvings until `a <= max spare`;
  `cools_in` the least ring `n` with
  `floor(n / P_e) + floor(n / k_v) >= halvings`, `P_e = ceil(1 / e)`;
  infall the largest `w` with `spare(w) >= a`; clouds
  `max(1, floor(W / infall))`. Built.
- Lighting (`formation::lights`, `formation::lighting`): lights where
  clouds > 0 and `cooled + heat < 1/2`; `cools_in` the least over lit
  pieces; `lit` = lit holding over all holding. Built.
- Reach in galaxy rings (`stars::reach_rings`): `r_kpc / galaxy_kpc`.
  Built.
- First light (`galaxy::birth_gyr`): `max(cools_in, 1) bell` for the piece
  at the star's reach; none where it never lights. Built.
- Birth along the ring (`chem::Chemistry::birth_along`): `t` with
  `F(t) = F(t0) + u (F(top) - F(t0))`, `top` the 30 Gyr horizon, found by
  48 halvings; `F` the ring's `sfr` integrated term by term:
  `c p! / k^(p+1) (1 - e^(-kt) sum_(j<=p) (kt)^j / j!)`, or
  `c t^(p+1) / (p + 1)` at `k = 0` (`chem::formed_by`). Built.
- Star formation per ring (`chem::Model::of`): `efficiency x Omega / 2 pi`,
  `Omega = v_flat / r`; gas a sum of `c t^m e^(-lambda t)` with infall on the
  ring's first-light timescale toward `galaxy::density x sigma_r0`. Partial
  (rulers `v_flat`, `sigma_r0`, `prompt`, `slow`, `loading`, ring width).
- Yields (`chem::Calibration`): per-channel scales from
  `Rulers::ism_today_dex` at the reference ring at `Rulers::today_gyr`;
  `relative` is 1 in the solar-birth reference gas; hydrogen and helium from
  the founding's counts with `Rulers::dy_dz` and `Rulers::reference_metallicity`
  (`chem::Light`). Placeholder; next: yields from the founding (roadmap,
  Founding from the package manifest).
- Placement (`galaxy::star`): reach where the rings' cumulative holding
  reaches the silver-turn pivot; disc phase by the golden turn; ball
  planes by the plastic and golden turns; rate by the turning law. Built.
- Initial mass function (`galaxy::imf`):
  `m = lo + mb x / (1 - x)`, `x = u (hi - lo) / (hi - lo + mb)`,
  `mb = 1/2 x 2d / (1 + d)`, `lo = 0.08`, `hi = 60`. Placeholder.
- Vertical orbit (`galaxy::vertical`, `galaxy::layer_kpc`,
  `galaxy::vertical_frequency`): declared scales. Placeholder.

## 6. Stars and systems

- Star's rim deficit (`stars::delta_star`): `r_g,sun / (R_sun + r_g,sun)`.
  Placeholder (solar rim).
- Ring per bell (`stars::ring_per_bell`): `1 / (pi (r + 1))`. Built.
- Burn per bell (`stars::burn_per_bell`): `delta_star / (pi (r + 1))`. Built.
- Hydrogen step (`stars::transduced_hydrogen`): `1 - hold(He) / (4 hold(H))
  = 1/4`. Built.
- Helium step (`stars::transduced_helium`):
  `h + (1 - h)(1 - hold(C) / (3 hold(He)))`. Built.
- Life (`stars::life`): `(1/4) / burn` bells. Giant end (`stars::ends`):
  `life x transduced_He / transduced_H`. Remnant (`stars::remnant_share`):
  `1 - transduced_He`. Built.
- Light (`systems::luminosity`, `stars::light`):
  `L = M / (r + 1) (1 + pi bell q(t))`. Built.
- Flicker (`stars::flicker`): `(pi / 2) |cos(2 pi t / echo)|`. Built.
- Star's cut (`disc::star_cut`, `formation::hold_at_heat`): spin
  `h w / (R (R + w))`; heat `2 ring_per_bell(r)`; `p0 = 1 + 2 burn_per_bell(r)`;
  each kind carried to `l = p p0 (e / e_mean)(w_mean / w) + spin`, held where
  `h* / (l / h* + h*) >= p` at the largest fixed point `h*` of the held
  hold. Built.
- Disc agitation (`disc::star_cut`):
  `a_disc = min(1, turning x swing x grip / shed)`,
  `grip = (1 - star) / (2 - star)`, `shed = e_rest + ln 2 / k_v`. Built.
- Star's manifold (`galaxy::disc_frame`): gathering
  `min((mass / stellar_mass) / (cloud / infall), infall)`; reach
  `manifold::reach(r, m / M(<r)) = 2 pi r m / (M + m)`; star share
  `min(gathering / reach, 1)`. Built.
- Worlds on the manifold (`disc::manifold_worlds`): degree `k` of 360 at
  `x = s + (k + 1)(1 - s) / 360`; each adds `(1 - a)(1 - s) / 360`; a world
  where `m / (s + m) >= tide x`, reach the gathered matter's mean; belts
  `1 - s - sum worlds`. Built.
- Solids (`systems::system`): `M_star / s x Z_rel x Rulers::reference_metallicity` (world
  masses); a region's mass `holding x solids x enrichment`; skipped inside
  the rock line `min Tc(Fe, Si, Mg)`; phase `k x golden turn`. Partial
  (`reference_metallicity`).
- Formation temperature (`systems::Spec::t_form`): `278 L / a`. Placeholder.
- Envelope (`systems::system`): core `>= 10` world masses; declared
  gathering law. Placeholder.
- Settling rate (`systems::settle_about`): `omega(A) = [M / (A + M)] / A`
  in `r_g`. Built.
- Kick (`systems::settle_about`): `eps = [m / (r + m)] / r / |dw|`;
  `N = age |dw|`; `q` the least continued-fraction denominator with
  `|s - p/q| N q <= 1`; `c = |1 - 2 frac(q psi_0)|`;
  swing `eps (floor(N / q) c + N mod q)`. Built.
- Merge (`systems::settle_about`): first age with
  `swing x inner / gap >= 1`, 48 halvings to the 30 Gyr horizon; merges by
  the settled age (3 Gyr) combine mass, reach and phase mass-weighted;
  later ages recorded on the lighter body (`World::merged_at`). Built.
- Orbit (`bodies::orbits`): `e0 = swing`; tilt `swing / (1 + swing)` of a
  quarter turn; node and periapsis the settled phase. Built.
- Moon manifold (`systems::moons`): reach `2 pi a m / (M + m)` in the
  world's radii from 5; budget
  `min(a_disc x (reach / a_out) x solids, belt pool)`; world share
  `m / (m + budget)`; tide `s / (x + s)`, `x = a / a_out`; degree law with
  no heat; settled about the world. Built.
- Moon forced eccentricity (`systems::moons`): neighbours' mass over the
  world's over the distance from the nearest first-order resonance, at most
  `1/10`. Built.
- Belt (`systems::belts`): the remaining pool beyond the outermost world to
  `a_out`; comet cloud a declared share of the giants' mass. Partial.
- Resolution guard (`systems::exp`): `exp(x)` enclosed by `[0, 2^-bits]`
  for `x <= -(bits + 1)`. Built.
- Angle between bodies (`manifold::turn_between`, `chain::Place::arc`):
  `acos(a . b)`. Partial; next: the versor between frames (roadmap,
  Worlds on the manifold chain).
- Disc-mass scale of the manifold: next (roadmap, Worlds on the manifold
  chain).
- Degree of a star's life (`worlds::star_degree_gyr`):
  `max(life / 360, bell)`. Built.
- Orbital speed (`turning.rs`): `v = c delta(<r)`, below light at every
  reach. Built.

## 7. Life and minds

- Fidelity (`life::fidelity`): `eps0 = exp(-d E_weak / kT)`. Built.
- Rung (`life::ladder`): `bits_s = b / eps^(s+1)`, capped at `I_MIND`,
  at most 12 rungs. Built.
- Selection (`life::bits_per_generation`):
  `min(1, b excess / ln pop)` bits per generation, `excess = 1/2`. Built.
- Climb (`life::climb`): rung time `(e^(mu m) - 1) / mu` under impacts at
  rate `mu`; sterilisation at `1 / hazard`. Built.
- Strikes (`life::strikes_per_gyr`): `debris x orbits per Gyr x R / (2 pi a)`.
  Built.
- Viability (`life::viable`): 500 bodies. Placeholder.
- Forecast (`minds::schedule`): a mind at the last rung where the world
  then feeds a viable people; first age one degree of the star's life;
  horizon 30 Gyr (`life::horizon_gyr`). Built.
- Ancients (`minds::ancients`): core-born minds within
  `minds::ancient_window` (the nucleus's first light) of the core's first
  core-born mind. Built.
