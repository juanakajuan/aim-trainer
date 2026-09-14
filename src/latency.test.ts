import { test } from "node:test";
import assert from "node:assert/strict";
import { LatencyMeter, eventTime } from "./latency";
import { captureMouse } from "./pointer-lock";

test("measures every input to submission, not just the newest event or frame interval", () => {
  const m = new LatencyMeter();
  m.beginFrame(100);
  m.recordInput(101, 103, 0);
  m.recordInput(105, 106, 0);
  m.submitted(107, 110);
  const s = m.snapshot(110);
  assert.deepEqual(s.input, { mean: 7, p95: 9, samples: 2 });
  assert.deepEqual(s.dispatch, { mean: 1.5, p95: 2, samples: 2 });
  assert.deepEqual(s.cpu, { mean: 3, p95: 3, samples: 1 });
  assert.equal(s.frame, null);
  m.beginFrame(116);
  m.submitted(116, 118);
  assert.equal(m.snapshot(118).fps, 62.5);
  assert.equal(m.snapshot(118).input?.samples, 2);
});
test("invalid and incomparable timestamps are omitted; epoch times are normalized", () => {
  assert.equal(eventTime(1_700_000_000_100, 105, 1_700_000_000_000), 100);
  for (const t of [0, -1, NaN, Infinity, 106])
    assert.equal(eventTime(t, 105, 0), null);
});
test("stale input disappears and pause/reset cannot produce bogus latency", () => {
  const m = new LatencyMeter();
  m.recordInput(100, 101, 0);
  m.submitted(102, 103);
  assert.equal(m.snapshot(2104).input, null);
  m.recordInput(2200, 2201, 0);
  m.reset();
  m.submitted(5000, 5001);
  assert.equal(m.snapshot(5001).input, null);
  assert.equal(m.snapshot(5001).frame, null);
});
test("input bursts are bounded and overflow is disclosed", () => {
  const m = new LatencyMeter();
  for (let i = 0; i < 3000; i++) m.recordInput(100, 101, 0);
  m.submitted(102, 103);
  const s = m.snapshot(103);
  assert.equal(s.input?.samples, 2048);
  assert.equal(s.droppedSamples, 952);
});
test("raw input requested with fallback only for unsupported raw input", async () => {
  const calls: (PointerLockOptions | undefined)[] = [];
  assert.equal(
    await captureMouse({
      requestPointerLock: async (options) => {
        calls.push(options);
      },
    }),
    "raw-requested",
  );
  assert.deepEqual(calls, [{ unadjustedMovement: true }]);
  const fallbackCalls: (PointerLockOptions | undefined)[] = [];
  assert.equal(
    await captureMouse({
      requestPointerLock: async (options) => {
        fallbackCalls.push(options);
        if (options) throw new DOMException("unsupported", "NotSupportedError");
      },
    }),
    "standard",
  );
  assert.deepEqual(fallbackCalls, [{ unadjustedMovement: true }, undefined]);
  let attempts = 0;
  await assert.rejects(
    captureMouse({
      requestPointerLock: async () => {
        attempts++;
        throw new DOMException("denied", "SecurityError");
      },
    }),
    { name: "SecurityError" },
  );
  assert.equal(attempts, 1);
});
