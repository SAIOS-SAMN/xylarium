# Equations

This document lists Xylarium's physics equations, formula by formula, in the
order of [model.md](model.md), which explains how each follows from the one
before. Terms are defined in [glossary.md](glossary.md).
Each entry gives the formula, its code location and its status. The status
is one of four values:

- Built: the formula is derived from the laws above it and read by the
  engine.
- Partial: the formula is derived, but it rests on a declared input or is
  read only by the observer.
- Planned: the formula is specified in [model.md](model.md) and scheduled
  in [roadmap.md](roadmap.md); it replaces the entries marked "replaced".
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
- Geometric product (`xyl_core::cga::Mv::geo`), in `Cl(p, q)` with the
  signature `NEG` (the generators squaring to `-1`) carried by the type
  `Mv<NEG>`: `e_A e_B = (-1)^(r + |A & B & NEG|) e_(A xor B)`, where `r`, the
  parity of the pairs of a generator of `B` below one of `A`, is
  `|B & S(A)| mod 2` with `S(A)` the word folded by doubling shifts; no walk
  over generators. `Mv` is the conformal `Mv<EM>`, `Cl(n+1, 1)`; two
  signatures never compose (the product does not compile). The four
  Concordance 2.11 fixtures hold in `Cl(3, 2)`. Status: built.
- Capture phase (`circuit::found`): `(share Z) mod 1`, exact; its
  denominator divides the torsion share's. Status: built.
- Coordinate (`xyl_core::coordinate`, LAW-COORDINATE v1): a lane is an
  exact rational of either sign; its sector at depth `d` is
  `[m 60^-d, (m + 1) 60^-d)` with `m = floor(v 60^d)`, exact and nested;
  `Reading::within` reads Below, Within or Above on the sector's exact
  bounds and refuses readings on two lanes; places (`2;20`, `-1;40`) and
  `coordinate::beat` (whole rings and the phase in degrees) are display
  only. Status: built.

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
- SI bridge (`package::hole_mass`): the core's mass in SI units, declared by the
  package, which fixes `r_g` in metres for display in kpc, Gyr, AU and
  kilograms; the SI constants are in `package::Si`. Status: placeholder; the
  next step is the core's mass from the package (roadmap, Rules from the
  founding and the galactic cores).

## 3. The bounce and the founding

- Deficit law (`xyl_core::shares::deficit`, `deficit_dy`):
  `delta(R, M) = M / (R + M)`, and the received share is `1 - delta`.
  Status: built.
- Dilation and flow law (`xyl_core::shares::dilation`, `flow`):
  `D(p) = (1 + p) / (1 - p)` for `-1 < p < 1`; `x (+) y = (x + y) / (1 + x
  y)`; `D(x (+) y) = D(x) D(y)`. Status: built.
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
  antipode of `P` under 180 degrees on each isoclinic plane
  (`circuit::plane_antipole`); its return is the next bounce's, at the
  horizon's deficit, one third.
  Status: built.
- Spin draw (`origin::Origin::of`): the spin is drawn uniformly on the 2^-40
  grid, strictly inside `(0, 1)` (`origin::clamp_spin`). Status: built.
- Torsion share (`bounce::torsion_share`), one law for every throat:
  `a = (H + J) / (2H + J)` with `H = 2M` and `J = a_Kerr M^2`; at the
  universal core (`M = 1`) `a = 1/2` at zero spin and `a = 3/5` at the
  extremal spin. The inverse (`bounce::spin_of_share`) is
  `J = H (2a - 1) / (1 - a)`, put on the origin's grid by
  `origin::clamp_spin`. The founding reads the share once
  (`circuit::Substrate::found`) and holds it as `Substrate::share`. Status: built.
- Bond (`atoms::Bond`; docs/model.md, Scalar collapses; #107): two faces,
  the two atoms whole. Axis plane: each face's pole push
  `p = (U - W) / (U + W)`, `W = 3Z - U` (`Bond::pole_pushes`), their
  separation `p_a (+) (-p_b)` by the flow law (`Bond::axis_separation`),
  each face's polarity (`Bond::polarities`). Isoclinic planes: each open
  winding's tightness `H / n` (`Atom::winding_tightness`), its push and its
  departure (T20, `Bond::held`). Status: built.
- Stripping (T20; `atoms::Atom::winding_push`, `atoms::departure`,
  `atoms::unpaired_hot`): a winding `(n, l)` encloses
  `H = Z - (electrons of every other shell m with m (m + 1) < (n + 1)(l + 1))`
  (`atoms::shielded`) at reach `r = Z (n + 1)(l + 1) / 2`, and pushes
  `delta_w = H / (r + H)`; under a received isoclinic push `q` it is held
  while `q <= delta_w`, else it departs after `ln 2 / ln(D(q) / D(delta_w))`
  echoes, at once for `q >= 1`. Hydrogen 1s: 1/2; oxygen (2, 1): 1/5.
  Status: built.
- Echo (`circuit::echo_at`; docs/model.md, The ring, the throat orbit and
  the echo's torsion; T4, axiom 7): a throat at torsion share `s` echoes
  after `bell / (1 - s)` on its near face and `bell / s` on its far face;
  two bells at zero spin. The universal core's founding reads its founded
  share (`Substrate::echo`); every tier after its bounce reads the share of
  its released spin (`circuit::echo`). Status: built.
- Received turning (`atoms::Received`; docs/model.md, Chemistry from the
  elements; #103): `{ axis, isoclinic }`, the axis plane's push per echo
  (the pole state's dilation, a star's compactness) and the isoclinic
  planes' turning per echo in hydrogen units (stripping:
  `atoms::unpaired_hot` strips windings of tightness at most
  `min(isoclinic, 1)`; a bond holds `min(1, grip / isoclinic)`,
  `compound::held`). `stars::Sequence::of` reads the steps on the isoclinic
  planes and the throat climb on the axis plane. Status: built.
- Link remanence (docs/model.md, Phase weakening and phase strengthening;
  T19, #84): `w = J - (a_0 M_0) M`; per passage
  `w' = w (1 - m / M) + J (m / M) (1 - rho_c / rho_hold)`; conserved between
  passages; coercivity `m* = w M / (w + J (rho_c / rho_hold - 1))`; for one
  release, the remanent share `1 - rho_hold / rho_closed`. Seed 7: 0.1982,
  70.88 of `360 a* = 357.59` degrees per unit hold in the core's units.
  Status: built (`circuit::SpinOrbit::remanence`; read by
  `xylarium observe universal-core`).
- Automorphism of the chain (docs/model.md, The automorphic, synaptic
  chain; #97): `M / (r + M)` is of degree zero in length, so
  `deficit(lambda M, lambda r) = deficit(M, r)` and the flow law of the two
  pushes is unchanged for every dilation `lambda > 0`; the kernel's tier
  ladder `60^p` (`units::Tier`, a declared ruler) is one instance. Status: built at the law
  (`shares::tests::deficit_is_invariant_under_dilation`); the chain's
  links are planned.
- Isoclinic frame (docs/model.md, The frame, the place and the chain; T18,
  #90): `H_j = H_iso / 5` and `J_j = J / 5` on each of the five planes, so
  T14's push `J_j / (6 H_j + J_j)` is the same on all five: one isoclinic
  phase per body. A frame is the axis-plane phase, the isoclinic phase and
  the turning between planes into its parent. Status: derived; not built.
- Paired throat (docs/model.md, The core as a paired throat; #86): near
  face `D(s) D(n)^A`, far face `D(s) D(n)^-A`, product `(3 + a)^2`, so the
  shared hold is the closed kinds `M_c` (T16). The spin's orbit (T17): a
  kind's separation across the attractor pair is
  `rho = q ln D(s) - (1 - q) ln D(1 - s)`, `q = U / (U + W)`,
  `D(s) = 3 + a`, `D(1 - s) = (6 + a) / (2 + a)` (the far face exchanges
  `q` and `1 - q`); per cohort `J' = J - J (m / M) (rho_cohort / rho_hold)`,
  `M' = M - m`, `a' = J' / M'^2`, at the core's bounces `N* = 3 (4 + a) /
  (2a)` echoes apart;
  the pair closes while `a <= 1`. Seed 7: one cohort, 0.5648 of the hold at
  `rho = -0.646` against `-0.798`, giving `a* = 0.9933`. Status: built
  (`circuit::Substrate::spin_orbit`, `atoms::axis_polarity`,
  `circuit::Anchor::core_spin`; the firmament is the closed kinds).
- Cascade readings (`origin::Origin::of`): the lumpiness is
  `sum max(state - mean, 0) / sum state`, the held share is
  `firmament mass / holding`, the assembly time is `Lighting::cools_in` and
  the efficiency is `Lighting::lit`. Status: built.
- Closed share (`elements::closed_share`): `h_Z = hold_Z / 3Z = (3Z - u_Z) / 3Z`
  (`compound::hold`). Status:
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
- Orientations on S^11 (`fibres::directions`): a band has
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
  `s_final = 1/2 held_final / (held_final + H)`. Status: built; replaced by the founding as flow (planned).
- Shell reach (`formation::founding_shells`): `R_Z = (K + s_i) / w_Z / H`,
  with `s_i` the stack's shares and `w_Z = Z + Q`; the spread is
  `R (1 +/- s_final)`, and a shell is held where `R (1 + s_final) <= 1`.
  Status: built; replaced by the founding as flow (planned).
- Filter (`formation::founding_shells`): the contact is `1/2` for a whole
  passage and `1/2` of the share crossed for a partial one;
  `D = M / (M + 1 - contact)`, `T = |d_j - d_i| / (d_j + d_i)` and
  `g = min(s D, T) / T`. The filter takes `contact g m_i / (m_i + m_j)` of
  `m_j`, scaled by held over asked, and the new reach is the
  holding-weighted mean of the release reaches. Status: built; replaced by the founding as flow (planned).
- Kind density in a run (`Substrate::found`): the density is `mass / t`,
  with `t = |delta(lo, 1) - delta(hi, 1)| / (delta(lo, 1) + delta(hi, 1))`;
  beads lie at the local maxima of the superposed hold and are parted at its
  minima. Status: built; replaced by the founding as flow (planned).
- Bead (`Substrate::found`): the state is the recipe mass; the reach and
  phase are those of the most abundant kind, with phase `360 frac(a Z)`
  degrees on its plane (`turn(a Z)` in code); and the
  inside is `1 + 2 sum(inner states / pushed-out mass)`. Status: built; replaced by the founding as flow (planned).
- Rim (`Substrate::found`): the rim is `max R (1 + s_final)` over every
  shell. Status: built; replaced by the founding as flow (planned).

- The founding as flow ([model.md](model.md#the-founding-as-flow)).
  Status: planned.
  - Release: kind `k` leaves at `circuit::onset`, at
    `m0 (1 - exp(-rate (t - onset)))`, from `R_h(H_0) = 2 H_0`, with
    `H_0(t) = 1 - sum_k outflow_k(t)`.
  - Pushes: `p_c = H / (r + H)` (continuum, outward) and
    `p_v = m / (r + m)` (vacuum, inward), composed by the flow law
    `x (+) y = (x + y) / (1 + x y)`: `p_net = (H - m) / (r + H + m)`, and
    `D(p_net) = (r + R_h(H)) / (r + R_h(m))`.
  - Reach rate: `dr/dtau = [R_h(H) - R_h(m)] r / (r + R_h(m))`, in echoes;
    with `H` and `m` fixed,
    `tau - tau0 = [(r - r0) + R_h(m) ln(r / r0)] / (R_h(H) - R_h(m))`.
  - Angular state: `dphi/dt = delta(r, H) bell / r` in degrees; swing as
    torsion and twist on phase and plane.
  - Interaction: coherence `c = 1 - 2d` in `[0, 1]`; a meeting exchanges by
    the deficit contrast `T` times `c`, with `sum m l` conserved.
  - Perturbation: the held share `m / (r + m)` of the echo on the parcel's
    frames (`loop_flux` vacuum channel), never on its reach.

## 4. The firmament and the core

- The ring: the throat orbit and the echo's torsion
  ([model.md](model.md#the-ring-the-throat-orbit-and-the-echos-torsion),
  [#60](https://github.com/SAIOS-SAMN/xylarium/issues/60)). Status: planned.
  - Orbit about `M` at reach `r`: `v = delta = M / (r + M)`; ellipse with
    `a = r`, `e = delta`, `r_p = r (1 - delta)`, `r_a = r (1 + delta)`;
    `r_a / r_p = (1 + delta) / (1 - delta) = 1 + 2M / r`.
  - Shear: periapsis advance `360 delta` degrees per traversal in the
    orbital plane (a degree of structure); return after
    `q` traversals, `q` the denominator of the advance; resonances
    `r_k = (k - 1) M` return after `k`; far-face images `M / (k - 1)`.
  - Ring: the throat orbit's reading at count `t`, `R(t) = (D(n)^t by its
    exponent; phi_i(t) = t theta_i mod 1 in Q/Z, i = 1..5; whole
    traversals in Z and each plane's order)`, never one scalar; durations
    (`2 pi M / s` per traversal, in `r_g / c`) are display through a
    certified pi. Turning in degrees per traversal on the orbit's plane:
    `360 (delta(r) / r) (M / s)`,
    exact.
  - Local ring at reach `r`: one traversal of the orbit at `r`, lasting
    `s r (r + M) / M^2` core traversals (exact; pi cancels); every law step
    at a reach counts local traversals.
  - Twist and dipole, spin `a = m / N`, lowest common denominator
    `L = 4N + m`:

    ```text
    s   = (2N + m) / L            torsion share (open, Q)
    n   = m / L                   twist per echo, 2s - 1 = a / (4 + a)
    Q   = (2N + m) / L            (1 + n) / 2
    Z   = 2N / L                  (1 - n) / 2
    Q/Z = (2N + m) / 2N           (1 + n) / (1 - n) = 1 + a / 2
    ```

  - Influence, over `L^2` and powers of two:

    ```text
    dn/da    =  4N^2 / L^2
    dQ/da    =  2N^2 / L^2  = -dZ/da
    d(Q/Z)/da = 1/2
    dD(n)/dn =  L^2 / 8N^2
    d delta/dr = -M / (r + M)^2,   dD(delta)/dr = -2M / r^2
    ```

  - Seed 7 (`N = 2^40`, `m = 381123025677`,
    `L = 4779169536781`, the denominator of the founding's torsion share):

    | Reading | Numerator over `L` |
    |---|---|
    | `n` | `381123025677` |
    | `Q` | `2580146281229` |
    | `Z` | `2199023255552` (`2^41`) |

    `Q/Z = 2580146281229 / 2^41`; the core's full return is `L` rings;
    over `L^2 = 22840461461295518111841961`: `dn/da = 2^82 / L^2`,
    `dQ/da = 2^81 / L^2`; `dD(n)/dn = L^2 / 2^83`.


- Firmament (`Substrate::found`): the firmament holds the held kinds inside
  the vessel, with spin equal to reach on the 2^-64 grid and phase
  `360 frac(a Z)` degrees on its plane (`turn(a Z)` in code); amounts are
  scaled by `1 / sum mass_of(z, amount)` so that the firmament's mass is 1. Status: built.
- Mass of an amount (`circuit::mass_of`): `amount (Z + Q) / hold_Z`.
  Status: built.
- Held share (`Substrate::held_share`): the held share is
  `firmament mass / (firmament mass + sum bead masses)`. Status: built.
- Throat push (`circuit::throat_push`): `p = delta(H, H) = 1/2`. Status:
  built.
- Dilation per echo (`xyl_core::shares::dilation_dy`), one law for every holder:
  `rho = (1 + p) / (1 - p)`, `p` the push the holder receives. Status:
  built.
- Jet dilation (`shares::dilation_dy` of the torsion share): the continuum carries the
  throat's torsion share `s = (H + J) / (2H + J)` at each crossing, so
  `rho = D(s) = (1 + s) / (1 - s)`: 3 at `s = 1/2` (no spin), 4 at
  `s = 3/5` (the universal core's extremal spin). Status: built.
- Echoes to the throat (`circuit::echoes_to_throat`), one law for the
  firmament's kinds and a star's collective: `n = ln(1 / t) / ln rho`, zero
  where `t >= 1`, none where `t <= 0` or `rho <= 1`. Status: built.
- Reverberation (`Substrate::reverberation`): in order of spin,
  `h = 1 + inside / unit`, `r = spin / h`, `delta = h / (r + h)`,
  `beyond = clamp((r (1 + delta) - h) / (2 delta r), 0, 1)` and
  `rate = beyond / bell`. Status: built.
- The equator: inflow, viscosity and pairs
  ([model.md](model.md#the-equator-inflow-viscosity-and-pairs), [#60](https://github.com/SAIOS-SAMN/xylarium/issues/60)).
  Status: planned (derived).
  - Continuity: `s (+) (-s) = 0`, so `D(s) D(-s) = 1`; `D(s) = 3 + a`;
    equatorial inflow per echo `1 - 1 / D(s) = (2 + a) / (3 + a)`;
    `Z / Q = 1 / D(n)`.
  - Swirl: `Gamma(r) = 2 pi M r / (r + M)`, the passed share.
  - Viscosity: `nu(t) = t / (1 + t)`, `t = U / W`; reverted
    `nu(1 / t) = 1 - nu(t)`; `nu(1) = 1/2`.
  - Positron: `R(-x) = -R(x)`, `R = kappa (1 - 2x)`, `x` in Q/Z.
  - Pair share per echo: `p_pair = s (+) (-(1 - s)) = 4n / (3 + n^2) =
    a (4 + a) / (12 + 6a + a^2)`; `D(p_pair) = (3 + a)(2 + a) / (6 + a)`;
    `p_pair / 2` electrons (held face) and `p_pair / 2` positrons (open
    face); 0 spinless, `5/19` at `a = 1`.
  - Over `a = m / N`: `D(s) = (3N + m) / N`; inflow `(2N + m) / (3N + m)`;
    `p_pair = m (4N + m) / (12N^2 + 6Nm + m^2)`;
    `D(p_pair) = (3N + m)(2N + m) / (N (6N + m))`.
  - Seed 7 (`N = 2^40`, `m = 381123025677`): `D(s) = 3679657909005 / 2^40`;
    inflow `2580146281229 / 3679657909005`;
    `p_pair = 607150518027107086308579 / 5722219928715645461402851`;
    `D(p_pair) = 3164685223371376273855715 / 2557534705344269187547136`.

- Weights on the six planes
  ([model.md](model.md#weights-on-the-six-planes), [#60](https://github.com/SAIOS-SAMN/xylarium/issues/60)).
  - Multiplicity: `mult_l(m) = C(q + 4, 4)`, `q = (l - sum |mi|) / 2`;
    `M_l(m0) = C(l - |m0| + 9, 9)`; `sum_m mult_l(m) =
    sum_m0 M_l(m0) = orientations(l)`. Status: built (test).
  - Flow at count `t`: axis `D(n)^(m0 t)`; phase
    `t s (m1 + ... + m5) mod 1`. Status: planned.
  - Filling within a band: `m0` ascending (Z side first), singly before
    pairing; unpaired on the Z side below half filling, on the Q side
    above. Status: planned.
  - Axis polarity: `sum m0` over the unpaired electrons; founded Z 1 to
    1092: 535 Z side, 517 Q side, 40 neutral. Status: planned.

- The antipoles, the shear and the pinch
  ([model.md](model.md#the-antipoles-the-shear-and-the-pinch), [#60](https://github.com/SAIOS-SAMN/xylarium/issues/60)).
  Status: planned (derived; checked exactly).
  - Antipole group: `m`, `(-m0, m1..m5)`, `(m0, -m1..-m5)`, `-m`; each
    keeps `sum |mi|` and the multiplicity.
  - Flip `c -> -c`: multiplicity equal on `(m0, c)` and `(m0, -c)` for
    `l <= 18`; axis levels `C(l - |m0| + 9, 9)` kept; 1092 of 1092 founded
    kinds keep `U` and `A`.
  - Antipode deviation: planes `c n` per traversal; axis
    `D(s)^m0 D(1 - s)^-m0 = D(p_pair)^m0`.
  - Reconciliation: `p_pair = n (+) (n / 3)`; `D(p_pair) = D(n) D(n / 3)`;
    `D(n / 3) = (6 + 2a) / (6 + a)`; flip share per pair per echo
    `n / 3 = a / (3 (4 + a))`, over `a = m / N`: `m / (3 (4N + m))`.
  - Linking: per echo `dTw = n/2 - n/2`, `dWr = n/3 - n/3`; relative
    writhe `2 N n / 3`, first whole at `N*`.
  - Pinch: `N* = 3 / (2n) = 3 (4 + a) / (2a)` echoes, at 180 degrees on the isoclinic planes.
  - Seed 7: `n / 3 = 127041008559 / 4779169536781`;
    `N* = 4779169536781 / 254082017118` echoes;
    `2 N* = 4779169536781 / 127041008559` rings, ring 37 at
    `222 deg 52' 1911753317040/42347002853"` on the T lane; spinless: none.

- What a pinch carries
  ([model.md](model.md#what-a-pinch-carries-the-next-throat-and-its-pressure),
  [#60](https://github.com/SAIOS-SAMN/xylarium/issues/60)). Status: planned (derived; checked exactly).
  - Spin of the pinched throat: `a_g = a_p M_p / H_g` (degrees of
    turning per unit hold `360 a_p M_p` carried, Kerr `J = 360 a M^2`
    degrees); no throat where `a_g > 1`, that is `j > 360 H_g`; seed 7:
    `j = 360 a_0 = 17150536155465 / 137438953472` degrees per unit hold; twist `n_g = a_g / (4 + a_g)`.
  - Pressure on a state: `p(m0) = (D(n)^m0 - 1) / (D(n)^m0 + 1)`; held by
    pressure where `D(s) D(n)^A <= 1 - 2 delta`.
  - Seed 7 (over `L = 4779169536781`): tier 0 `n = 381123025677 / L`, held
    `A <= -8`; tier 1 (Kerr): `a_1 = a_0 M_0 / H_1` with `H_1` the first
    cohort's hold, 1.2976 core units.

- Holding in a pinched throat
  ([model.md](model.md#axioms-and-theorems-of-the-founding), [#35](https://github.com/SAIOS-SAMN/xylarium/issues/35)).
  Status: planned (derived).
  - Per plane: cohort hold `H_j = sum_k M_k w_kj`, `w_kj` the kind's share
    of turning in plane `j`; `s_j = (2 H_j + J_j) / (4 H_j + J_j)`,
    `J_j = J / 5`, `J_0 = 0`.
  - Enclosed matter on the throat orbit: push `J_j / (6 H_j + J_j)` per
    echo; none on the axis plane.
  - A nucleus: `D_k = D(p_plane)^w_k D(n)^A`; held where `D_k <= 1`; else
    leaves after `ln 2 / ln D_k` echoes.
  - Seed 7 gate: held 0.4352 (core), 0.4569 (tier 1); 0.108 too light to
    close, medium.

- The state at the pinch
  ([model.md](model.md#the-state-at-the-pinch-the-fold-and-its-circuits),
  [#35](https://github.com/SAIOS-SAMN/xylarium/issues/35)). Status: planned (derived).
  - Neutral surface: `sum_l m_l nu_l ln(r_l / r_0) = 0`; none where
    `sum m_l nu_l = 0` (the cohort flows).
  - Circuits per layer `k` since its onset `n_k`, `N = N* - n_k` echoes:
    reach `ln r = N ln(D(s) D(n)^A)`; flipped share `1 - (1 - n / 3)^N`;
    pressure `D(n)^(A N)`; pairs `D(p_pair)^N`; heat composed per echo
    from held `s W / 3Z` net of emission `(U / 3Z)` of it.
  - Seed 7: 605 layers, 0.5648 of the founded mass; `ln r_0 = 17.966`;
    inside 0.1783, outside 0.3865, without grip 0.4569.

- The bounce at a pinched throat
  ([model.md](model.md#the-bounce-at-a-pinched-throat), [#78](https://github.com/SAIOS-SAMN/xylarium/issues/78)).
  Status: planned (derived).
  - Far face: `t -> 1 / t` per layer; axis polarity `A -> -A`; phases 180
    degrees on each of the five isoclinic planes.
  - New throat's jet: `a = D(s_g) D(n_g)^(-A) / (1 - delta)`,
    `b = delta / (1 - delta)`, `delta = m / (2 H_g + m)`; held where
    `D(s_g) D(n_g)^(-A) <= 1 - 2 delta`.
  - Seed 7 gate: eleven throats in a chain; the bulk's climb over the
    pinch interval about 0.67, so one pinch carries each tier's matter.

- Throat holding (`circuit::onset_echoes`; derived): with no internal
  moment, a kind is held by the throat exactly when it has no open frames
  (`compound::open = 0`): it has no pole state, and the jet never acts on
  it. With its net axis weight `A`, an open kind dilates by
  `D(s) D(n)^A` per echo and is held where `D(s) D(n)^A <= 1 - 2 delta`;
  on the continuum alone, `A <= -log_D(n) D(s)`, for seed 7 `A <= -8`
  (`D(n)^8 >= D(s) > D(n)^7`, exact): 461 open kinds held, Z 65-124,
  279-338, 422-762 (planned; the test below covers the moment-free map). A kind with `U` open and
  `W` held frames crosses within `ln(W / U) / ln D(s)` echoes on the
  continuum alone, since the map `t -> a t + b` has `a >= D(s) >= 3` and
  `b >= 0`. Over the founded kinds (seed 7) 20 are held, Z 2, 4, 28, 30, 32,
  34, 58, 208, 210, 234, 236, 260, 262, 264, 266, 268, 270, 272, 418, 1092,
  with 43.5% of the founded mass; 524 of the vessel's 525 kinds are open.
  Z 1093 to 1700 hold 22 closed kinds (Z 1094, 1118, 1272-1276, 1298-1302,
  1326, 1328, 1352, 1354, 1378-1396, even), unfounded: the founding stops
  at the vessel's seal, Z 1092. Status: built (the test
  `a_kind_is_held_exactly_when_it_has_no_open_frames`), and the firmament
  is the closed kinds (`circuit::Substrate::found`).
- Jet onset (`circuit::onset_echoes`): `circuit::throat_crossing` of
  `t = open / hold` under the kind's jetstream map at the torsion share `s`
  and the kind's vacuum share `delta`, `a = D(s) / (1 - delta)`,
  `b = delta / (1 - delta)` (`circuit::jet_map`); a sealed kind has
  no pole state and no onset, and the onset is zero where `t >= 1`.
  Status: built.
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
- Jetstream law (`circuit::jetstream`), one law for every holder of hold `w` and
  open frames `u`: continuum (outflow) at push `p_c`, `rho = (1 + p_c) / (1 -
  p_c)`, moves `(rho - 1) u w / (w + rho u)` frames Z to Q; vacuum (inflow) at
  `p_v = H / (R + H)` moves `p_v w` frames Z to Q; electron phase moves `R u`
  frames Q to Z (mean zero over an echo). `rho_net` is the product of the
  channels' dilations, the torsion the echo leaves. Firmament kinds: `p_c = 1/2`
  at the throat's reach. A star's collective: `p_c = p_v = delta`, the vacuum's
  share at the star's rim about the star's mass, as a firmament kind's is read
  at its reach about the kind's hold. Mean echo map (`circuit::jet_map`): `t ->
  a t + b`, `a = rho / (1 - p_v)`, `b = p_v / (1 - p_v)`. Firmament kinds: `p_c
  = 1/2`, `p_v` the kind's vacuum share at its reach about its hold. Status:
  built for both.
- Throat crossing (`circuit::throat_crossing`): iterating `t -> a t + b`,
  `t_n = t* + (t0 - t*) a^n` with `t* = b / (1 - a)`, so the throat falls
  after `ln((1 - t*) / (t0 - t*)) / ln a` echoes; `circuit::echoes_to_throat`
  is the case `b = 0`. Status: built.
- Timeline (`stars::timeline`, `stars::remnant`): fusion steps run their
  transduced share over the transduction per bell until the throat
  crossing (a step running there is cut), fission steps from it; the
  remnant is one less what ran. Status: built.
- Waveform (`Substrate::amplitude`):
  `A(r, t) = [cos(2 pi (t - tau) / echo) / echo + q(t - tau)] / (r + 1)`,
  with `tau = (r / rim) bell / 2`. Status: built.
- Pull and delay (`Substrate::ocean_shares`): the pull is `h / (r + h)` with
  `h = state / mean state` (the home galaxy has 1 at reach 0); a galaxy's
  share is its pull over the sum, and its delay is `tau`. Status: built.
- Coherence (`circuit::coherence`): `1 - 2 min(d, 1 - d)`, with `d` the
  phase difference on their shared plane as a share of 360 degrees.
  Status: built.
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
- Galaxy selection (`Substrate::holds_a_star`, `Substrate::galaxies`): a
  bead is a galaxy where `lit x state >= m_p` and `state <= 1` (core
  masses), with `m_p = s c` the star mass of the piece that lights first
  (`galaxy::first_star_mass`). Status: built.
- Galactic core mass (`cores::founding_of`): `hole_suns = hole_mass x state`.
  Status: built.
- Circulation (`cores::founding_of`): the circulation is `1440 delta / echo`
  degrees per Gyr on the galaxy's orbital plane, with `delta = inside / (reach + inside)`. Status: built.
- Galaxy plane (`cores::founding_of`): the node lies at
  `index x` the golden angle about the throat's axis, and the tilt toward
  it holds `index x` the plastic angle's share of the S^360 measure (a
  placement by index, roadmap item 40) (`galaxy::node_of`, `galaxy::tilt_of`). Status: built.
- Galaxy scale (`cores::Shape::scale`): `2 m / (1 + m)`, with
  `m = hole_suns / hole_mass`. Status: placeholder.

### Perturbations, throats and radiation

- Perturbation front (`bounce::curved_reach`, `Anchor::bell`): one sphere
  per bounce, grown from the core across its enclosure of curvature radius
  `rho`; reaches are `rho sin(R / rho)`; the bell is the front's round trip
  to the home galaxy's rim, `2 rim 60^5` in `r_g / c`. Status: built.
- Throat (`bounce::horizon`): the sphere through the crossing at the
  one-third deficit, `R_h = 2M`, with a near and a far face. Status: built.
- Echo (`circuit::echo`): two rings, the near face's ring and the far
  face's return. Status: built.
- Equator (`circuit::throat_push`, `circuit::axis_antipole`):
  `delta(H, H) = 1/2`, the fixed point of `delta -> 1 - delta`; on the pole
  axis the inversion `X(t) -> X(1 / t)` fixes `t = 1`. Status: built.
- Arrival (`Substrate::amplitude`): `(r / rim) bell / 2` after the bounce,
  amplitude falling as `1 / (r + 1)`. Relay between holes
  (`pulse::relays`): continuity `cos(2 pi detour / wavelength)`. Status:
  built; the single front echoed at surfaces with delay `r + H(<r)` is
  under review (roadmap, milestone 3).
- Irradiation, held and passed: received `1 / (pi (r + 1))` per bell
  (`stars::ring_per_bell`); held `delta = M / (R + M)` of it; passed
  `1 - delta`; received = held + passed. Status: built for stars
  (`stars::transduced_per_bell`); next for worlds (layer C).
- Emission: a transducing body returns its held share outward; a star's
  power is `P = m transduced_per_bell(r) / bell` (`stars::power`); a nucleus
  radiates `open / weight` (`formation::emission`). Status: built.
- World heat (next, layer C): a world holds its deficit share of its star's
  light at its reach; its temperature follows from the held share; its
  re-emission is that heat turned outward. Replaces `278 L / a`,
  340 W/m^2, 288 K and `greenhouse_tau`. Status: placeholder.

### Z, Q and R (the pole model)

- Weights (`circuit::nucleus_weights`, `circuit::mix_weights`): `Z` is the
  proton count and `Q` the neutron count (the mass number
  `A = floor(a + 1/2)` less the charge, `circuit::mass_number`,
  from the element table; a cluster sums its parts), and a kind weighs
  `Z + Q` nucleons. Status: built.
- Electron phase state (`circuit::electron_phase`):
  `R = kappa (1 - 2 frac(p))` in `[-kappa, kappa]`, with `p` the bounce's
  pulse as a share of 360 degrees on the isoclinic planes and `kappa` the electrons' coherence on S^11 (`fibres`,
  `2 G / ends` for the charge's electrons); `R` is zero for a kind with no
  charge. Status: built.
- Pole point (`circuit::pole_point`, `circuit/poles.rs`): `X(t)` lies on the
  pole axis at `t = open / hold`, with the Z pole at the origin `e_o`, the Q
  pole at the point at infinity `e_inf` and the throat at `t = 1`; the
  inversion is `X(t) -> X(1 / t)` (`circuit::axis_antipole`). Status: built.
- Axes (`circuit/axes.rs`): the pulse has one fixed axis, the pole axis
  (base axis 1), and five isoclinic planes on base axes 2-3, 4-5, 6-7, 8-9
  and 10-11 (`circuit::plane_blade`). 180 degrees on each isoclinic plane
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
- Stellar mass (`galaxy::stellar_mass`): `Lighting::lit` core masses.
  Status: built.
- Mass inside (`galaxy::inside_mass`):
  `1 + rings within r / all rings` core masses, with a ring counted by the
  share of its span inside `r` (galaxy rings). Status: built.
- Turning law (`turning::turning`, `turning::turning_dy`):
  `omega = delta(<r) bell / r` radians per ring, with `r` in `r_g`
  (`r Tier::in_rg`), `delta(<r) = M(<r) / (r + M(<r))`, `M` in core masses
  and `bell` in `r_g / c` (`Anchor::bell`). Status: built.
- Piece of gas (`formation::cooling`): `lambda = (state / unit) / W`; the
  agitation is `a0 = (p - delta) / p` with `delta = h / (R + h)` and
  `h = 1 + held share`; `e = sum s_z e_z` and `mu = lambda e`; `n = 1`
  beyond one galaxy ring and `n = 2` otherwise. Status: built.
- Gathering (`formation::gathered`): `m(w) = (lambda + mu) W share_n(theta)`,
  with `theta = 360 w / (2 (w + R))` degrees on the infall's plane
  (`xyl-core measure::share_turns`).
  Status: built.
- Grip and shear: the grip is `m / (w + m)`, and the shear is
  `h w / (R (R + w))`. Status: built.
- Vacuum halving (`formation::vacuum_halving`): `k_v` is the least whole
  `k >= ln 2 / ln((R + h) / R)`. Status: built.
- Heat (`formation::collapse`): the heat is
  `min(1, ring x swing x grip(W) / shed)` (`formation::pulse_heat`, the one
  form for the gas and the disc), with `ring = 1 / (pi (R + 1))` and
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
- A star's reach (`galaxy::Star::reach_rings`): its orbit radius about its
  galactic core, galaxy rings; the star's class, its system's light and its
  worlds' life and minds all read it. Status: built.
- First light (`galaxy::birth`): the first light of the piece at the
  star's reach falls at `max(cools_in, 1)` rings; there is none where the
  piece never lights. Status: built.
- Birth along the ring (`chem::Chemistry::birth_along`): the birth time is the
  `t` (rings) with `F(t) = F(t0) + u (F(top) - F(t0))`, where `top` is the 30
  Gyr horizon read in rings, found by 48 halvings. `F` is the ring's `sfr`
  integrated term by term, `c p! / k^(p+1) (1 - e^(-kt) sum_(j<=p) (kt)^j /
  j!)`, or `c t^(p+1) / (p + 1)` at `k = 0` (`chem::formed_by`). Status: built.
- Star formation per ring (`chem::Model::of`): the rate is
  `efficiency x Omega / 2 pi` with `Omega = v_flat / r`; the gas is a sum of
  `c t^m e^(-lambda t)` terms, with infall on the ring's first-light
  timescale toward `galaxy::density x sigma_r0`. Status: partial (rulers
  `v_flat`, `sigma_r0`, `prompt`, `slow`, `loading` and the ring width).
- Yields (`chem::Calibration`): per-channel scales are set from
  `Rulers::ism_today_dex` at the reference ring at `Rulers::today` (13.8 Gyr in
  rings, the calibration epoch until phase 5); `relative` is 1 in the
  calibration's reference birth gas; hydrogen and helium follow from the
  founding's counts with `Rulers::dy_dz` and `Rulers::reference_metallicity`
  (`chem::Light`). Status: placeholder; the next step is yields from the
  founding (roadmap, Rules from the founding and the galactic cores).
- Placement (`galaxy::star`): the reach is where the rings' cumulative
  holding reaches the silver-angle pivot; the disc phase follows the golden
  angle on the disc's plane, the ball planes follow the plastic and golden
  angles (a placement by index, roadmap item 40), and the rate
  follows the turning law. Status: built.
- Piece star mass (`galaxy::piece_star_mass`): `m_p = s c` core masses,
  with `s` the share of the piece's infall the star tier's cut holds
  (`disc::StarHold::star`) and `c` the infall's holding
  (`disc::StarHold::cloud`). Status: built.
- Initial mass function (`galaxy::imf`): `m = x m_p` with
  `x = b u / (1 + b - u)`, the inverse of `F(x) = x / (x + b)` on `[0, 1]`,
  and `b = d / (1 + d)` from the crowding `d`. Status: built.
- Vertical orbit (`galaxy::vertical`, `galaxy::layer`,
  `galaxy::vertical_frequency`): `nu = omega 2d / (1 + d)` radians per ring
  (turning law about `M(<r)`, crowding `d`);
  `h = r m / (M(<r) + m)` galaxy rings (`m` the ring's mass,
  `galaxy::ring_mass`); `s = t / (1 + t)` with
  `t = age / cooling time` (`galaxy::birth`); `sigma = h nu s 2d / (1 + d)`;
  `vz = -sigma ln(1 - u)`, `A = vz / nu`, period `2 pi / nu`, exposure
  `atan(h / A) / (pi / 2)` where `A > h`, else 1. Status: built.

## 6. Stars and systems

- Star's compactness (`disc::rim_share`, `galaxy::Star::compactness`):
  `delta = m / (R 60 + m)`, `m` in core masses, `R` the rim of the cloud's
  settled cut in system-tier units (`galaxy::piece_rim`). Status: built.
- Collapse and ignition (`disc::star_hold`): dark cut `s0`, `R0`;
  `delta0 = s0 c / (R0 60 + s0 c)`; transduction per echo `delta0 x turning`;
  settled cut at `p0 = 1 + transduced`. Status: built.
- Ring per bell (`stars::ring_per_bell`): `1 / (pi (r + 1))`. Status: built.
- Transduction per bell (`stars::transduced_per_bell`): `delta / (pi (r + 1))`.
  Status: built.
- Frames at heat (`stars::held_frames`): `u_z(q)` open frames
  (`compound::open_hot`), `w_z(q) = 3 Z - u_z(q)` held; `A_z` the element
  table's `a` rounded to the nearest whole (`stars::nucleon_count`). Status:
  built.
- Pole state of a star (`stars::mixture_pole`): `t0 = sum n_z u_z / sum n_z
  w_z` over the held mixture (`disc::StarHold::held`), `n_z = share_z / w_z`;
  `t0 <= 1/2`. Status: built.
- A star's throat (`stars::Sequence::throat_echoes`):
  `circuit::echoes_to_throat` of its pole point under
  `shares::dilation_dy(delta)`, `n* = ln(1 / t0) / ln((1 + delta) / (1 - delta))`,
  echoes from ignition; none where `t0 = 0`. Status: built.
- Fusion step (`stars::fusion_step`): the least `k >= 2` with a kind at `A'
  = k A_z` releasing `1 - w' / (k w) > 0`, the greatest release there.
  Status: built.
- Fission step (`stars::fission_step`): kinds with `A1 + A2 = A_z` nearest
  equal halves releasing `1 - (w1 + w2) / w > 0`, the greatest release
  there. Status: built.
- Sequence (`stars::Sequence::of`): at each stage the held kind of greatest
  release steps (ties by the lesser `A`), transducing `share x release` and
  passing `share x (1 - release)` to its products (fission: in proportion to
  their held frames), until no held kind releases; fusion stages, then
  fission stages where `t0 > 0`. Remnant `1 - sum transduced`
  (`stars::remnant`). Status: built.
- Life (`stars::life`, `stars::ends`): step `i` ends at `sum_{j <= i}
  transduced_j / transduction` bells; the life is the first step's end. The step
  at an age (`field::transduction`) is `Unlit` (no step), `Step(i)` or
  `Remnant`; the transduction-out share (`field::transduced_share`) is the
  step's release. Status: built.
- Power (`stars::power`): `P = m transduced_per_bell(r) / bell` core masses per
  ring (`c = 1`). Status: built.
- Light (`stars::light`): `P (1 + pi bell q(t))`. Status: built. Boundary
  (`stars::suns_of_power`): `P` read in the prototype's display unit of
  luminosity read by the system and world layers.
- Flicker (`stars::flicker`): `(pi / 2) |cos(2 pi t / echo)|`. Status:
  built.
- Star's cut (`disc::star_hold`, `formation::hold_at`): the spin is
  `h w / (R (R + w))`, the heat is `2 ring_per_bell(r)` and
  `p0 = 1 + 2 transduced_per_bell(r)`; each kind is carried to
  `l = p p0 (e / e_mean)(w_mean / w) + spin` and is held where
  `h* / (l / h* + h*) >= p`, at the largest fixed point `h*` of the held
  hold. Status: built.
- Disc agitation (`disc::star_hold`):
  `a_disc = min(1, turning x swing x grip / shed)` (`formation::pulse_heat`), with
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
  `k x` the golden angle on the disc's plane. Status: partial (`reference_metallicity`).
- Formation temperature (`systems::Spec::t_form`): `278 L / a`. Status:
  placeholder.
- Envelope (`systems::system`): a world gathers an envelope when its core
  holds at least 10 world masses, by a declared gathering law. Status:
  placeholder.
- Settling rate (`systems::settle_about`): the turning law
  `omega(A) = bell [M / (A + M)] / A` per ring with `A` in `r_g`
  (`turning::turning_dy`). Status: built.
- Kick (`systems::settle_about`): `eps = omega_m(r) / |dw|`, the turning law
  about the pair's mass `m` across its gap `r` (`turning::turning_dy`), and
  `N = age |dw|` with the age in rings (`circuit::rings_of_gyr_dy`); `q` is the least continued-fraction denominator with
  `|s - p/q| N q <= 1`, `c = |1 - 2 frac(q psi_0)|`, and the swing is
  `eps (floor(N / q) c + N mod q)`. Status: built.
- Merge (`systems::settle_about`): the merge age is the first age with
  `swing x inner / gap >= 1`, found by 48 halvings to the 30 Gyr horizon;
  merges by the settled age (3 Gyr) combine mass, reach and phase
  mass-weighted, and later ages are recorded on the lighter body
  (`World::merged_at`). Status: built.
- Orbit (`bodies::orbits`): `e0 = swing`; the tilt is `swing / (1 + swing)`
  of 90 degrees toward the axis of the star's plane, and the node and periapsis take the settled phase.
  Status: built.
- Moon manifold (`systems::moons`): the reach is `2 pi a m / (M + m)`
  (`manifold::reach`) in the world's radii from 5; the budget is
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
- Angle between bodies (`manifold::gap`, `chain::Place::arc`):
  the certified angle `pi / 2 -/+ asin(|a . b| / |a| |b|)`
  (`xyl_core::rotor::angle_between`), the directions exact, read at its
  midpoint; the rotor's inverse is its conjugate (`Rotor::inverse`). The
  arc on a sphere is that angle times the chord `R sin(s / R)`
  (`bounce::curved_reach`). Status: built.
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

## 8. Shared laws

The Concordance (sections 1.14 and 2.10) places the laws that every
universe on the network uses in mycelium's `forms` crate, the core shell,
stated over each universe's own constants. A kernel calls those forms and
states only its founding. A law enters the core shell only when a
Concordance amendment rules it universal; until then it is a named form of
Xylarium's shell (Concordance A7). The coordinator writes `forms`, one law
per pull request, reproducing the fixtures Xylarium supplies in
[#33](https://github.com/SAIOS-SAMN/xylarium/issues/33); until a form lands,
the Xylarium function is the implementation. The ids are proposals.

| Proposed id | Form | Shell proposed | Xylarium implementation |
|---|---|---|---|
| LAW-DEFICIT-SHARE | `delta = M / (r + M)` | core | `xyl_core::shares::deficit` |
| LAW-DILATION | `D(p) = (1 + p) / (1 - p)`, `p` in `(-1, 1)` | core | `xyl_core::shares::dilation` |
| LAW-TORSION-SHARE | `s(H, J) = (H + J) / (2H + J)`, `H` the horizon reach and `J` the spin's share, both stated by the universe | core | `bounce::torsion_share` |
| LAW-JET-MAP | `t -> a t + b`, `a = D(p_c) / (1 - p_v)`, `b = p_v / (1 - p_v)` | Xylarium | `circuit::jet_map` |
| LAW-THROAT-CROSSING | `t_n = t* + (t0 - t*) a^n`, crossing at `n = ln((1 - t*) / (t0 - t*)) / ln a`; refused for `t0 <= 0` | Xylarium | `circuit::throat_crossing` |
| LAW-POLARITY-SEQUENCE | pole point `sum(n u) / sum(n w)`; fusion below the throat, fission above, ordered by release | Xylarium | `stars::sequence::mixture_pole`, `Sequence::of` |

LAW-TORSION-SHARE takes the horizon reach as an argument: Xylarium states
`H = M (1 - delta_h) / delta_h = 2M` (`bounce::horizon_reach`) with the
horizon's held share `delta_h = 1/3` a declared ruler of its founding, and
`J = a M^2`. The rim share (`disc::rim_share`) is LAW-DEFICIT-SHARE with the reach
`R k`, `k` the tier ratio. The dilation turns the flow law of Concordance
section 2.6 into a product, `D(x (+) y) = D(x) D(y)` with
`x (+) y = (x + y) / (1 + x y)`, and the court's throat level is
`c = D(t^2)`; the jet map with `b = 0` is therefore the flow law in the
dilation coordinate, while the vacuum term `b > 0` keeps it a form of
Xylarium's shell. Xylarium's Cl(12,1) blades stand as an outer shell of Xylarium
over the Cl(4,1) centre of `forms`.

Xylarium evaluates the dilation, jet map and crossing on `Dy` readings;
it holds them in `Q` (the crossing count as an enclosure) when it calls the
forms. Tracked in roadmap item 27
([#33](https://github.com/SAIOS-SAMN/xylarium/issues/33)).
