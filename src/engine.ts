import * as THREE from "three";
import { LatencyMeter, type LatencySnapshot } from "./latency";
import { captureMouse, type InputMode } from "./pointer-lock";
import {
  accuracy,
  horizontalToVertical,
  scoreFor,
  type Scenario,
  type Settings,
  type Result,
} from "./core";
export type Phase = "idle" | "countdown" | "running" | "paused" | "results";
export interface Snapshot {
  phase: Phase;
  time: number;
  score: number;
  accuracy: number;
  hits: number;
  countdown: number;
  free: boolean;
}
interface Target {
  mesh: THREE.Mesh<THREE.SphereGeometry, THREE.MeshStandardMaterial>;
  health: number;
  direction: number;
  turn: number;
}
export type ArenaRenderer = Pick<
  THREE.WebGLRenderer,
  "setPixelRatio" | "setClearColor" | "setSize" | "render" | "dispose"
>;
export interface PerformanceSnapshot extends LatencySnapshot {
  inputMode: InputMode;
}
export class Trainer {
  private renderer: ArenaRenderer;
  private scene = new THREE.Scene();
  private camera = new THREE.PerspectiveCamera();
  private ray = new THREE.Raycaster();
  private targets: Target[] = [];
  private targetMeshes: Target["mesh"][] = [];
  private intersections: THREE.Intersection[] = [];
  private center = new THREE.Vector2(0, 0);
  private movement = new THREE.Vector3();
  private up = new THREE.Vector3(0, 1, 0);
  private meter = new LatencyMeter();
  private inputMode: InputMode = "not-captured";
  private capturePending = false;
  private telemetryDue = 0;
  private sceneDirty = true;
  private readonly animate = (time: number): void => this.tick(time);
  private phase: Phase = "idle";
  private beforePause: Phase = "running";
  private elapsed = 0;
  private countdown = 3;
  private shots = 0;
  private hits = 0;
  private onTime = 0;
  private fireTime = 0;
  private held = false;
  private free = false;
  private last = 0;
  private frame = 0;
  private hudTime = 0;
  private keys = new Set<string>();
  private audio: AudioContext | null = null;
  constructor(
    private canvas: HTMLCanvasElement,
    private scenario: Scenario,
    private settings: Settings,
    private onUpdate: (s: Snapshot) => void,
    private onResult: (r: Result) => void,
    private onError: (message: string) => void,
    createRenderer: (canvas: HTMLCanvasElement) => ArenaRenderer = (canvas) =>
      new THREE.WebGLRenderer({
        canvas,
        antialias: false,
        powerPreference: "high-performance",
        alpha: false,
        stencil: false,
      }),
    private onPerformance: (snapshot: PerformanceSnapshot) => void = () => {},
  ) {
    this.renderer = createRenderer(canvas);
    this.renderer.setPixelRatio(this.settings.renderScale);
    this.renderer.setClearColor("#70787d");
    this.scene.fog = new THREE.Fog("#70787d", 28, 65);
    this.camera.position.set(0, 2, 9);
    this.camera.rotation.order = "YXZ";
    this.scene.add(new THREE.HemisphereLight(0xffffff, 0x657079, 2.3));
    const light = new THREE.DirectionalLight(0xffffff, 2);
    light.position.set(-4, 8, 7);
    this.scene.add(light);
    const floor = new THREE.Mesh(
      new THREE.PlaneGeometry(60, 60),
      new THREE.MeshStandardMaterial({ color: "#969b9d", roughness: 1 }),
    );
    floor.rotation.x = -Math.PI / 2;
    this.scene.add(floor);
    const grid = new THREE.GridHelper(60, 30, 0x656d71, 0x82898b);
    grid.position.y = 0.01;
    this.scene.add(grid);
    for (const [x, y, z, ry] of [
      [0, 10, -8, 0],
      [-15, 10, 7, Math.PI / 2],
      [15, 10, 7, -Math.PI / 2],
    ]) {
      const wall = new THREE.Mesh(
        new THREE.PlaneGeometry(30, 20),
        new THREE.MeshStandardMaterial({
          color: "#b0b5b6",
          side: THREE.DoubleSide,
        }),
      );
      wall.position.set(x ?? 0, y ?? 10, z ?? -8);
      wall.rotation.y = ry ?? 0;
      this.scene.add(wall);
    }
    const wallGrid = new THREE.GridHelper(30, 20, 0x92999b, 0x9fa6a8);
    wallGrid.rotation.x = Math.PI / 2;
    wallGrid.position.set(0, 10, -7.98);
    this.scene.add(wallGrid);
    window.addEventListener("resize", () => this.resize());
    document.addEventListener("pointerlockchange", () => {
      if (
        document.pointerLockElement !== canvas &&
        (this.phase === "running" || this.phase === "countdown")
      )
        this.pause();
    });
    document.addEventListener("pointerlockerror", () => {
      if (this.capturePending) return;
      this.pause();
      this.onError("Mouse capture failed. Click Resume to try again.");
    });
    document.addEventListener("mousemove", (e) => {
      if (
        document.pointerLockElement !== canvas ||
        !["running", "countdown"].includes(this.phase)
      )
        return;
      this.recordInput(e);
      const scale = (this.settings.sensitivity * 0.022 * Math.PI) / 180;
      this.camera.rotation.y -= e.movementX * scale;
      this.camera.rotation.x = THREE.MathUtils.clamp(
        this.camera.rotation.x - e.movementY * scale,
        -1.48,
        1.48,
      );
    });
    document.addEventListener("mousedown", (e) => {
      if (e.button !== 0 || document.pointerLockElement !== canvas) return;
      this.recordInput(e);
      this.held = true;
      if (this.phase === "running" && this.scenario.mode === "click")
        this.shoot();
    });
    document.addEventListener("mouseup", (e) => {
      if (e.button !== 0) return;
      this.recordInput(e);
      this.held = false;
    });
    document.addEventListener("keydown", (e) => {
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLSelectElement
      )
        return;
      if (!e.repeat && ["KeyW", "KeyA", "KeyS", "KeyD"].includes(e.code))
        this.recordInput(e);
      this.keys.add(e.code);
      if (e.code === "KeyR" && ["running", "countdown"].includes(this.phase))
        this.start(this.free);
    });
    document.addEventListener("keyup", (e) => this.keys.delete(e.code));
    window.addEventListener("blur", () => this.pause());
    document.addEventListener("visibilitychange", () => {
      if (document.hidden) this.pause();
    });
    this.resetTargets();
    this.resize();
    this.frame = requestAnimationFrame(this.animate);
  }
  resize(): void {
    this.sceneDirty = true;
    this.renderer.setSize(innerWidth, innerHeight);
    this.camera.aspect = innerWidth / innerHeight;
    this.camera.fov = horizontalToVertical(
      this.settings.fov,
      this.camera.aspect,
    );
    this.camera.updateProjectionMatrix();
  }
  configure(settings: Settings): void {
    this.settings = settings;
    this.renderer.setPixelRatio(settings.renderScale);
    this.meter.reset();
    for (const t of this.targets) t.mesh.material.color.set(settings.color);
    this.resize();
  }
  select(s: Scenario): void {
    this.sceneDirty = true;
    this.scenario = s;
    this.camera.position.set(0, 2, 9);
    this.camera.rotation.set(0, 0, 0);
    this.resetTargets();
  }
  private resetTargets(): void {
    if (
      this.targets.length === this.scenario.count &&
      this.targets[0]?.mesh.geometry.parameters.radius === this.scenario.radius
    ) {
      for (const target of this.targets) this.place(target);
      return;
    }
    for (const t of this.targets) {
      this.scene.remove(t.mesh);
      t.mesh.geometry.dispose();
      t.mesh.material.dispose();
    }
    this.targets = [];
    this.targetMeshes = [];
    for (let i = 0; i < this.scenario.count; i++) {
      const mesh = new THREE.Mesh(
        new THREE.SphereGeometry(this.scenario.radius, 24, 16),
        new THREE.MeshStandardMaterial({
          color: this.settings.color,
          roughness: 0.45,
        }),
      );
      this.scene.add(mesh);
      const t: Target = {
        mesh,
        health: 0.3,
        direction: Math.random() < 0.5 ? -1 : 1,
        turn: 1,
      };
      this.targets.push(t);
      this.targetMeshes.push(mesh);
      this.place(t);
    }
  }
  private place(t: Target): void {
    const span = this.scenario.id === "micro" ? 3 : 6;
    for (let i = 0; i < 100; i++) {
      t.mesh.position.set(
        (Math.random() - 0.5) * span,
        0.8 + Math.random() * 3,
        -3,
      );
      if (
        this.targets.every(
          (other) =>
            other === t ||
            other.mesh.position.distanceTo(t.mesh.position) >
              this.scenario.radius * 2.6,
        )
      )
        break;
    }
    t.health = 0.3;
    t.mesh.scale.setScalar(1);
  }
  async capture(): Promise<void> {
    if (document.pointerLockElement === this.canvas || this.capturePending)
      return;
    this.capturePending = true;
    try {
      this.inputMode = await captureMouse(this.canvas);
      // Legacy implementations can return before pointerlockchange.
    } catch {
      this.inputMode = "not-captured";
      this.pause();
      this.onError("Click Resume to capture the mouse. Use a desktop browser.");
    } finally {
      this.capturePending = false;
    }
  }
  private recordInput(event: MouseEvent | KeyboardEvent): void {
    if (
      !this.settings.showLatency ||
      document.pointerLockElement !== this.canvas ||
      (this.phase !== "running" && this.phase !== "countdown")
    )
      return;
    this.meter.recordInput(
      event.timeStamp,
      performance.now(),
      performance.timeOrigin,
    );
  }
  start(free: boolean): void {
    this.meter.reset();
    this.telemetryDue = 0;
    this.last = 0;
    this.free = free;
    this.elapsed = 0;
    this.countdown = 3;
    this.hits = 0;
    this.shots = 0;
    this.onTime = 0;
    this.fireTime = 0;
    this.held = false;
    this.keys.clear();
    this.camera.position.set(0, 2, 9);
    this.camera.rotation.set(0, 0, 0);
    this.resetTargets();
    this.phase = "countdown";
    this.emit();
    if (!this.audio)
      this.audio = new AudioContext({ latencyHint: "interactive" });
    void this.audio.resume().catch(() => {});
    void this.capture();
  }
  pause(): void {
    if (this.phase !== "running" && this.phase !== "countdown") return;
    this.beforePause = this.phase;
    this.phase = "paused";
    this.held = false;
    this.keys.clear();
    if (document.pointerLockElement === this.canvas) document.exitPointerLock();
    this.emit();
  }
  resume(): void {
    this.meter.reset();
    this.telemetryDue = 0;
    this.last = 0;
    this.phase = this.beforePause === "countdown" ? "countdown" : "running";
    this.emit();
    void this.capture();
  }
  menu(): void {
    this.phase = "idle";
    this.held = false;
    document.exitPointerLock();
    this.emit();
  }
  private hitTarget(): Target | undefined {
    // Mouse events and respawns can happen between renders. Raycasting must use current transforms.
    this.camera.updateMatrixWorld(true);
    for (const mesh of this.targetMeshes) mesh.updateMatrixWorld(true);
    this.ray.setFromCamera(this.center, this.camera);
    this.intersections.length = 0;
    const hit = this.ray.intersectObjects(
      this.targetMeshes,
      false,
      this.intersections,
    )[0];
    return this.targets.find((t) => t.mesh === hit?.object);
  }
  private sound(): void {
    if (!this.audio || this.settings.volume === 0) return;
    const oscillator = this.audio.createOscillator();
    const gain = this.audio.createGain();
    oscillator.frequency.setValueAtTime(850, this.audio.currentTime);
    oscillator.frequency.exponentialRampToValueAtTime(
      430,
      this.audio.currentTime + 0.04,
    );
    gain.gain.setValueAtTime(
      this.settings.volume * 0.14,
      this.audio.currentTime,
    );
    gain.gain.exponentialRampToValueAtTime(
      0.001,
      this.audio.currentTime + 0.05,
    );
    oscillator.connect(gain);
    gain.connect(this.audio.destination);
    oscillator.start();
    oscillator.stop(this.audio.currentTime + 0.05);
  }
  private shoot(): void {
    this.shots++;
    const target = this.hitTarget();
    if (target) {
      this.hits++;
      this.place(target);
      this.sound();
    }
  }
  private getAccuracy(): number {
    return this.scenario.mode === "click"
      ? accuracy(this.hits, this.shots)
      : accuracy(this.onTime, this.fireTime);
  }
  private getScore(): number {
    return this.scenario.mode === "switch"
      ? Math.round(this.hits * 100)
      : scoreFor(this.scenario.mode, this.hits, this.shots, this.onTime);
  }
  private finish(): void {
    this.phase = "results";
    this.held = false;
    document.exitPointerLock();
    this.onResult({
      scenario: this.scenario.id,
      score: this.getScore(),
      accuracy: this.getAccuracy(),
      hits: this.hits,
      shots: this.shots,
      duration: 60,
      date: new Date().toISOString(),
    });
    this.emit();
  }
  private emit(): void {
    this.onUpdate({
      phase: this.phase,
      time: this.free ? this.elapsed : Math.max(0, 60 - this.elapsed),
      score: this.getScore(),
      accuracy: this.getAccuracy(),
      hits: this.hits,
      countdown: Math.ceil(this.countdown),
      free: this.free,
    });
  }
  private tick(now: number): void {
    const started = performance.now();
    const active = this.phase === "running" || this.phase === "countdown";
    const measuring = active && this.settings.showLatency;
    if (measuring) this.meter.beginFrame(started);
    const delta = Math.min((now - (this.last || now)) / 1000, 0.05);
    this.last = now;
    if (
      this.phase === "countdown" &&
      document.pointerLockElement === this.canvas
    ) {
      this.countdown -= delta;
      if (this.countdown <= 0) {
        this.phase = "running";
        this.held = false;
      }
    }
    if (this.phase === "running") {
      const dt = this.free ? delta : Math.min(delta, 60 - this.elapsed);
      this.elapsed += dt;
      const movement = this.movement.set(
        Number(this.keys.has("KeyD")) - Number(this.keys.has("KeyA")),
        0,
        Number(this.keys.has("KeyS")) - Number(this.keys.has("KeyW")),
      );
      movement
        .normalize()
        .applyAxisAngle(this.up, this.camera.rotation.y)
        .multiplyScalar(dt * 3);
      this.camera.position.add(movement);
      this.camera.position.x = THREE.MathUtils.clamp(
        this.camera.position.x,
        -10,
        10,
      );
      this.camera.position.z = THREE.MathUtils.clamp(
        this.camera.position.z,
        2,
        14,
      );
      for (const t of this.targets) {
        if (this.scenario.speed > 0) {
          t.turn -= dt;
          if (Math.abs(t.mesh.position.x) > 4.5) {
            t.direction = t.mesh.position.x > 0 ? -1 : 1;
          }
          if (this.scenario.id === "reactive" && t.turn <= 0) {
            t.direction *= -1;
            t.turn = 0.25 + Math.random() * 0.8;
          }
          t.mesh.position.x += t.direction * this.scenario.speed * dt;
          if (this.scenario.id === "smooth")
            t.mesh.position.y = 2 + Math.sin(this.elapsed * 0.8) * 0.6;
        }
      }
      if (this.held && this.scenario.mode !== "click") {
        this.fireTime += dt;
        const target = this.hitTarget();
        if (target) {
          this.onTime += dt;
          if (this.scenario.mode === "switch") {
            target.health -= dt;
            target.mesh.scale.setScalar(0.8 + (0.2 * target.health) / 0.3);
            if (target.health <= 0) {
              this.hits++;
              this.place(target);
              this.sound();
            }
          }
        }
      }
      if (!this.free && this.elapsed >= 60) this.finish();
    }
    if (active || this.sceneDirty) {
      this.renderer.render(this.scene, this.camera);
      this.sceneDirty = false;
    }
    if (measuring) {
      const submitted = performance.now();
      this.meter.submitted(started, submitted);
      if (submitted >= this.telemetryDue) {
        this.onPerformance({
          ...this.meter.snapshot(submitted),
          inputMode: this.inputMode,
        });
        this.telemetryDue = submitted + 500;
      }
    }
    // Submit the 3D frame before touching the DOM. HUD and telemetry have independent rates.
    this.hudTime += delta;
    if (active && this.hudTime > 0.05) {
      this.emit();
      this.hudTime = 0;
    }
    this.frame = requestAnimationFrame(this.animate);
  }
  dispose(): void {
    cancelAnimationFrame(this.frame);
    this.renderer.dispose();
  }
}
