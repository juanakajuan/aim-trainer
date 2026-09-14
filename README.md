# Aim Room

A native Linux aim trainer inspired by the core Kovaak's sandbox. Written in Rust with raylib, GLFW and OpenGL. Runs on CachyOS, including KDE Wayland through XWayland. No browser, web server, account or network connection is needed.

## Start

Open **Aim Room** from the application menu after installation, or run:

```sh
./run.sh
```

The standalone executable is `target/release/aim-room`. Fonts, sounds and shaders are built into it. You can move the executable without its source folder.

To build and install for your user:

```sh
./scripts/install.sh
```

This installs `~/.local/bin/aim-room` and an application menu entry. It does not need root access.

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
| F12 | Save a screenshot to `~/Pictures/Aim Room/` |

## Settings and data

Click a numeric field, type the value, then press Enter. Settings include sensitivity, three sensitivity scales, mouse DPI, horizontal FOV, target and crosshair colors, crosshair size/gap, sound volume, fullscreen, VSync and frame limit. Changing the sensitivity scale converts the value to preserve turning speed within the allowed range. DPI is used only to calculate cm/360.

Sensitivity uses degrees per mouse count: Source/Quake `0.022`, Valorant `0.07`, Overwatch `0.0066`. FOV is the actual horizontal angle at the current window aspect ratio. It is not a game-specific FOV scale.

GLFW enables raw mouse motion during capture if the system supports it. Aim has no smoothing or interpolation. The default frame limit is 360 FPS, with VSync off. Menu screens use a 60 FPS limit. The desktop compositor or driver can still limit presentation. The timing display measures frame intervals, not physical mouse-to-screen latency.

- Settings: `$XDG_CONFIG_HOME/aim-room/settings.json`, default `~/.config/aim-room/settings.json`.
- Last 100 challenge results: `$XDG_STATE_HOME/aim-room/results.json`, default `~/.local/state/aim-room/results.json`.
- Writes use a temporary file and rename. Invalid values are rejected or reset to valid defaults. A save failure is shown in the app.

## Build and verify

Needs Rust 1.88 or later, CMake, a C compiler, Clang/libclang, OpenGL and X11 development libraries. These were available on the target CachyOS system. The built executable uses the system graphics and audio drivers.

```sh
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo build --release --locked
./target/release/aim-room --smoke-test artifacts/native-qa
```

The six unit tests cover hit geometry, same-frame aim and fire, scores, challenge timing, pause, free play, sensitivity/FOV and disk saves. The native smoke check opens a real GPU window, drives all six drills with controlled inputs, captures screens and checks saved results after reload. It uses an isolated save directory and an accelerated simulation clock. It does not measure human aim or physical input latency.

## Scope and sources

This is an independent recreation of the basic training loop. The UI, room, sounds and scenario logic are local implementations. It has no Kovaak's assets, exact scenario files, Workshop, online leaderboard, scenario editor or advanced weapon simulation. The old browser implementation remains in Git history.

Reference: [Kovaak's sandbox and practice guidance](https://www.kovaak.com/fpsaimtrainer/), [sensitivity scales](https://www.kovaak.com/sensitivity-matcher/), and [GLFW raw mouse input](https://www.glfw.org/docs/latest/input_guide.html#raw_mouse_motion).

Code: MIT. DejaVu fonts: see `assets/FONT-LICENSE.txt`. raylib and GLFW retain their upstream licenses.
