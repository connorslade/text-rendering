@group(0) @binding(0) var<uniform> ctx: Uniform;
@group(0) @binding(1) var<storage, read> points: array<vec2f>;

const ε: f32 = 0.00001;

struct Uniform {
    viewport: vec2f,
}

struct Instance {
    @location(2) position: vec2f,
    @location(3) size: vec2f,
    @location(4) color: vec3f,

    @location(5) glyph_start: u32,
    @location(6) glyph_length: u32
}

struct VertexInput {
    @location(0) pos: vec4f,
    @location(1) uv: vec2f
}

struct VertexOutput {
    @builtin(position) pos: vec4f,
    @location(1) uv: vec2f,

    @location(2) color: vec3f,
    @location(3) glyph_start: u32,
    @location(4) glyph_length: u32
}

@vertex
fn vert(
    model: VertexInput,
    instance: Instance
) -> VertexOutput {
    let pos = (model.pos.xy * instance.size + instance.position * 2.0 + instance.size) / ctx.viewport * 2.0 - vec2(1.0);
    return VertexOutput(vec4(pos, 0.0, 1.0), model.uv, instance.color, instance.glyph_start, instance.glyph_length);
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4f {
    var hits = 0;
    let end = in.glyph_start + in.glyph_length;
    for (var i = in.glyph_start; i < end; i += 3) {
        let a = points[i];
        let b = points[i + 1];
        let c = points[i + 2];

        hits += i32(ray_line_intersection(a, b, in.uv));
        hits += i32(ray_line_intersection(b, c, in.uv));
    }

    let in_glyph = hits % 2 == 1;
    return vec4(in.color * f32(in_glyph), 1.0);
}

fn ray_line_intersection(a: vec2f, b: vec2f, t: vec2f) -> bool {
    if abs(a.x - b.x) < ε {
        return b.x >= t.x && ((t.y >= a.y && t.y <= b.y) || (t.y >= b.y && t.y <= a.y));
    }

    let slope = (a.y - b.y) / (a.x - b.x);
    let offset = a.y - slope * a.x;
    let x = (t.y - offset) / slope;

    return x >= t.x && ((x >= a.x && x <= b.x) || (x <= a.x && x >= b.x));
}
