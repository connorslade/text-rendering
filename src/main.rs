use std::hash::{Hash, Hasher};

use anyhow::Result;
use encase::ShaderType;
use ordered_float::OrderedFloat;
use owned_ttf_parser::OwnedFace;
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

mod font;
mod misc;
use misc::{hash, INSTANCE_LAYOUT};

struct App {
    face: OwnedFace,
    ui: Ui,

    uniform: UniformBuffer<Uniform>,
    render: RenderPipeline,
    instances: VertexBuffer<Instance>,
    points: StorageBuffer<Vec<Vector2<f32>>, Immutable>,
    glyph_count: u32,
}

struct Ui {
    text: String,
    size: f32,
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

const FONT: &[u8] = include_bytes!("/opt/wine-staging/share/wine/fonts/times.ttf");

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
            ui: Ui::default(),

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

        (old_hash != hash(&self.ui)).then(|| self.rebuild());
    }
}

impl Hash for Ui {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.hash(state);
        OrderedFloat(self.size).hash(state);
    }
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            text: String::new(),
            size: 0.3,
        }
    }
}
