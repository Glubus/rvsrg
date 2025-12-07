use crate::input::events::{EditMode, EditorTarget, GameAction};
use crate::state::editor::EditorState;
use crate::state::global::GlobalState;
use crate::state::global::app_state::AppState;

pub fn apply(
    state: &mut GlobalState,
    editor: &mut EditorState,
    action: &GameAction,
) -> Option<AppState> {
    match action {
        GameAction::Back => {
            editor.engine.audio_manager.stop();
            state.requested_leaderboard_hash = None;
            let menu = state.saved_menu_state.clone();
            let request_hash = menu.get_selected_beatmap_hash();
            state.request_leaderboard_for_hash(request_hash);
            Some(AppState::Menu(menu))
        }
        GameAction::EditorSelect(t) => {
            if editor.target == Some(*t) {
                editor.mode = match editor.mode {
                    EditMode::Resize => EditMode::Move,
                    EditMode::Move => EditMode::Resize,
                };
            } else {
                editor.target = Some(*t);
                editor.mode = match t {
                    EditorTarget::Notes | EditorTarget::Receptors | EditorTarget::HitBar => {
                        EditMode::Resize
                    }
                    _ => EditMode::Move,
                };
            }
            None
        }
        GameAction::Navigation { x, y } => {
            if editor.target.is_some() {
                let (dx, dy) = (*x as f32, *y as f32);
                if let Some((old_dx, old_dy)) = editor.modification_buffer {
                    editor.modification_buffer = Some((old_dx + dx, old_dy + dy));
                } else {
                    editor.modification_buffer = Some((dx, dy));
                }
            }
            None
        }
        GameAction::EditorModify { x, y } => {
            if editor.target.is_some() {
                if let Some((old_dx, old_dy)) = editor.modification_buffer {
                    editor.modification_buffer = Some((old_dx + *x, old_dy + *y));
                } else {
                    editor.modification_buffer = Some((*x, *y));
                }
            }
            None
        }
        GameAction::EditorSave => {
            editor.save_requested = true;
            None
        }
        GameAction::UpdateVolume(value) => {
            state.settings.master_volume = *value;
            editor.engine.audio_manager.set_volume(*value);
            state.persist_settings();
            None
        }
        GameAction::Hit { column } => {
            editor
                .engine
                .handle_input(GameAction::Hit { column: *column });
            None
        }
        GameAction::Release { column } => {
            editor
                .engine
                .handle_input(GameAction::Release { column: *column });
            None
        }
        _ => None,
    }
}
