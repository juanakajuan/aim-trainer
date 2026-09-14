import { test } from "node:test";
import assert from "node:assert/strict";
import { Window } from "happy-dom";
import * as THREE from "three";
import {
  Trainer,
  type Snapshot,
  type ArenaRenderer,
  type PerformanceSnapshot,
} from "./engine";
import { scenarios, defaults, type Result } from "./core";

test("training loop: capture, countdown, raycast hit, pause, resume, finish and free play", async () => {
  const window = new Window();
  const canvas = window.document.createElement("canvas");
  window.document.body.append(canvas);
  let nextFrame: FrameRequestCallback = () => {};
  let now = 0;
  let captured: unknown = null;
  Object.defineProperty(window.document, "pointerLockElement", {
    get: () => captured,
  });
  Object.defineProperty(canvas, "requestPointerLock", {
    value: async () => {
      captured = canvas;
    },
  });
  Object.defineProperty(window.document, "exitPointerLock", {
    value: () => {
      captured = null;
      window.document.dispatchEvent(new window.Event("pointerlockchange"));
    },
  });
  const globals: Record<string, unknown> = {
    window,
    document: window.document,
    HTMLInputElement: window.HTMLInputElement,
    HTMLSelectElement: window.HTMLSelectElement,
    innerWidth: 1280,
    innerHeight: 720,
    devicePixelRatio: 2,
    performance: { now: () => now, timeOrigin: 0 },
    requestAnimationFrame: (f: FrameRequestCallback) => {
      nextFrame = f;
      return 1;
    },
    cancelAnimationFrame: () => {},
    AudioContext: class {
      resume(): Promise<void> {
        return Promise.resolve();
      }
    },
  };
  for (const [key, value] of Object.entries(globals))
    Object.defineProperty(globalThis, key, {
      value,
      configurable: true,
      writable: true,
    });
  let state: Snapshot = {
    phase: "idle",
    time: 60,
    score: 0,
    accuracy: 0,
    hits: 0,
    countdown: 3,
    free: false,
  };
  const results: Result[] = [];
  let targetMeshes: THREE.Mesh[] = [];
  let camera: THREE.Camera | undefined;
  let renderCount = 0;
  let pixelRatio = 0;
  const measurements: PerformanceSnapshot[] = [];
  const renderer: ArenaRenderer = {
    setPixelRatio: (ratio) => {
      pixelRatio = ratio;
    },
    setClearColor: () => {},
    setSize: () => {},
    dispose: () => {},
    render: (scene, view) => {
      renderCount++;
      now += 1; // One millisecond of simulated render submission.
      scene.updateMatrixWorld(true);
      view.updateMatrixWorld(true);
      camera = view;
      targetMeshes = scene.children.filter(
        (child): child is THREE.Mesh =>
          child instanceof THREE.Mesh &&
          child.geometry instanceof THREE.SphereGeometry,
      );
    },
  };
  const scenario = scenarios[0];
  assert.ok(scenario);
  // The DOM implementation is validated at runtime, then passed through its real HTML interface.
  const inputCanvas: unknown = canvas;
  function isCanvas(value: unknown): value is HTMLCanvasElement {
    return value instanceof window.HTMLCanvasElement;
  }
  assert.ok(isCanvas(inputCanvas));
  const trainer = new Trainer(
    inputCanvas,
    scenario,
    { ...defaults, volume: 0 },
    (s) => {
      state = s;
    },
    (r) => results.push(r),
    (message) => assert.fail(message),
    () => renderer,
    (measurement) => measurements.push(measurement),
  );
  assert.equal(pixelRatio, 1, "avoid automatic high-DPI supersampling");
  function advance(seconds: number): void {
    for (let i = 0; i < Math.ceil(seconds / 0.02); i++) {
      now += 20;
      nextFrame(now);
    }
  }
  trainer.start(false);
  await new Promise<void>((resolve) => setImmediate(resolve));
  advance(3.1);
  assert.equal(state.phase, "running");
  assert.equal(targetMeshes.length, 6);
  const target = targetMeshes[0];
  assert.ok(target);
  assert.ok(camera);
  // Establish a previous rendered frame with no target under the old crosshair.
  targetMeshes.forEach((mesh, i) => {
    mesh.position.set(4 + i * 2, 2, -3);
    mesh.updateMatrixWorld(true);
  });
  camera.rotation.set(0, 0, 0);
  camera.updateMatrixWorld(true);
  // Then deliver a real mouse event and click without an intervening render.
  target.position.set(3, 2, -3);
  const flick = new window.MouseEvent("mousemove");
  Object.defineProperties(flick, {
    movementX: {
      value:
        Math.atan2(3, 12) / ((defaults.sensitivity * 0.022 * Math.PI) / 180),
    },
    movementY: { value: 0 },
    timeStamp: { value: now - 2 },
  });
  window.document.dispatchEvent(flick);
  window.document.dispatchEvent(
    new window.MouseEvent("mousedown", { button: 0 }),
  );
  window.document.dispatchEvent(
    new window.MouseEvent("mouseup", { button: 0 }),
  );
  advance(0.1);
  assert.equal(state.hits, 1);
  assert.equal(state.score, 100);
  assert.equal(state.accuracy, 100);
  advance(0.6);
  const inputMeasurement = measurements.find((m) => m.input !== null);
  assert.ok(
    inputMeasurement?.input,
    "live telemetry receives actual mouse events",
  );
  assert.ok(inputMeasurement.input.mean >= 2);
  assert.ok(inputMeasurement.cpu && inputMeasurement.cpu.mean >= 1);
  trainer.pause();
  const pausedRenderCount = renderCount;
  const remaining = state.time;
  advance(2);
  assert.equal(state.phase, "paused");
  assert.equal(state.time, remaining);
  assert.equal(
    renderCount,
    pausedRenderCount,
    "paused scene does not consume GPU frames",
  );
  trainer.resume();
  await new Promise<void>((resolve) => setImmediate(resolve));
  advance(60);
  assert.equal(state.phase, "results");
  assert.equal(results.length, 1);
  assert.equal(results[0]?.duration, 60);
  trainer.menu();
  assert.equal(state.phase, "idle");
  trainer.start(true);
  await new Promise<void>((resolve) => setImmediate(resolve));
  advance(65);
  assert.equal(state.phase, "running");
  assert.equal(state.free, true);
  assert.ok(state.time > 60);
  assert.equal(results.length, 1);
  trainer.menu();
  for (const id of ["smooth", "switch"]) {
    const drill = scenarios.find((s) => s.id === id);
    assert.ok(drill);
    trainer.select(drill);
    trainer.start(false);
    await new Promise<void>((resolve) => setImmediate(resolve));
    advance(3.1);
    window.document.dispatchEvent(
      new window.MouseEvent("mousedown", { button: 0 }),
    );
    for (let i = 0; i < 25; i++) {
      const moving = targetMeshes[0];
      assert.ok(moving);
      assert.ok(camera);
      camera.lookAt(moving.position);
      camera.updateMatrixWorld(true);
      advance(0.02);
    }
    window.document.dispatchEvent(
      new window.MouseEvent("mouseup", { button: 0 }),
    );
    advance(0.1);
    assert.ok(state.score > 0, `${id} must score sustained fire`);
    assert.ok(state.accuracy > 90);
    trainer.menu();
  }
  trainer.dispose();
  await window.happyDOM.close();
});
