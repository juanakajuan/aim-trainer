mod app;
mod model;
mod qa;
mod storage;
mod ui;
mod world;

use app::{Action, App, Screen};
use glam::Vec2;
use model::{Input, Phase};
use raylib::prelude::*;
use std::{
    env,
    error::Error,
    path::{Path, PathBuf},
};
use ui::{Ui, UiInput};

fn main() {
    if let Err(error) = run() {
        eprintln!("Aim Trainer: {error}");
        std::process::exit(1);
    }
}

fn sound<'a>(audio: &'a RaylibAudio, bytes: &[u8]) -> Option<Sound<'a>> {
    let wave = audio.new_wave_from_memory(".wav", bytes).ok()?;
    audio.new_sound_from_wave(&wave).ok()
}

fn screenshot(rl: &RaylibHandle, thread: &RaylibThread, path: &Path) -> Result<(), Box<dyn Error>> {
    let image = rl.load_image_from_screen(thread);
    let png = image.export_image_to_memory(".png")?;
    std::fs::write(path, &*png)?;
    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let smoke_dir = match args.next().as_deref() {
        None => None,
        Some("--smoke-test") => Some(PathBuf::from(
            args.next()
                .ok_or("--smoke-test needs an output directory")?,
        )),
        Some("--help" | "-h") => {
            println!(
                "Aim Trainer - native Linux aim trainer\n\nRun: aim-trainer\nVerify GPU and gameplay: aim-trainer --smoke-test <output-directory>"
            );
            return Ok(());
        }
        Some(arg) => return Err(format!("Unknown argument: {arg}").into()),
    };
    let store = if let Some(dir) = &smoke_dir {
        storage::Store::from_paths(dir.join("config"), dir.join("state"))
    } else {
        storage::Store::load()
    };
    let mut app = App::new(store);
    let mut smoke = smoke_dir.map(qa::Smoke::new).transpose()?;
    let (mut rl, thread) = raylib::init()
        .size(1440, 900)
        .title("Aim Trainer")
        .resizable()
        .msaa_4x()
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();
    rl.set_exit_key(None);
    rl.set_window_min_size(1024, 640);
    let regular = rl.load_font_from_memory(
        &thread,
        ".ttf",
        include_bytes!("../assets/DejaVuSans.ttf"),
        64,
        None,
    )?;
    let bold = rl.load_font_from_memory(
        &thread,
        ".ttf",
        include_bytes!("../assets/DejaVuSans-Bold.ttf"),
        80,
        None,
    )?;
    regular
        .texture()
        .set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
    bold.texture()
        .set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
    let audio = RaylibAudio::init_audio_device().ok();
    let hit_sound = audio
        .as_ref()
        .and_then(|audio| sound(audio, include_bytes!("../assets/hit.wav")));
    let shot_sound = audio
        .as_ref()
        .and_then(|audio| sound(audio, include_bytes!("../assets/shot.wav")));
    let ready_sound = audio
        .as_ref()
        .and_then(|audio| sound(audio, include_bytes!("../assets/ready.wav")));
    let mut world = world::World::new(&mut rl, &thread);
    let mut preview = rl.load_render_texture(&thread, 768, 432)?;
    preview
        .texture()
        .set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
    let mut editor = None;
    let mut captured = false;
    let mut ignore_input = 0_u32;
    let mut applied_fullscreen = false;
    let mut applied_vsync = false;
    let mut hit_cooldown = 0.0_f64;
    let mut stats_time = 0.0_f64;
    let mut stats_frames = 0_u32;
    let mut capture_index = 0_u32;

    while !rl.window_should_close() {
        let real_dt = f64::from(rl.get_frame_time()).max(0.000_001);
        stats_time += real_dt;
        stats_frames += 1;
        if stats_time >= 0.5 {
            app.frame_ms = (stats_time / f64::from(stats_frames) * 1000.0) as f32;
            app.fps = (f64::from(stats_frames) / stats_time).round() as i32;
            stats_time = 0.0;
            stats_frames = 0;
        }
        if let Some(qa) = smoke.as_mut() {
            qa.before(&mut app);
        }
        let mut action = Action::None;
        if editor.is_none() {
            if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                match app.screen {
                    Screen::Play if app.active() => app.session.pause(),
                    Screen::Play if app.session.phase == Phase::Paused => action = Action::Resume,
                    Screen::Play => action = Action::Library,
                    Screen::Settings => action = Action::Back,
                    Screen::History => action = Action::Library,
                    Screen::Library => {}
                }
            }
            if rl.is_key_pressed(KeyboardKey::KEY_R) && app.screen == Screen::Play {
                action = Action::Restart;
            }
            if rl.is_key_pressed(KeyboardKey::KEY_ENTER) && app.screen == Screen::Library {
                action = Action::Start(false);
            }
            if rl.is_key_pressed(KeyboardKey::KEY_F2) && app.screen != Screen::Settings {
                action = Action::Settings;
            }
            if rl.is_key_pressed(KeyboardKey::KEY_F3) {
                app.store.settings.show_fps = !app.store.settings.show_fps;
                app.store.save_settings();
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F11) {
            app.store.settings.fullscreen = !app.store.settings.fullscreen;
            app.store.save_settings();
        }
        app.act(action);
        if !rl.is_window_focused() && smoke.is_none() && app.active() {
            app.session.pause();
        }
        let active = app.active();
        if active != captured {
            if active {
                rl.disable_cursor();
            } else {
                rl.enable_cursor();
            }
            captured = active;
            ignore_input = 2;
        }
        let mouse = rl.get_mouse_delta();
        let mut input = Input {
            look: Vec2::new(mouse.x, mouse.y),
            movement: Vec2::new(
                f32::from(rl.is_key_down(KeyboardKey::KEY_D))
                    - f32::from(rl.is_key_down(KeyboardKey::KEY_A)),
                f32::from(rl.is_key_down(KeyboardKey::KEY_W))
                    - f32::from(rl.is_key_down(KeyboardKey::KEY_S)),
            ),
            click: rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT),
            firing: rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT),
        };
        if ignore_input > 0 {
            input = Input::default();
            ignore_input -= 1;
        }
        let dt = if let Some(qa) = smoke.as_ref() {
            input = qa.input(&app);
            0.1
        } else {
            real_dt
        };
        let old_hits = app.session.hits;
        let old_shots = app.session.shots;
        let old_on_target = app.session.on_target;
        let old_countdown = app.session.countdown.ceil();
        if active {
            app.session.tick(dt, &input, &app.store.settings);
        }
        app.record_finished();
        hit_cooldown -= real_dt;
        if smoke.is_none() {
            if app.session.hits > old_hits
                || (app.session.on_target > old_on_target && hit_cooldown <= 0.0)
            {
                if let Some(sound) = &hit_sound {
                    sound.set_volume(app.store.settings.volume);
                    sound.play();
                }
                hit_cooldown = 0.085;
            } else if app.session.shots > old_shots
                && let Some(sound) = &shot_sound
            {
                sound.set_volume(app.store.settings.volume * 0.32);
                sound.play();
            }
            if app.session.countdown.ceil() < old_countdown
                && let Some(sound) = &ready_sound
            {
                sound.set_volume(app.store.settings.volume * 0.55);
                sound.play();
            }
        }
        if !app.active() && captured {
            rl.enable_cursor();
            captured = false;
        }
        if app.store.settings.fullscreen != applied_fullscreen && smoke.is_none() {
            rl.toggle_borderless_windowed();
            applied_fullscreen = app.store.settings.fullscreen;
        }
        if app.store.settings.vsync != applied_vsync {
            let flag = WindowState::default().set_vsync_hint(true);
            if app.store.settings.vsync {
                rl.set_window_state(flag);
            } else {
                rl.clear_window_state(flag);
            }
            applied_vsync = app.store.settings.vsync;
        }
        rl.set_target_fps(if smoke.is_some() {
            240
        } else if app.active() {
            app.store.settings.fps_limit
        } else {
            60
        });
        let width = rl.get_screen_width() as f32;
        let height = rl.get_screen_height() as f32;
        let scale = (width / 1440.0).min(height / 900.0);
        let offset = Vector2::new(
            (width - 1440.0 * scale) * 0.5,
            (height - 900.0 * scale) * 0.5,
        );
        let mouse = rl.get_mouse_position();
        let mut chars = String::new();
        while let Some(ch) = rl.get_char_pressed() {
            chars.push(ch);
        }
        let ui_input = UiInput {
            mouse: Vector2::new((mouse.x - offset.x) / scale, (mouse.y - offset.y) / scale),
            clicked: !captured && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT),
            chars,
            backspace: rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE),
            enter: rl.is_key_pressed(KeyboardKey::KEY_ENTER),
            escape: rl.is_key_pressed(KeyboardKey::KEY_ESCAPE),
        };
        if app.screen == Screen::Library {
            let mut texture = rl.begin_texture_mode(&thread, &mut preview);
            texture.clear_background(ui::BG);
            world.draw(&mut texture, &app.preview, &app.store.settings, 16.0 / 9.0);
        }
        let screen_action;
        {
            let mut draw = rl.begin_drawing(&thread);
            draw.clear_background(ui::BG);
            if app.screen == Screen::Play {
                world.draw(&mut draw, &app.session, &app.store.settings, width / height);
            }
            {
                let mut canvas = draw.begin_mode2D(Camera2D {
                    offset,
                    target: Vector2::zero(),
                    rotation: 0.0,
                    zoom: scale,
                });
                let mut ui = Ui {
                    draw: &mut canvas,
                    regular: &regular,
                    bold: &bold,
                    input: &ui_input,
                    editor: &mut editor,
                };
                screen_action = app.draw(&mut ui, &preview);
            }
            if app.active() {
                crosshair(
                    &mut draw,
                    width * 0.5,
                    height * 0.5,
                    &app.store.settings,
                    app.session.hit_flash > 0.0,
                );
            }
        }
        if !matches!(screen_action, Action::None) {
            editor = None;
            if app.screen == Screen::Settings && !matches!(screen_action, Action::Back) {
                app.store.save_settings();
            }
        }
        if matches!(screen_action, Action::Quit) {
            break;
        }
        app.act(screen_action);
        if rl.is_key_pressed(KeyboardKey::KEY_F12) {
            let dir = env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(env::temp_dir)
                .join("Pictures/Aim Trainer");
            std::fs::create_dir_all(&dir)?;
            let path = dir.join(format!(
                "aim-trainer-{}-{capture_index}.png",
                storage::timestamp()
            ));
            if let Err(error) = screenshot(&rl, &thread, &path) {
                app.store.notice = Some(format!("Screenshot was not saved: {error}"));
            }
            capture_index += 1;
        }
        if let Some(qa) = smoke.as_mut()
            && qa.after(&app, &mut rl, &thread)?
        {
            break;
        }
    }
    rl.enable_cursor();
    app.store.save_settings();
    Ok(())
}

fn crosshair<D: RaylibDraw>(draw: &mut D, x: f32, y: f32, settings: &model::Settings, hit: bool) {
    let size = settings.crosshair_size;
    let gap = settings.crosshair_gap;
    let color = world::CROSSHAIR_COLORS[settings.crosshair_color];
    for area in [
        ui::rect(x - gap - size, y - 1.0, size, 2.0),
        ui::rect(x + gap, y - 1.0, size, 2.0),
        ui::rect(x - 1.0, y - gap - size, 2.0, size),
        ui::rect(x - 1.0, y + gap, 2.0, size),
    ] {
        draw.draw_rectangle_rec(
            ui::rect(
                area.x - 1.0,
                area.y - 1.0,
                area.width + 2.0,
                area.height + 2.0,
            ),
            Color::new(0, 0, 0, 210),
        );
        draw.draw_rectangle_rec(area, color);
    }
    draw.draw_rectangle_rec(ui::rect(x - 1.0, y - 1.0, 2.0, 2.0), color);
    if hit {
        for (dx, dy) in [(-1.0, -1.0), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)] {
            draw.draw_line_ex(
                Vector2::new(x + dx * 14.0, y + dy * 14.0),
                Vector2::new(x + dx * 18.0, y + dy * 18.0),
                2.0,
                ui::ACCENT,
            );
        }
    }
}
