use crate::model::{Drill, Session, Settings, vertical_fov};
use raylib::prelude::*;

pub const TARGET_COLORS: [Color; 5] = [
    Color::new(236, 62, 83, 255),
    Color::new(35, 195, 235, 255),
    Color::new(174, 112, 247, 255),
    Color::new(245, 175, 47, 255),
    Color::new(101, 224, 149, 255),
];
pub const CROSSHAIR_COLORS: [Color; 4] = [
    Color::new(235, 255, 235, 255),
    Color::new(88, 255, 143, 255),
    Color::new(74, 223, 255, 255),
    Color::new(255, 203, 72, 255),
];

fn vector(v: glam::Vec3) -> Vector3 {
    Vector3::new(v.x, v.y, v.z)
}

pub struct World {
    shader: Shader,
}

impl World {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let vertex = "#version 330\nin vec3 vertexPosition;\nin vec3 vertexNormal;\nin vec4 vertexColor;\nuniform mat4 mvp;\nout vec3 normal;\nout vec4 color;\nvoid main(){ normal=vertexNormal; color=vertexColor; gl_Position=mvp*vec4(vertexPosition,1.0); }";
        let fragment = "#version 330\nin vec3 normal;\nin vec4 color;\nout vec4 finalColor;\nvoid main(){ vec3 n=normalize(normal); float light=max(dot(n,normalize(vec3(-0.4,0.8,0.6))),0.0); float gloss=pow(max(dot(n,normalize(vec3(-0.3,0.6,0.8))),0.0),40.0); finalColor=vec4(color.rgb*(0.58+light*0.42)+vec3(gloss*0.18),color.a); }";
        Self {
            shader: rl.load_shader_from_memory(thread, Some(vertex), Some(fragment)),
        }
    }

    pub fn draw<D: RaylibDraw>(
        &mut self,
        draw: &mut D,
        session: &Session,
        settings: &Settings,
        aspect: f32,
    ) {
        let camera = Camera3D::perspective(
            vector(session.position),
            vector(session.position + session.direction()),
            Vector3::new(0.0, 1.0, 0.0),
            vertical_fov(settings.fov, aspect),
        );
        let mut scene = draw.begin_mode3D(camera);
        scene.draw_cube(
            Vector3::new(0.0, -0.12, -0.5),
            20.0,
            0.2,
            23.0,
            Color::new(120, 128, 137, 255),
        );
        scene.draw_cube(
            Vector3::new(0.0, 4.0, -11.0),
            20.0,
            8.0,
            0.15,
            Color::new(163, 170, 178, 255),
        );
        scene.draw_cube(
            Vector3::new(-10.0, 4.0, -0.5),
            0.15,
            8.0,
            23.0,
            Color::new(135, 144, 154, 255),
        );
        scene.draw_cube(
            Vector3::new(10.0, 4.0, -0.5),
            0.15,
            8.0,
            23.0,
            Color::new(145, 153, 163, 255),
        );
        scene.draw_cube(
            Vector3::new(0.0, 8.0, -0.5),
            20.0,
            0.15,
            23.0,
            Color::new(179, 185, 192, 255),
        );
        scene.draw_cube(
            Vector3::new(0.0, 4.0, 11.0),
            20.0,
            8.0,
            0.15,
            Color::new(143, 153, 163, 255),
        );
        let floor_line = Color::new(101, 110, 121, 255);
        let wall_line = Color::new(144, 153, 162, 255);
        for x in -10..=10 {
            let x = x as f32;
            scene.draw_line3D(
                Vector3::new(x, 0.001, -10.9),
                Vector3::new(x, 0.001, 10.9),
                floor_line,
            );
            scene.draw_line3D(
                Vector3::new(x, 0.0, -10.91),
                Vector3::new(x, 8.0, -10.91),
                wall_line,
            );
        }
        for z in -10..=10 {
            let z = z as f32;
            scene.draw_line3D(
                Vector3::new(-9.91, 0.001, z),
                Vector3::new(9.91, 0.001, z),
                floor_line,
            );
            scene.draw_line3D(
                Vector3::new(-9.91, 0.0, z),
                Vector3::new(-9.91, 8.0, z),
                floor_line,
            );
            scene.draw_line3D(
                Vector3::new(9.91, 0.0, z),
                Vector3::new(9.91, 8.0, z),
                floor_line,
            );
        }
        for y in 1..8 {
            let y = y as f32;
            scene.draw_line3D(
                Vector3::new(-10.0, y, -10.91),
                Vector3::new(10.0, y, -10.91),
                wall_line,
            );
            scene.draw_line3D(
                Vector3::new(-9.91, y, -11.0),
                Vector3::new(-9.91, y, 11.0),
                floor_line,
            );
            scene.draw_line3D(
                Vector3::new(9.91, y, -11.0),
                Vector3::new(9.91, y, 11.0),
                floor_line,
            );
        }
        // Thin base rails give a stable horizon without distracting from targets.
        scene.draw_cube(
            Vector3::new(0.0, 0.12, -10.85),
            20.0,
            0.24,
            0.1,
            Color::new(73, 85, 98, 255),
        );
        let mut lit = scene.begin_shader_mode(&mut self.shader);
        let color = TARGET_COLORS[settings.target_color];
        for target in &session.targets {
            if session.drill == Drill::Frenzy {
                lit.draw_cube(
                    vector(target.position),
                    target.radius * 2.0,
                    target.radius * 2.0,
                    0.30,
                    color,
                );
            } else {
                lit.draw_sphere_ex(vector(target.position), target.radius, 20, 28, color);
            }
        }
    }
}
