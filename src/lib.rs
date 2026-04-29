mod camera;
mod core;
mod gpu;
mod mesh;
mod render;
mod texture;

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{CursorGrabMode, Window},
};

struct App {
    state: Option<core::State>,
}

impl App {
    fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler<()> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes()
            .with_title("wgpu_project")
            .with_inner_size(winit::dpi::LogicalSize::new(800.0f32, 800.0f32));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let _ = window.set_cursor_grab(CursorGrabMode::Confined);
        window.set_cursor_visible(false);

        self.state = Some(pollster::block_on(core::State::new(window)).unwrap());
        log::info!("Window created or recreated");
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };
        state.window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    log::warn!("Surface lost or outdated. Trying to reconfigure...");
                    let size = state.window.inner_size();
                    state.resize(size.width, size.height);
                }
                Err(e) => {
                    log::error!("{}", e);
                }
            },
            _ => {}
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    log::info!("Starting app...");

    let event_loop = EventLoop::new()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
