# Changelog

What changed in Ruptura Systematis, newest first. How to play it is in the [README](README.md).

## 30 October 2026

- **The rifle is blue**: its glow, its sight's glass, its chamber, its bolts and the light at its
  muzzle, where the pistol's are green.
- **Droids talk to each other**: two walking about by themselves that meet near you now and
  then stop, face each other and talk - an explorer telling a repair droid the street light is
  out, a sentry asking a passer-by whether it has seen the thief, two repair droids counting
  last night's broken terminals - twelve conversations in System Latin, with subtitles.
- **Select fire**: X takes the rifle from semi-automatic to a three-round burst to full auto
  and round again; the HUD shows which after its name. The mouse wheel changes guns.
- **The rifle is carried across the chest** at the ready, and on the move: its muzzle turned to
  the left and dipped, near parallel to the chest, rather than pointing straight ahead.

## 29 October 2026

- **The rifle fires faster, and sounds its own**: held down, it fires ten bolts a second, where
  the pistol fires once a click, five a second at most. Its shot is its own, from
  `data/sounds/rifle_formants.json`: a deep, short thump with a crack on top, where the pistol's
  is a falling "pyew".
- **The cyber rifle**, carried as well as the pistol: X or the mouse wheel changes from one to the
  other, the one out put away and the other drawn. The rifle is held in both hands, at the ready
  too, with walk, jog and run cycles of its own that keep both hands on it. In first person it
  sits lower at the hip and is raised to look through its reflex sight. The name over the ammo
  says which gun is out. It is built by `~/Documents/blender/weapons/build_cyber_rifle.py`, and
  the droid's hold by `~/Documents/blender/droid/build_rifle.py`.
- **The head turns on the neck**: squaring the face to the front, and the rifle's cheek weld, now
  turn the neck and carry the head, rather than turning the head alone, which wrung the chin.
- **Evidence lockers in the precincts**: a bank of steel lockers, a model of its own
  (`data/locker.glb`), in place of the bronze slab. Its door swings open as you take your pistol
  back.
- **Witnesses, as in Red Dead Redemption 2.** A droid that sees a shooting knows who did it only
  if it saw you within 15 m. A sentry that knows comes after you; one that doesn't searches where
  it happened. A witness running for security can be stopped on the way and questioned: it can be
  intimidated, bribed or talked out of what it saw, or sent home. You are wanted only for what a
  witness could tell was you, not for a body found.
- **The player starts with 200 credits.**
- **Subura**, the undercity west of Urbs, `data/story/story_subura.glb`: a neon slum grown upon
  itself, with shanties, towers of stacked containers, tenement slabs, the Strip under its great
  signs, the Sump of stilt shacks, the Hive's four towers round a court, the Night Market, the
  Pit, and the Stronghold, the gang's compound. A third train, the west train, runs to it on a line
  of its own.
- **Gangs keep to groups of up to six.** One left by itself makes a group afresh with others of
  its kind rather than turning up alone, and the gang left standing after a shootout goes on as
  one group.
- **No more stutter while pursued**: the AI's searches no longer make arrays the size of the
  whole grid, 130 million cells over the cities, each time.
- **Two trains, and the railway on into Curia and Caelum**: the beam goes on over both bridges, to
  a station in Curia past the Gate of Nations and one in Caelum by the terminal, each ending in a
  buffer. The line is one track, so it runs as two that never meet: the north train from Curia
  through Urbs' north station to the middle one, the south train from the middle one through
  Urbs' south station to Caelum. In the middle each ends at a station of its own either side of
  the middle crossing, and riders change trains by crossing the street between them.
- **Curia**, the capital north of Urbs, `data/story/story_curia.glb`, built by
  `data/story/story_curia.py`: a model of its own, played along with Urbs, Ferrum and Caelum,
  Urbs' middle avenue going on over a bridge into it through the Gate of Nations, a triumphal arch.
  Fourteen by fourteen blocks in white stone and gold light, the avenue its axis: the Mall's
  lawns, reflecting pools and obelisks; the Parliament across it in the middle, a rotunda under a
  dome ribbed in gold behind its columns, a portico and two wings on a podium; the Gardens of the
  Republic behind it and the Tower of the Republic at the avenue's head; the Supreme Court and the
  State Archive either side; two Plazas of Nations ringed with flags; colonnaded ministries all
  round, embassies in the west, terraced residences in the east, and eight office towers with lifts.
- **The droids' left hand grips the right wrist properly**: its fingers swung over the top of the
  wrist and their tips closed onto it, rather than standing up off it, and the left shoulder
  rolled forward so the arm reaches out to the wrist rather than lying across the chest.
- **The droids' hand wraps the grip**: the right hand set on the pistol with its knuckles down the
  front of the grip and the wrist cocked down off the forearm, each finger closing until it rests
  on the pistol, the middle, ring and little fingers round the grip, the index on the trigger, the
  thumb round the back. The trigger guard and trigger moved back to meet the grip, as on a pistol,
  where the finger reaches them. The support hand on the right wrist found again for it, pressing
  in no more than before. The crosshair on the sight glows green in `cyber_pistol.blend` itself.

## 28 October 2026

- **Office towers to go up**, fifteen in Urbs' midtown and downtown, seven in Ferrum's workers'
  quarter, ten on Caelum's campus and port: a lobby off the street, open floors over it with desks,
  lit along their ceilings, their windows open to the city over sills too high to jump, and a lift
  up the back to every floor and the roof. The car stops at each floor in turn, up and back down,
  a gate before the shaft standing open while it is there and shut while it is not, and waits
  while someone stands in the doorway. The droids keep off the bottom of the shafts. Built by
  `office_tower` in `data/story/story_urbs.py`, in place of a tower built out of sight so that
  the rest of each city is as it was; driven by `src/lift.rs`.
- **The cyber pistol is a pistol**, built by `~/Documents/blender/weapons/build_cyber_pistol.py`
  in place of the claw and crystal: a raked grip with charge cells down it, a frame with a rail
  and a trigger guard, a slide with serrations and lines of light, the energy core tumbling in a
  window in it, a compensator at the muzzle and a reflex sight at the back, its reticle the sight
  in first person. Every glow on it the green of the bolts, as `cyber_pistol.blend` has it. The
  droids hold it as before, the grip found again for the new shape.
- **A bolt leaves the muzzle**: the pistol held in first person turns its barrel onto whatever
  the middle of the view is on, and a shot leaves the muzzle from where it is that frame, along
  the barrel, before the pistol kicks; drawn out of the muzzle on its first frame, rather than
  half a meter on with a gap between.
- **Caelum**, the spaceport south of Urbs, `data/story/story_caelum.glb`, built by
  `data/story/story_caelum.py`: a model of its own, played along with Urbs and Ferrum, Urbs'
  middle avenue going on over a bridge under an arch of light into it. Fourteen by fourteen
  blocks in white and cyan: the campus by the bridge, its labs under solar panels, its domes and
  the Spire, a tower turning as it rises; the terminal under a roof like a wave; the control tower
  over the field, operations round it, landing fields with spaceplanes parked; the antenna array
  in the west, hangars and cryogenic fuel in the east; and six launch pads along the south, a
  rocket standing on each beside its service tower. Surveying all three cities takes about half
  as long again as Urbs and Ferrum.
- **Three stations on Urbs' railway**: out in the north, in the middle and out in the south by
  the canal, each with platforms either side of the beam, climbed by stairs. The train stands at
  each with its doors open, from the north to the south and back, round the line in a little
  over three minutes. Step up into it from the platform, ride, and step off at another.
- **Droids take the train**, a few at a time: one near a station climbs to the platform, waits
  beside a doorway, gets on when the doors open, stands looking out as it rides, and gets off at
  another station to go about its business there.
- **The cities draw three times as fast**: about 7 ms a frame in Urbs, from about 21. The
  renderer pays by the mesh, not the triangle, so the cities are gathered into tiles of 128 m
  rather than 64. Beyond about 220 m each block of 256 m is drawn from a simplified copy of
  itself, made with the city: the same buildings and lines of light, without the road markings,
  lamp posts and short trim too small to make out that far off. The lit windows are their own
  meshes, near and far alike.

## 27 October 2026

- **A loading screen**, from picking a game until it can be played: the map's name and what it
  is, a bar through loading its models, building the level and surveying it, and a tip.
- **Ferrum**, the works city east of Urbs, `data/story/story_ferrum.glb`, built by
  `data/story/story_ferrum.py`: a model of its own, played along with Urbs, a bridge from Urbs'
  east gate leading into it. Fourteen by fourteen blocks of heavy industry in amber and red: the
  Crucible, a blast furnace with its stoves, bustle pipe and skip incline and iron running in
  channels across its floor; a power station under three chimneys up to 175 m; a rail yard with
  gantry cranes; rolling-mill halls, tank farms, refinery columns and burning flares,
  gasholders in their lattice frames, silos and ore heaps; workers' tenements with access decks
  by the bridge; and quay cranes over Urbs' canal, which carries on through it. Pipe racks cross
  its streets. Surveying the two together takes about twice as long as Urbs alone.
- **The train** on Urbs' elevated railway, `data/train.glb` (made in Blender by
  `~/Documents/blender/train/build_train.py`): three cars floating over the beam, framed in
  light, that stand at the station with their doors open both sides, then run the line to its
  far end in the south and back. Step aboard and ride along.
- **A drain in every cell.** In the corner past the bunk's foot, under a grate lit cyan: E to
  pry it up, a little at a time, unseen, then E to drop down the drain into the escape tunnels,
  where the precinct's hatch comes out. Down there you have got away. And going down any hatch
  with droids after you, they lose you - though those that saw a crime still go to tell a
  sentry.

## 26 October 2026

- **Escape tunnels under Urbs**, as Skyrim's interiors are behind their doors: hatches in each
  precinct's escape tunnel, each bank's strongroom and three manholes in the street, E at any of
  which takes you, the screen going dark, down into a warren hung far under the city - and no
  droid follows. A ring of passages links all twelve hatches, so you can rob a bank and come up
  across the city; off the ways in to its middle, side rooms with caches of credits; and at its
  heart, the den, where a gang keeps that minds you no more than its visitors, and a fence who
  will wipe your record clean - and your escape with it - for a price.
- **Jail in Urbs.** Three precincts stand on podiums in midtown: a hall with the desk and the
  evidence locker, a firewall gate through to the cell block, and cells behind firewalls. When
  security takes your last hit, or you put your hands up with H, you are arrested rather than
  deleted: your pistol goes in the locker, everyone after you stands down, and you go in a cell
  for as long as you are wanted for - every crime witnessed, reported or found adds to it. Five
  ways out: serve the sentence; pick the firewall's lock, then the gate's; bribe a guard;
  lift a guard's key card as it passes with its back to you; or, in the first cell, pry the
  loose panel off its back wall and take the stair down into the tunnel under the podium.
  Out without being let go, you have escaped: a guard that sees you in the precinct goes after
  you, and caught again you are back in for longer. Your pistol is in the locker, behind the
  desk. A guard that sees you picking or prying adds to your sentence.
- **Picking a lock** is a phase lock: rings of light round a core, each drifting round, each
  with a gap. Pick a ring and turn it - held, it stops drifting - until its gap is in the slot
  at the top, where it lights up, and latch it there; latch one out of line and the trace jumps on. Every ring latched, the
  firewall drops; the trace run out, the lock throws you out to start again.
- **Any droid can be shot down**, and those that see it are witnesses: security goes after you,
  and the rest run to the nearest sentry to tell it, unless you stop them first. A body no one
  saw is found by the first to come across it. Self-defense is no crime - but shooting security
  always is.

## 17 October 2026

- **Urbs has its own dialogue**, `data/dialogue/urbs.json`: the same three kinds of droid, in
  the same voices, talking of the city rather than a maze's exit. Security stops you by night
  and wants to know who you are - off the train, or a citizen on the way home, a check with a
  bribe - and is hunting the thief who has been robbing the stores and a bank, whom you can send
  off to the docks; a scout has mapped every street to Summa but the data is classified; a
  maintenance unit can't keep the lights on, since all the power goes to Summa, by the
  administrator's order. Every line is checked by `data/system_latin/sl_parser.py`, and System
  Latin has the city's words now: urbs, turris, summa, argentaria, taberna, aurum, creditum,
  civis, fur, furtum, nox and dies.
- **Urbs is peopled**: 200 droids rather than a maze's 6, and about half its security droids walk
  about in pairs, one keeping beside the other, waiting for it if it falls behind or is talked
  to. Turn one hostile and the other comes too.
- **Droids keep to the pavements** in Urbs, stepping onto the road only at the crossings by the
  junctions, or over the canal's bridges and quays. `data/story/story_urbs.py` writes where the
  roads and crossings are to `story_urbs.roads.json` beside the model, and a step onto the
  road costs a droid's route twenty times a step along the pavement. A droid after you runs
  wherever it likes.
- **A crowd that keeps up**: a droid's next walk is found by heading for a spot a walk away,
  rather than by mapping every way from where it stands over the whole city - 0.1 ms rather
  than half a second, in a debug build - and droids far from you are posed now and then rather
  than every frame: ten times a second past 35 m, four past 80, twice past 160. Each makes way
  only for those in the 3 m squares of ground round it rather than looking through everyone,
  and works out how far it has left to go only as far as it takes to stop. Moving 240 droids
  along costs about 5 ms a frame in a debug build, where at first 48 cost 13.
- **Far droids cost the engine little.** A droid far off and going about its business is moved
  and posed in the scene only as often as it is posed - every bone of one that moves has to be
  worked out afresh, which for 200 droids took the engine 9 ms a step, now under 1. A droid's
  eyes light what is round them only within 30 m, and the lines of light are lit round the 8
  droids nearest you rather than all within 70 m: 23 lights to draw rather than 160.

## 10 October 2026

- **Reparators in orange**, no longer near enough red to pass for a hostile droid.

- **Stores and banks to rob in Urbs.** Thirteen stores to walk into, each with a computer at the
  back keeping its takings, 40 to 600 CR. Six banks, each with four computers holding 1,000 to
  5,000 CR apiece behind a longer breach against a faster trace; clearing one sets off the alarm
  and brings a drone. The one beside the vault's door lists OPEN VAULT once it is breached: the
  door swings open on a strongroom of gold, 10,000 to 30,000 CR to whoever walks in - and two
  more drones on their way.

- **Droids a quarter the size** - each droid's file is 3.3 MB rather than 12, for the same 98
  clips moving the same way: the bones that never move in any clip are no longer keyed in all
  of them, nor the keys a straight line puts back. The site downloads 44 MB less.

## 9 October 2026

- **Nothing ray traced in the cities traced in light** - the Grid, Nexus, the heist districts
  and Urbs: no shadows and no reflections. Their look is their glow, and a shadow from each of
  hundreds of street lamps cost more than it showed. Urbs runs at about 20 frames a second where
  it ran at 8, the Grid at 58 where it ran at 43. Every other map still traces its shadows.
- **Lamps fade out with distance** on maps built from a model: full within 50 m of the player,
  out by 70 m, their glass still glowing beyond. Looking down a street in Urbs, about 28 lamps
  are lit rather than 400, and it runs at about 28 frames a second. The droids glow their own colours there, to be
  seen across the dark.

## 8 October 2026

- **Urbs**, `data/story/story_urbs.glb`, a whole city for a story mode to come, built by
  `data/story/story_urbs.py` with Nexus's helpers: fourteen by fourteen blocks between streets 16 m wide,
  on a platform 912 m across in the void - twenty times Nexus's ground - and some 900
  buildings, every one its own.
  Downtown, towers 70 to 150 m: stepped back as they rise, round, twisting a quarter turn,
  tapering to a point, wedges with a sloping face, towers on podiums of shops, and twin towers
  joined by a skybridge; Summa, the tallest, 185 m, over a terrace climbed from the street.
  Round it, midtown: mid-rise towers, apartment blocks with balconies, courtyard blocks round a
  garden, buildings on stilts over a lit open floor, and rows of shops with awnings and signs.
  Further out, low apartments, townhouses, terraced housing climbed by stairs, and out to the
  edge the suburbs, streets of townhouses and terraces with yards between; south of a canal,
  the docks, with warehouses, sawtooth-roofed works, cooling towers, container stacks and a crane.
  And the central plaza and its monument, six parks with ponds, the market, the civic hall under
  a ribbed dome behind its colonnade, the arena, an elevated railway down the middle avenue with
  a station climbed by stairs, bridges over the canal at every street with steps down to its
  water, and gates out east and west to pads over the void. Windows are lit or dark at random,
  warm in homes, cool in offices, in facades of glass, bronze, concrete, stone, brick and sand;
  streets have dashed lines, crossings, lamps, trees and parked light cycles. Empties named
  `place_*` mark its landmarks for a story to use.
- **Story**, in the main menu, for the story to come. For now it is a crossing of Urbs: off the
  train at the station, find the way through the city to Summa, its tallest tower.
- **No more walking under the platform.** On maps in the void, a street too close to a wall to
  stand on had the platform's underside taken for its ground, and from there the walk spread
  under the whole platform: more than half of what was walkable in Nexus and Helix was under it,
  where droids could be put down. Now a floor under one with no room round it is only walked to,
  never taken for the ground.

## 6 October 2026

- **Battle royale.** Battle Royale in the main menu, on the map of your choice - the Grid or
  Nexus: you and up to 15 droids - 16 in all - every one against every other. Each has three lives: shot down, you come
  back a few seconds later at a start inside the ring, and the third time you are out, told how
  you placed. The last one in wins. A ring of glowing posts closes in on the map in five stages,
  each somewhere new inside the last, hurting anyone caught outside it. `MAZE_ROYALE` sets how
  many play.
- **Battle royale has its own dialogue**, `data/dialogue/royale.json`, not the maze droids'
  talk of exits and codes: four kinds of competitor, each in a model and a voice of its own - the
  proud Gladiator, the hunter Venator, the calculating Eliminator and the wary Vigilator. They
  call out as they hunt the player and go after anyone, and in battle royale's own moments: when
  they have shot someone down, when they come back after losing a life, and when the ring catches
  them outside. Only those within 30 m are heard. Talked to, they taunt the player rather than
  talk. Every line is checked by `data/system_latin/sl_parser.py`.
- **A shield, on the 1 key**: a see-through egg of glass closes round your droid, glowing
  faintly, two rings of light round it, and stops what would hit you - droids' bolts, drones'
  shots, a droid catching you - but not the closing ring. It has health of its own, four hits,
  and its glow is its health: cyan whole, through yellow, to red a hit from breaking, flashing
  as it takes each. Raised, it stays up until it breaks, and only then charges again, over 18
  seconds, so it cannot be raised and let go again and again. While it is up, the health bar is
  its: blue, named SHIELD ACTIVE, as full as the shield's health. While it can be raised -
  charged, or charging with a spare - the bar is named `HEALTH [1]`. The egg is made in Blender by
  `data/shield.py`; from inside it, aiming over the shoulder, its rings are hidden.
- **Shield cells**, spare charges for the shield, float about the levels as the hearts do, with
  a blue lamp - but rarer, about one for every three hearts (one for every 15,000 cells of
  floor, between 1 and 4). Walked into, one is kept as a spare, two at most; a spare raises the
  shield at once while it is still charging after breaking. Picking one up says how many are
  kept. Going down loses them. The model, `data/shield_pickup.glb`, is exported from
  `shield.blend` in Blender.
- **Heist**, a new game in the main menu, in districts of the city in the void: break into a
  bank's vault, steal what you can, and get out. Three **firewall doors** stand between you and
  the vault - a doorway's frame traced in orange with a pane of a firewall's glass in it,
  flickering, made by `data/heist/door.py` - each opened by hacking the computer on its near
  side: the compound's gate from the street, the bank's door from the courtyard, the vault's from
  the lobby. Guards keep posts in the streets, the courtyard and the lobby, and drones patrol.
  Once the vault is open, its six terminals are there to hack, each carrying far more credits
  than a computer elsewhere - and security sends waves of guards in after you, each bigger than
  the last, with a drone called in each time and a lull between them. A failed hack, at any
  stage, brings a wave five seconds later. Get out at either extraction pad whenever you like:
  only then are the credits yours, and going down loses them. The status line says what to do
  next, how much is stolen and when the next wave comes. The first district is **Aurum**,
  `data/arena/heist_aurum.glb`, built by `data/arena/heist_aurum.py` with Nexus's helpers: a
  walled bank compound among towers, pods and domes, its lines of light lighting what is round
  them. The second is **Cortex**, `data/arena/heist_cortex.glb`, built by
  `data/arena/heist_cortex.py`: a data quarter on an octagonal platform, its walled yard watched
  from guard towers at its corners - stairs up to a platform with a parapet, a guard on each - and
  in it the data hall, rows of glowing server racks to creep between, and at the hall's end the
  core chamber, its glowing data core ringed with terminals; server blocks banded with light and
  cooling units stand about its streets. The third is **Helix**, `data/arena/heist_helix.glb`,
  built by `data/arena/heist_helix.py`: a gene lab's campus in a round wall, sentry pods either
  side of its gate, greenhouse domes in its yard; the lab's atrium rises to the roof, stairs climb
  its west wall to a gallery, and on the gallery, walled off, the vault - a double helix of light
  turning over a plinth, six terminals round it. Helix towers twist a quarter turn as they rise
  among specimen tanks about its streets. `MAZE_HEIST_OPEN=<n>` opens the first n doors, to try a
  heist out.
- **Stamina cells** freeze the player's breath for 10 seconds: running and sprinting spend none
  of it, though walking still gets it back, and a player out of breath is no longer winded. They
  float about as hearts do, with a green lamp, rarer than hearts but commoner than shield cells
  (one for every 10,000 cells of floor, between 2 and 6), and are only picked up while breath is
  not frozen already. While it is, the stamina bar is the colour of ice, named STAMINA FROZEN
  with the seconds it has left. The model, `data/stamina_pickup.glb`, is exported from
  `stamina.blend` in Blender.
- **Voices echo on the Grid and in Nexus**, off the buildings of a city in the open: each line
  comes back a quarter of a second later, again and again, fainter and duller each time, for
  about a second and a half after it is said (`synth::CITY_ECHO`). A map says whether its voices
  echo (`ctf::Map::echoes`); capture the flag's do not.
- **The town is gone**: battle royale's first map, a walled town at night, was too dark to play,
  and the Grid and Nexus took its place. `data/arena/br_town.py`, which built it, stays: the
  Grid's buildings, and the helpers both new maps are made with, are its.
- **The Grid**, `data/arena/br_grid.glb`, a city traced in light floating in the void, built by
  `data/arena/br_grid.py` with the town's buildings: black blocks on a black platform ruled with
  a grid of light, every corner, floor, roof and door of every building lined in glowing colour -
  each block a district of its own colour, cyan, orange, pink, yellow, green, violet or red - and
  every crate, low wall and parked light cycle with a glowing edge. A barrier lit
  along its top runs round the platform's edge, but for a gap on each side where a bridge goes
  out over the void, no rail on it, to a pad with a tower: the highest ground there is. Light
  pylons stand at every other corner. The engine lights what a surface gives off by the surface's
  own colour, so each line of light is its own colour too, to glow at full strength.
- **The Grid's lines of light light what is round them**, with traced soft shadows: a droid by a
  glowing wall is lit in its colour, and stands out against the dark. The map's script writes
  every line, cut into pieces 3 m long at most, to `data/arena/br_grid.glow.json`; each frame the
  game lights as area lights the pieces nearest the player and nearest each droid within 70 m,
  the nearest droids first, fading each in and out - 64 at once, the engine's most (raised from
  16). It costs nothing measurable. Only where shadows are traced, so not in the browser, where
  their light would shine through walls; `MAZE_AREA_LIGHTS=0` turns them off.
- **Nexus**, `data/arena/br_nexus.glb`, a third battle royale map, built by
  `data/arena/br_nexus.py`: a futuristic city on an octagonal platform in the void, every edge
  traced in light, with nothing in it a house or an office. A spire, the Core, tapers 48 m up
  from the middle with rings of light floating round it; round it a skyway, an octagonal ring of
  walkway 7 m up on lit pillars, stairs up from the four ways; on the diagonals four ziggurats
  turned to face the Core, three tiers each with stairs up the outside, a bridge from the skyway
  landing on each one's second terrace; four ribbed domes; eight pods, discs on a single column
  with stairs up and a lamp shining down under each; and bridges out to pads with lit obelisks.
  The floor is ruled with lines of light running out from the plaza. Its lines of light light
  what is round them, as on the Grid (`br_nexus.glow.json`).
- **Lamps on floors over one another** each get their own: fixtures share a lamp only on the
  same floor.

## 1 October 2026

- **A choice of maps for capture the flag.** Capture the Flag in the main menu opens a page of
  maps: Lanes, the first, and Balconies, `data/arena/ctf_balconies.glb`, on two floors - each
  flag in an open well under a balcony round three sides of it, catwalks along the side walls
  joining the balconies, a raised hub in the middle, and stairs between them all.
  `data/arena/ctf_balconies.py` builds it in Blender, its cover laid out at random from a seed and
  mirrored end to end.
- **The menus work from the keyboard**: the arrow keys or W and S go up and down the buttons, the
  one picked shown in light blue, and Enter or Space presses it - in the main menu, its maps, the
  pause menu and the options.
- **Stairs.** The droids find their way up and down stairs: the walk grid's floor in each cell is
  now the highest with headroom over it, and only what can be climbed to from the ground a
  stair's step at a time (hydroxus-ai's `MAX_CLIMB`, 0.55 m) is walkable - so stairs and the
  floors they lead to are, and the tops of crates and railings no longer are. A droid on the
  stairs puts each foot down on its own step: the hips go down to the lower foot and each leg
  bends at the knee to reach its step, keeping the walk's lift and the foot's angle. The player
  walks up steps of up to 0.4 m too, the eyes coming up after the body over a moment. The team
  battle climbs as well.

## 30 September 2026

- **Your allies say something when you talk to them** in capture the flag: E by one of blue's
  droids has it say one of a handful of lines, in its own voice, without a conversation. As it
  speaks it turns its head to you, as far as 90 degrees either way, without turning round. Red's
  are not talked to.
- **You play for blue in capture the flag**, and take red's flag. Your allies are blue's, in the
  cyan droid, and the guards against you red's, in the red one; the voices stay with the parts,
  your allies' high and quick, the guards' deep, slow and rasping.
- **Capture the flag is taken home.** With red's firewall down, reaching their flag takes it:
  it rides on your droid's back, shrunk and slung across it at 45 degrees, and the marker moves
  to your own flag. Bringing it home to blue's wins the round.
- **A main menu** to pick the maze or capture the flag, which the pause menu can go back to.
- **A capture-the-flag map**, `data/arena/ctf_map.glb`: a red base at one end and a cyan base at
  the other, each with its flag, joined by three lanes. The team battle can be fought in it
  with `BATTLE_MAP`.
- **Sides in capture the flag.** Each side has three droids and a drone. Red's are yours: they
  pay you no heed and go after blue's. Blue's guard their end, watching for you all the while,
  and go after you or red's, whichever they see. Each droid keeps to a post of its own in its
  side's half - inside its base, in the yard before the door, and forward in the middle lane -
  and each drone patrols round its flag. Droids
  shoot with their pistols and drones fire as ever, and each side's shots harm only the other
  side, yours included. Your own bolts pass red's by.
- **Capture the flag's droids say their own things**, from `data/dialogue/ctf.json`: red's
  Protegators and blue's Obstruators call out as they fight, and are not talked to. Each side
  has a voice of its own, red's high and quick, blue's deep, slow and rasping, to tell them
  apart by ear.
- **The firewall's shell no longer turns solid up close.** It does not bend what is seen through
  it, as glass does, and glows less, so what is inside it shows through from any distance.
- **Flags in firewalls.** Each flag stands in a firewall, a flickering shell of orange glass that
  nothing gets through, until the computer it answers to, in the same base, is hacked. Take
  blue's flag to win the round. The flags and the firewall are the models made in Blender,
  exported by `data/ctf/export_models.py`.
- **The droids see only in front of them, always.** On ALERT too, and right next to them too: a
  droid or a drone can be crept up on from behind, or slipped round while it looks the other
  way. A hunting droid that loses you round its back has to search for you.
- **The AI is a crate of its own**, `hydroxus-ai` (see above), fetched from GitHub as Hydroxus
  and fyrox-gfx are. The droids behave as before, but for what they see.

- **The engine fork is now Hydroxus.** The fork of Fyrox the game is built on has a name, and a
  README of its own crediting Fyrox. Its crates keep their Fyrox names, so nothing in the game
  changes but where Cargo fetches it from.
- **Upstream Fyrox merged into Hydroxus**, up to 27 September: Dmitry Stepanov's markdown and
  inline text elements in the UI, and fixes to the editor and inspector. Nothing in the game's
  rendering changes.
- **The team battle example shows the battle.** It started its camera above the arena's roof,
  so all it showed was the ceiling; now it starts inside, looking across the arena from one long
  side. Its scoreboard is at the top in the middle and its help along the bottom, instead of on
  top of each other.
- **The team battle is fought with the game's own droids.** Each side is one of the game's
  droids - red in `droid_hostile`, cyan in `droid_full_deform` - animated as the maze's droids
  are: jogging as they advance, running for cover, crouching behind it, pistol raised at a target,
  eyes in their team's colour and flashing white when hit. They go at the pace of their own
  animations, so their feet keep to the floor, which makes a match slower than it was. For this,
  the game is a library as well as a program (`src/lib.rs`), which the examples can use.
- **Every one of the droid's animations in the team battle.** Walking with no enemy known of,
  jogging to one, running for cover and sprinting when falling back hurt, with the skids that
  come with stopping and turning; strafing and crouch-strafing while facing a target; a short or
  high jump now and then as a droid breaks for cover; the pistol drawn, raised, aimed up, down
  and to the side at the target, fired, and holstered once the fighting is over. Shot down, a
  droid falls limp, knocked back by the shot, and may lose what it was hit in, as in the game;
  it comes back as a new droid. Fallen droids and what they lost do not stop shots.
- **The team battle's droids fire the game's green bolts**, as the player's pistol does, in
  place of red and cyan beams: each leaves the muzzle as the pistol fires, flies at the bolt's
  speed, lights up what it passes, and glows where it lands - on the droid it hits, whose harm it
  does when it gets there, or on the cover or wall it hits instead. The pistol's bolts now fly by
  themselves (`Bolts::fly`), for anyone to fire; the player's fly as before.
- **The team battle's bolts leave straight out of the pistol**, the way its barrel points, as
  the player's do in third person. The droids used to face the arena's fixed forward while they
  strafed, so their pistols pointed well off to the side of whoever they shot at; each droid's
  body now turns the way it faces, as the player's does with the camera. A droid also learns how
  far its barrel ends up above or beside where it aims, and aims that much the other way.
- **The maze's droids search together, and cut you off.** Once they lose sight of you - or
  hear you, or answer an alarm - they share one picture of where you could be by now, spreading
  along the corridors as fast as you could run, and cleared wherever one of them looks and does
  not see you. Each goes to look where you are likeliest to be, clear of where the others are
  going, instead of wandering off at random (hydroxus-ai's `search`). In a simulated maze, three
  of them find a player who has run off and hidden 98% of the time this way, against 83% before.
  While several of them are after you, only the nearest runs straight at you; the others make
  for somewhere ahead of where you are going, to cut you off.
- **An alarm you can hear.** A droid sounding the alarm, and a failed hack calling a drone in,
  set off a klaxon - three rising whoops - that carries through the maze
  (`data/sounds/alarm_formants.json`).
- **The team battle's sides fight as teams.** Each droid picks its target - whoever is shooting
  at it, hurt, or in the open, before whoever is merely nearest - and takes cover from every
  enemy its side knows of, spread out from its teammates. They move out of the enemy's sight
  where they can, two of each five go round the side, they leave cover only while a teammate is
  firing, and they fire at where an enemy was just seen to keep their head down, which does. A
  droid getting nowhere finds somewhere else to go, and one caught without cover sidesteps
  rather than stand there. `BATTLE_OLD=red` (or `cyan`, `both`) has a side fight as before:
  against the old way, fighting as a team took 64% of the kills over two runs, one from each
  end of the arena.

## 29 September 2026

- **Bribes.** A Speech check can now be tried with credits instead, offered under it: `[Speech 75%]
  [40.00 CR] ...`. It is still a Speech check, only likelier to succeed, and the credits are paid
  whether it works or not; trying the check or its bribe uses up both. Without the credits, the
  bribe is dimmed and cannot be picked. Sentries take a bribe to let a "maintenance unit" through
  (40% or 75% for 40 CR) and to believe you saw something behind you (30% or 65% for 25 CR), and
  scouts to share their map to the exit (50% or 85% for 20 CR). Maintenance units cannot be
  bribed - they do not know where the exit is - and drones cannot be talked to at all. Bribes are
  set per check in `data/dialogue/droids.json`.
- **A security drone patrols the maze.** It is put down well away from where you start and
  hovers along the corridors from one spot to the next, calm, its glow green, paying you no
  heed. It turns on you only when a scout or maintenance unit sounds the alarm, or when a bolt
  from your pistol hits a droid (or the drone): it says so ("Sirenatio accipatum. Zeto
  aggressor."), flies to where you were, and searches there and nearby for 30 seconds, scanning
  at each spot, its glow orange, before it calms down and patrols again. Seeing you, it goes red,
  comes on to 6 m and fires a plasma shot every second and a half while it can see you; a shot
  that hits takes a third of your health, as a sentry's touch does. Three bolts bring it down:
  its rings break into four pieces each, which fly apart, and it drops with them to the floor.
  It sees in front of it as a sentry that is not on Alert does. What it says is in
  `data/dialogue/drone.json`, by what it says it about.
- **The drone's shot** is a glowing ball with a tail and two small rings turning about it, made
  in Blender (`data/drone_shot.glb`), in the colour of the drone's mood, lighting what it passes;
  it sounds and hums as the pistol's bolts do. It flies at 12 m/s, slow enough to step out of the
  way of from a few steps off, and stops at the first thing it hits.
- **Credits.** Every computer carries credits, whatever else is on it: a random amount, most
  of them a few credits and now and then a good deal more, up to 500. Clearing its hack
  transfers them to you - the terminal says how many under ACCESS GRANTED - and your credits
  show in gold in the top right, kept from one maze to the next. Credits go as dollars and cents
  do: whole credits and hundredths of one, written `1,234.56 CR`.
- **The fallen stay down.** A droid shot down lies where it fell, broken parts and all, for the
  rest of the round, rather than disappearing a few seconds later; the others walk over it.
- **A failed hack calls in a drone.** When the trace completes, the patrolling drone nearest the
  computer comes to search there, if it is within 25 m; if not, one more flies in from out of
  sight, 20 to 40 m off by the corridors - there are four in all - and failing that, the nearest
  one patrolling or searching goes there instead, however far. Some drone always comes, and
  finds its way however far it has to go; its 30 s of searching start once it gets there.
- **Health bars over the sentries and the drone.** A sentry's red health bar sits over its
  stamina bar, shown with it - once it is after you, has spent some breath or has been hit -
  and a bolt takes a third of it. The drone has a health bar alone, while it is after you or has
  been hit.
- **Jog, run and sprint.** Caps Lock still goes between walking and jogging. A tap of Shift now
  goes between walking and running, a pace between a jog and a sprint - the sprint's stride,
  stepped out slower - which costs stamina, if far less than a sprint: 20 seconds of it on a
  full breath, to a sprint's 6. Held down, Shift sprints for as long as it is held, from any
  pace, and lets go back to it. Running is heard further off than jogging, if not as far as a
  sprint.
- **The run has a stride of its own,** `droid_running_cycle`, in place of the sprint played
  slower: a mid-foot landing, the hands half curled, the arms pumping between the jog's and the
  sprint's. Every droid has it: a sentry runs after you once it has you, sprinting now and then,
  and jogs to where it lost you. The run has its own strafes, skids and jumps too: running flat
  out, a droid now skids round, cuts across and slides to a stop as it does sprinting, if in a
  shorter slide, and strafing - or with the pistol out - a sprint slows to a run rather than a
  jog. The other droids' colours are the player's droid recoloured by
  `data/recolour_droids.py`, to be run again whenever the droid is exported afresh, so that
  they have every animation it has.
- **The alert, as in Metal Gear and Fallout,** is at the top in the middle, in its colour: ALERT
  in red, EVASION in amber and CAUTION in yellow, each counting down but ALERT, and ALERT and
  CAUTION blinking.
- **Stamina** is a bar in the bottom left corner: green, yellow once it runs low, and red,
  blinking, while you are winded.
- **The cyber pistol's ammo** shows in the bottom right corner while it is out: ∞, since it
  never runs out.
- **Hearts float in the corridors.** A few health items - three in a small maze, up to twelve in
  a big one, the first 8 to 20 m from the start - are scattered afresh each maze on open floor
  away from the start and from each other, each floating at chest height, slowly spinning and bobbing. For now they only float
  there: there is no health for them to give back yet. The model is `data/health.glb`, exported
  from `health.blend` without texture coordinates (it has none, and a constant set of them
  leaves it unlit); its see-through outer layer is made refractive glass in the game, since the
  engine draws a glTF model's surfaces solid whatever their alpha.
- **What glows lights what is round it,** with ray-traced shadows like every other light: each
  heart in red, each computer's frame in its colour (red while locked, blue once cleared), from four lamps round its rim that together light as the frame does, and leave no spot of light on the screen, the
  cyber pistol's muzzle in green while it is out, and every droid's eyes - dimly - in the
  colour they glow. The engine lights nothing from a glowing surface itself, so each has a small
  lamp of its own, as the drone does.
- **Drones speak System Latin,** as they start each of their animations, in lines from
  `data/dialogue/drone.json` - but as beeps, hums and buzzes, which no one would hear as speech:
  each vowel a beep on a note of its own, m, n, l and r a hum, the hissing letters a buzz and the
  stops a click (`data/sounds/drone_voice.json`).
- **Sentries keep their own colours** while they are after you. The hostile droid's colours,
  `data/droid_hostile.glb`, are for another kind of droid, one given `hostile_colours` in
  `data/dialogue/droids.json`; until there is one, the model is not even loaded.
- **Shadows of what moves follow it** with ray tracing too: droids, their guns and the drone
  cast their shadows from where they are, droids in the pose they are drawn in, as they do with
  shadow maps.

## 26 September 2026

- **A computer to hack, as in Welcome to the Game.** A monitor and keyboard float against a wall
  a few steps from where you start, the monitor's frame glowing red while it is locked. Close to
  it and in front of it, `Computer` and `E) Hack` show under the middle of the screen; E takes
  the view in close, square on to the screen, and a terminal in green text comes up on the
  screen itself (`MAZE_TERMINAL_OVERLAY=1` puts it over the screen instead, as a panel of the
  game's interface, as it used to be). Enter starts a breach: six commands, one at a time, each
  to be typed exactly into the box under the output before its trace runs out, 15 seconds for
  each. A wrong key does not go in, costs half a second, flashes `ERR` and jolts the
  terminal, and the trace bar flashes once it is nearly out. If the trace completes first,
  access is denied and Enter tries again at once; all six typed, access is granted and the
  frame glows blue. Tab walks away. The clock keeps running, and the droids keep going about
  their business, while you type. For now it opens nothing: it is there to try the hacking out.
  The model is `data/computer.glb`, made from `unlockables/computer.blend`; the terminal is in
  DejaVu Sans Mono (`data/fonts/`).
- **Shot down, droids fall limp**, as ragdolls: every part of the body falls as the physics has
  it, knocked back by the bolt, and a bolt that hits one lying there shoves it. A few seconds
  later the droid is gone.
- **Bolts break droids apart - no blood, just broken geometry.** The bolt that brings a droid
  down, or any bolt that hits one lying there, breaks off the part it hits: its head, an arm at
  the shoulder or the elbow, a leg at the hip or the knee (a hand breaks at the elbow, a foot at
  the knee). The part falls away by itself, both broken ends showing the torn shell and the
  glowing green voxels inside, and a handful of loose voxels spill out of them and bounce across
  the floor.
- **Running at the ready.** With the pistol held low, the droid's left arm pumps as it runs,
  its shoulders twist against its hips with each stride, and the pistol bobs, the muzzle dipping
  as each landing drops the hips.

## 25 September 2026

- **Talking to the droids, as in Fallout 3.** Close to a droid and facing it, its name shows under
  the middle of the screen with `E) Talk`. E starts talking: the droid stops and turns to you,
  yours turns to it, and the camera leaves its shoulder for a close-up of the droid's face, the
  view narrowed as with a long lens. What it says runs along the bottom of the screen in a green
  panel, in System Latin with the English under it, and under that the replies. Pick one with
  the mouse, with W and S or the arrows and then E, Enter or Space, or with its number; Tab walks
  away. A reply can be a Speech check (`[Speech 40%]`), which goes one way if it succeeds and
  another if it fails, and can be tried only once. Replies already given are dimmed. The clock
  stops while you talk.
- **Three kinds of droid**, each with its own code: a sentry (Defendator), a scout (Explorator)
  and a maintenance unit (Reparator), in turn. Talk the scout or the sentry round and it tells
  you how far off the exit is and which way, as the crow flies. What they say is in
  `data/dialogue/droids.json`, which can be rewritten without a rebuild. Every line parses with
  the System Latin parser in `data/system_latin/`.
- **The droids speak.** Each line is said out loud in a voice made from formants, as the pistol's
  sounds are, heard from the droid's face. System Latin is read as it is written, each sound
  gliding into the next, each word a moment apart and each vowel on a note of its own, falling
  at the end of a sentence and rising at a question. The sentry speaks low and slow, the scout
  high and quick, and the maintenance unit breathily; every droid a little higher or lower than
  the rest of its kind. The sounds and voices are in `data/sounds/voice_formants.json`, which
  can be retuned without a rebuild. A line is cut off by the next one, or by walking away.
- **The close-up looks level and square on at the droid's face**, from low enough that the
  face is up above the conversation at the bottom of the screen.
- **Moods.** The conversation's panel takes the colour of how the droid feels about what it is
  saying: green as usual, blue for success, yellow for a warning or a question, orange for
  agitation or a failed Speech check, and red when it is hostile. Each line in
  `data/dialogue/droids.json` can say its own; one that does not is blue after a check that
  succeeded, orange after one that failed, and green otherwise.
- **The eyes go with the mood too**: the droid being talked to has its eyes glow the panel's
  colour, and their own green again as usual and once the conversation is over. The colour
  they glow as usual, and how brightly, is the model's own `eye_glow` material.
- **The voice goes with the mood**: higher when pleased, questioning or agitated, lower when
  hostile, as `moods` in `data/sounds/voice_formants.json` has it.
- **Pausing stops the sound too**: a droid's line, the pistol and its bolts carry on from where
  they were once the game is resumed.
- **Always someone to talk to.** One of the droids stands just in front of you at the start of
  each round, facing you, and stays there rather than wandering off, even once you have talked
  to it.
- **Sentries turn hostile.** Fail the sentry's Speech check to pass as a maintenance unit, and it
  orders you out of its zone, in orange; walk away and that is the end of it, but refuse and it
  says "You are an intruder. Prepare for deletion." in red. Fail to send it after something
  behind you, and it sees through you and says the same. Its eyes stay red, it cannot be
  talked to any more, and once the conversation is over, a moment later it comes after you at
  a run, and hunts you as below. If it catches you,
  you are deleted, and a new maze begins a few seconds later. Three bolts from the pistol stop
  it: it goes down in a crouch, its eyes dark, and stays there. A line in
  `data/dialogue/droids.json` with `"attacks": true` has its droid do this.
- **Stealth, as in Metal Gear.** A hostile droid goes through Metal Gear's phases, shown in the
  status line and in its eyes:
  - **ALERT** (red eyes): it can see you, and runs at you.
  - **EVASION** (orange), with the seconds of searching it has left: it has lost you. It runs to
    where you were heading when it last saw you and looks about, then walks from one spot to
    another nearby, looking about at each, for 30 seconds.
  - **CAUTION** (yellow), with the seconds left: it has given up, and wanders, still watching
    for you. After a minute it calms down, its eyes go back to green, and it can be talked to
    again.

  It sees only what is in front of it, 55 degrees either way, in every phase, even right next
  to it: it can be crept up on, or slipped round. On ALERT it sees you 30 m off however low you
  are; otherwise 30 m off standing, half as far crouched and under a third as far crawling.
  Seeing you in any phase puts it back on ALERT. A bolt that hits it without it seeing you has
  it search where you fired from. As it spots you, loses you and gives up, it says so out loud: "Intrusor detectum. Sta." ("Intruder detected.
  Halt."), "Intrusor perdatum. Zeto intrusor." ("Intruder lost. Searching for the intruder.")
  and "Phantasma. Resumo patrolium." ("A sensor ghost. Resuming patrol."), with subtitles
  under the status line. These are the sentry's `barks` in `data/dialogue/droids.json`.
- **Sentries join a chase.** A sentry that sees another sentry on ALERT, running after you, goes
  after you too, calm or not: the one it sees shows it where you are. It sees the other as it
  would see you standing there - in front of it, less far with the lights off -
  and a calm one stands a second before it sets off, as one does that has just turned hostile.
  A sentry that sees one that has joined in joins in as well. Once it sees neither you nor
  anyone after you, it searches where you were, as usual.
- **Noise.** Out of ALERT, a hostile droid listens as well. Walking, crouching and crawling are
  silent; running is heard 5 m off, sprinting 10 m, landing hard from a fall 6 m, a pistol shot
  12 m and a bolt hitting something 8 m - measured along the corridors, so a wall between you
  muffles it. A droid that hears you says "Quid? Sonus detectum. Verifico." ("What? A sound
  detected. Checking."), runs to where the noise was and searches from there, and pays no heed
  to another noise for 3 seconds. A bolt fired into a wall away from you makes a distraction.
- **Darkness.** With the lights off, a droid sees only about a third as far, whichever phase it
  is in - unless your flashlight is on, which gives you away as if the lights were on.
- **The crosshair is always there.** The pistol's screen, crosshair and all, shows the whole
  time the pistol is out, rather than only for a moment as it fires; each pull of the trigger
  now flashes it brighter, the crosshair with it. The crosshair's marks keep the model's own
  solid green glow instead of being turned into glass with the screen, and the screen's glass no longer mirrors its
  surroundings or catches highlights from lights: only what is seen through it, and its glow.
- **Don't point that at me.** Every droid minds having the pistol pointed at it - the middle
  of the view on it, within 20 m, with the pistol out. Keep it there and the droid stops, turns
  to face you and warns you, its eyes yellow. Each warning then holds 2.5 seconds longer than
  that, for it to be said and taken in, before the next: the last warning, its eyes orange, and
  then being provoked. Sentries are the least patient, warning you after 0.8 seconds, again at
  4.1 and provoked at 7.4, and provoked, they come after you ("Aggressor detectum.
  Deletabso tu." - "Attacker detected. I will delete you."). Scouts and maintenance units
  warn you after 2 seconds, again at 6.5 and provoked at 11, and then sound the alarm ("Aggressor detectum. Sireno."):
  every sentry within 40 m that is not already after you comes to search where you are
  ("Sirenatio accipatum. Zeto aggressor." - "Alarm received. Searching for the attacker.").
  Looking away only holds the next stage off; lower the pistol or keep it off a droid for 2.5
  seconds and it calms down ("Accipatum."). Shooting a droid provokes it at once. How patient each kind is, and what it
  does, is its `threatened` in `data/dialogue/droids.json`, and what it says are its `barks`.
- **The pistol in first person, as in Call of Duty.** Seen through the droid's eyes, the pistol
  is held in view, low and to the right, pointing in at the middle of the view. Holding the
  right mouse button raises it into the middle to aim, its screen - crosshair and all - square
  in the middle of the view like a sight. It comes up from below as it is drawn and goes back
  down as it is holstered, kicks back and up with each shot, and tucks down out of the way
  right up against a wall; the ball at its muzzle tumbles and its screen flashes as the
  droid's own do. In first person a shot leaves its muzzle for whatever the middle of the view
  is on.
- **A bolt shows where it hits.** It stops with its tip against whatever it hit and glows there
  a moment, lighting it up, rather than going out a step short of it.

## 24 September 2026

- **A pistol.** R draws the droid's pistol and holsters it again; the left mouse button draws it
  too, and once it is drawn fires it. Drawn, the droid aims ahead and strafes as with the right
  mouse button, its upper body in the pistol's poses (`droid_pistol_draw`, `_aim`, `_fire`,
  `_holster`) while its legs walk, run or stand as ever. The pistol shows and goes partway
  through the draw and the holster, as `data/droid_motion.json` says, and is out of sight in
  its hand until then. Drawn, it is held lower, at the ready (`droid_pistol_ready`); holding
  the right mouse button raises it to aim, and a shot from the ready raises it, fires as soon as
  it is up and lowers it again a second after the last shot. Raised or at the ready, the pistol
  follows the camera: the droid's upper body leans it
  up, down and round towards wherever the camera looks, up to 60 degrees each way, blending the
  model's aims (`droid_pistol_aim_*`, laid out in `droid_motion.json`). A shot is a glowing bolt
  from the muzzle that flies the way the gun points and stops at the first thing it hits,
  harming nothing yet. The
  bolt is green and carries a green light with it, lighting up the droid as it leaves and
  everything it passes in the dark. The pistol's screen, see-through green glass, flashes with
  each pull of the trigger; the ball at its muzzle - a glowing core in a see-through green
  shell - shows all the while the pistol is out, the core and the shell tumbling every way
  round, each the opposite way to the other.
- **Square to the front.** Strafing, or with the pistol drawn, the droid's face and shoulders
  stay square to straight ahead, however its hips turn to the way it steps.
- **N for a new maze**, since R is the pistol's now.
- **Over the right shoulder.** The camera sits closer and further out to the droid's right, as in
  Fallout: the droid stands to the left of the view, the way ahead clear on the right. It had
  been over the left shoulder. Holding the right mouse button brings it in closer, to aim over
  the shoulder, and letting go takes it back out. Holding the middle mouse button to swing the
  camera round and look at the droid leaves the droid as it was: its pistol no longer follows
  the camera round.
- **Head bob in first person.** Seen through the droid's eyes, the head rises and falls in step
  with the stride again, and fades out on going over to the view from behind.
- **The pistol sounds.** A shot cracks at the muzzle, and each bolt hums as it flies. The sounds
  are made from formants when the game starts - a buzz and noise shaped by resonances, as a
  voice is - described in `data/sounds/pistol_formants.json`, which can be retuned without a
  rebuild; `src/formants` makes them. What sounds is heard from the camera.
- **The flashlight starts off.** F switches it on.

## 23 September 2026

- **Strafing.** Holding the right mouse button keeps the droid facing ahead whichever way it goes:
  it steps sideways, back and along every diagonal in its new strafes (`droid_strafe_walk_*`,
  `droid_strafe_run_*` and `droid_strafe_crouch_*`, crouched or crawling, each `_L`, `_R`,
  `_FL`, `_FR`, `_B`, `_BL` and `_BR`), turning only a few degrees whichever way it goes.
  Strafing, a sprint slows to a run, picking up again once the button is let go, and the droid
  does not skid. In cover the
  wall still sets which way it faces.
- **Skids of every kind, sprinting only.** Sprinting flat out, the droid skids round
  (`droid_skid_sprint_turn_L`/`_R`) and runs out through the run, cuts across a quarter turn
  (`droid_skid_sprint_turn90_L`/`_R`), and let go of, slides to a standstill
  (`droid_skid_sprint_stop`) and idles. Running, it no longer skids at all. Whether it is
  sprinting goes by how fast it is going, not the keys held. The skids are made on the spot, and `data/droid_motion.json`, exported along with them,
  has where each takes the droid and how far round: the body follows that path, as far as it
  goes for how fast the droid went in, and the droid swings round with it.

## 22 September 2026

- **A droid to play as.** The player is seen from behind as a droid (`data/droid_full_deform.glb`),
  with the camera over its shoulder and pulled in when a wall is in the way. V switches to first
  person, and holding the middle mouse button swings the camera round the droid.
- **Its feet set the speed.** Walking, running, sprinting and crouching each play the droid's own
  cycle, and each gait goes as fast as that cycle's stride, so the feet stay on the floor. That
  makes every gait slower than before.
- **It faces the way it goes, and skids round.** The droid turns to face whichever way the keys
  send it, and stands idling when still. Turning round at a run or a sprint, it skids to a stop,
  swings round and sets off the other way.
- **It jumps, low or high.** Tap Space for a low jump, hold it for a high one; each press jumps
  once. From standing still the droid springs straight up and lands on the spot; on the move it
  leaps in its stride and lands running. It lands hard from a high jump or a long drop, lightly
  otherwise, and falling off an edge it flies and lands the same way.
- **Inhabitants.** Six more droids live in the maze, made from the same model. Each wanders off
  somewhere, keeping to the middle of the corridors, idles a while when it gets there, and sets
  off again. They are solid and make way for each other: walking, they veer to their right round
  whoever is ahead, and standing about, they step aside for anyone coming straight at them.
  `MAZE_INHABITANTS` sets how many.
- **Cover.** Tab puts the droid up against the wall ahead. A and D then slide it along the wall,
  facing the way it goes, as far as the wall's edge - the corner to take cover behind. Holding A
  or D on past the edge leans out round the corner; Ctrl no longer leans. Tab again, pushing away
  from the wall, or jumping leaves cover. Until there are cover animations (`droid_cover_idle`,
  `droid_cover_walk`, picked up once the model has them) it idles and walks as usual.
- **No more head bob** from behind. The camera is held steady; the droid's cycles show the stride.
- The droid is not yet in the traced shadows, which are gathered once and would leave its shadow
  where it started.

## 19 September 2026

- **Pause menu.** Escape now opens a menu with Resume, Lights, New maze and Quit, and stops the
  clock, the player and the physics. It used to only release the mouse. A round in play also
  pauses when the window loses focus.
- **Lights switch.** From the pause menu, the lamps, the glow of their fixtures, the sun and nearly
  all the ambient light can be turned off, leaving the flashlight to see by. The setting carries
  over to each new maze.
- **The end-of-round banner** is now centred on the window.
- **The code is split into modules** - `game`, `level`, `culling`, `fixtures`, `survey`, `hud`,
  `menu`, `diagnostics` and a `player/` folder - instead of one large `main.rs` and
  `player.rs`. Nothing about how the game plays changed with it.

## The engine, Hydroxus, compared with upstream Fyrox

Hydroxus's `vulkan` branch began as two commits on top of upstream Fyrox as of 13 September 2026:

1. **Make the wgpu (Vulkan) backend render like the OpenGL one.** Fixes found by rendering the
   same scenes on both backends and comparing the frames:
   - clip depth is remapped to wgpu's range;
   - textures sampled from projected positions (shadows, SSAO, decals, rendered cube maps) and
     UI rendered into textures are no longer upside down;
   - light volumes and bloom line up with the G-buffer;
   - a GPU hang from stale uniforms is fixed, and integer vertex attributes, pipeline caching,
     scissors, readback padding and rendering into mip levels are corrected;
   - uniform pages are capped at 1 MB, where a 2 GB limit reported by the driver froze loading.

   The `fyrox` crate also stops pulling in the OpenGL backend by default, so choosing
   `backend_wgpu` takes effect.
2. **Add hardware ray-traced shadows and further wgpu fixes.**
   - Hardware ray tracing, where the graphics card has it, traces every light's shadows. It is off
     unless a game asks for it.
   - Point and spot lights are drawn only over the part of the screen they can reach.
   - The soft-shadow filter no longer leaves a hard edge, on both backends.
   - With FXAA off, the frame is no longer upside down.
   - glTF meshes without a material come out plain white instead of dark white metal.
   - Animations saved by older engine versions load their property paths.

`VULKAN.md` in Hydroxus explains each change in detail.
