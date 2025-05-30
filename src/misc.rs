use std::hash::{DefaultHasher, Hash, Hasher};

use tufa::export::wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

pub const INSTANCE_LAYOUT: VertexBufferLayout = VertexBufferLayout {
    array_stride: 4 * 12,
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
        VertexAttribute {
            format: VertexFormat::Uint32,
            offset: 4 * 8,
            shader_location: 6,
        },
    ],
};

pub fn hash<T: Hash>(t: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}
