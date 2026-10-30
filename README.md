# Ruptura Systematis

**A stealth-action game about droids, set in a neon city, written in Rust.** Talk your way past
the sentries, or rob the banks and outrun the drones. Get arrested, and pick the lock of your cell
to break out through the tunnels under the city.

[![Play in the browser](https://img.shields.io/badge/play-in%20the%20browser-27ff3f?style=for-the-badge)](https://d4140n-4h3-1.github.io/MazeGame-web/)
[![Rust](https://img.shields.io/badge/Rust-1.94%2B-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org)
[![Vulkan](https://img.shields.io/badge/Vulkan-ray%20traced-a41e22?style=for-the-badge&logo=vulkan)](#built-on)
[![License](https://img.shields.io/badge/license-MIT%20%2F%20CC%20BY--SA-blue?style=for-the-badge)](#license)

[![Talking to one of the droids (video)](media/talking_to_a_droid.jpg)](media/talking_to_a_droid.mp4)

*Talking to one of the droids. Click the picture to watch the video.*

**[▶ Play it in your browser](https://d4140n-4h3-1.github.io/MazeGame-web/)**: nothing to
install. All it needs is a desktop browser with WebGL 2. You can also build it yourself in one
command (see [Quick start](#quick-start)).

## What it is

You play as a droid in Urbs, a city of 200 other droids, with four more cities across its bridges.
Each droid goes about its own business. Every one of them can be talked to, and every one notices
what you do.

- **Stealth, as in Metal Gear.** Sentries see only what is in front of them, and they hear your
  footsteps. They go from CAUTION to ALERT to EVASION, search where they last saw you, call each
  other in, and cut you off. In the dark they see a third as far.
- **Talk, as in Fallout 3.** Each droid has its own lines, spoken aloud in System Latin by a voice
  synthesised from formants. Its mood colours the panel, its eyes and its voice. Pass a Speech
  check, bribe it, or intimidate it.
- **Witnesses, as in Red Dead Redemption 2.** A droid that sees you shoot someone runs for
  security. Stop it on the way and question it, bribe it, talk it out of what it saw, or send it
  home.
- **Crime and punishment.** Rob the thirteen stores and the banks by hacking their computers, as
  in Welcome to the Game. Get caught, and security takes you to a precinct and locks your pistol
  in the evidence locker. Serve your sentence, or pick the lock (rings of light, each drifting
  round a core), pry up the drain and escape through the tunnels. Down there is a gang's den, and
  a fence who will wipe your record clean for a price.
- **A pistol and a rifle.** Green energy bolts, held as in Call of Duty in first person. A bolt
  breaks a droid into pieces, with no blood, and the droid falls as a ragdoll.
- **Gangs.** They keep to groups of up to six, guard their turf and their hideouts, and regroup
  when they are broken up.
- **Five cities, played as one:**
  - **Urbs**, downtown towers and a canal;
  - **Ferrum**, a blast furnace, chimneys and a rail yard;
  - **Caelum**, a spaceport with rockets standing on their pads;
  - **Curia**, a capital in white stone and gold;
  - **Subura**, a neon slum of stacked containers.

  Maglev trains run between them, and the droids ride the trains too. The office towers have
  working lifts.
- **Moving as you would want to.** Walk, jog, run and sprint on stamina. Crouch and crawl. Jump
  low or high, take cover and lean round corners, and skid round when you turn at a sprint. The
  speed always matches what the droid's feet are doing.

### Game modes

| Mode              | What you do                                                                  |
| ----------------- | ---------------------------------------------------------------------------- |
| **Story**         | Cross Urbs and the cities round it: rob, hide, get arrested, break out.      |
| **Heist**         | Break into a district of a city floating in the void: Aurum, Cortex or Helix. |
| **Battle Royale** | Last droid standing on the Grid or in Nexus, cities traced in light.         |
| **Capture the Flag** | Hack red's firewall, take their flag and bring it home, with allies at your side. |
| **Maze**          | A new random maze each round, built from tiles: find the way out.            |

### Under the hood

- **Ray-traced shadows** from every lamp, and from everything that glows, on any graphics card
  with hardware ray tracing. Shadow maps are the fallback, and the browser version uses them.
- **Its own engine fork.** [Hydroxus](https://github.com/d4140n-4h3-1/Hydroxus) is
  [Fyrox](https://github.com/FyroxEngine/Fyrox) made to render properly on Vulkan, with hardware
  ray tracing added.
- **Big cities at a high frame rate.** Each city is gathered into tiles, with far blocks drawn
  from simplified copies. That took Urbs from about 21 ms a frame to 7.
- **The AI and the voices are crates of their own.**
  [hydroxus-ai](https://github.com/d4140n-4h3-1/hydroxus-ai) covers sight, hearing, alert phases
  and pathfinding over 130 million cells.
  [hydroxus-voice](https://github.com/d4140n-4h3-1/hydroxus-voice) covers formant speech and
  echoes.
- **Every model is built by a script**, from Python run in Blender. The droid, its animations,
  the guns, the trains and the cities can all be rebuilt from nothing.

What's new: see **[CHANGELOG.md](CHANGELOG.md)**, the whole history, day by day.

## Quick start

```sh
git clone https://github.com/d4140n-4h3-1/RupturaSystematis.git
cd RupturaSystematis
cargo run --release
```

## Requirements

- **A graphics card with Vulkan.** The game checks at startup that it is rendering with Vulkan,
  and exits if it is not. Ray tracing is used for shadows when the card supports it; without it
  the game falls back to shadow maps.
- **Rust 1.94 or newer**, the version the engine requires.
- **`git`, and a network connection for the first build.** Cargo downloads the engine and its
  effects by itself, straight from GitHub, so all there is to get is this project:

  ```sh
  git clone https://github.com/d4140n-4h3-1/RupturaSystematis.git
  cd RupturaSystematis
  cargo run
  ```

  The engine, [Hydroxus](https://github.com/d4140n-4h3-1/Hydroxus), is about 400 MB. `.cargo/config.toml` has Cargo fetch it with the system `git`
  rather than its own library, which can resume an interrupted download instead of starting it
  over. `Cargo.lock` pins the commits used; `cargo update` moves to the latest of each branch.

  The engine is modified, so upstream Fyrox will not do: Hydroxus is a fork of Fyrox whose
  [`vulkan` branch](https://github.com/d4140n-4h3-1/Hydroxus/tree/vulkan) makes the wgpu backend
  render like the OpenGL one and adds the hardware ray tracing that the traced shadows use. Its
  crates keep Fyrox's names, so the game still says `fyrox`. `VULKAN.md` in Hydroxus describes
  every change.
  [`fyrox-gfx`](https://github.com/d4140n-4h3-1/fyrox-gfx) holds the graphics effects the game
  adds on top of the engine, [`hydroxus-ai`](https://github.com/d4140n-4h3-1/hydroxus-ai) the
  droids' and drones' AI, and [`hydroxus-voice`](https://github.com/d4140n-4h3-1/hydroxus-voice)
  their voices.

This project is a Cargo workspace of its own. That keeps other crates from switching on the
engine's OpenGL backend, which it would otherwise pick over Vulkan.

## Running

```sh
cargo run
```

It can be started from anywhere; it finds its models in `data/` next to `Cargo.toml`. The engine
is compiled with optimizations even in a debug build, and the game itself is not, so `cargo run`
is quick enough to play while still easy to debug. `cargo run --release` optimizes the game as
well.

It opens on the main menu, to pick which game to play:

- **Story**: Urbs and the cities round it.
- **Heist**: break into Aurum, Cortex or Helix.
- **Battle Royale**: the Grid or Nexus.
- **Maze**: find the way out of a new random maze each round.
- **Capture the Flag**: you are blue. Pick a map - Lanes (`data/arena/ctf_map.glb`) or
  Balconies (`data/arena/ctf_balconies.glb`), Escape going back - then hack the computer in red's
  base to take down the firewall round their flag, take it, and bring it home to your own, with
  blue's droids and drone on your side and red's against you.

With `MAZE_MODEL` set, it skips the menu and plays that model.

### In a browser

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version <the wasm-bindgen version in Cargo.lock>
./web/build.sh
python3 -m http.server -d site
```

`web/build.sh` builds the game to WebAssembly and puts it in `site/` with the page and `data/`,
ready to serve as it is; open http://localhost:8000 and press Play. In a browser the game draws
with WebGL 2, the one graphics API every browser has, and so shadows come from shadow maps: no
browser offers ray tracing. The published copy lives in
[MazeGame-web](https://github.com/d4140n-4h3-1/MazeGame-web), which serves `site/` on GitHub
Pages.

## Controls

| Key              | Action                                                        |
| ---------------- | ------------------------------------------------------------- |
| W A S D, arrows  | Move                                                          |
| Mouse            | Look around                                                   |
| Caps Lock        | Walk or jog; it stays as you left it                          |
| Shift (tap)      | Walk or run; it stays as you left it. Running costs stamina   |
| Shift (hold)     | Sprint while held. Costs stamina faster than running; running out leaves you walking |
| Space            | Jump: tap for a low jump, hold for a high one                 |
| C                | Crouch, or stand back up                                      |
| Z                | Crawl, or stand back up                                       |
| Q (hold)         | Look behind you while still moving forward                    |
| Tab              | Take cover against the wall ahead, or leave it                |
| A / D in cover   | Slide along the wall; hold on past its edge to lean round it  |
| F                | Flashlight on or off (it starts off)                          |
| V                | Third person (behind the droid) or first person               |
| Middle mouse (hold) | Swing the camera round the droid to see it from any side    |
| Right mouse (hold) | Strafe: keep facing ahead whichever way you go; anything faster than a walk is a jog |
| R                | Draw the pistol, or holster it                                |
| X, mouse wheel   | Change to the rifle, or back to the pistol: the one out is put away and the other drawn |
| Left mouse       | Draw the pistol; once it is drawn, fire                       |
| Right mouse, drawn | Raise the pistol to aim, rather than hold it at the ready   |
| E                | Talk to the droid close by and in front of you, or use the computer in front of you |
| H                | In Urbs, with security after you: put your hands up and go quietly, for a shorter sentence |
| E, held in a cell | At the firewall: pick its lock (with a key card, open it). At a guard facing you: ask its price to look away, then E again to pay. At a guard with its back to you, close by: lift its key card. At a loose panel, or the grate over the drain: pry it, a little at a time. At the drain, its grate up: drop down into the escape tunnels |
| Lock: W / S      | Pick a ring of the lock, further out or in                    |
| Lock: A / D (hold) | Turn the picked ring; held, it stops drifting               |
| Lock: Space      | Latch the ring, once its gap is in the slot at the top - it lights up yellow |
| Lock: Tab        | Give up picking it                                            |
| Talking: mouse, W / S, arrows | Pick a reply; E, Enter, Space or a click says it  |
| Talking: 1 to 9  | Say that reply                                                |
| Talking: Tab     | Walk away                                                     |
| Computer: Enter  | Start a breach, or try again once denied                      |
| Computer: typing | Type the command shown, exactly                               |
| Computer: Tab    | Walk away                                                     |
| N                | New maze                                                      |
| `[` `]`          | Turn slower or faster                                         |
| `-` `=`          | Narrower or wider view                                        |
| Escape           | Pause                                                         |

### Pause menu

Escape pauses the game: the clock, the player, the physics and every sound all stop, sounds
carrying on from where they were once it is resumed. Switching to another
window in the middle of a round pauses it too. The menu has:

- **Resume**: carry on where you left off.
- **Lights**: switch the maze's lights off, or back on. Off, the lamps, the glow of their
  fixtures, the sun and nearly all the ambient light go out, leaving your flashlight and the
  exit's own glow. The setting carries over to each new maze.
- **Options**: switch the subtitles of what the droids say on or off, the System Latin and the
  English each by itself. With only the English on, it is as big as the System Latin would be.
  Back, or Escape, returns to the menu.
- **Start again**: the same as N: a new maze, or a new round in a fixed maze model or capture the
  flag.
- **Main menu**: leave the game under way for the main menu, to pick another.
- **Quit**.

Every menu - this one, its options, the main menu and its maps - works from the keyboard as well
as the mouse: the arrow keys, or W and S, go up and down the buttons, the one picked in light
blue, and Enter or Space presses it. Start again is passed over while there is nothing to start
again.

## Options

Options are set with environment variables, for example `MAZE_SIZE=10x10 cargo run`. In the
browser, add them to the page's address instead:
`https://d4140n-4h3-1.github.io/MazeGame-web/?MAZE_SIZE=10x10&MAZE_SEED=7`. What `MAZE_DEBUG`
logs goes to the browser's console there.

| Variable                 | Effect                                                                  |
| ------------------------ | ----------------------------------------------------------------------- |
| `MAZE_SIZE=<w>x<d>`      | How many junctions wide and deep the maze is. The default is `20x20`.   |
| `MAZE_SEED=<n>`          | Makes every maze and round the same, for comparing two runs.            |
| `MAZE_MODEL=<path>`      | Plays a fixed maze model (`.glb`, `.gltf` or `.fbx`) instead of random mazes. |
| `MAZE_INHABITANTS=<n>`   | How many droids live in the maze. The default is 6.                     |
| `MAZE_ROYALE=<n>`        | How many play battle royale, you included, from 2 to 16. The default is 16. |
| `MAZE_DEBUG=1`           | Logs the walkable map of each level, and rendering statistics once a second. |
| `MAZE_WINDOWED=1`        | Opens the game in a window instead of filling the screen.               |
| `MAZE_AUDIO_LATENCY_MS=<n>` | How late sound reaches you, in milliseconds. Subtitles wait that long after a voice starts. On Linux the game works it out itself, guessing for Bluetooth earphones, which the sound server cannot measure. |
| `MAZE_VSYNC=0`           | Uncaps the frame rate, for measuring what a frame costs.                |
| `MAZE_RT=0`              | Shadow maps instead of ray-traced shadows.                              |
| `MAZE_HARD_SHADOWS=1`    | Ray-traced shadows with sharp edges instead of soft ones.               |
| `MAZE_SHADOW_BUDGET=0`   | With shadow maps, gives every lamp in range one, not just the nearest.  |
| `MAZE_SSAO=0`            | Turns ambient occlusion off.                                            |
| `MAZE_REFLECTIONS=0`     | Turns floor reflections off.                                            |
| `MAZE_KNOCKDOWN=<s>`     | That many seconds into a round, shoots down the droid nearest you, to try the ragdolls out. |
| `MAZE_ARREST=<s>`        | In Urbs, that many seconds into a round, arrests you where you stand, to try jail out; with `MAZE_PICK=1` as well, starts picking your cell's lock. |
| `MAZE_TUNNELS=<s>`       | In Urbs, that many seconds into a round, takes you down the first hatch into the escape tunnels. |
| `MAZE_TUNNELS_UP=<s>`    | With `MAZE_TUNNELS`, that many seconds after going down, brings you back up through the same hatch. |
| `MAZE_GRATE=<s>`         | With `MAZE_ARREST`, that many seconds after the arrest, pries up your cell's grate and drops you down the drain into the escape tunnels. |
| `MAZE_HEIST_OPEN=<n>`    | In a heist, the first n doors' computers count as hacked, to try it out. |
| `MAZE_SHIELD=<s>`        | That many seconds into a round, raises your shield, as the 1 key does. |
| `MAZE_DRONE_ALARM=<s>`   | That many seconds into a round, sends the drone after you as if the alarm had sounded, to try it out. |
| `MAZE_DISMEMBER=<parts>` | With `MAZE_KNOCKDOWN`, breaks those parts off it too, such as `head,forearm.L,shin.R`. |
| `MAZE_COMPUTER=1`        | Puts you at the computer, using it, as soon as it is placed; `breach` starts a breach too, and `look` only puts you in front of it. |
| `MAZE_TERMINAL_OVERLAY=1` | Shows the computer's terminal over its screen, as a panel of the game's interface, instead of on the screen itself. |

### Fixed maze models

A model given with `MAZE_MODEL` needs no special structure, but the game reads a few things from
it:

- Surfaces colored **pure magenta** (255, 0, 255) are the glass of light fixtures. They become
  refractive glass, and each fixture gets a lamp.
- Small meshes standing on their own, narrower than 3.5 m, are taken as **something to find**:
  the round ends there instead of at a random exit.
- FBX models are taken to be in centimeters and scaled down.
- Ground is only walkable with a **ceiling** somewhere above it, and the floor has to sit at or
  above the game's own safety floor, whose top is at -0.05 m.
- **Floors up above** are walkable where stairs lead up to them from the ground, each step no
  more than 0.55 m: a spot keeps one floor, the highest with 2 m of headroom, so make raised
  floors solid down to the ground rather than leaving room under them.

Anything in the model that glows by itself goes dark with the lights.

`data/arena/combat_map.glb` is one such model: an indoor arena of hexagonal pillars and
rectangular cover blocks, for trying out combat. Play it with

```sh
MAZE_MODEL=data/arena/combat_map.glb cargo run
```

`examples/team_battle.rs` plays the same arena as a battle between two teams of droids, red and
cyan, five a side, all of them AI: they fight from cover, fall back when hurt and come back at
their own end when downed, and the first team to 30 kills wins. The camera is free to fly about
the arena or follow one droid (Tab).

```sh
cargo run --release --example team_battle
```

`data/arena/ctf_map.glb` is an arena for capture the flag, 84 by 52 m. Red's base is at the -x
end and cyan's at the +x end. Each base is a walled room with its team's colour round the walls
and doorways. Its flag stands on a pad in the middle, and there are three ways in: a front door
off the yard, and a door in each side wall off a corridor round the back. Between the bases, two
long walls with doorways through them split the field into three lanes. The middle lane is open
round a central tower, and each side lane is narrower, with a low wall across its middle. Both
halves of the map are the same, mirrored, with low and tall cover and pillars throughout.
`data/arena/ctf_map.py` builds it in Blender (`blender -b --python data/arena/ctf_map.py`).
It is what **Capture the Flag** on the main menu plays, as `MAZE_MODEL` does too: you start at
blue's end, take red's flag and bring it home to blue's.

The model marks where the game puts things with empties. `flag_red` and `flag_blue` get that
side's flag, `data/ctf/flag_red.glb` or `data/ctf/flag_blue.glb`, standing in a firewall,
`data/ctf/firewall.glb`, turned the way the empty's +x points. `computer_red` and `computer_blue`
get the computer that opens that firewall, its screen facing along the empty's +x; without one,
the computer goes against the wall nearest the firewall. `post_red_1`, `post_red_2` and on, and
`post_blue_1` and on, are each side's droids' posts, the first droid at the first; a droid with
no post keeps to its flag. The firewall's orange shell stands in the
way until its computer is hacked, then goes, and the flag can be taken from beside its plinth;
taken, it is carried on the taker's back.
The shell is see-through glass that tints what is behind it orange, and its glow and its light
flicker. The team battle fights in it with

```sh
BATTLE_MAP=data/arena/ctf_map.glb cargo run --release --example team_battle
```

## Tests

```sh
cargo test
```

The tests cover maze generation and tile fitting, what can be seen from where, the walkable grid
and round planning, the player's movement, breath, head motion, leaning and keys, the ragdoll and
how a droid comes apart, and the computer's hack and where it goes.

## How it fits together

| Module          | What it does                                                               |
| --------------- | -------------------------------------------------------------------------- |
| `main.rs`       | Starts the engine and sets up the graphics effects.                        |
| `game.rs`       | The game: loading levels, rounds, input, the pause menu, the lights.       |
| `level.rs`      | A level in the scene: its pieces, collider, lamps and walkable ground.     |
| `generate.rs`   | Plans random mazes on a grid of junctions and turns them into tiles.       |
| `tiles.rs`      | Measures the tile models and assembles a maze from them.                   |
| `layout.rs`     | Where a round starts and ends, on `hydroxus-ai`'s walkable grid.           |
| `survey.rs`     | Finds the walkable ground of a level by casting rays into it.              |
| `culling.rs`    | Hides the pieces and lamps that cannot be seen from where the player is.   |
| `fixtures.rs`   | Light fixtures: the glass, the lamps, and everything that glows.           |
| `inward.rs`     | Makes the tiles' surfaces visible from inside and out.                     |
| `hud.rs`        | The status line, the banner, the alert at the top, the stamina bar and the pistol's ammo. |
| `hearts.rs`     | The hearts: where they go in each maze, and their floating and spinning.   |
| `drone.rs`      | The security drone: its patrol, the alarm, searching and firing, its glow and mood. |
| `credits.rs`    | Credits, the maze's money: amounts in hundredths, how they are written, and what a computer carries. |
| `drone_shot.rs` | The drone's shots: their flight, what they hit, and their glow.            |
| `menu.rs`       | The pause menu.                                                            |
| `computer.rs`   | The computer to hack: where it goes, the hack, and its terminal.           |
| `ragdoll.rs`    | A droid gone limp: its bodies and joints, from `data/droid_motion.json`.   |
| `dismember.rs`  | Breaking a droid apart: its pieces and broken ends, and the voxels that spill. |
| `dialogue/`     | Talking to the droids: their conversations from `data/dialogue/`, and the panel they are shown in. |
| `diagnostics.rs`| The Vulkan check and the rendering statistics.                             |
| `formants/`     | Sounds made from formants: the file format, the synthesizer that makes the pistol's sounds from `data/sounds/`, the droids' speech, and the drones' beeps (`chirps`). |
| `player/`       | The player, one file per part: posture, movement, breath, head, lean, input, view, the droid (`avatar`), the camera behind it (`third_person`) and its close-up when talking (`talk`). |

The tile models, the droid (`droid_full_deform.glb`) and the computer (`computer.glb`) are in
`data/`, along with where the droid's skids take it, its ragdoll and how it comes apart
(`droid_motion.json`), and the terminal's font (`fonts/`, DejaVu Sans Mono, under its own
license in `fonts/DejaVuSansMono.LICENSE`).

## Built on

### Hydroxus

[Hydroxus](https://github.com/d4140n-4h3-1/Hydroxus) is a fork of Fyrox. Its `vulkan` branch makes
the wgpu (Vulkan) backend render like the OpenGL one and adds hardware ray-traced shadows. What it
changes from upstream is in [CHANGELOG.md](CHANGELOG.md#the-engine-hydroxus-compared-with-upstream-fyrox)
and, in full, in `VULKAN.md` in Hydroxus.

### fyrox-gfx

Graphics effects the game adds on top of the engine, kept out of it: refractive glass, softer
shadow edges, temporal anti-aliasing, further-reaching ambient occlusion, screen-space
reflections, a budget that keeps shadow maps for the nearest lamps only, the ray-traced
shadows (its `raytracing` feature), and surfaces drawn from both sides for the maze's tube-shaped
pieces.

### hydroxus-ai

The droids' and drones' AI, in a crate of its own
([hydroxus-ai](https://github.com/d4140n-4h3-1/hydroxus-ai)) that knows nothing of the engine
but its vectors: the walk grid and the routes over it, what an NPC sees and hears, Metal Gear's
alert phases, making way in a corridor, getting about over more than one floor and by ferry,
joining a chase, pacing it, taking a pistol pointed at it, and walking about in pairs and groups.
The game casts the rays, moves the droids and plays
their animations; the crate decides what they make of what is round them.

### hydroxus-voice

The voices, in a crate of their own
([hydroxus-voice](https://github.com/d4140n-4h3-1/hydroxus-voice)) that knows nothing of the
engine: sounds made from formants, System Latin read aloud by the droids and chirped by the
drones, and the city's echo. The game reads the JSON that describes them and plays the samples
the crate makes.

## Changelog

Every change, day by day, is in **[CHANGELOG.md](CHANGELOG.md)**.

## License

Copyright (c) 2026 VulpesPhantasma ([d4140n-4h3-1](https://github.com/d4140n-4h3-1) on GitHub).

The code is under the MIT license (`LICENSE-MIT`). The models, animations and other files in
`data/` are under Creative Commons Attribution-ShareAlike 4.0 International (`LICENSE-CC-BY-SA`):
share and adapt them, crediting VulpesPhantasma (d4140n-4h3-1), with anything made from them
shared under the same license.
