use std::hash::{DefaultHasher, Hash, Hasher};

use anyhow::Result;
use encase::ShaderType;
use ordered_float::OrderedFloat;
use owned_ttf_parser::{AsFaceRef, OwnedFace};
use tufa::{
    bindings::buffer::{mutability::Immutable, StorageBuffer, UniformBuffer, VertexBuffer},
    export::{
        egui::{Context, Slider, TextEdit, Window},
        nalgebra::{Vector2, Vector3},
        wgpu::{include_wgsl, RenderPass, ShaderStages},
        winit::window::WindowAttributes,
    },
    gpu::Gpu,
    interactive::{GraphicsCtx, Interactive},
    pipeline::render::RenderPipeline,
};

mod consts;
mod font;
use consts::INSTANCE_LAYOUT;
use font::BèzierBuilder;

struct Ui {
    text: String,
    size: f32,
}

struct App {
    face: OwnedFace,
    ui: Ui,

    uniform: UniformBuffer<Uniform>,
    render: RenderPipeline,
    instances: VertexBuffer<Instance>,
    points: StorageBuffer<Vec<Vector2<f32>>, Immutable>,
    glyph_count: u32,
}

#[derive(ShaderType, Default)]
struct Uniform {
    viewport: Vector2<f32>,
}

#[derive(Debug, ShaderType)]
struct Instance {
    position: Vector2<f32>,
    size: Vector2<f32>,
    color: Vector3<f32>,

    glyph_start: u32,
    glyph_length: u32,
}

const FONT: &[u8] = include_bytes!(
    "/home/connorslade/Downloads/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-Regular.ttf"
);

fn main() -> Result<()> {
    let gpu = Gpu::new()?;
    let face = OwnedFace::from_vec(FONT.to_vec(), 0)?;

    let uniform = gpu.create_uniform(&Uniform::default());
    let points = gpu.create_storage_empty(1024);
    let instances = gpu.create_vertex_empty(1024);
    let render = gpu
        .render_pipeline(include_wgsl!("render.wgsl"))
        .instance_layout(INSTANCE_LAYOUT)
        .bind(&uniform, ShaderStages::VERTEX_FRAGMENT)
        .bind(&points, ShaderStages::VERTEX_FRAGMENT)
        .finish();

    gpu.create_window(
        WindowAttributes::default().with_title("Text Rendering"),
        App {
            face,
            ui: Ui {
                text: String::new(),
                size: 0.3,
            },

            uniform,
            render,
            instances,
            points,
            glyph_count: 0,
        },
    )
    .run()?;

    Ok(())
}

impl Interactive for App {
    fn render(&mut self, gcx: GraphicsCtx, render_pass: &mut RenderPass) {
        let inner_size = gcx.window.inner_size();
        let viewport = Vector2::new(inner_size.width, inner_size.height).map(|x| x as f32);
        self.uniform.upload(&Uniform { viewport });

        self.render
            .instance_quad(render_pass, &self.instances, 0..self.glyph_count);
    }

    fn ui(&mut self, _gcx: GraphicsCtx, ctx: &Context) {
        let old_hash = hash(&self.ui);
        Window::new("Text Rendering")
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.add(TextEdit::multiline(&mut self.ui.text).desired_width(200.0));
                ui.horizontal(|ui| {
                    ui.add(Slider::new(&mut self.ui.size, 0.0..=1.0));
                    ui.label("Font Size");
                });
            });

        if old_hash != hash(&self.ui) {
            self.rebuild();
        }
    }
}

impl App {
    fn rebuild(&mut self) {
        let (mut instances, mut points) = (Vec::new(), Vec::new());

        let face = self.face.as_face_ref();
        let ui = &self.ui;

        let lines = self.ui.text.lines().count() - 1;
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

impl Hash for Ui {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.hash(state);
        OrderedFloat(self.size).hash(state);
    }
}

fn hash<T: Hash>(t: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}
