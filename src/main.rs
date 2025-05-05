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

#[derive(ShaderType)]
struct Instance {
    position: Vector2<f32>,
    size: Vector2<f32>,
    color: Vector3<f32>,
    glyph: u32,
}

const FONT: &[u8] = include_bytes!(
    "/home/connorslade/Downloads/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-Regular.ttf"
);

fn main() -> Result<()> {
    let face = Face::parse(FONT, 0)?;
    let glyph = face.glyph_index('m').unwrap();
    let bounds = face.glyph_bounding_box(glyph).unwrap();

    let mut builder = BèzierBuilder::default();
    face.outline_glyph(glyph, &mut builder).unwrap();
    let bèzier_points = builder.into_inner();

    let gpu = Gpu::new()?;
    let uniform = gpu.create_uniform(&Uniform::default());

    let points = gpu.create_storage::<_, Immutable>(&bèzier_points);
    let instances = gpu.create_vertex(&[Instance {
        position: Vector2::new(0.0, 0.0),
        size: Vector2::new(bounds.width(), bounds.height()).map(|x| x as f32) * 0.1,
        color: Vector3::new(1.0, 0.0, 0.0),
        glyph: 0,
    }]);
    let render = gpu
        .render_pipeline(include_wgsl!("shaders/render.wgsl"))
        .instance_layout(INSTANCE_LAYOUT)
        .bind(&uniform, ShaderStages::VERTEX_FRAGMENT)
        .bind(&points, ShaderStages::VERTEX_FRAGMENT)
        .finish();

    let lines = bèzier_points
        .chunks(3)
        .flat_map(|x| bèzier(x[0], x[1], x[2]))
        .tuple_windows()
        .flat_map(|(a, b)| {
            [
                Vertex::new(Vector4::new(a.x, a.y, 1.0, 1.0), Vector2::zeros()),
                Vertex::new(Vector4::new(b.x, b.y, 1.0, 1.0), Vector2::zeros()),
            ]
        })
        .collect::<Vec<_>>();
    let line_count = lines.len() as u32;
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
            glyph_count: 1,

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
