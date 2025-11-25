use super::{GameState, MenuStateController, StateContext, StateTransition};
use crate::models::engine::NUM_COLUMNS;
use crate::models::menu::MenuState;
use std::sync::{Arc, Mutex};
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditTarget {
    Notes,
    Receptors,
    Combo,
    Score,
    Accuracy,
    Judgement,
    Counter,
    HitBar,
    Lanes,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMode {
    Resize,
    Move,
}

impl std::fmt::Display for EditTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditTarget::Notes => write!(f, "NOTES"),
            EditTarget::Receptors => write!(f, "RECEPTORS"),
            EditTarget::Combo => write!(f, "COMBO"),
            EditTarget::Score => write!(f, "SCORE"),
            EditTarget::Accuracy => write!(f, "ACCURACY"),
            EditTarget::Judgement => write!(f, "FLASH TEXT"),
            EditTarget::Counter => write!(f, "COUNTER LIST"),
            EditTarget::HitBar => write!(f, "HIT BAR"),
            EditTarget::Lanes => write!(f, "LANES"),
        }
    }
}
impl std::fmt::Display for EditMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditMode::Resize => write!(f, "RESIZE/SIZE"),
            EditMode::Move => write!(f, "MOVE"),
        }
    }
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
    fn with_menu_state<F>(&self, mut f: F)
    where
        F: FnMut(&mut MenuState),
    {
        if let Ok(mut state) = self.menu_state.lock() {
            f(&mut state);
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
                EditTarget::Notes
                | EditTarget::Receptors
                | EditTarget::Lanes
                | EditTarget::HitBar
                | EditTarget::Combo
                | EditTarget::Score
                | EditTarget::Accuracy
                | EditTarget::Counter => EditMode::Resize,
                _ => EditMode::Move,
            };
        }
    }

    fn apply_change(&self, ctx: &mut StateContext, dx: f32, dy: f32) {
        let Some(target) = self.current_target else {
            return;
        };
        ctx.with_renderer(|renderer| {
            let config = &mut renderer.skin.config;
            let multiplier = if renderer.settings.is_open { 0.0 } else { 1.0 };
            let speed = 2.0 * multiplier;

            match (target, self.current_mode) {
                (EditTarget::Notes, EditMode::Resize) => {
                    config.note_width_px += dx * speed;
                    config.note_height_px += dy * speed;
                }
                (EditTarget::Notes, EditMode::Move) => {
                    let p = config
                        .playfield_pos
                        .get_or_insert(crate::models::skin::UIElementPos { x: 0., y: 0. });
                    p.x += dx * speed;
                    p.y += dy * speed;
                }
                (EditTarget::Receptors, EditMode::Resize) => {
                    config.receptor_width_px += dx * speed;
                    config.receptor_height_px += dy * speed;
                }
                (EditTarget::Receptors, EditMode::Move) => {
                    let p = config
                        .playfield_pos
                        .get_or_insert(crate::models::skin::UIElementPos { x: 0., y: 0. });
                    p.x += dx * speed;
                    p.y += dy * speed;
                }
                (EditTarget::Lanes, EditMode::Resize) => {
                    config.column_width_px += dx * speed;
                    config.receptor_spacing_px += dy * speed;
                }

                (EditTarget::Combo, EditMode::Move) => {
                    let p = config
                        .combo_pos
                        .get_or_insert(crate::models::skin::UIElementPos { x: 0., y: 0. });
                    p.x += dx * speed;
                    p.y += dy * speed;
                }
                (EditTarget::Combo, EditMode::Resize) => {
                    config.combo_text_size += dy * speed;
                }

                (EditTarget::Score, EditMode::Move) => {
                    let p = config
                        .score_pos
                        .get_or_insert(crate::models::skin::UIElementPos { x: 0., y: 0. });
                    p.x += dx * speed;
                    p.y += dy * speed;
                }
                (EditTarget::Score, EditMode::Resize) => {
                    config.score_text_size += dy * speed;
                }

                (EditTarget::Accuracy, EditMode::Move) => {
                    let p = config
                        .accuracy_pos
                        .get_or_insert(crate::models::skin::UIElementPos { x: 0., y: 0. });
                    p.x += dx * speed;
                    p.y += dy * speed;
                }
                (EditTarget::Accuracy, EditMode::Resize) => {
                    config.accuracy_text_size += dy * speed;
                }

                (EditTarget::Judgement, EditMode::Move) => {
                    let p = config
                        .judgement_flash_pos
                        .get_or_insert(crate::models::skin::UIElementPos { x: 0., y: 0. });
                    p.x += dx * speed;
                    p.y += dy * speed;
                }

                (EditTarget::Counter, EditMode::Move) => {
                    let p = config
                        .judgement_pos
                        .get_or_insert(crate::models::skin::UIElementPos { x: 0., y: 0. });
                    p.x += dx * speed;
                    p.y += dy * speed;
                }
                (EditTarget::Counter, EditMode::Resize) => {
                    config.judgement_text_size += dy * speed;
                }

                (EditTarget::HitBar, EditMode::Move) => {
                    let p = config
                        .hit_bar_pos
                        .get_or_insert(crate::models::skin::UIElementPos { x: 0., y: 0. });
                    p.x += dx * speed;
                    p.y += dy * speed;
                }
                (EditTarget::HitBar, EditMode::Resize) => {
                    config.hit_bar_height_px += dy * speed;
                }
                _ => {}
            }

            let pf_conf = &mut renderer.gameplay_view.playfield_component_mut().config;
            pf_conf.column_width_pixels = config.column_width_px;
            pf_conf.note_width_pixels = config.note_width_px;
            pf_conf.note_height_pixels = config.note_height_px;
            pf_conf.receptor_width_pixels = config.receptor_width_px;
            pf_conf.receptor_height_pixels = config.receptor_height_px;
            pf_conf.receptor_spacing_pixels = config.receptor_spacing_px;
            renderer.update_component_positions();
        });
    }
}

impl GameState for EditorStateController {
    fn on_enter(&mut self, _ctx: &mut StateContext) {
        self.with_menu_state(|state| {
            state.in_menu = false;
            state.in_editor = true;
        });
    }
    fn handle_input(&mut self, event: &WindowEvent, ctx: &mut StateContext) -> StateTransition {
        if let WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    state: input_state,
                    physical_key: PhysicalKey::Code(key_code),
                    ..
                },
            ..
        } = event
        {
            let is_pressed = *input_state == ElementState::Pressed;
            if is_pressed {
                match key_code {
                    KeyCode::KeyW => self.select_target(EditTarget::Notes),
                    KeyCode::KeyX => self.select_target(EditTarget::Receptors),
                    KeyCode::KeyC => self.select_target(EditTarget::Combo),
                    KeyCode::KeyV => self.select_target(EditTarget::Score),
                    KeyCode::KeyB => self.select_target(EditTarget::Accuracy),
                    KeyCode::KeyN => self.select_target(EditTarget::Judgement),
                    KeyCode::KeyJ => self.select_target(EditTarget::Counter),
                    KeyCode::KeyK => self.select_target(EditTarget::HitBar),
                    KeyCode::KeyL => self.select_target(EditTarget::Lanes),
                    KeyCode::KeyS => {
                        ctx.with_renderer(|renderer| {
                            if let Err(e) = renderer.skin.save_user_config() {
                                eprintln!("Error: {}", e);
                            } else {
                                println!("Saved!");
                            }
                        });
                    }
                    KeyCode::Escape => {
                        ctx.with_renderer(|renderer| {
                            renderer.stop_audio();
                            renderer.editor_status_text = None;
                            renderer.editor_values_text = None;
                        });
                        self.with_menu_state(|state| {
                            state.in_menu = true;
                            state.in_editor = false;
                        });
                        return StateTransition::Replace(Box::new(MenuStateController::new(
                            Arc::clone(&self.menu_state),
                        )));
                    }
                    _ => {}
                }
            }
            if is_pressed {
                match key_code {
                    KeyCode::ArrowLeft => self.apply_change(ctx, -1.0, 0.0),
                    KeyCode::ArrowRight => self.apply_change(ctx, 1.0, 0.0),
                    KeyCode::ArrowUp => self.apply_change(ctx, 0.0, -1.0),
                    KeyCode::ArrowDown => self.apply_change(ctx, 0.0, 1.0),
                    _ => {}
                }
            }
            match key_code {
                KeyCode::KeyW
                | KeyCode::KeyX
                | KeyCode::KeyC
                | KeyCode::KeyV
                | KeyCode::KeyB
                | KeyCode::KeyN
                | KeyCode::KeyJ
                | KeyCode::KeyK
                | KeyCode::KeyL
                | KeyCode::KeyS
                | KeyCode::ArrowUp
                | KeyCode::ArrowDown
                | KeyCode::ArrowLeft
                | KeyCode::ArrowRight => {}
                _ => {
                    let key_name = format!("{:?}", key_code);
                    ctx.with_renderer(|renderer| {
                        let num_cols_str = NUM_COLUMNS.to_string();
                        let current_binds = renderer
                            .settings
                            .keybinds
                            .get(&num_cols_str)
                            .cloned()
                            .unwrap_or_default();
                        let mut column = current_binds.iter().position(|k| k == &key_name);
                        if column.is_none() {
                            let mut char_keys = Vec::new();
                            match *key_code {
                                KeyCode::KeyQ => char_keys.push("a"),
                                KeyCode::KeyW => char_keys.push("z"),
                                KeyCode::KeyA => char_keys.push("q"),
                                KeyCode::Comma => char_keys.push(";"),
                                KeyCode::Semicolon => char_keys.push("m"),
                                _ => {}
                            }
                            for ch in char_keys {
                                if let Some(col) = current_binds
                                    .iter()
                                    .position(|k| k.eq_ignore_ascii_case(ch))
                                {
                                    column = Some(col);
                                    break;
                                }
                            }
                        }
                        if let Some(col) = column {
                            if is_pressed {
                                renderer.engine.set_key_held(col, true);
                                renderer.engine.process_input(col);
                            } else {
                                renderer.engine.set_key_held(col, false);
                            }
                        }
                    });
                }
            }
        }
        StateTransition::None
    }
    fn update(&mut self, ctx: &mut StateContext) -> StateTransition {
        let target_copy = self.current_target;
        ctx.with_renderer(|renderer| {
            if let Some(target) = target_copy {
                renderer.editor_status_text =
                    Some(format!("EDITING: {} [{}]", target, self.current_mode));
            } else {
                renderer.editor_status_text = Some(
                    "SELECT: W(Note) X(Rec) C(Cmb) V(Scr) B(Acc) N(Flash) J(List) K(Bar) L(Lane)"
                        .to_string(),
                );
            }
            let conf = &renderer.skin.config;
            let val_str = match target_copy {
                Some(EditTarget::Notes) => format!(
                    "W: {:.0} | H: {:.0} | PosX: {:.0}",
                    conf.note_width_px,
                    conf.note_height_px,
                    conf.playfield_pos.map(|p| p.x).unwrap_or(0.0)
                ),
                Some(EditTarget::Receptors) => format!(
                    "W: {:.0} | H: {:.0} | PosX: {:.0}",
                    conf.receptor_width_px,
                    conf.receptor_height_px,
                    conf.playfield_pos.map(|p| p.x).unwrap_or(0.0)
                ),
                Some(EditTarget::Lanes) => format!(
                    "Col W: {:.0} | Spacing: {:.0}",
                    conf.column_width_px, conf.receptor_spacing_px
                ),
                Some(EditTarget::Combo) => format!(
                    "Size: {:.0} | Pos: {:.0}, {:.0}",
                    conf.combo_text_size,
                    conf.combo_pos.map(|p| p.x).unwrap_or(0.),
                    conf.combo_pos.map(|p| p.y).unwrap_or(0.)
                ),
                Some(EditTarget::Score) => format!(
                    "Size: {:.0} | Pos: {:.0}, {:.0}",
                    conf.score_text_size,
                    conf.score_pos.map(|p| p.x).unwrap_or(0.),
                    conf.score_pos.map(|p| p.y).unwrap_or(0.)
                ),
                Some(EditTarget::Accuracy) => format!(
                    "Size: {:.0} | Pos: {:.0}, {:.0}",
                    conf.accuracy_text_size,
                    conf.accuracy_pos.map(|p| p.x).unwrap_or(0.),
                    conf.accuracy_pos.map(|p| p.y).unwrap_or(0.)
                ),
                Some(EditTarget::Counter) => format!(
                    "Size: {:.0} | Pos: {:.0}, {:.0}",
                    conf.judgement_text_size,
                    conf.judgement_pos.map(|p| p.x).unwrap_or(0.),
                    conf.judgement_pos.map(|p| p.y).unwrap_or(0.)
                ),
                Some(EditTarget::HitBar) => format!(
                    "Height: {:.0} | Pos: {:.0}, {:.0}",
                    conf.hit_bar_height_px,
                    conf.hit_bar_pos.map(|p| p.x).unwrap_or(0.),
                    conf.hit_bar_pos.map(|p| p.y).unwrap_or(0.)
                ),
                Some(EditTarget::Judgement) => format!(
                    "Pos: {:.0}, {:.0}",
                    conf.judgement_flash_pos.map(|p| p.x).unwrap_or(0.),
                    conf.judgement_flash_pos.map(|p| p.y).unwrap_or(0.)
                ),
                None => "Select an element to see values".to_string(),
            };
            renderer.editor_values_text = Some(val_str);
        });
        let game_finished = ctx
            .with_renderer(|renderer| renderer.engine.is_game_finished())
            .unwrap_or(false);
        if game_finished {
            ctx.with_renderer(|renderer| renderer.engine.reset_time());
        }
        StateTransition::None
    }
}
