# The board: a blank universe to deploy and edit

Planned ([#162](https://github.com/SAIOS-SAMN/xylarium/issues/162), M5 Physics as a kit). This page scopes and
designs the board. The model it runs is the one in [model.md](model.md),
the formulas are those in [EQUATIONS.md](EQUATIONS.md), and the order of
the work is listed in [roadmap.md](roadmap.md).

## Why

A universe is founded today from its seed, top down: the origin, the
bounce, the capture chain, the founding shells, the core's spin orbit,
the galaxy, its stars and their worlds (`run::with`). Every law is
compiled code, and the founding's cache is keyed by a digest of that
code (`package::founding_digest`). A change to one substrate law, such as
the closed share of the capture chain (`elements::closed_share`), is read
only after a rebuild and a new founding of every tier above it: minutes
per reading, and one universe per process (`circuit::anchor`, #142).

The board is the other direction. It starts empty: an enclosure with no
matter in it. A developer, a test or an agent on the hypha deploys cores,
stars and worlds on it, adds or removes nuclei of each kind, and reads
what the laws make of them, in seconds and without a founding. It is a
universe on the S^360, not a chart: every body is a sphere in the
conformal model, placed about its anchor by a reach and a direction read
plane by plane in degrees, never a point on a square.

## What the board is and is not

- **The same laws.** A body on the board is read by the functions the
  founded universe uses: a star's pole state and sequence
  (`stars::mixture_pole`, `stars::Sequence`), its timeline, power and
  light (`stars::timeline`, `stars::power`, `stars::light`), the faces
  of gravity (`shares::faces`), the turning law (`turning`) and the
  manifolds (`manifold::manifolds`). The board adds no law and no
  parallel version of one.
- **Declared starts, emergent rest.** On the board a placement is a
  declared input, like the package's rulers: a core of a mass and spin, a
  star of a mass and a mixture. What follows (the star's sequence, its
  light, the hold inside each reach, a world's manifold) is read by the
  laws, so concentration stays a reading. Placing matter is the board's
  purpose and does not enter the founded universe, where matter is never
  placed.
- **Not the played universe.** The game's universe is still founded from
  its seed. The board is the bench beside it, where laws are tried,
  readings are compared and agents learn the physics. A board can be
  promoted to nothing; its findings move into the laws through the usual
  review.
- **Exact.** Values are `Q`; transcendentals are `Encl` and readings
  `Dy`, gridded back to `Q` before they are stored. Out-of-domain edits
  are refused with the reason, never clamped.

## The enclosure

A board's home holds a board record in place of a genesis record:

- **The package.** The laws and rulers the board runs under: the
  package's declared values today (`package::hole_mass`, `package::Si`),
  and each law's selected form once laws are selectable (#140).
- **The enclosure.** The S^360 that holds the board, of curvature radius
  `rho`, as the founded universe reads its reach along the enclosure
  that contains the core (`bounce::curved_reach`). Lengths are in `r_g`
  of the board's first core, and each tier is a power of sixty
  (`units::Tier`).
- **The ledger.** Every edit, in order, each at its count on the T lane
  (`coordinate::beat`): whole rings and the phase within the ring in
  degrees. The board's state is the ledger read by the laws; nothing
  else is stored except caches keyed by the ledger's height.

## Bodies: spheres on the enclosure

Every body is a dual sphere of the conformal model (`cga::sphere_through`)
about a point placed by its anchor, so the board is a tree of anchors,
core to star to world to moon, the chain the founded universe already
reads (`circuit::anchor`, the anchor chain in [model.md](model.md)).

- **Place.** A body is placed by its anchor, its reach `r` in the
  anchor's tier and its direction on the enclosure: a weight on the six
  planes (the axis plane and the five isoclinic planes, axiom 8), each
  turn in degrees with its return. Two bodies are related through the
  pivot, plane by plane, never by a distance alone
  ([model.md](model.md#pairs-the-pivot-per-plane)).
- **Kinds of body.**
  - A core: its mass `M` and spin `a`. Its horizon `2M`, torsion share,
    faces `c = M / (r + M)` and `d = r / (r + M)`, and bounces are read
    from them.
  - A star: its mass and its held mixture, the kinds it holds in whole
    nuclei. Its pole state, sequence, timeline, power and light are
    read.
  - A world: its mass and mixture about a star; its manifold (rate,
    plane, twist per turn) is read from the turning law.
  - A cloud: a mixture with no body yet, the board's medium; the hold
    inside a reach counts it.
- **Time.** Nothing is stepped. A body's state at a count is a closed
  form of the count since it was placed; changes between edits are
  events found by `event::crossing`.

## Elements on a body

A body's mixture is a list of kinds and whole nuclei. An edit adds or
removes `n` nuclei of kind `z` (an atom of the table or a cluster), and
is refused where the kind is outside the table or the count would go
below zero. After each edit the readings that depend on that body are
read again, and only those: the star's pole state `t0`, its sequence of
fusion and fission steps, its transduction and light, and the hold that
the bodies about it receive.

## The substrate bench

Below the bodies is the substrate: the atom of each kind, its capacity
and occupancy (`atoms::occupancy`), the element table
(`elements::table`), the capture chain and its founding bits
(`elements::founding_bits`), and each kind's pole and emission. The bench
reads the substrate alone, with no founding:

- **The reading.** A new observer word, `substrate`, prints per kind the
  occupancy (`E`, `U`, `2P`, `cap`), the closed share, the chain's count,
  the founding bits, the pole and the emission, in seconds.
- **Trials as data.** A substrate law under trial is written as an exact
  expression over a kind's integers (`Z`, `Q`, `A`, `E`, `U`, `P`, `cap`)
  and evaluated in `Q` over every kind, beside the built law. The trial
  is data, not code, so it is never a parallel model. Adopting it
  replaces the built law in place, with a founding witnessed before and
  after, as the vents of #158 are.

## The board on the hypha

The board is a home, so it runs under the same loop as live play
(`run::run_with`) with a link beside it (`run::Link`). Through the link
an agent on the hypha:

- sends the board's edits (place, add, remove, set) as words, each
  ledgered at its count and refused with the reason when out of domain;
- reads the board's readings, the same words `observe` prints;
- reconstructs any reading from the ledger and the package, since the
  board holds nothing else.

What a board costs on the network, and who may edit a shared board, are
rulings for the network's agreement with Mycelium (#136), not for the
kernel.

## Efficiency

- **No founding.** The substrate's tables are built once per process
  (the atoms and the element table: seconds). Nothing above them exists
  until a body is placed.
- **Read on demand.** A body's readings are computed when read and cached
  by the body and the ledger's height; an edit drops only the caches of
  the body it touches and of the bodies that read its hold.
- **The chain, not all pairs.** A body reads its anchor's chain and its
  own children, so a reading costs the chain's depth, not the number of
  bodies; the hold inside a reach sums only the bodies nearer along the
  chain.
- **Per board, not per process.** The anchor and the grain's unit are
  held by the board (#142), so one process holds several boards and the
  comparison reading reads two at once (#141).

## Phases

| Phase | Work | Issue | Done when |
|---|---|---|---|
| B1 | The substrate bench: `observe substrate`, no founding | [#163](https://github.com/SAIOS-SAMN/xylarium/issues/163) | Every kind's occupancy, closed share, chain count, pole and emission printed in seconds on an empty home |
| B2 | The board home and its ledger | [#164](https://github.com/SAIOS-SAMN/xylarium/issues/164) | A blank board created, edited and read back from its ledger alone; depends on #142 |
| B3 | Bodies as spheres on the enclosure | [#165](https://github.com/SAIOS-SAMN/xylarium/issues/165) | A core, a star and a world placed and read through the built laws, each place stated per plane in degrees |
| B4 | Elements on a body | [#166](https://github.com/SAIOS-SAMN/xylarium/issues/166) | Adding and removing nuclei re-reads the star's pole, sequence and light, and only the dependent readings |
| B5 | Substrate trials as data | [#167](https://github.com/SAIOS-SAMN/xylarium/issues/167) | A trial expression read beside the built law over every kind; ties into #140 |
| B6 | The board on the hypha | [#168](https://github.com/SAIOS-SAMN/xylarium/issues/168) | An agent edits a board and reads it back through `run::Link` |

B1 comes first: it is the bench the remaining vents of #158 are read on.
B2 waits on the anchor per universe (#142).
