use crate::{
    model::{CHALLENGE_SECONDS, Drill, Mode, Phase, Session},
    storage::{Store, timestamp},
    ui::*,
    world::{CROSSHAIR_COLORS, TARGET_COLORS},
};
use raylib::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Library,
    Settings,
    History,
    Play,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Clicking,
    Tracking,
    Switching,
}

impl Filter {
    fn accepts(self, drill: Drill) -> bool {
        match self {
            Self::All => true,
            Self::Clicking => drill.mode() == Mode::Click,
            Self::Tracking => drill.mode() == Mode::Track,
            Self::Switching => drill.mode() == Mode::Switch,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Action {
    None,
    Start(bool),
    Restart,
    Resume,
    Library,
    Settings,
    History,
    Back,
    Quit,
}

pub struct App {
    pub store: Store,
    pub screen: Screen,
    settings_return: Screen,
    pub selected: Drill,
    pub preview: Session,
    pub session: Session,
    filter: Filter,
    pub recorded: bool,
    pub best_before: f64,
    pub fps: i32,
    pub frame_ms: f32,
}

impl App {
    pub fn new(store: Store) -> Self {
        Self {
            store,
            screen: Screen::Library,
            settings_return: Screen::Library,
            selected: Drill::Six,
            preview: Session::new(Drill::Six, true, 712),
            session: Session::new(Drill::Six, false, 712),
            filter: Filter::All,
            recorded: false,
            best_before: 0.0,
            fps: 0,
            frame_ms: 0.0,
        }
    }
    pub fn active(&self) -> bool {
        self.screen == Screen::Play
            && matches!(self.session.phase, Phase::Countdown | Phase::Running)
    }
    pub fn act(&mut self, action: Action) {
        match action {
            Action::Start(free) => {
                self.session = Session::new(
                    self.selected,
                    free,
                    timestamp() ^ u64::from(std::process::id()),
                );
                self.best_before = self.store.best(self.selected);
                self.recorded = false;
                self.screen = Screen::Play;
            }
            Action::Restart => self.act(Action::Start(self.session.free_play)),
            Action::Resume => {
                self.session.resume();
                self.screen = Screen::Play;
            }
            Action::Library => {
                self.session.pause();
                self.screen = Screen::Library;
            }
            Action::Settings => {
                self.settings_return = self.screen;
                self.session.pause();
                self.screen = Screen::Settings;
            }
            Action::History => {
                self.screen = Screen::History;
            }
            Action::Back => {
                self.store.save_settings();
                self.screen = self.settings_return;
            }
            Action::None | Action::Quit => {}
        }
    }
    pub fn record_finished(&mut self) {
        if !self.recorded
            && let Some(result) = self.session.result(timestamp())
        {
            self.store.record(result);
            self.recorded = true;
        }
    }
    pub fn draw<D: RaylibDraw>(&mut self, ui: &mut Ui<'_, D>, preview: &RenderTexture2D) -> Action {
        let mut action = Action::None;
        if self.screen != Screen::Play {
            ui.fill(rect(0.0, 0.0, 1440.0, 900.0), BG);
            ui.fill(rect(0.0, 0.0, 1440.0, 84.0), PANEL);
            ui.draw.draw_circle_lines(51, 42, 14.0, ACCENT);
            ui.draw.draw_circle(51, 42, 4.0, ACCENT);
            ui.fill(rect(49.0, 21.0, 3.0, 11.0), ACCENT);
            ui.fill(rect(30.0, 40.0, 11.0, 3.0), ACCENT);
            ui.strong("AIM ROOM", 82.0, 22.0, 28.0, TEXT);
            ui.text("SANDBOX", 84.0, 54.0, 11.0, MUTED);
            if ui.tab(
                "SCENARIOS",
                rect(370.0, 0.0, 164.0, 84.0),
                self.screen == Screen::Library,
            ) {
                action = Action::Library;
            }
            if ui.tab(
                "LOCAL RECORDS",
                rect(534.0, 0.0, 182.0, 84.0),
                self.screen == Screen::History,
            ) {
                action = Action::History;
            }
            if ui.tab(
                "SETTINGS",
                rect(716.0, 0.0, 156.0, 84.0),
                self.screen == Screen::Settings,
            ) && self.screen != Screen::Settings
            {
                action = Action::Settings;
            }
            ui.draw.draw_circle(1165, 41, 4.0, GREEN);
            ui.text("OFFLINE / LINUX", 1179.0, 34.0, 13.0, MUTED);
            if ui.button("EXIT", rect(1333.0, 24.0, 74.0, 36.0), false) {
                action = Action::Quit;
            }
            ui.fill(rect(0.0, 852.0, 1440.0, 48.0), PANEL);
            ui.text(
                "ENTER  Challenge     F2  Settings     F11  Fullscreen",
                36.0,
                868.0,
                13.0,
                MUTED,
            );
            ui.text("Native OpenGL  /  v0.2", 1190.0, 868.0, 13.0, MUTED);
        }
        let page_action = match self.screen {
            Screen::Library => self.library(ui, preview),
            Screen::Settings => self.settings(ui),
            Screen::History => self.history(ui),
            Screen::Play => self.play(ui),
        };
        if !matches!(page_action, Action::None) {
            action = page_action;
        }
        if let Some(message) = &self.store.notice {
            ui.fill(rect(32.0, 794.0, 1376.0, 42.0), Color::new(77, 47, 34, 255));
            ui.text(
                &message.chars().take(125).collect::<String>(),
                46.0,
                807.0,
                13.0,
                ACCENT,
            );
        }
        action
    }

    fn library<D: RaylibDraw>(&mut self, ui: &mut Ui<'_, D>, preview: &RenderTexture2D) -> Action {
        let mut action = Action::None;
        ui.strong("Scenarios", 36.0, 114.0, 31.0, TEXT);
        ui.text(
            "Choose a drill. Build speed, control and accuracy.",
            36.0,
            158.0,
            16.0,
            MUTED,
        );
        let filters = [
            (Filter::All, "ALL"),
            (Filter::Clicking, "CLICKING"),
            (Filter::Tracking, "TRACKING"),
            (Filter::Switching, "SWITCHING"),
        ];
        for (i, (filter, label)) in filters.into_iter().enumerate() {
            if ui.tab(
                label,
                rect(36.0 + i as f32 * 145.0, 204.0, 135.0, 42.0),
                filter == self.filter,
            ) {
                self.filter = filter;
            }
        }
        ui.text("6 LOCAL SCENARIOS", 714.0, 218.0, 12.0, MUTED);
        ui.panel(rect(36.0, 267.0, 854.0, 471.0));
        ui.text("SCENARIO", 62.0, 282.0, 12.0, MUTED);
        ui.text("TYPE", 516.0, 282.0, 12.0, MUTED);
        ui.text("BEST SCORE", 750.0, 282.0, 12.0, MUTED);
        for (row, drill) in Drill::ALL
            .into_iter()
            .filter(|d| self.filter.accepts(*d))
            .enumerate()
        {
            let y = 315.0 + row as f32 * 69.0;
            let area = rect(37.0, y, 852.0, 68.0);
            if drill == self.selected {
                ui.fill(area, Color::new(48, 49, 47, 255));
                ui.fill(rect(37.0, y, 3.0, 68.0), ACCENT);
            } else if ui.hovered(area) {
                ui.fill(area, RAISED);
            }
            let color = match drill.mode() {
                Mode::Click => ACCENT,
                Mode::Track => Color::new(101, 194, 232, 255),
                Mode::Switch => Color::new(180, 147, 235, 255),
            };
            ui.draw
                .draw_circle_lines(73, (y + 32.0) as i32, 11.0, color);
            ui.draw.draw_circle(73, (y + 32.0) as i32, 3.0, color);
            ui.strong(drill.name(), 100.0, y + 12.0, 18.0, TEXT);
            ui.text(
                &format!(
                    "{} target{}  /  60 seconds",
                    drill.count(),
                    if drill.count() > 1 { "s" } else { "" }
                ),
                100.0,
                y + 38.0,
                12.0,
                MUTED,
            );
            ui.text(drill.category(), 516.0, y + 27.0, 12.0, color);
            let best = self.store.best(drill);
            ui.strong(
                &if best > 0.0 {
                    format!("{best:.0}")
                } else {
                    "--".into()
                },
                766.0,
                y + 23.0,
                20.0,
                if best > 0.0 { TEXT } else { MUTED },
            );
            ui.fill(rect(58.0, y + 67.0, 809.0, 1.0), BORDER);
            if ui.clicked(area) {
                self.selected = drill;
                self.preview = Session::new(drill, true, 712);
            }
        }
        ui.panel(rect(918.0, 113.0, 486.0, 625.0));
        ui.text("SCENARIO PREVIEW", 942.0, 134.0, 12.0, MUTED);
        ui.draw.draw_texture_pro(
            preview.texture(),
            rect(0.0, 0.0, 768.0, -432.0),
            rect(942.0, 165.0, 438.0, 246.0),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
        ui.fill(rect(954.0, 178.0, 113.0, 25.0), Color::new(24, 31, 39, 230));
        ui.text(self.selected.category(), 963.0, 184.0, 11.0, ACCENT);
        ui.strong(self.selected.name(), 942.0, 435.0, 27.0, TEXT);
        ui.wrapped(
            self.selected.description(),
            942.0,
            483.0,
            430.0,
            16.0,
            MUTED,
        );
        ui.text(self.selected.scoring(), 942.0, 576.0, 13.0, MUTED);
        if ui.button("CHALLENGE", rect(942.0, 621.0, 438.0, 46.0), true) {
            action = Action::Start(false);
        }
        if ui.button("FREE PLAY", rect(942.0, 679.0, 438.0, 38.0), false) {
            action = Action::Start(true);
        }
        ui.panel(rect(36.0, 758.0, 1368.0, 71.0));
        let settings = &self.store.settings;
        ui.text("MOUSE", 57.0, 778.0, 11.0, MUTED);
        ui.strong(
            &format!("{}  /  {:.3}", settings.scale.name(), settings.sensitivity),
            123.0,
            778.0,
            16.0,
            TEXT,
        );
        ui.text(
            &format!("{:.1} cm/360", settings.cm_per_turn()),
            540.0,
            778.0,
            16.0,
            ACCENT,
        );
        ui.text(
            &format!("FOV  {:.0} horizontal", settings.fov),
            746.0,
            778.0,
            16.0,
            TEXT,
        );
        if ui.button("EDIT SETTINGS", rect(1181.0, 774.0, 200.0, 37.0), false) {
            action = Action::Settings;
        }
        action
    }

    fn settings<D: RaylibDraw>(&mut self, ui: &mut Ui<'_, D>) -> Action {
        ui.strong("Settings", 36.0, 114.0, 31.0, TEXT);
        ui.text(
            "Click a number to type an exact value. Press Enter to finish.",
            36.0,
            158.0,
            16.0,
            MUTED,
        );
        ui.panel(rect(36.0, 207.0, 666.0, 506.0));
        ui.panel(rect(738.0, 207.0, 666.0, 506.0));
        ui.strong("MOUSE & CAMERA", 64.0, 232.0, 16.0, ACCENT);
        ui.strong("DISPLAY & CROSSHAIR", 766.0, 232.0, 16.0, ACCENT);
        let s = &mut self.store.settings;
        ui.text("Sensitivity scale", 64.0, 285.0, 17.0, TEXT);
        if ui.button(s.scale.name(), rect(370.0, 276.0, 280.0, 42.0), false) {
            let yaw = s.scale.yaw();
            s.scale = s.scale.next();
            s.sensitivity = (s.sensitivity * yaw / s.scale.yaw()).clamp(0.01, 100.0);
            *ui.editor = None;
        }
        ui.number(
            "Sensitivity",
            rect(64.0, 333.0, 586.0, 52.0),
            Field::Sensitivity,
            &mut s.sensitivity,
            0.01,
            100.0,
        );
        ui.number(
            "Mouse DPI (for cm/360)",
            rect(64.0, 392.0, 586.0, 52.0),
            Field::Dpi,
            &mut s.dpi,
            100.0,
            32000.0,
        );
        ui.number(
            "Horizontal FOV",
            rect(64.0, 451.0, 586.0, 52.0),
            Field::Fov,
            &mut s.fov,
            60.0,
            140.0,
        );
        ui.fill(rect(64.0, 530.0, 586.0, 73.0), BG);
        ui.text("MOUSE TRAVEL FOR A FULL TURN", 84.0, 547.0, 11.0, MUTED);
        ui.strong(
            &format!("{:.2} cm / 360", s.cm_per_turn()),
            391.0,
            550.0,
            23.0,
            ACCENT,
        );
        ui.wrapped("Mouse capture uses raw motion when the system supports it. No aim smoothing. DPI does not change game sensitivity.", 64.0, 629.0, 586.0, 14.0, MUTED);
        ui.toggle("Fullscreen", 766.0, 274.0, &mut s.fullscreen);
        ui.toggle("VSync", 766.0, 327.0, &mut s.vsync);
        ui.text("Frame limit", 766.0, 399.0, 17.0, TEXT);
        let limit = if s.fps_limit == 0 {
            "Unlimited".into()
        } else {
            format!("{} FPS", s.fps_limit)
        };
        if ui.button(&limit, rect(1200.0, 388.0, 126.0, 40.0), false) {
            s.fps_limit = match s.fps_limit {
                144 => 240,
                240 => 360,
                360 => 500,
                500 => 0,
                _ => 144,
            };
        }
        ui.number(
            "Crosshair size",
            rect(766.0, 448.0, 560.0, 52.0),
            Field::Crosshair,
            &mut s.crosshair_size,
            1.0,
            16.0,
        );
        ui.number(
            "Crosshair gap",
            rect(766.0, 507.0, 560.0, 52.0),
            Field::Gap,
            &mut s.crosshair_gap,
            0.0,
            12.0,
        );
        ui.text("Target color", 766.0, 585.0, 17.0, TEXT);
        for (i, color) in TARGET_COLORS.into_iter().enumerate() {
            let area = rect(1091.0 + i as f32 * 49.0, 576.0, 35.0, 35.0);
            ui.fill(area, color);
            if s.target_color == i {
                ui.draw.draw_rectangle_lines_ex(
                    rect(area.x - 3.0, area.y - 3.0, 41.0, 41.0),
                    2.0,
                    TEXT,
                );
            }
            if ui.clicked(area) {
                s.target_color = i;
            }
        }
        ui.text("Crosshair color", 766.0, 644.0, 17.0, TEXT);
        for (i, color) in CROSSHAIR_COLORS.into_iter().enumerate() {
            let area = rect(1140.0 + i as f32 * 49.0, 635.0, 35.0, 35.0);
            ui.fill(area, color);
            if s.crosshair_color == i {
                ui.draw.draw_rectangle_lines_ex(
                    rect(area.x - 3.0, area.y - 3.0, 41.0, 41.0),
                    2.0,
                    ACCENT,
                );
            }
            if ui.clicked(area) {
                s.crosshair_color = i;
            }
        }
        ui.number(
            "Sound volume (0-1)",
            rect(64.0, 737.0, 586.0, 52.0),
            Field::Volume,
            &mut s.volume,
            0.0,
            1.0,
        );
        let mut action = Action::None;
        if ui.button("SAVE & BACK", rect(1184.0, 753.0, 220.0, 52.0), true) {
            action = Action::Back;
        }
        ui.text(
            "F3 toggles FPS. F11 toggles fullscreen. Esc returns.",
            766.0,
            820.0,
            12.0,
            MUTED,
        );
        action
    }

    fn history<D: RaylibDraw>(&mut self, ui: &mut Ui<'_, D>) -> Action {
        ui.strong("Local records", 36.0, 114.0, 31.0, TEXT);
        ui.text(
            "60-second challenges. The last 100 results are saved on this computer.",
            36.0,
            158.0,
            16.0,
            MUTED,
        );
        for (i, drill) in Drill::ALL.into_iter().enumerate() {
            let x = 36.0 + i as f32 * 231.0;
            ui.panel(rect(x, 210.0, 213.0, 113.0));
            ui.text(drill.name(), x + 16.0, 229.0, 15.0, TEXT);
            ui.strong(
                &format!("{:.0}", self.store.best(drill)),
                x + 16.0,
                260.0,
                33.0,
                ACCENT,
            );
        }
        ui.panel(rect(36.0, 351.0, 1368.0, 442.0));
        for (label, x) in [
            ("RECENT CHALLENGES", 61.0),
            ("SCORE", 667.0),
            ("ACCURACY", 845.0),
            ("HITS", 1056.0),
            ("WHEN", 1197.0),
        ] {
            ui.text(label, x, 373.0, 12.0, MUTED);
        }
        if self.store.results.is_empty() {
            ui.center(
                "No challenges completed yet",
                rect(36.0, 476.0, 1368.0, 55.0),
                26.0,
                TEXT,
            );
            ui.center(
                "Choose a scenario and start a 60-second challenge.",
                rect(36.0, 536.0, 1368.0, 36.0),
                16.0,
                MUTED,
            );
            if ui.button("CHOOSE A SCENARIO", rect(555.0, 604.0, 330.0, 50.0), true) {
                return Action::Library;
            }
        }
        for (i, result) in self.store.results.iter().rev().take(8).enumerate() {
            let y = 414.0 + i as f32 * 45.0;
            ui.fill(rect(61.0, y + 33.0, 1316.0, 1.0), BORDER);
            ui.text(result.drill.name(), 61.0, y, 17.0, TEXT);
            ui.strong(&format!("{:.0}", result.score), 667.0, y, 18.0, ACCENT);
            ui.text(&format!("{:.1}%", result.accuracy), 845.0, y, 17.0, TEXT);
            ui.text(
                &if result.drill.mode() == Mode::Track {
                    format!("{:.1}s", result.tracking_seconds)
                } else {
                    result.hits.to_string()
                },
                1056.0,
                y,
                17.0,
                TEXT,
            );
            let age = timestamp().saturating_sub(result.timestamp);
            let ago = if age < 60 {
                "Just now".into()
            } else if age < 3600 {
                format!("{} min ago", age / 60)
            } else if age < 86400 {
                format!("{} hr ago", age / 3600)
            } else {
                format!("{} days ago", age / 86400)
            };
            ui.text(&ago, 1197.0, y, 15.0, MUTED);
        }
        Action::None
    }

    fn play<D: RaylibDraw>(&self, ui: &mut Ui<'_, D>) -> Action {
        let session = &self.session;
        if session.phase == Phase::Paused {
            ui.fill(
                rect(-1000.0, -1000.0, 3440.0, 2900.0),
                Color::new(10, 15, 22, 210),
            );
            ui.panel(rect(475.0, 199.0, 490.0, 481.0));
            ui.center("PAUSED", rect(475.0, 231.0, 490.0, 50.0), 36.0, TEXT);
            ui.center(
                session.drill.name(),
                rect(475.0, 292.0, 490.0, 35.0),
                17.0,
                MUTED,
            );
            if ui.button("RESUME", rect(520.0, 361.0, 400.0, 52.0), true) {
                return Action::Resume;
            }
            if ui.button("RESTART", rect(520.0, 427.0, 400.0, 46.0), false) {
                return Action::Restart;
            }
            if ui.button("SETTINGS", rect(520.0, 487.0, 400.0, 46.0), false) {
                return Action::Settings;
            }
            if ui.button("SCENARIOS", rect(520.0, 547.0, 400.0, 46.0), false) {
                return Action::Library;
            }
            ui.center(
                "Time is stopped. Esc resumes. R restarts.",
                rect(475.0, 618.0, 490.0, 30.0),
                13.0,
                MUTED,
            );
        } else if session.phase == Phase::Finished {
            return self.results(ui);
        } else {
            ui.fill(rect(31.0, 29.0, 294.0, 65.0), Color::new(18, 24, 32, 223));
            ui.strong(session.drill.name(), 49.0, 42.0, 19.0, TEXT);
            ui.text(
                if session.free_play {
                    "FREE PLAY"
                } else {
                    "CHALLENGE / 60 SECONDS"
                },
                49.0,
                73.0,
                11.0,
                ACCENT,
            );
            ui.fill(rect(596.0, 25.0, 248.0, 81.0), Color::new(18, 24, 32, 223));
            let time = if session.free_play {
                session.elapsed
            } else {
                (CHALLENGE_SECONDS - session.elapsed).max(0.0)
            };
            ui.center(
                &format!(
                    "{:02}:{:02}",
                    time.ceil() as u32 / 60,
                    time.ceil() as u32 % 60
                ),
                rect(596.0, 26.0, 248.0, 55.0),
                32.0,
                TEXT,
            );
            ui.center(
                if session.free_play {
                    "ELAPSED"
                } else {
                    "REMAINING"
                },
                rect(596.0, 78.0, 248.0, 19.0),
                10.0,
                MUTED,
            );
            ui.fill(rect(1078.0, 29.0, 329.0, 65.0), Color::new(18, 24, 32, 223));
            ui.text("SCORE", 1097.0, 41.0, 10.0, MUTED);
            ui.strong(
                &format!("{:.0}", session.score()),
                1097.0,
                62.0,
                23.0,
                ACCENT,
            );
            ui.text("ACCURACY", 1242.0, 41.0, 10.0, MUTED);
            ui.strong(
                &format!("{:.1}%", session.accuracy()),
                1242.0,
                62.0,
                23.0,
                TEXT,
            );
            ui.fill(rect(499.0, 846.0, 442.0, 30.0), Color::new(18, 24, 32, 180));
            ui.center(
                "WASD  Move     LMB  Fire     R  Restart     Esc  Pause",
                rect(499.0, 846.0, 442.0, 30.0),
                11.0,
                TEXT,
            );
            if self.store.settings.show_fps {
                ui.fill(rect(31.0, 823.0, 236.0, 54.0), Color::new(18, 24, 32, 200));
                ui.text(
                    &format!("{} FPS  /  {:.2} ms frame", self.fps, self.frame_ms),
                    44.0,
                    834.0,
                    13.0,
                    GREEN,
                );
                ui.text("F3  Hide timing", 44.0, 858.0, 10.0, MUTED);
            }
            if session.phase == Phase::Countdown {
                ui.fill(
                    rect(539.0, 249.0, 362.0, 157.0),
                    Color::new(18, 24, 32, 210),
                );
                ui.center("GET READY", rect(539.0, 267.0, 362.0, 30.0), 17.0, ACCENT);
                ui.center(
                    &format!("{:.0}", session.countdown.ceil()),
                    rect(539.0, 303.0, 362.0, 91.0),
                    70.0,
                    TEXT,
                );
                ui.center(
                    session.drill.cue(),
                    rect(459.0, 502.0, 522.0, 38.0),
                    16.0,
                    BG,
                );
            }
            if session.drill.mode() == Mode::Switch
                && let Some(index) = session.nearest_hit()
            {
                ui.fill(rect(690.0, 482.0, 60.0, 5.0), BG);
                ui.fill(
                    rect(
                        690.0,
                        482.0,
                        60.0 * (session.targets[index].health / 0.3) as f32,
                        5.0,
                    ),
                    ACCENT,
                );
            }
        }
        Action::None
    }

    fn results<D: RaylibDraw>(&self, ui: &mut Ui<'_, D>) -> Action {
        let s = &self.session;
        ui.fill(
            rect(-1000.0, -1000.0, 3440.0, 2900.0),
            Color::new(10, 15, 22, 215),
        );
        ui.panel(rect(350.0, 135.0, 740.0, 630.0));
        ui.center(
            "CHALLENGE COMPLETE",
            rect(350.0, 166.0, 740.0, 34.0),
            14.0,
            MUTED,
        );
        ui.center(s.drill.name(), rect(350.0, 208.0, 740.0, 46.0), 28.0, TEXT);
        ui.center(
            &format!("{:.0}", s.score()),
            rect(350.0, 274.0, 740.0, 95.0),
            76.0,
            ACCENT,
        );
        ui.center(
            if s.score() > self.best_before {
                "NEW PERSONAL BEST"
            } else {
                "FINAL SCORE"
            },
            rect(350.0, 373.0, 740.0, 24.0),
            13.0,
            if s.score() > self.best_before {
                GREEN
            } else {
                MUTED
            },
        );
        let metrics = [
            ("ACCURACY", format!("{:.1}%", s.accuracy())),
            (
                if s.drill.mode() == Mode::Track {
                    "ON TARGET"
                } else {
                    "TARGETS HIT"
                },
                if s.drill.mode() == Mode::Track {
                    format!("{:.2}s", s.on_target)
                } else {
                    s.hits.to_string()
                },
            ),
            (
                "PERSONAL BEST",
                format!("{:.0}", s.score().max(self.best_before)),
            ),
        ];
        for (i, (label, value)) in metrics.into_iter().enumerate() {
            let x = 389.0 + i as f32 * 229.0;
            ui.fill(rect(x, 433.0, 205.0, 83.0), BG);
            ui.center(label, rect(x, 442.0, 205.0, 21.0), 10.0, MUTED);
            ui.center(&value, rect(x, 471.0, 205.0, 29.0), 26.0, TEXT);
        }
        ui.text("RECENT SCORES", 393.0, 543.0, 11.0, MUTED);
        let history: Vec<f64> = self
            .store
            .results
            .iter()
            .rev()
            .filter(|r| r.drill == s.drill)
            .take(14)
            .map(|r| r.score)
            .collect();
        let high = history.iter().copied().fold(1.0, f64::max);
        for (i, value) in history.iter().rev().enumerate() {
            let height = (*value / high * 56.0).max(2.0) as f32;
            ui.fill(
                rect(393.0 + i as f32 * 46.5, 631.0 - height, 31.0, height),
                if i + 1 == history.len() {
                    ACCENT
                } else {
                    BORDER
                },
            );
        }
        ui.fill(rect(393.0, 632.0, 653.0, 1.0), BORDER);
        if ui.button("PLAY AGAIN  [R]", rect(389.0, 672.0, 316.0, 52.0), true) {
            return Action::Restart;
        }
        if ui.button("SCENARIOS", rect(730.0, 672.0, 316.0, 52.0), false) {
            return Action::Library;
        }
        Action::None
    }
}
