use crate::pipelines::ui::UiDrawer;
use epaint::emath::{Pos2, Rect, Vec2};
use epaint::{Color32, Stroke, TextStyle};

pub struct Scale {
    rect: Rect,
    background_color: Color32,
    text_color: Color32,
    line_color: Color32,
    ticks: Vec<String>,
    tick_offset: f32,
    tick_stride: f32,
    direction: Direction,
    line_stroke: Stroke,
    text_style: TextStyle,
    tick_length: f32,
}

impl Scale {
    pub fn draw(&mut self, drawer: &mut UiDrawer) {
        match self.direction {
            Direction::Left => {
                drawer.line(
                    Pos2::new(self.rect.max.x, self.rect.min.y),
                    Pos2::new(self.rect.max.x, self.rect.max.y),
                    self.line_stroke,
                );
                let mut line_pos = Pos2::new(self.rect.max.x, self.rect.min.y + self.tick_offset);
                for _ in 0..self.ticks.len() {
                    let end = Pos2::new(line_pos.x - self.tick_length, line_pos.y);
                    drawer.line(line_pos, end, self.line_stroke);
                    line_pos.y += self.tick_stride;
                }
                let mut text_pos = Pos2::new(self.rect.min.x, self.rect.min.y + self.tick_offset);
                for text in &self.ticks {
                    drawer.text_single_line(self.text_style, text, text_pos, self.text_color);
                    text_pos.y += self.tick_stride;
                }
            }
            Direction::Top => {
                drawer.line(
                    Pos2::new(self.rect.min.x, self.rect.max.y),
                    Pos2::new(self.rect.max.x, self.rect.max.y),
                    self.line_stroke,
                );
                let mut line_pos = Pos2::new(self.rect.min.x + self.tick_offset, self.rect.max.y);
                for _ in 0..self.ticks.len() {
                    let end = Pos2::new(line_pos.x, line_pos.y - self.tick_length);
                    drawer.line(line_pos, end, self.line_stroke);
                    line_pos.x += self.tick_stride;
                }
                let mut text_pos = Pos2::new(self.rect.min.x, self.rect.min.y + self.tick_offset);
                for text in &self.ticks {
                    drawer.text_single_line(self.text_style, text, text_pos, self.text_color);
                    text_pos.y += self.tick_stride;
                }
            }
            Direction::Right => {}
            Direction::Bottom => {}
        }
    }
}

#[derive(Clone, Copy)]
pub enum Direction {
    Left,
    Top,
    Right,
    Bottom,
}
