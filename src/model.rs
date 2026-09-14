use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

pub const CHALLENGE_SECONDS: f64 = 60.0;
pub const EYE: Vec3 = Vec3::new(0.0, 2.6, 2.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Drill {
    Six,
    Frenzy,
    Micro,
    Smooth,
    Strafes,
    Switching,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Click,
    Track,
    Switch,
}

impl Drill {
    pub const ALL: [Self; 6] = [
        Self::Six,
        Self::Frenzy,
        Self::Micro,
        Self::Smooth,
        Self::Strafes,
        Self::Switching,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Six => "1wall 6targets",
            Self::Frenzy => "Tile Frenzy",
            Self::Micro => "Micro Precision",
            Self::Smooth => "Smooth Tracking",
            Self::Strafes => "Close Strafes",
            Self::Switching => "Target Switching",
        }
    }

    pub fn mode(self) -> Mode {
        match self {
            Self::Six | Self::Frenzy | Self::Micro => Mode::Click,
            Self::Smooth | Self::Strafes => Mode::Track,
            Self::Switching => Mode::Switch,
        }
    }

    pub fn category(self) -> &'static str {
        match self.mode() {
            Mode::Click => "CLICKING",
            Mode::Track => "TRACKING",
            Mode::Switch => "SWITCHING",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Six => {
                "Six targets on one wall. Click each target once. Aim for clean stops and accurate flicks."
            }
            Self::Frenzy => {
                "Three large square targets. Move quickly from tile to tile. Keep your clicks under control."
            }
            Self::Micro => {
                "Five small targets in a narrow area. Make small corrections. Put accuracy before speed."
            }
            Self::Smooth => {
                "Hold fire and follow a moving sphere. Keep your crosshair on the target through long, smooth turns."
            }
            Self::Strafes => {
                "Hold fire on a close target. React to short, random changes in direction without moving too far."
            }
            Self::Switching => {
                "Hold fire and move between four targets. Clear each target with 0.3 seconds of accurate fire."
            }
        }
    }

    pub fn count(self) -> usize {
        match self {
            Self::Six => 6,
            Self::Frenzy => 3,
            Self::Micro => 5,
            Self::Smooth | Self::Strafes => 1,
            Self::Switching => 4,
        }
    }

    pub fn radius(self) -> f32 {
        match self {
            Self::Six => 0.34,
            Self::Frenzy => 0.65,
            Self::Micro => 0.17,
            Self::Smooth => 0.48,
            Self::Strafes => 0.52,
            Self::Switching => 0.40,
        }
    }

    pub fn cue(self) -> &'static str {
        match self.mode() {
            Mode::Click => "CLICK EACH TARGET",
            Mode::Track => "HOLD FIRE AND FOLLOW",
            Mode::Switch => "HOLD FIRE TO CLEAR TARGETS",
        }
    }

    pub fn scoring(self) -> &'static str {
        match self.mode() {
            Mode::Click => "Score = hits x 100 x accuracy",
            Mode::Track => "Score = seconds on target x 100",
            Mode::Switch => "Score = targets cleared x 100",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SensScale {
    #[default]
    Source,
    Valorant,
    Overwatch,
    MarvelRivals,
}

impl SensScale {
    pub fn name(self) -> &'static str {
        match self {
            Self::Source => "Source / Quake",
            Self::Valorant => "Valorant",
            Self::Overwatch => "Overwatch",
            Self::MarvelRivals => "Marvel Rivals",
        }
    }
    pub fn yaw(self) -> f32 {
        match self {
            Self::Source => 0.022,
            Self::Valorant => 0.07,
            Self::Overwatch => 0.0066,
            Self::MarvelRivals => 0.0175,
        }
    }
    pub fn next(self) -> Self {
        match self {
            Self::Source => Self::Valorant,
            Self::Valorant => Self::Overwatch,
            Self::Overwatch => Self::MarvelRivals,
            Self::MarvelRivals => Self::Source,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub sensitivity: f32,
    pub scale: SensScale,
    pub dpi: f32,
    pub fov: f32,
    pub crosshair_size: f32,
    pub crosshair_gap: f32,
    pub target_color: usize,
    pub crosshair_color: usize,
    pub volume: f32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub fps_limit: u32,
    pub show_fps: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sensitivity: 1.0,
            scale: SensScale::Source,
            dpi: 800.0,
            fov: 103.0,
            crosshair_size: 6.0,
            crosshair_gap: 3.0,
            target_color: 0,
            crosshair_color: 0,
            volume: 0.3,
            fullscreen: false,
            vsync: false,
            fps_limit: 360,
            show_fps: true,
        }
    }
}

impl Settings {
    pub fn validate(&mut self) {
        fn bounded(value: &mut f32, min: f32, max: f32, default: f32) {
            if !value.is_finite() || !(min..=max).contains(value) {
                *value = default;
            }
        }
        bounded(&mut self.sensitivity, 0.01, 100.0, 1.0);
        bounded(&mut self.dpi, 100.0, 32000.0, 800.0);
        bounded(&mut self.fov, 60.0, 140.0, 103.0);
        bounded(&mut self.crosshair_size, 1.0, 16.0, 6.0);
        bounded(&mut self.crosshair_gap, 0.0, 12.0, 3.0);
        bounded(&mut self.volume, 0.0, 1.0, 0.3);
        if self.target_color > 4 {
            self.target_color = 0;
        }
        if self.crosshair_color > 3 {
            self.crosshair_color = 0;
        }
        if ![0, 144, 240, 360, 500].contains(&self.fps_limit) {
            self.fps_limit = 360;
        }
    }
    pub fn cm_per_turn(&self) -> f32 {
        360.0 * 2.54 / (self.scale.yaw() * self.sensitivity * self.dpi)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Countdown,
    Running,
    Paused,
    Finished,
}

#[derive(Clone, Debug)]
pub struct Target {
    pub position: Vec3,
    pub radius: f32,
    pub health: f64,
    pub velocity: f32,
    pub turn_in: f64,
}

#[derive(Default)]
pub struct Input {
    pub look: Vec2,
    pub movement: Vec2,
    pub click: bool,
    pub firing: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunResult {
    pub drill: Drill,
    pub score: f64,
    pub accuracy: f64,
    pub hits: u32,
    pub shots: u32,
    pub tracking_seconds: f64,
    pub timestamp: u64,
}

impl RunResult {
    pub fn valid(&self) -> bool {
        self.score.is_finite()
            && (0.0..=1_000_000.0).contains(&self.score)
            && self.accuracy.is_finite()
            && (0.0..=100.0).contains(&self.accuracy)
            && self.tracking_seconds.is_finite()
            && (0.0..=CHALLENGE_SECONDS).contains(&self.tracking_seconds)
            && self.hits <= self.shots
            && self.shots <= 100_000
            && self.timestamp > 0
    }
}

pub struct Session {
    pub drill: Drill,
    pub free_play: bool,
    pub phase: Phase,
    resume_phase: Phase,
    pub countdown: f64,
    pub elapsed: f64,
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub targets: Vec<Target>,
    pub hits: u32,
    pub shots: u32,
    pub on_target: f64,
    pub firing_time: f64,
    pub hit_flash: f64,
    pub shot_flash: f64,
    rng: u64,
}

impl Session {
    pub fn new(drill: Drill, free_play: bool, seed: u64) -> Self {
        let mut session = Self {
            drill,
            free_play,
            phase: Phase::Countdown,
            resume_phase: Phase::Countdown,
            countdown: if free_play { 1.0 } else { 3.0 },
            elapsed: 0.0,
            position: EYE,
            yaw: 0.0,
            pitch: 0.0,
            targets: Vec::new(),
            hits: 0,
            shots: 0,
            on_target: 0.0,
            firing_time: 0.0,
            hit_flash: 0.0,
            shot_flash: 0.0,
            rng: seed.max(1),
        };
        for _ in 0..drill.count() {
            let target = session.make_target(None);
            session.targets.push(target);
        }
        session
    }

    fn random(&mut self, min: f32, max: f32) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        let fraction = f32::from(u16::try_from(self.rng >> 48).expect("top 16 bits fit")) / 65535.0;
        min + fraction * (max - min)
    }

    fn make_target(&mut self, replace: Option<usize>) -> Target {
        let radius = self.drill.radius();
        let mut position = Vec3::ZERO;
        for _ in 0..128 {
            position = match self.drill {
                Drill::Micro => Vec3::new(self.random(-2.8, 2.8), self.random(1.2, 4.0), -10.0),
                Drill::Smooth => Vec3::new(0.0, 2.8, -7.0),
                Drill::Strafes => Vec3::new(0.0, 2.6, -4.5),
                _ => Vec3::new(self.random(-6.0, 6.0), self.random(1.0, 5.5), -10.0),
            };
            if self.targets.iter().enumerate().all(|(i, t)| {
                Some(i) == replace || t.position.distance(position) > (radius + t.radius) * 1.5
            }) {
                break;
            }
        }
        Target {
            position,
            radius,
            health: 0.3,
            velocity: if self.random(0.0, 1.0) < 0.5 {
                -1.0
            } else {
                1.0
            },
            turn_in: f64::from(self.random(0.25, 0.75)),
        }
    }

    pub fn direction(&self) -> Vec3 {
        Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        )
    }

    pub fn pause(&mut self) {
        if matches!(self.phase, Phase::Running | Phase::Countdown) {
            self.resume_phase = self.phase;
            self.phase = Phase::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.phase == Phase::Paused {
            self.phase = self.resume_phase;
        }
    }

    pub fn nearest_hit(&self) -> Option<usize> {
        let direction = self.direction();
        self.targets
            .iter()
            .enumerate()
            .filter_map(|(i, target)| {
                let hit = if self.drill == Drill::Frenzy {
                    ray_box(
                        self.position,
                        direction,
                        target.position,
                        Vec3::new(target.radius, target.radius, 0.15),
                    )
                } else {
                    ray_sphere(self.position, direction, target.position, target.radius)
                };
                hit.map(|distance| (i, distance))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)
    }

    pub fn tick(&mut self, dt: f64, input: &Input, settings: &Settings) {
        if !dt.is_finite() || dt <= 0.0 || matches!(self.phase, Phase::Paused | Phase::Finished) {
            return;
        }
        // A suspended process or stalled GPU must not silently consume a challenge.
        if dt > 0.5 {
            self.pause();
            return;
        }
        self.yaw = (self.yaw
            + (input.look.x * settings.sensitivity * settings.scale.yaw()).to_radians())
        .rem_euclid(std::f32::consts::TAU);
        self.pitch = (self.pitch
            - (input.look.y * settings.sensitivity * settings.scale.yaw()).to_radians())
        .clamp(-1.50, 1.50);
        if self.phase == Phase::Countdown {
            self.countdown = (self.countdown - dt).max(0.0);
            if self.countdown == 0.0 {
                self.phase = Phase::Running;
            }
            return;
        }
        let active_dt = if self.free_play {
            dt
        } else {
            dt.min(CHALLENGE_SECONDS - self.elapsed)
        };
        self.hit_flash = (self.hit_flash - active_dt).max(0.0);
        self.shot_flash = (self.shot_flash - active_dt).max(0.0);
        // Clicks use the current mouse aim, before the next movement step.
        if input.click && self.drill.mode() == Mode::Click {
            self.shots += 1;
            self.shot_flash = 0.045;
            if let Some(index) = self.nearest_hit() {
                self.hits += 1;
                self.hit_flash = 0.10;
                self.targets[index] = self.make_target(Some(index));
            }
        }
        let mut remaining = active_dt;
        while remaining > 1e-9 {
            let step = remaining.min(1.0 / 240.0);
            self.step(step, input);
            remaining -= step;
        }
        if !self.free_play && self.elapsed >= CHALLENGE_SECONDS - 1e-8 {
            self.elapsed = CHALLENGE_SECONDS;
            self.phase = Phase::Finished;
        }
    }

    fn step(&mut self, dt: f64, input: &Input) {
        self.elapsed += dt;
        let dt32 = dt as f32;
        let move_axis = input.movement.normalize_or_zero();
        let forward = Vec3::new(self.yaw.sin(), 0.0, -self.yaw.cos());
        let right = Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin());
        self.position += (right * move_axis.x + forward * move_axis.y) * dt32 * 4.0;
        self.position.x = self.position.x.clamp(-8.0, 8.0);
        self.position.z = self.position.z.clamp(-1.5, 8.0);
        for index in 0..self.targets.len() {
            let target = &mut self.targets[index];
            match self.drill {
                Drill::Smooth => {
                    let time = self.elapsed as f32;
                    target.position.x = (time * 0.65).sin() * 4.7;
                    target.position.y = 2.8 + (time * 0.9).sin() * 0.9;
                }
                Drill::Strafes => {
                    target.position.x += target.velocity * 4.5 * dt32;
                    target.turn_in -= dt;
                    let turn = target.turn_in <= 0.0 || target.position.x.abs() > 4.5;
                    if turn {
                        target.velocity *= -1.0;
                        target.position.x = target.position.x.clamp(-4.5, 4.5);
                        let interval = f64::from(self.random(0.22, 0.8));
                        self.targets[index].turn_in = interval;
                    }
                }
                Drill::Switching => {
                    target.position.x += target.velocity * 0.8 * dt32;
                    if target.position.x.abs() > 6.3 {
                        target.velocity *= -1.0;
                        target.position.x = target.position.x.clamp(-6.3, 6.3);
                    }
                }
                _ => {}
            }
        }
        if input.firing && self.drill.mode() != Mode::Click {
            self.firing_time += dt;
            self.shot_flash = 0.045;
            if let Some(index) = self.nearest_hit() {
                self.on_target += dt;
                self.hit_flash = 0.06;
                if self.drill.mode() == Mode::Switch {
                    self.targets[index].health -= dt;
                    if self.targets[index].health <= 1e-8 {
                        self.hits += 1;
                        self.shots += 1;
                        self.targets[index] = self.make_target(Some(index));
                    }
                }
            }
        }
    }

    pub fn accuracy(&self) -> f64 {
        match self.drill.mode() {
            Mode::Click => {
                if self.shots > 0 {
                    f64::from(self.hits) / f64::from(self.shots) * 100.0
                } else {
                    0.0
                }
            }
            _ => {
                if self.firing_time > 0.0 {
                    self.on_target / self.firing_time * 100.0
                } else {
                    0.0
                }
            }
        }
    }

    pub fn score(&self) -> f64 {
        match self.drill.mode() {
            Mode::Click => (f64::from(self.hits) * self.accuracy()).round(),
            Mode::Track => (self.on_target * 100.0).round(),
            Mode::Switch => f64::from(self.hits) * 100.0,
        }
    }

    pub fn result(&self, timestamp: u64) -> Option<RunResult> {
        (!self.free_play && self.phase == Phase::Finished).then(|| RunResult {
            drill: self.drill,
            score: self.score(),
            accuracy: self.accuracy(),
            hits: self.hits,
            shots: self.shots,
            tracking_seconds: self.on_target.min(CHALLENGE_SECONDS),
            timestamp,
        })
    }
}

pub fn vertical_fov(horizontal_degrees: f32, aspect: f32) -> f32 {
    (2.0 * ((horizontal_degrees.to_radians() * 0.5).tan() / aspect).atan()).to_degrees()
}

pub fn ray_sphere(origin: Vec3, direction: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let offset = origin - center;
    let b = offset.dot(direction);
    let discriminant = b * b - offset.length_squared() + radius * radius;
    if discriminant < 0.0 {
        return None;
    }
    let near = -b - discriminant.sqrt();
    let far = -b + discriminant.sqrt();
    if near >= 0.0 {
        Some(near)
    } else if far >= 0.0 {
        Some(far)
    } else {
        None
    }
}

pub fn ray_box(origin: Vec3, direction: Vec3, center: Vec3, half: Vec3) -> Option<f32> {
    let mut near: f32 = 0.0;
    let mut far = f32::INFINITY;
    for axis in 0..3 {
        let offset = origin[axis] - center[axis];
        if direction[axis].abs() < 1e-7 {
            if offset.abs() > half[axis] {
                return None;
            }
        } else {
            let a = (-half[axis] - offset) / direction[axis];
            let b = (half[axis] - offset) / direction[axis];
            near = near.max(a.min(b));
            far = far.min(a.max(b));
            if near > far {
                return None;
            }
        }
    }
    (far >= 0.0).then_some(near)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn running(drill: Drill) -> Session {
        let mut session = Session::new(drill, false, 42);
        for _ in 0..31 {
            session.tick(0.1, &Input::default(), &Settings::default());
        }
        assert_eq!(session.phase, Phase::Running);
        session
    }

    #[test]
    fn ray_hits_use_real_target_volume_and_ignore_targets_behind() {
        assert_eq!(
            ray_sphere(Vec3::ZERO, -Vec3::Z, Vec3::new(0.0, 0.0, -5.0), 1.0),
            Some(4.0)
        );
        assert!(ray_sphere(Vec3::ZERO, -Vec3::Z, Vec3::new(0.0, 0.0, 5.0), 1.0).is_none());
        assert!(ray_sphere(Vec3::ZERO, -Vec3::Z, Vec3::new(2.0, 0.0, -5.0), 1.0).is_none());
        assert_eq!(
            ray_box(Vec3::ZERO, -Vec3::Z, Vec3::new(0.0, 0.0, -5.0), Vec3::ONE),
            Some(4.0)
        );
        assert!(
            ray_box(
                Vec3::new(2.0, 0.0, 0.0),
                -Vec3::Z,
                -Vec3::Z * 5.0,
                Vec3::ONE
            )
            .is_none()
        );
    }

    #[test]
    fn flick_and_click_in_same_frame_uses_new_aim_and_scores_misses() {
        let mut session = running(Drill::Six);
        let angle = 12.0_f32.to_radians();
        session.targets[0].position = EYE + Vec3::new(angle.sin(), 0.0, -angle.cos()) * 10.0;
        session.tick(
            0.01,
            &Input {
                look: Vec2::new(12.0 / 0.022, 0.0),
                click: true,
                ..Input::default()
            },
            &Settings::default(),
        );
        assert_eq!(session.hits, 1);
        assert_eq!(session.score(), 100.0);
        session.pitch = 1.4;
        session.tick(
            0.01,
            &Input {
                click: true,
                ..Input::default()
            },
            &Settings::default(),
        );
        assert_eq!(session.shots, 2);
        assert_eq!(session.score(), 50.0);
    }

    #[test]
    fn challenge_pause_countdown_and_free_play_have_separate_clocks() {
        let mut session = Session::new(Drill::Six, false, 1);
        session.tick(
            0.2,
            &Input {
                click: true,
                ..Input::default()
            },
            &Settings::default(),
        );
        assert_eq!(session.shots, 0);
        session.pause();
        session.tick(0.2, &Input::default(), &Settings::default());
        assert_eq!(session.countdown, 2.8);
        session.resume();
        assert_eq!(session.phase, Phase::Countdown);
        let mut session = running(Drill::Six);
        session.pause();
        let elapsed = session.elapsed;
        session.tick(0.1, &Input::default(), &Settings::default());
        assert_eq!(session.elapsed, elapsed);
        session.resume();
        for _ in 0..601 {
            session.tick(0.1, &Input::default(), &Settings::default());
        }
        assert_eq!(session.phase, Phase::Finished);
        assert_eq!(session.elapsed, 60.0);
        assert!(session.result(1).is_some());
        let mut free = Session::new(Drill::Six, true, 1);
        for _ in 0..650 {
            free.tick(0.1, &Input::default(), &Settings::default());
        }
        assert_eq!(free.phase, Phase::Running);
        assert!(free.result(1).is_none());
    }

    #[test]
    fn tracking_and_switching_require_fire_and_count_time_on_target() {
        let mut track = running(Drill::Smooth);
        track.targets[0].radius = 10.0;
        track.tick(0.1, &Input::default(), &Settings::default());
        assert_eq!(track.score(), 0.0);
        track.tick(
            0.1,
            &Input {
                firing: true,
                ..Input::default()
            },
            &Settings::default(),
        );
        assert_eq!(track.score(), 10.0);
        assert!((track.accuracy() - 100.0).abs() < 1e-8);
        track.pitch = 1.5;
        track.targets[0].radius = 0.1;
        track.tick(
            0.1,
            &Input {
                firing: true,
                ..Input::default()
            },
            &Settings::default(),
        );
        assert!((track.accuracy() - 50.0).abs() < 1e-8);
        let mut switch = running(Drill::Switching);
        switch.targets[0].position = EYE - Vec3::Z * 10.0;
        for _ in 0..3 {
            switch.tick(
                0.1,
                &Input {
                    firing: true,
                    ..Input::default()
                },
                &Settings::default(),
            );
        }
        assert_eq!(switch.hits, 1);
        assert_eq!(switch.score(), 100.0);
    }

    #[test]
    fn settings_and_projection_keep_valid_ranges() {
        let mut settings = Settings {
            sensitivity: f32::NAN,
            fov: 200.0,
            target_color: 10,
            fps_limit: u32::MAX,
            ..Settings::default()
        };
        settings.validate();
        assert_eq!(settings.sensitivity, 1.0);
        assert_eq!(settings.fov, 103.0);
        assert_eq!(settings.target_color, 0);
        assert_eq!(settings.fps_limit, 360);
        assert!((settings.cm_per_turn() - 51.95).abs() < 0.01);
        settings.scale = SensScale::MarvelRivals;
        assert!((settings.cm_per_turn() - 65.3143).abs() < 0.001);
        assert!((vertical_fov(90.0, 1.0) - 90.0).abs() < 0.001);
        assert!((vertical_fov(90.0, 16.0 / 9.0) - 58.7155).abs() < 0.001);
    }
}
