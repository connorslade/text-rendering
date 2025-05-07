@group(0) @binding(0) var<uniform> ctx: Uniform;
@group(0) @binding(1) var<storage, read> points: array<vec2f>;

const ε: f32 = 1e-6;
const ι: f32 = 3.40282347e38;

struct Uniform {
    viewport: vec2f,
    pan: vec2f
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
    let pos = (model.pos.xy * instance.size + instance.position * 2.0 + instance.size + ctx.pan) / ctx.viewport * 2.0 - vec2(1.0);
    return VertexOutput(vec4(pos, 0.0, 1.0), model.uv, instance.color, instance.glyph_start, instance.glyph_length);
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4f {
    let brightness = f32(sample(in.glyph_start, in.glyph_length, in.uv));
    return vec4(in.color * brightness, 1.0);
}

fn sample(start: u32, length: u32, uv: vec2f) -> bool {
    var hits = 0u;
    let end = start + length;

    var last = vec2(ι);
    for (var i = start; i < end; i += 3) {
        let a = points[i];
        let b = points[i + 1];
        let c = points[i + 2];

        let roots = bézier_roots(a, b, c, uv);
        hits += roots.count;

        if roots.last.x != ι {
            hits -= u32(last.x != ι && roots.count > 0 && length(roots.last - last) < ε);
            last = roots.last;
        }
    }

    return hits % 2 == 1;
}

struct BézierRoots {
    last: vec2f,
    count: u32
}

fn bézier_roots(p1: vec2f, p2: vec2f, p3: vec2f, t: vec2f) -> BézierRoots {
    // (a - 2 b + c) t^2 + (2b - 2a) t + a
    // t = -b±√(b²-4ac)/2a

    if (p1.x < t.x && p2.x < t.x && p3.x < t.x) || (p1.y < t.y && p2.y < t.y && p3.y < t.y) {
        return BézierRoots(vec2(ι), 0);
    }

    let a = p1.y - 2.0 * p2.y + p3.y;
    let b = 2.0 * (p2.y - p1.y);
    let c = p1.y - t.y;

    if abs(a) < ε {
        if abs(b) < ε {
            if p1.x >= t.x && abs(p1.y - t.y) < ε {
                return BézierRoots(p3, 1);
            }
        } else {
            let t0 = -c / b;
            if t0 >= -ε && t0 <= 1.0 + ε && mix(p1.x, p3.x, t0) >= t.x {
                let c = quadratic_bézier(p1, p2, p3, saturate(t0));
                return BézierRoots(c, 1);
            }
        }
    } else {
        let Δ = b * b - 4.0 * a * c;
        if Δ < 0.0 { return BézierRoots(vec2(ι), 0); }
        let δ = sqrt(Δ);

        let t1 = (-b + δ) / (2.0 * a);
        let t2 = (-b - δ) / (2.0 * a);

        let c1 = quadratic_bézier(p1, p2, p3, saturate(t1));
        let c2 = quadratic_bézier(p1, p2, p3, saturate(t2));

        var last = vec2(ι);
        var count = 0u;

        if t1 >= -ε && t1 < 1.0 + ε && c1.x >= t.x {
            last = c1;
            count++;
        }

        if t2 >= -ε && t2 < 1.0 + ε && c2.x >= t.x {
            last = c2;
            count++;
        }

        return BézierRoots(last, count);
    }

    return BézierRoots(vec2(ι), 0);
}

fn quadratic_bézier(p1: vec2f, p2: vec2f, p3: vec2f, t: f32) -> vec2f {
    return (p1 - 2.0 * p2 + p3) * t * t + (p2 - p1) * 2.0 * t + p1;
}
