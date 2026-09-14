# Aim Room

A desktop browser aim trainer inspired by the basic KovaaK’s sandbox loop.

## Run

```sh
npm install
npm run dev
```

Open the local URL. Use a desktop mouse and a browser with WebGL and pointer lock support.

## Features

- Six 3D drills: 1wall 6targets, Tile Frenzy, Micro Precision, Smooth Tracking, Close Strafes, Target Switching.
- 60-second challenges with a three-second countdown; unlimited free play.
- Mouse camera control, center crosshair, raycast hits and WASD movement.
- Escape pauses; Resume captures the mouse again; R restarts.
- Source/Quake sensitivity scale, horizontal FOV, target color, crosshair size and hit sound volume.
- Local scores and last 100 results. Free play does not save results.

Click drills: one click per shot, score = hits × 100 × accuracy. Tracking: hold fire, score = seconds on target × 100. Switching: hold fire for 0.3 seconds per target, score = targets cleared × 100. Tracking accuracy is time on target divided by time firing.

This is an independent implementation. It does not use KovaaK’s assets or exact scenario files. It does not include workshop content, online leaderboards, scenario editing, or all game sensitivity profiles. Browser input and performance can differ from the native game.

## Checks

```sh
npm test
npm run build
```

Tests cover score calculations, FOV conversion, external storage validation, raycast hits, tracking, switching, countdown, pause/resume, challenge completion and free play. Engine checks use a simulated DOM and renderer; they do not verify GPU output or physical mouse input. Build checks strict TypeScript and produces `dist/` for static hosting.

## Low-latency mode and measurements

The renderer requests the high-performance GPU, disables anti-aliasing, and defaults to one rendering pixel per CSS pixel. On a device pixel ratio of 2, this renders one quarter as many pixels as the previous default. Change **Settings → Render scale** (0.5–1.5) to trade clarity for GPU work. The HUD and crosshair remain at browser resolution.

Raw mouse movement is requested, with standard pointer lock as a fallback when raw input is unsupported. “Raw input requested” is deliberate: some browsers silently ignore that option. Mouse look has no smoothing or interpolation. Shots use current camera and target transforms, including input received between renders. The render loop reuses vectors and raycast arrays, reuses targets on restart, submits the scene before HUD work, and avoids rendering unchanged menu/pause frames. The display updates twice per second; the score HUD updates at 20 Hz. Audio requests the interactive latency mode.

**F3** toggles the live timing panel during play. This also disables timing collection when hidden. The preference is saved when mouse capture ends. A checkbox is available in Settings.

- **Input → render submit:** browser event timestamp to the return of the first renderer call containing that event. Includes event dispatch wait, wait for the next frame, game work and CPU render submission. All delivered mouse movement/button and movement-key-down samples are counted, not just the newest event in each frame.
- **Input p95:** 95% of recent input samples are at or below this value.
- **Event dispatch:** browser event timestamp to entry into the input handler.
- **Frame avg / p95 and FPS:** measured intervals between active frame callbacks. These are not input latency or confirmed display presentation times.
- **CPU to submit:** frame callback entry to renderer return. This excludes the later HUD/telemetry work and does not wait for the GPU.

Statistics use samples from the last two seconds, bounded to 4096 input samples and 2048 frame samples. More than 2048 input events between submissions are counted as omitted. No input shows a dash after samples expire. Start/resume clears samples; pause freezes the last values. Invalid/incomparable event timestamps are ignored. Browser timestamp precision and event coalescing limit accuracy.

These are **software measurements, not total mouse-to-screen latency**. They exclude mouse hardware/polling before event creation, GPU completion/queueing, the browser compositor, scanout and panel response. Use an external latency tester or a high-speed camera to measure total latency. Do not use FPS or these values to claim parity with a native game. Rendering remains scheduled by the browser; there is no portable browser switch to disable VSync.

Validation: the integration test sends a mouse flick and click between renders and checks that the first shot uses the new aim. Timing tests cover known event/submit intervals, p95, stale samples, reset, timestamp validation and overflow. Raw input tests cover unsupported-option fallback and permission errors. Tests do not measure GPU or physical mouse latency.
