use crate::models::engine::NoteData;
use crate::models::stats::{HitStats, Judgement};
use crate::models::menu::MenuState; // On aura besoin de cloner le state pour l'affichage

#[derive(Clone, Debug)]
pub enum RenderState {
    /// Rien à afficher (initialisation)
    Empty,
    /// Mode Menu : On envoie une copie de l'état du menu nécessaire au rendu
    Menu(MenuState),
    /// Mode Jeu : On envoie le snapshot gameplay qu'on a créé à l'étape 2
    InGame(GameplaySnapshot),
    /// Mode Résultat (à faire plus tard, on peut utiliser Menu pour l'instant)
    Result(crate::models::menu::GameResultData),
}

/// Une "photo" de l'état du jeu à un instant T (Déjà fait à l'étape 2)
#[derive(Clone, Debug)]
pub struct GameplaySnapshot {
    pub audio_time: f64,
    pub rate: f64,
    pub scroll_speed: f64,
    pub visible_notes: Vec<NoteData>,
    pub keys_held: Vec<bool>,
    pub score: u32,
    pub accuracy: f64,
    pub combo: u32,
    pub hit_stats: HitStats,
    pub remaining_notes: usize,
    pub last_hit_judgement: Option<Judgement>,
    pub last_hit_timing: Option<f64>,
}