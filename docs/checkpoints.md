# Checkpoints

How a universe in play is kept and brought back, once time is a coordinate
(E032, ruled 2026-11-15) rather than one count of the bell. The build-out is
tracked on [#285](https://github.com/SAIOS-SAMN/xylarium/issues/285) and its
sub-issues; the defect it replaces is
[#46](https://github.com/SAIOS-SAMN/xylarium/issues/46).

## What is wrong with a count

Today a checkpoint holds one count `at` (rings after the origin), the seat
holders, every people's numbers and knowledge, and every input at its
count (`xyl_kernel::checkpoint`). A restore regenerates the universe from its
seed, carries it to `at` in one leap with the recorded exoduses and
settlements injected, and lays the held records over the result without
checking them. Three things break under the ruling:

- **There is no one now.** An event's time is its point's coordinate on the
  T lane, and the front reaches `r` at `tau(r) = r + H(<r)`: a people deep
  in its galaxy's hold and one at the rim are not at one count when they are
  read together. A single `at` is a slice the physics does not have.
- **Order is partial, not total.** Two events whose separation in `tau` is
  less than the front needs to carry one to the other cannot have caused one
  another. Applying every input in one total order invents causes; leaping
  to a count loses the order the laws would have read.
- **Held state is trusted, not reconstructed** (#46): the records are laid
  over the regeneration, so a divergence is overwritten, never found.

## A checkpoint is a cut

A checkpoint is a **cut**: each subject's own coordinate, and nothing else
about time.

- **Subjects.** A subject is anything that carries a coordinate on the T lane apart from the others:
  a people (and each seat on it, by its `SeatKey`, #264), a star's system, a
  galaxy's core, the universal core. Each sits in one tier (universe,
  cluster, galaxy, system, planet).
- **Its coordinate.** Each subject's place in the cut is its full point,
  `origin / h : S : G : T : H : K` (FORMS-COORDINATE v4), every lane an exact
  rational, read in the subject's own tier and never converted to another
  tier's unit: a change of tier dilates reach and hold together, so a cut
  holds each subject in the calibre of its tier and relates two only through their
  separation.
- **Consistency.** A cut is causally closed: for every event inside it at
  subject `b`, every event at any subject `a` that could have reached it,
  `tau_a + (tau(b) - tau(a)) <= tau_e` along the front between them, is
  inside it too. A cut that is not closed is refused by name, with the
  event whose cause it lacks.
- **Projection.** Until the ring is each tier's unit (#280, stage 4), the
  home tier's subjects share the bell's count, and the cut's projection on
  that tier is the old `at`. Nothing else may read the projection.

## Orthogonality is kept, never collapsed

The laws read state plane by plane and lane by lane, so a checkpoint keeps
every independent axis apart.

- **Lanes.** S (the reach, boost and dilation), G (the change of kind), T
  (the time coordinate, the beat and the passages) and H (the turnings
  between planes) are held separately for every subject; a separation is
  read per lane, never as one distance.
- **Planes.** A turning held across the six Hopf planes (the axis plane and
  the five isoclinic planes) is held per plane as its Diophantine pair: the
  whole winding in `Z` and the phase in `Q / Z`, with each plane's return
  (model.md, Pairs: the pivot per plane). Never one angle, never one scalar.
- **Rotors.** A held rotor is held whole, with its dimensions
  (`Rotor::dimensions`): the planes it turns and its capacity.

## Three strata of state

| Stratum | What | Kept as | Restored by |
|---|---|---|---|
| Law | Everything a closed form of the coordinates gives: the founding, the galaxy, orbits and their breath, bells and their arrivals, the field's books | nothing | regenerated from the seed and the package, read at each subject's coordinate in the cut |
| Emergent | What the laws decided while the universe was carried: peoples' numbers and knowledge, the carry's exoduses and settlements, ley lines built, annals | its **books**, per subject: the conserved totals the laws keep (counts, holds, energy in and out) | reconstructed by the laws from the seed and the external inputs, then checked against the books |
| External | What no law decides: a mind's or player's `Take`, `Focus`, `Handle`, and every decision a seat holder makes (#266) | every input, whole | applied at its coordinate, under the holder check (#265) |

Only the external stratum is truth the universe cannot regenerate. The
carry's own exoduses and settlements are decisions of the laws and are
decided again, never injected (#46).

## An external input carries its derivation

Every external input is held with what reconstructs it:

- the seat's key and the caller (#264, #265);
- its subject's coordinate when it was decided, and the TUniversal final
  point it was heard under;
- its **causes**: the digests of the percepts and events the holder decided
  on (the percept a mind received, the proposals it had heard). A word
  carries its constructor derivation; a restore reconstructs the input
  where its causes reconstruct, and refuses it by name where they do not.

## Restore reconstructs to the cut

1. Read the cut and refuse it, by name, if it is not causally closed or its
   version is not this one.
2. Regenerate the law stratum from the seed and the package.
3. Carry every subject to the coordinate the cut holds for it by the laws, in the
   causal order: an event is applied once every event in its past cone is.
   Two events that cannot reach one another commute, so any order of them
   gives one result; this is tested, not assumed.
4. Apply each external input at its coordinate, refused where its causes do
   not reconstruct or its caller does not hold the seat.
5. Check the reconstruction's books against the held books, subject by
   subject. Books that close match; a subject whose books do not is a named
   divergence (the subject, the book, both totals), reported and kept, never
   overwritten. Digit parity of carried readings is not the test (ruling
   2026-10-08, regressed 2026-10-24); conservation is.

## The chain of segments

A checkpoint is written as a **segment** of one chain, the same form the
uplink posts to the court as the universe's ledger (Mycelium 78.154), so the
local record and the network's are one object, not two:

- the cut (each subject's point);
- the external inputs since the parent segment, in causal order, with their
  causes;
- the books per subject at the cut, and the state commitment over them
  (the sparse Merkle tree of the ledger, one leaf per subject key);
- the events digest of the inputs it applies;
- the parent segment's hash.

A segment is written whole or not at all: to a temporary file, synced, then
renamed into place, and the last few generations are kept. A segment that
does not read or does not verify is refused by name and the restore falls
back to its parent. Any one subject can be verified alone from its leaf and
its proof, so a partial restore reads only what it needs.

## What this changes in the code

| Today | Under this design |
|---|---|
| `Checkpoint::at`, one count | the cut: each subject's point, in its tier |
| `Checkpoint::holders` and `Input::Take` by seat | unchanged (keys, #264) |
| `Input::Move`, `Input::Settle` recorded and injected | decided again by the carry; their books held |
| `Held` records laid over the regeneration | books checked; divergence named |
| `inputs` in one total order by count | external inputs in causal order, with their causes |
| one text file, overwritten | a chain of segments, written atomically, verified by commitment |
| `commits`, the keepers' counts and final points | the segments' final points; the keepers read their time as separations (#280, stage 3) |

## Order of building

The sub-issues of #285, each with its done-when:

1. [#46](https://github.com/SAIOS-SAMN/xylarium/issues/46): the books per subject and the restore that checks them, the
   emergent stratum reconstructed, divergences named.
2. [#286](https://github.com/SAIOS-SAMN/xylarium/issues/286): external inputs only, with their causes; the carry's acts
   decided again.
3. [#287](https://github.com/SAIOS-SAMN/xylarium/issues/287): segments, atomic writes, generations, the chain and its
   commitment, one object with the uplink's ledger entry.
4. [#288](https://github.com/SAIOS-SAMN/xylarium/issues/288): the cut, each subject's point in its tier, causal closure, the
   causal order of restore and the commuting test; after [#280](https://github.com/SAIOS-SAMN/xylarium/issues/280)'s stage 4.
