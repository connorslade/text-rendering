use std::hash::{Hash, Hasher};

use anyhow::Result;
use encase::ShaderType;
use ordered_float::OrderedFloat;
use owned_ttf_parser::OwnedFace;
use tufa::{
    bindings::buffer::{mutability::Immutable, StorageBuffer, UniformBuffer, VertexBuffer},
    export::{
        egui::{self, Context, Slider, TextEdit, Window},
        nalgebra::{Vector2, Vector3},
        wgpu::{include_wgsl, RenderPass, ShaderStages},
        winit::window::WindowAttributes,
    },
    gpu::Gpu,
    interactive::{ui::vec2_dragger, GraphicsCtx, Interactive},
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
    color: [f32; 3],

    pan: Vector2<f32>,
    masa: u32,
    show_quads: bool,
}

#[derive(ShaderType, Default)]
struct Uniform {
    viewport: Vector2<f32>,
    pan: Vector2<f32>,
    masa: u32,
    flags: u32,
}

#[derive(Debug, ShaderType)]
struct Instance {
    position: Vector2<f32>,
    size: Vector2<f32>,
    color: Vector3<f32>,

    glyph_start: u32,
    glyph_length: u32,
}

const FONT: &[u8] = include_bytes!("/usr/share/fonts/google-noto-vf/NotoSerif[wght].ttf");

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
        self.uniform.upload(&Uniform {
            viewport,
            pan: self.ui.pan,
            masa: self.ui.masa,
            flags: self.ui.show_quads as u32,
        });

        self.render
            .instance_quad(render_pass, &self.instances, 0..self.glyph_count);
    }

    fn ui(&mut self, gcx: GraphicsCtx, ctx: &Context) {
        let old_hash = hash(&self.ui);

        let dragging_viewport = ctx.dragged_id().is_none() && !ctx.is_pointer_over_area();
        let scale_factor = gcx.window.scale_factor() as f32;
        ctx.input(|input| {
            if input.pointer.any_down() && dragging_viewport {
                let delta = input.pointer.delta() * scale_factor;
                self.ui.pan += Vector2::new(delta.x, -delta.y);
            }
        });

        Window::new("Text Rendering")
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.add(
                    TextEdit::multiline(&mut self.ui.text)
                        .desired_width(f32::INFINITY)
                        .desired_rows(3),
                );

                egui::Grid::new("settings_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Viewport Pan");
                        vec2_dragger(ui, &mut self.ui.pan, |x| x);
                        ui.end_row();

                        ui.label("Multi-sampling");
                        ui.add(Slider::new(&mut self.ui.masa, 1..=4));
                        ui.end_row();

                        ui.label("Font Size");
                        ui.add(Slider::new(&mut self.ui.size, 0.0..=96.0));
                        ui.end_row();

                        ui.label("Text Color");
                        ui.horizontal(|ui| {
                            ui.color_edit_button_rgb(&mut self.ui.color);
                            ui.checkbox(&mut self.ui.show_quads, "");
                        });
                        ui.end_row();
                    });
            });

        (old_hash != hash(&self.ui)).then(|| self.rebuild(scale_factor));
    }
}

impl Hash for Ui {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.hash(state);
        OrderedFloat(self.size).hash(state);
        self.color.map(OrderedFloat).hash(state);
        self.pan.map(OrderedFloat).hash(state);
    }
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            text: String::new(),
            size: 24.0,
            color: [1.0; 3],

            pan: Vector2::zeros(),
            masa: 2,
            show_quads: false,
        }
    }
}
