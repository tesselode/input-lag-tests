use bytemuck::{Pod, Zeroable};
use glam::Vec2;
use wgpu::{VertexAttribute, VertexBufferLayout};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
	pub position: Vec2,
}

impl Vertex {
	const ATTRIBUTES: [VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

	pub fn layout() -> VertexBufferLayout<'static> {
		use std::mem;

		VertexBufferLayout {
			array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &Self::ATTRIBUTES,
		}
	}
}
