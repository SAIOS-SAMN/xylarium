# The physical model

This document describes Xylarium's physics: the anchor and its units, the founding,
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

### Turns: axis, degrees and the return

Every turn names its axis: the
plane it turns, by the generators that span it (the axis plane, base axes
0 and 1; the isoclinic planes, base axes 2-3, 4-5, 6-7, 8-9 and 10-11; a
turning between planes, by the axes it mixes). A turn on one plane never
stands for the others, so the dimensional capacity is kept. Its physics is
torsion, stated as degrees of modification (a change of phase or of kind),
of structure (a turning held as form, such as the twist held as torsion)
or of perturbation (the echo's swing on the occupied states). Its inverse, the return
trip, is stated with it and is read from the gravitational density the
return passes through, the deficit `M / (r + M)` at the return's reach
(axiom 2), its reach and hold read in one calibre (Calibre and
dimensional capacity), across the attractor pair of the faces (T17): never assumed
equal to the outbound turn. Where the code still holds an angle as a
share of a whole turn, the text gives it in degrees on its plane, and the
code's change is roadmap item 28.

Angles are held as exact degrees on a named plane where the rate is
rational and are read in degrees: the coordinate (`xyl_core::coordinate`, LAW-COORDINATE v1) reads
a value in sectors of base 60 to any depth, exactly, and writes degrees and
their sexagesimal places as display; on the T lane the whole rings are
kept (`coordinate::beat`). Phases about a throat are exact: a
kind's capture phase is the torsion share turned Z times, whose denominator
divides the share's, so it never grows. Only drawn values carry a grid,
the 2^-40 resolution of the draw.

### Pairs: the pivot, per plane

Planned ([#118](https://github.com/SAIOS-SAMN/xylarium/issues/118)). A reading between two things (a separation, an arc, a
closeness: every `a, b`) keeps the dimensional capacity of what it relates
and reads both in one calibre:

1. `a` and `b` are coordinates of one origin (LAW-COORDINATE), read in
   their lowest common enclosure; different origins are refused.
2. Their relative versor is found by conjugation, `P_b^-1 o P_a`
   (Mycelium's pivot engine), and refused where none relates them.
3. It is read per turning plane, by generator, in exact degrees with its
   winding `K` (in ZZ) and its phase (in QQ/ZZ; Winding and phase);
   never summed. The vortex ([#121](https://github.com/SAIOS-SAMN/xylarium/issues/121)) gives the pair its
   object: mutual linking and the fibres' placement in `CP^5`.
4. `d(a, b)` and `d(b, a)` are read across the attractor pair, each return
   from the deficit at its reach, never assumed equal.
5. Closeness is the two-sided form in shares,
   `C2 = sum_k min(w_k, w'_k) (1 - 2 |d_k|) + (1 - m) / 2`.
6. A single angle is shown only for display, as a certified sector.

It retires `rotor::angle_between`, one unnamed plane through
`a . b / sqrt(|a|^2 |b|^2)` and asin; its readers are listed on [#118](https://github.com/SAIOS-SAMN/xylarium/issues/118).

### Winding and phase: the Diophantine split

Planned ([#149](https://github.com/SAIOS-SAMN/xylarium/issues/149)). The model's integer relations are Diophantine: integer
counts related exactly, held in the rationals (QQ) and read back to the
integers (ZZ) where a relation closes.

- **Built.**
  - Nucleons: the span `3Z`, the hold `w = 3Z - u`, the capacity
    `cap = sum_n sum_(l < n) 2 d(l)` and the balance `R_Q = 6Z - 3Q`. The
    seals solve `Z = cap` (Z 2, 28, 208 and 1092). The pole point
    `t = u / w` is a ratio of integers.
  - Orbits: an orbit returns after `q` traversals, `q` the denominator of
    its advance in lowest terms (The ring, Resonance).
  - Settling: the first continued-fraction convergent `p / q` of the phase
    step with `|s - p/q| N q <= 1` (`systems::settle_about`).
  - Placement: four turns held as convergents of their recurrences,
    independent over QQ (`golden_turns`).
- **The split.** A turn held in QQ, in degrees over 360, splits by
  `0 -> ZZ -> QQ -> QQ/ZZ -> 0` into its winding `K` (in ZZ, whole turns)
  and its phase (in QQ/ZZ, the degrees within the turn). Both are carried,
  never merged: a ring count or a value in radians merges them, `frac`
  drops the winding, and with pi no advance is a rational part of a turn.
  `coordinate::beat` keeps both on the T lane.
- **Calibre-free.** A change of calibre multiplies by a rational, `60^p`,
  and keeps every relation in QQ: `u / w`, `p / q` and a dipole's faces
  (The dipole, section 2).
- **Where it is collapsed.** The turning law gives radians per ring
  ([#127](https://github.com/SAIOS-SAMN/xylarium/issues/127)); `settle::period` rebuilds `p / q` from a dyadic step instead
  of carrying it from the laws that produce it.

### Push and resistance

Ruled 2026-11-11 ([#195](https://github.com/SAIOS-SAMN/xylarium/issues/195)). A push is offset by its resistance, through the
flow law with the resistance reversed:

```text
p_net = p (+) (-r) = (p - r) / (1 - p r)        D(p_net) = D(p) / D(r)
```

- **The balance.** At `p = r` the net push is 0 and the dilation is 1.
- **The sign.** A net push below 0 carries the holder back: the resistance
  wins.
- **The resistance.** Unless a law names another, it is the hold of the
  medium at the reach, read through its deficit face. The expansion law
  already reads its push this way: the continuum `H / (r + H)` against the
  vacuum `m / (r + m)`.
- **Speeds.** A speed is a push in `(-1, 1)` of `c`. Finite compositions
  stay below 1, so light speed is a limit no finite set of states reaches.
  A drive's exhaust is the composition of the states it produces, each
  offset by its resistance (civilizations.md, Ley lines and the worth law).
- **Where it is not yet read.** The sites that read a push bare or by
  subtraction are registered as stand-ins, class P, S114 to S128
  ([assumptions.md](assumptions.md)). The founding's sites move its matter,
  so they land with the capacity face (#158, #172).

### Faces of the dipole on S^n

Ruled 2026-11-11 ([#197](https://github.com/SAIOS-SAMN/xylarium/issues/197)). A reading resolves as many faces as its sphere carries,
not two. The two-face law (`shares::faces`) is the case of one dipole.
Letters: `h` held, `o` open; `c` is the speed of light.

- **The faces.** `S^n` in `R^(n+1)` has `N = n + 1` axes, each a dipole
  with two antipodal poles. Its `2N` faces are the cells nearer one pole
  than any other, each `1 / (2N)` of the sphere's measure by symmetry.
  A complex carrier `S^(2m-1)` in `C^m` reads its dipoles as its `m`
  planes: the Hopf carrier's six on `S^11` (#183), and the universe's
  `S^360` has 361.
- **A body's weights.** `w_k >= 0` with `sum w_k = 1`, the share of the
  body's hold on dipole `k`:
  - a state, by its plane shares `|m_k| / l` (`hopf::plane_shares`);
  - an extended body, by the caps' shares of its volume
    (`measure::share_turns`);
  - an isotropic body or medium, `1 / N`.
- **Faces per dipole.** At reach `r`, with `m_k = w_k m`:
  `h_k = m_k / (r + m_k)`, `o_k = r / (r + m_k)`, `t_k = r / m_k`, and the
  push is the faces' imbalance `p_k = h_k - o_k = 2 h_k - 1`, with
  `D(p_k) = h_k / o_k = 1 / t_k`. A share is a face, not a push: the share
  `h` is the push `2h - 1`, and its complement `1 - h` is the push's flow
  inverse.
- **Resistance per dipole.** The medium's hold on the dipole, `H_k = sigma_k
  H`, with face `rho_k = H_k / (r + H_k)`. Offset by the flow law
  (Push and resistance), in poles: `t_net,k = t_k / t_rho,k = H_k / m_k`. The
  reach cancels, and the balance is `m_k = H_k`, the expansion law's rest
  point (`H = m`).
- **Capacity.** A reading carries `N` components, `v_k = p_net,k c`, each in
  `(-c, c)`. They are read per face and never summed into one speed (no
  root). The reading's capacity is `N c`: the more dipoles it engages, the
  more it carries.
- **First degree.** Every coupling reads a face: `h = M / (r + M) = 1 /
  (1 + t)`, the hold and the reach in one first-degree denominator, never
  `M / r^2` or `M / d` ([#199](https://github.com/SAIOS-SAMN/xylarium/issues/199)). A source is not flux spread over a shell of
  area `r^2`; a traversal is a turn of `360 h` on its named plane, between
  an attractor pair. So the face is finite at `r = 0` (`h = 1`), carries
  no leftover length (no dimensional coupling constant enters it), and
  holds its share far out (`h = 1 / (1 + t)`, not `1 / t^2`). Sites that
  still read a square, a root, or a reach without its hold are registered
  (class Q, S133 to S135, with S025 and S034).
- **Reduction.** For `N = 1`, `w = sigma = 1`, these are the two-face law,
  the faces' push and the Push and resistance law.
- **Open.**
  - The expansion law reads its pushes as deficit shares (`p_c = H / (r +
    H)`), which gives a reach-dependent rate. Under the faces law the reach
    cancels per dipole. Which form governs is to rule (EQUATIONS.md, Updated
    equations, I002).
  - The medium's weights `sigma_k` are to derive. Isotropic `1 / N` stands
    until a law gives them.
  - A reach per dipole is open with #128.

## 2. Units and the anchor

- Base units are set by the universal core: `G = c = M_core = 1`, so the
  core's mass, called the universal state, is 1. The unit of length is the
  core's gravitational radius `r_g = G M_core / c^2` (`xyl-core units.rs`).
- Each tier (see [glossary.md](glossary.md)) has a length unit that is an
  exact power of sixty of `r_g` (`xyl-core units.rs` `Tier::in_rg`):
  universe `60^10`, cluster `60^7`, galaxy `60^5`, system `60` and planet
  `60^-2`. A conversion between tiers multiplies by a power of sixty
  (`manifold::dilation`) and is exact. The substrate's reaches are in
  cluster units (`60^7 r_g`) and every star's reach in galaxy units
  (`60^5 r_g`): lengths, never rings
  ([#115](https://github.com/SAIOS-SAMN/xylarium/issues/115)).
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
- **Ring has one meaning: the harmonic of the bell, echo and pulse.** A
  ring of time is one bell; counts of time (cooling, first light, a star's
  transduction and life) are whole numbers of rings, and the ring a place
  receives per bell (`stars::ring_per_bell`) is that harmonic's share
  delivered there. A throat's echo is its sheared return at its torsion
  share `s`: `bell / (1 - s)` on the near face and `bell / s` on the far
  face, two bells each only at zero spin (`circuit::echo_at`). A one-way
  crossing to reach `r` takes `(r / rim) bell / 2`, a fraction of a ring.
  A body is never held in a ring: it is caught in an orbit on the plane of
  the tier above, and the hold inside a reach is what the orbits nearer
  carry within it
  ([The hold inside an orbit](#the-hold-inside-an-orbit),
  [#116](https://github.com/SAIOS-SAMN/xylarium/issues/116)). The galaxy's
  bands (`galaxy::Structure`, "rings" in the code) are retired with it.
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

### Calibre and dimensional capacity

Ruled (B) on 2026-11-06 ([#123](https://github.com/SAIOS-SAMN/xylarium/issues/123)).

- **Calibre**: the pair of units in which a share's reach and hold are both
  read. Tier-bound (each tier has one), conformal (a change of tier dilates
  reach and hold together by the same power of sixty) and share-preserving
  (`delta(lambda M, lambda R) = delta(M, R)`). A reach in one calibre
  against a hold in another is not a share. Tier `k` reads both in
  `60^(p_k) r_g`.
- **The ruling (B)**: masses dilate with lengths per tier. Each tier reads
  `M_k / (R_k + M_k)`; a hold given in core masses (from suns or kilograms
  at the SI seam) enters tier `k` as `M_core / 60^(p_k)`; a hold the
  founding gives is already in its tier's calibre.
- **Dimensional capacity**: the number of components a reading holds as
  named, separate parts (turning planes, a vortex's fibres, twist, writhe
  and linking, the coordinate's lanes), each in its calibre. A change of
  calibre keeps them; summing, an equal split or two calibres read as one
  loses them. Each component is held as a dipole (The dipole: 3Z/3Q and
  3Q/3Z across R).
- **Every equation names its calibre** in [EQUATIONS.md](EQUATIONS.md), and
  every reading is checked for magnitude in its calibre, not only for
  having moved (CONTRIBUTING.md, Verification).
- **Open**: the entries tagged mixed in EQUATIONS.md, the star's own mass
  ([#145](https://github.com/SAIOS-SAMN/xylarium/issues/145)) and the bead's reading at two tiers ([#36](https://github.com/SAIOS-SAMN/xylarium/issues/36)). Part 1 in scalar form is on draft [#152](https://github.com/SAIOS-SAMN/xylarium/pull/152),
  blocked by [#148](https://github.com/SAIOS-SAMN/xylarium/issues/148): it reads one face of the dipole.

### The dipole: 3Z/3Q and 3Q/3Z across R

Planned ([#148](https://github.com/SAIOS-SAMN/xylarium/issues/148)). Every component of a reading is a dipole: its Z face
and its Q face, with the electron phase state `R` between them.

- **The faces.** The Z face reads `3Z/3Q + R` and the Q face
  `3Q/3Z - R`: `3Z` the span, `3Q` the capacity (`3 cap`), and
  `R = kappa (1 - 2 frac(p))` in `[-kappa, kappa]`
  (`circuit::electron_phase`). The vacuum's inversion `X(t) -> X(1/t)`
  swaps the faces and the pulse's hemisphere flips `R` with them: `+R` on
  the Z face and `-R` on the Q face, and the reverse after the inversion.
- **The separation.** At `R = 0` the faces are each other's inverse. `R`
  is their separation, the shear between the attractor and the inverse
  attractor, never an equal split.
- **The equator.** From built laws, the seals (`rho_Q = +1`,
  `atoms::Atom::sealed`) are exactly where `Z = cap`
  (`atoms::capacity`), that is `3Z = 3Q`. There the faces read `1 + R` and
  `1 - R` and differ by `R` alone. Every seal is closed, `u = 0`, so the
  dipole's equator lies at the pole point's Z pole, `t = 0`, not at the
  throat, `t = 1`: the two ratios are different readings, and the relation
  between them is open step 1.
- **Dimensional capacity counts dipoles.** Each named component (a turning
  plane, a lane, a vortex's fibre, twist, writhe or linking) is held as its
  dipole, and a reading of one face holds half of it. The deficit
  `M / (r + M)` is the held face's share and `1 - delta` the other; a site
  that reads one of them alone loses the other.
- **Calibre-free.** `3Z/3Q` is a ratio of integers and `R` is rational
  (Winding and phase, section 1). A change of calibre, the dilator
  `D(60^p)`, fixes both poles and keeps the faces. A calibre question
  arises only when one pole's absolute size is read alone, as a hold in
  core masses or a reach in `r_g`.
- **Where one face is read today.**
  - The turning law and chronometry's wells each take one face of the
    deficit.
  - The founding's holds (`galaxy::inside_mass`, `turning::home_mass`) are
    read as one magnitude, so the scalar part 1 of [#123](https://github.com/SAIOS-SAMN/xylarium/issues/123) (draft
    [#152](https://github.com/SAIOS-SAMN/xylarium/pull/152)) reads the core's hold as 1 in both the galaxy and the
    cluster calibre ([#36](https://github.com/SAIOS-SAMN/xylarium/issues/36)).
  - The anchor reads one rim at `60^5` for the bell and at `60^7` for the
    radius (`circuit/anchor.rs`).
- **Open, in order.**
  1. The dipole's relation to the pole point `t = u / w` and the hold
     `w = 3Z - u`: the dipole's equator (`3Z = 3Q`) is the pole point's Z
     pole (`t = 0`), so neither is the other.
  2. `R` per plane: the pulse is one share of 360 degrees for the five
     isoclinic planes ([#128](https://github.com/SAIOS-SAMN/xylarium/issues/128)).
  3. The founding's output carried per kind as dipoles; the rim placed by
     the containment chain (dilator, rotor and cap), not by an exponent.
  4. The deficit and the turning law on both faces; then [#123](https://github.com/SAIOS-SAMN/xylarium/issues/123) part 2.
- **The name Q.** Ruled 2026-11-10 ([#151](https://github.com/SAIOS-SAMN/xylarium/issues/151)): two counts, named apart.
  The nucleon `Q` is the neutron count, and the mass weight
  `m_Z = Z + Q` is the hold that inertia, gravity and the founding read;
  the dipole's Q side is the capacity, `3Q = 3 cap` here, the room
  (The six planes: a face per plane). `Q` in backticks is also the
  rational type. Neither reading reproduces the throat: for seed 7
  the throat's `s_Q / s_Z` is `1.1733`, while the firmament's whole-nuclei
  `3Q / 3Z` reads `1.1164` as neutrons and `3.7926` as capacity, and the
  whole founding's `0.7813` and `4.9354` (`observe seals`). `R` is not in
  that comparison; the faces with the mix's `R` at the founding's pulse are
  a further step.

### The state (3W, p, R) and the transitions of formation

Started 2026-11-11 ([#201](https://github.com/SAIOS-SAMN/xylarium/issues/201)). A kind's state is one object with three
components, and each formation mechanic moves one of them.

- **The whole.** `3W = 3(Z + Q)`, the nucleon dimensions, three per
  nucleon: `3Z` protons and `3Q` neutrons, both read from the body
  (LAW-COUPLING v2's `3w`). The letter clashes with the held count
  `W = 3Z - U`; the held count is to be renamed (EQUATIONS, I010).
- **The split.** The dipole `3Q / 3Z` is the axis plane's share split, read
  as faces of the whole (Faces of the dipole on S^n):
  `h = 3Z / 3W`, `o = 3Q / 3W`, pole `t = 3Q / 3Z`, and the push over the
  whole `p = h - o = (3Z - 3Q) / 3W`. Its dilation is the Z face itself,
  `D(p) = h / o = 3Z / 3Q`, and the Q face is its antipole,
  `D(-p) = 3Q / 3Z`.
- **The phase.** `R` is each Hopf plane's phase, `R_p`, joining the common
  phase (#183). It turns the faces by composition, never by addition
  (ruled 2026-11-11, E018): read as a push in `(-1, 1)`, it composes with
  the nucleon split by the flow law, `p_net,p = p_nuc,p (+) R_p`, so the
  dilations multiply, `D(p_net,p) = D(p_nuc,p) D(R_p)`. The Z face reads
  `(3Z_p / 3Q_p) D(R_p)` and the Q face `(3Q_p / 3Z_p) D(-R_p)`; their
  product is one, and `R_p = 0` leaves the bare dipole. #148's
  `3Z / 3Q + R` added a turn to a dilation (I009, closed).
- **The rest point.** At a bounce the medium's push is zero, so the rest
  point is `p_net,p = 0`: `p*_nuc,p = -R_p`, `3Z*_p / 3Q*_p = (1 - R_p) /
  (1 + R_p)`. A kind at `abs(R_p) = 1` has none and is refused. The counts
  are rational; their reading back to whole nucleons is the Diophantine
  split (#149).
- **The universal table** (ruled 2026-11-11, #201). The bounce captures
  first, then converts. The founding shells are read at the capture mass
  `m = Z` (`Q0 = 0`): the stack's push `p_stack = s_final (+) s_i`, the
  throat's final share composed with the stack's own face, against the
  kind's resistance `D(r_m) = W_e / 3`, carries it to
  `R = H D(p_stack) / D(r_m)`. Each parcel then rests at the antipolar face
  of its reach, `Q* / Z = H / R` (the involution `t -> 1/t`, the law's sole
  ground), so at release `Q* / Z = (3Z - U) / (3 D(p_stack))`. A kind's
  `Q_z` is its parcels' rest points weighted by amount, a rational on the 2^-64 grid
  (`circuit::rest_neutrons`); every reading of a nucleus after the shells
  weighs `Z + Q_z`. At seed 7 no kind is refused and `Q* / Z` runs 0.42 to
  557, rising with `W_e`.
- **The two holds.** `3W = 3Z + 3Q` is the nucleon whole. The electron
  hold `W_e = 3Z - U` (`compound::electron_hold`) is the held states the
  bound electrons screen, against which the open channel `abs(R_p) u` acts
  (I010, closed).
- **The transitions.** Each mechanic of formation moves one component:

  | Mechanic | Moves | Keeps | Law |
  |---|---|---|---|
  | Capture and stripping of electrons (`atoms::unpaired_hot`, the jetstream's pole channels) | `R`, and the electrons' open and paired states | `3W`, `p` | built: the jetstream moves the electrons' pole state `t = U / W_held`, held to open, not nucleons |
  | The fill, both ways, per Hopf plane (`atoms::Band::plane_levels`, `circuit::fill_shares`) | the electrons' levels on each plane, so the polarities `A_p` and both dipoles of each plane's split | `3W` | built (ruled 2026-11-11): on plane `p` the share `f_p = abs(R_p) u / (W_e + abs(R_p) u)` enters from the Q side toward Z when `R_p > 0`, none when `R_p <= 0`, at the plane's capture phase; exact shares, never floored |
  | Fusion of `k` nuclei, fission (`stars::fusion_step`, the sequence) | `3W`: summed, or split | the split by weight: `h' = (sum W_i h_i) / (sum W_i)`, the mediant of the parts' `D` | built for `Z`; `Q` read from the universal table, back to whole nucleons |
  | Nucleon conversion, Z to Q and back | `p`, by flow steps: `D(p') = D(p) D(s)` | `3W` | built at the universal core's bounce (ruled 2026-11-11): capture at `m = Z`, then each parcel rests at `Q* / Z = H / R`, the antipolar face of its reach; a kind's `Q_z` gathers its parcels (`circuit::rest_neutrons`). Every galaxy reads this universal table (S138) |
  | A bond | share through the bound electrons `g` between the strata | `3W` | LAW-COUPLING v2 (#241) |

- **What the conversion law must meet.** It keeps `3W`, moves `p` by flow
  steps `s`, carries the charge it moves on `R`, and rests at the natural
  balance of push and resistance: `p_net = p (+) (-r) = 0` (#195), per
  dipole (#197), first-degree in the faces (#199). It is never chosen by
  closeness to a measured value.
- **Where the tables are built** (operator, 2026-11-11). The rest point is
  read at a singularity, so each singularity builds a table:
  - **The universal table:** every genesis builds the universe's table at
    the universal core's bounce. Each kind's `3Q / 3Z = D(-p*)` is read
    against the universal singularity's medium (#195's resistance, at the
    throat), so it is that universe's, by its seed and its laws.
  - **The galactic tables:** each galactic core is a black hole and white
    hole pair with a bounce (Cores are paired throats). Its singularity
    re-reads the conversion against the core's medium, so its galaxy's
    elements form there, as an event at the core's bounce.
  - **Locality:** traversing outward from the universal singularity, the
    elemental distribution is local by core. A reader of a kind's nucleons
    names the singularity whose table it reads; no galaxy reads another's,
    and no homologue reads the home galaxy's (S010).
- **Open, in order.**
  1. The rest point at a singularity: the throat's medium as the
     resistance, read per dipole. At a throat the medium's share is 1/2, a
     push of 0, so the rest point is the kind's own axis split: its states
     on the axis plane by side (`atoms::Band::plane_levels`), giving
     `t = 3Q / 3Z`. Benched by `observe split` over Z 1 to 1700, it reads:
     - the seals exactly on the equator, `t = 1`, as The dipole states;
     - by count (the neutral level halved at the equator), `t <= 1` for
       every kind, with p50 0.92;
     - by weight (E005), no Q-side weight up to Z 15.

     Ruled 2026-11-11: both are kept as dipoles, the count dipole and the
     weight dipole, never collapsed. The fill is bidirectional (S136
     remediated). A band's fill enters from the Q side toward Z by the face
     of the electron phase's channel against the held count,
     `f_Q = abs(R) u / (W + abs(R) u)` with the aperture `u`, when `R > 0`, and
     from the Z side alone when `R <= 0`. It is read at the kind's capture
     phase `(s Z) mod 1`, the crossing phase the founding carries, and held
     as exact shares into the axis polarity `A` that the universal core's
     cohorts read (`Substrate::spin_orbit`).

     Ruled 2026-11-11: the split, the fill and `R` are read per Hopf plane,
     not on the axis plane alone (S137 remediated, E016). The six planes
     are the carrier's six dipoles, twelve faces on `S^11`, Faces of the
     dipole on S^n. On plane `p` the levels are the plane's charge `m_p`,
     `C(l - |m_p| + 9, 9)` orientations each: a harmonic's multiplicity
     reads only `sum |m_p|`, so every plane counts alike
     (`atoms::tests::every_plane_counts_its_levels_alike`). Each plane reads
     the capture phase `psi_p = (w_p s Z) mod 1` at the winding `w`,
     its phase `R_p = kappa (1 - 2 frac(psi_p))`, the fibre's coherence
     carried whole on every plane, and its fill `f_p`
     (`circuit::fill_shares`), so its polarity `A_p`
     (`atoms::plane_polarities`). At the bounce the winding is the common
     phase (Hopf carrier), so the six planes read alike and the scalar
     reading lost nothing there: the planes part only where a winding is
     not common (the Seifert twist, #128) or a tilt turns the pole axis.
     The boundaries per plane are two (E017): the balance `t_p = 1`, which
     is `p_net,p = 0` and `D(p_net,p) = 1` at once, and the fill's reversal
     `R_p = 0`. `observe split LO HI W` reads the six planes at any
     winding.
  2. The flow step `s` of one conversion, and its carry on `R`.
  3. Each galactic core's table at its bounce, from its medium, as an event
     in the galaxy's history. `circuit::mass_number(z)` becomes a reading of
     a named singularity's table.
  4. Fusion's products read from the table of the core where they form.

### Gravity: the deformation about a mass invariant

Ruled 2026-11-10 ([#155](https://github.com/SAIOS-SAMN/xylarium/issues/155)): gravity is the topological deformation of the
manifold itself around a mass invariant, and it produces convergence and
divergence as antipoles. The antipole is not an opposing force, a reversed
vector or a negative mass: it is the dual deformation about the same
invariant, with the holonomy around it conjugate.

- **The invariant.** The mass invariant `M` is what the deformation keeps:
  a count of whole nuclei and their states (axiom 4), exact and calibre-free.
  Which nucleon `Q` enters it is [#151](https://github.com/SAIOS-SAMN/xylarium/issues/151).
- **The pole coordinate.** At reach `r` the deformation is read by
  `t = r / M`, reach over hold (open over held, as the pole point is). It
  is a ratio, so every reading below is calibre-free.
- **The two faces** (`xyl_core::shares::faces`):

  ```text
  convergent face   c = M / (r + M) = 1 / (1 + t)     the deficit (axiom 2)
  divergent face    d = r / (r + M) = t / (1 + t)     c + d = 1,  c / d = 1 / t
  ```

  The convergent face is held (the Z side); the divergent face is passed
  (the Q side). The turning law reads the convergent face; the well's
  clocks run at the divergent face (chronometry).
- **The antipole.** The inversion (axiom 5) `t -> 1 / t`, that is
  `r -> M (M / r)`, exchanges the faces: `c(1 / t) = d(t)`.
- **The imbalance as a push.** `p = c - d = (1 - t) / (1 + t)` is a push
  with `D(p) = c / d = M / r`; the antipole reverses it, `p -> -p`, which is
  the flow law's `D(-p) = 1 / D(p)` (axiom 3). It is positive within
  `r = M`, zero on it, negative beyond. Exactly, at every reach,
  `p = 2c - 1 = 1 - 2d`: every reading of the imbalance is one face read
  linearly.
- **The equator.** `t = 1`, `r = M`: the antipole's fixed circle, where
  `c = d = 1/2` and `p = 0`. It is where the invariant is read as a reach:
  a change of calibre moves the circle with `r` and `M` together and keeps
  both faces.
- **In angles.** The convergent face's turn per traversal is `+360 c`
  degrees (a deficit angle, inward, elliptic); the divergent face's is
  `-360 d` degrees (an excess angle, outward, hyperbolic). The antipole
  makes them conjugate: the divergent turn at `t` is minus the convergent
  turn at `1 / t`.
- **One class, two representatives.** Since `-360 d = -360 + 360 c`, the
  two turns are the same element of QQ/ZZ (Winding and phase): they differ
  by `360 (c + d) = 360`, one whole turn. A body reads that one class, not
  the sum of the two: its winding is `-1` (the divergent representative's
  whole turn, the orbit's own revolution) and its phase is `360 c`, the
  shear T3 reads (`360 delta`). Adding the faces would count the one
  deficit twice; no reading changes.
- **The torsion share is the divergent face.** Axiom 7,
  `s = (H + J) / (2H + J)`, is `d` with the horizon's hold `H` at reach
  `H + J`, its pole `t = 1 + J / H = 1 + a / 2 = s_Q / s_Z`
  (`turning::tests`). The spin needs no second model: it is gravity's open
  face at the horizon.
- **Holonomy** ([#150](https://github.com/SAIOS-SAMN/xylarium/issues/150)). `c` changes with `r`, so it is the
  deformation's reading at a loop, not the loop-independent class: the
  class is `M`, read where the faces balance. A loop's holonomy is inward
  on the convergent face and outward on the divergent face.
- **No repulsion.** A body on the divergent face is not pushed: it follows
  the manifold's flare. The focusing and defocusing of geodesics name the
  two faces; their squared form is not a law here.
- **Open, in order.**
  1. The invariant's count ([#151](https://github.com/SAIOS-SAMN/xylarium/issues/151)) and its reading as enclosed
     curvature (Gauss-Bonnet) on the containment manifolds.
  2. How each manifold of the chain deforms about its invariant, as the
     vortex's core and swirl ([#121](https://github.com/SAIOS-SAMN/xylarium/issues/121)), never placed.
  3. The turning law, the wells and the calibre ([#123](https://github.com/SAIOS-SAMN/xylarium/issues/123)) read on the
     deformed manifold; draft #152 rebuilt on it.

### The six planes: a face per plane

Derived from built laws ([#128](https://github.com/SAIOS-SAMN/xylarium/issues/128), ruled 2026-11-06: separation, never an
equal split; [#151](https://github.com/SAIOS-SAMN/xylarium/issues/151), ruled 2026-11-10: two counts named apart), after
Gravity ([#155](https://github.com/SAIOS-SAMN/xylarium/issues/155)). The identities are tested
(`shares::tests::a_faces_dilation_is_linear_in_its_pole`,
`turning::tests::t2_is_the_face_law_at_the_horizon`,
`turning::tests::t14s_plane_push_is_a_divergent_face`); the per-plane
split is not yet built.

- **Two counts.** The mass weight `m_Z = Z + Q` (`formation::mass_weight`,
  `Q` the neutron count) is the hold: inertia, gravity's invariant and the
  founding's reach `R_Z = (K + s) / (m_Z H)` read it. The capacity `cap`
  (`atoms::capacity`) is the room: the dipole's Q side and the seals'
  equator. Counting empty room as hold would put the outermost turning's
  envelope in the mass (from Z 28 to Z 33 the capacity runs 28, 1092,
  4550). The axis plane reads the hold; the five isoclinic planes bound
  the room.
- **A face per plane.** Each of the six planes reads gravity's face law
  with a pole of that plane, reach over hold on it:

  ```text
  a kind's states     t   = U / 2P            the pole point (axiom 9; built U / W)
  axis plane          t_0 = r / M             gravity's faces (axiom 2)
  isoclinic plane j   t_j = J_j / (6 H_j)     T14: push J_j / (6 H_j + J_j) = d(t_j)
  horizon             t_s = 1 + J / H         the torsion share s = d(t_s) (axiom 7)
  ```

- **Dilation is linear in the pole.** For every face,

  ```text
  D(d(t)) = 1 + 2t,   D(c(t)) = 1 + 2 / t,   D(d - c) = t
  ```

  with no square and no root. So T2 is the face law at the horizon
  (`M = 1`, `t_s = 1 + a / 2`): `D(s) = 3 + a`, the twist
  `n = d - c = a / (4 + a)`, `D(n) = t_s = s_Q / s_Z`. And T14's plane
  push dilates by `1 + J_j / (3 H_j)`.
- **Separation.** No plane's share is set by an equal split. Plane `j`
  is read by `t_j` from the plane's hold `H_j = sum_k M_k w_kj` (each
  kind's hold weighted by its share of turning on that plane; Holding in a
  pinched throat, EQUATIONS.md) and the plane's twist `J_j`. What moves
  between planes is twist and hold, and their totals are kept:
  `sum_j J_j = J`, `sum_j H_j = H_iso` (axioms 4 and 11). Without spin
  (`J = 0`) every `t_j` is zero and nothing turns; a spinless hole does not
  bounce (axiom 7), so every founded universe separates its planes.
- **The capacity face.** Axiom 9's capacity reads the same law at one
  kind, between its held and open states: convergent `2P / Z`, divergent
  `U / Z`, pole `t = U / 2P`, `D(U / Z) = 1 + U / P`; the antipole
  `t -> 1 / t` exchanges held and open (axiom 5), and the equator
  `U = 2P` is the pole point's throat. The room `E = cap - Z` stands beside
  them, and the occupancy `Z / cap` is the dipole's Z face. Built, the code
  reads the held count `W = 3Z - U` instead: `W / 3Z` is the closed share
  (`elements::closed_share`) and `t = U / W` the pole point (#158).
- **The transfer.** A state of weight `m` in Z^6, with turning shares
  `w_k = |m_k| / sum_i |m_i|`, feels its planes composed by the flow law
  (T15): its hyperbolic angle is the sum over planes of `w_k ln D_k`, with
  `D_0` the axis plane's and `D_j = 1 + 2 t_j` on the isoclinic planes,
  and `D(n)^A` for its axis polarity. The angles add because the planes
  are orthogonal and commute; each `ln D_k` is a certified enclosure,
  never stored.
- **Deformation** ([#121](https://github.com/SAIOS-SAMN/xylarium/issues/121)). A manifold of the chain deforms about its
  invariant as its poles move: on the axis plane by its reach about its
  hold, on each isoclinic plane by the twist and hold moved into it. A
  change of calibre dilates reach and hold together on every plane and
  keeps every `t_k`, so the deformation reads the same in every tier.
- **Open, in order.**
  1. The per-plane split of `J` and `H` from the spin: `bounce::spin_bivector`
     gives every blade the full spin ([#128](https://github.com/SAIOS-SAMN/xylarium/issues/128)).
  2. The axis plane's own count: a band's harmonics carry an axis weight
     `m0` too, so how the room divides between the axis and the isoclinic
     planes is to be read from the weights (Weights on the six planes).
  3. The poles as the vortex's windings of its fibres about the core
     ([#121](https://github.com/SAIOS-SAMN/xylarium/issues/121)).
  4. The turning law and the calibre ([#123](https://github.com/SAIOS-SAMN/xylarium/issues/123)) on per-plane poles; #152
     rebuilt on them.

## 3. The bounce and the founding

### The bounce (`bounce.rs`)

A universe begins as the white-hole face of a parent's black hole, computed
exactly in the conformal algebra `Cl(13, 1)` over `Q` (twelve base axes: the
axis plane, its first axis the pole axis, and the five isoclinic planes).

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
- The spin is the bivector `K = a sum_k B_k` over the blades of the six
  planes, the axis plane in the common phase (`spin_bivector`, #183; at a
  crossing off the axis plane its axis part adds nothing to the source). The white hole is the null generator
  `N = P ^ (P . K)` at the crossing point (`xyl-core cga::Source`); it is
  verified to be null, fixes `P` and preserves the horizon. The throat's far
  face is the crossing point's antipode under 180 degrees on each of the five isoclinic planes
  (`circuit::plane_antipole`), a degree of structure; its return is the
  next bounce's, read at the horizon's deficit, one third (axiom 6).
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
- The founding is made once at the drawn spin (`cores::substrate_for`);
  whether the universal core's paired throat closes is read on its spin
  orbit (`circuit::Substrate::spin_orbit`, T17; The core as a paired
  throat), and the anchor holds the core's spin after it
  (`circuit::Anchor::core_spin`).
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
and clusters, `compound`), each kind's pole state (held count `w`, open
states `u` and the electron phase `R`), its heat (the held share of the
irradiation it has received), its hold `m` (a reading of its counts on the
founding, in core masses), its reach `r` from its throat (in `r_g`) and its
angular state (its phase and plane in degrees on the S^360). Counts, occupancy
and capacity are conserved through every change; mass is read from the counts,
never carried.

**Release.** Kind `k` leaves the throat at its onset, the echo at which its
pole state crosses the throat under the jet map (`circuit::onset`,
`circuit::throat_crossing`), at the throat's outflow rate
`m0 (1 - exp(-rate (t - onset)))`, from the throat's horizon reach
`R_h(H_0) = 2 H_0`. What has left is no longer held:
`H_0(t) = 1 - sum_k outflow_k(t)`. A kind released over an interval is a
filament: the locus of the parcels released along it.

**What the throat holds.** A kind with no open states has no pole state,
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
vessel's band (built: `circuit::Substrate::found` holds the closed kinds
as the firmament, T16). Heavier closed kinds exist
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
torsion and twist, a degree of perturbation: it turns the parcel's phase on
the isoclinic planes, and its plane toward the axis plane, by the torsion
share of the throat's spin, and never moves its reach; the return through
the vacuum is read at the parcel's own deficit at its reach. Parcels interact by
the degrees between them, the coherence `c = 1 - 2d` (`circuit::coherence`,
`d` the shorter way round on the plane the two phases share, read as
`c = 1 - d / 180` with `d` in degrees, so `c` is in `[0, 1]`): 1 in
phase, 0 opposed. Opposed parcels do not exchange.

**Perturbation.** The part of the echo a parcel's elements cannot pass, its
held share `m / (r + m)`, reverberates in its occupied states: the vacuum channel of
the parcel's loop (`circuit::Substrate::loop_flux`, swung by the inbound
echo) moves states between held and open, and so sets its heat. The same
net push `p_net` dilates the parcel's pole state per echo. Chemistry is what
a parcel's kinds close into at the parcel's heat (`compound::closing`,
`compound::held`), and a parcel whose pole state reaches the throat
(`t = 1`) begins to transduce (`stars::Sequence`).

**Meetings.** Where two parcels reach the same `r`, they exchange matter by
the contrast of their deficits `T` times their coherence, with `sum m l`
conserved (the filter of `formation::founding_shells`). Meetings and onsets are the
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
  the closed kinds, the paired throat's shared hold (T16), each at its
  reach (its spin about the throat) and its capture phase. Their amounts are scaled, preserving
  their founded proportions, so that their total mass is the universal
  state, 1, which is the core's mass. The held share is then
  `1 / (1 + unit)`, where `unit` is the sum of the beads' states
  (`held_share`).
- The deficit law gives the share that a mass `M` holds at reach `R`, and
  every hold in the model takes its form: `delta(R, M) = M / (R + M)`, with
  received share `1 - delta`. The vacuum's share at reach `R` about a hold
  `H` is that deficit, the share the pulse carries there
  (`xyl_core::shares::deficit_dy`); read at the throat, it is the throat's push,
  `p = delta(H, H) = 1/2` (`circuit::throat_push`). The jet carries the
  throat's torsion share `s = (H + J) / (2H + J)` at each crossing
  (`bounce::torsion_share`), so the continuum dilates the pole coordinate
  by `rho = D(s) = (1 + s) / (1 - s)` per echo (`shares::dilation_dy`):
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
  the hold, and the electrons' phase state `R` on the open states
  (`circuit::electron_phase`, `R = kappa (1 - 2 frac(p))`). The residual is
  the observed dilation over the product of the channels, which is the
  torsion that the echo leaves on the kind. The loop closes, conserving the
  kind's 3Z hold, where the residual is 1. There is one jetstream per held
  kind. Status: partial (read by the observer).
- The ocean is described by the waveform at reach `r`,
  `A(r, t) = [cos(2 pi (t - tau) / echo) / echo + q(t - tau)] H / (r + H)`,
  with `tau = (r / rim) bell / 2` and `H = H(r, t)` the held medium inside
  the reach (`Substrate::amplitude`; see
  [The held medium: the ocean as a carry](#the-held-medium-the-ocean-as-a-carry)). A galaxy's share
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
  the throat at `1440 delta` degrees per echo on its orbital plane, with
  `delta = H / (reach + H)`, `H` the held medium inside its reach
  (`Substrate::held_within`). Its plane is set by the repulsion
  pivots one tier up (the node by the golden angle about the throat's axis,
  the tilt toward it by the plastic angle's share of the S^360 measure; a
  placement by index, to be replaced, roadmap item 40),
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
  `(r / rim) bell / 2` after the bounce, and the share of it carried there
  is the held medium's deficit, `H(r, t) / (r + H(r, t))`
  (`Substrate::amplitude`). In the galaxy tier the front is carried
  between holes by the relay network (`pulse::relays`): every galactic core
  and stellar remnant passes it on continuous in phase. Next: the relay is
  under review against a single front echoed only at surfaces (cores,
  the firmament, the rim) with the held medium's delay `r + H(<r)`, shared
  with Netstratum (roadmap, milestone 3).

### The held medium: the ocean as a carry

Built ([#113](https://github.com/SAIOS-SAMN/xylarium/issues/113)), in the founding calibre. What reaches a place is what the
ocean's medium carries there, not a push; by axiom 2 the share at reach `r`
is the deficit about the hold within `r`.

- **The medium** (`Substrate::held_within`, `Substrate::medium`): as
  founded, the core's 1 plus the beads nearer, `H(r)`; at `t`,
  `H(r, t) = H(r) - passed(r, t) / F`, `passed` the outflow by
  `t - tau(r)` bound beyond `r`. Matter in flight or taken within `r` stays,
  so the books close.
- **The share carried** (`Substrate::carry`): `delta = H(r, t) / (r + H(r,
  t))`, read by the waveform, a galaxy's circulation and push, and a
  piece's star cut (`stars::ring_per_bell`); 1 at the throat.
- **Outward, from the echo**: the near face's open share out against the
  far face's held share back, `s (+) (-(1 - s))` per echo, the pair share's
  composition.
- **Turning, from the torsion**: `360 delta` degrees a traversal on the
  isoclinic planes, returned after `1 / (1 - delta)` (T4).

Open: a star's life still reads about the core's hold 1 ([#113](https://github.com/SAIOS-SAMN/xylarium/issues/113) step 1);
the echo share the ocean reads ([#131](https://github.com/SAIOS-SAMN/xylarium/issues/131)).

### The hold inside an orbit

Planned ([#116](https://github.com/SAIOS-SAMN/xylarium/issues/116); draft [#125](https://github.com/SAIOS-SAMN/xylarium/pull/125)).
A body is caught in an orbit, never a band. Body `b` (mass `m_b`, reach
`a_b`) orbits `h_b = 1 + sum(m_c, nearer)` with eccentricity
`delta_b = h_b / (a_b + h_b)`, swinging between `a_b (1 -/+ delta_b)`. Its
share within `r` is the reverberation's swing law,
`chi_b(r) = clamp((r - a_b (1 - delta_b)) / (2 a_b delta_b), 0, 1)`, and
the hold is `H(r) = 1 + sum_b m_b chi_b(r)`: one model for
`Substrate::held_within` and `galaxy::inside_mass`, a share, never a step.

### The vortex: core, swirl, twist, writhe, linking

Planned ([#121](https://github.com/SAIOS-SAMN/xylarium/issues/121)). Founded matter is a vortex in the medium, and every
body stands in its parent's. The founding already derives it and the code
reads it as two numbers at a point:

| Vortex | In the founding |
|---|---|
| Core | the filament, `X(t)` at `t = U / W` |
| Stretching | the axis plane's dilation `D(n)^t` |
| Swirl | the isoclinic action; circulation `Gamma(r) = 2 pi H r / (r + H)` about the held medium |
| Vortex lines | the Hopf fibres, `S^1 -> S^11 -> CP^5` |
| Twist | `n` per echo |
| Writhe | `+/-120 n` degrees per echo |
| Linking | `Lk = Tw + Wr`, conserved (T10, axiom 11) |
| Reconnection | `Lk` changes by one after `N* = 3 (4 + a) / (2a)` echoes |

The fibre is the circulation loop and `CP^5` is where it stands, so a body
is its core curve, circulation, twist and linking with every other vortex,
not a list of angles. Bodies nest: each core rides a streamline of its
parent's swirl, in its parent's calibre dilated conformally. The vortex
replaces `galaxy::Structure`'s bands, `galaxy::Orbit`'s circles on drawn
planes and `chain::Place`'s points (streamlines are the throat orbit's
sheared ellipses, `e = delta`, `360 delta` degrees a traversal).

Open: the core curve between tiers; helicity without its `Gamma^2` self
term; per-plane twist by separation
([#128](https://github.com/SAIOS-SAMN/xylarium/issues/128), ruled). The
fibres' carrier, their separation and their linking are ruled and built
(The Hopf carrier, below).

### Pivotry from the fibrations

Planned ([#207](https://github.com/SAIOS-SAMN/xylarium/issues/207)). A tether between two bodies is a Hopf fibre on
the twelve axes, and its behaviour is read per plane `k` from the
fibration, never from a threshold or a material table:

| Behaviour | Law per plane `k` | From |
|---|---|---|
| Pivot | the relative versor `P_b^-1 P_a`, per plane `(K_k, phi_k)` | `rotor::Rotor::conjugate`; Winding and phase |
| Twist | `Tw_k = w_i,k - w_j,k`, the turn `360 delta_k` | `hopf::turn`; [#128](https://github.com/SAIOS-SAMN/xylarium/issues/128) |
| Writhe | `Wr = Lk - Tw` | `hopf::linking`; The vortex |
| Strength, weakness | the overlap's held face `h_k` and open face `o_k = 1 - h_k` | `hopf::overlap` |
| Pull | `p_k = h_k - o_k`, offset by the medium, `p_k (+) (-r_k)` | Push and resistance |
| Break | the event at which `Lk` changes | `xyl_core::event`; The vortex (reconnection) |

- **Break is topological.** `Lk` is invariant under every continuous
  deformation, so a tether bends, stretches and twists without breaking,
  and breaks exactly where its linking changes: the vortex's reconnection.
- **Built:** the conjugation, the turn at any winding, the overlap's faces,
  the linking, the holonomy and the tilt (The Hopf carrier).
- **Not built:** the pivot per plane ([#118](https://github.com/SAIOS-SAMN/xylarium/issues/118)); a non-common
  winding from the founding ([#128](https://github.com/SAIOS-SAMN/xylarium/issues/128)); the twist per plane
  (`turning::twist_per_turn` reads one plane, S130); writhe, pull and
  break as readings.
- **Consumers:** the anchor chain's stretch, torque and twist; the bond
  (`atoms::Bond`); material strength in place of the calibrated tables.

### The Hopf carrier: the six planes in one phase

Ruled 2026-11-11 ([#183](https://github.com/SAIOS-SAMN/xylarium/issues/183)); built in `xyl_core::hopf`.

- **The carrier.** The band counts (1, 12, 77, 352 orientations) are the
  harmonics on `S^11` in `C^6`, each orientation a fibre of
  `S^1 -> S^11 -> CP^5`. That fibration needs a free circle action, so all
  six planes turn in the common phase, the axis plane included:
  `z_p -> e^{i theta} z_p` (`hopf::turn` with `hopf::COMMON`). Turning only
  the five isoclinic planes would fix the circle `z_iso = 0` and leave a
  cone over `CP^4`, not `CP^5`.
- **The stretch.** The Z-Q stretch is the axis plane's share split, not
  its phase: `S^11 = S^1_axis * S^9_iso`, a join, each point a split
  `c0 = 1 / (1 + t0)`, `d0 = 1 - c0` between the axis circle and the
  isoclinic `S^9` (which fibres over `CP^4`). The axis plane's phase joins
  the common turn, so a pure axis state still turns.
- **The twist.** The per-plane twist (#128) is the weighted Seifert action
  `z_p -> e^{i w_p theta} z_p`, `w` in `Z^6` (`hopf::turn` with any
  winding).
- **Separation.** Two fibres hold `c = sum_p min(s_p(i), s_p(j)) (1 -
  arc_p / 180)` and pass `d = 1 - c` (`hopf::overlap`), `s_p` their plane
  shares (`|m_p| / l` for a harmonic, `hopf::plane_shares`) and `arc_p` the
  shorter arc of their phase difference on plane `p` in degrees. It is
  exact, with no root and no arccosine; its pole `t = d / c` reads as the
  faces law's.
- **Linking.** Two circles do not link in `S^11`. The axis circle and the
  isoclinic `S^9` link once (1 + 9 + 1 = 11, `hopf::STRATA_LINKING`). On
  the 3-sphere `S^3_{p,q}` that two planes cut from `S^11` fibres link by
  their winding products (`hopf::pair_linking`), and the axis-isoclinic
  linking of two fibres is
  `Lk = sum_{q=1..5} (w_i0 w_jq + w_j0 w_iq)` (`hopf::linking`).
- **S^360.** It is even (`chi = 2`), so it has no circle fibration. It
  stays the universe's measure space, and each body carries an
  `S^11` bundle of the body over its site (#162).
- **Layout.** The founding's points have the twelve base axes: the axis
  plane on `cga` axes 1-2, its first the pole axis, and the isoclinic
  planes on 3-4 to 11-12 (`circuit::plane_blade`, `bounce::AXES`).
- **The charged spin.** Ruled 2026-11-11. The spin is charged, the
  manifold's deformation itself, and it rests on its axis tilt through the
  charge:
  - Its magnitude is the holonomy at the body's reach,
    `theta = 360 delta` degrees per traversal (`hopf::holonomy`), returned
    after `1 / (1 - delta)` traversals (T4), on the six planes in the
    common phase.
  - The body's open charge is `pi = (p / w) (1 - delta)` (`hopf::open_charge`,
    LAW-COUPLING v2's push on xylarium's orientation, the bound share
    `g = delta`).
  - The tilt turns the pole axis toward the open charge's isoclinic plane
    (`hopf::tilt`), one generator, since a complex-linear mixing of planes
    leaves the common phase unchanged (`T K T~ = K`).
  - The tilt's faces are read off the motion, `c = 1 - eps / 90` and
    `d = eps / 90` (`hopf::tilt_faces_l1`: the overlap law on the share circle,
    `psi = 2 eps`).
  - Its natural balance is where those faces equal the charge's,
    `eps* = 90 pi` degrees (`hopf::tilt_equilibrium`), restoring on both
    sides. The writhe carries the tilt toward it, `Lk = Tw + Wr`.
  - The charged spin is `K = delta (T B_0 T~ + sum_{q=1..5} B_q)`
    (`hopf::charged_spin`).
- **Where it stands.** The pieces are built in `xyl_core::hopf`. The
  bounce's generator (`bounce::spin_bivector`) turns the six planes in the
  common phase but is still untilted, with the drawn Kerr parameter as its
  magnitude. Untilted, its axis part adds nothing at the crossing
  (`P . e1 e2 = 0`), so the founding reads unchanged; that is the absence
  of the tilt, not a law. The tilt enters once the parent hole's charge
  (`p / w`) is read, and then the source carries `sin eps (P . e3 e2)`.
  The bells and the worlds read it (ruled 2026-11-11, the bounce held
  until the parent's charge is derived, and `bounce::torsion_share` on the
  drawn spin until then):
  - A stellar hole's spin axis stands `eps*` from its orbit normal
    (`field::stellar_tilt`, `field::stellar_axis`). The charge is its
    progenitor's held mixture's proton share (`circuit::mix_weights`), and
    `delta` is the progenitor's compactness, the bound share at a star. The
    tilt turns the pole toward the line of nodes.
  - A world's tilt is `eps*` of its matter's proton share
    (`sum m_i Z_i / A_i` over `sum m_i`) and its held face at its rim,
    `m / (R + m)` in `r_g` (`bodies::world_tilt`). It replaces the drawn
    obliquity.
  - A world is seasonal where the tilt's passed face exceeds its kept one,
    past the faces' equator at 45 degrees (`life::seasonal`). It replaces
    `life::body`'s `tilt > 23`, Earth's number, in the r-strategy.
- **Open.** For the plane-pair products, which torus is
  inside is set by the share split; a fibre on several plane pairs needs
  its winding on each.

### The ring: the throat orbit and the echo's torsion

Planned ([#60](https://github.com/SAIOS-SAMN/xylarium/issues/60)); it replaces the bell as light's round trip to the
rim. Units: `G = c = M = 1`, reaches in `r_g`; counts are in Z and angles in
degrees, exact rationals modulo 360 on a named plane (never decimals); an
irrational value (pi, a cosine at a general angle) is read only as a certified enclosure for
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
  periapsis by `360 delta` degrees in its orbital plane
  (`turning::twist_per_turn`), a degree of structure returned after
  `1 / (1 - delta)` traversals (T4), and the spin moves the orbit across the six planes
  (`bounce::spin_bivector`): every direction turns on a great circle of
  S^11 in the twelve dimensions the planes span, by the same degrees on
  each plane (T18, the axis plane in the common phase by #183), and the shear carries the
  orbit from one circle to the next.
- **Resonance.** An orbit returns exactly after `q` traversals, `q` the
  denominator of its advance in lowest terms. The reaches `r_k = (k - 1) M`
  advance `360 / k` degrees in their orbital plane and return after `k`: the throat (`k = 2`,
  apsis ratio 3, the throat push's dilation), the horizon (`k = 3`, apsis
  ratio 2), and so on; the inversion `r -> M^2 / r` maps each to
  `M / (k - 1)` on the far face, the throat to itself.
- **The ring.** A ring is the reading of the core's orbit at the throat
  (`r = M`, `delta = 1/2`), never one scalar. At a count `t` (exact in Q) it
  is

  ```text
  R(t) = ( the axis:    D(n)^t, held by its exponent t
           the planes:  phi_i(t) = t theta_i mod 1, i = 1..5, each in QQ/ZZ
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
  the network's height per segment, never fixed. With no spin each isoclinic
  plane advances 180 degrees per traversal and returns after two, the two
  faces of the attractor pair then reading the same density (`s = 1 - s`): the echo is the
  throat orbit's return.
- **The echo at any spin** (built: `circuit::echo_at`). The near face
  advances by the torsion share `s` per traversal (axiom 7), so by T4 it
  returns after `bell / (1 - s)`, and the far face after `bell / s`; the two
  are equal only at zero spin. Each throat's echo is read at that throat's
  torsion share: the universal core's founding at the founded share
  (`circuit::Substrate::echo`), every tier after the core's bounce at the
  share of its spin then (`circuit::echo`, `Anchor::core_spin`, T17).
  `xylarium observe universal-core` prints both faces' echoes at each.
- **The local ring.** At reach `r` about a hold `M`, the local ring is one
  traversal of the orbit at `r`, and every law step at that reach (a turning
  on the orbit's plane, the vacuum's halving of an agitation, a cooling) counts local
  traversals. Two orbits' traversals compare exactly, since pi cancels in
  the ratio of their angular rates: one local traversal at `r` lasts

  ```text
  (r / delta(r)) (s / M) = s r (r + M) / M^2   core traversals
  ```

  The engine counts core traversals; each tier reads the orbits at the reaches
  within it and converts by this ratio. The turning law is then 360 degrees on the
  orbit's plane per local traversal, `360 (delta(r) / r) (M / s)` degrees
  per core traversal; no
  light-time and no SI enters a law.
- **The twist and the dipole.** With spin `a`, the throat carries the
  torsion share `s = (2 + a) / (4 + a)` on the open side and holds `1 - s`
  on the held side. Each echo twists the open pole's share `s_Q` up and the
  held pole's share `s_Z` down about the equator `1/2` by half the twist
  `n`:

  ```text
  n = 2s - 1 = a / (4 + a)        the twist per echo, in QQ/ZZ
  s_Q = (1 + n) / 2,  s_Z = (1 - n) / 2
  s_Q / s_Z = (1 + n) / (1 - n) = D(n) = 1 + a / 2
  ```

  `s_Q` moves as the antipole of `s_Z` (the inversion `t -> 1 / t`
  exchanges them), and the dipole keeps the twist as torsion: `s_Q / s_Z`
  depends on the
  spin alone and does not drift into a phase. The twist is the echo's
  residual, held per echo, never summed into the count.
- **Influence.** The derivatives of the readings in the spin and in the
  reach are exact:

  ```text
  dn/da = 4 / (4 + a)^2,   ds_Q/da = 2 / (4 + a)^2 = -ds_Z/da,   d(s_Q/s_Z)/da = 1/2
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
- **Viscosity.** A medium is viscous where its states grip their
  neighbours and fluid where they do not: closed kinds and bonds heat has
  broken slide. In the pole coordinate `t = U / W` the viscosity is the
  open share, `nu(t) = t / (1 + t)`, the deficit form `delta(t, 1)`; heat
  enters through `compound::open_hot`. The inversion reverts its polarity:
  `nu(1 / t) = 1 / (1 + t) = 1 - nu(t)`, so what grips on the fusion side
  (`t < 1`) slides on the fission side (`t > 1`), and at the throat
  (`t = 1`) both faces read one half.
- **Positrons.** The electrons' phase state is `R = kappa (1 - 2x)`,
  `x = frac(p)` in QQ/ZZ (`circuit::weights::electron_phase`). The reversion
  runs the echo backwards, `x -> -x`, and `R(-x) = -R(x)`: the positron is
  the electron's phase read through the reversion, with the same
  coherence and the opposite sign. A pair arises at the echo's reversal,
  180 degrees on the isoclinic planes (`x = 1/2`, `R = 0`), and separates, the electron toward the held face
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
  addition of its hyperbolic angle) and the spin's turning, `360 s` degrees on
  each of the five isoclinic planes. The parts lie in orthogonal planes and commute, so a state's
  reading at a count `t` is a closed form: on the axis the dilation
  `D(n)^(m0 t)`, exact in Q at whole `t`; on the planes the phase
  `t s (m1 + ... + m5) mod 1`, exact in QQ/ZZ. The state moves on a flat line
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
  `t = U / W = 1/2`, while its only state has weight zero: the built pole
  coordinate reads the held count (`3Z` per kind, a delimiter) while the
  weights count electron states (`Z`). Axiom 9's capacity account makes them
  one, `t = U / 2P` over the states (#158). The planes share one
  rate while the spin is isoclinic.

### The capacity's occupancy: the vents to open

Planned ([#158](https://github.com/SAIOS-SAMN/xylarium/issues/158)), after Weights on the six planes and The six
planes, under axiom 9.

Every law that reads the atom reads two integers per kind: the held count
`W = 3Z - U` (`compound::electron_hold`), three per proton, a delimiter, and the open
states `U` (`compound::open`, `atoms::unpaired`). The atom carries its
capacity (`atoms::Atom`, `atoms::capacity`): bands `(n, l)`, each
orientation empty, single or paired; each state's weight `m` in Z^6 over
the six planes; each orientation a Hopf fibre over `CP^5`. Held and open
keep only whether a fibre's turnings cancel, a counter-turning pair, or
not, a single electron: on the isoclinic planes, the winding in ZZ read
mod 2 (a pair is a state and its plane antipode, twist charges `c` and
`-c`); on the axis plane a pair does not cancel, its two electrons
sharing the level `m0`. The bands, the
planes, the signed windings and the room are gone before any law reads
them, so the manifold's topology and its cohomology are read as two
dimensions.

- **The vents to open.** Each replaces the two counts with the capacity's
  structure:
  1. **States per band.** Each band holds its empty slots, single
     orientations and paired orientations apart: its room, its open states
     and its held states. They sum to the room `cap - Z`, to `U` and to
     `2P = Z - U`: `cap = E + U + 2P` per kind (axiom 9), readings only (the
     hold and the room named apart, [#151](https://github.com/SAIOS-SAMN/xylarium/issues/151)). Built: `atoms::band_occupancy`
     over every band reached, empty bands included, and `atoms::occupancy`,
     their sums; the atom's unpaired count reads it (`atoms::Band::unpaired`),
     and the partition holds over Z 1 to 1700
     (`atoms::tests::the_occupancy_partitions_the_capacity`).
  2. **States per plane.** Each state's weight `m` places it on the six
     planes: `2P_k`, `U_k` and the pole `t_k = U_k / 2P_k` per plane (The
     six planes). Built on every plane (#201, E016): each band's occupancy
     by the plane's charge `m_p` (`atoms::Band::plane_levels`; the level
     holds `C(l - |m_p| + 9, 9)` orientations on every plane, filled from
     both sides by the plane's share `f_p`), and each kind's Z side,
     neutral level and Q side per plane (`atoms::plane_sides`); the plane
     polarities `A_p` read it. At rest the planes part only by their fill
     shares, which are equal in the common phase; their twist signs wait on
     the spin's per-plane turning
     ([#128](https://github.com/SAIOS-SAMN/xylarium/issues/128)).
  3. **Windings, not parity.** Each orientation's winding is kept in ZZ.
     On every plane the weights are built (`atoms::plane_weights`): the
     open states weigh `sum m U_m`, the plane polarity `A_p`, and the held
     states `sum 2 m P_m`, a pair's two electrons sharing the level `m`;
     a seal is balanced, both zero. A band at least half full has every
     orientation occupied once and its levels symmetric, so its held weight
     is minus twice its open weight; a band below half has none
     (`atoms::tests`). On the isoclinic planes
     a pair's twist charges cancel (`c`, `-c`), and a single's sign is
     symmetric at rest, so it waits on the spin
     ([#128](https://github.com/SAIOS-SAMN/xylarium/issues/128)).
  4. **Classes, not counts** ([#150](https://github.com/SAIOS-SAMN/xylarium/issues/150)). The occupancy is a cochain
     over the kind's fibres. Pairing and stripping move states between the
     poles with the capacity fixed: they are coboundaries. What a kind
     keeps through every pairing and stripping, its occupancy `Z`, its
     capacity and its windings' total, is its class, and the books close on
     it.
  5. **The delimiter retired.** The three per proton is a delimiter, not a
     capacity. Every law that reads the held count `W` is re-read on the
     capacity: the closed share, the grain's whole nuclei, the mass of an
     amount, the jet map's pole state, a star's pole and transduction, and
     the emission share. Values move here; each is witnessed before and
     after.
  6. **Seams per band and plane.** A seam, `u(z - 1) + 1 - u(z) < 0`
     (`atoms::vessel`), is read on the scalar `u`. Read per band and plane,
     a single-nucleon peel releases where a component does, and the
     vessel's lower edge follows from those components.
  7. **Received per plane and band.** Stripping (`atoms::unpaired_hot`,
     `compound::open_hot`) reads one isoclinic turning
     (`atoms::Received`) against each band's tightness; it is to be read
     per plane, with the return (Chemistry from the elements, vent 1;
     [#128](https://github.com/SAIOS-SAMN/xylarium/issues/128)).
- **Sites.** `compound::electron_hold`, `compound::open`, `compound::open_hot`,
  `atoms::unpaired`, `atoms::unpaired_hot`, `atoms::vessel`,
  `elements::closed_share`, `circuit::grain` (whole nuclei and the mass of
  an amount), the jet map's pole state (`circuit::core`), `formation`'s
  shells, hold and emission, the stars' held count and pole
  (`stars::held_count`, `stars::mixture_pole`), and `observe seals` and
  `observe founding`.
- **Order of the work.**
  1. States per band (vent 1). Done: their sums equal today's `U` and the
     room, so no reading moved.
  2. States per plane (vent 2). Done on the axis plane, no reading moved;
     the isoclinic split waits on the spin (#128). Windings (vent 3): done
     on the axis plane, no reading moved; the isoclinic signs wait on the
     spin.
  3. The delimiter retired (vent 5): the first vent that moves readings.
  4. Classes (vent 4), with the chain's cohomology ([#150](https://github.com/SAIOS-SAMN/xylarium/issues/150)).
  5. Seams (vent 6) and received turning (vent 7).

  A reading moves only where a law reads a component in place of a sum,
  and it is witnessed before and after.

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
  pairing and every founded kind keeps its open states and axis polarity.
- **The shear on the antipode.** A state and its antipode lie on opposite
  faces, which advance `s` and `1 - s` per traversal, so the pair does not
  cancel: on the isoclinic planes it drifts `360 c n` degrees per traversal,
  a degree of perturbation returned on the antipode's face at its density
  `1 - s`, and on the
  axis it dilates by `D(s)^m0 D(1 - s)^-m0 = D(p_pair)^m0`.
- **The dipole's reconciliation.** The pair's push splits exactly,
  `p_pair = n (+) (n / 3)`, `D(p_pair) = D(n) D(n / 3)`, per state
  `D(p_pair)^m0 = D(n)^m0 D(n / 3)^m0`. The dipole reconciles `n`, the
  twist it holds as torsion; the residual `n / 3 = a / (3 (4 + a))`, with
  `D(n / 3) = (6 + 2a) / (6 + a)`, is the shear it cannot hold, and it
  flips into writhe: the fold's rotation.
- **Fold forward and back.** The residual folds the near face forward,
  `120 n` degrees per echo on the isoclinic planes in the spin's direction,
  and the antipode back, `-120 n` degrees. Per echo the faces cancel, so
  `Lk = Tw + Wr` is conserved: the twist moves the two faces by their
  separations across the attractor pair (T17; never equal halves), and the
  writhe by `+120 n` and `-120 n` degrees.
- **The pinch.** The two folds meet where `N n / 3 = -N n / 3 (mod 1)`,
  first at the antipodal point, 180 degrees apart on the isoclinic
  planes: after
  `N* = 3 / (2n) = 3 (4 + a) / (2a)` echoes, where the folds' relative
  writhe first reaches 360 degrees and the linking number may change by
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
  not exceed 360 degrees per unit hold on its planes, `j <= 360 H_g` (`a_g <= 1`); a
  lighter one stays in the flow as medium, and a heavy one makes a slowly
  spinning throat. The new throat's twist and pinch follow
  from its spin, `n_g = a_g / (4 + a_g)` and `3 (4 + a_g) / (2 a_g)`
  echoes; a spinless throat makes no pinch and no tier.
- **The place.** The pinch's coordinates (The antipoles, the shear and the
  pinch): 180 degrees on the isoclinic planes, after `3 (4 + a) / (2a)`
  echoes.
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

1. **Exactness.** Counts are in Z, angles in degrees modulo 360 on a named
   plane, everything else exact in Q; an irrational is a certified enclosure, never stored; no rational is
   rounded.
2. **Deficit.** Gravity is the topological deformation of the manifold
   about a mass invariant `M`: at reach `r` its convergent face keeps
   `M / (r + M)` of what reaches it and its divergent face passes
   `r / (r + M)` (Gravity: the deformation about a mass invariant).
3. **Flow.** Pushes compose as `x (+) y = (x + y) / (1 + x y)`;
   `D(p) = (1 + p) / (1 - p)`.
4. **Conservation.** Nuclei, their occupancy and their capacity are
   conserved; mass is a reading of the counts.
5. **Inversion.** `t -> 1 / t` exchanges the Z and Q poles; `x -> -x`
   reverses a phase. On gravity's faces, `t = r / M`, it is the antipole:
   it exchanges the convergent and divergent faces and fixes the equator
   `r = M`.
6. **Horizon.** `R_h(M) = 2M`, the held share one third.
7. **Torsion.** `s = (R_h + J) / (2 R_h + J)`, `J = a M^2`.
8. **Six planes.** R^12 = C^6: the axis plane and five isoclinic planes,
   which the spin turns together; a state is a harmonic on S^11 with a
   weight `m` in Z^6.
9. **Capacity.** A kind's states stand on the directions of S^11: its
   capacity `cap` is the room of every shape of every turning its atom
   reaches (`3Q = 3 cap`, the dipole's Q side). Its `Z` electrons occupy
   it, each orientation empty, single or paired: `cap = E + U + 2P`, with
   the room `E`, the open states `U` (singles) and the held states `2P`
   (`P` pairs). Shares are read against the capacity, never against a
   delimiter: occupancy `Z / cap = 3Z / 3Q` (the dipole's Z face), held
   `2P / cap`, open `U / cap`, room `E / cap`; the pole state is
   `t = U / 2P`, open over held. Built: the code still reads the held count
   `W = 3Z - U`, three per proton, and `t = U / W`; the capacity account
   replaces it ([#158](https://github.com/SAIOS-SAMN/xylarium/issues/158)), and readings move where it does.
10. **Shear.** Every cycle is a sheared orbit read in closed form; nothing
    repeats closed and nothing is stepped.
11. **Degrees of turning.** The degrees a body turns, its hold times its
    degrees per unit hold, are conserved; the linking number's
    conservation is their topological reading.

Theorems:

| | Statement | From |
|---|---|---|
| T1 | `D(x (+) y) = D(x) D(y)`; continuity `s (+) (-s) = 0`, equatorial inflow `(2 + a) / (3 + a)` | 3, 4 |
| T2 | `D(s) = 3 + a`; twist `n = a / (4 + a)`; dipole `s_Q / s_Z = D(n) = 1 + a / 2`: the face law at the horizon's pole `t_s = 1 + a / 2` | 3, 7 |
| T3 | orbit about `M`: speed and eccentricity `delta`, apsis ratio `1 + 2M / r`; resonances `(k - 1) M` returning after `k` | 2, 6 |
| T4 | sheared return `k / (1 - theta)`; the echo the throat orbit's return | 10 |
| T5 | local ring `s r (r + M) / M^2` core traversals | 10 |
| T6 | multiplicity `C(q + 4, 4)`; axis level `C(l - |m0| + 9, 9)` | 8 |
| T7 | filling Z side first; axis polarity `A` | 8, 3 |
| T8 | held where closed, or where `D(s) D(n)^A <= 1 - 2 delta` | 3, 9 |
| T9 | pair share `p_pair = s (+) -(1 - s) = n (+) n / 3`; positron `R -> -R` | 3, 5 |
| T10 | the flip keeps every state; residual `n / 3`; `Lk` conserved; pinch after `3 (4 + a) / (2a)` echoes at 180 degrees on the isoclinic planes | 5, 8 |
| T11 | pinched throat's spin `a_g = a_p M_p / H_g` (Kerr, with the parent's degrees of turning per unit hold, `360 a_p M_p`); a cohort with `a_g > 1` does not close and stays as medium | 7, 11 |
| T12 | stretching pressure `(D(n)^m0 - 1) / (D(n)^m0 + 1)` | 3, T2 |
| T13 | a pinch encloses the whole cohort released since the last, in every direction | 8, T10 |
| T14 | enclosed matter, from the reach its layer has at the pinch (The state at the pinch); in plane `j` it is pushed out at `J_j / (6 H_j + J_j)` per echo, `H_j` the cohort's hold in the plane and `J_j` the plane's twist, `sum_j J_j = J` (none on the axis plane) | 2, 3, 7 |
| T15 | a whole nucleus feels its planes composed, `D_k = D(p_plane)^w_k D(n)^A`, `w_k` its share of turning in the isoclinic planes; held where `D_k <= 1`, else it leaves after `ln 2 / ln D_k` echoes | 3, T14 |
| T16 | a paired throat's faces dilate an open kind by `D(s) D(n)^A` and `D(s) D(n)^-A`, product `(3 + a)^2`; no open kind is held on both faces; the shared hold is the closed kinds | 3, 5, 9, T2, T8 |
| T17 | the spin is a sheared orbit over the core's bounces: a kind's separation is `rho = q ln D(s) - (1 - q) ln D(1 - s)` across the attractor pair; a cohort takes `J (m / M) (rho_cohort / rho_hold)`; a cohort less separated than its hold spins the throat up; the pair closes while `a <= 1` | 3, 7, 9, 11, T2, T10, T16 |
| T18 | the five isoclinic planes are interchangeable in the multiplicity and the filling (no state's plane is chosen); plane `j` is read by its pole `t_j = J_j / (6 H_j)`, its hold the kinds' turning on it, by separation (ruled, #128), never an equal split; the planes' phases differ by their pushes and by a link's turning between planes (The six planes) | 8, T6, T7, T14 |
| T19 | a link's remanence `w = J - (a_0 M_0) M`: per passage `w' = w (1 - m / M) + J (m / M) (1 - rho_c / rho_hold)`; conserved between passages; cleared by a passage of hold `m* = w M / (w + J (rho_c / rho_hold - 1))`; for one release, the remanent share `1 - rho_hold / rho_closed` | 7, 11, T10, T17 |

Reading (seed 7; a retired floating-point screen with exact held sets, to
be read on the binary once the pinches are built, #79): the universal core holds 0.4352 of the
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
  | Pole state | each plane's open and held count | the throat, `t = 1` | the reach on the axis plane, `ln r = (N* - n_k) ln(D(s) D(n)^A)` since the kind's onset `n_k` |
  | Heat | each plane's deficit | the balance of held and emitted | held irradiation net of emission, composed over the echoes since onset |
  | Twist | each plane's weight | the antipode on each plane | the share of antipodal pairs flipped, `1 - (1 - n / 3)^N` |
  | Place | each plane's phase and anomaly | the periapsis | phase and anomaly per plane, and returns made |
  | Pressure | the axis plane, per state | the dipole's Z and Q sides | `D(n)^(A N)` |
  | Pairs | each plane's electron phase | the echo's reversal, 180 degrees | `D(p_pair)^N` |
  | Fold | each layer's writhe | the meeting of the folds | each layer's strain `r / r_0` about the neutral surface |

  The enclosed matter starts at the reach its layer has when the fold
  closes (this replaces T14's starting reach on the throat orbit).
- **Reading** (seed 7; a retired floating-point screen, the relations
  exact; to be read on the binary once the pinches are built, #79): the first pinch encloses 605 layers, 0.5648 of the founded mass;
  the neutral surface lies at `ln r_0 = 17.966` on the axis plane, set by
  the gripping layers (their grip-weighted hold 0.0096 of the founded
  mass); 0.1783 of the founded mass lies inside it, compressed, and 0.3865
  outside, stretched; 0.4569 has no grip and flows. Per layer the reach
  runs from `ln r = 0.22` to `255`, the flipped share from 0.065 to 0.395,
  and the phase on the isoclinic planes over all 360 degrees.

### The bounce at a pinched throat

Derived ([#78](https://github.com/SAIOS-SAMN/xylarium/issues/78)); not yet built.

- **The far face.** A pinched throat bounces: its matter passes to its far
  face, which is both antipoles at once (`bounce::Bounce::of`,
  `circuit::axes`). The axis antipole, the inversion `t -> 1 / t`, carries a
  layer at pole state `t` relative to the parent to `1 / t` in the new
  throat, and turns every state's axis weight `m0 -> -m0`, so the kind's
  axis polarity becomes `-A`; the plane antipole turns its phases 180
  degrees on each of the five isoclinic planes.
- **The new throat's jet.** From there the layer climbs back to the new
  throat by that throat's own jet, `t -> a t + b` with
  `a = D(s_g) D(n_g)^(-A) / (1 - delta)` and `b = delta / (1 - delta)`,
  `delta = m / (2 H_g + m)` the kind's own deficit at the new throat's
  horizon; it is held there by pressure where `D(s_g) D(n_g)^(-A) <= 1 - 2 delta`,
  and a layer carried far out by the parent arrives deep and climbs late.
  The new throat's pinches enclose what it releases, and a pinch's cohort
  closes where `a = j' / H <= 1`.
- **Reading** (seed 7; a retired floating-point screen, to be read on the
  binary once the pinches are built, #79): eleven throats in
  a chain, the universal core alone holding more than `10^-6` of the
  founded mass. The inversion spreads the tail, layers carried far out,
  over several pinches of the new throat, but those cohorts are too light
  to close and stay as trace medium; the bulk, at mass-weighted
  `ln r = 19.8`, climbs back in about 16 echoes, within the new throat's
  first pinch interval of about 24, so each tier passes it on whole. The
  ratio of the bulk's climb to the pinch interval, about 0.67, decides it:
  below one a throat's matter goes into a single pinch, so these laws give
  one chain, not a population (#79).

### The core as a paired throat

Derived and built ([#86](https://github.com/SAIOS-SAMN/xylarium/issues/86)):
the firmament is the closed kinds (`circuit::Substrate::found`), the spin
orbit is `circuit::Substrate::spin_orbit` with each kind's axis polarity
from `atoms::plane_polarities` (T7, the axis plane's), and the anchor holds the core's spin
after it (`circuit::Anchor::core_spin`); `xylarium observe universal-core`
prints the cohorts. A pinched throat's own cohort spin waits on the
pinches being built. A core is one throat with two faces, near and far, and never an open
singularity: the far face of the universal core is the parent's core, and the
far face of a pinched throat is the core it bounces into (The bounce at a
pinched throat). Its stability is read from axioms 3, 4, 5, 7, 9 and 11
and theorems T2, T8 and T11. Every identity below is exact.

- **Both faces dilate one kind.** On the near face an open kind of axis
  polarity `A` is dilated per echo by `D(s) D(n)^A` (T8). The far face is
  the inversion, `t -> 1 / t` and `A -> -A` (axiom 5), so there the same
  kind is dilated by `D(s) D(n)^-A`. The product of the two faces is
  `D(s)^2 = (3 + a)^2 >= 9`, whatever the polarity (T2).
- **T16, the pair holds its closed kinds.** A kind is held by pressure on a
  face where its dilation is at most `1 - 2 delta <= 1`, with `delta` its
  deficit at the shared horizon, the same on both faces. Two such factors
  would multiply to at most 1, but they multiply to at least 9. So no open
  kind is held on both faces: a kind held on one face is released on the
  other. The only states every dilation `t -> a t` fixes are the two poles
  `t = 0` and `t = 1 / 0`, which the inversion exchanges. These are the
  closed kinds (`U = 0`, axiom 9). A paired throat's shared hold is its
  closed kinds, `M_c`. They never pinch, so no pinch reduces `M_c`.
- **T17, the spin is a sheared orbit, turned by the faces' separation.**
  The two faces are an attractor and its inverse at the same level: the
  near face advances `s` per traversal, the antipode `1 - s`. A kind's
  states stand on both, its open share `q = U / (U + W)` (built; `U / Z`
  on the capacity, axiom 9) on the
  near face and its held share on the antipode, so its degree of
  separation is the flow-law sum of the two advances (axiom 3), never their
  mean: the rapidity `rho = q ln D(s) - (1 - q) ln D(1 - s)`, with
  `D(s) = 3 + a` (T2) and `D(1 - s) = (6 + a) / (2 + a)`. The far face
  exchanges `q` and `1 - q`. A cohort of hold `m` takes the turning in
  proportion to its separation against the throat's present hold,
  `J' = J - J (m / M) (rho_cohort / rho_hold)`, `M' = M - m`,
  `a' = J' / M'^2` (axioms 7 and 11); releasing everything takes `J`
  whole. What the pair cannot hold, the residual `n / 3` (T10), is
  friction, not repulsion.
  - The cohorts leave at the core's bounces, which are events, not steps.
    At a core the folds' meeting (T10) is a bounce through the far face,
    the quantum torsion of the core; a pinch is a fold meeting between
    gravity wells, where a wormhole opens. Between bounces each open kind
    climbs in closed form, `ln t += N* ln(D(s) D(n)^A)`, and the bounce
    interval `N* = 3 (4 + a) / (2 a)` moves with the spin. That is the
    orbit's shear.
  - A cohort less separated than the hold it leaves takes less than its
    share of the turning and spins the throat up: phase strengthening. One
    more separated spins it down: phase weakening (#84).
  - Where a cohort's separation equals the hold's, the law reduces to
    `a' = a M / M'`, the ratio `a_0 M_0 / M_c`.
  - Once the open kinds have gone, nothing leaves: the closed kinds
    exchange twist and writhe at constant `Lk` (T10), and the spin holds at
    `a*`.
  - The pair closes on both faces while `a <= 1` along the orbit, so a
    core can open or close as its release turns its spin.
- **T11 read with the conserved turning.** The parent's degrees per unit
  hold are those of its founded hold, so a pinched cohort of hold `H_g`
  carries `J (H_g / M) (rho_g / rho_hold)` and spins at that over `H_g^2`
  (T17). The retired screen of roadmap item 32 had used
  the core's retained hold, which understated `a_g` by the factor
  `M_c / M_0`. With the conserved turning that screen still gave one chain
  of throats, with the same held shares.
- **Reading** (`xylarium observe universal-core`, seed 7):
  - `a_0 = 381123025677 / 2^40`, about 0.3466.
  - The 20 closed kinds of 1092 hold `M_c / M_0 = 0.4352`, on both faces.
  - Open kinds held by pressure on the near face only: 1.87e-18 of the
    founded hold, released by the far face; none on both faces, the faces'
    product `(3 + a)^2` being 11.2.
  - The core releases its open kinds in one cohort at its first bounce,
    after 18.81 echoes. The cohort is 0.5648 of the founded hold at
    separation `rho = -0.646`, against the hold's `-0.798` (the closed
    kinds' is `-0.995`), so it takes 0.8095 of its share of the turning.
    The spin goes from 0.3466 to `a* = 0.9933`, under the Kerr bound by
    0.7%: the universal core is a closed paired throat. In closed form,
    `a* = a_0 M_0 rho_closed / (rho_hold M_c)`.
- **What follows.**
  - **No admission.** The founding reads the orbit in place of the
    reach-based admission, which read the echo's swing (#37). Whether
    the core closes is a reading of the orbit, and the universe is founded
    either way: an unclosed core leaves every kind as medium.
  - **Pinched throats hold no closed kinds.** Their cohorts are open kinds
    only, so by T16 a pinched throat has no shared hold and holds only on
    one face, by pressure. A stable galactic core needs closed kinds, made
    in it by core fusion or brought to it (#79; Phase weakening and phase
    strengthening).
  - **What persists on the child side.** A core persists on the child side
    of a bounce only through the closed kinds it carries.

### The rotor, the place and the chain

Planned ([#90](https://github.com/SAIOS-SAMN/xylarium/issues/90), roadmap
item 40); audited 2026-11-05. This section records where the cartography
loses the model's dimensions and where the manifold chain is not composed,
and the order in which that is to be opened up.

- **The root.** A body's plane was held as a rational quaternion, the even
  part of Cl(3); it is now a rotor, an even versor of `xyl_core::cga` over
  any base axes (`xyl_core::rotor::Rotor`), each turn on a plane named by
  its generators and stated in degrees. The rotors built so far turn three
  axes only. A place is `chain::Place`, a direction in three
  coordinates with a reach, a climb and an offset. The model's states live
  on the axis plane and five isoclinic planes (axiom 8); the algebra that
  holds them is `xyl_core::cga`, and the planes' blades are
  `circuit::axes::plane_blade`.
- **Rotors built, not composed.**
  - Four sites build a rotor from a node and a tilt about the fixed axes
    `z` and `x`: `galaxy::Orbit::rotor`, `cores::Shape::place_at`, the
    founding's `dir0` in `cores`, and `rotor_of` in `manifold`.
  - A rotor is read through one image: the orbit normal as the image of
    `z` in `field::Bell::of`, and a place as the image of `(cos, sin, 0)`.
  - A planet's rotor ignores its star's rotor, and a moon's is the identity.
  - `manifold::galaxy_place` keeps a world's distance from its star and
    drops its direction (`chain::Place::standing_off`).
  - Field anchors sit on the first axis (`field::Anchor::on_orbit`).
  - `transit::separation_m` reads two worlds in different systems as their
    stars' arc.
- **Units hop through SI in the chain.** A star's reach goes through kpc
  and metres to AU; `manifold::manifolds` goes through kilograms and metres;
  moons through AU, metres and kilograms; the spheres' radii through kpc.
  Between tiers the chain's own map is `manifold::dilation`.
- **Relations not read in the lowest common tier.** `pulse::hear` weighs
  bells over a flat arc in kpc as one complex phasor on one plane;
  chronometry adds distances in kpc; galaxies carry no cross-ties.
- **Separation by squares.** `xyl_core::rotor::angle_between` (a dot product
  and a root) is the distance under every arc. Separations are sums of one
  angle and differences of radii, and the field's shear and work are sums
  of squares, against axiom 3's composition and caps in degrees.
- **Links without shear.** A star's galactic orbit and a core's circulation
  are closed circles. A moon's eccentricity never relaxes, no link carries
  a torque, and settling's kicks repeat every `q` meetings, against
  axiom 10.
- **Containment by radius.** Membership is decided by comparing radii, and
  no cap decides anything.
- **Exactness.** Directions and angles are stored as enclosure midpoints,
  and phases evolve in radians as dyadics.
- **Dimension cut down.**
  - The disc's collapse is read on a circle and the bulge's on a sphere,
    and a body's extent on a sphere.
  - The spin is one scalar on all five planes, and the horizon crossing
    turns one plane only.
  - Homologue galaxies read their stars at home values.
- **T18, founded matter turns its five planes together.**
  - The five isoclinic planes are interchangeable in every count the
    founding makes. The multiplicity `C(q + 4, 4)` (T6) counts the weights
    `(m_1, ..., m_5)` by `sum |m_j|`, and the filling reads the axis levels
    only (T7, T10). No state's plane is chosen.
  - So no count fixes a plane's share. Plane `j` holds the kinds' turning
    on it, `H_j = sum_k M_k w_kj`, and carries its part `J_j` of the spin,
    read by separation (ruled, #128), never an equal split; T14's pushes
    differ by plane as the poles `t_j` do (The six planes). The spin turns
    all five planes, each by that plane's advance per echo.
  - Distinct phases on the planes come from these pushes, plane by plane,
    and from a link that turns one plane into another: a body whose planes
    are turned relative to its parent's.
  - A body's turning is therefore three parts: the axis-plane phase; the isoclinic
    phases, one per plane, each a sheared orbit (T4); and the turning that carries the body's
    planes into its parent's, the turning between planes of the coordinate's
    holonomic lane. The present node and tilt are that turning restricted
    to one pair of planes.
  - The turning a link carries is still to be derived from the fold's
    writhe where the link forms (T10, T13). Until then the drawn node and
    tilt stand as its factors, composed along the chain.
- **Order of the work.**
  1. One rotor carrier over the six planes, composed along the chain, a
     place being its parent's rotor acting on the body's own state.
     T18 settles its shape: an axis-plane phase, the isoclinic phases (one per plane) and
     the turning that carries the body's planes into its parent's.
  2. The chain's tier maps by `manifold::dilation` only, with SI read once
     at the inlet, and relations in the lowest common tier.
  3. Separation and containment by caps in degrees.
  4. Shear, stretch, torque and twist on every link.
  5. Exact enclosures and exact degrees on named planes.
  6. The dimension of the disc, the bulge and a body.
  7. Placement from the founding's coordinate, not the sample index (#36,
     #79).

### The automorphic, synaptic chain

Planned ([#97](https://github.com/SAIOS-SAMN/xylarium/issues/97), roadmap
item 41), with The rotor, the place and the chain (#90), Phase weakening and
phase strengthening (#84) and the polity (civilizations.md, Minds in the
ancient seats, #94).

- **Automorphic: one map at every tier.**
  - The attractor pair is the same at every tier: the faces advance `s`
    and `1 - s`, a body's separation is
    `rho = q ln D(s) - (1 - q) ln D(1 - s)` (T17), and its return is read
    from the deficit `M / (r + M)` (axiom 2). This holds alike for the
    core, a star, a planet, a moon and a nucleon.
  - The map commutes with the change of tier: a link read in one tier and
    carried by the dilation `lambda` to another (`manifold::dilation`) is
    the link read there. The law holds for any `lambda > 0`; the kernel's
    tier ladder, `lambda = 60^k` (`units::Tier`), is a declared ruler and
    one instance of it, and no law names 60.
  - At the law this holds now. The deficit is of degree zero in length, so
    it and the flow law are invariant under every dilation
    (`xyl_core::shares`, the test
    `deficit_is_invariant_under_dilation`).
  - The chain fails it today. Tiers hop through SI, rotors are rebuilt per
    tier, and homologue galaxies reuse home values (The rotor, the place
    and the chain).
- **Synaptic: a link holds a weight set by its history.**
  - **The weight.** It is the link's stored turning between planes, `tau`,
    the H lane's inter-plane component (FORMS-COORDINATE v3). It connects
    one plane to another across the link.
  - **Its memory.** It is the link's remanence: the part of a drive's
    displacement the link keeps when the drive falls to its mean (Phase
    weakening and phase strengthening).
  - **Strengthening and weakening.** A passage less separated than the
    link's hold strengthens it, as release turns a throat's spin up (T17).
    A passage more separated than the hold weakens it (T19). Between
    passages the weight is conserved, never faded by disuse; clearing it
    takes a passage of the opposite sign, its coercivity.
  - **The signal.** It is the pulse, the bell and its echo, carried from
    link to link by each link's weight and delay. This replaces the sum of
    every bell as one phasor over a flat arc (`pulse::hear`).
  - **Friction.** It is what a link cannot hold, the residual `n / 3`
    turned to writhe (T10): the loss at each junction.
- **One structure for matter and minds.** A council's convergence is the
  minds' readings coming to coincide across the links between their
  houses, weighted by those links.
- **Order of the work.**
  1. One link type for the whole chain (roadmap item 40).
  2. The identity at the link: reading then dilating equals dilating then
     reading.
  3. The weight as a closed form of the passages through the link, sheared,
     never stepped.
  4. The pulse carried along the links.
  5. The council's convergence on the same links.

### Cohomology: pairs, loops and the books

Planned ([#150](https://github.com/SAIOS-SAMN/xylarium/issues/150)), after The dipole ([#148](https://github.com/SAIOS-SAMN/xylarium/issues/148)), Winding and phase
([#149](https://github.com/SAIOS-SAMN/xylarium/issues/149)) and Pairs ([#118](https://github.com/SAIOS-SAMN/xylarium/issues/118)).

- **The complex.** On the containment chain (core, star, planet, moon,
  nucleon; its links and cross-ties, The rotor, the place and the chain):
  - 0-cochains: each body's state, its dipoles per plane;
  - 1-cochains: each pair's relation through the pivot `P_b^-1 o P_a`, per
    plane, as a winding in ZZ and a phase in QQ/ZZ, on both faces.
- **Coboundary.** A pair read from two states of one origin is the
  coboundary of those states: the two ends fix its relation.
- **Cocycle.** Around a loop of the chain the relations compose to the
  identity. The books' closing at the pole (`circuit/poles.rs`: the
  composed channel versors equal the observed one projectively) is this
  condition.
- **Holonomy.** What a loop holds when it does not close is its first
  cohomology: the twist held as torsion, a link's remanence (T19), the
  echo's sheared return. Its ZZ part is the winding and its QQ/ZZ part the
  phase; `0 -> ZZ -> QQ -> QQ/ZZ -> 0` connects a phase that accumulates to
  whole turns with the winding it becomes.
- **The integer invariant.** The vortex's `Lk = Tw + Wr` (The vortex) is
  the integer a closed loop keeps.
- **Obstruction.** A loop that fails to close where no holonomy is held is
  a defect: broken is unbalanced.
- **Open.** The coefficients (dipoles under the geometric product, or
  windings and phases added); the 2-cells (the cross-ties, [#90](https://github.com/SAIOS-SAMN/xylarium/issues/90)); the
  relation to the automorphic, synaptic chain ([#97](https://github.com/SAIOS-SAMN/xylarium/issues/97)); where the
  mathematics lives (`xyl-core`, or the hypha SDK as a law shared with
  Mycelium and Netstratum).

### Phase weakening and phase strengthening

Derived for a link's turning (T19, below); the pole state's own remanence
and the cooling record are planned
([#84](https://github.com/SAIOS-SAMN/xylarium/issues/84), roadmap item 39).

- **T19, a link's remanence.** A link (a throat, or a link of the chain)
  holds turning `J` with hold `M`. Its remanence is the turning it holds
  beyond what it would hold had every passage taken the founded turning per
  unit hold: `w = J - (a_0 M_0) M`.
  - **Per passage.** A passage of hold `m` at separation `rho_c` against
    the hold's `rho_hold` takes `J (m / M) (rho_c / rho_hold)` (T17), so
    `w' = w (1 - m / M) + J (m / M) (1 - rho_c / rho_hold)`, exact in the
    holds and turnings.
  - **Strengthening and weakening.** A passage less separated than the hold
    builds remanence: phase strengthening. One more separated wears it
    down: phase weakening.
  - **Coercivity.** The hold of a passage at separation `rho_c` that clears
    `w` is `m* = w M / (w + J (rho_c / rho_hold - 1))`.
  - **Persistence.** Between passages the remanence is conserved (axiom 11),
    never faded by disuse. Friction, the residual `n / 3` (T10), moves it
    from twist into writhe at constant `Lk`: a degree of structure, not a
    loss.
  - **Closed form.** For one release down to the closed hold, the
    remanent share of the held turning is `1 - rho_hold / rho_closed`.
  - **Reading** (`xylarium observe universal-core`, seed 7): the universal
    core's remanent share is 0.1982, with `rho_hold = -0.7977` and
    `rho_closed = -0.9949`. In the core's units (the closed hold 1) the core
    turns `360 a* = 357.59` degrees per unit hold, of which 70.88 are
    remanent: the memory its founding left in it.

The rest of this section records what the pole state's own remanence is to
answer, the code it builds on and what that code lacks today.

- **Phase weakening (remanence).** A drive moves a kind's pole state
  `t = U / W` away from its rest: the push `h / (R + h) (1 + A_in(t))`
  with the inbound echo `A_in` (`circuit::push_at`, `Substrate::inbound`,
  `inbound_mean`), the jet map `t -> a t + b` (`circuit::jet_map`) and the
  jetstream's residual dilation (`Substrate::loop_flux`). When the drive
  falls back to its mean, the part of the displacement that stays held is
  the remanence, and the phase weakens by what is released. The candidates
  are read from the dipole's split, `D(p_pair) = D(n) D(n / 3)` (The
  antipoles, the shear and the pinch): `n` held as torsion, `n / 3` released
  as writhe. The coercivity is the reverse push that takes the held part
  back to zero.
- **Phase strengthening (the inverse).** The antipole of weakening drives
  the pole state further from its rest. Three sources are to be derived:
  - mass at distance from a core, a star's own included, through the
    deficit `M / (r + M)` at its reach (`xyl_core::shares::deficit`);
  - a throat with an Einstein-Cartan bounce, whose far face inverts
    `t -> 1 / t` and the axis polarity `A -> -A` (`bounce`; The bounce at
    a pinched throat);
  - core fusion, while the collective is on the Z side, `t < 1`
    (`stars::fusion_step`; Polarity, fusion and fission).
- **What the code lacks.** Every law is a closed form of the present state.
  The jet map is affine and invertible. The electron phase
  `R = kappa (1 - 2 frac(p))` (`circuit::electron_phase`) reads the current
  pulse phase. The T8 holding is a threshold on the present state, not a
  memory. A kind's axis polarity at rest is neutral, and there is no
  coercivity quantity. The law needs one carrier: the held residual as a
  closed form of the drive's history, with no time-stepping.
- **What it would give.** A cooling front that freezes a kind's axis
  polarity at the phase of the pulse it formed in, so that a world's matter
  keeps a physical record of its formation for archaeology and the annals
  to read.

### Scalar collapses: the register and the order of opening

Planned ([#107](https://github.com/SAIOS-SAMN/xylarium/issues/107), roadmap
item 43); scanned 2026-11-05 across the five crates, 50 sites, 44 of them
load-bearing. Each is a quantity the model carries with more structure
reduced to one number, a graded reading reduced to a boolean, or several
candidates reduced to one by a scalar. Mu and the TUniversal points are
clean: their lanes are never summed, ordered or compared as one number.

- **The atom and the bond.**
  - Stripping is all or nothing per band (`atoms::unpaired_hot`), so
    `compound::stands` is a boolean.
  - A bond is read across its two faces (opened, `atoms::Bond`): on the
    axis plane each face's pole push and polarity, the pushes composed by
    the flow law; on the isoclinic planes every open winding of each face
    with its tightness. Its hold rule is still a cut (held whole within
    a winding's tightness, else tightness over turning, the turning capped
    at one), so `xylarium observe bond` reads every winding tighter than
    hydrogen's held whole at any turning: the graded stripping law is to be
    derived.
  - The atom's hold sums its modes into one number (`atoms::hold`).
- **The founding and the stars.**
  - The spin orbit folds each kind's separation into one spread
    (`circuit::Substrate::spin_orbit`).
  - A bead is read from its most abundant kind.
  - A star's sequence steps one kind per stage, the greatest release
    (`stars::fusion_step`, `stars::fission_step`), on one pole point for
    the mixture (`stars::mixture_pole`).
  - `stars::Sequence::ignites` is a boolean that decides whether a pulse
    node exists.
- **Galaxy, systems and worlds.**
  - Booleans decide what exists: `galaxy::Star::in_core`, and the ancient
    flag it sets; a world's being a world; `systems::World::icy`; a
    giant; tidal locking with an equal-split season.
  - Mass-weighted means stand for merged bodies and for a world's reach.
  - `chain::Place::arc` folds a relation across tiers into one distance.
  - `pulse::hear` sums every bell into one phasor.
- **Life.** A world keeps its single best lineage, its best medium decides
  its chemistries, and sterilisation and a mind's completion are booleans
  where graded survival shares exist.
- **The kernel.**
  - **Peoples:** they move to the world nearest by array index; a fall is a
    boolean; turning points, figures and the lead domain are picked by a
    scalar; focus is one domain, and knowledge growth sums the four
    domains.
  - **Transit:** the fastest drive is picked; the change of velocity is
    split equally; the ley network grows by one winner at a time; binding
    and quarantine are booleans at one half.
  - **Percepts and clocks:** a neighbour is heard or not by one distance,
    and a world has one temperature; chronometry assumes a closed repeat
    and sums distances across tiers.
- **xyl-core.** `Encl::mid` stands for an enclosure in load-bearing places,
  and the coordinate's H lane is described as one plane.
- **The uplink.**
  - The T reading is floored to a whole ring.
  - A mind's act is one count.
  - Every act is written on the S lane.
- **Order of opening.**
  1. The bond and stripping as graded shares, read by both faces and their
     separation.
  2. Candidates held together, never picked: steps, cohorts, beads,
     lineages, media, drives, the ley network.
  3. Booleans to shares where they gate existence.
  4. Relations held by tier, the pulse as the per-bell list.
  5. Lanes and planes kept whole: a four-share focus, knowledge growth per
     domain, the H lane per plane, the uplink's exact T and per-kind lanes.
  6. Enclosures kept where a midpoint is stored or decides.

### Chemistry from the elements

Planned ([#103](https://github.com/SAIOS-SAMN/xylarium/issues/103), roadmap
item 42), with the galaxy's gas (#8), condensates (#15) and geochemistry
(#17). Chemistry and biology are not modelled: they emerge from the
founding's physics, read at the atomic tier by the same laws the throats
obey. Nothing here names a state, a species or a substance in advance;
what the physics produces is read, and the minds of a universe name it
(Names emerge, civilizations.md).

- **The law at the atomic tier.** A bond is a paired throat at the
  smallest tier: two atoms' faces as an attractor pair, each face advancing
  by that atom's torsion share, the bond read by its separation across them,
  `rho = q ln D(s) - (1 - q) ln D(1 - s)` (T17), never by one tightness. It
  holds its turning as remanence between passages and is cleared at its
  coercivity (T19). The automorphism of the chain carries the law from the
  core down to the atom unchanged (The automorphic, synaptic chain).
- **The nuclear and the atomic: orthogonal, analogous, coupled.**
  - **Orthogonal in the planes.** The nuclear acts on the axis plane: the
    Z-Q stretch, the pole state `t = U / W`, the dipole `D(n)` and the axis
    polarity, a hyperbolic dilation. The atomic acts on the five isoclinic
    planes: the bands' Hopf fibres, the electron phase, the twist charge and
    a bond's turning. The two lie in orthogonal planes and commute (Weights
    on the six planes), so a nuclear change and a chemical change are
    distinct components and never one number.
  - **Analogous in the law.** One attractor pair, separation and remanence
    at both tiers: a nucleus is its Z and Q faces, a bond its two atoms'
    faces, and both release held count.
  - **Coupled through the occupancy.** Each proton's three units of span stand on both
    (axiom 9): the open states `U` are the unpaired electrons, the atomic
    side, standing at the nucleus's Q pole; the held count `W = 3Z - U` are
    its bonding at the Z pole. Pairing electrons moves states from the Q pole
    to the Z pole, a chemical act changing the nuclear pole state, and
    stripping moves them back; a change on one set of planes shows on the
    other only through the occupancy.
  - **Received turning carries both** (`atoms::Received`): the axis plane's
    push, which the pole state reads (a star's compactness), and the
    isoclinic planes' turning, which strips the windings and sets a bond's
    hold (`atoms::unpaired_hot`, `compound::held`), each read by the laws
    of the planes it acts on (`stars::Sequence::of` reads both).
- **Built from the elements.**
  - The atom: its bands filled on S^11, its open and held count, its
    tightness and its stripping by received turning (`atoms::atom`,
    `atoms::unpaired_hot`, `atoms::tightness`), its two-atom bond read
    across both faces by planes (`atoms::bond`, `atoms::Bond`), and its
    weights on the six planes: the axis polarity
    on each plane (`atoms::plane_polarities`, T7) and the twist charge on
    the isoclinic planes (T6).
  - The two-part closure: the smallest cluster of two atoms with every
    state paired, `(u_B / g) A . (u_A / g) B` (`compound::closing`,
    `compound::held`, `compound::stands`).
  - The star's nucleosynthesis: steps by release (`stars::fusion_step`,
    `stars::fission_step`, `stars::Sequence`).
- **Where the chain is not yet emergent** (an inventory of 2026-11-05, 55
  findings). Every one is a calibration to the Earth, the Sun or the Milky
  Way, or a catalogue standing where the physics is to be read:
  - **Stars to galactic gas.** Enrichment reads a channel-share column of
    the element table rescaled to the Milky Way and the Sun (`chem`'s
    calibration), never the stars' products (`stars::Step::products`).
  - **Gas to solids.** Condensation, the split of a body's matter and its
    frozen matter are tables (`systems::ICES`, the mantle partition
    `MANTLE_KEPT`, the element table's condensation temperatures and
    classes); `compound::closing`, `compound::held`, `compound::stands` and
    `atoms::bond` have no consumer outside `compound` yet.
  - **Closures to volatiles.** Six catalogued species with Earth's phase
    diagrams (`worlds::volatiles`), whose formulas disagree with the closure
    law (`compound::closing` closes oxygen with four hydrogen states
    paired); the greenhouse and albedos are calibrated to Earth.
  - **Nuclides.** One mass number per element, from the table's atomic
    weight (`circuit::mass_number`); four Earth isotopes and an Earth
    inventory stand for radiogenic heat (`bodies::isotopes`).
  - **Life and civilisation.** Bond energies in electronvolts, cohesion from
    heats of vaporisation, chosen element groups, a biotic window in kelvin
    and a metabolism ruler are tabled (`worlds::media`, `life`); the drives'
    constants are tabled and offered without reading a world's elements
    (`xyl-kernel propulsion`).
- **The vents to open.** Each replaces a scalar with the structure the
  physics carries:
  1. **Received turning, not heat.** The pulse a body absorbs is a versor
     across the planes (the field law). What a bond network receives is the
     H-lane components it delivers, per plane and between planes, with the
     return read from the deficit at the body's reach, never one temperature.
     Opened in part: the carrier `atoms::Received` holds the axis plane's
     push and the isoclinic planes' turning apart through the atom, the
     two-part closure, the star cut and the star's sequence; the turnings
     between planes and the return are to come with the rotor carrier
     (roadmap item 40).
  2. **Closure held, not chosen.** Every closure a mixture admits is held
     together, a superposition, never the one releasing most. Passage,
     perception or the network's own shear collapses it, as a vortex is
     collapsed (MU-RECURSION law 10).
  3. **States as orbits, not phases.** A bond network's state is its sheared
     orbit across the planes: which of its links hold their remanence and
     which pass, and how the received turning advances them. Held
     throughout, flowing, released and stripped are readings of that orbit;
     the minds name them, and no phase list or diagram is kept.
  4. **Nuclides on every plane.** A nuclide is the attractor pair at the
     nuclear tier, its Z and Q faces with the separation and twist of the pair;
     its stability is that pair's reading against the dipole of the tier it
     formed in (T2), and its decay a passage clearing remanence, an event in
     closed form. A ratio frozen by a cooling front is a physical record
     (Phase weakening and phase strengthening).
  5. **Living matter on the synaptic chain.** An organism is a set of links
     whose weights are their remanence: memory and heredity are held
     turning, metabolism is passage through the links, reproduction is the
     automorphism (a copy commuting with the change of tier), and selection
     is coercivity, what can clear a link's weight and what cannot. Of the
     two drives given, eating is taking a passage and reproducing is the
     copy; everything else emerges.
  6. **No Earth units in a law.** Energies are releases of held count and
     received turning is in degrees on its planes; kelvin, electronvolts and
     joules stand only at the SI seam, for display (`package::Si`).
- **Order of the work.**
  1. Received turning per plane (vent 1).
  2. Closure held as a superposition (vent 2), over two-part closures first,
     then the networks a mixture admits.
  3. States as the network's orbit (vent 3).
  4. Nuclides as attractor pairs (vent 4).
  5. The galactic gas from the stars' products (#8), and the solids, frozen
     matter and volatiles read from 1 to 4 (#15, #17).
  6. Living matter on the synaptic chain (vent 5, with #97).
  7. A people's chemistry from its world's own elements.

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
  is its open states over its held count, the Q pole's share
  (`formation::emission`; `U / 2P` on the capacity, axiom 9).
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
  spans in galaxy units (60^5 r_g) and their states as their holding. The
  core's reach is one galaxy unit, where `1 / (r + 1)` is one half
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
  galaxy unit is `lambda = (state / unit) / W`; its agitation is
  `(p - delta) / p` with `delta = h / (R + h)` and `h = 1 + held share`; its
  emission is `e = sum s_z open_z / hold_z`, and its magnetism is
  `mu = lambda e`. Outside the core's reach the gas is a circle (`n = 1`);
  within it, a ball (`n = 2`). An infall of extent `w` gathers
  `m(w) = (lambda + mu) W share_n(theta)`, with `theta = w / (w + R)` of
  180 degrees on the infall's plane (`xyl-core measure::share_turns`), and grips it as
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
  measured in galaxy units, the same unit in which its piece was lit; kpc
  appear only at display and at the marked boundaries of the layers not yet
  converted (`galaxy::kpc_of_units`).
- Placement (`galaxy::star`): sample stars stand for equal shares of the
  stellar mass and are placed along the rings' cumulative holding by
  irrational pivots (the golden, silver, plastic and `sqrt 3 - 1` angles;
  a placement by index, to be replaced, roadmap item 40), which
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

### The formation chain

Planned ([#208](https://github.com/SAIOS-SAMN/xylarium/issues/208)). Stars, systems and world states are drawn and
calibrated (the prototype's population synthesis, `e11a9ec`), not formed
from the founding; the stand-ins are registered (docs/assumptions.md). The
chain, in order, each step a founding change:

1. **Matter:** `Q` from the conversion's rest point at each core's bounce
   (E014, [#201](https://github.com/SAIOS-SAMN/xylarium/issues/201)). Built for the universal table (S029 retired); the
   galactic cores' tables remain (S138).
2. **Stars:** condensed from the galactic core's held gas, mass and place
   read where the hold closes; retires S006 to S014 and S020
   ([#191](https://github.com/SAIOS-SAMN/xylarium/issues/191)), each remnant from the remnant's compactness (S003,
   [#188](https://github.com/SAIOS-SAMN/xylarium/issues/188)).
3. **Worlds:** condensed from the star's held mixture
   (`disc::StarHold::held`, `disc::manifold_worlds`); retires S066 to S070.
4. **States:** world states and life from their matter and their star;
   retires S041 to S063, then S073 to S081.

The sections below describe the readings as built.

- The transduction (`stars`): a star at reach `r` absorbs the ring that the bell
  brings there, `1 / (pi (r + 1))` per bell, plus the vessel's drift `bell q(t)
  / (r + 1)`, and holds its compactness `delta` of that absorption: its deficit
  at its rim, `delta = m / (R 60 + m)` (`disc::rim_share`), with `m` the star's
  mass in core masses and `R` the rim of the held core its cloud's cut gives, in
  system-tier units (`galaxy::piece_rim`, `galaxy::Star::compactness`). The rim
  comes from the atoms: the cut carries each kind to a spin from its open states
  and its mass per nucleus and holds it where its deficit meets the push
  (`formation::hold_at`). The cut is read in three states. At collapse the
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
  `h / (l / h + h) >= p` on the held hold (`formation::hold_at`). The
  disc's agitation is `turning x swing x grip / shed`, with
  `grip = (1 - star) / (2 - star)` and `shed = e_rest + ln 2 / k_v`.
- The star's manifold (`galaxy::disc_manifold`): the star's gathering extent is
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
  line; its phase is its index times the golden angle on the disc's plane. A core of ten world
  masses or more acquires an envelope by the declared gathering law.
- Settling (`systems::settle_about`): about an anchor of mass `M` in core
  units, each body turns by the turning law, `omega(A) = bell [M / (A + M)] / A`
  per ring (`turning::turning_dy`). Neighbours meet `N = age |dw|` times,
  and each meeting kicks by `eps = omega_m(r) / |dw|`, the turning law about
  the pair's mass across its gap. The kicks repeat on the continued
  fraction period `q` of the phase step and cancel to
  `c = |1 - 2 frac(q psi_0)|` per period, so the greatest swing is
  `eps (floor(N / q) c + N mod q)`. A pair merges at the first age at which
  its swing spans its gap, found by 48 halvings out to the horizon. Merges
  before the settled age (3 Gyr) combine the pair, mass-weighted; later
  meetings are recorded on the lighter body as its merge age
  (`World::merged_at`). A body's starting eccentricity is its swing, and its
  tilt is `swing / (1 + swing)` of 90 degrees toward the axis of its
  star's plane (`bodies::orbits`).
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
- Angles between bodies: `manifold::gap` and `chain::Place::arc`
  read the angle between two directions as the certified angle between them
  (`xyl_core::rotor::angle_between`): the directions stay exact and only the
  angle, which has no rational form in general, is an enclosure.
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

- **Held and open states under irradiation.** A kind `z` irradiated at
  heat `q` (the turning the star receives per echo, in hydrogen units)
  keeps `u_z(q)` open states (`atoms::unpaired_hot`), its magnetism, the Q
  pole; its held count is `w_z(q) = 3Z - u_z(q)`, its bonding, the Z pole
  (`compound::electron_hold`; on the capacity its held states `Z - u_z(q)`,
  axiom 9).
- **The collective's pole point.** Over the held mixture, with `n_z` the
  count of nuclei of each kind (its share of the hold over `w_z`),
  `t = sum n_z u_z(q) / sum n_z w_z(q)` (`circuit::pole_point`). The star's
  polarity is the inversion `1 / t`: bonding over irradiated states.
- **Irradiation carries the pole point.** A held kind's open share is at
  most `Z` of its span `3Z`, so the built `t = u / w <= 1/2` at ignition:
  every collective starts on the Z side. On the capacity, `t = U / 2P` has
  no such bound (#158). Each echo carries the pole point outward by the
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
  proportion to their held count. Each step's duration is its transduced share
  over the star's transduction per bell (`stars::transduced_per_bell`); the
  first step is the main sequence, later steps the giant phases, and the hold no
  step transduced is the remnant (`stars::Sequence::of`, `stars::fusion_step`,
  `stars::fission_step`, `stars::mixture_pole`, `circuit::echoes_to_throat`).
  Status: built.
- **A held body that does not ignite.** Where no step of the held mixture
  releases hold under its polarity, the body holds its matter and transduces
  nothing.

Open, read from the emergent sequences (cold hydrogen gives four
hydrogens to the kind of four nucleons, three of those to the kind of
twelve, two to the kind of twenty-four, then eight to `Z 77` and ten to
`Z 752`; 296 kinds of the table take no step in either direction):

- The held count per nucleon fall with the nucleon count across the whole
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
  holder holds, `rho = (1 + p) / (1 - p)` (`shares::dilation_dy`): `p = 1/2`
  for a firmament kind at the throat, the compactness `delta` for a star's
  collective. It moves `(rho - 1) u w / (w + rho u)` from the Z pole
  to the Q pole.
- **Vacuum (inflow).** The return is through the vacuum's share at the
  holder's reach about the universal state, `p_v = H / (R + H)`
  (`xyl_core::shares::deficit_dy`), swung by the inbound echo; it moves `p_v w`
  from Z to Q.
- **Electron phase.** The electrons' phase state `R = kappa (1 - 2 frac(p))`
  moves `R u` from Q to Z (`circuit::electron_phase`); over a whole
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
| Remnant | Every step of the sequence has ended: the star keeps the hold no step transduced (`stars::remnant`) and becomes a hole in the pulse network (`pulse::nodes`); a held body that does not ignite forms none. | Stand-in: every igniting star is made a hole at its first step's end (S001, S002; [#188](https://github.com/SAIOS-SAMN/xylarium/issues/188)). |
| Compact remnant (white dwarf), denser remnant (neutron star) | A remnant whose held core's rim, from the remnant's cut of its remaining mixture, gives compactness below one half; the two separated by where the remnant's held kinds lose their open states. | Next: needs that cut of the remnant's mixture. |
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
