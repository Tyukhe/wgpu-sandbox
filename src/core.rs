use crate::camera::Camera;
use crate::gpu::Gpu;
use crate::mesh::{Mesh, Surface, Vertex};
use crate::render::Render;
use crate::scene::Scene;
use std::sync::Arc;
use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode, window::Window};

pub struct State {
    gpu: Gpu,
    render: Render,
    scene: Scene,
    cube1: usize,
    cube2: usize,
    pub window: Arc<Window>,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let gpu = pollster::block_on(Gpu::new(window.clone(), wgpu::Backends::PRIMARY)).unwrap();
        let camera = Camera {
            position: glam::vec3(0.0, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            aspect: gpu.config.width as f32 / gpu.config.height as f32,
            fovy: 1.5,
            znear: 0.1,
        };
        let mut scene = Scene::new(&gpu, camera);
        let mut verts = vec![
            Vertex {
                position: [1.0, 1.0, 1.0],
                color: [1.0, 0.0, 0.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [-1.0, 1.0, 1.0],
                color: [1.0, 0.0, 0.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [1.0, -1.0, 1.0],
                color: [1.0, 0.0, 0.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [-1.0, -1.0, 1.0],
                color: [1.0, 0.0, 0.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0, -1.0],
                color: [1.0, 0.0, 0.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [-1.0, 1.0, -1.0],
                color: [1.0, 0.0, 0.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [1.0, -1.0, -1.0],
                color: [1.0, 0.0, 0.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [-1.0, -1.0, -1.0],
                color: [1.0, 0.0, 0.0],
                tex_coords: [1.0, 1.0],
            },
        ];
        let inds: Vec<u32> = vec![
            0, 1, 2, 1, 3, 2, 5, 4, 7, 4, 6, 7, 1, 5, 3, 5, 7, 3, 4, 0, 6, 0, 2, 6, 4, 5, 0, 5, 1,
            0, 2, 3, 6, 3, 7, 6,
        ];
        let cube1 = scene.add_object(Mesh::new_manually(
            verts.clone(),
            inds.clone(),
            Surface::Textured(0),
            true,
        ));
        verts[0].color = [0.0, 1.0, 1.0];
        verts[1].color = [0.0, 0.0, 1.0];
        verts[2].color = [0.0, 0.0, 1.0];
        verts[3].color = [0.0, 0.0, 1.0];
        verts[4].color = [0.0, 0.0, 1.0];
        verts[5].color = [0.0, 0.0, 1.0];
        let cube2 = scene.add_object(Mesh::new_manually(verts, inds, Surface::Textured(0), true));
        if let Some(mesh) = scene.get_object(cube2) {
            mesh.set_position(glam::vec3(5.0, 1.0, 2.0));
        }
        scene.build_models_buffers(&gpu);

        let render = pollster::block_on(Render::new(
            &gpu,
            &scene.camera_bind_group_layout,
            &scene.transform_bind_group_layout,
            &scene.diffuse_bind_group_layout,
        ))
        .unwrap();

        Ok(Self {
            gpu,
            render,
            scene,
            cube1,
            cube2,
            window,
        })
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
        let _ = self.scene.camera_controller.handle_key(code, is_pressed);
    }

    pub fn handle_mouse_delta(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.scene.camera_controller.handle_mouse(
            glam::vec2(mouse_dx as f32, mouse_dy as f32),
            &mut self.scene.camera,
        );
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.gpu.configure_surface(width, height);
            self.scene.camera.aspect = width as f32 / height as f32;
            self.scene.update_camera(&self.gpu);
            log::info!("Window resized");
        }
    }

    fn update(&mut self) {
        if let Some(mesh) = self.scene.get_object(self.cube2) {
            mesh.turn(glam::Quat::from_axis_angle(
                glam::Vec3::new(1.0, 0.5, 0.5),
                0.01,
            ));
        }
        self.scene.build_transform_buffers(&self.gpu);
        self.scene.update_camera(&self.gpu);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if !self.gpu.is_surface_configured {
            return Ok(());
        }

        self.update();

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
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            if let (Some(v_buf), Some(i_buf)) =
                (&self.scene.vertex_buffer, &self.scene.index_buffer)
            {
                render_pass.set_pipeline(&self.render.render_pipeline_textured);
                render_pass.set_bind_group(0, &self.scene.camera_bind_group, &[]);
                render_pass.set_bind_group(1, &self.scene.transform_bind_group, &[]);
                render_pass.set_bind_group(2, &self.scene.diffuse_bind_group, &[]);
                render_pass.set_vertex_buffer(0, v_buf.slice(..));
                render_pass.set_index_buffer(i_buf.slice(..), wgpu::IndexFormat::Uint32);
                for model in &self.scene.models {
                    render_pass.draw_indexed(
                        model.index_range.clone(),
                        model.base_vertex as i32,
                        model.index_transform..(model.index_transform + 1),
                    );
                }
            }
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
