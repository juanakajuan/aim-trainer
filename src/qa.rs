//! Runs the real native renderer with controlled inputs and an isolated save directory.
use crate::{
    app::{Action, App},
    model::{Drill, Input, Phase},
};
use glam::Vec2;
use raylib::prelude::*;
use std::{error::Error, fs, path::PathBuf};

pub struct Smoke {
    directory: PathBuf,
    frame: u32,
    drill: usize,
    play_frame: u32,
    completed: usize,
    small_capture: Option<(String, u32)>,
    imported_start: Option<glam::Vec3>,
    imported_id: Option<String>,
}

impl Smoke {
    pub fn new(directory: PathBuf) -> Result<Self, Box<dyn Error>> {
        fs::create_dir_all(&directory)?;
        Ok(Self {
            directory,
            frame: 0,
            drill: 0,
            play_frame: 0,
            completed: 0,
            small_capture: None,
            imported_start: None,
            imported_id: None,
        })
    }
    pub fn before(&mut self, app: &mut App) {
        if self.small_capture.is_some() {
            return;
        }
        match self.frame {
            15 => app.act(Action::Settings),
            30 => app.act(Action::History),
            40 => {
                app.act(Action::Library);
                app.select_import(0);
                self.imported_id = app.session.scenario.as_ref().map(|s| s.id.clone());
            }
            44 => {
                app.selected = Drill::Six;
                app.preview = crate::model::Session::new(Drill::Six, true, 712);
            }
            45 => app.act(Action::Start(false)),
            _ => {}
        }
        if self.frame >= 45 {
            self.play_frame += 1;
            if self.play_frame == 70 {
                app.session.pause();
            }
            if self.play_frame == 80 {
                app.act(Action::Resume);
            }
            if self.completed > self.drill && self.play_frame > 15 {
                self.drill += 1;
                if let Some(drill) = Drill::ALL.get(self.drill) {
                    app.selected = *drill;
                    app.act(Action::Start(false));
                    self.play_frame = 0;
                } else if self.drill == Drill::ALL.len() {
                    app.select_import(0);
                    self.imported_id = app.preview.scenario.as_ref().map(|s| s.id.clone());
                    app.act(Action::Start(false));
                    self.imported_start = Some(app.session.targets[0].position);
                    self.play_frame = 0;
                }
            }
        }
    }
    pub fn input(&self, app: &App) -> Input {
        if self.frame < 45 || app.session.phase != Phase::Running {
            return Input::default();
        }
        let session = &app.session;
        let delta = (session.targets[0].position - session.position).normalize();
        let yaw = delta.x.atan2(-delta.z);
        let pitch = delta.y.asin();
        let yaw_delta = (yaw - session.yaw + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        let degrees_per_count = app.store.settings.sensitivity * app.store.settings.scale.yaw();
        Input {
            look: Vec2::new(
                yaw_delta.to_degrees() / degrees_per_count,
                -(pitch - session.pitch).to_degrees() / degrees_per_count,
            ),
            click: self.play_frame.is_multiple_of(3),
            firing: true,
            ..Input::default()
        }
    }
    fn screenshot(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        name: &str,
    ) -> Result<(), Box<dyn Error>> {
        let path = self.directory.join(format!("{name}.png"));
        crate::screenshot(rl, thread, &path)?;
        self.small_capture = Some((name.to_string(), 3));
        rl.set_window_size(1024, 640);
        Ok(())
    }
    pub fn after(
        &mut self,
        app: &mut App,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
    ) -> Result<bool, Box<dyn Error>> {
        if let Some((name, frames_left)) = self.small_capture.take() {
            // Let the resized swap chain render before reading pixels.
            if frames_left > 0 {
                self.small_capture = Some((name, frames_left - 1));
                return Ok(false);
            }
            let path = self.directory.join(format!("{name}-small.png"));
            crate::screenshot(rl, thread, &path)?;
            rl.set_window_size(1440, 900);
            return Ok(false);
        }
        match self.frame {
            10 => self.screenshot(rl, thread, "library")?,
            25 => self.screenshot(rl, thread, "settings")?,
            35 => self.screenshot(rl, thread, "records")?,
            42 => self.screenshot(rl, thread, "imported-library")?,
            50 => self.screenshot(rl, thread, "countdown")?,
            _ => {}
        }
        if self.frame >= 45 {
            if self.play_frame == 75 {
                self.screenshot(rl, thread, &format!("pause-{}", self.drill))?;
            }
            if self.drill == Drill::ALL.len() && app.session.targets.len() != 4 {
                return Err("Imported target count changed".into());
            }
            if self.drill == Drill::ALL.len()
                && self.play_frame == 65
                && self
                    .imported_start
                    .is_some_and(|p| p.distance(app.session.targets[0].position) < 0.05)
            {
                return Err("Imported target did not move".into());
            }
            if self.play_frame == 95 {
                self.screenshot(rl, thread, &format!("play-{}", self.drill))?;
            }
            if app.session.phase == Phase::Finished && self.completed == self.drill {
                if app.session.score() <= 0.0 {
                    return Err(format!("No score in {}", app.session.name()).into());
                }
                self.screenshot(rl, thread, &format!("result-{}", self.drill))?;
                self.completed += 1;
                self.play_frame = 0;
                println!(
                    "PASS: {} / score {:.0} / accuracy {:.1}%",
                    app.session.name(),
                    app.session.score(),
                    app.session.accuracy()
                );
            }
            if self.completed == Drill::ALL.len() + 1 && self.small_capture.is_none() {
                let loaded = crate::storage::Store::from_paths(
                    self.directory.join("config"),
                    self.directory.join("state"),
                );
                if !Drill::ALL.iter().all(|drill| loaded.best(*drill) > 0.0) {
                    return Err("Scores did not survive reload".into());
                }
                let id = self
                    .imported_id
                    .as_ref()
                    .ok_or("Missing imported identity")?;
                if !loaded.scenarios.iter().any(|s| &s.id == id) || loaded.best_import(id) <= 0.0 {
                    return Err("Imported scenario/results did not survive reload".into());
                }
                let s = &app.session;
                let expected = s
                    .scenario
                    .as_ref()
                    .ok_or("Missing scenario")?
                    .score(s.hits, s.shots);
                if (s.score() - expected).abs() > 1e-8 {
                    return Err("Source scoring mismatch".into());
                }
                fs::write(
                    self.directory.join("report.json"),
                    serde_json::to_vec_pretty(&loaded.results)?,
                )?;
                app.act(Action::Library);
                app.act(Action::RemoveImport);
                let removed = crate::storage::Store::from_paths(
                    self.directory.join("config"),
                    self.directory.join("state"),
                );
                if removed.scenarios.iter().any(|scenario| &scenario.id == id)
                    || app.selected == Drill::Imported
                    || app.selected_import.is_some()
                    || app.preview.scenario.is_some()
                    || removed.best_import(id) != loaded.best_import(id)
                {
                    return Err("Removal did not persist or changed result history".into());
                }
                println!("PASS: imported removal persists, selection resets, results kept");
                println!(
                    "PASS: six built-in drills plus persisted Pasu, countdown, pause/resume, results, disk reload, GPU screenshots"
                );
                return Ok(true);
            }
        }
        self.frame += 1;
        if self.frame > 6000 {
            return Err("Native smoke test timed out".into());
        }
        Ok(false)
    }
}
