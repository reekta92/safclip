use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use std::sync::mpsc::{channel, Sender, Receiver};
use parking_lot::RwLock;
use ::mpris::PlayerFinder;


pub mod discovery;
pub mod mpris;

#[derive(Debug, Clone, Default)]
pub struct PlayerState {
    pub position: f64,
    pub duration: f64,
    pub is_paused: bool,
    pub track_title: Option<String>,
    pub source_path: Option<String>,
    pub identity: String,
}

pub trait PlayerController {
    fn play(&mut self) -> Result<(), String>;
    fn pause(&mut self) -> Result<(), String>;
    fn toggle_play(&mut self) -> Result<(), String>;
    fn seek(&mut self, offset: f64) -> Result<(), String>;      // relative seconds
    fn seek_absolute(&mut self, position: f64) -> Result<(), String>;
    fn position(&self) -> f64;
    fn duration(&self) -> f64;
    fn is_paused(&self) -> bool;
    fn track_title(&self) -> Option<String>;
    fn source_path(&self) -> Option<String>;  // best-effort file path from metadata
    fn identity(&self) -> String;             // returns unique identifier/name of player
}

enum PlayerCommand {
    Play,
    Pause,
    TogglePlay,
    Seek(f64),
    SeekAbsolute(f64),
}

pub struct BackgroundPlayer {
    state: Arc<RwLock<PlayerState>>,
    command_tx: Sender<PlayerCommand>,
    _handle: thread::JoinHandle<()>,
}

impl BackgroundPlayer {
    pub fn new(identity: String) -> Self {
        let state = Arc::new(RwLock::new(PlayerState {
            identity: identity.clone(),
            ..Default::default()
        }));

        let (command_tx, command_rx): (Sender<PlayerCommand>, Receiver<PlayerCommand>) = channel();
        let state_clone = Arc::clone(&state);
        let identity_clone = identity.clone();

        let _handle = thread::spawn(move || {
            let mut player_opt: Option<Box<dyn PlayerController>> = None;
            let mut last_metadata_refresh = Instant::now();

            loop {
                // 1. Ensure we have a player
                if player_opt.is_none() {
                    if let Ok(finder) = PlayerFinder::new() {
                        if let Ok(players) = finder.find_all() {
                            for p in players {
                                if p.identity() == identity_clone {
                                    let mpris_player = crate::player::mpris::MprisPlayer::new(p);
                                    // Initial poll
                                    let mut s = state_clone.write();
                                    s.position = mpris_player.position();
                                    s.duration = mpris_player.duration();
                                    s.is_paused = mpris_player.is_paused();
                                    s.track_title = mpris_player.track_title();
                                    s.source_path = mpris_player.source_path();
                                    s.identity = mpris_player.identity();
                                    player_opt = Some(Box::new(mpris_player));
                                    break;
                                }
                            }
                        }
                    }
                }

                if let Some(player) = player_opt.as_mut() {
                    // 2. Handle commands
                    while let Ok(cmd) = command_rx.try_recv() {
                        let _ = match cmd {
                            PlayerCommand::Play => player.play(),
                            PlayerCommand::Pause => player.pause(),
                            PlayerCommand::TogglePlay => player.toggle_play(),
                            PlayerCommand::Seek(offset) => player.seek(offset),
                            PlayerCommand::SeekAbsolute(pos) => player.seek_absolute(pos),
                        };
                    }

                    // 3. Poll state
                    let position = player.position();
                    let is_paused = player.is_paused();
                    
                    // Periodically refresh metadata (every 5s)
                    let now = Instant::now();
                    if now.duration_since(last_metadata_refresh) > Duration::from_secs(5) {
                        let duration = player.duration();
                        let track_title = player.track_title();
                        let source_path = player.source_path();
                        let identity = player.identity();
                        
                        let mut s = state_clone.write();
                        s.position = position;
                        s.is_paused = is_paused;
                        s.duration = duration;
                        s.track_title = track_title;
                        s.source_path = source_path;
                        s.identity = identity;
                        last_metadata_refresh = now;
                    } else {
                        let mut s = state_clone.write();
                        s.position = position;
                        s.is_paused = is_paused;
                    }
                }

                thread::sleep(Duration::from_millis(100));
            }
        });

        Self {
            state,
            command_tx,
            _handle,
        }
    }
}

impl PlayerController for BackgroundPlayer {
    fn play(&mut self) -> Result<(), String> {
        self.command_tx.send(PlayerCommand::Play).map_err(|e| e.to_string())
    }

    fn pause(&mut self) -> Result<(), String> {
        self.command_tx.send(PlayerCommand::Pause).map_err(|e| e.to_string())
    }

    fn toggle_play(&mut self) -> Result<(), String> {
        self.command_tx.send(PlayerCommand::TogglePlay).map_err(|e| e.to_string())
    }

    fn seek(&mut self, offset: f64) -> Result<(), String> {
        self.command_tx.send(PlayerCommand::Seek(offset)).map_err(|e| e.to_string())
    }

    fn seek_absolute(&mut self, position: f64) -> Result<(), String> {
        self.command_tx.send(PlayerCommand::SeekAbsolute(position)).map_err(|e| e.to_string())
    }

    fn position(&self) -> f64 {
        self.state.read().position
    }

    fn duration(&self) -> f64 {
        self.state.read().duration
    }

    fn is_paused(&self) -> bool {
        self.state.read().is_paused
    }

    fn track_title(&self) -> Option<String> {
        self.state.read().track_title.clone()
    }

    fn source_path(&self) -> Option<String> {
        self.state.read().source_path.clone()
    }

    fn identity(&self) -> String {
        self.state.read().identity.clone()
    }
}
