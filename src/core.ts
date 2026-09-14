export type Mode = "click" | "track" | "switch";
export interface Scenario {
  id: string;
  name: string;
  mode: Mode;
  category: string;
  description: string;
  count: number;
  radius: number;
  speed: number;
}
export const scenarios: readonly Scenario[] = [
  {
    id: "six",
    name: "1wall 6targets",
    mode: "click",
    category: "Static clicking",
    description:
      "Six small targets on one wall. Click each target once. Build precise flicks and clean stops.",
    count: 6,
    radius: 0.34,
    speed: 0,
  },
  {
    id: "frenzy",
    name: "Tile Frenzy",
    mode: "click",
    category: "Static clicking",
    description:
      "Three large targets. Move fast between them and keep each click under control.",
    count: 3,
    radius: 0.65,
    speed: 0,
  },
  {
    id: "micro",
    name: "Micro Precision",
    mode: "click",
    category: "Static clicking",
    description:
      "Five tiny targets in a tight area. Focus on small corrections and first-shot accuracy.",
    count: 5,
    radius: 0.18,
    speed: 0,
  },
  {
    id: "smooth",
    name: "Smooth Tracking",
    mode: "track",
    category: "Tracking",
    description:
      "Hold fire and follow a moving target. Stay on target through long, smooth strafes.",
    count: 1,
    radius: 0.55,
    speed: 1.5,
  },
  {
    id: "reactive",
    name: "Close Strafes",
    mode: "track",
    category: "Tracking",
    description:
      "Hold fire on a fast target that changes direction. React without overcorrecting.",
    count: 1,
    radius: 0.65,
    speed: 3.3,
  },
  {
    id: "switch",
    name: "Target Switching",
    mode: "switch",
    category: "Switching",
    description:
      "Hold fire to clear moving targets. Each target takes 0.3 seconds of accurate fire.",
    count: 4,
    radius: 0.45,
    speed: 0.7,
  },
];
export interface Settings {
  sensitivity: number;
  fov: number;
  volume: number;
  crosshair: number;
  color: string;
  renderScale: number;
  showLatency: boolean;
}
export const defaults: Settings = {
  sensitivity: 1,
  fov: 103,
  volume: 0.25,
  crosshair: 6,
  color: "#f04452",
  renderScale: 1,
  showLatency: true,
};
export interface Result {
  scenario: string;
  score: number;
  accuracy: number;
  hits: number;
  shots: number;
  duration: number;
  date: string;
}
export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
function bounded(
  v: unknown,
  min: number,
  max: number,
  fallback: number,
): number {
  return typeof v === "number" && Number.isFinite(v) && v >= min && v <= max
    ? v
    : fallback;
}
export function parseSettings(v: unknown): Settings {
  if (!isRecord(v)) return { ...defaults };
  return {
    renderScale: bounded(v.renderScale, 0.5, 1.5, 1),
    showLatency: typeof v.showLatency === "boolean" ? v.showLatency : true,
    sensitivity: bounded(v.sensitivity, 0.1, 10, 1),
    fov: bounded(v.fov, 60, 130, 103),
    volume: bounded(v.volume, 0, 1, 0.25),
    crosshair: bounded(v.crosshair, 2, 14, 6),
    color:
      typeof v.color === "string" && /^#[0-9a-f]{6}$/i.test(v.color)
        ? v.color
        : defaults.color,
  };
}
export function parseResults(v: unknown): Result[] {
  if (!Array.isArray(v)) return [];
  return v
    .filter(
      (x: unknown): x is Result =>
        isRecord(x) &&
        typeof x.scenario === "string" &&
        scenarios.some((s) => s.id === x.scenario) &&
        ["score", "accuracy", "hits", "shots", "duration"].every(
          (k) => typeof x[k] === "number" && Number.isFinite(x[k]) && x[k] >= 0,
        ) &&
        typeof x.date === "string" &&
        Number.isFinite(Date.parse(x.date)),
    )
    .slice(-100);
}
export function horizontalToVertical(fov: number, aspect: number): number {
  return (
    (2 * Math.atan(Math.tan((fov * Math.PI) / 360) / aspect) * 180) / Math.PI
  );
}
export function accuracy(hits: number, shots: number): number {
  return shots > 0 ? Math.min(100, (hits / shots) * 100) : 0;
}
export function scoreFor(
  mode: Mode,
  hits: number,
  shots: number,
  trackingSeconds: number,
): number {
  return Math.round(
    mode === "track"
      ? trackingSeconds * 100
      : hits * 100 * (accuracy(hits, shots) / 100),
  );
}
