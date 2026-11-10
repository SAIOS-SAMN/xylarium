# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html). Until 1.0.0, minor versions
may change public APIs and the model's laws.

## Unreleased

### Added

- The bench reads the entered capacity and candidate closed shares
  (#172): `cap_in`, twice the orientations of every band holding an
  electron (`Z <= cap_in <= cap`, proven over Z 1 to 1700), and the closed
  share as `2P / cap`, `2P / cap_in` and `Z / cap_in` beside the built
  `W / 3Z`, with the bits each chain needs to the vessel's top (seed 7:
  5546, 1996 and 1568 against 1094). It also reads the capacity face's
  divergent share `U / Z` beside the emission. Readings only. The
  founding is unchanged.
- The substrate bench (#163, board B1): `observe bench [LO] [HI]` reads
  the substrate alone, with no founding and no cache. Per kind it prints
  the occupancy, the held count and closed share, the capture's step per
  band, the pole `U / W` and `U / 2P`, the emission, and the chain's count
  per hydrogen. Then the founding bits and the seals' counts. The seed's
  spin is read by `origin::drawn_spin`. Every capture adds one electron
  over the bands it changes, proven over Z 1 to 1700.
- The board's design (docs/board.md, #162): a blank S^360 universe with
  no founding, where cores, stars and worlds are placed as conformal
  spheres about their anchors and nuclei of each kind are added or
  removed, read by the built laws; scoped in phases B1 to B6 (#163 to
  #168), the substrate bench first.
- Axis weights in ZZ (#158, vent 3): `atoms::axis_weights` gives a kind's
  open states' axis weight `sum m0 U_m0` (the polarity `A`) and its held
  states' `sum 2 m0 P_m0`: a pair cancels its twist charge on the
  isoclinic planes but its two electrons share the axis level. A seal is
  balanced (both zero); a band at least half full holds minus twice its
  open weight, one below half none; proven over Z 1 to 1700. `observe seals` prints
  both. The isoclinic signs wait on the spin (#128). No reading moves.
- The occupancy on the axis plane (#158, vent 2): `atoms::Band::axis_occupancy`
  places each band's room, open states and pairs by axis level `m0` (the
  level holds `C(l - |m0| + 9, 9)` orientations; singles fill from the Z
  side, pairs from the Z side beyond half full), and `atoms::axis_sides`
  sums each kind's Z side, neutral level and Q side; the axis polarity
  reads it. Proven over Z 1 to 1700 against the singles-only rule it
  replaces. `observe seals` prints the sides. The isoclinic planes' split
  waits on the spin (#128). No reading moves.
- The capacity's occupancy per band (#158, vent 1): `atoms::Occupancy`
  (room, open states, pairs), `atoms::band_occupancy` over every band an
  atom reaches (empty bands included) and `atoms::occupancy`, their sums,
  `cap = E + U + 2P`; `atoms::Band::unpaired` reads it. `observe seals`
  prints each kind's room, open and held states and both pole readings
  (the built `U / W` and the capacity's `U / 2P`). Proven over Z 1 to 1700:
  the occupancy partitions the capacity and its open states are the
  unpaired count. No reading moves.
- The capacity's occupancy, the vents to open (docs/model.md; #158;
  roadmap item 56): `W = 3Z - U` and `U` keep two counts of the atom's
  bands, planes and fibre windings, the winding read mod 2. Seven vents
  (states per band, per plane, windings in ZZ, classes, the delimiter
  retired, seams, received turning), with their sites. No code changes.
- The dipole, winding and phase, and the chain's cohomology stated
  (docs/model.md; #148, #149, #150; roadmap items 50 to 52). Every
  component of a reading is a dipole, `3Z/3Q + R` on the Z face and
  `3Q/3Z - R` on the Q face, calibre-free, with the seals as its equator
  (`Z = cap`). A turn carries its winding in ZZ and its phase in QQ/ZZ.
  Pairs are coboundaries on the containment chain, loops cocycles, the
  books' closing the cocycle condition, and twist, remanence and sheared
  returns its holonomy. The three meanings of Q are #151. No code changes.
- The capacity in the cosmos (#151): `atoms::capacity` (`cap`, `3Q = 3
  cap`), `atoms::balance` (`R_Q`, `rho_Q`), `circuit::kind_capacity` and
  `circuit::mix_capacity`; `observe seals` reads them (it computed the
  capacity itself before), prints each kind's pole point `t = u / w`, and
  compares the founded mix's `3Q / 3Z` (as neutrons and as capacity) with
  the throat's `s_Q / s_Z`. Proven over Z 1 to 1700: `rho_Q = +1` exactly
  where `Z = cap`, exactly at the seals, each closed (`t = 0`, the Z pole).
  For seed 7 neither reading of Q reproduces the throat.

- The path (docs/roadmap.md): the destination, where the universe stands,
  and five stages as GitHub milestones (M3 to M7) with exit readings.
- Calibre and dimensional capacity (docs/model.md, #123): a share reads its
  reach and hold in one calibre; every EQUATIONS.md entry names its calibre.
- The vortex (docs/model.md, #121): the shape the flat structures become.
- The pair rule (docs/model.md, #118): pair readings through the pivot, per
  plane, in one calibre.
- The register of scalar collapses (docs/model.md, roadmap item 43, #107):
  50 sites across the five crates and the order of opening them.
- Embedded minds planned (docs/network.md, roadmap section 4 items 10 to
  12, #108): the client's cognition hosted by the uplink, one per ancient
  seat, through the house's seat-surface path; the uplink's first fixes.

- Chemistry from the elements recorded (docs/model.md, roadmap item 42,
  #103): the chain from the atom through the star to matter and life by the
  release of held frames, the inventory of where it breaks today (55
  findings across eight stages), and the order of the work. Notes that the
  fixed volatiles' formulas disagree with `compound::closing` (water closes
  as O.4H).

- T19, a link's remanence (docs/model.md, #84): `w = J - (a_0 M_0) M`,
  built and worn by passages by their separation, conserved between them,
  cleared at its coercivity. For seed 7 the universal core keeps 0.1982 of
  its turning as remanence. Corrects #97's "fades with disuse": the weight
  is conserved between passages.

- The automorphic, synaptic chain (docs/model.md, roadmap item 41, #97):
  one attractor-pair map at every tier, commuting with the tier's dilation,
  and links weighted by their history. At the law, the deficit and the flow
  law are invariant under every tier's dilation
  (`shares::tests::deficit_is_invariant_under_dilation`).

- The core as a paired throat, built (#86). The firmament is the closed
  kinds (T16; for seed 7 it was 525 kinds of the vessel band). Each kind's
  axis polarity is `atoms::axis_polarity` (T7). The universal core's spin
  orbit is `circuit::Substrate::spin_orbit` (T17), and the anchor holds the
  core's spin after it (`circuit::Anchor::core_spin`). `xylarium observe
  universal-core` prints the cohorts and whether the pair closes.

- Minds in the ancient seats (docs/civilizations.md, docs/network.md,
  roadmap section 4 items 6 to 9, #94): what is built of a mind seated on
  both faces, the house in the kernel and its key on the hypha, and the
  polity still to derive (the council's convergence, the house's relay,
  consciousness passed down, the seat's place on the paired throat).

- T18 (docs/model.md, #90): founded matter turns its five isoclinic planes
  together, so one isoclinic phase per body. A frame is the axis-plane
  phase, that phase and the turning between planes into its parent.

- The audit of the frame, the place and the chain recorded as planned work
  (docs/model.md, roadmap item 40, #90).

- The throat's spin as a sheared orbit (docs/model.md, T17, #86). A kind's
  separation across the attractor pair of the faces,
  `rho = q ln D(s) - (1 - q) ln D(1 - s)`, apportions the turning each
  cohort takes. There is no equal split and no admission; whether the core
  closes is read along the orbit. For seed 7, `a* = 0.9933`.
- The core as a paired throat (docs/model.md, T16, #86). The near and far
  faces dilate an open kind by factors whose product is `(3 + a)^2`, so the
  pair holds on both faces only its closed kinds.

- Phase weakening (remanence) and phase strengthening recorded as planned
  work (docs/model.md, roadmap item 39, #84), with the code they build on
  and what that code lacks.

- `xyl_core::cga` is `Cl(p, q)` (Concordance 2.10, draft 5): `Mv<NEG>`
  carries the signature in its type, `Mv` is the conformal `Mv<EM>`, and
  two signatures never compose; the reordering sign is read in closed form
  from the masks; the four passage fixtures of 2.11 hold in `Cl(3, 2)`.

- The bounce at a pinched throat (docs/model.md, EQUATIONS.md, #78): the
  far face inverts each layer's pole state and axis polarity and turns its
  phases half a turn; the new throat's own jet releases it; for seed 7 the
  gate gives eleven throats in one chain, the bulk's climb over the pinch
  interval about 0.67.

- The state at the pinch (docs/model.md, EQUATIONS.md, #35): the fold as a
  thick, layered roll whose least radius its grip sets; the neutral surface
  from the layers' strains composing to the identity,
  `sum m nu ln(r / r_0) = 0`; seven circuits read per layer and plane
  (pole state, heat, twist, place, pressure, pairs, fold); for seed 7 the
  first pinch encloses 605 layers, 0.1783 of the founded mass inside the
  neutral surface, 0.3865 outside, 0.4569 without grip.

- The founding's axioms (eleven) and theorems (T1 to T15), with holding in a
  pinched throat per plane (T14: push `J_j / (6 H_j + J_j)`, none on the
  axis plane) and per nucleus (T15: planes composed by turning share); the
  pinched throat's spin from the Kerr relation, `a_g = a_p M_p / H_g`, with
  degrees of turning conserved (axiom 11), a cohort with `a_g > 1` staying as
  medium; the gate's reading for seed 7: the core holds 0.4352, one
  pinched throat 0.4569, 0.108 stays as medium (docs/model.md,
  EQUATIONS.md, #35).

- What a pinch carries (docs/model.md, EQUATIONS.md, #60, #35): the next
  throat's spin (now `a_p M_p / H_g`, by the Kerr relation), its place, its
  hold as a reading of the flow, and the stretching pressure a throat's
  spin puts on each state, `(D(n)^m0 - 1) / (D(n)^m0 + 1)`.

- `xyl_core::orbit`: an orbit's exact reading (axis dilation by exponent,
  each plane's periapsis and anomaly in Q/Z), its sheared return
  `k / (1 - theta)`, its return to the form against facets and its
  capacity (#60). `coordinate::Reading::within` compares exact shares
  against exact sector bounds, and `arc` writes degrees, minutes and
  seconds with an exact remainder; the written places are display only.

- The local ring (#60): at reach `r` one traversal of the orbit there,
  `s r (r + M) / M^2` core traversals exactly; every law step at a reach
  (turning, the vacuum's halving, cooling) counts local traversals.

- The ring is specified as the throat orbit's reading, never one scalar
  (#60): the axis dilation by its exponent and each of the five planes'
  phases in Q/Z, with durations in `r_g / c` or seconds as display only;
  an epoch is a position in the nested reading, and play's advance per loop
  is read per network segment.

- The antipoles, the shear and the pinch (docs/model.md, EQUATIONS.md,
  #60), each checked exactly: the antipole group keeps multiplicity; the
  flip `c -> -c` keeps every state; the antipode's deviation
  `p_pair = n (+) n/3`, the dipole reconciling `n` and the residual `n/3`
  folding forward on the near face and back on the far; `Lk = Tw + Wr`
  conserved; the pinch where the folds meet, after `3 (4 + a) / (2a)`
  echoes at half a turn (seed 7: ring 37 at 222 deg 52').

- Throat holding refined by the axis moment (docs/model.md, EQUATIONS.md):
  an open kind dilates by `D(s) D(n)^A` per echo and is held where that is
  at most `1 - 2 delta`; for seed 7, 461 open kinds with `A <= -8` are held
  beside the 20 closed ones.

- Weights on the six planes (docs/model.md, EQUATIONS.md, #60): a band's
  harmonics split by weight in Z^6, `C((l - sum |mi|) / 2 + 4, 4)` each and
  `C(l - |m0| + 9, 9)` by axis weight (tested); a state's flow as a closed
  form, axis dilation and plane phase; filling Z side first; each kind's
  emergent axis polarity.

- The equator, derived (docs/model.md, EQUATIONS.md, #60): the equatorial
  inflow `D(-s)` per echo by continuity (`s (+) (-s) = 0`), the swirl as the
  passed share, viscosity `nu(t) = t / (1 + t)` reverting to `1 - nu`
  across the throat, the positron as the electron's phase under `x -> -x`,
  and the pair share `p_pair = s (+) (-(1 - s)) = a (4 + a) / (12 + 6a + a^2)`
  per echo, with exact values for seed 7.

- Throat holding, derived and tested: a kind is held exactly when it has
  no open frames; every open kind crosses within `ln(W / U) / ln D(s)`
  echoes (`a_kind_is_held_exactly_when_it_has_no_open_frames`, Z 1 to
  1700). The founded firmament under this law is 20 closed kinds (seed 7);
  22 closed kinds between Z 1093 and Z 1700 are unfounded.

- The ring as the throat orbit, specified (docs/model.md, EQUATIONS.md
  section 4, #60): one traversal of the core's sheared elliptic orbit
  (`e = delta`, apsis ratio `D(delta) = 1 + 2M / r`); resonances
  `r_k = (k - 1) M`; the echo's twist `n = a / (4 + a)` held by the dipole
  as torsion, `Q / Z = 1 + a / 2`; readings over the lowest common
  denominator `L = 4N + m` and their exact derivatives in spin and reach.

- `xyl_core::shares`: the deficit share, the dilation `D(p)` and the flow
  law (`flow`, `opposed`), exact on `Q` with `Dy` forms; the duplicated
  deficits in `circuit`, `turning`, `disc`, `formation` and `stars` read it
  (#42, #43).
- The coordinate in degrees of arc (`xyl_core::coordinate`): a protractor of
  `360 x 60^5` marks, places `DDD:MM:SS:XX:XX:XX`, the whole turns kept
  (`Reading::rings`, the rings on the T lane) (#40).

- The founding as flow, specified (docs/model.md): parcels released at
  onset move by the reach rate `dr/dtau = [R_h(H) - R_h(m)] r / (r + R_h(m))`;
  angular state in degrees; perturbation on frames; meetings by contrast
  times coherence; concentrations as throats (#35). Roadmap items 28-37 and
  issues #36-#44 for the misalignments found in the physics library.

- The shared-laws ledger (docs/EQUATIONS.md section 8): the laws proposed
  for the core shell of mycelium's `forms`, with proposed ids (#33).

- The jetstream for every holder (`circuit::jet_map`,
  `circuit::throat_crossing`): out through the continuum, back through the
  vacuum; the firmament's onsets take the vacuum leg (the founding
  changes: onsets come earlier) and a star's collective crosses the throat
  by the same law. `stars::timeline` runs fusion steps until the crossing
  and fission steps from it; `stars::remnant` is what the steps that ran
  did not transduce.
- One law per formula: `bounce::torsion_share(mass, spin)`,
  `circuit::dilation`, `circuit::echoes_to_throat` (the duplicates in
  `origin` and `stars` are removed).
- Stellar transduction by polarity (`stars::Sequence`): a star's held
  collective's pole point `t0 = sum n u / sum n w`, dilated per echo by
  `(1 + delta) / (1 - delta)`, selects fusion before the throat and fission
  after it; each stage steps the held kind of greatest release, its
  products join the mixture, and no kind, product or order is set. Field
  `Transduction` states: unlit, step, remnant. Star classes name the held
  body that does not ignite and the fission star.
- The word "burn" is replaced by "transduction" throughout
  (`stars::transduced_per_bell`, `field::Transduction`,
  `field::transduced_share`).
- `Q::ratio` is the constant-ratio constructor; it returns zero on a zero
  denominator.
- `AGENTS.md` and `docs/concepts.md` are added, and the glossary is
  rewritten as a lexicon with code paths.
- The examples `found_and_observe` and `minimal_link`
  (`crates/xylarium/examples`) are added, with `# Examples` sections on
  `run::genesis`, `run::run`, `run::run_with` and `observe::observe`.
- `scripts/check-docs` and `scripts/check-style` are added and are run
  locally with the other checks (CONTRIBUTING.md, Verification).

### Changed

- `Dy` has no `/` operator (#179, MU-STANDARDS 2.4, rule 11): it returned
  zero for a zero divisor. `Dy::div` refuses a zero divisor, and
  `Dy::over` divides by a nonzero integer. `nonzero!(d)` and `dyadic!(n, d)`
  replace `Dy::ratio`, with the denominator checked at build time. In
  `xyl-core` each division is decided: a zero divisor where the law has a
  value (atan 0, acos 0, a hold of nothing), a divisor proved positive
  where it cannot be zero (pi, the measure of a sphere, `1 + sqrt(1 + x^2)`),
  and a refusal otherwise (`powi` takes a natural power). In `xyl-cosmos`,
  `xyl-kernel` and `xylarium` every division is written out as
  `a.div(&b).unwrap_or_default()`, so its zero default is visible at the
  site for that crate's rule-11 pass. No reading moves.
- `Q::ratio` is replaced by `ratio!(n, d)` (#179, MU-STANDARDS 2.4, rule
  11). The denominator is a constant checked when the build evaluates it,
  so a zero denominator fails the build instead of reading zero. The six
  call sites with a computed denominator read `Q::new` instead. `Q`'s
  `Small` arithmetic divides through `Q::over_positive`, proved by the
  invariant that a denominator is positive (a property test). No reading
  moves.
- `stars::held_frames` is renamed `stars::held_count`, retiring the last
  code use of "frames" as a count (#172).
- Axiom 9 restated on capacity, not delimiters (ruled 2026-11-10, #158):
  a kind's `Z` electrons occupy its capacity, `cap = E + U + 2P` (the
  room, the open states, the held states); shares are read against the
  capacity (occupancy `Z / cap = 3Z / 3Q`, held `2P / cap`, open `U / cap`,
  room `E / cap`) and the pole state is `t = U / 2P`. "Frames" as a
  nucleon count is retired across the docs and module comments: open
  frames are the open states `U`, held frames the held count
  `W = 3Z - U` (three per proton, the built delimiter, to be re-read on
  the capacity, vent 5); "frame" keeps only its geometric meaning. The
  frame face is now the capacity face; the vents section is *The
  capacity's occupancy*; glossary gains *Occupancy* and *Room*. No code
  changes value; the laws reading `W` are flagged where they move.
- The six planes, a face per plane (#128, #121; docs/model.md): each plane
  reads gravity's face law with its own pole (`t_0 = r / M`,
  `t_j = J_j / (6 H_j)`, the horizon's `t_s = 1 + J / H`); a face's
  dilation is linear in its pole (`D(d) = 1 + 2t`, `D(c) = 1 + 2 / t`,
  `D(d - c) = t`); T2 is the face law at the horizon and T14's plane push
  a divergent face (tests in `shares` and `turning`); T18 restated as
  separation, never an equal split (the plane's hold the kinds' turning on
  it, `J_j` its part of the spin; `J / 5` removed); axiom 9's capacity reads
  the same face law between held and open states (`t = U / 2P`).
  Two counts named apart
  (#151, ruled): the mass weight `m_Z = Z + Q` (neutrons) is the hold,
  the capacity `cap` the room.
- Gravity is the topological deformation of the manifold about a mass
  invariant, with convergence and divergence as antipoles (ruled
  2026-11-10, #155; docs/model.md, Gravity; axioms 2 and 5 restated).
  `xyl_core::shares::faces` gives both faces, `c = M / (r + M)` and
  `d = r / (r + M)` (`t = r / M`); `Faces::antipole` (the inversion
  `t -> 1 / t`), `Faces::push` (`p = c - d`, `D(p) = M / r`) and
  `Faces::pole`; `deficit` is the convergent face, chronometry's well the
  divergent face; `observe tiers` prints both faces and `t`. Derived:
  `p = 2c - 1 = 1 - 2d`; the faces' turns `+360 c` and `-360 d` are one
  element of QQ/ZZ, differing by one whole turn, so a body reads winding
  `-1` and phase `360 c` (T3's shear, unchanged); the torsion share
  (axiom 7) is the divergent face about `H` at reach `H + J`, pole
  `1 + a / 2`. No reading changes value.
- Names (#151): phases in the rationals mod the integers are written
  `QQ/ZZ`; the throat's pole shares `s_Q`, `s_Z` (were `Q`, `Z`); the
  spin's grid `g` (was `N`); the founding's mass weight `m_Z = Z + Q` (was
  `w_Z`, which names the hold `3Z - u`).
- Reaches are read in tier units (cluster and galaxy units), never rings
  (#115); no formula changed.
- "Ring" means only the bell's harmonic (docs/model.md, Units); the
  units section's echo reads #112's sheared return.
- The ocean carries at the share its held medium holds (#113):
  `Substrate::held_within`, `medium`, `carry`; the bead's underived
  `inside` is removed. Seed 7: the waveform at the rim 7.80 to 10.88 per
  Gyr; galaxy 22's circulation 1893 to 1110 turns a Gyr.
- A throat's echo is its sheared return at its torsion share (T4, axiom
  7): `bell / (1 - s)` on the near face, `bell / s` on the far
  (`circuit::echo_at`), in place of a fixed two bells. The universal core's
  founding reads the founded share (`Substrate::echo`), every tier after
  its bounce the share of its released spin (`circuit::echo`). The core's
  release of its open kinds is named its bounce, not a pinch: a pinch is a
  fold meeting between gravity wells. `founding_shells` no longer returns
  the retired reach-held flag, and `observe founding` shows the closed
  kinds as the firmament; `observe universal-core` prints both faces'
  echoes (#60, #107).

- A bond keeps both atoms whole and is read by planes (`atoms::Bond`): each
  face's pole push and polarity on the axis plane, the pushes composed by
  the flow law, and every open winding of each face on the isoclinic planes
  with its tightness (`Atom::winding_tightness`), in place of one weaker
  tightness and a ratio. `compound::closing` returns the bond;
  `compound::held` is removed (`Bond::held`). The `bonding` reading's own
  copy of the winding tightness is removed. New reading
  `xylarium observe bond A B [T]` (#107).

- A seat's focus is four exact shares of free thought, one per domain, each
  in `[0, 1]` and together at most 1 (`knowledge::Focus`), in place of one
  domain or none; `Decision::Focus` and `Input::Focus` carry it, and the
  checkpoint is `xylarium checkpoint v5` (v3 and v4 are refused by name; the
  live homes were converted, focus none as four zero shares). In the
  uplink, a percept's axis coordinate is the exact count, and FOCUS takes
  four exact shares stated in the seat listing (#107, #108).

- The scalar heat of the atom, the two-part closure, the star cut and the
  star's sequence is the typed received turning `atoms::Received`
  (`axis`, `isoclinic`): the stripping and bond laws read the isoclinic
  planes, the pole state's climb the axis plane, and `stars::Sequence::of`
  reads both. `formation::hold_at_heat` is renamed `formation::hold_at`;
  `disc::StarHold::heat` is `StarHold::received` (#103).

- Chemistry from the elements restated as emergent (docs/model.md, roadmap
  item 42, #103): no state, species or substance named in advance; a bond is
  a paired throat read by its separation and holding remanence; the vents
  are received turning per plane, closure held as a superposition, states
  as the bond network's orbit, nuclides as attractor pairs, living matter on
  the synaptic chain, and no Earth unit in a law.

- Every turn in the documents names its axis and is stated in degrees, with
  its physics as torsion (modification, structure or perturbation) and its
  return read from the deficit at the return's reach (docs/model.md, "Turns:
  axis, degrees and the return"; AGENTS.md; glossary, Turn).

- The coordinate reads LAW-COORDINATE v1 (Mycelium 78.173): lanes carry
  the network's meanings (S spatial, G gauge, T temporal, H holonomic),
  values are signed and exact, sectors of base 60 at any depth,
  `Reading::within` reads Below, Within or Above and refuses two lanes by
  name; the law's fixtures 2, 3 and 4 are the module's tests;
  `coordinate::beat` writes the T lane. Removed `Reading::mark`, `places`,
  `arc`, the six-place cap and `coordinate::direction`, an in-plane reading
  with no caller.

- The founding reads the torsion share once (`circuit::Substrate::found`,
  which now takes only the spin and founds the elements itself) and holds it
  as `Substrate::share`; readers take that held value. The inbound echo's
  swing `1 + A_in` is one function, `Substrate::echo_swing`, and the
  founding cache key reads `package::founding_digest`. The heat a piece's
  gas and a star's disc take from the pulse is one form,
  `formation::pulse_heat`. The mass number `floor(a + 1/2)` is
  `circuit::mass_number`, and the harmonic count on S^11 is
  `fibres::directions` with one binomial. The `founding` and `formation`
  readings read the live founding shells and the live star cut
  (`disc::star_hold` through `galaxy::disc_frame`).

- One turning law per carrier: `turning::turning` (exact) and
  `turning::turning_dy` (readings, through `xyl_core::shares::deficit_dy`);
  the settling's pair rates and kicks and the moons' resonances read
  `turning_dy` per ring (ages through `circuit::rings_of_gyr_dy`); rates in
  rad/s are `package::Si::rad_s` of `turning::turning_si`; the core's mass
  in kg is `package::core_kg` everywhere; `turning::span_in_rings` reads
  `circuit::rings_of_gyr`.

- A star's life is read at its own reach about its core
  (`galaxy::Star::reach_rings`, the orbit radius) in every layer:
  `life::lifetime` and `worlds::star_degree_gyr` take that reach in galaxy
  rings, and `worlds::Setting` and `life::Home` carry the star's
  home-equivalent reach `r` (crowding) and its reach `reach` (life) in place
  of a distance in kpc; `worlds::cosmic_rays` takes galaxy rings.

- One reading per quantity in the system, body and world layers: the ice
  share of a body is `systems::World::ice_share` and the one icy test
  `World::icy` (envelope worlds were read icy by one reader and not by the
  other); a child's reach is `manifold::reach` (moon budget, moons); the
  eccentricity's relaxation is `bodies::eccentricity` (also for
  `manifold::Manifold::e`); the Love number is `bodies::love`; the share
  `2x / (1 + x)` is `laws::share2`; the crowding is `galaxy::crowding`; the
  chord on a sphere is `bounce::curved_reach` (`chain::Place::arc`); one
  `manifold::per_turn`; one `field::dyadic`; draws are `Dy::unit`; the
  settled age is `systems::settled_gyr`; the unit world's age of 4.57 Gyr is
  one declared ruler, `bodies::unit_age_gyr`; the forecast's cache is keyed
  by `minds::laws_digest`; `stars::pole_point` is renamed
  `stars::mixture_pole` (the mixture's pole state, not
  `circuit::pole_point`).

- One present: the universe's count of rings (`Universe::now`) is the
  clock; `Cosmic::place`, `Cosmic::distance`, `Cosmic::power`,
  `transit::separation_m`, `Network::passage_years` and `Network::arc_at`
  take the time as an argument, and Gyr is read through
  `circuit::gyr_of_rings`. The arc between two stars is one reading,
  `cosmic::arc_kpc`; light's crossing time is the pulse's speed
  (`transit::crossing_years`, read by percepts too); the `chemistry` reading
  reads the present instead of a fixed 13.8 Gyr; `tiers` reads the echo
  rings from the keepers (`Chronometry::of`).

- One housing law at waking and in the carry: a mind wakes where its world
  houses a viable people of its traits (`civ::housing::houses`), no longer
  where `life::foragers` counts one; a people's drive is `civ::drive` for
  the carry and for discord, whose reach is `transit::reach_kpc`; deficit
  shares read `xyl_core::shares`; the free share of housing is
  `civ::housing::free_share` for the consciousness gauge and free thought;
  the SI electronvolt and atomic mass unit are `package::Si`'s.

- `Checkpoint::decode` returns `Err` naming its refusal, and `run` refuses a
  home whose checkpoint does not decode instead of founding it afresh.

- `manifold::turn_between` and `chain::Place::arc` read the certified angle
  between two directions (`xyl_core::rotor::angle_between`) in place of an
  uncertified `Dy` arccosine; `Rotor::inverse` is the rotor's conjugate
  (#26).

- The binding to TUniversal is read per segment (#56, MU-CHRONOMETRY 3.3):
  `Chronometry::dilations` gives each segment's rings over its heights
  between consecutive anchors under one origin, never one ratio and never
  across a change of origin, in place of `Chronometry::pace`. A heard point
  carries its origin (genesis and epoch) and exact lanes; checkpoint and
  chain entries follow (entry hash v3).

- Readings take their time in rings (#29): `universal-core [R]` reads the
  substrate R rings after the bounce (default: the first stars) and writes
  it on the T lane, and `people N [R]` and `sync [K] [R]` carry R rings;
  Gyr is shown only as a display conversion.

- The universe's clock is anchored on TUniversal (#52): ring 0 is the final
  point heard with the origin receipt (`Chronometry::anchor`,
  `run::Link::origin`); each commit keeps the whole heard point, height,
  lanes and K (`chronometry::Point`, `run::Link::point` in place of
  `height`); the present is read on the T lane
  (`Universe::t_coordinate`). Checkpoint v4 holds the points and the anchor
  and still reads v3 homes whose commits heard no point (#53, #54).

- Observer readings renamed for what they read: `core` is
  `universal-core [T]`, and `cores` is `galactic-core N`, one galactic core
  per reading (the universe reading still lists them all).

- The jet's continuum dilation reads the throat's torsion share `s`,
  `D(s) = (1 + s) / (1 - s)` (`circuit::jet_dilation`), in place of the
  spinless push 1/2 (`echo_dilation`, deleted): the jet onsets
  (`circuit::onset`, `onset_echoes`) and the filament channel of
  `Substrate::loop_flux` follow the spin.

- `Units::of` and `Units::in_tier` refuse a zero denominator (`None`)
  instead of returning zero; a stellar hole's spin is drawn from
  `Stream::derive` instead of a BLAKE3 hash (#44).

- Roles: MAINTAINERS.md names the coordinator (mycelium's maintainer
  session) for contract review; CONTRIBUTING's workflow sends
  `cross-project` changes to it before merge.
- A home without a checkpoint is read at the first mind's dawn, where
  genesis writes its first checkpoint, in place of 14 Gyr (#3).
- The minimum supported Rust version is 1.85, the version the locked
  dependencies require.
- Code style: Modules over about 800 lines are split into directory modules
  (`observe`, `transit`, `cosmic`, `formation`, `galaxy`, `chem`, `worlds`,
  `systems`, `life`, `bodies`, `minds`) with public paths unchanged. Long
  functions are split into named steps, and lines are at most 100 columns.
  Private one-letter numeric wrappers are replaced by the shared
  constructors, and inline comments state reasons and formulas.
- `run::Options` derives `Clone` and `Debug`.
- The README is restructured. The roadmap opens with the mathematics:
  number systems, algebra and geometry, and the nucleon model (the 3Z span,
  the 3Q capacity and the polarity R).
- The Markdown documentation is edited to complete sentences in technical
  prose.
- The turning law is evaluated in core units, `omega = delta bell / r`
  radians per ring (`turning::turning`, `turning::turning_dy`), in place of
  `turning_kpc` and `turning_kpc_dy`; the SI forms remain as boundaries for
  the system and body layers.
- The galaxy's structure, star reaches and orbits are in galaxy rings and
  core masses (`galaxy::core_reach`, `galaxy::nucleus_reach`,
  `DiscFrame::reach`); kpc and suns appear at display and at the marked
  boundaries `galaxy::kpc_of_rings` and `galaxy::suns_of_core_masses`.
- The vertical orbit is derived from the turning law, the crowding and the
  ring structure (`galaxy::layer`, `galaxy::vertical_frequency`), in place
  of the declared 75.2 rad/Gyr, 20 km/s, 10 Gyr and 0.3 kpc.
- Star masses are in core masses: a piece's star mass is the share of its
  infall the star tier's cut holds (`galaxy::piece_star_mass`), and the
  initial mass function (`galaxy::imf`) draws shares of it by the deficit
  form with the ring's crowding as break, in place of the bounds 0.08 and 60
  suns and the break scale. `Star::mass`, `Star::companion`,
  `Star::stands_for` and the mass argument of `galaxy::disc_frame` are in
  core masses.
- A bead is a galaxy where its lit holding is at least one star's mass on
  the chain (`galaxy::first_star_mass`) and its core is no heavier than the
  universal core (`Substrate::holds_a_star`, `Substrate::galaxies`), in
  place of one sun. This refounds the test genesis once.
- Star light is the power the burn transduces, in core masses per ring
  (`stars::power`, `stars::light`); `systems::luminosity` is removed, and
  the system layer reads suns through the boundary `stars::suns_of_power`.
- Stars burn by polarity (`stars::Sequence`, `stars/sequence.rs`): each
  cloud's dark cut records its held mixture (`disc::StarHold::held`), and the
  star's sequence of fusion and fission steps emerges from that mixture at
  the cut's heat, each stage the held kind of greatest release, with the
  throat after `n* = ln(1 / t0) / ln rho_s` echoes (`stars::throat_echoes`).
  `stars::life`, `stars::ends`, `stars::burned` and `stars::remnant_share`
  take the star's sequence (`galaxy::Star::sequence`,
  `galaxy::piece_sequence`, `galaxy::first_star_sequence`);
  `field::Burning` is `Unlit`, `Step(i)` or `Remnant`; `field::burning` and
  `field::burn_out_share` read the sequence, and `field::absorb` takes the
  step's burn-out share. `galaxy::Class::name` names held bodies that do not
  ignite, fission stars and giants from the sequence. `life::lifetime`,
  `life::first_age` and `worlds::star_degree_gyr` take the star's sequence,
  carried in `life::Home::sequence` and `worlds::Setting::sequence`. A held
  body that does not ignite forms no remnant node (`pulse::nodes_among`).

- One certified exponential: `motion::Boost::grown` splits the rapidity
  into whole halvings (an exact power of two) and a remainder in
  `[0, 2 ln 2)` that `certified::exp_of` encloses; `certified::exp_fixed`
  and motion's own `ln 2` and `pow2` are removed, and `certified` builds
  its dyadics with the one gcd-free constructor of `q`.

- `Dy::PI` is `Dy::pi()`, and `Dy`'s ln 2 is read the same way: each is
  the certified enclosure (`certified::pi`, `certified::ln2`) rounded down
  to 62 bits once, in place of a hard-coded literal.

### Removed

- The out-of-tree derivation screens (Python and a dump crate) are retired.
  Their readings come from the binary: `xylarium observe universal-core`
  now reads the pair's holds on both faces (T16), the core's remanence
  (T19, `circuit::SpinOrbit::remanence`) and the axis-polarity census (T7).
  The model.md readings cite it; readings that need the pinches built are
  marked as retired screens to be read on the binary (#79).

- The founding's admission: `Substrate::least_share`, `far_reach`,
  `margin`, `Anchor::margin` and the refounding at a raised spin in
  `cores::substrate_for`. The founding is made once at the drawn spin, and
  whether its core closes is read on the orbit.

- Parallel founding and formation models: the dead star cut
  `formation::of` with `Formation` and `Piece`, the dead disc model
  `disc::star_disc` with `disc::World` and the `formation` reading's scratch
  infalls, and the diagnostic cascades `formation::cascade`,
  `formation::equitable_cascade`, `Stack` and `EquitableReading`; the
  stored copy `Anchor::share` and `Bounce::share`; `atoms::orientations`,
  `atoms::closes` (replaced by `compound::closing`) and the hand-rolled gcd.
  Forward-only wrappers: `formation::throat_push`, `formation::weight`,
  `origin::spin_of_share`, `centre::galaxy_episode`, `circuit::dilation`,
  `circuit::jet_dilation` and `chem::Rulers::r0`.

- Parallel copies replaced by the one model above: `turning::turning_rad_s`,
  `turning::rad_s`, `turning::rate_per_ring` and the private rings-per-Gyr
  conversion; `stars::reach_rings`; `chain::kpc`; `chain::Place::arc_kpc_dy`
  (read `arc_kpc`); `bodies::Orbit::at` and `bodies::Now` (test only; the
  live orbit at an age is `worlds::Path`); the private `love_dy`,
  `mean_motion`, `surface_turning`, `star_length_m`, `held`, `unit`,
  `inv_two_pi`, `crowding_share` and `share2` copies; the forwarders
  `manifold::turn_between`, `field::bend` and `life::first_age`;
  `life::settled_gyr` and `minds::life_digest`.

- `universe::Bell` (its Gyr period rounded to 2^-60) and `Cosmic::now`:
  the second clock. `Network::arc`, `growth::world_w`,
  `growth::unit_metabolism_w`, `knowledge::unit_body_w` and
  `knowledge::knower_life_yr` (forwarders and repeats), the kernel's
  `propulsion::ev_j` and `propulsion::amu_kg`, the private surface area
  (`Body::area_m2` is read), and the v3 checkpoint reader (a v3 checkpoint
  is refused by name).

- The founding's rounding of rationals: `circuit::turn` (capture phases
  floored to the coordinate grid) and `circuit::grain::grid` (reaches,
  cooling's lambda, agitation, emission, mu and heat, the disc's reach and
  the firmament's spin floored to 2^-64). Each is now exact; a capture
  phase's denominator divides the torsion share's.

- `systems::survival`, which always returned 1: every world was kept
  regardless of stellar encounters. The binding share against passing stars
  is to be derived (#44). `circuit::vacuum_share`, `turning::deficit_share`
  and the private `grain::deficit`, replaced by `xyl_core::shares`.

- The fixed hydrogen-helium-carbon chain: `stars::transduced_hydrogen`,
  `stars::transduced_helium` and the measured binding energies of
  `field::burn_out_share` (`binding_mev`, `nucleon_mev`, `weak_mev`).
- Hosted CI (`.github/workflows/ci.yml`) and its README badge are removed;
  the checks are run locally.

- `xyl_core::orbit`, which no production code read; `Orbit::throat`
  re-derived the torsion share that `bounce::torsion_share` holds. The
  sheared return `k / (1 - theta)` stays a theorem of docs/model.md (T4).

- Unread readings of xyl-core: `rotor::Half::share` (the S^2 case of
  `measure::share_turns`), `shares::opposed` (`flow(x, -y)`) and
  `Dy::cosh`; no production code called them.

- `Rotor::matrix`: a rotor turns a vector by its own action,
  `v + (w t + u x t) / n` with `t = 2 u x v` (`Rotor::apply`), and
  `field::Bell::of` reads a stellar hole's axis as the frame's image of
  the z axis through it.

### Fixed

- Merged embryos' phases meet along the shorter arc between them,
  weighted by mass (`systems` settling); they were averaged as numbers, so
  0.95 and 0.05 met at 0.5.

## 0.3.0 - 2026-10-30

First public release.

Every layer, from the founding to the kernel, now reads the universal
core's units (`G = c = M_core = 1`), and worlds form on the manifold chain.

### Added

- `xyl_cosmos::package` (`package.rs`): what a universe is founded from that
  its laws do not derive - the universal core's mass (`hole_mass`), the SI
  seam (`Si`) and the founding digest (`founding_digest`) - moved from
  `galaxy.rs` and `bodies.rs`.
- `xyl_cosmos::disc` (`disc.rs`): a star's cut of its piece's infall
  (`star_cut`) and the worlds on its manifold (`manifold_worlds`), moved
  from `formation.rs`. It reads the founding and is not part of the
  founding digest.
- The founding digest (`XYL_FOUNDING_DIGEST`, `crates/xyl-cosmos/build.rs`):
  a digest of the founding's own files only (the Big Bang, the bounce, the
  elements and atoms, the cascade and lighting, the circuit, the package and
  the mathematics). The founding cache is keyed by it and the torsion share,
  so a change to a layer built on the founding keeps the cached founding.
- The genesis record v2 (`xylarium genesis`): it names the founding digest
  and the package, and carries the founding's census kind by kind
  (`Substrate::census`), with the held share, beads, galaxies, lit share and
  stellar mass.
- One test genesis (`xyl_cosmos::genesis::universe`): seed 7 founded once
  into one home under `target/genesis`; every test reads it.
- `chem::Chemistry::birth_along` and `Chemistry::cached`.
- `Anchor::galaxy_kpc`: the galaxy tier's ring in kiloparsecs.
- The worlds reading counts the stars without worlds by cause.
- Colonies: a sailing people's reach (open worlds of its system) and
  daughter peoples founded by settlers; an exodus goes to an unoccupied
  world.
- History: turnings from exodus, settlement, joining the network,
  quarantine, mastery and folding name eminent figures whose memory fades at
  the people's forgetting rate; held in checkpoints.
- Realms and stewards: realms form from ley-connected constellations, each
  with a steward seat that sets line handling (open, tolled, guarded,
  closed); passage reads handling for the passer.
- Space folding: peoples without lawful passage fold space when a passage
  is worth it at their own drive; folded energy is a breach share that
  raises discord toward quarantine.
- Exoduses, settlements and steward decisions are recorded inputs, applied again
  on restart.

### Changed

- Worlds form on the star's manifold (`disc::manifold_worlds`): the star's
  manifold is the arc its pull holds about the core, the star holds its
  gathering, and a world forms where its gathered matter's deficit share
  holds against the core's twist, read at every degree. This replaces the
  disc partition `formation::disc`.
- The galaxy's stellar mass is the lit share of its founding
  (`galaxy::stellar_mass`: `Lighting::lit` times the core's mass), in place
  of the galaxy holding what its core holds.
- Every world anchors its own manifold, and its moons gather there by the
  same law one tier down, against the star's tide at the world. The fixed
  moon budget (1e-4 of a giant) and moons for giants only are gone.
- A star is born along its ring's own star formation from first light
  (`chem::Chemistry::birth_along`), so stars carry their ring's metals at
  birth; previously every star was born at its ring's first light.
- A certified exponential below the resolution is enclosed by
  `[0, 2^-bits]` for `x <= -(bits + 1)` (`systems.rs`), exact and immediate.
- Settling counts each mass once: a pair that meets by the settled age
  merges; a pair that meets later stands as two bodies, with the meeting
  recorded on the lighter.
- The galaxy structure and the founding grain are cached only once the
  universe is founded.
- One reach per tier: every reach inside the home galaxy is read in galaxy
  rings (`stars::reach_rings`, `Star::reach_rings`); a star's reach is its
  own and is no longer summed with its galaxy's reach about the universal
  core.
- The ring of time is one full bell, the pulse's round trip from the throat
  to the rim and back; an echo is two rings.
- The rim is the outermost founding shell's outer edge,
  `reach x (1 + s_final)`, and the held share has one definition, by mass
  (`Substrate::held_share`), which `BigBang::of` reads.
- The element table extends to `Z = 1700` (`elements::TOP_Z`). The vessel's
  upper edge is the last seal of S^11 within the table (Z 1092). The
  founding's hydrogen count is derived as the fewest bits that found the
  sealing kind whole (`elements::founding_bits`), no longer a package input.
- The bell is derived from the founding: light's round trip to the home
  galaxy's rim, `2 rim_galaxy / c` (`circuit::anchor`), replacing two 15 kpc
  disc crossings. The pulse rides the filament across the universe within
  the same bell (3600 c, the ladder step between the galaxy and cluster
  tiers). The universe's curvature radius is `rim / pi`, the home galaxy's
  rim is the founding's rim read in the galaxy tier's unit, and galaxies sit
  at their founded reach.
- The substrate's push is the vacuum's share, `H / (R + H)` read at the
  reach pushed (`circuit::vacuum_share`), one half only at the throat
  (`circuit::throat_push`). The inbound echo (`Substrate::inbound`) swings
  every push as `1 + A_in(t)` (`Substrate::push_at`) and heats the home
  galaxy's gas.
- The home galaxy's gas is the founding's beads one tier down
  (`formation::lighting`): the disc's pieces on a circle and the bulge's in
  a ball (`measure::share_turns`); an infall gathers its own angle of a
  piece and grips it at its own rim, `m / (w + m)`; the pulse's heat is the
  ring the bell brings at the piece; a piece that never lights forms no
  stars, and no substitute is used.
- A star's cut of its piece's infall is read on the chain
  (`galaxy::disc_frame`, `disc::star_cut`): the disc's agitation is the
  pulse's heat on the disc, and the core-star tie is the galaxy's tide at
  the pinch. The Sun's disc mass, the 360 parts in `ln a`, `Spec::limits`
  and the stirred inner belt are gone.
- The jetstream reading: `Substrate::loop_flux` composes the filament's
  dilation, the vacuum's share and the electrons' phase state as dilators on
  each firmament kind's pole state (`circuit::PoleFlux`) and reads the
  residual.
- The parent's spin is admitted only where its torsion share holds the
  firmament whole against the pulse (`Substrate::least_share`,
  `bang::spin_of_share`); the genesis record states the margin.
- The Big Bang's only random draw is the parent's spin. The lumpiness, the held
  share, the assembly and the star efficiency are read off the cascade; the
  chemistry is calibrated on the universe's own history
  (`chem::Calibration::of`); the Big Bang `Kind`s and the `--kind` option are
  gone; the home defaults to `var/universe-7`.
- Stars are fuelled by the pulse (`stars`): the burn per bell is the ring
  the bell brings at the star's reach times its own deficit at its rim; its
  light is that burn; its life and its giant's end are its hydrogen and
  helium steps' holds over the burn (`stars::life`, `stars::ends`); every
  ended star's remnant is a bell of the network (`stars::remnant_share`).
- Radiation and temperature are distinct readings: a world's flux is the
  bell's mean of its star's burn, the swing within a bell is the light's own
  reading (`stars::flicker`), and temperatures are the media's agitation.
- The forecast of minds uses expected values in closed form: rungs take
  their means, impacts stretch them to `(e^(mu m) - 1) / mu`, and the
  sterilising pulse falls at its hazard's expected time; the forecast's
  default sight is the census of the sample.
- Planetary settling is carried as ages: each pair's merge age is the first
  age its swing spans the gap (`World::merged_at`); starting eccentricities,
  tilts, nodes and periapses come from the settle's own swing and phase.
- The jet's onset per kind is the filament's dilation count to the throat
  (`circuit::onset_gyr`); the landing shares each kind by pull times phase
  coherence (`Substrate::landing`); the horizon's source carries the five
  isoclinic planes' spin (`bounce::spin_bivector`); the galactic jet's
  episode is the galaxy's own echo (`centre::galaxy_episode_myr`).
- The home galaxy's structure is read from the founding one tier down
  (`galaxy::Structure`); the Milky Way disc, bulge and nucleus rulers and the
  26,000-star sample are gone.
- The bounce's share is derived from the parent's spin by torsion,
  `a = (H + J) / (2H + J)` (`bang::torsion_share`), so every seed founds its
  own capture chain, rim and bell.
- The bounce's horizon is read in its S^360 enclosure
  (`bounce::curved_reach`) at the deficit share `delta_h = 1/3`
  (`bounce::horizon_reach`), and the sphere is built through the crossing
  point (`cga::sphere_through`). The founding precedes the Big Bang, is
  founded once per process (`cores::substrate_for`) and is cached in the
  home directory as `founding-*`.
- Civilization layer consolidated in `civ`: one housing law
  (`civ::housing`), numbers as the logistic toward housing scaled by the
  radiation environment, history (`civ::history`), settlers when crowded
  past the logistic's inflection.
- Knowledge is solved in closed form (`knowledge::advance`), replacing
  backward-Euler steps.
- The unit body's generation is a named ruler,
  `growth::Rulers::generation_yr` (14.2 yr).
- Turnings carry one lowercase act set: `exodus`, `settlement`, `network`,
  `quarantine`, `biology`, `information`, `fold`.
- Whether space can be folded is a law of the universe: `Scales::ftl` carries
  `Laws::ftl`, and `Network::fold` reads it from the galaxy's scales.
- The lexicon's solvent ids are derived from `worlds::Species::ALL` plus the
  melt and the plasma (`lexicon::solvents`).
- `civ::housing::housing` takes no appetite.
- Seats: one sovereign seat per people, and an ancient and a progenitor
  seat per ancient people.
- The firmament carries the universal state: the throat's held kinds, each
  in its founded proportion, are scaled so their mass is 1
  (`circuit::Substrate::found`).
- A galactic core's mass is `hole_mass x state`, its bead's state read in
  the universal core's units (`cores::founding_of`), and
  `Substrate::holds_a_star` reads the state directly; previously both divided
  by the mean bead state.

### Removed

- Tests that pinned measured values (the Sun's, water's, Titan's and the
  Earth's) or the model's own output; tests now guard laws: books that
  close, exact identities, orderings and monotonicities.
- The Planck draw centres and the Big Bang `Kind`s; the Chiappini infall
  fit (`galaxy::infall_fit`); the stellar rulers `10 Gyr x M / L`, `L = M`,
  the faint-young-Sun slope and the 20-sun hole cut; the main-sequence
  brightening in `worlds`; the chemistry's reference history and its
  parameters.
- The disc partition `formation::disc`, replaced by
  `disc::manifold_worlds`.
- The trade-route optimiser in `transit` (`Offer`, `Market`, `Leg`, `Route`,
  `circuits`, `departures`, `spreads`, `supplies`) and the toll law's
  `around`; the fuel-choice machinery in `propulsion`; `Knowledge::learn`,
  `transit::binding`, `Cosmic::from_core`, `Chronometry::rim_heads`,
  `Seats::sovereigns` and the measure's radial shares, all uncalled.
- The 15 kpc disc edge as the bell's length, the Hubble length and the
  closure draw, the 0.3 Gyr first stars, the core's drawn spin
  (`hole_spin`), and the `XYL_FOUNDING_BITS` environment input.
- Earlier civilization machinery: human life stages and disease
  (`lifespan`), demographic transition and pronatal policy (`fertility`),
  four seasons and food reserves (`sustenance`), the policy fork
  (`demography`), central and federal banks, markets and backed currencies
  (`banks`, `exchange`, `backing`, `decide`), sedimentation and mining depth
  (`archaeology`), and captains' offices.
- Measured human data in knowledge; the r and K fertility ceilings (a
  people's natural rate is one e-fold per its own generation).
- Carbon as every chemistry's backbone (monomers are built from the
  solvent's own molecules) and fixed extinction rates (strikes come from
  each system's own debris).
- `primordial.rs` and `crust.rs`.
- `seats::council_size` and `seats::On::Ship`.

## 0.2.0 - 2026-10-28

Initial release.

### Added

- Deterministic universe generation from a seed: a bounce cosmology, element
  synthesis by successive capture, the founding cascade into a held
  firmament and galaxy-forming beads.
- The substrate: the core, the inter-galactic medium and each galaxy
  expressed as closed-form functions of time, with outflow, intake, force
  (drift and swing) and tide.
- Galaxies distributed over the universe's sphere and circulating about the
  throat; stars on double-cone manifolds; places as bearings and distances as
  arcs.
- Planetary systems formed from the matter available, nested manifolds for
  planets and moons, and world state read per degree of the star's life.
- Life and the forecast of minds; peoples created where their world feeds a
  viable population.
- One civilization carry for live play and readings: knowledge, the fall or
  exodus of a people, food, fertility, lifespan and gauges, with housing from
  natural production and energy-driven synthesis.
- Propulsion, constellations, ley lines sited by discovery and bounded
  optimisation with passage energy derived from the builder's drive, and
  quarantine by discord.
- Seats for external agents, applied as input events.
- Economy libraries (banks, markets, backed currencies, the decision rule),
  not yet integrated into the carry.
- `xyl_core::cga`, a conformal geometric algebra over exact rationals.
- `xylarium` binary with `genesis`, `run` and `observe`, and the
  `xylarium::run::Link` hook for network uplinks.
- Documentation: README, architecture, model, civilizations, network
  protocol, glossary, development guide and roadmap.
- Continuous integration: formatting, clippy with warnings denied, tests,
  documentation with warnings denied, and line coverage with
  `cargo llvm-cov`.
