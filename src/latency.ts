/** Software timing only: GPU completion, compositor and display are not observable here. */
export interface TimingSummary {
  mean: number;
  p95: number;
  samples: number;
}
export interface LatencySnapshot {
  input: TimingSummary | null;
  dispatch: TimingSummary | null;
  frame: TimingSummary | null;
  cpu: TimingSummary | null;
  fps: number | null;
  droppedSamples: number;
}
class Samples {
  private values: Float64Array;
  private times: Float64Array;
  private cursor = 0;
  private count = 0;
  constructor(capacity: number) {
    this.values = new Float64Array(capacity);
    this.times = new Float64Array(capacity);
  }
  clear(): void {
    this.cursor = 0;
    this.count = 0;
  }
  add(value: number, now: number): void {
    if (!Number.isFinite(value) || value < 0) return;
    this.values[this.cursor] = value;
    this.times[this.cursor] = now;
    this.cursor = (this.cursor + 1) % this.values.length;
    this.count = Math.min(this.count + 1, this.values.length);
  }
  summary(now: number): TimingSummary | null {
    const recent: number[] = [];
    for (let i = 0; i < this.count; i++) {
      const value = this.values[i];
      const time = this.times[i];
      if (value !== undefined && time !== undefined && now - time <= 2000)
        recent.push(value);
    }
    if (!recent.length) return null;
    recent.sort((a, b) => a - b);
    return {
      mean: recent.reduce((a, b) => a + b, 0) / recent.length,
      p95: recent[Math.ceil(recent.length * 0.95) - 1] ?? 0,
      samples: recent.length,
    };
  }
}
export function eventTime(
  timestamp: number,
  received: number,
  origin: number,
): number | null {
  // Some older implementations use epoch timestamps. Never substitute invented zero latency.
  const time = timestamp > 1e12 ? timestamp - origin : timestamp;
  return Number.isFinite(time) && time > 0 && time <= received ? time : null;
}
export class LatencyMeter {
  private input = new Samples(4096);
  private dispatch = new Samples(4096);
  private frames = new Samples(2048);
  private cpu = new Samples(2048);
  private pending = new Float64Array(2048);
  private pendingCount = 0;
  private lastFrame: number | null = null;
  private dropped = 0;
  reset(): void {
    this.input.clear();
    this.dispatch.clear();
    this.frames.clear();
    this.cpu.clear();
    this.pendingCount = 0;
    this.lastFrame = null;
    this.dropped = 0;
  }
  recordInput(timestamp: number, received: number, origin: number): void {
    const time = eventTime(timestamp, received, origin);
    if (time === null) return;
    if (this.pendingCount === this.pending.length) {
      this.dropped++;
      return;
    }
    this.pending[this.pendingCount++] = time;
    this.dispatch.add(received - time, received);
  }
  beginFrame(now: number): void {
    if (this.lastFrame !== null) this.frames.add(now - this.lastFrame, now);
    this.lastFrame = now;
  }
  submitted(start: number, end: number): void {
    this.cpu.add(end - start, end);
    for (let i = 0; i < this.pendingCount; i++) {
      const time = this.pending[i];
      if (time !== undefined) this.input.add(end - time, end);
    }
    this.pendingCount = 0;
  }
  snapshot(now: number): LatencySnapshot {
    const frame = this.frames.summary(now);
    return {
      input: this.input.summary(now),
      dispatch: this.dispatch.summary(now),
      frame,
      cpu: this.cpu.summary(now),
      fps: frame && frame.mean > 0 ? 1000 / frame.mean : null,
      droppedSamples: this.dropped,
    };
  }
}
