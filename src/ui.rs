use raylib::prelude::*;

pub const BG: Color = Color::new(15, 15, 15, 255);
pub const PANEL: Color = Color::new(31, 31, 31, 255);
pub const RAISED: Color = Color::new(43, 43, 43, 255);
pub const BORDER: Color = Color::new(62, 62, 62, 255);
pub const TEXT: Color = Color::new(245, 245, 245, 255);
pub const MUTED: Color = Color::new(170, 170, 170, 255);
pub const ACCENT: Color = Color::new(255, 92, 92, 255);
pub const ACTION: Color = Color::new(204, 0, 0, 255);
pub const HOVER: Color = Color::new(61, 61, 61, 255);
pub const SELECTED: Color = Color::new(62, 35, 35, 255);
pub const ERROR: Color = Color::new(255, 155, 155, 255);
pub const GREEN: Color = Color::new(107, 211, 157, 255);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Sensitivity,
    Dpi,
    Fov,
    Crosshair,
    Gap,
    Volume,
}

pub struct Editor {
    pub field: Field,
    pub text: String,
    pub replace: bool,
}

#[derive(Default)]
pub struct UiInput {
    pub mouse: Vector2,
    pub clicked: bool,
    pub chars: String,
    pub backspace: bool,
    pub enter: bool,
    pub escape: bool,
}

pub struct Ui<'a, D: RaylibDraw> {
    pub draw: &'a mut D,
    pub regular: &'a Font,
    pub bold: &'a Font,
    pub input: &'a UiInput,
    pub editor: &'a mut Option<Editor>,
}

pub fn rect(x: f32, y: f32, w: f32, h: f32) -> Rectangle {
    Rectangle::new(x, y, w, h)
}

impl<D: RaylibDraw> Ui<'_, D> {
    pub fn fill(&mut self, area: Rectangle, color: Color) {
        self.draw.draw_rectangle_rec(area, color);
    }
    pub fn rounded(&mut self, area: Rectangle, radius: f32, color: Color) {
        let roundness = (2.0 * radius / area.width.min(area.height)).min(1.0);
        self.draw.draw_rectangle_rounded(area, roundness, 8, color);
    }
    pub fn outline(&mut self, area: Rectangle, radius: f32, width: f32, color: Color) {
        let roundness = (2.0 * radius / area.width.min(area.height)).min(1.0);
        self.draw
            .draw_rectangle_rounded_lines_ex(area, roundness, 8, width, color);
    }
    pub fn panel(&mut self, area: Rectangle) {
        self.rounded(area, 12.0, PANEL);
        self.outline(area, 12.0, 1.0, BORDER);
    }
    pub fn text(&mut self, label: &str, x: f32, y: f32, size: f32, color: Color) {
        self.draw
            .draw_text_ex(self.regular, label, Vector2::new(x, y), size, 0.0, color);
    }
    pub fn strong(&mut self, label: &str, x: f32, y: f32, size: f32, color: Color) {
        self.draw
            .draw_text_ex(self.bold, label, Vector2::new(x, y), size, 0.0, color);
    }
    pub fn center(&mut self, label: &str, area: Rectangle, size: f32, color: Color) {
        let width = self.bold.measure_text(label, size, 0.0).x;
        self.strong(
            label,
            area.x + (area.width - width) * 0.5,
            area.y + (area.height - size) * 0.5,
            size,
            color,
        );
    }
    pub fn wrapped(&mut self, label: &str, x: f32, y: f32, width: f32, size: f32, color: Color) {
        let mut line = String::new();
        let mut current_y = y;
        for word in label.split_whitespace() {
            let next = if line.is_empty() {
                word.to_string()
            } else {
                format!("{line} {word}")
            };
            if self.regular.measure_text(&next, size, 0.0).x > width && !line.is_empty() {
                self.text(&line, x, current_y, size, color);
                current_y += size * 1.55;
                line = word.to_string();
            } else {
                line = next;
            }
        }
        self.text(&line, x, current_y, size, color);
    }
    pub fn hovered(&self, area: Rectangle) -> bool {
        area.check_collision_point_rec(self.input.mouse)
    }
    pub fn clicked(&self, area: Rectangle) -> bool {
        self.input.clicked && self.hovered(area)
    }
    pub fn button(&mut self, label: &str, area: Rectangle, primary: bool) -> bool {
        let hover = self.hovered(area);
        let color = if primary {
            if hover {
                Color::new(230, 24, 24, 255)
            } else {
                ACTION
            }
        } else if hover {
            HOVER
        } else {
            RAISED
        };
        self.rounded(area, 10.0, color);
        if !primary {
            self.outline(area, 10.0, 1.0, BORDER);
        }
        self.center(label, area, 16.0, TEXT);
        self.clicked(area)
    }
    pub fn tab(&mut self, label: &str, area: Rectangle, active: bool) -> bool {
        let chip = rect(
            area.x + 4.0,
            area.y + (area.height - 36.0) * 0.5,
            area.width - 8.0,
            36.0,
        );
        if active || self.hovered(area) {
            self.rounded(chip, 18.0, if active { SELECTED } else { HOVER });
        }
        if active {
            self.outline(chip, 18.0, 1.0, ACCENT);
        }
        self.center(label, area, 15.0, if active { TEXT } else { MUTED });
        self.clicked(area)
    }
    pub fn toggle(&mut self, label: &str, x: f32, y: f32, value: &mut bool) {
        let area = rect(x, y, 560.0, 54.0);
        self.text(label, x, y + 16.0, 17.0, TEXT);
        let button = rect(x + 434.0, y + 6.0, 126.0, 40.0);
        self.rounded(
            button,
            20.0,
            if *value {
                ACTION
            } else if self.hovered(area) {
                HOVER
            } else {
                RAISED
            },
        );
        if self.hovered(area) {
            self.outline(button, 20.0, 1.0, TEXT);
        }
        self.center(if *value { "ON" } else { "OFF" }, button, 15.0, TEXT);
        if self.clicked(area) {
            *value = !*value;
        }
    }
    pub fn number(
        &mut self,
        label: &str,
        area: Rectangle,
        field: Field,
        value: &mut f32,
        min: f32,
        max: f32,
    ) {
        self.text(label, area.x, area.y + 14.0, 17.0, TEXT);
        let entry = rect(area.x + area.width - 126.0, area.y + 4.0, 126.0, 42.0);
        if self.clicked(entry) {
            *self.editor = Some(Editor {
                field,
                text: format!("{value:.3}")
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_string(),
                replace: true,
            });
        }
        let active = self.editor.as_ref().is_some_and(|edit| edit.field == field);
        let mut display = format!("{value:.3}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();
        let mut valid = true;
        if active {
            if let Some(edit) = self.editor.as_mut() {
                for ch in self
                    .input
                    .chars
                    .chars()
                    .filter(|ch| ch.is_ascii_digit() || *ch == '.')
                {
                    if edit.replace {
                        edit.text.clear();
                        edit.replace = false;
                    }
                    if edit.text.len() < 12 {
                        edit.text.push(ch);
                    }
                }
                if self.input.backspace {
                    if edit.replace {
                        edit.text.clear();
                        edit.replace = false;
                    } else {
                        edit.text.pop();
                    }
                }
                if let Ok(parsed) = edit.text.parse::<f32>() {
                    valid = parsed.is_finite() && (min..=max).contains(&parsed);
                    if valid {
                        *value = parsed;
                    }
                } else {
                    valid = false;
                }
                display = format!("{}|", edit.text);
            }
            if self.input.enter || self.input.escape || (self.input.clicked && !self.hovered(entry))
            {
                *self.editor = None;
            }
        }
        self.rounded(entry, 8.0, BG);
        self.outline(
            entry,
            8.0,
            if active { 2.0 } else { 1.0 },
            if active {
                if valid { ACCENT } else { ERROR }
            } else if self.hovered(entry) {
                MUTED
            } else {
                BORDER
            },
        );
        self.center(&display, entry, 18.0, TEXT);
    }
}
