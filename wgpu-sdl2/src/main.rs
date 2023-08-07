mod vertex;

use glam::{Affine2, Vec2};
use sdl2::{event::Event, keyboard::Keycode};
use vertex::Vertex;
use wgpu::{
	include_wgsl,
	util::{BufferInitDescriptor, DeviceExt},
	BindGroupDescriptor, BindGroupLayoutDescriptor, BufferUsages, Color, ColorTargetState,
	ColorWrites, CommandEncoderDescriptor, DeviceDescriptor, FragmentState, Instance,
	InstanceDescriptor, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor,
	PrimitiveState, RenderPassColorAttachment, RenderPipelineDescriptor, RequestAdapterOptions,
	TextureFormat, TextureViewDescriptor, VertexState,
};

fn main() {
	let sdl = sdl2::init().unwrap();
	let video = sdl.video().unwrap();
	let window = video.window("Input Lag Test", 800, 600).build().unwrap();
	let (width, height) = window.size();
	let instance = Instance::new(InstanceDescriptor::default());
	let surface = unsafe { instance.create_surface(&window) }.unwrap();
	let adapter =
		pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default())).unwrap();
	let (device, queue) =
		pollster::block_on(adapter.request_device(&DeviceDescriptor::default(), None)).unwrap();
	let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));
	let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
		entries: &[],
		label: Some("bind_group_layout"),
	});
	let bind_group = device.create_bind_group(&BindGroupDescriptor {
		layout: &bind_group_layout,
		entries: &[],
		label: Some("bind_group"),
	});
	let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
		bind_group_layouts: &[&bind_group_layout],
		label: None,
		push_constant_ranges: &[],
	});
	let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
		layout: Some(&pipeline_layout),
		vertex: VertexState {
			buffers: &[Vertex::layout()],
			module: &shader,
			entry_point: "vs_main",
		},
		fragment: Some(FragmentState {
			targets: &[Some(ColorTargetState {
				format: TextureFormat::Bgra8UnormSrgb,
				blend: None,
				write_mask: ColorWrites::ALL,
			})],
			module: &shader,
			entry_point: "fs_main",
		}),
		primitive: PrimitiveState::default(),
		depth_stencil: None,
		label: None,
		multisample: MultisampleState {
			count: 1,
			mask: !0,
			alpha_to_coverage_enabled: false,
		},
		multiview: None,
	});
	let surface_caps = surface.get_capabilities(&adapter);
	let surface_format = surface_caps
		.formats
		.iter()
		.copied()
		.find(|f| f.is_srgb())
		.unwrap_or(surface_caps.formats[0]);
	let config = wgpu::SurfaceConfiguration {
		usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
		format: surface_format,
		width,
		height,
		present_mode: wgpu::PresentMode::Fifo,
		alpha_mode: wgpu::CompositeAlphaMode::Auto,
		view_formats: Vec::default(),
	};
	surface.configure(&device, &config);

	let mut event_pump = sdl.event_pump().unwrap();
	loop {
		for event in event_pump.poll_iter() {
			if let Event::Quit { .. }
			| Event::KeyDown {
				keycode: Some(Keycode::Escape),
				..
			} = event
			{
				return;
			}
		}
		let transform = Affine2::from_translation(Vec2::new(-1.0, 1.0))
			* Affine2::from_scale(Vec2::new(2.0 / width as f32, -2.0 / height as f32));
		let mouse_position = transform.transform_point2(Vec2::new(
			event_pump.mouse_state().x() as f32,
			event_pump.mouse_state().y() as f32,
		));
		let mesh_vertices = [
			Vertex {
				position: mouse_position,
			},
			Vertex {
				position: mouse_position + Vec2::new(0.25, 0.0),
			},
			Vertex {
				position: mouse_position + Vec2::new(0.0, 0.25),
			},
		];
		let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
			label: None,
			contents: bytemuck::cast_slice(&mesh_vertices),
			usage: BufferUsages::VERTEX,
		});
		let frame = surface.get_current_texture().unwrap();
		let output = frame.texture.create_view(&TextureViewDescriptor::default());
		let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
			label: Some("command_encoder"),
		});

		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				color_attachments: &[Some(RenderPassColorAttachment {
					view: &output,
					resolve_target: None,
					ops: Operations {
						load: LoadOp::Clear(Color::BLACK),
						store: true,
					},
				})],
				depth_stencil_attachment: None,
				label: None,
			});
			render_pass.set_pipeline(&render_pipeline);
			render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
			render_pass.set_bind_group(0, &bind_group, &[]);
			render_pass.draw(0..3, 0..1);
		}
		queue.submit([encoder.finish()]);
		frame.present();
	}
}
