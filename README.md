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
