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
        })
    }
    pub fn before(&mut self, app: &mut App) {
        match self.frame {
            15 => app.act(Action::Settings),
            30 => app.act(Action::Back),
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
        &self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        name: &str,
    ) -> Result<(), Box<dyn Error>> {
        let path = self.directory.join(format!("{name}.png"));
        crate::screenshot(rl, thread, &path)?;
        Ok(())
    }
    pub fn after(
        &mut self,
        app: &App,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
    ) -> Result<bool, Box<dyn Error>> {
        match self.frame {
            10 => self.screenshot(rl, thread, "library")?,
            25 => self.screenshot(rl, thread, "settings")?,
            50 => self.screenshot(rl, thread, "countdown")?,
            _ => {}
        }
        if self.frame >= 45 {
            if self.play_frame == 75 {
                self.screenshot(rl, thread, &format!("pause-{}", self.drill))?;
            }
            if self.play_frame == 95 {
                self.screenshot(rl, thread, &format!("play-{}", self.drill))?;
            }
            if app.session.phase == Phase::Finished && self.completed == self.drill {
                if app.session.score() <= 0.0 {
                    return Err(format!("No score in {}", app.session.drill.name()).into());
                }
                self.screenshot(rl, thread, &format!("result-{}", self.drill))?;
                self.completed += 1;
                self.play_frame = 0;
                println!(
                    "PASS: {} / score {:.0} / accuracy {:.1}%",
                    app.session.drill.name(),
                    app.session.score(),
                    app.session.accuracy()
                );
            }
            if self.completed == Drill::ALL.len() {
                let loaded = crate::storage::Store::from_paths(
                    self.directory.join("config"),
                    self.directory.join("state"),
                );
                if !Drill::ALL.iter().all(|drill| loaded.best(*drill) > 0.0) {
                    return Err("Scores did not survive reload".into());
                }
                fs::write(
                    self.directory.join("report.json"),
                    serde_json::to_vec_pretty(&loaded.results)?,
                )?;
                println!(
                    "PASS: six native scenarios, countdown, pause/resume, results, disk reload, GPU screenshots"
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
