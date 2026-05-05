#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub texture_id: Option<u32>,
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: f32,
    pub visible: bool,
}

impl Mesh {
    pub fn new_manually(verts: Vec<Vertex>, inds: Vec<u32>, visible: bool) -> Self {
        Self {
            vertices: verts,
            indices: inds,
            texture_id: None,
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: 1.0,
            visible,
        }
    }

    pub fn set_texture(&mut self, texture_id: u32) {
        self.texture_id = Some(texture_id);
    }

    pub fn set_position(&mut self, position: glam::Vec3) {
        self.position = position;
    }

    pub fn get_position(&self) -> glam::Vec3 {
        self.position.clone()
    }

    pub fn relocate(&mut self, offset: glam::Vec3) {
        self.position += offset;
    }

    pub fn set_rotation(&mut self, rotation: glam::Quat) {
        self.rotation = rotation;
    }

    pub fn get_rotation(&self) -> glam::Quat {
        self.rotation.clone()
    }

    pub fn turn(&mut self, offset: glam::Quat) {
        self.rotation *= offset;
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    pub fn get_scale(&self) -> f32 {
        self.scale
    }

    pub fn resize(&mut self, offset: f32) {
        self.scale += offset;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn get_visible(&self) -> bool {
        self.visible
    }

    pub fn build_model_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::splat(self.scale),
            self.rotation,
            self.position,
        )
    }
}
