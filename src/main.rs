use anyhow::Result;
use encase::ShaderType;
use tufa::{
    bindings::{mutability::Immutable, UniformBuffer, VertexBuffer},
    export::{
        nalgebra::{Vector2, Vector3},
        wgpu::{
            include_wgsl, RenderPass, ShaderStages, VertexAttribute, VertexBufferLayout,
            VertexFormat, VertexStepMode,
        },
        winit::window::WindowAttributes,
    },
    gpu::Gpu,
    interactive::{GraphicsCtx, Interactive},
    pipeline::render::RenderPipeline,
};

struct App {
    render: RenderPipeline,
    uniform: UniformBuffer<Uniform>,
    instances: VertexBuffer<Instance>,
    count: u32,
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

fn main() -> Result<()> {
    let gpu = Gpu::new()?;

    let points = gpu.create_storage::<_, Immutable>(&vec![
        Vector2::new(0.0, 0.0),
        Vector2::new(1.0, 0.0),
        Vector2::new(1.0, 1.0),
    ]);
    let uniform = gpu.create_uniform(&Uniform::default());
    let instances = gpu.create_vertex(&vec![Instance {
        position: Vector2::new(0.0, 0.0),
        size: Vector2::new(100.0, 200.0),
        color: Vector3::new(1.0, 0.0, 0.0),
        glyph: 0,
    }]);
    let render = gpu
        .render_pipeline(include_wgsl!("render.wgsl"))
        .instance_layout(INSTANCE_LAYOUT)
        .bind(&uniform, ShaderStages::VERTEX_FRAGMENT)
        .bind(&points, ShaderStages::VERTEX_FRAGMENT)
        .finish();

    gpu.create_window(
        WindowAttributes::default().with_title("Text Rendering"),
        App {
            render,
            uniform,
            instances,
            count: 1,
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
            .instance_quad(render_pass, &self.instances, 0..self.count);
    }
}
