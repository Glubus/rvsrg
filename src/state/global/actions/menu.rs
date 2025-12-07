use crate::input::events::GameAction;
use crate::state::global::GlobalState;
use crate::state::global::app_state::AppState;
use crate::state::global::helpers::create_debug_chart;
use crate::state::{GameEngine, MenuState};

pub fn apply(
    state: &mut GlobalState,
    menu: &mut MenuState,
    action: &GameAction,
) -> Option<AppState> {
    match action {
        GameAction::Navigation { x, y } => handle_navigation(state, menu, *x, *y),
        GameAction::SetSelection(idx) => handle_set_selection(state, menu, *idx),
        GameAction::SetDifficulty(idx) => handle_set_difficulty(state, menu, *idx),
        GameAction::Confirm => handle_confirm(state, menu),
        GameAction::LaunchPractice => handle_launch_practice(state, menu),
        GameAction::ToggleEditor => handle_toggle_editor(state, menu),
        GameAction::TabNext => {
            menu.increase_rate();
            None
        }
        GameAction::TabPrev => {
            menu.decrease_rate();
            None
        }
        GameAction::ToggleSettings => {
            menu.show_settings = !menu.show_settings;
            if menu.show_settings {
                menu.ensure_chart_cache();
            }
            None
        }
        GameAction::UpdateVolume(value) => {
            state.settings.master_volume = *value;
            state.persist_settings();
            None
        }
        GameAction::Rescan => {
            state.db_manager.rescan();
            state.last_db_version = u64::MAX;
            None
        }
        GameAction::ApplySearch(filters) => {
            menu.search_filters = filters.clone();
            state.db_manager.search(filters.clone());
            state.requested_leaderboard_hash = None;
            state.last_leaderboard_version = 0;
            None
        }
        GameAction::SetCalculator(calc_id) => {
            menu.set_calculator(calc_id);
            menu.ensure_difficulty_calculated();
            None
        }
        GameAction::UpdateHitWindow { mode, value } => {
            state.settings.hit_window_mode = *mode;
            state.settings.hit_window_value = *value;
            state.persist_settings();
            None
        }
        GameAction::SetResult(result_data) => Some(AppState::Result(result_data.clone())),
        GameAction::LaunchDebugMap => handle_launch_debug_map(state),
        _ => None,
    }
}

fn handle_navigation(
    state: &mut GlobalState,
    menu: &mut MenuState,
    x: i32,
    y: i32,
) -> Option<AppState> {
    if y < 0 {
        menu.move_up();
    }
    if y > 0 {
        menu.move_down();
    }
    if x < 0 {
        menu.previous_difficulty();
    }
    if x > 0 {
        menu.next_difficulty();
    }
    if menu.show_settings {
        menu.ensure_chart_cache();
    }
    let request_hash = menu.get_selected_beatmap_hash();
    state.request_leaderboard_for_hash(request_hash);
    None
}

fn handle_set_selection(
    state: &mut GlobalState,
    menu: &mut MenuState,
    idx: usize,
) -> Option<AppState> {
    if idx < menu.beatmapsets.len() {
        menu.selected_index = idx;
        menu.selected_difficulty_index = 0;
        if idx < menu.start_index {
            menu.start_index = idx;
            menu.end_index = (menu.start_index + menu.visible_count).min(menu.beatmapsets.len());
        } else if idx >= menu.end_index {
            menu.end_index = (idx + 1).min(menu.beatmapsets.len());
            menu.start_index = menu.end_index.saturating_sub(menu.visible_count);
        }
    }
    if menu.show_settings {
        menu.ensure_chart_cache();
    }
    let request_hash = menu.get_selected_beatmap_hash();
    state.request_leaderboard_for_hash(request_hash);
    None
}

fn handle_set_difficulty(
    state: &mut GlobalState,
    menu: &mut MenuState,
    idx: usize,
) -> Option<AppState> {
    menu.selected_difficulty_index = idx;
    if menu.show_settings {
        menu.ensure_chart_cache();
    }
    let request_hash = menu.get_selected_beatmap_hash();
    state.request_leaderboard_for_hash(request_hash);
    None
}

fn handle_confirm(state: &mut GlobalState, menu: &mut MenuState) -> Option<AppState> {
    state.reload_settings();
    menu.ensure_chart_cache();

    let engine = if let Some(cache) = menu.get_cached_chart() {
        let chart: Vec<_> = cache.chart.iter().map(|n| n.reset()).collect();
        let beatmap_hash = Some(cache.beatmap_hash.clone());

        log::info!(
            "GAME: Using cached chart ({} notes, hash: {:?})",
            chart.len(),
            beatmap_hash
        );
        GameEngine::from_cached(
            &state.bus,
            chart,
            cache.audio_path.clone(),
            menu.rate,
            beatmap_hash,
            state.settings.hit_window_mode,
            state.settings.hit_window_value,
        )
    } else if let Some(path) = menu.get_selected_beatmap_path() {
        let beatmap_hash = menu.get_selected_beatmap_hash();
        log::info!(
            "GAME: Loading chart from file (no cache), hash: {:?}",
            beatmap_hash
        );
        if let Some(e) = GameEngine::new(
            &state.bus,
            path,
            menu.rate,
            beatmap_hash,
            state.settings.hit_window_mode,
            state.settings.hit_window_value,
        ) {
            e
        } else {
            return None;
        }
    } else {
        return None;
    };

    let mut engine = engine;
    engine.scroll_speed_ms = state.settings.scroll_speed;
    engine
        .audio_manager
        .set_volume(state.settings.master_volume);
    Some(AppState::Game(engine))
}

fn handle_launch_practice(state: &mut GlobalState, menu: &mut MenuState) -> Option<AppState> {
    state.reload_settings();
    menu.ensure_chart_cache();

    let engine = if let Some(cache) = menu.get_cached_chart() {
        let chart: Vec<_> = cache.chart.iter().map(|n| n.reset()).collect();
        let beatmap_hash = Some(cache.beatmap_hash.clone());

        log::info!(
            "PRACTICE: Using cached chart ({} notes, hash: {:?})",
            chart.len(),
            beatmap_hash
        );
        GameEngine::from_cached(
            &state.bus,
            chart,
            cache.audio_path.clone(),
            menu.rate,
            beatmap_hash,
            state.settings.hit_window_mode,
            state.settings.hit_window_value,
        )
    } else if let Some(path) = menu.get_selected_beatmap_path() {
        let beatmap_hash = menu.get_selected_beatmap_hash();
        log::info!(
            "PRACTICE: Loading chart from file (no cache), hash: {:?}",
            beatmap_hash
        );
        if let Some(e) = GameEngine::new(
            &state.bus,
            path,
            menu.rate,
            beatmap_hash,
            state.settings.hit_window_mode,
            state.settings.hit_window_value,
        ) {
            e
        } else {
            return None;
        }
    } else {
        return None;
    };

    let mut engine = engine;
    engine.scroll_speed_ms = state.settings.scroll_speed;
    engine
        .audio_manager
        .set_volume(state.settings.master_volume);
    engine.enable_practice_mode();
    Some(AppState::Game(engine))
}

fn handle_toggle_editor(state: &mut GlobalState, menu: &mut MenuState) -> Option<AppState> {
    use crate::state::editor::EditorState;

    state.reload_settings();
    menu.ensure_chart_cache();

    let engine = if let Some(cache) = menu.get_cached_chart() {
        let chart: Vec<_> = cache.chart.iter().map(|n| n.reset()).collect();
        GameEngine::from_cached(
            &state.bus,
            chart,
            cache.audio_path.clone(),
            1.0,
            None,
            state.settings.hit_window_mode,
            state.settings.hit_window_value,
        )
    } else if let Some(path) = menu.get_selected_beatmap_path() {
        if let Some(e) = GameEngine::new(
            &state.bus,
            path,
            1.0,
            None,
            state.settings.hit_window_mode,
            state.settings.hit_window_value,
        ) {
            e
        } else {
            return None;
        }
    } else {
        return None;
    };

    let mut engine = engine;
    engine.scroll_speed_ms = state.settings.scroll_speed;
    engine
        .audio_manager
        .set_volume(state.settings.master_volume);

    Some(AppState::Editor(EditorState::new(engine)))
}

fn handle_launch_debug_map(state: &mut GlobalState) -> Option<AppState> {
    state.reload_settings();
    let chart = create_debug_chart();
    let engine = GameEngine::from_debug_chart(
        &state.bus,
        chart,
        state.settings.hit_window_mode,
        state.settings.hit_window_value,
    );
    let mut engine = engine;
    engine.scroll_speed_ms = state.settings.scroll_speed;
    Some(AppState::Game(engine))
}
