@group(0) @binding(0) var<uniform> ctx: Uniform;

struct Uniform {
    viewport: vec2f,
}

struct Instance {
    @location(2) position: vec2f,
    @location(3) size: vec2f,
    @location(4) color: vec3f,
    @location(5) glyph: u32
}

struct VertexInput {
    @location(0) pos: vec4f,
    @location(1) uv: vec2f
}

struct VertexOutput {
    @builtin(position) pos: vec4f,
    @location(1) uv: vec2f,
    @location(2) color: vec3f
}

@vertex
fn vert(
    model: VertexInput,
    instance: Instance
) -> VertexOutput {
    let pos = (model.pos.xy * instance.size + instance.position + instance.size) / ctx.viewport * 2.0 - vec2(1.0);
    return VertexOutput(vec4(pos, 0.0, 1.0), model.uv, instance.color);
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4f {
    return vec4(in.color, 1.0);
}
