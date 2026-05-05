use crate::gpu::Gpu;
use crate::mesh::Vertex;

pub struct Render {
    pub render_pipeline_colored: wgpu::RenderPipeline,
}

impl Render {
    pub async fn new(
        gpu: &Gpu,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        transform_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> anyhow::Result<Self> {
        let shader_colored = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader Colored"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/colored.wgsl").into()),
            });

        let render_pipeline_colored_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Colored Layout"),
                    bind_group_layouts: &[camera_bind_group_layout, transform_bind_group_layout],
                    immediate_size: 0,
                });

        let render_pipeline_colored =
            gpu.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline Colored"),
                    layout: Some(&render_pipeline_colored_layout),
                    vertex: wgpu::VertexState {
                        module: &shader_colored,
                        entry_point: Some("vs_main"),
                        buffers: &[Vertex::desc()],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_colored,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: gpu.config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview_mask: None,
                    cache: None,
                });
        log::info!("Render initialized");
        Ok(Self {
            render_pipeline_colored,
        })
    }
}
