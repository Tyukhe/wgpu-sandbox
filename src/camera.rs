use winit::keyboard::KeyCode;

pub struct Camera {
    pub position: glam::Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> glam::Mat4 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        let forward =
            glam::Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize();
        let view = glam::Mat4::look_to_rh(self.position, forward, glam::Vec3::Y);

        let proj = glam::Mat4::perspective_infinite_reverse_rh(self.fovy, self.aspect, self.znear);

        proj * view
    }
}

pub struct CameraController {
    speed: f32,
    sensitivity: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
        }
    }

    pub fn handle_mouse(&mut self, dposition: glam::Vec2, camera: &mut Camera) {
        camera.yaw += dposition.x * self.sensitivity;
        camera.pitch -= dposition.y * self.sensitivity;
        let limit = 89.9f32.to_radians();
        camera.pitch = camera.pitch.clamp(-limit, limit);
    }

    pub fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            KeyCode::Space | KeyCode::ShiftLeft => {
                self.is_up_pressed = is_pressed;
                true
            }
            KeyCode::KeyX | KeyCode::ControlLeft => {
                self.is_down_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        let (sin_yaw, cos_yaw) = camera.yaw.sin_cos();

        let move_forward = glam::Vec3::new(cos_yaw, 0.0, sin_yaw).normalize();
        let move_right = glam::Vec3::new(-sin_yaw, 0.0, cos_yaw).normalize();
        let move_up = glam::Vec3::Y;

        let mut velocity = glam::Vec3::ZERO;

        if self.is_forward_pressed {
            velocity += move_forward;
        }
        if self.is_backward_pressed {
            velocity -= move_forward;
        }
        if self.is_right_pressed {
            velocity += move_right;
        }
        if self.is_left_pressed {
            velocity -= move_right;
        }

        if self.is_up_pressed {
            velocity += move_up;
        }
        if self.is_down_pressed {
            velocity -= move_up;
        }

        if velocity.length_squared() > 0.0 {
            camera.position += velocity.normalize() * self.speed;
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: glam::Mat4,
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: glam::Mat4::IDENTITY,
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix();
    }
}
