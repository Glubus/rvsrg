use crate::database::models::{Beatmap, BeatmapRating, BeatmapWithRatings, Beatmapset};
use crate::models::engine::NoteData;
use crate::models::stats::{HitStats, Judgement};
use crate::shared::snapshot::{GameplaySnapshot, RenderState};
use crate::state::{GameResultData, MenuState};
use crate::views::components::editor::layout::EditorScene;
use std::time::Instant;

/// Génère un état de rendu factice basé sur la scène sélectionnée dans l'éditeur.
pub fn create_mock_state(scene: EditorScene) -> RenderState {
    match scene {
        EditorScene::Gameplay4K => create_mock_gameplay(4),
        EditorScene::Gameplay7K => create_mock_gameplay(7),
        EditorScene::SongSelect => create_mock_menu(),
        EditorScene::ResultScreen => create_mock_result(),
    }
}

fn create_mock_gameplay(key_count: usize) -> RenderState {
    let mut notes = Vec::new();
    let time_base = 2000.0;

    // Pattern en escalier pour visualiser les colonnes
    for i in 0..8 {
        let col = i % key_count;
        let time = time_base + (i as f64 * 200.0);
        notes.push(NoteData::tap(time, col));
    }

    // Un Hold (Note longue)
    if key_count > 0 {
        notes.push(NoteData::hold(time_base + 2000.0, 0, 500.0));
    }

    // Une Mine
    if key_count > 1 {
        notes.push(NoteData::mine(time_base + 2200.0, 1));
    }

    // Un Burst
    if key_count > 2 {
        notes.push(NoteData::burst(time_base + 2500.0, 2, 200.0, 4));
    }

    RenderState::InGame(GameplaySnapshot {
        audio_time: time_base + 500.0, // On se place un peu après le début pour voir les notes arriver
        timestamp: Instant::now(),
        rate: 1.0,
        scroll_speed: 650.0,
        visible_notes: notes,
        keys_held: vec![false; key_count], // Aucune touche pressée
        score: 125000,
        accuracy: 98.45,
        combo: 124,
        hit_stats: HitStats {
            marv: 100,
            perfect: 20,
            great: 4,
            good: 0,
            bad: 0,
            miss: 0,
            ghost_tap: 0,
        },
        remaining_notes: 50,
        last_hit_judgement: Some(Judgement::Marv), // Affiche un jugement pour tester la position
        last_hit_timing: Some(-4.5),
        nps: 12.5,
        practice_mode: false,
        checkpoints: vec![],
        map_duration: 120000.0,
    })
}

fn create_mock_menu() -> RenderState {
    let mut state = MenuState::new();

    // Création d'un set de beatmaps factice
    let set1 = Beatmapset {
        id: 1,
        path: String::from("mock_path_1"),
        image_path: None, // Le renderer utilisera le background par défaut
        artist: Some(String::from("Camellia")),
        title: Some(String::from("Ghost")),
    };

    let bm1 = Beatmap {
        hash: String::from("hash1"),
        beatmapset_id: 1,
        path: String::from("path1"),
        difficulty_name: Some(String::from("Expert")),
        note_count: 1540,
        duration_ms: 180000,
        nps: 15.4,
    };

    let ratings = vec![BeatmapRating {
        id: 1,
        beatmap_hash: String::from("hash1"),
        name: String::from("etterna"),
        overall: 24.5,
        stream: 22.0,
        jumpstream: 24.0,
        handstream: 20.0,
        stamina: 23.0,
        jackspeed: 15.0,
        chordjack: 18.0,
        technical: 12.0,
    }];

    std::sync::Arc::make_mut(&mut state.beatmapsets)
        .push((set1, vec![BeatmapWithRatings::new(bm1, ratings)]));
    state.selected_index = 0;

    RenderState::Menu(state)
}

fn create_mock_result() -> RenderState {
    RenderState::Result(GameResultData {
        hit_stats: HitStats {
            marv: 850,
            perfect: 120,
            great: 15,
            good: 2,
            bad: 0,
            miss: 1,
            ghost_tap: 5,
        },
        replay_data: crate::models::replay::ReplayData::empty(),
        replay_result: crate::models::replay::ReplayResult::new(), // Vide pour l'instant (graphes vides)
        score: 985420,
        accuracy: 99.12,
        max_combo: 850,
        beatmap_hash: Some(String::from("mock_hash")),
        rate: 1.1,
        judge_text: String::from("OD 8.5"),
        show_settings: false,
    })
}
