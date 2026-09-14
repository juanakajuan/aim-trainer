import { test } from "node:test";
import assert from "node:assert/strict";
import {
  accuracy,
  scoreFor,
  horizontalToVertical,
  parseSettings,
  parseResults,
  defaults,
} from "./core";
test("click score rewards hits and penalizes missed shots", () => {
  assert.equal(scoreFor("click", 10, 10, 0), 1000);
  assert.equal(scoreFor("click", 10, 20, 0), 500);
  assert.equal(scoreFor("click", 0, 0, 0), 0);
  assert.equal(accuracy(0, 0), 0);
});
test("tracking score depends on time on target", () => {
  assert.equal(scoreFor("track", 0, 0, 12.5), 1250);
  assert.equal(scoreFor("track", 0, 0, 60), 6000);
  assert.equal(accuracy(12.5, 25), 50);
});
test("FOV preserves horizontal angle across screen ratios", () => {
  for (const aspect of [1, 16 / 9, 21 / 9]) {
    const v = horizontalToVertical(103, aspect);
    const recovered =
      (2 * Math.atan(Math.tan((v * Math.PI) / 360) * aspect) * 180) / Math.PI;
    assert.ok(Math.abs(recovered - 103) < 1e-8);
  }
});
test("stored settings reject invalid external data", () => {
  assert.deepEqual(parseSettings(null), defaults);
  assert.deepEqual(
    parseSettings({
      sensitivity: NaN,
      fov: 999,
      volume: -1,
      crosshair: Infinity,
      color: "<script>",
    }),
    defaults,
  );
  assert.equal(parseSettings({ sensitivity: 2.5 }).sensitivity, 2.5);
});
test("stored results reject malformed data and cap retained history", () => {
  const r = {
    scenario: "six",
    score: 100,
    accuracy: 100,
    hits: 1,
    shots: 1,
    duration: 60,
    date: "2026-09-14T00:00:00Z",
  };
  assert.deepEqual(
    parseResults([
      r,
      { ...r, score: "100" },
      { ...r, score: Infinity },
      { ...r, date: "bad" },
      { ...r, scenario: "unknown" },
    ]),
    [r],
  );
  assert.equal(parseResults(Array.from({ length: 120 }, () => r)).length, 100);
});
