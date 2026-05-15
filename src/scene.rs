use crate::camera::{Camera, CameraController, CameraUniform};
use crate::gpu::Gpu;
use crate::mesh::{Mesh, Surface, Vertex};
use crate::texture;
use std::collections::HashMap;
use std::path::Path;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
struct InstanceData {
    transform: glam::Mat4,
    texture: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}

impl InstanceData {
    pub fn new() -> Self {
        Self {
            transform: glam::Mat4::IDENTITY,
            ..Default::default()
        }
    }

    pub fn build_from_mesh(mesh: &Mesh, actual_texture: u32) -> Self {
        Self {
            transform: mesh.build_model_matrix(),
            texture: actual_texture,
            ..Default::default()
        }
    }
}

pub struct ModelInfo {
    mesh_index: usize,
    texture_index: u32,
    pub index_range: std::ops::Range<u32>,
    pub base_vertex: usize,
    pub index_transform: u32,
}

pub struct Scene {
    meshes: Vec<Option<Mesh>>,
    textures: Vec<Option<texture::Texture>>,
    pub models_colored_drawing: Vec<ModelInfo>,
    pub models_textured_drawing: Vec<ModelInfo>,
    default_texture: texture::Texture,
    pub camera: Camera,
    pub camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group: wgpu::BindGroup,
    transform_buffer: wgpu::Buffer,
    pub transform_bind_group_layout: wgpu::BindGroupLayout,
    pub transform_bind_group: wgpu::BindGroup,
    pub vertex_colored_buffer: Option<wgpu::Buffer>,
    pub index_colored_buffer: Option<wgpu::Buffer>,
    pub vertex_textured_buffer: Option<wgpu::Buffer>,
    pub index_textured_buffer: Option<wgpu::Buffer>,
    pub textures_bind_group_layout: wgpu::BindGroupLayout,
    pub textures_bind_group: wgpu::BindGroup,
}

impl Scene {
    pub fn new(gpu: &Gpu, camera: Camera) -> Self {
        let camera_controller = CameraController::new(0.08, 0.01);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);
        let camera_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("camera_bind_group_layout"),
                });

        let camera_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let pulg_transform = InstanceData::new();

        let transform_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Transform Buffer"),
                contents: bytemuck::cast_slice(&[pulg_transform]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

        let transform_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("Transform Bind Group Layout"),
                });

        let transform_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &transform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            }],
            label: Some("Transform Bind Group"),
        });

        let models_colored_drawing = Vec::<ModelInfo>::new();
        let models_textured_drawing = Vec::<ModelInfo>::new();

        let default_texture_bytes = include_bytes!("../assets/textures/default.png");
        let default_texture = texture::Texture::from_bytes(
            &gpu.device,
            &gpu.queue,
            default_texture_bytes,
            "Default Texture",
        )
        .unwrap();
        let views: Vec<&wgpu::TextureView> = vec![&default_texture.view; 1024];
        let textures = Vec::<Option<texture::Texture>>::with_capacity(1024);

        let textures_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: std::num::NonZeroU32::new(1024),
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("Texture Bind Group Layout"),
                });

        let textures_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &textures_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&views),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&default_texture.sampler),
                },
            ],
            label: Some("Diffuse Bind Group"),
        });

        Self {
            meshes: Vec::<Option<Mesh>>::new(),
            textures,
            models_colored_drawing,
            models_textured_drawing,
            default_texture,
            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
            transform_buffer,
            transform_bind_group_layout,
            transform_bind_group,
            vertex_colored_buffer: None,
            index_colored_buffer: None,
            vertex_textured_buffer: None,
            index_textured_buffer: None,
            textures_bind_group_layout,
            textures_bind_group,
        }
    }

    pub fn add_object(&mut self, mesh: Mesh) -> usize {
        self.meshes.push(Some(mesh));
        self.meshes.len() - 1
    }

    pub fn add_texture(&mut self, texture: texture::Texture) -> usize {
        self.textures.push(Some(texture));
        self.textures.len() - 1
    }

    pub fn remove_object(&mut self, id: usize) {
        if id >= self.meshes.len() {
            return;
        };
        self.meshes[id] = None;
    }

    pub fn remove_texture(&mut self, id: usize) {
        if id >= self.textures.len() {
            return;
        };
        self.textures[id] = None;
    }

    pub fn change_object(&mut self, id: usize, mesh: Mesh) {
        if id >= self.meshes.len() {
            return;
        };
        self.meshes[id] = Some(mesh);
    }

    pub fn change_texture(&mut self, id: usize, texture: texture::Texture) {
        if id >= self.textures.len() {
            return;
        };
        self.textures[id] = Some(texture);
    }

    pub fn load_model_from_obj(&mut self, gpu: &Gpu, path: &str) -> usize {
        let load_options = tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        };

        let (models, materials) = tobj::load_obj(path, &load_options).unwrap();
        if let Some(model) = models.get(0) {
            let raw_mesh = &model.mesh;
            let mut verts = Vec::<Vertex>::with_capacity(raw_mesh.positions.len() / 3);
            for i in 0..(raw_mesh.positions.len() / 3) {
                let position = [
                    raw_mesh.positions[i * 3],
                    raw_mesh.positions[i * 3 + 1],
                    raw_mesh.positions[i * 3 + 2],
                ];
                let color = if !raw_mesh.vertex_color.is_empty() {
                    [
                        raw_mesh.vertex_color[i * 3],
                        raw_mesh.vertex_color[i * 3 + 1],
                        raw_mesh.vertex_color[i * 3 + 2],
                    ]
                } else {
                    [1.0, 1.0, 1.0]
                };

                let tex_coords = if !raw_mesh.texcoords.is_empty() {
                    [
                        raw_mesh.texcoords[i * 2],
                        1.0 - raw_mesh.texcoords[i * 2 + 1],
                    ]
                } else {
                    [0.0, 0.0]
                };

                verts.push(Vertex {
                    position,
                    color,
                    tex_coords,
                });
            }
            let mesh: Mesh;
            let materials = materials.unwrap();
            if let Some(mat_id) = raw_mesh.material_id {
                let mat = &materials[mat_id];
                if let Some(ref diffuse_texture_name) = mat.diffuse_texture {
                    let obj_path = Path::new(path);
                    let folder = obj_path.parent().unwrap();
                    let texture_path = folder.join(diffuse_texture_name);
                    if let Ok(image) = image::open(texture_path) {
                        let texture = texture::Texture::from_image(
                            &gpu.device,
                            &gpu.queue,
                            &image,
                            Some(format!("Texture For {}", path).as_str()),
                        )
                        .unwrap();
                        let texture_id = self.add_texture(texture);
                        mesh = Mesh::new_manually(
                            verts,
                            raw_mesh.indices.clone(),
                            Surface::Textured(texture_id as u32),
                            true,
                        );
                        self.add_object(mesh)
                    } else {
                        log::warn!("Не найдена текстура для модели {}", path);
                        mesh = Mesh::new_manually(
                            verts,
                            raw_mesh.indices.clone(),
                            Surface::Textured(0),
                            true,
                        );
                        self.add_object(mesh)
                    }
                } else {
                    log::warn!("Не найдена текстура для модели {}", path);
                    mesh = Mesh::new_manually(
                        verts,
                        raw_mesh.indices.clone(),
                        Surface::Textured(0),
                        true,
                    );
                    self.add_object(mesh)
                }
            } else {
                log::warn!("Не найдена текстура для модели {}", path);
                mesh =
                    Mesh::new_manually(verts, raw_mesh.indices.clone(), Surface::Textured(0), true);
                self.add_object(mesh)
            }
        } else {
            panic!("Unable to load model from {}", path);
        }
    }

    pub fn set_object_visibilty(&mut self, id: usize, visible: bool) {
        if id >= self.meshes.len() {
            return;
        };
        if let Some(mesh) = self.get_object(id) {
            mesh.visible = visible;
        }
    }

    pub fn get_object(&mut self, id: usize) -> Option<&mut Mesh> {
        self.meshes.get_mut(id).and_then(|slot| slot.as_mut())
    }

    pub fn update_camera(&mut self, gpu: &Gpu) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        gpu.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn build_transform_buffers(&mut self, gpu: &Gpu) {
        let trans_count: usize = self.meshes.iter().flatten().count();
        let mut trans: Vec<InstanceData> = Vec::with_capacity(trans_count);
        for model in &self.models_textured_drawing {
            if let Some(Some(cur_mesh)) = self.meshes.get(model.mesh_index) {
                trans.push(InstanceData::build_from_mesh(cur_mesh, model.texture_index));
            } else {
                log::warn!("Lost model");
            }
        }
        for model in &self.models_colored_drawing {
            if let Some(Some(cur_mesh)) = self.meshes.get(model.mesh_index) {
                trans.push(InstanceData::build_from_mesh(cur_mesh, model.texture_index));
            } else {
                log::warn!("Lost model");
            }
        }
        if trans.len() == 0 {
            trans.push(InstanceData::new());
        }
        self.transform_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Transform Buffer"),
                contents: bytemuck::cast_slice(&trans),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

        self.transform_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.transform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.transform_buffer.as_entire_binding(),
            }],
            label: Some("Transform Bind Group"),
        });
    }

    pub fn build_models_buffers(&mut self, gpu: &Gpu) {
        self.models_colored_drawing.clear();
        self.models_textured_drawing.clear();
        let mut textures_id_to_draw_id = HashMap::<u32, u32>::new();
        let mut views: Vec<&wgpu::TextureView> = vec![&self.default_texture.view; 1024];
        let mut last_texture_id: u32 = 1;
        let verts_textured_count: usize = self
            .meshes
            .iter()
            .flatten()
            .map(|mesh| {
                if matches!(mesh.surface, Surface::Textured(_)) {
                    mesh.vertices.len()
                } else {
                    0
                }
            })
            .sum();
        let mut verts_textured: Vec<Vertex> = Vec::with_capacity(verts_textured_count);
        let inds_textured_count: usize = self
            .meshes
            .iter()
            .flatten()
            .map(|mesh| {
                if matches!(mesh.surface, Surface::Textured(_)) {
                    mesh.indices.len()
                } else {
                    0
                }
            })
            .sum();
        let mut inds_textured: Vec<u32> = Vec::with_capacity(inds_textured_count);
        let verts_colored_count: usize = self
            .meshes
            .iter()
            .flatten()
            .map(|mesh| {
                if matches!(mesh.surface, Surface::Colored) {
                    mesh.vertices.len()
                } else {
                    0
                }
            })
            .sum();
        let mut verts_colored: Vec<Vertex> = Vec::with_capacity(verts_colored_count);
        let inds_colored_count: usize = self
            .meshes
            .iter()
            .flatten()
            .map(|mesh| {
                if matches!(mesh.surface, Surface::Colored) {
                    mesh.indices.len()
                } else {
                    0
                }
            })
            .sum();
        let mut inds_colored: Vec<u32> = Vec::with_capacity(inds_colored_count);
        let trans_textured_count: usize = self
            .meshes
            .iter()
            .flatten()
            .map(|mesh| {
                if matches!(mesh.surface, Surface::Textured(_)) {
                    mesh.indices.len()
                } else {
                    0
                }
            })
            .count();
        let mut trans_textured: Vec<InstanceData> = Vec::with_capacity(trans_textured_count);
        let trans_colored_count: usize = self
            .meshes
            .iter()
            .flatten()
            .map(|mesh| {
                if matches!(mesh.surface, Surface::Colored) {
                    mesh.indices.len()
                } else {
                    0
                }
            })
            .count();
        let mut trans_colored: Vec<InstanceData> = Vec::with_capacity(trans_colored_count);

        for (index, omesh) in self.meshes.iter().enumerate() {
            if let Some(mesh) = omesh {
                if let Surface::Textured(id) = mesh.surface {
                    let mut actual_texture: u32 = 0;
                    if let Some(actual_id) = textures_id_to_draw_id.get(&id).cloned() {
                        actual_texture = actual_id;
                    } else {
                        if let Some(Some(existing_texture)) = &self.textures.get(id as usize) {
                            textures_id_to_draw_id.insert(id, last_texture_id);
                            views[last_texture_id as usize] = &existing_texture.view;
                            if last_texture_id >= 1024 {
                                panic!("You can't render more than 1024 textures at a time");
                            }
                            actual_texture = last_texture_id;
                            last_texture_id += 1;
                        }
                    }
                    let model = ModelInfo {
                        mesh_index: index,
                        texture_index: actual_texture,
                        index_range: (inds_textured.len() as u32)
                            ..((inds_textured.len() + mesh.indices.len()) as u32),
                        base_vertex: verts_textured.len(),
                        index_transform: trans_textured.len() as u32,
                    };
                    verts_textured.extend_from_slice(&mesh.vertices);
                    inds_textured.extend_from_slice(&mesh.indices);
                    trans_textured.push(InstanceData::build_from_mesh(mesh, actual_texture));
                    self.models_textured_drawing.push(model);
                } else {
                    let model = ModelInfo {
                        mesh_index: index,
                        texture_index: 0,
                        index_range: (inds_colored.len() as u32)
                            ..((inds_colored.len() + mesh.indices.len()) as u32),
                        base_vertex: verts_colored.len(),
                        index_transform: trans_colored.len() as u32,
                    };
                    verts_colored.extend_from_slice(&mesh.vertices);
                    inds_colored.extend_from_slice(&mesh.indices);
                    trans_colored.push(InstanceData::build_from_mesh(mesh, 0));
                    self.models_colored_drawing.push(model);
                }
            }
        }
        let offset = trans_textured.len() as u32;
        self.models_colored_drawing
            .iter_mut()
            .for_each(|x| x.index_transform += offset);
        let mut trans: Vec<InstanceData> =
            trans_textured.into_iter().chain(trans_colored).collect();

        self.vertex_textured_buffer = match verts_textured.is_empty() {
            true => None,
            false => Some(
                gpu.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: bytemuck::cast_slice(&verts_textured),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
            ),
        };

        self.index_textured_buffer = match inds_textured.is_empty() {
            true => None,
            false => Some(
                gpu.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(&inds_textured),
                        usage: wgpu::BufferUsages::INDEX,
                    }),
            ),
        };

        self.textures_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.textures_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&views),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.default_texture.sampler),
                },
            ],
            label: Some("Diffuse Bind Group"),
        });

        self.vertex_colored_buffer = match verts_colored.is_empty() {
            true => None,
            false => Some(
                gpu.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: bytemuck::cast_slice(&verts_colored),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
            ),
        };

        self.index_colored_buffer = match inds_colored.is_empty() {
            true => None,
            false => Some(
                gpu.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(&inds_colored),
                        usage: wgpu::BufferUsages::INDEX,
                    }),
            ),
        };

        if trans.len() == 0 {
            trans.push(InstanceData::new());
        }

        self.transform_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Transform Buffer"),
                contents: bytemuck::cast_slice(&trans),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

        self.transform_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.transform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.transform_buffer.as_entire_binding(),
            }],
            label: Some("Transform Bind Group"),
        });
        log::info!("Buffers are built");
    }
}
