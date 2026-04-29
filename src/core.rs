use crate::gpu::Gpu;
use crate::render::Render;
use std::sync::Arc;
use winit::window::Window;

pub struct State {
    gpu: Gpu,
    _render: Render,
    pub window: Arc<Window>,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let gpu = pollster::block_on(Gpu::new(window.clone(), wgpu::Backends::PRIMARY)).unwrap();
        let _render = pollster::block_on(Render::new()).unwrap();
        Ok(Self {
            gpu,
            _render,
            window,
        })
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.gpu.configure_surface(width, height);
            log::info!("Window resized");
        }
    }
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if !self.gpu.is_surface_configured {
            return Ok(());
        }

        let output = self.gpu.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            #[warn(unused)]
            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
