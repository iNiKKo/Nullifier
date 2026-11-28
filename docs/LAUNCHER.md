# The launcher

One matrix, two launchers. `launcher/profiles.json` is the only place settings
live; `launcher/nullifier-launch.sh` (Linux/macOS) and
`launcher/nullifier-launch.ps1` (Windows) both read it. If you find yourself
typing `VICTORY_*` into a shell to play, either the profile is wrong and should
be fixed, or you are testing and should use `tools/bench.sh`.

```bash
./launcher/nullifier-launch.sh --list       # what profiles exist
./launcher/nullifier-launch.sh --detect     # what this machine gets, and why
./launcher/nullifier-launch.sh --game DIR   # play
```

```powershell
.\launcher\nullifier-launch.ps1 -List
.\launcher\nullifier-launch.ps1 -Detect
.\launcher\nullifier-launch.ps1 -Game "D:\games\title"
```

## The graphical launcher

`tools/launcher` is a small native front end (egui; Windows and Linux, one
binary) for the things the console's own menu never had:

```bash
./Nullifier.sh                                            # builds it once, then opens it
```

Windows: double-click `Nullifier.bat` (it builds `tools\launcher\target\release`
once if needed). Both wrappers are in the repository root; the command line
below is what they run.

```bash
cargo build --release --manifest-path tools/launcher/Cargo.toml
./tools/launcher/target/release/nullifier-launcher
```

* **First run** — a welcome screen: drop the game folder on the window (or use
  the folder picker), and Nullifier checks for `game_scarlett_master.exe` (or
  any `.exe`, two folders deep). If the engine has not been built yet, one
  **Set up now** button runs `tools/setup.sh` in the background with a live
  tail, plain progress text and a Cancel button. No terminal is needed.
* **Play** — the last run's numbers separate **in-game** frames (a few hundred
  draws or more) from menu/loading frames: the heading is the in-game median,
  with the menu's rate beside it. The chosen game has a green "game found", the measured settings
  for this card, and one big **Play**. **Change…** reopens the picker,
  **Settings…** opens Rendering, and the three plain-language presets
  (Recommended, Low memory, Safe) plus the last run's numbers sit below. The
  numbers behind a preset are in Rendering. The GUI runs the same
  `nullifier-launch.sh`/`.ps1` as the command line, so there is one source of
  truth.
* **Simple by default** — the rail shows Play and Graphics only.
  **Advanced settings** at the bottom reveals Rendering, Advanced and Tools and
  is remembered. Anything that needs the technical tabs (Settings…, Customise…)
  reveals them itself.
* **Controls** — an **Input mode**: **Auto** (a connected pad wins, the
  keyboard stands in when there is none), **Controller only** (the keyboard and
  mouse look are ignored) or **Mouse & keyboard** (a pad is ignored). Auto is
  the default and is right for almost everyone. The page lists the pads the
  system reports (Xbox, DualSense, DualShock) with **Check again**, and the
  keyboard bindings used while no pad is connected. Apply writes
  `VICTORY_KEYBOARD` for the next run; Accept is a normal key name (`Enter`,
  `Space`, `F1`, or one letter). Sticks and triggers stay W/A/S/D, I/J/K/L, Z/C.
  **Test input** shows the last key you pressed and which pad button it fires,
  the live mouse movement and the right-stick values it produces, so bindings
  can be checked before starting the game. **Mouse look** is opt-in: while the
  game has focus and no pad is connected, the pointer is locked to the game
  window and hidden, its movement turns the right stick, and (by default) the
  left button is the right trigger (fire) and the right button the left trigger
  (aim); sensitivity, inversion and the buttons are on the same page. The host
  is read every 8 ms, so the stick is a short average of the hand's speed
  rather than one 8 ms slice; the average is held for as long as the title
  takes between reads (100–450 ms), so a fast flick still reaches a title that
  renders at 3 fps. Menu navigation with the mouse is
  not included. The mode reaches the engine as
  `VICTORY_INPUT=auto|controller|keyboard`, the buttons can be turned off with
  `VICTORY_MOUSE_BUTTONS=0`. If it does nothing, the game log says why every
  five seconds (`mouse look idle` with the reason).
* **Community** — each run records the **translation layer** version (the
  title window's `v0.012` build string) and the launcher version separately;
  a nickname, **where** in the game the run was and **notes**
  (pop-in, flicker, HUD, crashes), then **Share last run**: it adds a local
  entry and copies a JSON block. Each entry keeps the Nullifier version, title
  build, profile, fps, frame and GPU time, draws, the full set of active
  settings, the GPU/OS and the sanitized log. The list has two-line rows with
  a fixed Name/Version/FPS/Where table, search, a **Filters** menu (game build,
  area, version, notes only) and newest/fastest sorting; the log window shows
  the settings and hardware too. A row with settings gets **Use**: it applies
  that run's settings temporarily and Play offers **Revert / Keep**. **Refresh
  board** reads the published `community/leaderboard.json` from `main` (raw
  GitHub, no token or server) and marks those rows **GitHub**;
  local entries stay in `~/.local/share/nullifier/community.json`.
  `NULLIFIER_BOARD_URL` points at another copy for testing.
* **Rendering** — Nullifier's own rendering settings: the title's render scale
  and **Title frame rate** first, then memory and copies, the draw path
  (including **Water's tiled light**), and timing — in plain words.
  Every switch has a small round `?` that explains it, and every number shows
  examples (`2048, 3072 or 4096 (MB)`) under its description. Resolution,
  V-Sync and the frame cap are on Graphics; the active resolution is repeated
  in amber on Play because it changes how the picture looks.
* **Warnings** — values that cannot work on this machine are called out in a
  yellow panel: a texture budget above the card's VRAM, world memory over half
  of it, 2+ frames prepared ahead, joined copies, relay 2, an output wider than
  16:9 (the title's HUD is laid out for 16:9; see `docs/FINDINGS.md` F.1), an
  unknown switch.
* **Graphics** — the title's own graphics settings, parsed from its
  `settings.xml` (88 values: HDR, gamma, anisotropic filtering, FSR
  quality/sharpen, LOD scales, shadows, motion blur, water reflections,
  resolution, frame cap). **Resolution, V-Sync and Frame Rate Limit are applied
  by Nullifier**, not by the title, because the title ignores its own values:
  they set `VICTORY_OUTPUT`, `VICTORY_VSYNC` and `VICTORY_FPS_CAP` immediately
  (V-Sync Adaptive behaves as On; the layer has no adaptive present mode).
  `VICTORY_OUTPUT` takes the title's own sizes (720p, 1664×936, 1080p, 1440p,
  2160p) and the 21:9 and 32:9 sizes it has no mode for (2560×1080, 3440×1440,
  3840×1080, 5120×1440); the host presents at whatever size the title's own
  setter is given. The title's HUD and menus are designed for 16:9 — see
  `docs/FINDINGS.md` F.1 for what an ultrawide size was checked to do.
  The rest are the title's settings. Values with a small named set are
  drop-downs (anisotropic 4×/8×/16×, FSR quality, quality presets, water
  reflection sizes), whole-number ranges are sliders, and 0–1 values show
  percentages. Only **gamma** is confirmed to change the picture; values
  measured to change nothing (frame rate, draws and copy-back) are marked
  **no effect**: among them motion blur, ray-traced shadows, `lodScale`,
  `grassLod`, FSR quality, the quality preset and the particle/alpha shadow
  switches. **Apply** writes the file and keeps `settings.xml.bak` beside it;
  **Discard edits** rereads the file, **Restore backup** puts the last backup
  back, and **Reset to defaults** moves the file aside (`settings.xml.reset`
  keeps a copy) so the game writes fresh defaults the next time it starts. The
  game must be closed while changing them; it rewrites this file on exit, so
  discard after playing.
* **Advanced** — the exact set in an aligned table (setting, help, value,
  red **Remove** on the right); **Add a setting…** is a menu of known
  switches, custom KEY=VALUE is collapsed; environment variables are explained
  and the profile is a readable on/off/value list with **Full note** folded
  away.
* **Log** — the footer shows the last lines and `Show log`/`Hide log` folds it
  away; while open it also has `Clear`, `Open log` and the detail level.
* **Tools** — scan the hardware, choose how much the log says, take still
  screenshots at chosen frame numbers, and a 20-second benchmark run by the
  launcher itself (the same code on Windows and Linux): it loads the game,
  warms up 6 s, measures 20 s, and reports median, p10, frames and crashes
  with a progress bar. `Measure…` on Play runs it too.

The version in the rail is checked once a day against the newest GitHub
release, or against the newest tag when no release is published (cached in
`~/.cache/nullifier/update.txt`; set `NULLIFIER_NO_UPDATE_CHECK=1` to skip it).
Tag releases as `vX.Y.Z`. A newer release adds a small blue **● update** next to the
version; clicking it opens the releases page. There is no silent update.

The GUI writes an exact profile to `~/.cache/nullifier/launcher-profile.json`
and passes it with `VICTORY_PROFILES`, forcing it by name. That is why the
launchers honour `VICTORY_PROFILES` at all: it lets a front end pass a set,
including switching an opt-in off by omission.

`Esc` returns to Play from any page. `--tab
play|graphics|controls|community|rendering|advanced|tools` starts on that page.

`--check` prints the resolved profile, the exact environment and the command
it would run, without launching. `--tab play|game|graphics|advanced|debug`
opens on that tab. The game settings also work headless:

```bash
./nullifier-launcher --game-list                     # section/key=value
./nullifier-launcher --game-set graphics/gamma=15    # writes, keeps a .bak
```

**Inert on purpose:** FSR upscaling and ray-traced light are shown but
disabled; each needs engine work (see `docs/ROADMAP.md`). The title's own
settings that changed nothing in the 2026-11-27 measurements are marked **no
effect** in the page (`docs/FINDINGS.md` 3.29c).

The crate is its own workspace: `cargo clippy --workspace` at the root does
not build it, and CI shared with the layer needs no GUI libraries.

## What it detects

| | Linux / macOS | Windows |
|---|---|---|
| OS | `uname -s` | always `windows` |
| Vendor | `lspci` VGA/3D line | `Win32_VideoController` name, discrete adapter preferred |
| VRAM | `/sys/class/drm/*/mem_info_vram_total` | `nvidia-smi` for NVIDIA, then `HardwareInformation.qwMemorySize` (two registry roots), then AdapterRAM if it is at least 1024 MB |

macOS reports `vendor=apple` and exits with "not supported yet" — see
`docs/ROADMAP.md`. Intel is matched as `intel` but only by the `*-other`
profiles until someone measures it.

## Selection rules

1. `--profile NAME` (or `-Profile`) wins.
2. Otherwise the **first** profile in `profiles.json` whose `match` applies:
   `os` and `vendor` may be `"*"`, plus optional `vram_min_mb` / `vram_max_mb`.
3. `--safe` and `--diagnostics` are applied on top (see `overrides`).
4. **Anything already exported in the environment wins over the profile**, so
   you can A/B a single switch without editing the file.

`--detect` prints the profile, every variable it set, the confidence level and
the note explaining *why* those numbers.

On Linux the vendor is that of the first display card `lspci` lists. The video
memory is the largest among cards of that vendor: from `nvidia-smi` for
NVIDIA, whose driver has no `mem_info_vram_total`, and from that file under
`/sys/class/drm` for the others. `./tools/check-detect.sh` checks this
against stand-ins for both commands.

On Windows the vendor is the first NVIDIA card WMI lists, then AMD, then
Intel, so an integrated card does not decide it on a laptop; the memory comes
from `nvidia-smi` for NVIDIA (WMI's `AdapterRAM` is a uint32 that caps) and
from the driver key in the registry otherwise. `./tools/check-detect.ps1`
checks this against a stand-in list (`-GpuJson`) on any machine, card or no
card.

## Adding or changing a profile

A profile is only allowed in if it has evidence:

```json
{
  "name": "linux-nvidia-16gb",
  "match": { "os": "linux", "vendor": "nvidia", "vram_min_mb": 13000 },
  "confidence": "measured",
  "sets": { "VICTORY_TEXTURE_NO_WAIT": "1", "VICTORY_LATE_WAIT": "1" },
  "note": "RTX 4080, Mesa x.y, median 1.4 fps over 3 runs"
}
```

* `confidence` is one of `measured` (you ran `tools/bench.sh` here),
  `reported` (from another person's bench row), `extrapolated` (scaled from a
  measured one) or `untested`.
* Keep the array ordered **most specific first**; selection is first-match.
* Put the measurement in `docs/FORK.md` too. A profile without a row is a
  guess and must say `untested`.

## The wineserver (Linux)

Wine's server kept a process's views in a list and walked it at every map and
unmap. The title keeps tens of thousands of views and remaps about a thousand
a second, so the server ran at a whole core and the frame waited behind it
(`docs/FINDINGS.md` 2.5). Wine fixed it on 2026-11-25 (commit 3217449b1f71);
Proton Experimental 11.0-20261112 does not have that commit.

```bash
./tools/build-wineserver.sh           # once, and again after Proton updates
./tools/build-wineserver.sh --check   # what is installed, builds nothing
```

It fetches Valve's Wine at the commit your Proton was built from, applies that
one commit (`tools/wine/server-view-tree.patch`), builds only `wineserver`,
checks that it speaks the same protocol as Proton's own, and installs it in
`~/.local/share/nullifier/wineserver`. Nothing of Proton's is changed. It needs
git, gcc, make, autoconf, bison, flex, perl and python3, and about 600 MB
under `~/.cache/nullifier`.

The launcher then starts that server for its prefix before it starts wine
(wine itself only starts the server that lies beside it). `--detect` says
which server a run will get, and the log's `wineserver:` line says which one
it got.

| | |
|---|---|
| `VICTORY_WINESERVER=0` | Proton's own server, for a comparison |
| `VICTORY_WINESERVER=PATH` | start this server binary instead |
| `VICTORY_WINESERVER_DIR` | where the built server is (default `~/.local/share/nullifier/wineserver`) |

* A server built for another Proton version is not used: the launcher says
  so and Proton's own serves. Run the build again.
* A Proton whose own server has the tree needs none of this; the build script
  and `--detect` say so.
* Only Valve's Proton is looked up by itself. For another build pass
  `--wine-commit` and `--wine-repo`; if its server speaks another protocol the
  script stops and installs nothing.
* A wineserver stays for 3 s after its last program. The launcher waits for
  one left by the run before, so two runs in a row with different servers do
  not share one.

Measured on an RTX 5080, six runs in turn: 3.55 / 3.45 / 3.50 fps with the
built server, 0.60 / 0.65 / 0.70 with Proton's own.

## Host wave width

`VICTORY_HOST_WAVE=32` runs every shader at 32 lanes even on a card that allows
64; unset uses the card's maximum. On a 64-lane card this puts the console's
wave64 compute programs in two host halves, and it costs frame rate. It is the
comparison to make when mouths or teeth look wrong on AMD: a picture that is
right at `32` and wrong at `64` points at the 64-lane form of a program. Both
forms add the face network's sums of 16-bit floats in the console's order
(`CS_TransposeVecMatMul*`). Before issue #61's fix for 64-lane cards, the
64-lane form added such a sum in single precision and rounded it once, where
only the lanes inside the matrix reach it.

## The title's own 30 fps cap

The title's graphics settings hold a frame rate of 30, and its own limiter and
timers keep to it: menus are a flat 33.3 ms. `VICTORY_FRAME_INTERVAL_MS=<ms>`
patches the three reads of that setting in the title (bytes checked; another
build is left alone) and makes Nullifier's frame clock tick at the same
interval. Rendering → **Title frame rate** offers 60, 120, 144 and Unlimited,
and is **off by default** (the title keeps 30). `VICTORY_TITLE_FPS=<rate>` gives
the title another rate outright, `off` leaves the title alone while our clock
still changes.

Measured here: with `VICTORY_FRAME_INTERVAL_MS=16.667`, all three title reads
patch and the menus run at 60.0 fps instead of 30.0. The world only gains where
it can render faster than 33 ms; on this machine it cannot, but on an RTX card
it is the difference between the title's 30 cap and 26–30 rendered.

## Waiting for a pipeline

`VICTORY_PIPELINE_WAIT_ONCE` (on by default): a host pipeline is waited for at
most one wait budget over the whole run, not one budget in every frame that
draws with it. A driver compile of tens of seconds (or one that never ends)
therefore stalls one frame, not the whole time it is in view; its draws are
dropped until it is made. `VICTORY_PIPELINE_WAIT_ONCE=0` waits as before.

## Water light (32-lane hosts)

Two tiled water-light programs (`5dc3b25986622949`, `6f225b641c5b0708`) of the
title run on a 32-lane host in a wave-wide form that leaves whole tiles of open
water dark — around boats, along shores, under bridges — and cost about 15 ms
of the card a frame on an RTX 5070. They are **left out by default** on such a
host: the water is darker and bluer but has no blocks. `VICTORY_WATER_LIGHT=1`
runs them again; Rendering → Draw path → **Water's tiled light** is the same
switch.

## Display output and render scale (experimental)

`VICTORY_OUTPUT=1280x720|1664x936|1920x1080|2560x1440|3840x2160` patches the
title's own screen-size block (byte-checked; another build is left alone) and
`VICTORY_RENDER_SCALE=1` uses the title's own upscaler at 1.0 instead of its
2/3. Graphics → Display sets the output; Rendering sets the render scale. The title still applies
its own scale, so output 1920x1080 with the default scale renders at 1280x720.

Measured on the opening scene here, 1024 MB card buffers:

| output | render scale | render size | median |
|---|---|---|---|
| 3840x2160 | 2/3 (default) | 2560x1440 | 5.5 fps |
| 2560x1440 | 2/3 | 1707x960 | 6.7 fps |
| 1920x1080 | 2/3 | 1280x720 | 7.1–7.4 fps |
| 1920x1080 | 1.0 | 1920x1080 | 6.1 fps |

The 1080p/720p combination is softer and its materials looked worse in play;
the title's HUD and menus are authored for 4K and were reported wrong at a
smaller output, so **native output stays the default**. The 1080p pause menu
and a native-scale 1080p prologue looked correct in captures here; the in-game
HUD at a smaller output has not been checked yet. Do not ship this on by
default until that is answered.

## Shaders and pipelines

Compiled host pipelines are kept in a D3D12 pipeline library under
`%LOCALAPPDATA%\nullifier\cache\pipelines` (inside the prefix on Linux), and
the title pipelines a run drew are listed so the next run starts them while the
title loads — about 2,700 on the current cache, and the world spends 0.0 ms a
frame waiting for a pipeline. Nothing has to be preloaded before a normal run.

For a cold cache (first install, or after a shader change) **Play → Shader
cache → Preload** translates the title's shaders from its archives and builds
their compute pipelines **without starting the game** (`warm`), then stops. It
takes about a minute and writes the same library and list a normal run would;
the first real session then starts warm. It is optional: without it the first
run compiles the same pipelines during its own loading. The run itself never
waits on this.

## Safety rules the launcher enforces

* `LATE_WAIT >= 2`, `JOIN_COPIES=1` and `RELAY` are not defaulted on Linux:
  on AMD (RADV) they remove the device (`0x887A0005`) within minutes. One
  profile is an exception, with its evidence in `docs/FORK.md`:
  `linux-nvidia-16gb` sets `LATE_WAIT=3` and `RELAY=2` with
  `RELAY_QUEUE=1`, measured on an RTX 5080 with the built wineserver.
  `--safe` turns all send-ahead off if a run still dies.
* vkd3d-proton below build **5742** segfaults RADV on the first world mesh
  pipeline. The launcher installs the newest one it can find (Proton
  Experimental) into the prefix once, keeps `.bak` copies, and records the
  build in `.vkd3d-build`.
* The prefix is created through Proton itself. A bare `wineboot` produces a
  prefix without `d3dcompiler_47.dll`/`wined3d.dll` and the title will not load.
* If a local LLM (`ollama`, `llama-server`, `assistant-brain`) is running, it
  warns: `-ngl 99` models take 11+ GB of a 16 GB card and push the title into
  GTT, costing roughly 10× frame time.

## Troubleshooting

| Symptom | Do |
|---|---|
| `device removed 0x887A0005` | `--safe` |
| Black screen, UI and audio fine | the log has `no argument clamp`; the build is missing PR #127 |
| Exit 3 right after a mesh pipeline | vkd3d-proton too old; check `--detect` shows `prefix vkd3d >= 5742` |
| `no profile applies` | add/repair a profile, or pass `--profile` |
| Runs but very slow | check for an LLM, then `--detect` and compare VRAM |
| Under 1.5 fps on Linux with `wineserver` at a whole core | `--detect` says `wineserver  Proton's own, which searches a list for every view`: run `tools/build-wineserver.sh` |
| `version mismatch` from wine at start | a server of another Wine is serving the prefix: stop it (`wineserver -k`), and build again if Proton was updated |
