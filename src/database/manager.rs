use crate::database::connection::Database;
use crate::database::models::{BeatmapWithRatings, Beatmapset, Replay};
use crate::database::query::{clear_all, get_all_beatmapsets};
use crate::database::scanner::scan_songs_directory;
use crate::models::search::MenuSearchFilters;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum DbStatus {
    Idle,
    Initializing,
    Loading,
    Searching,
    Scanning { current: usize, total: usize },
    Error(String),
}

#[derive(Debug, Clone)]
pub struct DbState {
    pub status: DbStatus,
    pub beatmapsets: Vec<(Beatmapset, Vec<BeatmapWithRatings>)>,
    pub error: Option<String>,
    pub version: u64,
    pub leaderboard: Vec<Replay>,
    pub leaderboard_hash: Option<String>,
    pub leaderboard_version: u64,
}

impl DbState {
    pub fn new() -> Self {
        Self {
            status: DbStatus::Idle,
            beatmapsets: Vec::new(),
            error: None,
            version: 0,
            leaderboard: Vec::new(),
            leaderboard_hash: None,
            leaderboard_version: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SaveReplayCommand {
    pub beatmap_hash: String,
    pub timestamp: i64,
    pub score: i32,
    pub accuracy: f64,
    pub max_combo: i32,
    pub rate: f64,
    pub data: String,
}

#[derive(Debug)]
pub enum DbCommand {
    Init,
    Load,
    Rescan,
    Search(MenuSearchFilters),
    SaveReplay(SaveReplayCommand),
    FetchLeaderboard(String),
    Shutdown,
}

pub struct DbManager {
    state: Arc<Mutex<DbState>>,
    command_sender: std::sync::mpsc::Sender<DbCommand>,
    _handle: thread::JoinHandle<()>,
}

impl DbManager {
    pub fn new(db_path: PathBuf, songs_path: PathBuf) -> Self {
        let state = Arc::new(Mutex::new(DbState::new()));
        let (tx, rx) = std::sync::mpsc::channel();

        let state_clone = Arc::clone(&state);
        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(Self::db_thread(state_clone, rx, db_path, songs_path));
        });

        Self {
            state,
            command_sender: tx,
            _handle: handle,
        }
    }

    async fn db_thread(
        state: Arc<Mutex<DbState>>,
        rx: std::sync::mpsc::Receiver<DbCommand>,
        db_path: PathBuf,
        songs_path: PathBuf,
    ) {
        let mut db: Option<Database> = None;

        loop {
            // Vérifier les commandes (non-bloquant)
            match rx.try_recv() {
                Ok(DbCommand::Init) => {
                    {
                        let mut s = state.lock().unwrap();
                        s.status = DbStatus::Initializing;
                        s.error = None;
                    }

                    match Database::new(&db_path).await {
                        Ok(d) => {
                            db = Some(d);
                            {
                                let mut s = state.lock().unwrap();
                                s.status = DbStatus::Idle;
                            }

                            // Si la DB existe, charger les maps
                            if db_path.exists() {
                                Self::load_maps(&state, db.as_ref().unwrap()).await;
                            }
                        }
                        Err(e) => {
                            let mut s = state.lock().unwrap();
                            s.status = DbStatus::Error(format!("Initialization error: {}", e));
                            s.error = Some(format!("{}", e));
                        }
                    }
                }
                Ok(DbCommand::Load) => {
                    if let Some(ref d) = db {
                        Self::load_maps(&state, d).await;
                    }
                }
                Ok(DbCommand::Rescan) => {
                    if let Some(ref d) = db {
                        Self::rescan_maps(&state, d, &songs_path).await;
                    }
                }
                Ok(DbCommand::Search(filters)) => {
                    if let Some(ref d) = db {
                        Self::search_maps(&state, d, filters).await;
                    }
                }
                Ok(DbCommand::SaveReplay(payload)) => {
                    if let Some(ref d) = db {
                        Self::persist_replay(&state, d, payload).await;
                    }
                }
                Ok(DbCommand::FetchLeaderboard(hash)) => {
                    if let Some(ref d) = db {
                        Self::load_leaderboard(&state, d, &hash).await;
                    }
                }
                Ok(DbCommand::Shutdown) => {
                    break;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // Pas de commande, continuer
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    // Le sender est fermé, on peut continuer ou sortir
                    break;
                }
            }

            // Petit sleep pour éviter de consommer 100% CPU
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    async fn load_maps(state: &Arc<Mutex<DbState>>, db: &Database) {
        {
            let mut s = state.lock().unwrap();
            s.status = DbStatus::Loading;
            s.error = None;
        }

        match get_all_beatmapsets(db.pool()).await {
            Ok(beatmapsets) => {
                let mut s = state.lock().unwrap();
                s.beatmapsets = beatmapsets;
                s.status = DbStatus::Idle;
                s.error = None;
                s.version = s.version.wrapping_add(1);
                s.leaderboard.clear();
                s.leaderboard_hash = None;
                s.leaderboard_version = s.leaderboard_version.wrapping_add(1);
            }
            Err(e) => {
                let mut s = state.lock().unwrap();
                s.status = DbStatus::Error(format!("Loading error: {}", e));
                s.error = Some(format!("{}", e));
            }
        }
    }

    async fn rescan_maps(state: &Arc<Mutex<DbState>>, db: &Database, songs_path: &Path) {
        {
            let mut s = state.lock().unwrap();
            s.status = DbStatus::Scanning {
                current: 0,
                total: 0,
            };
            s.error = None;
        }

        // Vider la DB
        if let Err(e) = clear_all(db.pool()).await {
            let mut s = state.lock().unwrap();
            s.status = DbStatus::Error(format!("Error clearing database: {}", e));
            s.error = Some(format!("{}", e));
            return;
        }

        // Scanner (pour l'instant on ne peut pas suivre la progression facilement)
        {
            let mut s = state.lock().unwrap();
            s.status = DbStatus::Scanning {
                current: 0,
                total: 1,
            };
        }

        if let Err(e) = scan_songs_directory(db, songs_path).await {
            let mut s = state.lock().unwrap();
            s.status = DbStatus::Error(format!("Scan error: {}", e));
            s.error = Some(format!("{}", e));
            return;
        }

        // Recharger les maps
        Self::load_maps(state, db).await;
    }

    async fn search_maps(state: &Arc<Mutex<DbState>>, db: &Database, filters: MenuSearchFilters) {
        {
            let mut s = state.lock().unwrap();
            s.status = DbStatus::Searching;
            s.error = None;
        }

        match db.search_beatmapsets(&filters).await {
            Ok(beatmapsets) => {
                let mut s = state.lock().unwrap();
                s.beatmapsets = beatmapsets;
                s.status = DbStatus::Idle;
                s.error = None;
                s.version = s.version.wrapping_add(1);
                s.leaderboard.clear();
                s.leaderboard_hash = None;
                s.leaderboard_version = s.leaderboard_version.wrapping_add(1);
            }
            Err(e) => {
                let mut s = state.lock().unwrap();
                s.status = DbStatus::Error(format!("Search error: {}", e));
                s.error = Some(format!("{}", e));
            }
        }
    }

    async fn persist_replay(
        state: &Arc<Mutex<DbState>>,
        db: &Database,
        payload: SaveReplayCommand,
    ) {
        match db
            .insert_replay(
                &payload.beatmap_hash,
                payload.timestamp,
                payload.score,
                payload.accuracy,
                payload.max_combo,
                payload.rate,
                &payload.data,
            )
            .await
        {
            Ok(_) => {
                Self::load_leaderboard(state, db, &payload.beatmap_hash).await;
            }
            Err(e) => {
                log::error!(
                    "DB: failed to insert replay for {}: {}",
                    payload.beatmap_hash,
                    e
                );
            }
        }
    }

    async fn load_leaderboard(state: &Arc<Mutex<DbState>>, db: &Database, beatmap_hash: &str) {
        match db.get_replays_for_beatmap(beatmap_hash).await {
            Ok(replays) => {
                let mut s = state.lock().unwrap();
                s.leaderboard = replays;
                s.leaderboard_hash = Some(beatmap_hash.to_string());
                s.leaderboard_version = s.leaderboard_version.wrapping_add(1);
            }
            Err(e) => {
                log::error!("DB: failed to load leaderboard for {}: {}", beatmap_hash, e);
            }
        }
    }

    pub fn get_state(&self) -> Arc<Mutex<DbState>> {
        Arc::clone(&self.state)
    }

    pub fn send_command(
        &self,
        cmd: DbCommand,
    ) -> Result<(), std::sync::mpsc::SendError<DbCommand>> {
        self.command_sender.send(cmd)
    }

    pub fn init(&self) {
        let _ = self.send_command(DbCommand::Init);
    }

    pub fn load(&self) {
        let _ = self.send_command(DbCommand::Load);
    }

    pub fn rescan(&self) {
        let _ = self.send_command(DbCommand::Rescan);
    }

    pub fn search(&self, filters: MenuSearchFilters) {
        let _ = self.send_command(DbCommand::Search(filters));
    }

    pub fn save_replay(&self, payload: SaveReplayCommand) {
        let _ = self.send_command(DbCommand::SaveReplay(payload));
    }

    pub fn fetch_leaderboard(&self, beatmap_hash: &str) {
        let _ = self.send_command(DbCommand::FetchLeaderboard(beatmap_hash.to_string()));
    }
}
