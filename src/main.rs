use anyhow::Result;
use encase::ShaderType;
use itertools::Itertools;
use ttf_parser::Face;
use tufa::{
    bindings::{mutability::Immutable, IndexBuffer, UniformBuffer, VertexBuffer},
    export::{
        nalgebra::{Vector2, Vector3, Vector4},
        wgpu::{include_wgsl, PrimitiveTopology, RenderPass, ShaderStages},
        winit::window::WindowAttributes,
    },
    gpu::Gpu,
    interactive::{GraphicsCtx, Interactive},
    pipeline::render::{RenderPipeline, Vertex},
};

mod bezier;
mod consts;
use bezier::{bèzier, BèzierBuilder};
use consts::INSTANCE_LAYOUT;

struct App {
    uniform: UniformBuffer<Uniform>,

    render: RenderPipeline,
    instances: VertexBuffer<Instance>,
    glyph_count: u32,

    line: RenderPipeline,
    lines: VertexBuffer<Vertex>,
    line_index: IndexBuffer,
    line_count: u32,
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

const SCALE: f32 = 0.25;
const FONT: &[u8] = include_bytes!(
    "/home/connorslade/Downloads/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-Regular.ttf"
);

fn main() -> Result<()> {
    let mut instances = Vec::new();
    let mut points = Vec::new();
    let mut lines = Vec::new();

    let face = Face::parse(FONT, 0)?;
    let mut position = Vector2::new(0.0, 20.0);
    for char in "hèllö".chars() {
        let glyph = face.glyph_index(char).unwrap();
        let spacing = face.glyph_hor_advance(glyph).unwrap();

        let mut builder = BèzierBuilder::default();
        let bounds = face.outline_glyph(glyph, &mut builder).unwrap();
        let bèzier_points = builder.into_inner();

        instances.push(Instance {
            position: position + Vector2::new(bounds.x_min, bounds.y_min).map(|x| x as f32) * SCALE,
            size: Vector2::new(bounds.width(), bounds.height()).map(|x| x as f32) * SCALE,
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
        lines.extend(bèzier_points.chunks(3).flat_map(|x| {
            bèzier(x[0], x[1], x[2])
                .into_iter()
                .tuple_windows()
                .flat_map(|(a, b)| {
                    [a, b]
                        .map(|x| x * SCALE + position)
                        .map(|x| Vertex::new(Vector4::new(x.x, x.y, 1.0, 1.0), Vector2::zeros()))
                })
        }));

        position += Vector2::x() * spacing as f32 * SCALE;
    }

    let line_count = lines.len() as u32;

    let gpu = Gpu::new()?;
    let uniform = gpu.create_uniform(&Uniform::default());

    let glyph_count = instances.len() as u32;
    let points = gpu.create_storage::<_, Immutable>(&points);
    let instances = gpu.create_vertex(&instances);
    let render = gpu
        .render_pipeline(include_wgsl!("shaders/render.wgsl"))
        .instance_layout(INSTANCE_LAYOUT)
        .bind(&uniform, ShaderStages::VERTEX_FRAGMENT)
        .bind(&points, ShaderStages::VERTEX_FRAGMENT)
        .finish();

    let lines = gpu.create_vertex(&lines);
    let line_index = gpu.create_index(&(0..line_count).collect::<Vec<_>>());
    let line = gpu
        .render_pipeline(include_wgsl!("shaders/line.wgsl"))
        .bind(&uniform, ShaderStages::VERTEX_FRAGMENT)
        .topology(PrimitiveTopology::LineList)
        .finish();

    gpu.create_window(
        WindowAttributes::default().with_title("Text Rendering"),
        App {
            uniform,

            render,
            instances,
            glyph_count,

            line,
            lines,
            line_index,
            line_count,
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
        self.line.draw(
            render_pass,
            &self.line_index,
            &self.lines,
            0..self.line_count,
        );
    }
}
