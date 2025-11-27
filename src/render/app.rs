use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

use crate::input::events::RawInputEvent;
use crate::render::renderer::Renderer;
use crate::system::bus::{SystemBus, SystemEvent};

pub struct App {
    bus: SystemBus,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
}

impl App {
    pub fn new(bus: SystemBus) -> Self {
        Self {
            bus,
            window: None,
            renderer: None,
        }
    }

    pub fn run(bus: SystemBus) {
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let mut app = App::new(bus);
        let _ = event_loop.run_app(&mut app);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            log::info!("RENDER: Creating window...");
            let win_attr = winit::window::Window::default_attributes()
                .with_title("rVsrg 2.0")
                .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));

            let window = Arc::new(event_loop.create_window(win_attr).unwrap());
            self.window = Some(window.clone());

            log::info!("RENDER: Initializing WGPU...");
            let renderer = pollster::block_on(Renderer::new(window.clone()));
            self.renderer = Some(renderer);

            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(renderer) = self.renderer.as_mut() {
            if let Some(window) = self.window.as_ref() {
                if renderer.handle_event(window, &event) {
                    return;
                }
            }
        }

        match event {
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if let PhysicalKey::Code(keycode) = key_event.physical_key {
                    if !key_event.repeat {
                        let raw_event = RawInputEvent {
                            keycode,
                            state: key_event.state,
                        };
                        let _ = self.bus.raw_input_tx.send(raw_event);
                    }
                }
            }
            WindowEvent::CloseRequested => {
                log::info!("RENDER: Close requested");
                let _ = self.bus.sys_tx.send(SystemEvent::Quit);
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(physical_size);
                }
                let _ = self.bus.sys_tx.send(SystemEvent::Resize {
                    width: physical_size.width,
                    height: physical_size.height,
                });
            }
            WindowEvent::RedrawRequested => {
                if let Some(_window) = self.window.as_ref() {
                    // Apply the latest state snapshot from the logic thread.
                    if let Some(snapshot) = self.bus.render_rx.try_iter().last() {
                        if let Some(renderer) = self.renderer.as_mut() {
                            renderer.update_state(snapshot);
                        }
                    }

                    // Rendu et envoi des actions UI (souris) vers la logique
                    if let Some(renderer) = self.renderer.as_mut() {
                        match renderer.render(_window) {
                            Ok(actions) => {
                                for action in actions {
                                    let _ = self.bus.action_tx.send(action);
                                }
                            }
                            Err(wgpu::SurfaceError::Lost) => renderer.resize(_window.inner_size()),
                            Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                            Err(e) => log::error!("Render error: {:?}", e),
                        }
                    }
                    _window.request_redraw();
                }
            }
            _ => {}
        }
    }
}
