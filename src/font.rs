use owned_ttf_parser::{AsFaceRef, OutlineBuilder};
use tufa::export::nalgebra::{Vector2, Vector3};

use crate::{App, Instance};

#[derive(Default)]
pub struct BèzierBuilder {
    position: Vector2<f32>,
    points: Vec<Vector2<f32>>,
}

impl App {
    pub fn rebuild(&mut self) {
        let (mut instances, mut points) = (Vec::new(), Vec::new());

        let face = self.face.as_face_ref();
        let ui = &self.ui;

        let lines = self.ui.text.chars().filter(|&x| x == '\n').count();
        let mut position = Vector2::new(0.0, lines as f32 * face.height() as f32 * ui.size);

        for char in ui.text.chars() {
            if char == '\n' {
                position.x = 0.0;
                position.y -= face.height() as f32 * ui.size;
                continue;
            }

            let glyph = face.glyph_index(char).unwrap();
            let spacing = face.glyph_hor_advance(glyph).unwrap();

            if char == ' ' {
                position.x += spacing as f32 * ui.size;
                continue;
            }

            let mut builder = BèzierBuilder::default();
            let bounds = face.outline_glyph(glyph, &mut builder).unwrap();
            let bèzier_points = builder.into_inner();

            instances.push(Instance {
                position: position
                    + Vector2::new(bounds.x_min, bounds.y_min).map(|x| x as f32) * ui.size,
                size: Vector2::new(bounds.width(), bounds.height()).map(|x| x as f32) * ui.size,
                color: Vector3::repeat(1.0),
                glyph_start: points.len() as u32,
                glyph_length: bèzier_points.len() as u32,
            });

            let (min, max) = (
                Vector2::new(bounds.x_min, bounds.y_min).map(|x| x as f32),
                Vector2::new(bounds.x_max, bounds.y_max).map(|x| x as f32),
            );
            points.extend(
                bèzier_points
                    .iter()
                    .map(|x| (x - min).component_div(&(max - min))),
            );

            position.x += spacing as f32 * ui.size;
        }

        self.glyph_count = instances.len() as u32;
        self.instances.upload(&instances);
        self.points.upload(&points);
    }
}

impl OutlineBuilder for BèzierBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.position = Vector2::new(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let new = Vector2::new(x, y);

        self.points.push(self.position);
        self.points.push((self.position + new) / 2.0);
        self.points.push(new);

        self.position = new;
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let control = Vector2::new(x1, y1);
        let end = Vector2::new(x, y);

        self.points.push(self.position);
        self.points.push(control);
        self.points.push(end);

        self.position = end;
    }

    fn curve_to(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _x: f32, _y: f32) {
        panic!("No support for cubic Bèziers.")
    }

    fn close(&mut self) {}
}

impl BèzierBuilder {
    pub fn into_inner(self) -> Vec<Vector2<f32>> {
        self.points
    }
}
