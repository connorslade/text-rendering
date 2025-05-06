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
    let brightness = f32(sample(in.glyph_start, in.glyph_length, in.uv));
    return vec4(in.color * brightness, 1.0);
}

fn sample(start: u32, length: u32, uv: vec2f) -> bool {
    var hits = 0;
    let end = start + length;
    for (var i = start; i < end; i += 3) {
        let a = points[i];
        let b = points[i + 1];
        let c = points[i + 2];

        ray_bézier_intersection(&hits, a, b, c, uv);
    }

    return hits % 2 == 1;
}

fn ray_bézier_intersection(hits: ptr<function, i32>, p1: vec2f, p2: vec2f, p3: vec2f, t: vec2f) {
    // (a - 2 b + c) t^2 + (2b - 2a) t + a
    // t = -b±√(b²-4ac)/2a

    let a = p1.y - 2.0 * p2.y + p3.y;
    let b = 2.0 * (p2.y - p1.y);
    let c = p1.y - t.y;

    if abs(a) < ε {
        if abs(b) < ε {
            *hits += i32(p1.x >= t.x && p1.y == t.y);
        } else {
            let t0 = -c / b;
            *hits += i32(t0 >= 0.0 && t0 <= 1.0 && mix(p1.x, p3.x, t0) >= t.x);
        }
        return;
    }

    let Δ = b * b - 4.0 * a * c;
    if Δ < 0.0 { return; }
    let δ = sqrt(Δ);

    let t1 = (-b + δ) / (2.0 * a);
    let t2 = (-b - δ) / (2.0 * a);

    let x1 = quadratic_bézier(p1, p2, p3, t1).x;
    let x2 = quadratic_bézier(p1, p2, p3, t2).x;

    *hits += i32(t1 > 0.0 && t1 < 1.0 && x1 >= t.x);
    *hits += i32(t2 >= 0.0 && t2 <= 1.0 && x2 >= t.x);
}

fn quadratic_bézier(p1: vec2f, p2: vec2f, p3: vec2f, t: f32) -> vec2f {
    return (p1 - 2.0 * p2 + p3) * t * t + (p2 - p1) * 2.0 * t + p1;
}
