use crate::database::connection::Database;
use crate::database::models::{Beatmap, Beatmapset};
use crate::database::query::{clear_all, get_all_beatmapsets};
use crate::database::scanner::scan_songs_directory;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum DbStatus {
    Idle,
    Initializing,
    Loading,
    Scanning { current: usize, total: usize },
    Error(String),
}

#[derive(Debug, Clone)]
pub struct DbState {
    pub status: DbStatus,
    pub beatmapsets: Vec<(Beatmapset, Vec<Beatmap>)>,
    pub error: Option<String>,
}

impl DbState {
    pub fn new() -> Self {
        Self {
            status: DbStatus::Idle,
            beatmapsets: Vec::new(),
            error: None,
        }
    }
}

#[derive(Debug)]
pub enum DbCommand {
    Init,
    Load,
    Rescan,
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
}
