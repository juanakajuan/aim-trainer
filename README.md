# Aim Trainer

A native Linux aim trainer inspired by the core Kovaak's sandbox. Written in Rust with raylib, GLFW and OpenGL. Runs on CachyOS, including KDE Wayland through XWayland. No browser, web server, account or network connection is needed.

## Start

From the project directory, build and run:

```sh
./run.sh
```

The script runs `cargo run --release --locked` and passes arguments to the app. Cargo rebuilds when source files change.

To build only:

```sh
cargo build --release --locked
```

The standalone executable is `target/release/aim-trainer`. Fonts, sounds and shaders are built into it. You can move the executable without its source folder.

## Practice

| Scenario | Task |
| --- | --- |
| 1wall 6targets | Click six small spheres; each hit creates a new target. |
| Tile Frenzy | Click three large square tiles. |
| Micro Precision | Click five small targets in a narrow area. |
| Smooth Tracking | Hold fire and follow a sphere through smooth turns. |
| Close Strafes | Hold fire and react to short, random direction changes. |
| Target Switching | Hold fire; each target needs 0.3 seconds on target. |

Challenge runs last 60 seconds after a three-second countdown. Free play has no time limit and does not save a score. Pause and focus loss stop the clock. A stall longer than half a second also pauses the session.

Clicking score = hits × 100 × accuracy fraction. Tracking score = seconds on target × 100. Switching score = targets cleared × 100. Tracking and switching accuracy use time on target divided by time firing. Scores are independent local results; they do not match official leaderboards.

| Control | Action |
| --- | --- |
| Mouse | Aim |
| Left mouse | Fire; hold for tracking and switching |
| WASD | Move |
| Esc | Pause or resume; return from settings |
| R | Restart the current drill |
| Enter | Start a challenge from the scenario menu |
| F2 | Settings |
| F3 | Show or hide FPS and average frame time |
| F11 | Toggle borderless fullscreen |
| F12 | Save a screenshot to `~/Pictures/Aim Trainer/` |

## Settings and data

Click a numeric field, type the value, then press Enter. Settings include sensitivity, four sensitivity scales, mouse DPI, horizontal FOV, target and crosshair colors, crosshair size/gap, sound volume, fullscreen, VSync and frame limit. Changing the sensitivity scale converts the value to preserve turning speed within the allowed range. DPI is used only to calculate cm/360.

Sensitivity uses degrees per mouse count: Source/Quake `0.022`, Valorant `0.07`, Overwatch `0.0066`, Marvel Rivals `0.0175`. FOV is the actual horizontal angle at the current window aspect ratio. It is not a game-specific FOV scale.

GLFW enables raw mouse motion during capture if the system supports it. Aim has no smoothing or interpolation. The default frame limit is 360 FPS, with VSync off. Menu screens use a 60 FPS limit. The desktop compositor or driver can still limit presentation. The timing display measures frame intervals, not physical mouse-to-screen latency.

Existing data keeps the `aim-room` directory name to preserve saved settings and results.

- Settings: `$XDG_CONFIG_HOME/aim-room/settings.json`, default `~/.config/aim-room/settings.json`.
- Last 100 challenge results: `$XDG_STATE_HOME/aim-room/results.json`, default `~/.local/state/aim-room/results.json`.
- Writes use a temporary file and rename. Invalid values are rejected or reset to valid defaults. A save failure is shown in the app.

## Build and verify

Needs Rust 1.88 or later, CMake, a C compiler, Clang/libclang, OpenGL and X11 development libraries. These were available on the target CachyOS system. The built executable uses the system graphics and audio drivers.

```sh
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo build --release --locked
./target/release/aim-trainer --smoke-test artifacts/native-qa
```

The unit tests cover hit geometry, same-frame aim and fire, scores, challenge timing, pause, free play, sensitivity/FOV and disk saves. The native smoke check opens a real GPU window, drives all six drills with controlled inputs, captures screens and checks saved results after reload. It uses an isolated save directory and an accelerated simulation clock. It does not measure human aim or physical input latency.

## Scope and sources

This is an independent recreation of the basic training loop. The UI, room, sounds and scenario logic are local implementations. It has no Kovaak's assets, bundled source scenarios, Workshop, online leaderboard, scenario editor or advanced weapon simulation.

Reference: [Kovaak's sandbox and practice guidance](https://www.kovaak.com/fpsaimtrainer/), [sensitivity scales](https://www.kovaak.com/sensitivity-matcher/), and [GLFW raw mouse input](https://www.glfw.org/docs/latest/input_guide.html#raw_mouse_motion).

Code: MIT. DejaVu fonts: see `assets/FONT-LICENSE.txt`. raylib and GLFW retain their upstream licenses.

## Import a local KovaaK scenario

This first importer supports the embedded Pasu family represented by VT Pasu Intermediate S5. It does not support general KovaaK scenarios. Select **IMPORT .sce** in the scenario menu and choose a local file. A successful import selects the saved scenario and shows its limits. Cancel leaves your selection and saved data unchanged. Invalid files show an error. The native picker uses `kdialog` (KDE) or `zenity`; at least one must be installed. The app stays responsive while the picker is open.

For terminal import, close the app, then run:

```sh
./target/release/aim-trainer --inspect-scenario "/path/to/VT Pasu Intermediate S5.sce"
./target/release/aim-trainer --import-scenario "/path/to/VT Pasu Intermediate S5.sce"
./target/release/aim-trainer
```

Open **IMPORTED**, select the scenario, read the limits, then select **CHALLENGE** or **FREE PLAY**. Imports persist in `$XDG_STATE_HOME/aim-room/scenarios.json` (default `~/.local/state/aim-room/scenarios.json`). The source file is needed only for import. Only normalized settings and spawn positions are saved. No map meshes, assets, or complete source file are copied. Up to 32 imports are supported, with six per page. Reimporting identical text does not add another entry. Changed source text gets a separate identity, even if the name stays the same. Imported results and best scores are separate from the six built-in drills and from other imports.

Supported behavior:

- Embedded scenario, bot, character, dodge, weapon and melee profile references; bounded UTF-8 `.sce` input, at most 2 MiB. Missing/duplicate profiles, invalid values and unsupported behavior return an error before saving. The saved library is also validated on load.
- Four sphere targets, fixed player, 60-second challenge, source radius and spawn positions, target replacement after the source respawn delay. A target is not visible or hittable during that delay.
- Semi-auto, single-hit hitscan fire with the source shot interval. Pasu S5 uses 0.1 seconds between shots and 0.1 seconds before replacement.
- Source horizontal speed, acceleration, air friction, turn intervals and strafe pauses. Source jump velocity, air-jump count/velocity, jump chance/intervals, gravity and terminal speed drive vertical motion.
- Eight untargetable Knocker placements resolve their melee radius, horizontal/vertical impulse and interval. Their forces repel targets near the edges. They are not rendered as targets and do not score.
- Source kill points and optional accuracy or square-root-accuracy multiplier. Pasu S5 score is `kills × 10 × sqrt(hits / shots)`. For 4 kills and 8 shots, score is 28.284271. Full score precision is saved; parts of the UI show a rounded value.

Limits are shown before play. Physics are a local approximation: 240 Hz integration, gravity base 980, source units scaled by 0.00375, map locations scaled by `MapScale`, and forward/back dodge projected onto a fixed target plane. Jump frequency is treated as a chance per source jump interval. Melee overlap events become continuous forces with distance falloff. Safe bounds keep targets in view; crossing the lower bound resets air jumps. These bounds replace brush collision, not the Knocker forces. Source spawn exclusion, brushes, materials, visual/audio effects, and exact FOV scale conversion are not reproduced. The local horizontal FOV is clamped to the source range without changing saved settings. Source spawn offsets use the supported Pasu profile convention.

The compatibility table rejects unknown behavioral fields and changes to unmodeled scalar settings. Known cosmetic fields are ignored. This intentionally limits imports to the supported family; it is not a general parser that silently turns other drills into Pasu. Missing external profiles, projectiles, weapon chains, damage-based scores, finite lives, disabled Knocker abilities and non-unit or malformed spawn weights are rejected.

This does not claim exact KovaaK physics or leaderboard compatibility. No comparison with a running KovaaK engine was made. The map collision, spawn exclusion, gravity convention, jump probability, Knocker timing and FOV conversion can differ.

Native QA imports a small synthetic fixture by default. To test a local source file without changing normal saves:

```sh
./target/release/aim-trainer --smoke-test artifacts/pasu-qa --scenario "/path/to/VT Pasu Intermediate S5.sce"
```

Both native modes run all six built-in drills and the imported scenario. They save screenshots at two window sizes, normalized import settings and a result report. They check movement, target count, scores and reload. Synthetic tests check malformed inputs, atomic import, firing cooldown, respawn, Knocker force effects, and result identity. The complete source scenario is not a test fixture in this repository.
