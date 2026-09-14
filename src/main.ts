import "./style.css";
import { Trainer, type Snapshot, type PerformanceSnapshot } from "./engine";
import {
  scenarios,
  parseSettings,
  parseResults,
  isRecord,
  type Result,
  type Settings,
} from "./core";
function element<T extends HTMLElement>(
  selector: string,
  type: { new (): T },
): T {
  const el = document.querySelector(selector);
  if (!(el instanceof type)) throw new Error(`Missing element: ${selector}`);
  return el;
}
function read(key: string): unknown {
  try {
    return JSON.parse(localStorage.getItem(key) ?? "null");
  } catch {
    return null;
  }
}
let storageWorks = true;
function save(key: string, value: unknown): void {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    storageWorks = false;
  }
}
let settings = parseSettings(read("aim-settings"));
let results = parseResults(read("aim-results"));
let selected = scenarios[0];
if (!selected) throw new Error("No scenarios");
let filter = "All";
let activeTab = "scenarios";
let lastResult: Result | null = null;
const app = element("#app", HTMLDivElement);
app.innerHTML = `<canvas id="arena" aria-label="3D aim training room"></canvas>
<div id="hud" hidden><div class="hud-name"><span id="hud-mode">CHALLENGE</span><strong id="hud-name"></strong></div><div class="hud-stats"><div><small>SCORE</small><b id="score">0</b></div><div><small>TIME</small><b id="time">1:00</b></div><div><small>ACCURACY</small><b id="accuracy">0%</b></div></div><span class="hud-help">R Restart &nbsp; F3 Latency &nbsp; ESC Pause</span></div><div id="crosshair" hidden><i></i><b></b></div><div id="countdown" hidden></div>
<aside id="latency-panel" hidden aria-label="Live software latency"><div class="latency-heading">LATENCY <span id="input-mode">Move mouse to measure</span></div><div class="latency-main"><b id="latency-mean">—</b><span>ms<br>input → render submit</span></div><dl><div><dt>Input p95</dt><dd id="latency-p95">—</dd></div><div><dt>Event dispatch</dt><dd id="latency-dispatch">—</dd></div><div><dt>Frame avg / p95</dt><dd id="latency-frame">—</dd></div><div><dt>CPU to submit</dt><dd id="latency-cpu">—</dd></div><div><dt>FPS</dt><dd id="latency-fps">—</dd></div></dl><p>Software only. Excludes mouse hardware, GPU completion and display delay.</p><small id="latency-samples">Move or click to collect input samples.</small><small>F3 show / hide · Esc pause</small></aside>
<main id="menu"><header><a class="brand" href="/" aria-label="Aim Room home"><span class="brand-mark">⌖</span><span>AIM<span class="brand-light">ROOM</span><small>AIM TRAINER</small></span></a><nav aria-label="Main"><button class="nav active" data-tab="scenarios">Scenarios</button><button class="nav" data-tab="settings">Settings</button><button class="nav" data-tab="history">Statistics</button></nav><span class="local-label">LOCAL TRAINING</span></header>
<section id="scenarios-view" class="view"><div class="section-heading"><div><p class="eyebrow">SANDBOX</p><h1>Choose your next challenge.</h1></div><span class="count-label">06 SCENARIOS</span></div><div class="workspace"><section class="browser-panel"><div class="browser-tools"><label class="search">⌕ <input id="search" placeholder="Search scenarios" aria-label="Search scenarios"></label><div class="filters" aria-label="Scenario category">${["All", "Clicking", "Tracking", "Switching"].map((x) => `<button data-filter="${x}" class="${x === "All" ? "active" : ""}">${x}</button>`).join("")}</div></div><div class="list-head"><span>SCENARIO</span><span>PERSONAL BEST</span></div><div id="scenario-list"></div><div class="list-footer">Built-in scenarios <span>Scores saved on this device</span></div></section><aside class="detail-panel"><div class="preview"><div class="preview-tag" id="preview-tag"></div><div class="target-art"><i></i><i></i><i></i><i></i><i></i><i></i></div><span>60 SECOND CHALLENGE</span></div><div class="detail-body"><p class="eyebrow" id="detail-category"></p><h2 id="detail-title"></h2><p id="detail-description"></p><div class="detail-metrics"><div><small>TARGETS</small><strong id="target-count"></strong></div><div><small>PERSONAL BEST</small><strong id="best"></strong></div></div><button id="challenge" class="primary">▶ &nbsp; Play challenge</button><button id="freeplay" class="secondary">Free play <span>No time limit</span></button><p class="control-note" id="control-note"></p></div></aside></div></section>
<section id="settings-view" class="view" hidden><p class="eyebrow">PREFERENCES</p><h1>Make it feel right.</h1><div class="settings-panel"><h2>Mouse & view</h2><label class="setting"><span>Sensitivity<small>Source / Quake scale · 0.022° per count at 1.0</small></span><input id="sensitivity" type="number" min="0.1" max="10" step="0.05"></label><label class="setting"><span>Horizontal field of view<small>Actual horizontal degrees at your screen ratio</small></span><input id="fov" type="number" min="60" max="130" step="1"></label><h2>Performance</h2><label class="setting"><span>Render scale<small>1.0 = one pixel per CSS pixel. Lower values reduce GPU work.</small></span><input id="render-scale" type="number" min="0.5" max="1.5" step="0.1"></label><label class="setting"><span>Live latency display<small>F3 toggles this during play.</small></span><input id="show-latency" type="checkbox"></label><p class="performance-note">Raw mouse input is requested automatically. Anti-aliasing is off. The display measures browser event time to render submission, not total mouse-to-screen latency. Values use recent samples from the last 2 seconds; p95 is the value that 95% of samples do not exceed.</p><h2>Crosshair & sound</h2><label class="setting"><span>Crosshair size</span><input id="crosshair-size" type="range" min="2" max="14" step="1"></label><label class="setting"><span>Target color</span><input id="target-color" type="color"></label><label class="setting"><span>Hit sound volume</span><input id="volume" type="range" min="0" max="1" step="0.05"></label><button id="reset-settings" class="secondary">Reset to default</button><p id="saved-message" role="status">Changes save automatically on this device.</p></div></section>
<section id="history-view" class="view" hidden><p class="eyebrow">YOUR PROGRESS</p><h1>Training statistics.</h1><div id="history"></div></section>
<footer><span><kbd>W</kbd><kbd>A</kbd><kbd>S</kbd><kbd>D</kbd> Move &nbsp; <kbd>LMB</kbd> Fire &nbsp; <kbd>ESC</kbd> Pause</span><span>Desktop + mouse required · Inspired by KovaaK’s</span></footer></main>
<div id="overlay" hidden><section class="modal"><p class="eyebrow" id="overlay-label">CHALLENGE</p><h1 id="overlay-title">Paused</h1><p id="overlay-description"></p><div id="result-stats"></div><button id="resume" class="primary">Resume</button><button id="restart" class="secondary">Restart challenge</button><button id="back" class="text-button">Back to scenarios</button></section></div><div id="error" role="alert" hidden></div>`;
const menu = element("#menu", HTMLElement);
const overlay = element("#overlay", HTMLDivElement);
const crosshair = element("#crosshair", HTMLDivElement);
const live = {
  hud: element("#hud", HTMLDivElement),
  countdown: element("#countdown", HTMLDivElement),
  score: element("#score", HTMLElement),
  time: element("#time", HTMLElement),
  accuracy: element("#accuracy", HTMLElement),
  mode: element("#hud-mode", HTMLElement),
  name: element("#hud-name", HTMLElement),
  latency: element("#latency-panel", HTMLElement),
};
const timing = {
  mean: element("#latency-mean", HTMLElement),
  p95: element("#latency-p95", HTMLElement),
  dispatch: element("#latency-dispatch", HTMLElement),
  frame: element("#latency-frame", HTMLElement),
  cpu: element("#latency-cpu", HTMLElement),
  fps: element("#latency-fps", HTMLElement),
  mode: element("#input-mode", HTMLElement),
  samples: element("#latency-samples", HTMLElement),
};
function setText(node: HTMLElement, value: string): void {
  if (node.textContent !== value) node.textContent = value;
}
function performanceUpdate(s: PerformanceSnapshot): void {
  setText(timing.mean, s.input ? s.input.mean.toFixed(1) : "—");
  setText(timing.p95, s.input ? `${s.input.p95.toFixed(1)} ms` : "—");
  setText(
    timing.dispatch,
    s.dispatch ? `${s.dispatch.mean.toFixed(1)} ms` : "—",
  );
  setText(
    timing.frame,
    s.frame ? `${s.frame.mean.toFixed(1)} / ${s.frame.p95.toFixed(1)} ms` : "—",
  );
  setText(timing.cpu, s.cpu ? `${s.cpu.mean.toFixed(2)} ms` : "—");
  setText(timing.fps, s.fps !== null ? Math.round(s.fps).toString() : "—");
  setText(
    timing.mode,
    s.inputMode === "raw-requested"
      ? "Raw input requested"
      : s.inputMode === "standard"
        ? "Standard input · raw unavailable"
        : "Mouse not captured",
  );
  setText(
    timing.samples,
    s.input
      ? `${s.input.samples} input samples · last 2s${s.droppedSamples ? ` · ${s.droppedSamples} samples omitted` : ""}`
      : "Move or click to collect input samples.",
  );
}
function best(id: string): number {
  return Math.max(
    0,
    ...results.filter((r) => r.scenario === id).map((r) => r.score),
  );
}
function detail(): void {
  if (!selected) return;
  element("#detail-category", HTMLParagraphElement).textContent =
    selected.category;
  element("#detail-title", HTMLHeadingElement).textContent = selected.name;
  element("#detail-description", HTMLParagraphElement).textContent =
    selected.description;
  element("#preview-tag", HTMLDivElement).textContent =
    selected.mode === "click"
      ? "CLICK TIMING"
      : selected.mode === "track"
        ? "SMOOTH TRACKING"
        : "TARGET SWITCHING";
  element("#target-count", HTMLElement).textContent = String(selected.count);
  element("#best", HTMLElement).textContent = best(selected.id)
    ? best(selected.id).toLocaleString()
    : "—";
  element("#control-note", HTMLParagraphElement).textContent =
    selected.mode === "click"
      ? "Click to fire · One hit per target"
      : "Hold left mouse button to fire";
  document.querySelectorAll<HTMLElement>(".target-art i").forEach((el, i) => {
    el.hidden = i >= (selected?.count ?? 0);
  });
}
function renderList(): void {
  const query = element("#search", HTMLInputElement).value.toLowerCase();
  const matching = scenarios.filter(
    (s) =>
      s.name.toLowerCase().includes(query) &&
      (filter === "All" ||
        (filter === "Clicking" && s.mode === "click") ||
        (filter === "Tracking" && s.mode === "track") ||
        (filter === "Switching" && s.mode === "switch")),
  );
  element("#scenario-list", HTMLDivElement).innerHTML = matching.length
    ? matching
        .map(
          (s) =>
            `<button class="scenario-row ${s.id === selected?.id ? "selected" : ""}" data-scenario="${s.id}"><span class="scenario-icon ${s.mode}">${s.mode === "click" ? "⊕" : s.mode === "track" ? "∿" : "⋈"}</span><span class="scenario-name"><strong>${s.name}</strong><small>${s.category}</small></span><span class="row-best">${best(s.id) ? best(s.id).toLocaleString() : "—"}</span><span class="row-arrow">›</span></button>`,
        )
        .join("")
    : '<p class="empty">No matching scenarios.</p>';
  document.querySelectorAll<HTMLButtonElement>("[data-scenario]").forEach(
    (b) =>
      (b.onclick = () => {
        const s = scenarios.find((s) => s.id === b.dataset.scenario);
        if (s) {
          selected = s;
          engine.select(s);
          renderList();
          detail();
        }
      }),
  );
}
function renderHistory(): void {
  element("#history", HTMLDivElement).innerHTML = results.length
    ? `<div class="summary"><div><small>CHALLENGES</small><b>${results.length}</b></div><div><small>TRAINING TIME</small><b>${results.length} min</b></div><div><small>AVERAGE ACCURACY</small><b>${(results.reduce((a, r) => a + r.accuracy, 0) / results.length).toFixed(1)}%</b></div></div><div class="table-wrap"><table><thead><tr><th>Scenario</th><th>Score</th><th>Accuracy</th><th>Date</th></tr></thead><tbody>${[
        ...results,
      ]
        .reverse()
        .map(
          (r) =>
            `<tr><td>${scenarios.find((s) => s.id === r.scenario)?.name ?? ""}</td><td>${r.score.toLocaleString()}</td><td>${r.accuracy.toFixed(1)}%</td><td>${new Date(r.date).toLocaleString()}</td></tr>`,
        )
        .join(
          "",
        )}</tbody></table></div><p class="muted">Last 100 challenges on this device. Free play does not record scores.</p>`
    : '<div class="empty-state"><span>⌖</span><h2>Your first score starts here.</h2><p>Complete a challenge to see your results.</p><button class="primary" id="history-play">Choose a scenario</button></div>';
  const button = document.querySelector("#history-play");
  if (button instanceof HTMLButtonElement)
    button.onclick = () => tab("scenarios");
}
function tab(name: string): void {
  activeTab = name;
  for (const id of ["scenarios", "settings", "history"])
    element(`#${id}-view`, HTMLElement).hidden = id !== name;
  document
    .querySelectorAll<HTMLButtonElement>("[data-tab]")
    .forEach((b) => b.classList.toggle("active", b.dataset.tab === name));
  if (name === "history") renderHistory();
}
let previousPhase = "";
function update(s: Snapshot): void {
  const playing = s.phase === "running" || s.phase === "countdown";
  menu.hidden = s.phase !== "idle";
  live.hud.hidden = !playing;
  live.latency.hidden =
    !settings.showLatency || (!playing && s.phase !== "paused");
  crosshair.hidden = !playing;
  overlay.hidden = s.phase !== "paused" && s.phase !== "results";
  live.countdown.hidden = s.phase !== "countdown";
  setText(live.countdown, String(Math.max(1, s.countdown)));
  setText(live.score, s.score.toLocaleString());
  const seconds = Math.ceil(s.time);
  setText(
    live.time,
    `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`,
  );
  setText(live.accuracy, `${s.accuracy.toFixed(1)}%`);
  setText(live.mode, s.free ? "FREE PLAY" : "CHALLENGE");
  setText(live.name, selected?.name ?? "");
  if (s.phase !== previousPhase) {
    if (s.phase === "paused") {
      element("#overlay-label", HTMLParagraphElement).textContent =
        selected?.name ?? "";
      element("#overlay-title", HTMLHeadingElement).textContent = "Paused";
      element("#overlay-description", HTMLParagraphElement).textContent =
        "Take your time. Resume when ready.";
      element("#result-stats", HTMLDivElement).innerHTML = "";
      element("#resume", HTMLButtonElement).hidden = false;
      element("#restart", HTMLButtonElement).textContent = s.free
        ? "Restart free play"
        : "Restart challenge";
    }
    if (s.phase === "results" && lastResult) {
      element("#overlay-label", HTMLParagraphElement).textContent =
        selected?.name ?? "";
      element("#overlay-title", HTMLHeadingElement).textContent =
        lastResult.score > 0 && lastResult.score >= best(lastResult.scenario)
          ? "Personal best!"
          : "Challenge complete";
      element("#overlay-description", HTMLParagraphElement).textContent =
        storageWorks
          ? "Result saved on this device."
          : "Result could not be saved. Browser storage is unavailable.";
      element("#result-stats", HTMLDivElement).innerHTML =
        `<div class="result-score">${lastResult.score.toLocaleString()}<small>SCORE</small></div><div class="result-sub"><span>${lastResult.accuracy.toFixed(1)}%<small>ACCURACY</small></span><span>60s<small>DURATION</small></span></div>`;
      element("#resume", HTMLButtonElement).hidden = true;
      element("#restart", HTMLButtonElement).textContent = "Play again";
    }
    if (s.phase === "idle") {
      renderList();
      detail();
      if (activeTab === "history") renderHistory();
    }
    previousPhase = s.phase;
  }
}
let errorTimer = 0;
function showError(message: string): void {
  const box = element("#error", HTMLDivElement);
  box.textContent = message;
  box.hidden = false;
  clearTimeout(errorTimer);
  errorTimer = window.setTimeout(() => {
    box.hidden = true;
  }, 6000);
}
const engine = new Trainer(
  element("#arena", HTMLCanvasElement),
  selected,
  settings,
  update,
  (r) => {
    lastResult = r;
    results = [...results, r].slice(-100);
    save("aim-results", results);
  },
  showError,
  undefined,
  performanceUpdate,
);
element("#challenge", HTMLButtonElement).onclick = () => engine.start(false);
element("#freeplay", HTMLButtonElement).onclick = () => engine.start(true);
element("#resume", HTMLButtonElement).onclick = () => engine.resume();
element("#restart", HTMLButtonElement).onclick = () =>
  engine.start(
    element("#restart", HTMLButtonElement).textContent === "Restart free play",
  );
element("#back", HTMLButtonElement).onclick = () => engine.menu();
element("#search", HTMLInputElement).oninput = renderList;
document
  .querySelectorAll<HTMLButtonElement>("[data-tab]")
  .forEach((b) => (b.onclick = () => tab(b.dataset.tab ?? "scenarios")));
document.querySelectorAll<HTMLButtonElement>("[data-filter]").forEach(
  (b) =>
    (b.onclick = () => {
      filter = b.dataset.filter ?? "All";
      document
        .querySelectorAll<HTMLButtonElement>("[data-filter]")
        .forEach((other) => other.classList.toggle("active", other === b));
      renderList();
    }),
);
type NumericOrColorSetting = Exclude<keyof Settings, "showLatency">;
const fields: readonly [string, NumericOrColorSetting][] = [
  ["sensitivity", "sensitivity"],
  ["fov", "fov"],
  ["crosshair-size", "crosshair"],
  ["target-color", "color"],
  ["volume", "volume"],
  ["render-scale", "renderScale"],
];
function fillSettings(): void {
  element("#show-latency", HTMLInputElement).checked = settings.showLatency;
  for (const [id, key] of fields)
    element(`#${id}`, HTMLInputElement).value = String(settings[key]);
  crosshair.style.setProperty("--size", `${settings.crosshair}px`);
  document.documentElement.style.setProperty("--target", settings.color);
}
for (const [id, key] of fields) {
  const input = element(`#${id}`, HTMLInputElement);
  input.onchange = () => {
    if (!input.checkValidity()) {
      input.reportValidity();
      return;
    }
    const next: unknown = {
      ...settings,
      [key]: key === "color" ? input.value : input.valueAsNumber,
    };
    if (isRecord(next)) settings = parseSettings(next);
    save("aim-settings", settings);
    engine.configure(settings);
    fillSettings();
    element("#saved-message", HTMLParagraphElement).textContent = storageWorks
      ? "Settings saved."
      : "Storage unavailable. Settings apply to this session.";
  };
}
element("#reset-settings", HTMLButtonElement).onclick = () => {
  settings = parseSettings(null);
  save("aim-settings", settings);
  engine.configure(settings);
  fillSettings();
};
fillSettings();
renderList();
detail();

function toggleLatency(value: boolean): void {
  settings = { ...settings, showLatency: value };
  engine.configure(settings);
  live.latency.hidden =
    !value ||
    (previousPhase !== "running" &&
      previousPhase !== "countdown" &&
      previousPhase !== "paused");
  // Disk-backed persistence is deferred until settings/menu work, not performed during a shot.
  element("#show-latency", HTMLInputElement).checked = value;
}
element("#show-latency", HTMLInputElement).onchange = (event) => {
  if (event.currentTarget instanceof HTMLInputElement) {
    toggleLatency(event.currentTarget.checked);
    save("aim-settings", settings);
  }
};
document.addEventListener("keydown", (event) => {
  if (event.code === "F3" && !event.repeat) {
    event.preventDefault();
    toggleLatency(!settings.showLatency);
  }
});
document.addEventListener("pointerlockchange", () => {
  if (document.pointerLockElement === null) save("aim-settings", settings);
});
