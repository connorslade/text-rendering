@group(0) @binding(0) var<uniform> ctx: Uniform;

struct Uniform {
    viewport: vec2f,
}

struct VertexInput {
    @location(0) pos: vec4f,
    @location(1) uv: vec2f
}

struct VertexOutput {
    @builtin(position) pos: vec4f
}

@vertex
fn vert(line: VertexInput) -> VertexOutput {
    let pos = (line.pos.xy / ctx.viewport) * 4.0 - vec2(1.0);
    return VertexOutput(vec4(pos, 0.0, 1.0));
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4f {
    return vec4(1.0);
}
