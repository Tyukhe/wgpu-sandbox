use crate::camera::{Camera, CameraController, CameraUniform};
use crate::gpu::Gpu;
use crate::mesh::{Mesh, Vertex};
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

    pub fn build_from_mesh(mesh: &Mesh) -> Self {
        Self {
            transform: mesh.build_model_matrix(),
            texture: mesh.texture_id.unwrap_or_default(),
            ..Default::default()
        }
    }
}

pub struct ModelInfo {
    pub index_range: std::ops::Range<u32>,
    pub base_vertex: usize,
    pub index_transform: u32,
}

pub struct Scene {
    meshes: Vec<Option<Mesh>>,
    pub camera: Camera,
    pub camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group: wgpu::BindGroup,
    transform_buffer: wgpu::Buffer,
    pub transform_bind_group_layout: wgpu::BindGroupLayout,
    pub transform_bind_group: wgpu::BindGroup,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub index_buffer: Option<wgpu::Buffer>,
    pub models: Vec<ModelInfo>,
    textures: (),
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
                        visibility: wgpu::ShaderStages::VERTEX,
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

        let models = Vec::<ModelInfo>::new();

        Self {
            meshes: Vec::<Option<Mesh>>::new(),
            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
            transform_buffer,
            transform_bind_group_layout,
            transform_bind_group,
            vertex_buffer: None,
            index_buffer: None,
            models,
            textures: (),
        }
    }

    pub fn add_object(&mut self, mesh: Mesh) -> usize {
        self.meshes.push(Some(mesh));
        self.meshes.len() - 1
    }

    pub fn remove_object(&mut self, id: usize) {
        if id >= self.meshes.len() {
            return;
        };
        self.meshes[id] = None;
    }

    pub fn change_object(&mut self, id: usize, mesh: Mesh) {
        if id >= self.meshes.len() {
            return;
        };
        self.meshes[id] = Some(mesh);
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
        for mesh in self.meshes.iter().flatten() {
            trans.push(InstanceData::build_from_mesh(mesh));
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
        self.models.clear();
        let verts_count: usize = self
            .meshes
            .iter()
            .flatten()
            .map(|mesh| mesh.vertices.len())
            .sum();
        let mut verts: Vec<Vertex> = Vec::with_capacity(verts_count);
        let inds_count: usize = self
            .meshes
            .iter()
            .flatten()
            .map(|mesh| mesh.indices.len())
            .sum();
        let mut inds: Vec<u32> = Vec::with_capacity(inds_count);
        let trans_count: usize = self.meshes.iter().flatten().count();
        let mut trans: Vec<InstanceData> = Vec::with_capacity(trans_count);
        for mesh in self.meshes.iter().flatten() {
            let model = ModelInfo {
                index_range: (inds.len() as u32)..((inds.len() + mesh.indices.len()) as u32),
                base_vertex: verts.len(),
                index_transform: trans.len() as u32,
            };
            verts.extend_from_slice(&mesh.vertices);
            inds.extend_from_slice(&mesh.indices);
            trans.push(InstanceData::build_from_mesh(mesh));
            self.models.push(model);
        }

        self.vertex_buffer = Some(gpu.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&verts),
                usage: wgpu::BufferUsages::VERTEX,
            },
        ));

        self.index_buffer = Some(gpu.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&inds),
                usage: wgpu::BufferUsages::INDEX,
            },
        ));

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
}
