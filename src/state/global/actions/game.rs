use crate::input::events::GameAction;
use crate::models::engine::hit_window::HitWindow;
use crate::models::settings::HitWindowMode;
use crate::state::GameEngine;
use crate::state::global::GlobalState;
use crate::state::global::app_state::AppState;

pub fn apply(
    state: &mut GlobalState,
    engine: &mut GameEngine,
    action: &GameAction,
) -> Option<AppState> {
    match action {
        GameAction::Back => {
            engine.audio_manager.stop();
            state.requested_leaderboard_hash = None;
            let menu = state.saved_menu_state.clone();
            let request_hash = menu.get_selected_beatmap_hash();
            state.request_leaderboard_for_hash(request_hash);
            Some(AppState::Menu(menu))
        }
        GameAction::UpdateVolume(value) => {
            state.settings.master_volume = *value;
            engine.audio_manager.set_volume(*value);
            state.persist_settings();
            None
        }
        GameAction::ReloadKeybinds => None,
        GameAction::UpdateHitWindow { mode, value } => {
            state.settings.hit_window_mode = *mode;
            state.settings.hit_window_value = *value;
            state.persist_settings();

            let hw = match mode {
                HitWindowMode::OsuOD => HitWindow::from_osu_od(*value),
                HitWindowMode::EtternaJudge => HitWindow::from_etterna_judge(*value as u8),
            };

            engine.hit_window = hw;
            engine.replay_data.hit_window_mode = *mode;
            engine.replay_data.hit_window_value = *value;

            None
        }
        GameAction::ScrollSpeedUp => {
            engine.scroll_speed_ms = (engine.scroll_speed_ms + 10.0).min(1500.0);
            state.settings.scroll_speed = engine.scroll_speed_ms;
            state.persist_settings();
            None
        }
        GameAction::ScrollSpeedDown => {
            engine.scroll_speed_ms = (engine.scroll_speed_ms - 10.0).max(100.0);
            state.settings.scroll_speed = engine.scroll_speed_ms;
            state.persist_settings();
            None
        }
        _ => {
            engine.handle_input(action.clone());
            None
        }
    }
}
