use crate::core::input::InputManager;
use crate::core::input::actions::{KeyAction, GameAction, UIAction};
use crate::database::DbManager;
use crate::logic::game_loop::LogicLoop;
use crate::models::engine::NUM_COLUMNS;
use crate::models::settings::SettingsState; // NOUVEAU IMPORTS
use crate::renderer::Renderer;
use crate::shared::messages::{LogicToMain, MainToLogic};
use crate::shared::snapshot::RenderState;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::window::{Window, WindowId};

pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    
    logic_tx: Sender<MainToLogic>,
    main_rx: Receiver<LogicToMain>,
    
    modifiers: ModifiersState,
    input_manager: InputManager,
    
    menu_state: Arc<std::sync::Mutex<crate::models::menu::MenuState>>,
    state_stack: Vec<Box<dyn crate::states::GameState>>,
    db_manager: DbManager,
}

impl App {
    pub fn new() -> Self {
        let db_path = PathBuf::from("main.db");
        let songs_path = PathBuf::from("songs");
        let db_manager = DbManager::new(db_path, songs_path);
        
        let menu_state = Arc::new(std::sync::Mutex::new(crate::models::menu::MenuState::new()));

        let (main_tx, main_rx) = channel::<LogicToMain>();
        let (logic_tx, logic_rx) = channel::<MainToLogic>();

        LogicLoop::start(logic_rx, main_tx, db_manager); 
        
        let db_path_dummy = PathBuf::from("main.db"); 
        let songs_dummy = PathBuf::from("songs");
        let db_manager_dummy = DbManager::new(db_path_dummy, songs_dummy);

        let mut input_manager = InputManager::new();
        
        // CORRECTION CHARGEMENT AU DEMARRAGE
        // On charge la config disque immédiatement pour avoir les bons keybinds
        let settings = SettingsState::load();
        input_manager.update_key_count(NUM_COLUMNS);
        input_manager.bindings.reload_from_settings(&settings, NUM_COLUMNS);

        let mut app = Self {
            window: None,
            renderer: None,
            logic_tx,
            main_rx,
            modifiers: ModifiersState::default(),
            input_manager,
            menu_state: Arc::clone(&menu_state),
            state_stack: Vec::new(),
            db_manager: db_manager_dummy,
        };

        app.enter_state(Box::new(crate::states::MenuStateController::new(menu_state)));
        app
    }

    fn send_to_logic(&self, msg: MainToLogic) {
        if let Err(e) = self.logic_tx.send(msg) {
            eprintln!("ERREUR: Echec envoi vers Logic: {}", e);
        }
    }
    
    fn make_state_context(&mut self) -> crate::states::StateContext {
        let renderer_ptr = self.renderer.as_mut().map(|r| r as *mut Renderer);
        let logic_tx = Some(self.logic_tx.clone());
        crate::states::StateContext::new(renderer_ptr, None, logic_tx)
    }

    fn enter_state(&mut self, mut state: Box<dyn crate::states::GameState>) {
        let mut ctx = self.make_state_context();
        state.on_enter(&mut ctx);
        self.state_stack.push(state);
    }

    fn exit_state(&mut self) {
        if let Some(mut state) = self.state_stack.pop() {
            let mut ctx = self.make_state_context();
            state.on_exit(&mut ctx);
        }
    }

    fn replace_state(&mut self, mut state: Box<dyn crate::states::GameState>) {
        if let Some(mut current) = self.state_stack.pop() {
            let mut ctx = self.make_state_context();
            current.on_exit(&mut ctx);
        }
        let mut ctx = self.make_state_context();
        state.on_enter(&mut ctx);
        self.state_stack.push(state);
    }

    fn with_active_state<F>(&mut self, f: F) -> crate::states::StateTransition
    where F: FnOnce(&mut dyn crate::states::GameState, &mut crate::states::StateContext) -> crate::states::StateTransition
    {
        if self.state_stack.is_empty() { return crate::states::StateTransition::None; }
        let mut ctx = self.make_state_context();
        if let Some(state) = self.state_stack.last_mut() { f(state.as_mut(), &mut ctx) } else { crate::states::StateTransition::None }
    }

    fn apply_transition(&mut self, transition: crate::states::StateTransition, event_loop: &ActiveEventLoop) {
        match transition {
            crate::states::StateTransition::None => {}
            crate::states::StateTransition::Push(state) => self.enter_state(state),
            crate::states::StateTransition::Pop => self.exit_state(),
            crate::states::StateTransition::Replace(state) => self.replace_state(state),
            crate::states::StateTransition::Exit => event_loop.exit(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = winit::window::Window::default_attributes()
                .with_title("rVsrg")
                .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));
            let window = Arc::new(event_loop.create_window(win_attr).unwrap());
            self.window = Some(window.clone());
            let renderer = pollster::block_on(Renderer::new(window.clone()));
            self.renderer = Some(renderer);
            
            // On re-applique les binds au renderer nouvellement créé (pour l'affichage UI des keys)
            // Note: input_manager les a déjà, mais si le renderer a besoin de charger quelque chose de visuel c'est ici.
            
            if let Some(window) = &self.window { window.request_redraw(); }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        while let Ok(msg) = self.main_rx.try_recv() {
            match msg {
                LogicToMain::StateUpdate(new_state) => {
                    if let Some(renderer) = self.renderer.as_mut() {
                        renderer.current_state = new_state.clone();
                    }
                    if let RenderState::Menu(updated_menu) = new_state {
                        if let Ok(mut lock) = self.menu_state.lock() {
                            *lock = updated_menu;
                        }
                    }
                },
                LogicToMain::TransitionToResult(result_data) => {
                    self.replace_state(Box::new(crate::states::ResultStateController::new(
                        Arc::clone(&self.menu_state),
                        result_data.hit_stats, result_data.replay_data, result_data.score, result_data.accuracy, result_data.max_combo
                    )));
                },
                LogicToMain::TransitionToMenu => {
                    self.replace_state(Box::new(crate::states::MenuStateController::new(Arc::clone(&self.menu_state))));
                },
                LogicToMain::TransitionToEditor => {
                    self.enter_state(Box::new(crate::states::EditorStateController::new(Arc::clone(&self.menu_state))));
                },
                LogicToMain::ToggleSettings => {
                    if let Some(renderer) = self.renderer.as_mut() {
                        renderer.toggle_settings();
                    }
                },
                LogicToMain::AudioCommand(_) => { },
                LogicToMain::ExitApp => { event_loop.exit(); }
            }
        }

        let mut event_consumed_by_egui = false;
        if let (Some(renderer), Some(window)) = (self.renderer.as_mut(), self.window.as_ref()) {
            if renderer.handle_event(window, &event) {
                event_consumed_by_egui = true;
            }
            if renderer.egui_ctx.wants_keyboard_input() {
                event_consumed_by_egui = true;
            }
        }

        match &event {
            WindowEvent::CloseRequested => {
                self.send_to_logic(MainToLogic::Shutdown);
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(renderer) = self.renderer.as_mut() { renderer.resize(*physical_size); }
                self.send_to_logic(MainToLogic::Resize { width: physical_size.width, height: physical_size.height });
            }
            WindowEvent::ModifiersChanged(new_modifiers) => { self.modifiers = new_modifiers.state(); }
            
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                if key_event.state == ElementState::Pressed && key_event.physical_key == PhysicalKey::Code(KeyCode::KeyO) {
                    if self.modifiers.control_key() {
                        self.send_to_logic(MainToLogic::Input(KeyAction::UI(UIAction::ToggleSettings)));
                        return; 
                    }
                }

                if !event_consumed_by_egui {
                    // On tente de traduire l'input
                    let action = self.input_manager.process_event(&event);
                    
                    // 1. Si c'est une action connue, on l'envoie au Logic
                    if let Some(act) = action {
                        self.send_to_logic(MainToLogic::Input(act));
                    }

                    // 2. CRUCIAL : On passe TOUJOURS l'event brut au State local
                    // C'est ça qui permet à l'EditorState de recevoir 'W', 'S' même si ce ne sont pas des actions bindées.
                    let transition = self.with_active_state(|state, ctx| state.handle_input(&event, action, ctx));
                    self.apply_transition(transition, event_loop);
                }
            }
            
            WindowEvent::RedrawRequested => {
                let transition = self.with_active_state(|state, ctx| match state.update(ctx) {
                    crate::states::StateTransition::None => state.render(ctx),
                    other => other,
                });
                self.apply_transition(transition, event_loop);

                if let (Some(renderer), Some(window)) = (self.renderer.as_mut(), self.window.as_ref()) {
                    match renderer.render(window) {
                        Ok((settings_changed, ui_messages)) => {
                            if settings_changed {
                                self.input_manager.bindings.reload_from_settings(&renderer.settings, NUM_COLUMNS);
                            }
                            for msg in ui_messages {
                                self.send_to_logic(msg);
                            }
                        }
                        Err(wgpu::SurfaceError::Lost) => {}
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}