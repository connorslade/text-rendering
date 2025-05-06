use owned_ttf_parser::OutlineBuilder;
use tufa::export::nalgebra::Vector2;

#[derive(Default)]
pub struct BèzierBuilder {
    position: Vector2<f32>,
    points: Vec<Vector2<f32>>,
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
