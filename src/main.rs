use anyhow::Result;
use encase::ShaderType;
use ttf_parser::{Face, OutlineBuilder};
use tufa::{
    bindings::{mutability::Immutable, IndexBuffer, UniformBuffer, VertexBuffer},
    export::{
        egui::Context,
        nalgebra::{Vector2, Vector3, Vector4},
        wgpu::{
            include_wgsl, PrimitiveTopology, RenderPass, ShaderStages, VertexAttribute,
            VertexBufferLayout, VertexFormat, VertexStepMode,
        },
        winit::window::WindowAttributes,
    },
    gpu::Gpu,
    interactive::{GraphicsCtx, Interactive},
    pipeline::render::{RenderPipeline, Vertex},
};

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

pub const INSTANCE_LAYOUT: VertexBufferLayout = VertexBufferLayout {
    array_stride: 4 * 8,
    step_mode: VertexStepMode::Instance,
    attributes: &[
        VertexAttribute {
            format: VertexFormat::Float32x2,
            offset: 0,
            shader_location: 2,
        },
        VertexAttribute {
            format: VertexFormat::Float32x2,
            offset: 4 * 2,
            shader_location: 3,
        },
        VertexAttribute {
            format: VertexFormat::Float32x3,
            offset: 4 * 4,
            shader_location: 4,
        },
        VertexAttribute {
            format: VertexFormat::Uint32,
            offset: 4 * 7,
            shader_location: 5,
        },
    ],
};

#[derive(Default)]
struct BèzierBuilder {
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
        let control = Vector2::new(x, y);
        let end = Vector2::new(x1, y1);

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

fn main() -> Result<()> {
    let face = Face::parse(
        include_bytes!(
            "/home/connorslade/Downloads/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-Regular.ttf"
        ),
        0,
    )?;
    let glyph = face.glyph_index('A').unwrap();
    let bounds = face.glyph_bounding_box(glyph).unwrap();

    let mut builder = BèzierBuilder::default();
    face.outline_glyph(glyph, &mut builder).unwrap();

    let gpu = Gpu::new()?;

    let uniform = gpu.create_uniform(&Uniform::default());

    let points = gpu.create_storage::<_, Immutable>(&builder.into_inner());
    let instances = gpu.create_vertex(&vec![Instance {
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

    let lines = gpu.create_vertex(&vec![
        Vertex::new(Vector4::new(0.0, 0.0, 0.0, 1.0), Vector2::zeros()),
        Vertex::new(Vector4::new(200.0, 100.0, 0.0, 1.0), Vector2::zeros()),
    ]);
    let line_index = gpu.create_index(&vec![0, 1]);
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
            line_count: 2,
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

    fn ui(&mut self, gcx: GraphicsCtx, ctx: &Context) {
        let window = gcx.window;
        let height = window.inner_size().height as f32 / window.scale_factor() as f32;
        let pointer = ctx.input(|i| i.pointer.latest_pos().unwrap_or_default());

        self.lines.upload(&vec![
            Vertex::new(Vector4::new(0.0, 0.0, 0.0, 1.0), Vector2::zeros()),
            Vertex::new(
                Vector4::new(pointer.x, height - pointer.y, 0.0, 1.0),
                Vector2::zeros(),
            ),
        ]);
    }
}
