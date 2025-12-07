//! Application window and event loop handler.
//!
//! This module manages the main window and bridges winit events to the
//! game's internal event system.

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

use crate::input::events::RawInputEvent;
use crate::render::renderer::Renderer;
use crate::system::bus::{SystemBus, SystemEvent};

/// Main application struct handling window events.
pub struct App {
    bus: SystemBus,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
}

impl App {
    /// Creates a new application instance.
    pub fn new(bus: SystemBus) -> Self {
        Self {
            bus,
            window: None,
            renderer: None,
        }
    }

    /// Runs the application event loop (blocking).
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
        if let Some(renderer) = self.renderer.as_mut()
            && let Some(window) = self.window.as_ref()
            && renderer.handle_event(window, &event)
        {
            return;
        }

        match event {
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if let PhysicalKey::Code(keycode) = key_event.physical_key
                    && !key_event.repeat
                {
                    let raw_event = RawInputEvent {
                        keycode,
                        state: key_event.state,
                    };
                    let _ = self.bus.raw_input_tx.send(raw_event);
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
                if let Some(window) = self.window.as_ref() {
                    // Update state from logic thread
                    if let Some(snapshot) = self.bus.render_rx.try_iter().last()
                        && let Some(renderer) = self.renderer.as_mut()
                    {
                        renderer.update_state(snapshot);
                    }

                    // Render and send UI actions (mouse) to logic
                    if let Some(renderer) = self.renderer.as_mut() {
                        match renderer.render(window) {
                            Ok(actions) => {
                                for action in actions {
                                    let _ = self.bus.action_tx.send(action);
                                }
                            }
                            // Surface lost or outdated - reconfigure
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                renderer.resize(window.inner_size());
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                log::error!("Render error: Out of memory!");
                                event_loop.exit();
                            }
                            Err(wgpu::SurfaceError::Timeout) => {
                                // Frame dropped, not critical - continue
                                log::warn!("Render timeout - frame dropped");
                            }
                            #[allow(unreachable_patterns)]
                            Err(e) => log::error!("Render error: {e:?}"),
                        }
                    }
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

