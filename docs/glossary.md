# Glossary

Terms used across Xylarium's code and documentation. Formulas are in
[EQUATIONS.md](EQUATIONS.md); module paths are relative to
`crates/xyl-cosmos/src` unless prefixed with another crate.

| Term | Meaning | Code |
|---|---|---|
| admission | the floor on the parent's drawn spin: its torsion share must reach the furthest held kind's reach over the rim, swung by the inbound echo's peak | `circuit::Substrate::least_share`, `cores::substrate_for` |
| anchor | what the founding fixes for every tier: rim, bell, home galaxy's rim, universe radius, first stars, admitted spin, torsion share, firmament margin | `circuit::anchor` |
| ancient | a mind born in a galactic core within one core assembly time of that core's first core-born mind; its house holds the reserve; ancient seats are the only seats a mind joined through the network can hold | `minds::ancients`, `xyl-kernel seats::Role::Ancient` |
| annal | an entry of a universe's record: what happened and when | `xyl-kernel cosmic::Annal` |
| bead | a pinch in a run of founding shells, with its reach, span, state, phase and recipe; a galaxy where its lit mass holds a sun | `circuit::Bead` |
| bell | the unit of time: light's round trip to the home galaxy's rim, `2 rim_galaxy / c` | `circuit::bell_gyr` |
| belt | a disc's matter that held no region of its own, beyond the outermost world | `disc::manifold_worlds`, `systems::Belt` |
| books | a reading's conservation balances, which close when the parts sum to the total: a star's absorbed pulse energy as gravity + light + burned out, the firmament as held + left = founded, and the matter that has left as taken by galaxies + in flight | `xyl-kernel field::Books`, `circuit::Substrate::books` |
| burn | the share of a star's hold the pulse transduces per bell at its reach | `stars::burn_per_bell` |
| capture chain | the founding's nuclei built one capture at a time, `count_Z = floor(count_(Z-1) a h_Z)` | `elements::founded` |
| carry | advancing every people's numbers, knowledge, housing and history over a span; live play and readings use the same carry | `xyl-kernel civ::Civilizations::carry` |
| cascade | the founding's throw of every kind to its own reach about the throat | `formation::founding_shells` |
| census (genesis) | the founding's founded kinds and their amounts, kind by kind, written into the genesis record | `circuit::Substrate::census` |
| checkpoint | the home's file of path-dependent state and recorded inputs, written whole at each save | `xyl-kernel checkpoint::Checkpoint` |
| coherence | the share of a kind a galaxy takes by phase, `1 - 2 min(d, 1 - d)` | `circuit::coherence` |
| constellation | stars bound by their mutual turning tighter than the galaxy's shear across them; the unit of passage | `xyl-kernel transit` |
| council | the ancient seats of an ancient house, where minds joined through the network sit | `xyl-kernel seats::Seats::council` |
| deficit | the share a mass holds at a reach, `delta = M / (R + M)`; the form every holding law in the model takes | `circuit::vacuum_share`, `turning::deficit_share` |
| degree (of a star's life) | its lifetime over 360, at least one bell; worlds in play are read per degree | `worlds::star_degree_gyr` |
| discord | a people's pull against its constellation, `1 - (1 - o f (1 - J))(1 - breach)` | `xyl-kernel transit::Discord` |
| `Dy` | a dyadic rational with a 62-bit mantissa, for readings at seams | `xyl-core dyadic` |
| echo | two bells: the paired throat's near and far rings | `circuit::echo_gyr` |
| eminent | a figure a people remembers a turning of its history by | `xyl-kernel civ::history::Eminent` |
| `Encl` | a certified enclosure: an interval of `Q` proven to contain a value | `xyl-core certified` |
| filament | the pole axis of the pulse, Z at the origin and Q at infinity, the throat at `t = 1` | `circuit::axes`, `circuit::poles` |
| firmament | the throat's held kinds in the vessel band, scaled so their mass is the universal state, 1 | `circuit::Kind`, `Substrate::found` |
| fold | a passage a people makes by folding space where its constellation holds no ley mouth; its energy counts toward its breach | `xyl-kernel transit::Network::fold` |
| forecast (of minds) | for each candidate world, the time (if any) at which its lineage reaches the rung that holds a mind; worked out at genesis and cached | `minds::schedule`, `minds::Schedule` |
| founding digest | a digest of the founding's own source files; it keys the founding cache | `package::founding_digest`, `XYL_FOUNDING_DIGEST` |
| galactic core | a galaxy's central hole, of mass `hole_mass x state` | `cores::Shape` |
| galaxy ring | a bead read one tier down: a ring of the home galaxy in galaxy units | `galaxy::Structure` |
| gauges | a people's instinct, intelligence, sentience and consciousness, each a share in `[0, 1]`, and their combination | `xyl-kernel civ::Gauges` |
| handling | how a ley line's holder handles passage: open, tolled, guarded or closed; set by the realm's steward | `xyl-kernel transit::Handling` |
| held share | the firmament's mass over everything founded | `Substrate::held_share` |
| home | a universe's directory: its genesis record, checkpoint and caches (`--home`) | `xylarium run::Options::home` |
| horizon | the sphere about a core through the crossing at the one-third deficit, `R_h = 2M`, read along its enclosure | `bounce::horizon` |
| house | an ancient people and the seats it holds; it builds ley lines and assigns progenitors | `xyl-kernel seats`, `transit` |
| hypha | the network Mycelium runs; Xylarium reaches it through the uplink | `xylarium run::Link` |
| inbound echo | the waveform at the throat per echo, `cos(2 pi t / echo) + echo q(t)`; every push swings by `1 + A_in` | `Substrate::inbound` |
| jet | the firmament's outflow: a kind leaves once its pole state reaches the throat | `circuit::onset_echoes`, `Substrate::outflow` |
| jetstream | the loop the pulse makes on a held kind over an echo: filament, vacuum and electron phase composed as dilators; one per held kind | `Substrate::loop_flux` |
| landing | a galaxy's share of a kind the jet throws: pull times coherence over the sum | `Substrate::landing` |
| ley line | a circulation held open between two constellations, built by an ancient house or its progenitors | `xyl-kernel transit::Ley` |
| lighting | the home galaxy's gas cooled to collapse: rings to first light, the lit share, each piece's record | `formation::Lighting` |
| lumpiness | the founding's contrast: the beads' states above the mean over every bead's state, in `[0, 1)` | `bang::BigBang::lumpiness` |
| manifold | the region a body holds about its anchor: the arc of its circle its own pull holds, `2 pi r m / (M + m)` | `manifold::reach`, `galaxy::DiscFrame` |
| merge age | the first age a pair's swing spans its gap; the lighter body ends then | `systems::World::merged_at` |
| mu | the Mycelium network's unit of account; Xylarium quotes passage costs in energy | `xyl-kernel transit` |
| ocean | the medium between galaxies, carrying the core's waveform and the jet | `circuit::core` |
| package | what a universe is founded from that its laws do not derive: the core's mass, the SI seam, the founding digest | `package.rs` |
| people | the population of one species with minds on one world | `xyl-kernel cosmic::People` |
| piece | a bead read as a parcel of the home galaxy's gas | `formation::cooling` |
| progenitor | a seat assigned by an ancient house: builds, keeps and runs passage | `xyl-kernel seats::Role::Progenitor` |
| `Q` | an exact rational | `xyl-core q` |
| quarantine | the share a constellation is held closed, `1 - prod(1 - discord)` | `xyl-kernel transit::Network` |
| reading | one `xylarium observe` output, named by a word; a law reading or a carried one (see [architecture.md](architecture.md#readings)) | `xylarium observe` |
| realm | the constellations a passage network joins; each has a steward | `xyl-kernel transit::Network::realms` |
| reverberation | a firmament kind's band swinging about the throat: its reach, hold, deficit, share past the throat and leaving rate | `circuit::Reverberation` |
| rim | the outermost founding shell's outer edge | `Substrate::rim` |
| ring (of time) | one full bell; every count of time is in whole rings | `xyl-kernel universe::ring` |
| ruler | a declared constant used until a law derives it | roadmap, Declared rulers |
| rung | one step of a chemistry's ladder toward a mind: the information it holds, its copying error, its wait and the bodies it feeds | `life::Step`, `life::Ladder` |
| seal | a kind whose every band and every shape of its outermost shell is full: Z 2, 28, 208, 1092 | `atoms::Atom::sealed` |
| seat | a role in the universe: ancient, progenitor, steward, sovereign; filled by the kernel's default rule or taken by a keyed mind | `xyl-kernel seats::Role` |
| settling | the closed-form merging of a disc's bodies by their kicks, carried as ages | `systems::settle_about` |
| sight | which sample stars are unrolled for the forecast: `census` every sample star, `cores` the stars within the cores' reach (`--sight`) | `minds::Sight` |
| sovereign | the seat over one people's worlds | `xyl-kernel seats::Role::Sovereign` |
| state | a bead's mass from its recipe, in the universal core's units | `circuit::Bead` |
| steward | the seat over a realm: diplomacy, trade and disputes between its sovereigns, and its lines' handling | `xyl-kernel seats::Role::Steward` |
| substrate | the core, ocean and galaxies as functions of `t` | `circuit` |
| swing | the greatest kick a body's neighbours give it at the settled age, as a share of its reach | `systems::settle_about` |
| throat | a core's paired faces, near and far; every core is one face of a throat | `bounce::Bounce` |
| throat push | the vacuum's share at the throat itself, `delta(H, H) = 1/2` | `circuit::throat_push` |
| tier | a level of the containment chain, read in its own unit, a power of sixty of `r_g` | `xyl-core units::Tier` |
| tithe | a passage's energy for every lane's length it spans | `xyl-kernel transit::tithe` |
| torsion share | the share the jet carries through the throat at each crossing, `a = (H + J) / (2H + J)` | `bang::torsion_share` |
| TUniversal | the Mycelium network's signed clock; Xylarium measures its pace in rings per final point | `xyl-kernel chronometry` |
| turning (history) | an event of a people's history, remembered by an eminent figure | `xyl-kernel civ::history::Turning` |
| turning law | a body at `r` turns at `omega = c delta(<r) / r` | `turning.rs` |
| universal state | the core's own holding, 1 in its units; the firmament carries it | `Substrate::found` |
| uplink | the program that connects a universe to the hypha; it attaches to live play through `run::Link` and is not part of this repository | `xylarium run::Link`, `run::run_with` |
| vacuum's share | the push the pulse delivers at reach `R` about hold `H`, `H / (R + H)` | `circuit::vacuum_share` |
| vessel | the band of kinds no light shell can grip, from the heaviest seam below the last seal to that seal (Z 568 to 1092) | `atoms::vessel` |
| waveform | the ocean's non-tidal amplitude `A(r, t)` | `Substrate::amplitude` |
| worth law | a ley line is sited where a passage is worth most: the passer's mass times what it saves against normal transit, less its tithe | `xyl-kernel transit` |
