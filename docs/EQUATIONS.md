# Equations

This document lists Xylarium's physics equations, formula by formula, in the
order of [model.md](model.md), which explains how each follows from the one
before. Terms are defined in [glossary.md](glossary.md).
Each entry gives the formula, its code location, its calibre and its
status. The calibre (docs/model.md, Calibre and dimensional capacity) is the
pair of units its reaches and holds are read in, one of five:

- None: a calibre-free identity, count or algebraic form.
- Founding: reaches as ratios to the hold, `R_Z = (K + s) / (m_Z H)`, holds
  in core masses (the anchor declares these reaches cluster units).
- Core (`r_g`): reaches dilated to `r_g` by `Tier::in_rg`, holds in core
  masses.
- Mixed, founding and core (#123): the entry reads both, which no calibre
  allows; each is to be read in one calibre once #123 is ruled.
- SI seam: SI, kpc, Gyr, AU or kelvin, or a declared ruler.

The status is one of four values:

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

Letters: `h` and `o` are a dipole's held and open faces; `c` is the speed
of light only.

## Updated equations

This section tracks the equations ruled or derived since 2026-11-11 and the
forms they invalidate, so each change has one line of history. An E-row is
a new or updated equation; an I-row is a form in the code or the docs it
invalidates, with its sites and register IDs (docs/assumptions.md). A row
moves to Built with the PR that lands it, and an I-row closes when its
sites are restated. The derivations are in
[model.md](model.md#faces-of-the-dipole-on-sn).

| ID | Equation | Replaces | Ruling / issue | Status |
|---|---|---|---|---|
| E001 | Push and resistance: `p_net = p (+) (-r) = (p - r) / (1 - p r)`, `D(p_net) = D(p) / D(r)` | bare and subtracted pushes (I002, I005) | ruled; [#195](https://github.com/SAIOS-SAMN/xylarium/issues/195) | stated; sites open |
| E002 | A face's push: `p = h - o = 2h - 1`, pole `t = o / h`, `D(p) = h / o = 1 / t` (`shares::Faces::push`) | `p = 2c - 1` (I001) | [#155](https://github.com/SAIOS-SAMN/xylarium/issues/155), [#197](https://github.com/SAIOS-SAMN/xylarium/issues/197) | built (letters renamed) |
| E003 | A share offset by a resistance share: `t_net = t_a / t_b`, `h_net = 1 / (1 + t_a / t_b)` | composing raw shares as pushes (I002) | derived; [#195](https://github.com/SAIOS-SAMN/xylarium/issues/195), mycelium #241 | derived |
| E004 | The faces of S^n: `N = n + 1` dipoles (a complex carrier `S^(2m-1)`: its `m` planes), `2N` faces, each `1 / (2N)` of the measure | the two-face reading of a whole body (I003) | ruled; [#197](https://github.com/SAIOS-SAMN/xylarium/issues/197) | derived |
| E005 | A body's weights per dipole: `w_k >= 0`, `sum w_k = 1`; a state's plane shares `|m_k| / l` (`hopf::plane_shares`); an extended body's caps by volume (`measure::share_turns`); isotropic `1 / N` | one hold for every direction (I003) | [#197](https://github.com/SAIOS-SAMN/xylarium/issues/197) | derived |
| E006 | Per dipole at reach `r`: `m_k = w_k m`, `h_k = m_k / (r + m_k)`, `o_k = r / (r + m_k)`, `t_k = r / m_k`, `p_k = h_k - o_k` | one face pair (I003) | [#197](https://github.com/SAIOS-SAMN/xylarium/issues/197) | derived |
| E007 | Per dipole against the medium (`H_k = sigma_k H`, `rho_k = H_k / (r + H_k)`): `t_net,k = t_k / t_rho,k = H_k / m_k`, the reach cancelling; balance `m_k = H_k` | the expansion law's reach-dependent rate (I002, open) | derived; to rule; [#197](https://github.com/SAIOS-SAMN/xylarium/issues/197) | derived |
| E008 | Capacity: a reading carries `N` components, `v_k = p_net,k c` each in `(-c, c)`, never summed; its capacity is `N c` | light speed as the one ceiling, `v_e = c` (I004, I005) | ruled; [#197](https://github.com/SAIOS-SAMN/xylarium/issues/197) | derived |
| E009 | A drive's exhaust per face: `p_e,k = (p_1,k (+) (-r_1,k)) (+) ... (+) (p_n,k (+) (-r_n,k))` over the states it produces on face `k` | `best_spec`, `blended` (I004) | [#192](https://github.com/SAIOS-SAMN/xylarium/issues/192) | planned |
| E010 | The charged spin per plane: `theta_k = 360 delta_k`, `pi_k`, `eps*_k = 90 pi_k`, `K = sum_k delta_k (T_k B_k T_k~)` | one `delta` and one tilt for every plane (I006) | [#183](https://github.com/SAIOS-SAMN/xylarium/issues/183) | planned |
| E011 | Coupling is first-degree in the faces: `h = M / (r + M) = 1 / (1 + t)`, `o = r / (r + M) = t / (1 + t)`, the hold and the reach in one denominator, calibre-free; a traversal turns `360 h` on its named plane (the turning law, built); the measure of `S^n` (`sin^(n-1)`, `measure`) is the sphere's volume element, not a coupling | `M / r^2`, `M / d`, `a^2 / (4 d^2)`, `M / R^2`, roots (I008) | ruled; [#199](https://github.com/SAIOS-SAMN/xylarium/issues/199) | stated; sites open |
| E012 | The state `(3W, p, R)`: `3W = 3(Z + Q)`; `h = 3Z / 3W`, `o = 3Q / 3W`, `t = 3Q / 3Z`, `p = (3Z - 3Q) / 3W`, `D(p) = 3Z / 3Q`, `D(-p) = 3Q / 3Z`; `R` the axis plane's turn | the dipole's faces as a sum (I009) | [#201](https://github.com/SAIOS-SAMN/xylarium/issues/201), [#148](https://github.com/SAIOS-SAMN/xylarium/issues/148), mycelium #241 | derived |
| E013 | Formation's transitions: electrons move `R` (and `U`, `2P`); fusion and fission move `3W`, mixing the split by weight (the mediant of `D`); conversion moves `p` by flow steps `D(p') = D(p) D(s)`, keeping `3W`; a bond moves share through `g` | `Q` per kind from Earth's weights (I011) | [#201](https://github.com/SAIOS-SAMN/xylarium/issues/201) | derived; conversion not built |
| E014 | The conversion's rest point at a singularity: `p*` where `p (+) (-r) = 0`, `r` the singularity's medium at its throat, per dipole, first-degree; `3Q / 3Z = D(-p*)`. Read per Hopf plane, `p* = (p*_0, ..., p*_5)` (E016); at a bounce in the common phase the six are equal, so the planes part only where a winding is not common. Each genesis builds its universal table at the universal core's bounce; each galactic core builds its galaxy's table at the core's bounce, as an event. Built at the universal core: capture first (`m = Z`, `Q0 = 0`), then the conversion's rest point per parcel, `Q* / Z = H / R`, the antipolar face of the parcel's reach (the involution `t -> 1/t`, taken as the law's ground alone), with `R = H D(p_stack) / D(r_m)` (founding shells below), so at release `Q* / Z = D(r_m) / D(p_stack) = (3Z - U) / (3 D(p_stack))`; the push and the reach are held as face pairs, `R = [(H), D(s_final), D(s_i), (3, W_e)]`, and the rest point swaps them, exact (`shares::FacePairs`, #210); a kind's `Q_z` is its parcels' rest points weighted by amount, an exact sum, read on the 2^-64 grid for a weight and exactly for the whole-nucleon count (`formation::founding_shells`, `circuit::rest_neutrons`, `circuit::mass_number`) | every galaxy reads the universal table (S138) | [#201](https://github.com/SAIOS-SAMN/xylarium/issues/201), [#195](https://github.com/SAIOS-SAMN/xylarium/issues/195), [#7](https://github.com/SAIOS-SAMN/xylarium/issues/7) | built for the universal table (seed 7, one stack: every kind at its closed form `Q* / Z = 25 W_e / 147`, 50/147 to 3900/7, no kind refused); the galaxies' tables open |
| E015 | The axis fill both ways: `f_Q = abs(R) u / (W + abs(R) u)` from the Q side toward Z for `R > 0` (none for `R <= 0`), `R = kappa (1 - 2 frac(psi))` at the capture phase `psi = (s Z) mod 1`; exact shares per level (`atoms::Band::plane_levels`), the polarity `A` exact; both dipoles of the split kept, count (the neutral level halved) and weight | the Z-side-first integer fill (S136) | ruled; [#201](https://github.com/SAIOS-SAMN/xylarium/issues/201) | built |
| E016 | The split, fill and phase per Hopf plane `p` in `0..6` (6 dipoles, 12 faces on `S^11`): the level `m_p` holds `C(l - abs(m_p) + 9, 9)` orientations on every plane (a harmonic's multiplicity reads only `sum abs(m_p)`); the plane's capture phase `psi_p = (w_p s Z) mod 1` at the winding `w`; `R_p = kappa (1 - 2 frac(psi_p))`, `kappa` the whole fibre's coherence on every plane; `f_p = abs(R_p) u / (W_e + abs(R_p) u)`, `W_e = 3Z - U` the electron hold for `R_p > 0`, else 0; the plane polarity `A_p` (`circuit::fill_shares`, `atoms::plane_polarities`, `atoms::plane_sides`). In the common phase (`hopf::COMMON`) the six planes read alike, so the planes part only where the winding is not common (the Seifert twist, #128) or a tilt mixes the pole axis | the axis plane read alone, one scalar `R` (S137, I012) | ruled; [#201](https://github.com/SAIOS-SAMN/xylarium/issues/201), [#197](https://github.com/SAIOS-SAMN/xylarium/issues/197), [#183](https://github.com/SAIOS-SAMN/xylarium/issues/183) | built |
| E017 | The transitional boundaries per plane: the balance `t_p = 1`, which is `p_net,p = 0` and `D(p_net,p) = 1` (one boundary: `D(p) = 1/t`), with the faces `h_p = o_p = 1/2`; and the fill's reversal `R_p = 0`, at `frac(psi_p) = 1/2`. A body crossing on one plane and not another holds both phases at once, the boundary a surface inside it | a single fusion/fission threshold for the whole body | derived; [#201](https://github.com/SAIOS-SAMN/xylarium/issues/201) | the reversal a closed form of the phase; the balance's crossing waits on the conversion's motion (E013) |
| E018 | The nucleon dipole turned by the phase, per Hopf plane `p`: `p_nuc,p = (3Z_p - 3Q_p) / 3W_p` composed with `R_p` by the flow law, `p_net,p = p_nuc,p (+) R_p = (p_nuc,p + R_p) / (1 + p_nuc,p R_p)`, so `D(p_net,p) = D(p_nuc,p) D(R_p)`: the Z face `(3Z_p / 3Q_p) (1 + R_p) / (1 - R_p)` and the Q face, its antipole, `(3Q_p / 3Z_p) (1 - R_p) / (1 + R_p)`, product 1; `R_p = 0` is the bare dipole. At the bounce's rest point, `p_net,p = 0`: `p*_nuc,p = -R_p` and `3Z*_p / 3Q*_p = (1 - R_p) / (1 + R_p)`. `R_p` is read as a push in `(-1, 1)`, so a kind with `abs(R_p) = 1` (coherence 1 at a whole-turn crossing) has no rest point and is refused | `3Z / 3Q + R` (I009); one fixed table of weights (S029, I011) | ruled; [#201](https://github.com/SAIOS-SAMN/xylarium/issues/201), [#148](https://github.com/SAIOS-SAMN/xylarium/issues/148) | stated; the reading and the table from it are #201 step 3 |
| E019 | The pivot per plane: the relative versor `P_b^-1 P_a`, read on each plane `k` as its winding `K_k` in ZZ and phase `phi_k` in QQ/ZZ | `rotor::angle_between`, one scalar per pair | [#207](https://github.com/SAIOS-SAMN/xylarium/issues/207), [#118](https://github.com/SAIOS-SAMN/xylarium/issues/118) | planned |
| E020 | Twist per plane: `Tw_k = w_i,k - w_j,k`, the turn `360 delta_k`; writhe `Wr = Lk - Tw` | the turn on one plane (S130) | [#207](https://github.com/SAIOS-SAMN/xylarium/issues/207), [#128](https://github.com/SAIOS-SAMN/xylarium/issues/128) | planned |
| E021 | Strength and weakness per plane: the overlap's held face `h_k` and open face `o_k = 1 - h_k` (`hopf::overlap`) | calibrated material strengths | [#207](https://github.com/SAIOS-SAMN/xylarium/issues/207) | planned |
| E022 | Pull per plane: `p_k = h_k - o_k`, offset by the medium, `p_k (+) (-r_k)` | a pull read bare | [#207](https://github.com/SAIOS-SAMN/xylarium/issues/207), [#195](https://github.com/SAIOS-SAMN/xylarium/issues/195) | planned |
| E023 | Break: the event at which the linking `Lk` of two fibres changes; `Lk` is invariant under continuous deformation | a break threshold | [#207](https://github.com/SAIOS-SAMN/xylarium/issues/207), [#121](https://github.com/SAIOS-SAMN/xylarium/issues/121) | planned |
| E024 | The null pair: `ds^2 = u v`, `u = c dt - dx`, `v = c dt + dx`, never formed as a square; the flow law is velocity addition (`c = 1`), `D(p) = v / u`, `ln D(p)` twice the rapidity; the cone `p = +/-1` lies outside the dilation's domain | `ds^2 = c^2 dt^2 - abs(dx)^2`, a speed capped at `c` (I014, I015) | [#222](https://github.com/SAIOS-SAMN/xylarium/issues/222) | derived; the flow law and `D` built |
| E025 | A separation is the pivot per plane, `P_b^-1 P_a` (E019); a crossing is a count on the T lane, timelike, null or spacelike by the signs of `u` and `v` | one scalar arc over `c` (I014) | [#219](https://github.com/SAIOS-SAMN/xylarium/issues/219), [#207](https://github.com/SAIOS-SAMN/xylarium/issues/207) | planned |
| E026 | Speeds compose: exhaust, throat supply and the line's medium by `p (+) (-r)`; a spacelike span crosses only by a passage | `c min(S / T, c)` and `c - v_e` clamped (I015) | [#220](https://github.com/SAIOS-SAMN/xylarium/issues/220), to rule | planned |
| E027 | An event carried from `a` to `b`: the rapidities of the nested wells composed per plane, the place turned by `P_b^-1 P_a`, the T coordinate counted through the keepers between | a remote event read by light delay and the reader's keeper alone | [#221](https://github.com/SAIOS-SAMN/xylarium/issues/221) | planned |

| ID | Invalid form | Sites | Superseded by | Status |
|---|---|---|---|---|
| I001 | `p = 2c - 1` with `c` the held face beside the speed of light | docs (EQUATIONS faces entries renamed) | E002 | closed: renamed |
| I002 | A deficit share read as a push: the throat push `1/2` (the share 1/2 is the push 0), the push at reach, the jet map's `p_c` and `p_v`, the expansion law's `p_c = H / (r + H)` and `p_v = m / (r + m)`, the received axis push, and the 1/5 composition (withdrawn) | S114, S116, S117, S123; `circuit` parcel release (planned) | E002, E003, E007 | open: to rule, since it changes the expansion law's dynamics |
| I003 | One dipole read for a whole body or sphere: the pole point on one axis `t = U / W`, the turning law on one plane, the isoclinic turning as one number for five planes, LAW-COUPLING v2's split on the axis plane alone | S129, S130, S131; mycelium #241 | E004 to E006 | open |
| I004 | Light speed as the one ceiling: the drive's `v_e = c`, and "below `c`" as the bound (#195's first text) | S083, S085, S126 | E008, E009 | open |
| I005 | Speeds capped at `c` by `min`, not composed per face | S127, S128 | E001, E008 | open |
| I006 | One-dipole forms written on 2026-11-11: the tilt's faces and balance, the open charge, the charged spin's single `delta` | S124, S125, S132 | E010 | open |
| I007 | Offsetting the open charge by the bound share, `(p / w) (+) (-g)` (proposed on #183 and #195, withdrawn): `g` is the charge already moved onto the held stratum, so it would be counted twice; the push is the linear pair share, and the rate it drives is offset by the medium (E007) | S124 | E003, E007 | closed: withdrawn |
| I008 | A square or root at a coupling's root, or a reach decoupled from its hold: the field's intercepted share `a^2 / (4 d^2)`, the field's work `(M / a)^2 abs(dx)^2`, surface gravity `M / R^2` and pressure, `pulse::hear`'s `M / d`, `rotor::angle_between`'s root | S133, S134, S135, S025, S034 | E011 | open |
| I009 | The dipole's faces as a turn added to a dilation, `3Z / 3Q + R` and `3Q / 3Z - R` (#148) | docs (model.md, The dipole) | E012, E018 | closed: restated as the composition `D(p_nuc (+) R) = D(p_nuc) D(R)` (E018) |
| I010 | `W` names both the held count `3Z - U` and, in v2's letters, the nucleon whole `3W = 3(Z + Q)` | `compound::electron_hold` (was `compound::hold`), `circuit::fill_shares`, `circuit::Substrate::loop_flux_swung`, `spin_orbit` | E012 | closed: the electron hold is `W_e = 3Z - U` (`compound::electron_hold`, `w_e` at its sites); `3W = 3Z + 3Q` is the nucleon whole |
| I011 | One fixed table of measured weights (`crates/xyl-cosmos/data/elements.tsv`, through `circuit::mass_number`) for every genesis and every galaxy, so `3Q`, `3W` and the split are the same everywhere: neither the universe's own nor each galactic core's | S029, S010, `stars::fusion_step` products | E013, E014 | closed: the universal table from the rest point (E014); the galaxies' tables remain (S138) |
| I012 | The axis plane read alone for a kind's split, fill and polarity, with one scalar `R`: one dipole where `S^11` carries six | S137; `atoms`, `circuit::weights` | E016 | closed: read per plane |
| I013 | The coherence split across the planes by the states' plane shares, `kappa_p` from `abs(m_p)` (proposed with the six-plane ruling): the Hopf fibre turns all six planes at once, so its coherence is carried whole on each plane; the plane shares weigh where a state sits, not how coherent the fibre is | none | E016 | closed: not adopted |
| I014 | A separation read as one scalar (an arc in kpc, a gap in metres) and its light time as that scalar over `c` | S140; `cosmic::arc_kpc`, `transit::separation_m`, `transit::crossing_years`, `percept::perceive` | E024, E025 | open |
| I015 | A speed capped at `c`, `c min(S / T, c)`, and an exhaust clamped to `[0, c]`, where the flow law composes speeds below `c` | S126, S127, S128; `transit::passage_j`, `transit::building`, `transit::discord` | E024, E026 | open |

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
  Concordance 2.11 fixtures hold in `Cl(3, 2)`. Calibre: none. Status: built.
- Hopf carrier (`xyl_core::hopf`, #183): the six planes of `C^6`, plane
  `p` on `cga` axes `2p + 1`, `2p + 2`; the weighted turn
  `z_p -> e^{i w_p theta} z_p` (`turn`), the Hopf action on `S^11` for
  `w = (1, ..., 1)` (`COMMON`), free; plane shares `s_p = |m_p| / l`
  (`plane_shares`, refused for `l = 0`); the overlap of two fibres
  `c = sum_p min(s_p(i), s_p(j)) (1 - arc_p / 180)`, `d = 1 - c`, `arc_p` in
  `[0, 180]` degrees (`overlap`, `arc`); linking
  `Lk = sum_{q=1..5} (w_i0 w_jq + w_j0 w_iq)` (`linking`), the strata's
  `Lk(S^1_axis, S^9_iso) = 1` (`STRATA_LINKING`). Calibre: none. Status:
  built (ruled 2026-11-11).
- Capture phase (`circuit::found`): `(share Z) mod 1`, exact; its
  denominator divides the torsion share's. Calibre: none. Status: built.
- Coordinate (`xyl_core::coordinate`, LAW-COORDINATE v1): a lane is an
  exact rational of either sign; its sector at depth `d` is
  `[m 60^-d, (m + 1) 60^-d)` with `m = floor(v 60^d)`, exact and nested;
  `Reading::within` reads Below, Within or Above on the sector's exact
  bounds and refuses readings on two lanes; places (`2;20`, `-1;40`) and
  `coordinate::beat` (whole rings and the phase in degrees) are display
  only. Calibre: none. Status: built.
- Pair reading ([model.md](model.md#pairs-the-pivot-per-plane), #118): for
  coordinates `a`, `b` of one origin (else refused), read in their lowest
  common enclosure, the relative versor is `P = P_b^-1 o P_a` (refused
  where no lawful versor relates them); its reading is a vector over the
  turning planes, `d_k(a, b)` in exact degrees with winding
  `K_k = floor(|d_k| / 360)`, one component per plane by its generators
  (six for founded matter: the axis plane by its exponent, five isoclinic
  phases), never summed; `d(b, a)` is read with the return's deficit,
  never assumed equal; closeness is
  `C2 = sum_k min(w_k, w'_k) (1 - 2 |d_k|) + (1 - m) / 2`,
  `m = sum_k min(w_k, w'_k)`, `d_k` in turns. A single displayed angle is a
  certified sector of `P`'s angle, display only. Calibre: none. Status: planned; replaces
  `rotor::angle_between` (`cos = a . b / sqrt(|a|^2 |b|^2)`, one radian
  enclosure, one unnamed plane).
- Winding and phase ([model.md](model.md#winding-and-phase-the-diophantine-split), #149):
  a turn `a` in QQ, in turns (degrees over 360), is held as `(K, phi)`
  with `K = floor(a)` in ZZ and `phi = a - K` in QQ/ZZ,
  `0 -> ZZ -> QQ -> QQ/ZZ -> 0`; a resonance is the carried `(p, q)` of its
  advance in lowest terms. Calibre: none. Status: planned.
- Cohomology of the chain ([model.md](model.md#cohomology-pairs-loops-and-the-books), #150):
  0-cochains the bodies' dipoles per plane, 1-cochains the pairs' pivots
  `P_b^-1 o P_a` as `(K, phi)` per plane; a pair of one origin is the
  coboundary of its two states; a loop's cocycle condition is that its
  relations compose to the identity, and its holonomy is the class it
  holds when they do not. Calibre: none. Status: planned.

## 2. Units and the anchor

- Base units (`xyl-core units.rs`): `G = c = M_core = 1`, and
  `r_g = G M_core / c^2` (`Units::of`). Calibre: core (`r_g`). Status: built.
- Tier unit (`Tier::in_rg`): the unit of a tier is `60^k r_g`, with `k` = 10
  for the universe, 7 for the cluster, 5 for the galaxy, 1 for the system and
  -2 for the planet; between tiers,
  `dilation(from, to) = 60^(k_from - k_to)` (`manifold::dilation`). Calibre: core (`r_g`). Status:
  built.
- Bell (`circuit::anchor::of`): `bell = 2 rim 60^5 r_g / c`, with the rim
  read in the galaxy tier. Calibre: mixed, founding and core (#123). Status: built.
- Ring and echo (`circuit::bell`, `circuit::echo`): one ring is one
  bell, `echo = 2 bell`, and a one-way crossing to reach `r` takes
  `(r / rim) bell / 2`. Calibre: mixed, founding and core (#123). Status: built.
- Ocean speed (`Substrate::c_ocean`): `c_ocean = rim / (bell / 2) = 3600 c`.
  Calibre: mixed, founding and core (#123). Status: built.
- Universe radius (`circuit::anchor::of`): the radius is `rim / pi`, given
  in kpc and in `r_g`. Calibre: mixed, founding and core (#123). Status: built.
- First stars (`circuit::anchor::of`): the first stars form at
  `max(cools_in, 1) bell`. Calibre: founding. Status: built.
- SI bridge (`package::hole_mass`): the core's mass in SI units, declared by the
  package, which fixes `r_g` in metres for display in kpc, Gyr, AU and
  kilograms; the SI constants are in `package::Si`. Calibre: SI seam. Status: placeholder; the
  next step is the core's mass from the package (roadmap, Rules from the
  founding and the galactic cores).

## 3. The bounce and the founding

- Deficit law (`xyl_core::shares::deficit`, `deficit_dy`):
  `delta(R, M) = M / (R + M)`, gravity's convergent face; the received
  share `1 - delta` is its divergent face. Calibre: none. Status: built.
- Gravity's faces ([model.md](model.md#gravity-the-deformation-about-a-mass-invariant),
  #155; `xyl_core::shares::faces`, `shares::Faces`): with `t = r / M`,
  held (convergent) `h = M / (r + M) = 1 / (1 + t)`, open (divergent)
  `o = r / (r + M) = t / (1 + t)`; `h + o = 1`, `h / o = 1 / t`
  (`Faces::pole`). Calibre: none (a ratio). Status: built.
- Gravity's antipole (`Faces::antipole`, `Faces::push`): the inversion
  `t -> 1 / t`, `r -> M (M / r)`, gives `h(1 / t) = o(t)`; the imbalance
  `p = h - o = (1 - t) / (1 + t)` has `D(p) = h / o = M / r` and goes to
  `-p`, so `D(-p) = 1 / D(p)`; the equator `r = M` is fixed, `h = o = 1/2`,
  `p = 0`; exactly `p = 2h - 1 = 1 - 2o`. Calibre: none. Status: built (identities,
  `shares::tests::the_faces_are_antipoles_under_the_inversion`).
- Push and resistance ([#195](https://github.com/SAIOS-SAMN/xylarium/issues/195), ruled 2026-11-11): every push is offset
  by its resistance through the flow law,
  `p_net = p (+) (-r) = (p - r) / (1 - p r)`, `D(p_net) = D(p) / D(r)`.
  The balance is `p = r` (`p_net = 0`, `D = 1`); a net push may be negative,
  carrying the holder back. The resistance is the hold of the medium at the
  reach, read through its deficit face, unless a law names another. Built
  in this form: the expansion law, the bond's separation, the ocean's pair
  share, the faces' imbalance. Read bare or by subtraction, each marked
  (docs/assumptions.md, class P, S114 to S128): the throat push, the
  dilation's input, the push at reach, the jet map, a galaxy's push, the
  stack push, the carried reach, the agitation, the star tier's cut, the
  received axis push, the open charge, the tilt's balance, the passage
  energy and the kernel's capped speeds. Calibre: none. Status: law ruled;
  sites to be offset one by one.
- Turns on the faces (#155): `+360 h` degrees per traversal on the
  convergent face (deficit angle, inward), `-360 o` on the divergent face
  (excess angle, outward); the divergent turn at `t` is minus the
  convergent turn at `1 / t`. They are one element of QQ/ZZ:
  `-360 o = -360 + 360 h`, differing by the whole turn `360 (h + o) = 360`.
  A body reads winding `-1` and phase `360 h` (T3's `360 delta`), never
  their sum. Calibre: none. Status: derived; no reading changes.
- Torsion share as a face (axiom 7; `bounce::torsion_share`):
  `s = (H + J) / (2H + J)` is the divergent face about the hold `H` at
  reach `H + J`, pole `t = 1 + J / H = 1 + a / 2 = s_Q / s_Z`. Calibre:
  none. Status: built (identity,
  `turning::tests::the_torsion_share_is_gravitys_divergent_face`).
- Face dilations ([model.md](model.md#the-six-planes-a-face-per-plane)):
  `D(d(t)) = 1 + 2t`, `D(c(t)) = 1 + 2 / t`, `D(d - c) = t`; linear in the
  pole. Calibre: none. Status: built (identity,
  `shares::tests::a_faces_dilation_is_linear_in_its_pole`).
- T2 at the horizon: `s = d(t_s)`, `t_s = 1 + a / 2` (`M = 1`):
  `D(s) = 3 + a`, `n = d - c = a / (4 + a)`, `D(n) = t_s`. Calibre: none.
  Status: built (identity, `turning::tests::t2_is_the_face_law_at_the_horizon`).
- Plane faces (#128): plane `j` reads `t_j = J_j / (6 H_j)`; T14's push
  `J_j / (6 H_j + J_j) = d(t_j)` dilates by `1 + J_j / (3 H_j)`; the axis
  plane reads `t_0 = r / M`. Separation: a pole per plane, `t_j` distinct, totals
  `sum_j J_j = J`, `sum_j H_j = H_iso`, never an equal split.
  Calibre: none. Status: identity built
  (`turning::tests::t14s_plane_push_is_a_divergent_face`); the per-plane
  split planned.
- Capacity (axiom 9; `atoms::capacity`): `cap = E + U + 2P`, the room
  `E = cap - Z`, the open states `U` (`compound::open`), the held states
  `2P = Z - U`; shares against the capacity: occupancy `Z / cap = 3Z / 3Q`,
  held `2P / cap`, open `U / cap`, room `E / cap`. Calibre: none. Status:
  partly built (#158): the partition `cap = E + U + 2P` per band and per
  kind (`atoms::band_occupancy`, `atoms::occupancy`, identity over Z 1 to
  1700); the shares and the pole planned, the code reading the held count
  `W = 3Z - U`.
- Capacity face (axiom 9): convergent `2P / Z`, divergent `U / Z`, pole
  `t = U / 2P`, `D(U / Z) = 1 + U / P`; the antipole exchanges held and
  open; the equator `U = 2P`. Entered capacity (bench reading, #172):
  `cap_in = sum 2 d(l)` over the bands holding an electron,
  `Z <= cap_in <= cap`, equal to `Z` at a seal; the chain under
  `2P / cap_in` needs 1996 bits to the vessel's top at seed 7's share,
  under `2P / cap` 5546, under the built `W / 3Z` 1094. Built in its place: the held count's face,
  `W / 3Z` the closed share (`elements::closed_share`), `t = U / W` the
  pole point (`compound::electron_hold`). Calibre: none. Status: planned (#158).
- Share transfer (T15, #128): a state of weight `m` with shares
  `w_k = |m_k| / sum_i |m_i|` composes its planes' hyperbolic angles,
  `sum_k w_k ln D_k` with `D_j = 1 + 2 t_j`, and `D(n)^A` for its axis
  polarity; each `ln D_k` a certified enclosure. Calibre: none. Status:
  planned.
- Dilation and flow law (`xyl_core::shares::dilation`, `flow`):
  `D(p) = (1 + p) / (1 - p)` for `-1 < p < 1`; `x (+) y = (x + y) / (1 + x
  y)`; `D(x (+) y) = D(x) D(y)`. Calibre: none. Status: built.
- Horizon reach (`bounce::horizon_reach`): `delta_h = 1/3`, so
  `R_h = M (1 - delta_h) / delta_h = 2M`. Calibre: none. Status: built.
- Curved reach (`bounce::curved_reach`): the reach is `rho sin(r / rho)` on
  the S^360 enclosure of radius `rho`. Calibre: founding. Status: built.
- Horizon sphere (`bounce::horizon`): `sigma = sphere_through(C, P)`, where
  `P` is the crossing on isoclinic plane 1 at reach `rho sin(R_h / rho)` and
  the pulse's phase (`bounce::crossing`). Calibre: founding. Status: built.
- Spin bivector (`bounce::spin_bivector`): `K = a sum_(k=0..5) B_k` over
  the six planes, `B_0` the axis plane's blade in the common phase (#183),
  untilted; at a crossing off the axis plane `P . B_0 = 0`. Calibre: none.
  Status: built; to become the charged spin below.
- Charged spin (`xyl_core::hopf`, ruled 2026-11-11, #183):
  - the holonomy `theta = 360 delta` (`holonomy`);
  - the open charge `pi = (p / w) (1 - delta)` (`open_charge`);
  - the tilt of the pole axis toward isoclinic plane 1 by `eps` (`tilt`),
    with faces `c = 1 - eps / 90` and `d = eps / 90` (`tilt_faces_l1`);
  - the balance `eps* = 90 pi` (`tilt_equilibrium`);
  - `K = delta (T B_0 T~ + sum_{q=1..5} B_q)` (`charged_spin`), with
    `T K T~ = K` for every complex-linear `T`.

  Calibre: none. Status: built in `xyl_core`; read by the bells and the
  worlds (below); not by the bounce, which needs the parent hole's `p / w`.
- Stellar hole's axis (`field::stellar_tilt`, `field::stellar_axis`):
  - `eps* = 90 (p / w) (1 - delta)`, with `p / w` the progenitor's held
    mixture's proton share (`circuit::mix_weights`) and `delta` its
    compactness `m / (R + m)` (`Star::compactness`);
  - the axis is the orbit's rotor after a turn of the pole toward the line
    of nodes by `eps*`, so `n . n' = cos eps*` exactly (tested).

  Calibre: none. Status: built.
- World's tilt (`bodies::world_tilt`): `eps* = 90 (p / w) (1 - delta)` with
  `p / w = sum m_i Z_i / A_i / sum m_i` over the world's elements and
  `delta = m / (R + m)`, `m` in core masses and `R` in `r_g`; the cosine
  is the tilt's exact circle point. Seasonal (`life::seasonal`): `eps / 90
  > 1 - eps / 90`, that is past 45 degrees. Calibre: one per share
  (`r_g`). Status: built.
- White hole (`bounce::Bounce::of`, `xyl-core cga::Source`):
  `N = P ^ (P . K)` is null, fixes `P` and keeps `sigma`; the far face is the
  antipode of `P` under 180 degrees on each isoclinic plane
  (`circuit::plane_antipole`); its return is the next bounce's, at the
  horizon's deficit, one third.
  Calibre: none. Status: built.
- Spin draw (`origin::Origin::of`): the spin is drawn uniformly on the 2^-40
  grid, strictly inside `(0, 1)` (`origin::clamp_spin`). Calibre: none. Status: built.
- Torsion share (`bounce::torsion_share`), one law for every throat:
  `a = (H + J) / (2H + J)` with `H = 2M` and `J = a_Kerr M^2`; at the
  universal core (`M = 1`) `a = 1/2` at zero spin and `a = 3/5` at the
  extremal spin. The inverse (`bounce::spin_of_share`) is
  `J = H (2a - 1) / (1 - a)`, put on the origin's grid by
  `origin::clamp_spin`. The founding reads the share once
  (`circuit::Substrate::found`) and holds it as `Substrate::share`. Calibre: none. Status: built.
- Bond (`atoms::Bond`; docs/model.md, Scalar collapses; #107): two faces,
  the two atoms whole. Axis plane: each face's pole push
  `p = (U - W) / (U + W)`, `W = 3Z - U` (`Bond::pole_pushes`), their
  separation `p_a (+) (-p_b)` by the flow law (`Bond::axis_separation`),
  each face's polarity (`Bond::polarities`). Isoclinic planes: each open
  winding's tightness `(Z - shallower electrons) / n`
  (`Atom::winding_tightness`) and the share a turning leaves held,
  `min(1, tau / min(turning, 1))` (`Bond::held`). Calibre: none. Status: built; the graded
  stripping law is to be derived.
- Echo (`circuit::echo_at`; docs/model.md, The ring, the throat orbit and
  the echo's torsion; T4, axiom 7): a throat at torsion share `s` echoes
  after `bell / (1 - s)` on its near face and `bell / s` on its far face;
  two bells at zero spin. The universal core's founding reads its founded
  share (`Substrate::echo`); every tier after its bounce reads the share of
  its released spin (`circuit::echo`). Calibre: none. Status: built.
- Received turning (`atoms::Received`; docs/model.md, Chemistry from the
  elements; #103): `{ axis, isoclinic }`, the axis plane's push per echo
  (the pole state's dilation, a star's compactness) and the isoclinic
  planes' turning per echo in hydrogen units (stripping:
  `atoms::unpaired_hot` strips windings of tightness at most
  `min(isoclinic, 1)`; a bond holds `min(1, grip / isoclinic)`,
  `compound::held`). `stars::Sequence::of` reads the steps on the isoclinic
  planes and the throat climb on the axis plane. Calibre: founding. Status: built.
- Link remanence (docs/model.md, Phase weakening and phase strengthening;
  T19, #84): `w = J - (a_0 M_0) M`; per passage
  `w' = w (1 - m / M) + J (m / M) (1 - rho_c / rho_hold)`; conserved between
  passages; coercivity `m* = w M / (w + J (rho_c / rho_hold - 1))`; for one
  release, the remanent share `1 - rho_hold / rho_closed`. Seed 7: 0.1982,
  70.88 of `360 a* = 357.59` degrees per unit hold in the core's units.
  Calibre: none. Status: built (`circuit::SpinOrbit::remanence`; read by
  `xylarium observe universal-core`).
- Automorphism of the chain (docs/model.md, The automorphic, synaptic
  chain; #97): `M / (r + M)` is of degree zero in length, so
  `deficit(lambda M, lambda r) = deficit(M, r)` and the flow law of the two
  pushes is unchanged for every dilation `lambda > 0`; the kernel's tier
  ladder `60^p` (`units::Tier`, a declared ruler) is one instance. Calibre: none. Status: built at the law
  (`shares::tests::deficit_is_invariant_under_dilation`); the chain's
  links are planned.
- Isoclinic rotor (docs/model.md, The rotor, the place and the chain; T18,
  #90, #128): plane `j` holds `H_j = sum_k M_k w_kj` and carries `J_j`
  by separation, so T14's push `J_j / (6 H_j + J_j)` differs by plane: an
  isoclinic phase per plane. A body's turning is the axis-plane phase, the isoclinic
  phases and
  the turning between planes into its parent. Calibre: none. Status: derived; not built.
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
  `rho = -0.646` against `-0.798`, giving `a* = 0.9933`. Calibre: none. Status: built
  (`circuit::Substrate::spin_orbit`, `atoms::plane_polarities`,
  `circuit::Anchor::core_spin`; the firmament is the closed kinds).
- Cascade readings (`origin::Origin::of`): the lumpiness is
  `sum max(state - mean, 0) / sum state`, the held share is
  `firmament mass / holding`, the assembly time is `Lighting::cools_in` and
  the efficiency is `Lighting::lit`. Calibre: founding. Status: built.
- Closed share (`elements::closed_share`): `h_Z = hold_Z / 3Z = (3Z - u_Z) / 3Z`
  (`compound::electron_hold`). Calibre: none. Status:
  built.
- Capture chain (`elements::founded`): `count_1 = 2^b` and
  `count_Z = floor(count_(Z-1) a h_Z)`. Calibre: none. Status: built.
- Founding bits (`elements::founding_bits`): `b` is the least value with
  `chain(2^b) >= 1` at the vessel's sealing kind (Z 1092). Calibre: none. Status: built.
- Element table (`elements::table`): Z 1 to 99 are measured rows, Z 100 to
  150 are predicted rows (`data/elements.tsv`), and Z 151 to `TOP_Z` = 1700
  are generated (`elements::systematic_row`); the mass number lies on the
  valley `Z = A / (1.98 + 0.334 e(A / 100))` (`elements::valley_a`). Calibre: SI seam. Status:
  the counts are built; the catalogue columns (mass, melt, boil, density,
  condensation, class) are placeholders, and the next step derives them from
  the atom.
- Orientations on S^11 (`fibres::directions`): a band has
  `C(l + 11, 11) - C(l + 9, 11)` orientations, and its room is twice that.
  Calibre: none. Status: built.
- Fill (`atoms::atom`): each electron goes to the tightest open winding, with
  tightness `(Z - inside) / n` and winding reach `(n + 1)(l + 1)`. Calibre: none. Status:
  built.
- Seals (`atoms::Atom::sealed`): the sealed kinds are Z 2, 28, 208 and 1092.
  Calibre: none. Status: built.
- Vessel (`atoms::vessel`): the upper edge is the last seal, and the lower
  edge is the greatest `z < upper` with `u(z - 1) + 1 - u(z) < 0` (Z 568 on
  the current table). Calibre: none. Status: built.
- Cascade shares (`formation::founding_shells`): on the capture mass
  `m = Z`, `held` is the chain's gathered share, `H = 1 + held_final`,
  `s_final = 1/2 held_final / (held_final + H)` and a stack's share
  `s_i = 1/2 held_i / (held_i + H)`, `held_i` the stack's amount over the
  whole. The running capture shares are subtotals of one enclosure, so
  they are neither summed nor composed. Calibre: founding. Status: built
  (ruled 2026-11-11, #201).
- Shell reach (`formation::founding_shells`): the stack's push
  `p_stack = s_final (+) s_i`, the kind's resistance `D(r_m) = W_e / 3`
  (`W_e = 3Z - U`), `p_net = p_stack (+) (-r_m)` and
  `R_Z = H D(p_net) = H D(s_final) D(s_i) / D(r_m)`; the spread is
  `R (1 +/- s_final)`, and a shell is held where `R (1 + s_final) <= 1`.
  Calibre: founding. Status: built (ruled 2026-11-11, #201).
- Rest point (`formation::founding_shells`, `circuit::rest_neutrons`): per
  parcel at reach `R`, `Q* / Z = H / R`, the reach's face pairs swapped
  (`shares::Parcel::antipole`); a kind is its parcels' rest points,
  unsummed, and a reader reads `Q_z = Z sum(a H / R) / sum(a)` as an
  enclosure at the reader's precision (`shares::weighted`,
  `circuit::neutrons_at`); a kind with no parcel keeps its capture state
  `Q = 0`. Calibre: founding. Status: built (E014).
- Filter (`formation::founding_shells`): the contact is `1/2` for a whole
  passage and `1/2` of the share crossed for a partial one;
  `D = M / (M + 1 - contact)`, `T = |d_j - d_i| / (d_j + d_i)` and
  `g = min(s D, T) / T`. The filter takes `contact g m_i / (m_i + m_j)` of
  `m_j`, scaled by held over asked, and the take is carried by the
  catcher's push against the caught kind's resistance,
  `R = H D(p_catcher) / D(r_m, caught)` (`carry`), no mean. Calibre: founding. Status: built; replaced by the founding as flow (planned).
- Kind density in a run (`Substrate::found`): the density is `mass / t`,
  with `t = |delta(lo, 1) - delta(hi, 1)| / (delta(lo, 1) + delta(hi, 1))`;
  beads lie at the local maxima of the superposed hold and are parted at its
  minima. Calibre: founding. Status: built; replaced by the founding as flow (planned).
- Bead (`Substrate::found`): the state is the recipe mass; the reach and
  phase are those of the most abundant kind, with phase `360 frac(a Z)`
  degrees on its plane (`turn(a Z)` in code); the hold inside its ring is
  the held medium at its reach (`Substrate::held_within`). Calibre: founding. Status: built;
  replaced by the founding as flow (planned).
- Held medium (`Substrate::held_within`, `Substrate::medium`): the hold
  inside reach `r` (core masses) is `H(r) = 1 + sum(states of the beads
  nearer than r)` as founded, and at `t` it is `H(r, t) = H(r) -
  passed(r, t) / F`, with `F` the firmament's founded mass and
  `passed(r, t) = sum_k mass(outflow_k(t - tau(r))) L_k(> r)`, `L_k(> r)`
  the landing share of kind `k` bound for galaxies whose delay exceeds
  `tau(r)`. Calibre: founding. Status: built; the step "nearer than r" is replaced by the
  hold inside an orbit (planned, #116).
- Hold inside an orbit ([model.md](model.md#the-hold-inside-an-orbit),
  #116): body `b` (mass `m_b`, reach `a_b`) orbits `h_b = 1 + sum(m_c, c
  nearer)` with `e = delta_b = h_b / (a_b + h_b)` (LAW-DEFICIT-SHARE),
  apsides `a_b (1 -/+ delta_b)`; its share inside `r` is
  `chi_b(r) = clamp((r - a_b (1 - delta_b)) / (2 a_b delta_b), 0, 1)`
  (`circuit::beyond_share` from the inside); the hold is
  `H(r) = 1 + sum_b m_b chi_b(r)`. One model for `Substrate::held_within`
  and `galaxy::inside_mass`. Calibre: founding. Status: planned.
- Vortex ([model.md](model.md#the-vortex-core-swirl-twist-writhe-linking),
  #121): core the filament, `X(t)`, `t = U / W`; stretching `D(n)^t` along
  it; circulation about the held medium `Gamma(r) = 2 pi H(r) r / (r +
  H(r))`; vortex lines the Hopf fibres `S^1 -> S^11 -> CP^5`; twist `n`
  per echo; writhe `+/-120 n` degrees per echo; `Lk = Tw + Wr` conserved,
  changing by one only at the pinch, `N* = 3 (4 + a) / (2a)` echoes; a
  streamline is the throat orbit's sheared ellipse, `e = delta`, apsides
  `r (1 -/+ delta)`, `360 delta` degrees a traversal. Open: the core
  curve's shape between tiers, mutual linking `Lk_ij`, fibre separation in
  `CP^5` in shares, helicity without its `Gamma^2` self term, and the
  per-plane twist by separation (#128). Calibre: one per tier, nested vortices dilated conformally. Status: planned; replaces `galaxy::Structure` bands,
  `galaxy::Orbit` circles and `chain::Place` points.
- Rim (`Substrate::found`): the rim is `max R (1 + s_final)` over every
  shell. Calibre: founding. Status: built; replaced by the founding as flow (planned).

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
    the deficit contrast `T` times `c`; the catcher's push carries what
    it takes.
  - Perturbation: the held share `m / (r + m)` of the echo on the parcel's
    states (`loop_flux` vacuum channel), never on its reach.

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
    exponent; phi_i(t) = t theta_i mod 1 in QQ/ZZ, i = 1..5; whole
    traversals in ZZ and each plane's order)`, never one scalar; durations
    (`2 pi M / s` per traversal, in `r_g / c`) are display through a
    certified pi. Turning in degrees per traversal on the orbit's plane:
    `360 (delta(r) / r) (M / s)`,
    exact.
  - Local ring at reach `r`: one traversal of the orbit at `r`, lasting
    `s r (r + M) / M^2` core traversals (exact; pi cancels); every law step
    at a reach counts local traversals.
  - Twist and dipole, spin `a = m / g` (`g` the spin's grid), lowest common
    denominator `L = 4g + m`:

    ```text
    s         = (2g + m) / L      torsion share (open, the Q pole)
    n         = m / L             twist per echo, 2s - 1 = a / (4 + a)
    s_Q       = (2g + m) / L      (1 + n) / 2
    s_Z       = 2g / L            (1 - n) / 2
    s_Q / s_Z = (2g + m) / 2g     (1 + n) / (1 - n) = 1 + a / 2
    ```

  - Influence, over `L^2` and powers of two:

    ```text
    dn/da         =  4g^2 / L^2
    ds_Q/da       =  2g^2 / L^2  = -ds_Z/da
    d(s_Q/s_Z)/da = 1/2
    dD(n)/dn      =  L^2 / 8g^2
    d delta/dr = -M / (r + M)^2,   dD(delta)/dr = -2M / r^2
    ```

  - Seed 7 (`g = 2^40`, `m = 381123025677`,
    `L = 4779169536781`, the denominator of the founding's torsion share):

    | Reading | Numerator over `L` |
    |---|---|
    | `n` | `381123025677` |
    | `s_Q` | `2580146281229` |
    | `s_Z` | `2199023255552` (`2^41`) |

    `s_Q / s_Z = 2580146281229 / 2^41`; the core's full return is `L` rings;
    over `L^2 = 22840461461295518111841961`: `dn/da = 2^82 / L^2`,
    `dQ/da = 2^81 / L^2`; `dD(n)/dn = L^2 / 2^83`.


- Firmament (`Substrate::found`): the firmament holds the held kinds inside
  the vessel, with spin equal to reach on the 2^-64 grid and phase
  `360 frac(a Z)` degrees on its plane (`turn(a Z)` in code); amounts are
  scaled by `1 / sum mass_of(z, amount)` so that the firmament's mass is 1. Calibre: founding. Status: built.
- Mass of an amount (`circuit::mass_of`): `amount (Z + Q) / hold_Z`.
  Calibre: none. Status: built.
- Held share (`Substrate::held_share`): the held share is
  `firmament mass / (firmament mass + sum bead masses)`. Calibre: none. Status: built.
- Throat push (`circuit::throat_push`): `p = delta(H, H) = 1/2`. Calibre: none. Status:
  built, read bare: the convergent face, unresisted by the divergent face
  1/2, so the net push at the equator is 0 (S114, #195).
- Dilation per echo (`xyl_core::shares::dilation_dy`), one law for every holder:
  `rho = (1 + p) / (1 - p)`, `p` the push the holder receives. Calibre: none. Status:
  built; its callers pass the gross push, to become the net push
  `p (+) (-r)` (S115, #195).
- Jet dilation (`shares::dilation_dy` of the torsion share): the continuum carries the
  throat's torsion share `s = (H + J) / (2H + J)` at each crossing, so
  `rho = D(s) = (1 + s) / (1 - s)`: 3 at `s = 1/2` (no spin), 4 at
  `s = 3/5` (the universal core's extremal spin). Calibre: none. Status: built.
- Echoes to the throat (`circuit::echoes_to_throat`), one law for the
  firmament's kinds and a star's collective: `n = ln(1 / t) / ln rho`, zero
  where `t >= 1`, none where `t <= 0` or `rho <= 1`. Calibre: none. Status: built.
- Reverberation (`Substrate::reverberation`): in order of spin,
  `h = 1 + inside / unit`, `r = spin / h`, `delta = h / (r + h)`,
  `beyond = clamp((r (1 + delta) - h) / (2 delta r), 0, 1)` and
  `rate = beyond / bell`. Calibre: founding. Status: built.
- The equator: inflow, viscosity and pairs
  ([model.md](model.md#the-equator-inflow-viscosity-and-pairs), [#60](https://github.com/SAIOS-SAMN/xylarium/issues/60)).
  Status: planned (derived).
  - Continuity: `s (+) (-s) = 0`, so `D(s) D(-s) = 1`; `D(s) = 3 + a`;
    equatorial inflow per echo `1 - 1 / D(s) = (2 + a) / (3 + a)`;
    `Z / Q = 1 / D(n)`.
  - Swirl: `Gamma(r) = 2 pi M r / (r + M)`, the passed share.
  - Viscosity: `nu(t) = t / (1 + t)`, `t = U / W`; reverted
    `nu(1 / t) = 1 - nu(t)`; `nu(1) = 1/2`.
  - Positron: `R(-x) = -R(x)`, `R = kappa (1 - 2x)`, `x` in QQ/ZZ.
  - Pair share per echo: `p_pair = s (+) (-(1 - s)) = 4n / (3 + n^2) =
    a (4 + a) / (12 + 6a + a^2)`; `D(p_pair) = (3 + a)(2 + a) / (6 + a)`;
    `p_pair / 2` electrons (held face) and `p_pair / 2` positrons (open
    face); 0 spinless, `5/19` at `a = 1`.
  - Over `a = m / g`: `D(s) = (3g + m) / g`; inflow `(2g + m) / (3g + m)`;
    `p_pair = m (4g + m) / (12g^2 + 6gm + m^2)`;
    `D(p_pair) = (3g + m)(2g + m) / (g (6g + m))`.
  - Seed 7 (`g = 2^40`, `m = 381123025677`): `D(s) = 3679657909005 / 2^40`;
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
    `n / 3 = a / (3 (4 + a))`, over `a = m / g`: `m / (3 (4g + m))`.
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
    `J_j` the plane's part of the spin, `sum_j J_j = J`, `J_0 = 0`.
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
  moment, a kind is held by the throat exactly when it has no open states
  (`compound::open = 0`): it has no pole state, and the jet never acts on
  it. With its net axis weight `A`, an open kind dilates by
  `D(s) D(n)^A` per echo and is held where `D(s) D(n)^A <= 1 - 2 delta`;
  on the continuum alone, `A <= -log_D(n) D(s)`, for seed 7 `A <= -8`
  (`D(n)^8 >= D(s) > D(n)^7`, exact): 461 open kinds held, Z 65-124,
  279-338, 422-762 (planned; the test below covers the moment-free map). A kind with `U` open and
  `W` held count crosses within `ln(W / U) / ln D(s)` echoes on the
  continuum alone, since the map `t -> a t + b` has `a >= D(s) >= 3` and
  `b >= 0`. Over the founded kinds (seed 7) 20 are held, Z 2, 4, 28, 30, 32,
  34, 58, 208, 210, 234, 236, 260, 262, 264, 266, 268, 270, 272, 418, 1092,
  with 43.5% of the founded mass; 524 of the vessel's 525 kinds are open.
  Z 1093 to 1700 hold 22 closed kinds (Z 1094, 1118, 1272-1276, 1298-1302,
  1326, 1328, 1352, 1354, 1378-1396, even), unfounded: the founding stops
  at the vessel's seal, Z 1092. Calibre: none. Status: built (the test
  `a_kind_is_held_exactly_when_it_has_no_open_states`), and the firmament
  is the closed kinds (`circuit::Substrate::found`).
- Jet onset (`circuit::onset_echoes`): `circuit::throat_crossing` of
  `t = open / hold` under the kind's jetstream map at the torsion share `s`
  and the kind's vacuum share `delta`, `a = D(s) / (1 - delta)`,
  `b = delta / (1 - delta)` (`circuit::jet_map`); a sealed kind has
  no pole state and no onset, and the onset is zero where `t >= 1`.
  Calibre: founding. Status: built.
- Outflow (`Substrate::outflow`): `m(t) = m0 (1 - exp(-rate (t - onset)))`
  for `t > onset`. Calibre: founding. Status: built.
- Outflow rate (`Substrate::rate`):
  `q(t) = sum m0 (1 + e_z) rate exp(-rate (t - onset))`, with
  `e_z = open_z / hold_z` (`formation::emission`). Calibre: founding. Status: built.
- Inbound echo (`Substrate::inbound`):
  `A_in(t) = cos(2 pi t / echo) + echo q(t)`. Calibre: founding. Status: built.
- Inbound peak (`Substrate::inbound_peak`): the peak is
  `1 + echo max_j q(onset_j)`. Calibre: founding. Status: built.
- Inbound mean (`Substrate::inbound_mean`): the mean is
  `echo (sin(2 pi t / echo) / 2 pi + Q(t)) / t`, with `Q` the state out by
  `t`. Calibre: founding. Status: built.
- Push at reach (`Substrate::push_at`):
  `p(R, t) = h / (R + h) (1 + A_in(t))`. Calibre: founding. Status: built,
  read bare and unbounded (S116, #195).
- Band past the throat at `t` (`Substrate::beyond_at`): the share is
  `beyond` with the deficit replaced by `p(r, t)`. Calibre: founding. Status: built.
- Pulse phase (`Substrate::pulse`): the phase is `frac((t / echo) / t_u)`,
  with `t_u = floor(2 (rim + 2)) + 1` echoes. Calibre: founding. Status: built.
- Jetstream (`Substrate::loop_flux`, `circuit::pole_flux`): on a kind of hold
  `w` and open states `u`, the filament channel moves
  `(rho - 1) u w / (w + rho u)` from Z to Q, the vacuum channel moves
  `p(r, t) w` from Z to Q (none where `p >= 1`), and the echo channel
  moves `R u` from Q to Z. Each channel is a dilator applied to the
  state left by the previous channel; the residual is the observed dilation
  over the channels' product and equals 1 where the loop closes. Calibre: founding. Status:
  partial (read by the observer).
- Jetstream law (`circuit::jetstream`), one law for every holder of hold `w` and
  open states `u`: continuum (outflow) at push `p_c`, `rho = (1 + p_c) / (1 -
  p_c)`, moves `(rho - 1) u w / (w + rho u)` Z to Q; vacuum (inflow) at
  `p_v = H / (R + H)` moves `p_v w` Z to Q; electron phase moves `R u`
  Q to Z (mean zero over an echo). `rho_net` is the product of the
  channels' dilations, the torsion the echo leaves. Firmament kinds: `p_c = 1/2`
  at the throat's reach. A star's collective: `p_c = p_v = delta`, the vacuum's
  share at the star's rim about the star's mass, as a firmament kind's is read
  at its reach about the kind's hold. Mean echo map (`circuit::jet_map`): `t ->
  a t + b`, `a = rho / (1 - p_v)`, `b = p_v / (1 - p_v)`. Firmament kinds: `p_c
  = 1/2`, `p_v` the kind's vacuum share at its reach about its hold.
  Calibre: founding. Status: built for both; the vacuum adds to the
  continuum instead of resisting it, where the net is `p_c (+) (-p_v)`
  (S117, #195).
- Throat crossing (`circuit::throat_crossing`): iterating `t -> a t + b`,
  `t_n = t* + (t0 - t*) a^n` with `t* = b / (1 - a)`, so the throat falls
  after `ln((1 - t*) / (t0 - t*)) / ln a` echoes; `circuit::echoes_to_throat`
  is the case `b = 0`. Calibre: none. Status: built.
- Timeline (`stars::timeline`, `stars::remnant`): fusion steps run their
  transduced share over the transduction per bell until the throat
  crossing (a step running there is cut), fission steps from it; the
  remnant is one less what ran. Calibre: mixed, founding and core (#123). Status: built.
- Waveform (`Substrate::amplitude`):
  `A(r, t) = [cos(2 pi (t - tau) / echo) / echo + q(t - tau)] H / (r + H)`,
  with `tau = (r / rim) bell / 2` and `H = H(r, t)` the held medium inside
  the reach (1 at `r = 0`, where the share is 1). Calibre: founding. Status: built.
- Carry (`Substrate::carry`, `circuit::Carry`): at reach `r` the medium
  carries `delta = H(r, t) / (r + H(r, t))` of the pulse; the ocean
  carries `s (+) (-(1 - s)) = (2s - 1) / (1 - s (1 - s))` outward per echo
  (the near face's open share out composed with the far face's held share
  back; the pair share); and each traversal at `r` turns `360 delta`
  degrees on each isoclinic plane, returned after `1 / (1 - delta)`
  traversals. Calibre: founding. Status: built.
- Pull and delay (`Substrate::ocean_shares`): the pull is `h / (r + h)` with
  `h = state / mean state` (the home galaxy has 1 at reach 0); a galaxy's
  share is its pull over the sum, and its delay is `tau`. Calibre: founding. Status: built.
- Coherence (`circuit::coherence`): `1 - 2 min(d, 1 - d)`, with `d` the
  phase difference on their shared plane as a share of 360 degrees.
  Calibre: none. Status: built.
- Landing (`Substrate::landing`): a galaxy's share is its pull times its
  coherence over the sum, or the pull alone where every coherence is zero.
  Calibre: founding. Status: built.
- Intake and composition (`Substrate::intakes`, `Substrate::composition`):
  the intake is the landing share times `outflow(t - tau)`, and the
  composition is the recipe plus the intake. Calibre: founding. Status: built.
- Books (`Substrate::books`): mass is conserved as held + left = founded and
  taken + in flight = left. Calibre: none. Status: built.
- Push on a galaxy (`Substrate::push`):
  `l(t) = R inside + d [Q(t - tau) + sin(2 pi (t - tau) / echo) / 2 pi]
  inside / ((R + inside) h)` and `reach = l / inside`, with `inside =
  H(R)` the held medium at the galaxy's reach and `h` its state over the
  mean; the recession is `dl/dt / inside`,
  and the shift is `z = recession / c_ocean`. Calibre: founding. Status:
  built, the recession unresisted by the hold inside the reach (S118,
  #195).
- Tide (`Substrate::tide`): `A(R - x, t) - A(R + x, t)`. Calibre: founding. Status: built.
- Galaxy selection (`Substrate::holds_a_star`, `Substrate::galaxies`): a
  bead is a galaxy where `lit x state >= m_p` and `state <= 1` (core
  masses), with `m_p = s c` the star mass of the piece that lights first
  (`galaxy::first_star_mass`). Calibre: founding. Status: built.
- Galactic core mass (`cores::founding_of`): `hole_suns = hole_mass x state`.
  Calibre: mixed, founding and core (#123). Status: built.
- Circulation (`cores::founding_of`): the circulation is `1440 delta / echo`
  degrees per Gyr on the galaxy's orbital plane, with `delta = H / (reach + H)` and `H = H(reach)` the held medium. Calibre: founding. Status: built.
- Galaxy plane (`cores::founding_of`): the node lies at
  `index x` the golden angle about the throat's axis, and the tilt toward
  it holds `index x` the plastic angle's share of the S^360 measure (a
  placement by index, roadmap item 40) (`galaxy::node_of`, `galaxy::tilt_of`). Calibre: none. Status: built.
- Galaxy scale (`cores::Shape::scale`): `2 m / (1 + m)`, with
  `m = hole_suns / hole_mass`. Calibre: SI seam. Status: placeholder.

### Perturbations, throats and radiation

- Perturbation front (`bounce::curved_reach`, `Anchor::bell`): one sphere
  per bounce, grown from the core across its enclosure of curvature radius
  `rho`; reaches are `rho sin(R / rho)`; the bell is the front's round trip
  to the home galaxy's rim, `2 rim 60^5` in `r_g / c`. Calibre: mixed, founding and core (#123). Status: built.
- Throat (`bounce::horizon`): the sphere through the crossing at the
  one-third deficit, `R_h = 2M`, with a near and a far face. Calibre: none. Status: built.
- Echo (`circuit::echo`): two rings, the near face's ring and the far
  face's return. Calibre: none. Status: built.
- Equator (`circuit::throat_push`, `circuit::axis_antipole`):
  `delta(H, H) = 1/2`, the fixed point of `delta -> 1 - delta`; on the pole
  axis the inversion `X(t) -> X(1 / t)` fixes `t = 1`. Calibre: none. Status: built.
- Arrival (`Substrate::amplitude`): `(r / rim) bell / 2` after the bounce,
  amplitude carried as `H(r, t) / (r + H(r, t))`. Relay between holes
  (`pulse::relays`): continuity `cos(2 pi detour / wavelength)`. Calibre: founding. Status:
  built; the single front echoed at surfaces with delay `r + H(<r)` is
  under review (roadmap, milestone 3).
- Irradiation, held and passed: received `h / (pi (r + h))` per bell about
  the hold `h` inside the reach (`stars::ring_per_bell`; a piece's cut
  about the held medium, a star's life about the core's hold 1 until the
  galaxy tier's medium is carried, roadmap item 44); held `delta = M / (R + M)` of it; passed
  `1 - delta`; received = held + passed. Calibre: mixed, founding and core (#123). Status: built for stars
  (`stars::transduced_per_bell`); next for worlds (layer C).
- Emission: a transducing body returns its held share outward; a star's
  power is `P = m transduced_per_bell(r) / bell` (`stars::power`); a nucleus
  radiates `open / weight` (`formation::emission`). Calibre: mixed, founding and core (#123). Status: built.
- World heat (next, layer C): a world holds its deficit share of its star's
  light at its reach; its temperature follows from the held share; its
  re-emission is that heat turned outward. Replaces `278 L / a`,
  340 W/m^2, 288 K and `greenhouse_tau`. Calibre: SI seam. Status: placeholder.

### Z, Q and R (the pole model)

- Weights (`circuit::nucleus_weights`, `circuit::mix_weights`): `Z` is the
  proton count and `Q` the neutron count (the mass number
  `A = floor(a + 1/2)` less the charge, `circuit::mass_number`,
  from the element table; a cluster sums its parts), and a kind weighs
  `Z + Q` nucleons, the mass weight `m_Z` (ruled #151: the hold; the
  capacity `cap` is named apart). Calibre: none. Status: built.
- Electron phase state (`circuit::electron_phase`):
  `R = kappa (1 - 2 frac(p))` in `[-kappa, kappa]`, with `p` the bounce's
  pulse as a share of 360 degrees on the isoclinic planes and `kappa` the electrons' coherence on S^11 (`fibres`,
  `2 G / ends` for the charge's electrons); `R` is zero for a kind with no
  charge. Calibre: none. Status: built.
- Pole point (`circuit::pole_point`, `circuit/poles.rs`): `X(t)` lies on the
  pole axis at `t = open / hold`, with the Z pole at the origin `e_o`, the Q
  pole at the point at infinity `e_inf` and the throat at `t = 1`; the
  inversion is `X(t) -> X(1 / t)` (`circuit::axis_antipole`). Calibre: none. Status: built.
- Dipole ([model.md](model.md#the-dipole-3z3q-and-3q3z-across-r), #148): the Z face
  `3Z/3Q + R` and the Q face `3Q/3Z - R`, `3Q = 3 cap`, swapped by the
  inversion `X(t) -> X(1 / t)` with the sign of `R`; `R` is the faces'
  separation. Calibre: none. Status: planned.
- Capacity and balance (`atoms::capacity`, `atoms::balance`): `cap =
  sum_(n <= n_out) sum_(l < n) 2 d(l)` over the turnings the atom reaches,
  `3Q = 3 cap`; `R_Q = 6Z - 3 cap`, `rho_Q = (2Z - cap) / cap`. A cluster
  sums its parts (`circuit::kind_capacity`), a mix its whole nuclei
  (`circuit::mix_capacity`). Calibre: none. Status: built.
- Seals as the dipole's equator: `rho_Q = +1` exactly where `Z = cap`, so
  `3Z = 3Q`, and exactly the sealed kinds (Z 2, 28, 208 and 1092;
  `atoms::Atom::sealed`), where the faces read `1 + R` and `1 - R`. Every
  seal is closed (`u = 0`), so its pole point is `t = 0`, the Z pole, not
  the throat (`t = 1`). Calibre: none. Status: built (identities over Z 1
  to 1700, `atoms::tests`).
- Axes (`circuit/axes.rs`): the pulse has the six planes of
  `xyl_core::hopf`: the axis plane on base axes 1-2, its first the pole
  axis, and five isoclinic planes on base axes 3-4, 5-6, 7-8, 9-10 and
  11-12 (`circuit::plane_blade`; `cga` counts axes from 1, model.md from
  0). 180 degrees on each isoclinic plane
  (`circuit::plane_antipole`) sends a point to its antipode, and together
  with the axis's inversion it forms the pulse's antipole
  (`circuit::antipole`). The antinode share of harmonic `q` is 0 for even `q`
  and `1/q` for odd `q` (`circuit::antinode`). Calibre: none. Status: built.
- The capacity's occupancy, opened ([model.md](model.md#the-capacitys-occupancy-the-vents-to-open), #158):
  per band `(n, l)` the empty slots, single and paired orientations (room,
  open, held), summing to `cap - Z`, `U` and `2P = Z - U` (built:
  `atoms::Occupancy`, `atoms::band_occupancy`); per plane `2P_k`,
  `U_k`, `t_k = U_k / 2P_k` (built on each of the six planes by its charge
  `m_p`, level multiplicity `C(l - |m_p| + 9, 9)` on every plane:
  `atoms::Band::plane_levels`, `atoms::plane_sides`, E016; the planes part
  only by their fill shares, alike in the common phase); the weights per
  plane, open `sum m U_m = A_p` and held
  `sum 2 m P_m` (built, `atoms::plane_weights`; per band `-2 A_b` at
  least half full, `0` below), held and open the
  isoclinic winding read mod 2; pairing and stripping as coboundaries,
  the occupancy, the capacity and the windings' total the class; every law
  reading the held count `W` re-read on the capacity; seams
  `u(z - 1) + 1 - u(z) < 0` read per band and plane. Calibre: none. Status:
  planned.
- Structure (`galaxy::Structure`): the rings are the beads sorted by reach,
  with edges `span x galaxy_kpc` and per-kpc holding `state / width`; the
  core reach is one galaxy unit, the nucleus is the first ring's outer edge,
  and the reference ring is where the cumulative holding crosses one half.
  Calibre: founding. Status: built; its rings are bands, retired with the hold inside an
  orbit (#116, #6).
- Crowding (`galaxy::density`): `lambda_ring / lambda_reference`. Calibre: founding. Status:
  built.
- Stellar mass (`galaxy::stellar_mass`): `Lighting::lit` core masses.
  Calibre: none. Status: built.
- Mass inside (`galaxy::inside_mass`):
  `1 + rings within r / all rings` core masses, with a ring counted by the
  share of its span inside `r` (galaxy units). Calibre: founding. Status: built; to be
  replaced by the hold inside an orbit (planned, #116). Its divisor, all
  rings, is taken as the core's state 1; the founded beads total 0.4008
  core masses, so the tier's units await a ruling (#116).
- Turning law (`turning::turning`, `turning::turning_dy`):
  `omega = delta(<r) bell / r` radians per ring, with `r` in `r_g`
  (`r Tier::in_rg`), `delta(<r) = M(<r) / (r + M(<r))`, `M` in core masses
  and `bell` in `r_g / c` (`Anchor::bell`). Calibre: core (`r_g`). Status: built.
- Piece of gas (`formation::cooling`): `lambda = (state / unit) / W`; the
  agitation is `a0 = (p - delta) / p` with `delta = h / (R + h)` and
  `h = 1 + held share`; `e = sum s_z e_z` and `mu = lambda e`; `n = 1`
  beyond one galaxy unit and `n = 2` otherwise. Calibre: founding. Status: built.
- Gathering (`formation::gathered`): `m(w) = (lambda + mu) W share_n(theta)`,
  with `theta = 360 w / (2 (w + R))` degrees on the infall's plane
  (`xyl-core measure::share_turns`).
  Calibre: founding. Status: built.
- Grip and shear: the grip is `m / (w + m)`, and the shear is
  `h w / (R (R + w))`. Calibre: founding. Status: built.
- Vacuum halving (`formation::vacuum_halving`): `k_v` is the least whole
  `k >= ln 2 / ln((R + h) / R)`. Calibre: founding. Status: built.
- Heat (`formation::collapse`): the heat is
  `min(1, ring x swing x grip(W) / shed)` (`formation::pulse_heat`, the one
  form for the gas and the disc), with `ring = 1 / (pi (R + 1))` and
  `shed = e + ln 2 / k_v`. Calibre: founding. Status: built.
- Spare (`formation::collapse`): the spare is `grip(w) - shear(w) - heat` on
  `w = W / 2^k` for `k < 48`. Calibre: founding. Status: built.
- Cooling (`formation::collapse`): the gas halves until `a <= max spare`;
  `cools_in` is the least ring `n` with
  `floor(n / P_e) + floor(n / k_v) >= halvings`, where `P_e = ceil(1 / e)`;
  the infall is the largest `w` with `spare(w) >= a`, and the number of
  clouds is `max(1, floor(W / infall))`. Calibre: founding. Status: built.
- Lighting (`formation::lights`, `formation::lighting`): a piece lights where
  clouds > 0 and `cooled + heat < 1/2`; `cools_in` is the least over lit
  pieces, and `lit` is the lit holding over all holding. Calibre: founding. Status: built.
- A star's reach (`galaxy::Star::reach_units`): its orbit radius about its
  galactic core, galaxy units; the star's class, its system's light and its
  worlds' life and minds all read it. Calibre: mixed, founding and core (#123). Status: built.
- First light (`galaxy::birth`): the first light of the piece at the
  star's reach falls at `max(cools_in, 1)` rings; there is none where the
  piece never lights. Calibre: founding. Status: built.
- Birth along the ring (`chem::Chemistry::birth_along`): the birth time is the
  `t` (rings) with `F(t) = F(t0) + u (F(top) - F(t0))`, where `top` is the 30
  Gyr horizon read in rings, found by 48 halvings. `F` is the ring's `sfr`
  integrated term by term, `c p! / k^(p+1) (1 - e^(-kt) sum_(j<=p) (kt)^j /
  j!)`, or `c t^(p+1) / (p + 1)` at `k = 0` (`chem::formed_by`). Calibre: SI seam. Status: built.
- Star formation per ring (`chem::Model::of`): the rate is
  `efficiency x Omega / 2 pi` with `Omega = v_flat / r`; the gas is a sum of
  `c t^m e^(-lambda t)` terms, with infall on the ring's first-light
  timescale toward `galaxy::density x sigma_r0`. Calibre: SI seam. Status: partial (rulers
  `v_flat`, `sigma_r0`, `prompt`, `slow`, `loading` and the ring width).
- Yields (`chem::Calibration`): per-channel scales are set from
  `Rulers::ism_today_dex` at the reference ring at `Rulers::today` (13.8 Gyr in
  rings, the calibration epoch until phase 5); `relative` is 1 in the
  calibration's reference birth gas; hydrogen and helium follow from the
  founding's counts with `Rulers::dy_dz` and `Rulers::reference_metallicity`
  (`chem::Light`). Calibre: SI seam. Status: placeholder; the next step is yields from the
  founding (roadmap, Rules from the founding and the galactic cores).
- Placement (`galaxy::star`): the reach is where the rings' cumulative
  holding reaches the silver-angle pivot; the disc phase follows the golden
  angle on the disc's plane, the ball planes follow the plastic and golden
  angles (a placement by index, roadmap item 40), and the rate
  follows the turning law. Calibre: none. Status: built.
- Piece star mass (`galaxy::piece_star_mass`): `m_p = s c` core masses,
  with `s` the share of the piece's infall the star tier's cut holds
  (`disc::StarHold::star`) and `c` the infall's holding
  (`disc::StarHold::cloud`). Calibre: founding. Status: built.
- Initial mass function (`galaxy::imf`): `m = x m_p` with
  `x = b u / (1 + b - u)`, the inverse of `F(x) = x / (x + b)` on `[0, 1]`,
  and `b = d / (1 + d)` from the crowding `d`. Calibre: none. Status: built.
- Vertical orbit (`galaxy::vertical`, `galaxy::layer`,
  `galaxy::vertical_frequency`): `nu = omega 2d / (1 + d)` radians per ring
  (turning law about `M(<r)`, crowding `d`);
  `h = r m / (M(<r) + m)` galaxy units (`m` the ring's mass,
  `galaxy::ring_mass`); `s = t / (1 + t)` with
  `t = age / cooling time` (`galaxy::birth`); `sigma = h nu s 2d / (1 + d)`;
  `vz = -sigma ln(1 - u)`, `A = vz / nu`, period `2 pi / nu`, exposure
  `atan(h / A) / (pi / 2)` where `A > h`, else 1. Calibre: core (`r_g`). Status: built.

## 6. Stars and systems

- Star's compactness (`disc::rim_share`, `galaxy::Star::compactness`):
  `delta = m / (R 60 + m)`, `m` in core masses, `R` the rim of the cloud's
  settled cut in system-tier units (`galaxy::piece_rim`). Calibre: core (`r_g`). Status: built.
- Collapse and ignition (`disc::star_hold`): dark cut `s0`, `R0`;
  `delta0 = s0 c / (R0 60 + s0 c)`; transduction per echo `delta0 x turning`;
  settled cut at `p0 = 1 + transduced`. Calibre: core (`r_g`). Status: built.
- Ring per bell (`stars::ring_per_bell`): `h / (pi (r + h))` about a hold
  `h`. Calibre: founding. Status: built.
- Transduction per bell (`stars::transduced_per_bell`): `delta / (pi (r + 1))`.
  Calibre: founding. Status: built.
- Held count at heat (`stars::held_count`): `u_z(q)` open states
  (`compound::open_hot`), `w_z(q) = 3 Z - u_z(q)` held; `A_z` the element
  table's `a` rounded to the nearest whole (`stars::nucleon_count`). Calibre: none. Status:
  built.
- Pole state of a star (`stars::mixture_pole`): `t0 = sum n_z u_z / sum n_z
  w_z` over the held mixture (`disc::StarHold::held`), `n_z = share_z / w_z`;
  `t0 <= 1/2`. Calibre: none. Status: built.
- A star's throat (`stars::Sequence::throat_echoes`):
  `circuit::echoes_to_throat` of its pole point under
  `shares::dilation_dy(delta)`, `n* = ln(1 / t0) / ln((1 + delta) / (1 - delta))`,
  echoes from ignition; none where `t0 = 0`. Calibre: none. Status: built.
- Fusion step (`stars::fusion_step`): the least `k >= 2` with a kind at `A'
  = k A_z` releasing `1 - w' / (k w) > 0`, the greatest release there.
  Calibre: none. Status: built.
- Fission step (`stars::fission_step`): kinds with `A1 + A2 = A_z` nearest
  equal halves releasing `1 - (w1 + w2) / w > 0`, the greatest release
  there. Calibre: none. Status: built.
- Sequence (`stars::Sequence::of`): at each stage the held kind of greatest
  release steps (ties by the lesser `A`), transducing `share x release` and
  passing `share x (1 - release)` to its products (fission: in proportion to
  their held count), until no held kind releases; fusion stages, then
  fission stages where `t0 > 0`. Remnant `1 - sum transduced`
  (`stars::remnant`). Calibre: none. Status: built.
- Life (`stars::life`, `stars::ends`): step `i` ends at `sum_{j <= i}
  transduced_j / transduction` bells; the life is the first step's end. The step
  at an age (`field::transduction`) is `Unlit` (no step), `Step(i)` or
  `Remnant`; the transduction-out share (`field::transduced_share`) is the
  step's release. Calibre: mixed, founding and core (#123). Status: built.
- Power (`stars::power`): `P = m transduced_per_bell(r) / bell` core masses per
  ring (`c = 1`). Calibre: mixed, founding and core (#123). Status: built.
- Light (`stars::light`): `P (1 + pi bell q(t))`. Calibre: mixed, founding and core (#123). Status: built. Boundary
  (`stars::suns_of_power`): `P` read in the prototype's display unit of
  luminosity read by the system and world layers.
- Flicker (`stars::flicker`): `(pi / 2) |cos(2 pi t / echo)|`. Calibre: none. Status:
  built.
- Star's cut (`disc::star_hold`, `formation::hold_at`): the spin is
  `h w / (R (R + w))`, the heat is `2 ring_per_bell(r, H(r))` and
  `p0 = 1 + 2 transduced_per_bell(r)`; each kind is carried to
  `l = p p0 (e / e_mean)(w_mean / w) + spin` and is held where
  `h* / (l / h* + h*) >= p`, at the largest fixed point `h*` of the held
  hold. Calibre: mixed, founding and core (#123). Status: built.
- Disc agitation (`disc::star_hold`):
  `a_disc = min(1, turning x swing x grip / shed)` (`formation::pulse_heat`), with
  `grip = (1 - star) / (2 - star)` and `shed = e_rest + ln 2 / k_v`. Calibre: mixed, founding and core (#123). Status:
  built.
- Star's manifold (`galaxy::disc_manifold`): the gathering is
  `min((mass / stellar_mass) / (cloud / infall), infall)`, the reach is
  `manifold::reach(r, m / M(<r)) = 2 pi r m / (M + m)`, and the star share
  is `min(gathering / reach, 1)`. Calibre: mixed, founding and core (#123). Status: built.
- Worlds on the manifold (`disc::manifold_worlds`): degree `k` of 360 lies
  at `x = s + (k + 1)(1 - s) / 360`, and each degree adds
  `(1 - a)(1 - s) / 360`; a world forms where `m / (s + m) >= tide x`, with
  its reach at the gathered matter's mean, and the belts hold
  `1 - s - sum worlds`. Calibre: mixed, founding and core (#123). Status: built.
- Solids (`systems::system`): the solids are
  `M_star / s x Z_rel x Rulers::reference_metallicity` (world masses); a
  region's mass is `holding x solids x enrichment`; regions inside the rock
  line `min Tc(Fe, Si, Mg)` are skipped, and the phase is
  `k x` the golden angle on the disc's plane. Calibre: SI seam. Status: partial (`reference_metallicity`).
- Formation temperature (`systems::Spec::t_form`): `278 L / a`. Calibre: SI seam. Status:
  placeholder.
- Envelope (`systems::system`): a world gathers an envelope when its core
  holds at least 10 world masses, by a declared gathering law. Calibre: SI seam. Status:
  placeholder.
- Settling rate (`systems::settle_about`): the turning law
  `omega(A) = bell [M / (A + M)] / A` per ring with `A` in `r_g`
  (`turning::turning_dy`). Calibre: core (`r_g`). Status: built.
- Kick (`systems::settle_about`): `eps = omega_m(r) / |dw|`, the turning law
  about the pair's mass `m` across its gap `r` (`turning::turning_dy`), and
  `N = age |dw|` with the age in rings (`circuit::rings_of_gyr_dy`); `q` is the least continued-fraction denominator with
  `|s - p/q| N q <= 1`, `c = |1 - 2 frac(q psi_0)|`, and the swing is
  `eps (floor(N / q) c + N mod q)`. Calibre: core (`r_g`). Status: built.
- Merge (`systems::settle_about`): the merge age is the first age with
  `swing x inner / gap >= 1`, found by 48 halvings to the 30 Gyr horizon;
  merges by the settled age (3 Gyr) combine mass, reach and phase
  mass-weighted, and later ages are recorded on the lighter body
  (`World::merged_at`). Calibre: core (`r_g`). Status: built.
- Orbit (`bodies::orbits`): `e0 = swing`; the tilt is `swing / (1 + swing)`
  of 90 degrees toward the axis of the star's plane, and the node and periapsis take the settled phase.
  Calibre: none. Status: built.
- Moon manifold (`systems::moons`): the reach is `2 pi a m / (M + m)`
  (`manifold::reach`) in the world's radii from 5; the budget is
  `min(a_disc x (reach / a_out) x solids, belt pool)`; the world share is
  `m / (m + budget)`; the tide is `s / (x + s)` with `x = a / a_out`; moons
  follow the degree law with no heat and settle about the world. Calibre: mixed, founding and core (#123). Status:
  built.
- Moon forced eccentricity (`systems::moons`): the eccentricity is the
  neighbours' mass over the world's mass, over the distance from the
  nearest first-order resonance, and is at most `1/10`. Calibre: none. Status: built.
- Belt (`systems::belts`): the belt is the remaining pool beyond the
  outermost world out to `a_out`; the comet cloud is a declared share of the
  giants' mass. Calibre: SI seam. Status: partial.
- Resolution guard (`systems::exp`): `exp(x)` is enclosed by
  `[0, 2^-bits]` for `x <= -(bits + 1)`. Calibre: none. Status: built.
- Angle between bodies (`manifold::gap`, `chain::Place::arc`):
  the certified angle `pi / 2 -/+ asin(|a . b| / |a| |b|)`
  (`xyl_core::rotor::angle_between`), the directions exact, read at its
  midpoint; a rotor's inverse is its reverse (`Rotor::inverse`). The
  arc on a sphere is that angle times the chord `R sin(s / R)`
  (`bounce::curved_reach`). Calibre: none. Status: built; to be replaced by the pair
  reading (#118, section 1).
- Disc-mass scale of the manifold: this scale is not yet built; it is the
  next step (roadmap, Worlds on the manifold chain).
- Degree of a star's life (`worlds::star_degree_gyr`):
  `max(life / 360, bell)`. Calibre: SI seam. Status: built.
- Orbital speed (`turning.rs`): `v = c delta(<r)`, which is less than `c` at
  every reach since `delta < 1`. Calibre: core (`r_g`). Status: built.

## 7. Life and minds

- Fidelity (`life::fidelity`): `eps0 = exp(-d E_weak / kT)`. Calibre: SI seam. Status: built.
- Rung (`life::ladder`): `bits_s = b / eps^(s+1)`, capped at `I_MIND`, with
  at most 12 rungs. Calibre: SI seam. Status: built.
- Selection (`life::bits_per_generation`): selection gains
  `min(1, b excess / ln pop)` bits per generation, with `excess = 1/2`.
  Calibre: SI seam. Status: built.
- Climb (`life::climb`): the rung time is `(e^(mu m) - 1) / mu` under
  impacts at rate `mu`, and sterilisation falls at `1 / hazard`. Calibre: SI seam. Status:
  built.
- Strikes (`life::strikes_per_gyr`):
  `debris x orbits per Gyr x R / (2 pi a)`. Calibre: SI seam. Status: built.
- Viability (`life::viable`): a viable people has 500 bodies. Calibre: SI seam. Status:
  placeholder.
- Forecast (`minds::schedule`): a mind appears at the last rung where the
  world then feeds a viable people; the first age is one degree of the
  star's life, and the horizon is 30 Gyr (`life::horizon_gyr`). Calibre: SI seam. Status:
  built.
- Ancients (`minds::ancients`): the ancients are core-born minds within
  `minds::ancient_window` (the nucleus's first light) of the core's first
  core-born mind. Calibre: SI seam. Status: built.

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
| LAW-DEFICIT-SHARE | `delta = M / (r + M)`, the convergent face; `1 - delta` the divergent face | core | `xyl_core::shares::deficit`, `xyl_core::shares::faces` |
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
