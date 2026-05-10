use crate::camera::Camera;
use crate::gpu::Gpu;
use crate::mesh::{Mesh, Surface, Vertex};
use crate::render::Render;
use crate::scene::Scene;
use crate::texture;
use rand::RngExt;
use std::sync::Arc;
use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode, window::Window};

pub struct State {
    gpu: Gpu,
    render: Render,
    depth_texture: texture::Texture,
    scene: Scene,
    cube1: usize,
    cube2: usize,
    cube3: usize,
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
        let texture_bytes_gman = include_bytes!("../assets/textures/gman.png");
        let texture_id_gman = scene.add_texture(
            texture::Texture::from_bytes(
                &gpu.device,
                &gpu.queue,
                texture_bytes_gman,
                "Gman Texture",
            )
            .unwrap(),
        );
        let cube1 = scene.add_object(Mesh::new_manually(
            verts.clone(),
            inds.clone(),
            Surface::Textured(texture_id_gman as u32),
            true,
        ));
        let texture_bytes_doctor = include_bytes!("../assets/textures/doctor.png");
        let texture_id_doctor = scene.add_texture(
            texture::Texture::from_bytes(
                &gpu.device,
                &gpu.queue,
                texture_bytes_doctor,
                "Doctor Texture",
            )
            .unwrap(),
        );
        let cube2 = scene.add_object(Mesh::new_manually(
            verts.clone(),
            inds.clone(),
            Surface::Textured(texture_id_doctor as u32),
            true,
        ));
        let cube3 = scene.add_object(Mesh::new_manually(
            verts.clone(),
            inds.clone(),
            Surface::Colored,
            true,
        ));
        if let Some(mesh) = scene.get_object(cube2) {
            mesh.set_position(glam::vec3(5.0, 1.0, 2.0));
        }
        if let Some(mesh) = scene.get_object(cube3) {
            mesh.set_position(glam::vec3(-5.0, 0.0, -2.0));
        }
        let mut rng = rand::rng();

        for _ in 0..1000 {
            let x: f32 = rng.random_range(-50.0..50.0);
            let y: f32 = rng.random_range(-50.0..50.0);
            let z: f32 = rng.random_range(-50.0..50.0);

            let cube_id = scene.add_object(Mesh::new_manually(
                verts.clone(),
                inds.clone(),
                Surface::Textured(if rng.random_bool(0.5) {
                    texture_id_doctor as u32
                } else {
                    texture_id_gman as u32
                }),
                true,
            ));

            if let Some(mesh) = scene.get_object(cube_id) {
                mesh.set_scale(rng.random_range(0.5..3.0));
            }
            if let Some(mesh) = scene.get_object(cube_id) {
                mesh.set_rotation(
                    glam::Quat::from_axis_angle(
                        glam::vec3(
                            rng.random_range(-1.0..1.0),
                            rng.random_range(-1.0..1.0),
                            rng.random_range(-1.0..1.0),
                        ),
                        rng.random_range(0.0..6.28),
                    )
                    .normalize(),
                );
            }
            if let Some(mesh) = scene.get_object(cube_id) {
                mesh.set_position(glam::vec3(x, y, z));
            }
        }
        scene.remove_object(cube1);
        scene.build_models_buffers(&gpu);

        let depth_texture =
            texture::Texture::create_depth_texture(&gpu.device, &gpu.config, "depth_texture");

        let render = pollster::block_on(Render::new(
            &gpu,
            &scene.camera_bind_group_layout,
            &scene.transform_bind_group_layout,
            &scene.textures_bind_group_layout,
        ))
        .unwrap();

        Ok(Self {
            gpu,
            render,
            depth_texture,
            scene,
            cube1,
            cube2,
            cube3,
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
            self.depth_texture = texture::Texture::create_depth_texture(
                &self.gpu.device,
                &self.gpu.config,
                "depth_texture",
            );
            self.scene.camera.aspect = width as f32 / height as f32;
            self.scene.update_camera(&self.gpu);
            log::info!("Window resized");
        }
    }

    fn update(&mut self) {
        if let Some(mesh) = self.scene.get_object(self.cube2) {
            mesh.rotate(glam::Quat::from_axis_angle(
                glam::Vec3::new(1.0, 0.5, 0.5),
                0.01,
            ));
        }
        if let Some(mesh) = self.scene.get_object(self.cube3) {
            mesh.vertices[0].color[1] += 0.001;
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            render_pass.set_bind_group(0, &self.scene.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.scene.transform_bind_group, &[]);
            if let (Some(v_buf_tex), Some(i_buf_tex)) = (
                &self.scene.vertex_textured_buffer,
                &self.scene.index_textured_buffer,
            ) {
                render_pass.set_pipeline(&self.render.render_pipeline_textured);
                render_pass.set_bind_group(2, &self.scene.textures_bind_group, &[]);
                render_pass.set_vertex_buffer(0, v_buf_tex.slice(..));
                render_pass.set_index_buffer(i_buf_tex.slice(..), wgpu::IndexFormat::Uint32);
                for model in &self.scene.models_textured_drawing {
                    render_pass.draw_indexed(
                        model.index_range.clone(),
                        model.base_vertex as i32,
                        model.index_transform..(model.index_transform + 1),
                    );
                }
            }
            if let (Some(v_buf_col), Some(i_buf_col)) = (
                &self.scene.vertex_colored_buffer,
                &self.scene.index_colored_buffer,
            ) {
                render_pass.set_pipeline(&self.render.render_pipeline_colored);
                render_pass.set_vertex_buffer(0, v_buf_col.slice(..));
                render_pass.set_index_buffer(i_buf_col.slice(..), wgpu::IndexFormat::Uint32);
                for model in &self.scene.models_colored_drawing {
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
