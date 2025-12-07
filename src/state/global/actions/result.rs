use crate::input::events::GameAction;
use crate::models::replay::simulate_replay;
use crate::models::settings::HitWindowMode;
use crate::state::GameResultData;
use crate::state::global::GlobalState;
use crate::state::global::app_state::AppState;

pub fn apply(
    state: &mut GlobalState,
    result: &mut GameResultData,
    action: &GameAction,
) -> Option<AppState> {
    match action {
        GameAction::Back | GameAction::Confirm => {
            state.requested_leaderboard_hash = None;
            let menu = state.saved_menu_state.clone();
            let request_hash = menu.get_selected_beatmap_hash();
            state.request_leaderboard_for_hash(request_hash);
            Some(AppState::Menu(menu))
        }
        GameAction::ToggleSettings => {
            result.show_settings = !result.show_settings;
            None
        }
        GameAction::UpdateHitWindow { mode, value } => {
            state.settings.hit_window_mode = *mode;
            state.settings.hit_window_value = *value;
            state.persist_settings();

            result.replay_data.hit_window_mode = *mode;
            result.replay_data.hit_window_value = *value;
            result.judge_text = match state.settings.hit_window_mode {
                HitWindowMode::OsuOD => format!("OD {:.1}", state.settings.hit_window_value),
                HitWindowMode::EtternaJudge => {
                    format!("Judge {:.0}", state.settings.hit_window_value)
                }
            };

            let chart_opt = state
                .saved_menu_state
                .get_cached_chart()
                .map(|c| c.chart.iter().map(|n| n.reset()).collect::<Vec<_>>());

            if let Some(chart) = chart_opt {
                log::info!(
                    "RESULT: Re-judging replay with {} notes (Mode: {:?}, Value: {})",
                    chart.len(),
                    *mode,
                    *value
                );
                let hit_window = result.replay_data.build_hit_window();
                let sim_res = simulate_replay(&result.replay_data, &chart, &hit_window);

                log::info!(
                    "RESULT: New Accuracy: {:.2}% (Marv: {}, Perf: {}, Miss: {})",
                    sim_res.accuracy,
                    sim_res.hit_stats.marv,
                    sim_res.hit_stats.perfect,
                    sim_res.hit_stats.miss
                );

                result.hit_stats = sim_res.hit_stats.clone();
                result.replay_result = sim_res.clone();
                result.score = sim_res.score;
                result.accuracy = sim_res.accuracy;
                result.max_combo = sim_res.max_combo;
            } else {
                log::warn!("RESULT: Cannot re-judge, chart not in cache!");
            }

            None
        }
        _ => None,
    }
}
