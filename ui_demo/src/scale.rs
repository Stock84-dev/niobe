use egui::epaint::Galley;
use egui::{NumExt, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};
use epaint::Color32;
use epaint::Rgba;
use std::sync::Arc;

pub struct Scale {
    //    ticks: Vec<String>,
//    tick_offset: f32,
//    tick_stride: f32,
//    direction: Direction,
//    tick_length: f32,
}

impl Scale {
    pub fn new() -> Self {
        //        Self {
        //            ticks: (0..10).into_iter().map(|_| String::new()).collect(),
        //            tick_offset: 13.0,
        //            tick_stride: 10.0,
        //            direction: Direction::Left,
        //            tick_length: 5.0,
        //        }
        Self {}
    }
}

impl Widget for Scale {
    fn ui(self, ui: &mut Ui) -> Response {
        let texts = vec![
            "hello".to_string(),
            "hello world".to_string(),
            "hi".to_string(),
        ];
        let mut max_size = 0.0f32;
        let galleys: Vec<_> = texts
            .into_iter()
            .map(|x| {
                let galley = ui.fonts().layout_single_line(ui.style().body_text_style, x);
                max_size = max_size.max(galley.size.x);
                galley
            })
            .collect();
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(max_size + 10., 400.), Sense::hover());
        let mut y_pos = rect.top() + 5.;
        for galley in galleys {
            scale_entry_ui(ui, galley, rect, y_pos);
            y_pos += 50.;
        }
        let visuals = ui.style().visuals.clone();
        let painter = ui.painter();
        painter.line_segment(
            [rect.right_top(), rect.right_bottom()],
            visuals.widgets.noninteractive.bg_stroke,
        );
        resp

        //        texts
        //            .into_iter()
        //            .map(|x| scale_entry_ui(ui, x))
        //            .reduce(|r1, r2| r1.union(r2))
        //            .unwrap()
    }
}

fn scale_entry_ui(ui: &mut Ui, galley: Arc<Galley>, rect: Rect, y_pos: f32) {
    let tick_size = 5.;
    let spacing = 5.;
    let additional = tick_size + spacing;

    //    let (rect, resp) = ui.allocate_exact_size(
    //        Vec2::new(galley.size.x + additional, galley.size.y),
    //        Sense::hover(),
    //    );
    let visuals = ui.style().visuals.clone();
    let painter = ui.painter();
    let text_position = Pos2::new(rect.left(), y_pos - 0.5 * galley.size.y);
    painter.galley(text_position, galley, visuals.text_color());
    painter.line_segment(
        [
            Pos2::new(rect.right(), y_pos),
            Pos2::new(rect.right() - tick_size, y_pos),
        ],
        visuals.widgets.noninteractive.bg_stroke,
    );
}

//impl Scale {
//    pub fn draw(&mut self, drawer: &mut UiDrawer) {
//        match self.direction {
//            Direction::Left => {
//                drawer.line(
//                    Pos2::new(self.rect.max.x, self.rect.min.y),
//                    Pos2::new(self.rect.max.x, self.rect.max.y),
//                    self.line_stroke,
//                );
//                let mut line_pos = Pos2::new(self.rect.max.x, self.rect.min.y + self.tick_offset);
//                for _ in 0..self.ticks.len() {
//                    let end = Pos2::new(line_pos.x - self.tick_length, line_pos.y);
//                    drawer.line(line_pos, end, self.line_stroke);
//                    line_pos.y += self.tick_stride;
//                }
//                let mut text_pos = Pos2::new(self.rect.min.x, self.rect.min.y + self.tick_offset);
//                for text in &self.ticks {
//                    drawer.text_single_line(self.text_style, text, text_pos, self.text_color);
//                    text_pos.y += self.tick_stride;
//                }
//            }
//            Direction::Top => {
//                drawer.line(
//                    Pos2::new(self.rect.min.x, self.rect.max.y),
//                    Pos2::new(self.rect.max.x, self.rect.max.y),
//                    self.line_stroke,
//                );
//                let mut line_pos = Pos2::new(self.rect.min.x + self.tick_offset, self.rect.max.y);
//                for _ in 0..self.ticks.len() {
//                    let end = Pos2::new(line_pos.x, line_pos.y - self.tick_length);
//                    drawer.line(line_pos, end, self.line_stroke);
//                    line_pos.x += self.tick_stride;
//                }
//                let mut text_pos = Pos2::new(self.rect.min.x, self.rect.min.y + self.tick_offset);
//                for text in &self.ticks {
//                    drawer.text_single_line(self.text_style, text, text_pos, self.text_color);
//                    text_pos.y += self.tick_stride;
//                }
//            }
//            Direction::Right => {}
//            Direction::Bottom => {}
//        }
//    }
//}
//
#[derive(Clone, Copy)]
pub enum Direction {
    Left,
    Top,
    Right,
    Bottom,
}
