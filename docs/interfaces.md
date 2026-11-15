# Interfaces

Two components of the model meet only through a transduction (axiom 13,
[model.md](model.md#axioms-and-theorems-of-the-founding)): a lift into the
receiving manifold, an act on it, and a projection out. This file lists every
interface between components, so that the model can be wound forward
(deriving each tier from the one below it) and unwound backward (taking a
reading apart into the readings it came from). The interlock and its plan are
tracked on [#245](https://github.com/SAIOS-SAMN/xylarium/issues/245).

Each row states:

- the move, by kind (lift, act, project, or a composition of them);
- what it carries, from which manifold and lane to which;
- the calibre it reads in;
- the invariant it keeps;
- how it unwinds: its inverse, or the residue left where it does not;
- the code site, and the equation or theorem that states it
  ([EQUATIONS.md](EQUATIONS.md)).

A row whose law is not yet aligned names its open invalidation (an I-row). A
new interface enters this file in the same change that builds it.

## The founding and the carrier

| Interface | Move | From → to | Calibre | Invariant | Unwinds by | Site | Law |
|---|---|---|---|---|---|---|---|
| Capture | lift | a kind `Z` at the torsion share `s` → each Hopf plane's capture phase `frac(w_p s Z)` | none (turns) | in the common winding the six planes read one phase | not inverted: the whole turns `floor(w_p s Z)` are the residue | `circuit::fill_shares` | E016 |
| Induced turn | act | a throat's torsion → every plane's phase, `+w_p 360 s` (spin face) or `-w_p 360 (1 - s)` (inverse face) | none (degrees) | each face is the weighted Seifert automorphism; it commutes with the Hopf action | each face by its opposite turn; the pair leaves the shear `w_p 360 (2s - 1)` | `hopf::induce` | E029, T22 |
| Reduction | project | a turn on the six planes → its reading on `CP^5` (the common phase taken out) | none | the point of `CP^5` under a common turn | not inverted: the common phase is the residue | `hopf::induce` (the common winding) | T23 |
| Electron phase | project | a plane's phase → `R_p = kappa (1 - 2 frac(psi_p))` in `[-1, 1]` turns | none (turns) | the sign is the face: + on Z, - on Q | not inverted: the hemisphere folds two phases onto one `R` | `circuit::electron_phase` | E016, E030 |
| Rest point | project | `R` (H, a turning) → the nucleon split `Q* / Z` (the join's share) | none | open | open | `formation::shells` (`rest_point`) | E018; open as I017 |
| Pooled release | act | a cohort's ionized pool spin `S` (S, a boost) → `S' = S (+) (-r)` past the charged medium | founding units | the books close: `sum m l + take = sum m p` | by the take, booked to the medium's charged share | `formation::shells` (`pooled`) | netstratum docs/134 |

## Time and motion

| Interface | Move | From → to | Calibre | Invariant | Unwinds by | Site | Law |
|---|---|---|---|---|---|---|---|
| Breath | act | a core's bell (its echo) → the enclosure's radius and the orbits about it | galaxy units | each returns after a whole echo; the swing is the rim's deficit, the orbit's `delta (1 - delta)` | over whole echoes to the mean | `chain::radius`, `chain::galaxy_radius`, `chain::breathing_turns` | #233 items 1, 5 |
| Throat push | act | the inbound echo's swing `1 + A_in` → the push at the throat | none | one half at the echo's mean | over an echo to the mean | `circuit::throat_push` | #233 item 3; S114, S116 |
| Reverberation | act | the ring's phase → a band past the throat on the arc about the crest | founding units | its share of each echo is `acos(c) / 180` | over whole echoes to the share | `circuit::Substrate::outflow` | #233 item 4 |

## Bodies, fields and coupling

| Interface | Move | From → to | Calibre | Invariant | Unwinds by | Site | Law |
|---|---|---|---|---|---|---|---|
| Field | act | a bell's pulse → a world's intercepted energy and its turn from its native direction | AU, kpc (one per share) | the pulse is one breath, out and back, with no drift | the re-lock work pays the turn; the unpaid shear is kept | `field::Bell::energy_at`, `field::absorb` | the field law, first-degree |
| Star transduction | act | the irradiation a star receives → the light it makes per bell | per bell at the reach | the star holds its compactness's share | not inverted: the passed share goes on | `stars::transduced_per_bell` | the radiation law |
| Coupling | act | a range's dipoles → the pull on a body's rate (a boost, by the flow law) | rates of the luminal bound | the rate stays inside `(-1, 1)` | by the opposite push | `xyl_core::coupling` | FORMS-COUPLING v1 |

## The network

| Interface | Move | From → to | Calibre | Invariant | Unwinds by | Site | Law |
|---|---|---|---|---|---|---|---|
| Passage | act | a throat state on a pole plane → the state past the swap, its span and its flux on T | none (counts of one throat) | the whole `H + R` | a second pass is the identity; the flux cancels on return | `coordinate::PolePlane`, `coordinate::Coordinate` (`pass`) | FORMS-PASSAGE v4 |
| Coordinate | lift | a body's state → its place `origin / h : S:G:T:H : K` | exact, per lane | the canonical order; `K` read per H component | by the coordinate's own lanes | `xyl_core::coordinate` | FORMS-COORDINATE v4 |
| Manifest | project | a universe's matter `(Z, Q, R, count)` → its delivery on TUniversal's T lane | none (counts and shares) | the share delivered by final point `h` is `d_T(h) = 1 - T_0 / T(h)` | not inverted: delivery only rises | the uplink's manifest (Mycelium 78.184) | E031 |
