use super::{GameState, MenuStateController, StateContext, StateTransition};
use crate::core::input::actions::{KeyAction, UIAction};
use crate::models::menu::MenuState;
use crate::shared::messages::MainToLogic;
use std::sync::{Arc, Mutex};
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditTarget { Notes, Receptors, Combo, Score, Accuracy, Judgement, Counter, HitBar, Lanes }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMode { Resize, Move }

impl std::fmt::Display for EditTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { match self { EditTarget::Notes => write!(f, "NOTES"), EditTarget::Receptors => write!(f, "RECEPTORS"), EditTarget::Combo => write!(f, "COMBO"), EditTarget::Score => write!(f, "SCORE"), EditTarget::Accuracy => write!(f, "ACCURACY"), EditTarget::Judgement => write!(f, "JUDGEMENT"), EditTarget::Counter => write!(f, "COUNTER"), EditTarget::HitBar => write!(f, "HIT BAR"), EditTarget::Lanes => write!(f, "LANES") } }
}
impl std::fmt::Display for EditMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { match self { EditMode::Resize => write!(f, "RESIZE"), EditMode::Move => write!(f, "MOVE") } }
}

pub struct EditorStateController {
    menu_state: Arc<Mutex<MenuState>>,
    pub current_target: Option<EditTarget>,
    pub current_mode: EditMode,
}

impl EditorStateController {
    pub fn new(menu_state: Arc<Mutex<MenuState>>) -> Self {
        Self {
            menu_state,
            current_target: None,
            current_mode: EditMode::Move,
        }
    }

    fn select_target(&mut self, target: EditTarget) {
        if self.current_target == Some(target) {
            self.current_mode = match self.current_mode {
                EditMode::Resize => EditMode::Move,
                EditMode::Move => EditMode::Resize,
            };
        } else {
            self.current_target = Some(target);
            self.current_mode = match target {
                // Certains éléments comme Notes ont du sens à être redimensionnés par défaut
                EditTarget::Notes | EditTarget::Receptors | EditTarget::HitBar => EditMode::Resize,
                _ => EditMode::Move,
            };
        }
    }

    fn apply_change(&self, ctx: &mut StateContext, dx: f32, dy: f32) {
        let Some(target) = self.current_target else { return; };
        
        ctx.with_renderer(|renderer| {
            let config = &mut renderer.skin.config;
            let speed = if renderer.settings.is_open { 0.0 } else { 2.0 }; // Bloquer si settings ouverts (sécurité)

            // Logique de modification selon le target
            match (target, self.current_mode) {
                (EditTarget::Notes, EditMode::Resize) => { 
                    config.note_width_px += dx * speed; 
                    config.note_height_px += dy * speed; 
                }
                (EditTarget::Receptors, EditMode::Resize) => { 
                    config.receptor_width_px += dx * speed; 
                    config.receptor_height_px += dy * speed; 
                }
                
                // Déplacement générique
                (_, EditMode::Move) => {
                    let pos_opt = match target {
                        EditTarget::Notes | EditTarget::Lanes | EditTarget::Receptors => &mut config.playfield_pos,
                        EditTarget::Combo => &mut config.combo_pos,
                        EditTarget::Score => &mut config.score_pos,
                        EditTarget::Accuracy => &mut config.accuracy_pos,
                        EditTarget::Judgement => &mut config.judgement_pos, 
                        EditTarget::HitBar => &mut config.hit_bar_pos,
                        _ => return,
                    };
                    
                    let p = pos_opt.get_or_insert(crate::models::skin::UIElementPos { x: 0., y: 0. });
                    p.x += dx * speed;
                    p.y += dy * speed; 
                }
                
                // Resize Text
                (EditTarget::Combo, EditMode::Resize) => config.combo_text_size += dy * speed,
                (EditTarget::Score, EditMode::Resize) => config.score_text_size += dy * speed,
                (EditTarget::Accuracy, EditMode::Resize) => config.accuracy_text_size += dy * speed,
                (EditTarget::Judgement, EditMode::Resize) => config.judgement_text_size += dy * speed,
                (EditTarget::HitBar, EditMode::Resize) => config.hit_bar_height_px += dy * speed,
                
                _ => {}
            }
            
            // Important : Mise à jour des positions réelles à l'écran
            renderer.update_component_positions();
        });
    }
}

impl GameState for EditorStateController {
    fn on_enter(&mut self, _ctx: &mut StateContext) {
        if let Ok(mut state) = self.menu_state.lock() {
            state.in_menu = false;
            state.in_editor = true;
        }
    }
    
    fn handle_input(&mut self, event: &WindowEvent, action: Option<KeyAction>, ctx: &mut StateContext) -> StateTransition {
        // 1. Gestion des Actions mappées (Flèches, Back)
        if let Some(KeyAction::UI(ui_action)) = action {
            match ui_action {
                // On inverse DY pour UIAction::Up car dans le repère écran, Up = Y négatif souvent, 
                // mais pour l'utilisateur "Monter" = augmenter Y ou diminuer Y selon la convention.
                // Ici on fait : Haut = -Y, Bas = +Y.
                UIAction::Up => self.apply_change(ctx, 0.0, -1.0),
                UIAction::Down => self.apply_change(ctx, 0.0, 1.0),
                UIAction::Left => self.apply_change(ctx, -1.0, 0.0),
                UIAction::Right => self.apply_change(ctx, 1.0, 0.0),
                
                UIAction::Back => {
                    // Quitter proprement
                    ctx.with_renderer(|r| { r.editor_status_text = None; r.editor_values_text = None; });
                    // On signale au logic de revenir au menu
                    ctx.send_to_logic(MainToLogic::Input(KeyAction::UI(UIAction::Back))); 
                    return StateTransition::Replace(Box::new(MenuStateController::new(Arc::clone(&self.menu_state))));
                }
                _ => {}
            }
        }

        // 2. Gestion des Touches Brutes pour la sélection
        if let WindowEvent::KeyboardInput { event: KeyEvent { state: ElementState::Pressed, physical_key: PhysicalKey::Code(key_code), .. }, .. } = event {
            match key_code {
                KeyCode::KeyW => self.select_target(EditTarget::Notes),
                KeyCode::KeyX => self.select_target(EditTarget::Receptors),
                KeyCode::KeyC => self.select_target(EditTarget::Combo),
                KeyCode::KeyV => self.select_target(EditTarget::Score),
                KeyCode::KeyB => self.select_target(EditTarget::Accuracy),
                KeyCode::KeyN => self.select_target(EditTarget::Judgement),
                KeyCode::KeyK => self.select_target(EditTarget::HitBar),
                KeyCode::KeyL => self.select_target(EditTarget::Lanes),
                
                KeyCode::KeyS => {
                    ctx.with_renderer(|renderer| {
                        if let Err(e) = renderer.skin.save_user_config() { eprintln!("Error saving: {}", e); }
                        renderer.editor_status_text = Some("SAVED!".to_string());
                    });
                }
                _ => {}
            }
        }
        
        StateTransition::None
    }

    fn update(&mut self, ctx: &mut StateContext) -> StateTransition {
        let target_copy = self.current_target;
        let mode_copy = self.current_mode;
        
        ctx.with_renderer(|renderer| {
            let config = &renderer.skin.config;

            if let Some(target) = target_copy {
                renderer.editor_status_text = Some(format!("EDIT: {} [{}]", target, mode_copy));
                
                // CORRECTION : On lit et formate les vraies valeurs
                let values_str = match target {
                    EditTarget::Notes => format!("W: {:.1} H: {:.1}", config.note_width_px, config.note_height_px),
                    EditTarget::Receptors => format!("W: {:.1} H: {:.1}", config.receptor_width_px, config.receptor_height_px),
                    EditTarget::HitBar => {
                        let p = config.hit_bar_pos.unwrap_or(crate::models::skin::UIElementPos { x: 0., y: 0. });
                        format!("X: {:.1} Y: {:.1} | H: {:.1}", p.x, p.y, config.hit_bar_height_px)
                    },
                    EditTarget::Combo => {
                        let p = config.combo_pos.unwrap_or(crate::models::skin::UIElementPos { x: 0., y: 0. });
                        format!("X: {:.1} Y: {:.1} | Size: {:.1}", p.x, p.y, config.combo_text_size)
                    },
                    EditTarget::Score => {
                        let p = config.score_pos.unwrap_or(crate::models::skin::UIElementPos { x: 0., y: 0. });
                        format!("X: {:.1} Y: {:.1} | Size: {:.1}", p.x, p.y, config.score_text_size)
                    },
                    EditTarget::Accuracy => {
                        let p = config.accuracy_pos.unwrap_or(crate::models::skin::UIElementPos { x: 0., y: 0. });
                        format!("X: {:.1} Y: {:.1} | Size: {:.1}", p.x, p.y, config.accuracy_text_size)
                    },
                    EditTarget::Judgement => {
                        let p = config.judgement_pos.unwrap_or(crate::models::skin::UIElementPos { x: 0., y: 0. });
                        format!("X: {:.1} Y: {:.1} | Size: {:.1}", p.x, p.y, config.judgement_text_size)
                    },
                    EditTarget::Lanes | EditTarget::Counter => {
                        let p = config.playfield_pos.unwrap_or(crate::models::skin::UIElementPos { x: 0., y: 0. });
                        format!("Playfield X: {:.1} Y: {:.1}", p.x, p.y)
                    }
                };
                
                renderer.editor_values_text = Some(values_str);
            } else {
                renderer.editor_status_text = Some("SELECT: W(Note) X(Rec) C(Cmb) V(Scr) B(Acc) N(Judg) K(Bar) | S(Save)".to_string());
                renderer.editor_values_text = Some("Press key to select element".to_string());
            }
        });
        StateTransition::None
    }
}