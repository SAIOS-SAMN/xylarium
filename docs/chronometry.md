# Chronometry

The telemetry of time: every quantity of time the engine holds or reads, in
one library, read in one place. Time is a coordinate (E032, ruled
2026-11-15): an event's time is its point's coordinate on the T lane, a
duration the separation of two points' coordinates, and every clock is a
reading of an orbit. The build-out is tracked on
[#299](https://github.com/SAIOS-SAMN/xylarium/issues/299).

## Why one library

Built so far (#300): the pure forms live in `xyl_core::chrono` (the front's
arrival, the well's rate, the sheared return, the breath and its turns, the
winding, the keeper's reading), every listed site moved onto them and its
copy deleted; the epicenter keeps time in root windings, a ring of one, so
no stored ring holds pi. The ring as a versor, the telemetry and the pace
are next (#301, #302, #303).

The quantities of time exist today, but in six places with six
conventions:

| Quantity | Where it is today |
|---|---|
| The T lane, the beat (whole rings, phase in degrees) | `xyl_core::coordinate` |
| The bell (one ring), the echo `bell / (1 - s)` and its far face | `circuit::bell`, `circuit::echo_at` |
| The front's arrival `tau(r) - tau(0)` | `Substrate::delay`, `field::path`, `xyl_core::chrono::arrival` |
| The breath `1 + delta cos(360 t / echo)` and the breathing orbit's turns | `chain::breath`, `xyl_core::chrono::breathing_turns`, `Manifold::stand` |
| The keepers: rate `r / (r + M(<r))`, delay, slip, coherence, rung, their chain | `xyl_kernel::chronometry` |
| The final points heard from the network | the uplink's clock, `Checkpoint::commits` |

Each reads time by a convention of the site: some take a global count, some an age, some a
reach. A law that needs two of them converts between conventions inline,
and two units of time have already been confused at a call site (a sphere
read at a time in Gyr, #260). The epicenter's ring stores pi as a rational
(`pi(64).mid()` in `Chronometry::of`), which the model allows only for
display. One library removes all three faults: one form per quantity, one
calibre per reading, and pi only at display.

## No metronome: windings and the T lane

There is no background clock. A space winds the cycles inside it, its windings
`K` on each turning plane, while the T lane stands; T is written only when
a state crosses a throat (`T -> T - 2 (H - R)`, `coordinate::PolePlane`) or
a closure is verified (TUniversal's beat, which stood still on the mesh
while no verified work was presented). So:

- the engine's count, one ring per loop of live play, is the root orbit's
  winding, not a global time; every other clock is the winding of the orbit
  it keeps, related to the root's by the ratio of angular rates;
- the T lane advances by crossings and closures, each writing its flux;
- the library and its telemetry read the two apart, per subject: its
  windings per plane, and its T coordinate.

## The quantities

Each is a closed form, exact in `Q` (or `Dy` on a grid), with its calibre
named, and read per lane and per plane, never as one scalar.

- **Coordinate.** A subject's point `origin / h : S : G : T : H : K`
  (FORMS-COORDINATE v4); its time is the T lane, held in the tier it stands in.
- **Separation.** `delta tau` between two points: the front's arrival,
  `tau(r) = r + H(<r)` about the source, reach and hold in the tier's one
  calibre, so `dtau / dr = 1 + dH / dr`. Separations add and reverse, and
  across a stretch holding nothing `delta tau = delta r` (E032).
- **Rate.** A clock's rate against the root: the well's divergent face
  `r / (r + M(<r)) = r / tau(r)`, nested wells multiplying.
- **Winding.** A subject's whole turns per plane, `K = floor(abs(theta) /
  360)`, read from its composite rotor; it advances while T stands.
- **Crossing.** A state through a throat writes `-2 (H - R)` on the T lane
  and carries every lane's change (S, G, T, the H lane's composite rotor,
  K) under a matching origin stack; out and back through one throat
  cancels, a path through several pivots does not.
- **Ring.** One traversal of an orbit, the multi-axial versor of the ring
  (model.md, The ring): the pull `D(delta)` and the twist `D(n)` on the
  axis by their exponents, each isoclinic plane's winding and phase,
  `t (delta_p + w_p sigma_p) mod 1`, and the returns. Rings nest along the
  anchor chain, two levels related by the ratio of their angular rates (pi
  cancels), so every ring count is rational (The ring hierarchy). It is
  held as the composite versor, the order of its motions kept: per-plane
  phases and windings are its readings (model.md, The ring as held).
- **Echo.** A throat's sheared return: the near face after `bell / (1 - s)`,
  the far face after `bell / s` (T4); equal only without spin.
- **Breath.** The bell's cycle on a manifold: the hold swinging
  `H (1 + delta cos(360 t / echo))`, an orbit's rate as `delta (1 - delta)`,
  its turns in closed form; over whole echoes the mean.
- **Keeping.** A keeper's reading at the root's count `N`: its local count
  `rate (N - delay) / ring`, its slip, its coherence
  `1 - 2 |slip - round(slip)|` and its rungs of 360 degrees.
- **Pace.** The engine's rings against the network's final points: rings
  per final point, read per segment, never fixed; the T lane's advance per
  final point (the package's delivery `d_T`, E031).
- **Display.** Seconds, years, Gyr, and any duration with pi in it, read
  through a certified enclosure at the boundary (`package::Si`), never
  stored and never entering a law.

## The layers

The library follows the crates' layering, each layer holding only what it
can derive.

| Layer | Holds | Reads |
|---|---|---|
| `xyl_core::chrono` | The pure forms: the separation, the rate, the ring reading and its nesting ratio, the sheared return, the breath and its turns, the keeper's reading, the display boundary | numbers only |
| `xyl-cosmos` | The holds and shares the forms read: `Substrate::held_within`, `galaxy::inside_mass`, each manifold's anchor; each throat's torsion share and echo; the bells' arrivals | the founding |
| `xyl-kernel` | The keepers and their chain, the checkpoint's cut (docs/checkpoints.md), the universe's clock, every input's coordinate | the universe in play |
| the uplink | The final points heard, the pace, the seam's `delta tau` | the network |

Every existing site moves onto the library in place: the old function is
deleted in the same change, never kept beside the new one.

## The telemetry

`xylarium observe clock` becomes the telemetry of time, read on a live home
at its present (`observe clock SUBJECT` for one subject):

- **Per subject** (the epicenter, each galaxy's hub and shells, each star in
  play, each world of a people): its tier; its T coordinate; its separation
  from the root and from its anchor; its rate; its windings `K` per plane
  and its ring reading (the composite versor's planes and volumes, the
  axis's exponents, each plane's phase, the returns); its echo, near
  and far; the breath's phase; its slip, coherence and rungs against the
  root.
- **The chain**: each keeper's head and the entry it cites inside it.
- **The network**: the last final point heard, the T lane's advance since
  the last segment, the pace in rings per final point, and the seam's
  `delta tau` with netstratum.
- **The checks**, each a residue read live, like `observe proofs`:
  separations add, every rate lies in `(0, 1]`, ring counts across the
  hierarchy are rational, each breath returns to its mean over whole
  echoes, and no stored time holds pi.

## Order of building

The sub-issues of #299, each with its done-when:

1. Built (#300): the library's pure forms in `xyl_core::chrono`, the existing sites moved
   onto it and their copies deleted; pi out of every stored ring.
2. The ring reading (#60): the versor per body and its nested reading, read
   by the telemetry; with #128 for each plane's own share.
3. The telemetry: `observe clock` as above, every check a residue.
4. The pace: the network's final points against the rings, per segment.
