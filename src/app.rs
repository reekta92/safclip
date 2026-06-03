use std::path::PathBuf;
use std::sync::mpsc;
use crate::model::*;
use crate::input::AppAction;
use crate::player::mpris::MprisPlayer;
use crate::player::PlayerController;
use crate::timeline::TimelineState;
use crossterm::event::MouseButton;

#[derive(Debug)]
pub enum ExportMsg {
    Progress(String),
    Done(Vec<String>),
    MergedDone(String),
    Failed(String),
}

pub struct AppState {
    pub source_path: Option<PathBuf>,
    pub cli_source_path: Option<PathBuf>,
    pub metadata: Option<MediaMetadata>,
    pub segments: Vec<Segment>,
    pub selected_segment: Option<usize>,
    pub current_time: f64,
    pub timeline_state: TimelineState,
    pub mode: AppMode,
    pub undo_stack: Vec<AppStateSnapshot>,
    pub redo_stack: Vec<AppStateSnapshot>,
    pub status_message: Option<String>,
    pub pending_in_point: Option<f64>,
    pub should_quit: bool,
    
    pub export_receiver: Option<mpsc::Receiver<ExportMsg>>,
    pub is_exporting: bool,
    
    pub probe_receiver: Option<mpsc::Receiver<Result<MediaMetadata, String>>>,
    pub is_probing: bool,

    pub available_players: Vec<MprisPlayer>,
    pub active_player_index: Option<usize>,
    pub player_playing: bool,
    
    pub terminal_size: (u16, u16),
    pub timeline_rect: (u16, u16, u16, u16), // x, y, width, height
    pub segments_rect: (u16, u16, u16, u16), // x, y, width, height
    // Mouse drag scrubbing states
    pub is_dragging_timeline: bool,
    pub is_panning_timeline: bool,
    pub drag_last_col: u16,
    pub drag_was_playing: bool,

    pub label_input: String, // to store editing label text
    pub poll_count: usize,
    pub pending_session: Option<crate::session::SessionData>,
    pub last_seek_time: std::time::Instant,
    pub last_click_time: std::time::Instant,
    pub last_click_pos: (u16, u16),
}

impl AppState {
    pub fn new() -> Self {
        Self {
            source_path: None,
            cli_source_path: None,
            metadata: None,
            segments: Vec::new(),
            selected_segment: None,
            current_time: 0.0,
            timeline_state: TimelineState::new(0.0),
            mode: AppMode::Normal,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            status_message: Some("Welcome to SafClip. Press '?' for help.".to_string()),
            pending_in_point: None,
            should_quit: false,
            export_receiver: None,
            is_exporting: false,
            probe_receiver: None,
            is_probing: false,
            available_players: Vec::new(),
            active_player_index: None,
            player_playing: false,
            terminal_size: (0, 0),
            timeline_rect: (0, 0, 0, 0),
            segments_rect: (0, 0, 0, 0),
            is_dragging_timeline: false,
            is_panning_timeline: false,
            drag_last_col: 0,
            drag_was_playing: false,
            poll_count: 0,
            label_input: String::new(),
            pending_session: None,
            last_seek_time: std::time::Instant::now(),
            last_click_time: std::time::Instant::now(),
            last_click_pos: (0, 0),
        }
    }

    pub fn set_terminal_size(&mut self, w: u16, h: u16) {
        self.terminal_size = (w, h);
    }

    pub fn active_player(&self) -> Option<&MprisPlayer> {
        self.active_player_index.and_then(|idx| self.available_players.get(idx))
    }

    pub fn active_player_mut(&mut self) -> Option<&mut MprisPlayer> {
        self.active_player_index.and_then(|idx| self.available_players.get_mut(idx))
    }

    pub fn refresh_players(&mut self) {
        if let Ok(discovery) = crate::player::discovery::PlayerDiscovery::new() {
            if let Ok(players) = discovery.list_players() {
                // Keep the active player if it still exists
                let mut still_valid = false;
                let mut new_index = None;
                if let Some(idx) = self.active_player_index {
                    if idx < self.available_players.len() {
                        let active_identity = self.available_players[idx].identity();
                        for (i, p) in players.iter().enumerate() {
                            if p.identity() == active_identity {
                                still_valid = true;
                                new_index = Some(i);
                                break;
                            }
                        }
                    }
                }

                self.available_players = players;
                if still_valid {
                    self.active_player_index = new_index;
                } else if !self.available_players.is_empty() {
                    self.select_player(0);
                } else {
                    self.active_player_index = None;
                }
            }
        }
    }

    pub fn select_player(&mut self, index: usize) {
        if index < self.available_players.len() {
            self.active_player_index = Some(index);
            self.metadata = None;
            self.timeline_state = TimelineState::new(0.0);
            
            // Check if player already has a source URL
            if let Some(player) = self.active_player() {
                if let Some(path) = player.source_path() {
                    self.start_probing(&path);
                } else if let Some(cli_path) = &self.cli_source_path {
                    // Fall back to CLI path if no URL reported by player
                    let path_str = cli_path.to_string_lossy().to_string();
                    self.start_probing(&path_str);
                }
            }
            self.status_message = Some(format!("Switched to player: {}", 
                self.active_player().map(|p| p.identity()).unwrap_or_default()
            ));
        }
    }

    pub fn cycle_player(&mut self) {
        if self.available_players.is_empty() {
            self.refresh_players();
        }
        if !self.available_players.is_empty() {
            let next_idx = match self.active_player_index {
                Some(idx) => (idx + 1) % self.available_players.len(),
                None => 0,
            };
            self.select_player(next_idx);
        }
    }

    fn start_probing(&mut self, path: &str) {
        self.is_probing = true;
        self.pending_session = None;
        self.source_path = Some(PathBuf::from(path));
        let (tx, rx) = mpsc::channel();
        self.probe_receiver = Some(rx);
        let path_clone = path.to_string();
        self.status_message = Some(format!("Probing metadata/keyframes for {}...", path));
        std::thread::spawn(move || {
            let res = crate::ffmpeg::probe::probe_media(&path_clone);
            let _ = tx.send(res);
        });
    }

    pub fn poll_player(&mut self) {
        // Periodically refresh available players (every ~1s or 10 poll cycles)
        self.poll_count = self.poll_count.wrapping_add(1);
        if self.poll_count % 10 == 0 {
            self.refresh_players();
        }

        // 1. Check background probing thread
        if self.is_probing {
            if let Some(rx) = &self.probe_receiver {
                if let Ok(res) = rx.try_recv() {
                    self.is_probing = false;
                    self.probe_receiver = None;
                    match res {
                        Ok(metadata) => {
                            self.metadata = Some(metadata.clone());
                            self.timeline_state = TimelineState::new(metadata.duration_seconds);
                            self.status_message = Some("Keyframe probing complete.".to_string());
                            if let Some(source_path) = &self.source_path {
                                if let Ok(session_data) = crate::session::load(source_path) {
                                    let validation = crate::session::validate(&session_data, source_path);
                                    if validation.source_exists {
                                        self.pending_session = Some(session_data.clone());
                                        self.mode = AppMode::SessionRestore;
                                        let seg_count = session_data.segments.len();
                                        if validation.modified_match {
                                            self.status_message = Some(format!(
                                                "Session found: {} segments from previous session. Restore? [Y/n]",
                                                seg_count
                                            ));
                                        } else {
                                            self.status_message = Some(format!(
                                                "Session found: {} segments. Warning: Source file may have changed, keyframe positions could differ. Restore? [Y/n]",
                                                seg_count
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            self.status_message = Some(format!("Probing failed: {}", e));
                        }
                    }
                }
            }
        }

        // 2. Check background export thread
        self.poll_export();

        // 3. Poll active player position and status
        if let Some(idx) = self.active_player_index {
            if idx < self.available_players.len() {
                // Temporarily get player state
                let (pos, duration, playing, source_path) = {
                    let player = &self.available_players[idx];
                    (player.position(), player.duration(), !player.is_paused(), player.source_path())
                };
                
                // INHIBIT POSITION POLLING DURING INTERACTION
                if !self.is_dragging_timeline && std::time::Instant::now().duration_since(self.last_seek_time) > std::time::Duration::from_millis(150) {
                    self.current_time = pos;
                }
                self.player_playing = playing;

                // Sync timeline duration if timeline duration is 0 but player reports a positive duration
                if duration > 0.0 && self.timeline_state.duration <= 0.0 {
                    self.timeline_state = TimelineState::new(duration);
                    
                    // If we have a CLI source path or active player reported path, start probing
                    if let Some(ref path) = source_path {
                        self.start_probing(path);
                    } else if let Some(cli_path) = &self.cli_source_path {
                        let path_str = cli_path.to_string_lossy().to_string();
                        self.start_probing(&path_str);
                    } else {
                        // Create dummy metadata
                        self.metadata = Some(MediaMetadata {
                            source_path: String::new(),
                            duration_seconds: duration,
                            format_name: None,
                            keyframes_seconds: Vec::new(),
                        });
                    }
                }

                // If player reported source path changes, start probing it
                if let Some(ref path) = source_path {
                    let current_path_str = self.source_path.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
                    if !path.is_empty() && path.as_str() != current_path_str.as_str() && !self.is_probing {
                        self.start_probing(path);
                    }
                }
            }
        }
        self.sync_auto_selection();
    }

    pub fn sync_auto_selection(&mut self) {
        let time = self.current_time;
        if let Some(idx) = self.segments.iter().position(|s| time >= s.start_seconds && time <= s.end_seconds) {
            if self.selected_segment != Some(idx) {
                self.selected_segment = Some(idx);
            }
        } else {
            self.selected_segment = None;
        }
    }

    pub fn poll_export(&mut self) {
        let Some(rx) = &self.export_receiver else { return };
        match rx.try_recv() {
            Ok(ExportMsg::Progress(msg)) => {
                self.status_message = Some(msg);
            }
            Ok(ExportMsg::Done(outputs)) => {
                self.status_message = Some(format!("Exported {} clips successfully.", outputs.len()));
                self.mode = AppMode::Normal;
                self.is_exporting = false;
                self.export_receiver = None;
            }
            Ok(ExportMsg::MergedDone(output)) => {
                self.status_message = Some(format!("Merged clip exported to: {}", output));
                self.mode = AppMode::Normal;
                self.is_exporting = false;
                self.export_receiver = None;
            }
            Ok(ExportMsg::Failed(e)) => {
                self.status_message = Some(format!("Export failed: {}", e));
                self.mode = AppMode::Normal;
                self.is_exporting = false;
                self.export_receiver = None;
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                self.export_receiver = None;
                self.is_exporting = false;
                self.mode = AppMode::Normal;
            }
        }
    }

    pub fn apply_action(&mut self, action: AppAction) {
        if self.mode == AppMode::EditLabel {
            match &action {
                AppAction::Char(c) => {
                    self.label_input.push(*c);
                }
                AppAction::Backspace => {
                    self.label_input.pop();
                }
                AppAction::ConfirmSegment | AppAction::EditLabel => {
                    if let Some(idx) = self.selected_segment {
                        if idx < self.segments.len() {
                            self.push_undo();
                            self.segments[idx].label = if self.label_input.is_empty() { None } else { Some(self.label_input.clone()) };
                            self.save_session();
                        }
                    }
                    self.mode = AppMode::Normal;
                }
                AppAction::Cancel => {
                    self.mode = AppMode::Normal;
                }
                AppAction::Quit => {
                    self.should_quit = true;
                }
                _ => {}
            }
            return;
        }
        if self.mode == AppMode::SessionRestore {
            match &action {
                AppAction::RestoreSession | AppAction::ConfirmSegment => {
                    if let Some(session_data) = self.pending_session.take() {
                        self.segments = session_data.segments;
                        self.selected_segment = if self.segments.is_empty() { None } else { Some(0) };
                        self.undo_stack.clear();
                        self.redo_stack.clear();
                        self.mode = AppMode::Normal;
                        self.status_message = Some("Session restored.".to_string());
                        self.save_session();
                        self.sync_auto_selection();
                    } else {
                        self.mode = AppMode::Normal;
                    }
                }
                AppAction::DiscardSession | AppAction::Cancel => {
                    self.pending_session = None;
                    self.mode = AppMode::Normal;
                    self.status_message = Some("Session discarded.".to_string());
                    self.save_session();
                    self.sync_auto_selection();
                }
                AppAction::Quit => {
                    self.should_quit = true;
                }
                _ => {}
            }
            return;
        }

        let duration = self.timeline_state.duration;

        match &action {
            AppAction::None => {}
            AppAction::Quit => {
                self.should_quit = true;
            }
            AppAction::Cancel => {
                let had_in_point = self.pending_in_point.is_some();
                self.mode = AppMode::Normal;
                self.pending_in_point = None;
                if !had_in_point {
                    self.selected_segment = None;
                }
                self.status_message = Some("Cancelled".to_string());
            }
            AppAction::ToggleHelp => {
                self.mode = if matches!(self.mode, AppMode::Help) {
                    AppMode::Normal
                } else {
                    AppMode::Help
                };
            }
            AppAction::TogglePlay => {
                if let Some(player) = self.active_player_mut() {
                    let _ = player.toggle_play();
                }
            }
            AppAction::SeekForward(n) => {
                if let Some(player) = self.active_player_mut() {
                    let _ = player.seek(*n);
                }
                self.current_time = (self.current_time + *n).min(duration);
                self.last_seek_time = std::time::Instant::now();
                self.sync_auto_selection();
            }
            AppAction::SeekBackward(n) => {
                if let Some(player) = self.active_player_mut() {
                    let _ = player.seek(-*n);
                }
                self.current_time = (self.current_time - *n).max(0.0);
                self.last_seek_time = std::time::Instant::now();
                self.sync_auto_selection();
            }
            AppAction::SeekToStart => {
                if let Some(player) = self.active_player_mut() {
                    let _ = player.seek_absolute(0.0);
                }
                self.current_time = 0.0;
                self.last_seek_time = std::time::Instant::now();
                self.sync_auto_selection();
            }
            AppAction::SeekToEnd => {
                if let Some(player) = self.active_player_mut() {
                    let _ = player.seek_absolute(duration);
                }
                self.current_time = duration;
                self.last_seek_time = std::time::Instant::now();
                self.sync_auto_selection();
            }
            AppAction::SetInPoint => {
                self.pending_in_point = Some(self.current_time);
                self.status_message = Some(format!("In-point set at {}", self.format_time(self.current_time)));
            }
            AppAction::SetOutPoint | AppAction::ConfirmSegment => {
                if let Some(start) = self.pending_in_point {
                    self.push_undo();
                    let end = self.current_time;
                    let (s, e) = if start < end { (start, end) } else { (end, start) };
                    let segment = Segment {
                        id: uuid::Uuid::new_v4().to_string(),
                        start_seconds: s,
                        end_seconds: e,
                        label: None,
                    };
                    self.segments.push(segment);
                    self.segments.sort_by(|a, b| a.start_seconds.partial_cmp(&b.start_seconds).unwrap());
                    self.pending_in_point = None;
                    self.status_message = Some(format!("Segment added: {} - {}", self.format_time(s), self.format_time(e)));
                    self.save_session();
                    self.sync_auto_selection();
                } else if !self.segments.is_empty() {
                    // Try to find the segment to the left of or containing the playhead
                    let mut best_idx = None;
                    for (i, seg) in self.segments.iter().enumerate() {
                        if seg.start_seconds <= self.current_time {
                            best_idx = Some(i);
                        }
                    }
                    if let Some(idx) = best_idx {
                        self.push_undo();
                        let id = self.segments[idx].id.clone();
                        {
                            let seg = &mut self.segments[idx];
                            seg.end_seconds = self.current_time;
                            if seg.end_seconds < seg.start_seconds {
                                std::mem::swap(&mut seg.start_seconds, &mut seg.end_seconds);
                            }
                        }
                        self.segments.sort_by(|a, b| a.start_seconds.partial_cmp(&b.start_seconds).unwrap());
                        // Re-select the modified segment
                        if let Some(new_pos) = self.segments.iter().position(|s| s.id == id) {
                            self.selected_segment = Some(new_pos);
                            let s = self.segments[new_pos].start_seconds;
                            let e = self.segments[new_pos].end_seconds;
                            self.status_message = Some(format!("Segment updated: {} - {}", self.format_time(s), self.format_time(e)));
                        }
                        self.save_session();
                    } else {
                        self.status_message = Some("Set in-point first (Press 'a')".to_string());
                    }
                } else {
                    self.status_message = Some("Set in-point first (Press 'a')".to_string());
                }
            }
            AppAction::DeleteSegment => {
                if let Some(idx) = self.selected_segment {
                    if idx < self.segments.len() {
                        self.push_undo();
                        self.segments.remove(idx);
                        if self.segments.is_empty() {
                            self.selected_segment = None;
                        } else if idx >= self.segments.len() {
                            self.selected_segment = Some(self.segments.len() - 1);
                        }
                        self.status_message = Some("Segment deleted".to_string());
                        self.save_session();
                        self.sync_auto_selection();
                    }
                }
            }
            AppAction::SelectPrevSegment => {
                if let Some(idx) = self.selected_segment {
                    if idx > 0 {
                        self.selected_segment = Some(idx - 1);
                    }
                } else if !self.segments.is_empty() {
                    self.selected_segment = Some(self.segments.len() - 1);
                }
            }
            AppAction::SelectNextSegment => {
                if let Some(idx) = self.selected_segment {
                    if idx + 1 < self.segments.len() {
                        self.selected_segment = Some(idx + 1);
                    }
                } else if !self.segments.is_empty() {
                    self.selected_segment = Some(0);
                }
            }
            AppAction::SeekToSegmentStart => {
                if let Some(idx) = self.selected_segment {
                    if idx < self.segments.len() {
                        let start = self.segments[idx].start_seconds;
                        if let Some(player) = self.active_player_mut() {
                            let _ = player.seek_absolute(start);
                        }
                        self.current_time = start;
                        self.last_seek_time = std::time::Instant::now();
                        self.sync_auto_selection();
                    }
                }
            }
            AppAction::SeekToSegmentEnd => {
                if let Some(idx) = self.selected_segment {
                    if idx < self.segments.len() {
                        let end = self.segments[idx].end_seconds;
                        if let Some(player) = self.active_player_mut() {
                            let _ = player.seek_absolute(end);
                        }
                        self.current_time = end;
                        self.last_seek_time = std::time::Instant::now();
                        self.sync_auto_selection();
                    }
                }
            }
            AppAction::ZoomIn => {
                self.timeline_state.zoom_in(1.5, self.current_time);
            }
            AppAction::ZoomOut => {
                self.timeline_state.zoom_out(1.5, self.current_time);
            }
            AppAction::PanLeft => {
                self.timeline_state.pan(-10, self.timeline_rect.2);
            }
            AppAction::PanRight => {
                self.timeline_state.pan(10, self.timeline_rect.2);
            }
            AppAction::Undo => {
                self.undo();
                self.save_session();
                self.sync_auto_selection();
            }
            AppAction::Redo => {
                self.redo();
                self.save_session();
                self.sync_auto_selection();
            }
            AppAction::EditLabel => {
                if self.selected_segment.is_some() {
                    self.mode = AppMode::EditLabel;
                    self.label_input = self.selected_segment
                        .and_then(|idx| self.segments.get(idx))
                        .and_then(|seg| seg.label.clone())
                        .unwrap_or_default();
                } else {
                    self.status_message = Some("Select a segment to label".to_string());
                }
            }
            AppAction::SnapToKeyframe => {
                if let Some(meta) = &self.metadata {
                    if let Some(&nearest) = meta.keyframes_seconds.iter().min_by(|a, b| {
                        (*a - self.current_time).abs().partial_cmp(&(*b - self.current_time).abs()).unwrap()
                    }) {
                        if let Some(player) = self.active_player_mut() {
                            let _ = player.seek_absolute(nearest);
                        }
                        self.current_time = nearest;
                        self.last_seek_time = std::time::Instant::now();
                        self.status_message = Some(format!("Snapped to keyframe: {}", self.format_time(nearest)));
                        self.sync_auto_selection();
                    }
                }
            }
            AppAction::OpenFile(path) => {
                self.start_probing(path);
            }
            AppAction::Export => {
                if self.segments.is_empty() {
                    self.status_message = Some("Create at least one segment to export".to_string());
                    return;
                }
                if let Some(path) = self.source_path.as_ref().and_then(|p| p.to_str()) {
                    let (tx, rx) = mpsc::channel();
                    self.export_receiver = Some(rx);
                    self.mode = AppMode::Export;
                    self.is_exporting = true;
                    let source = path.to_string();
                    let segments = self.segments.clone();
                    std::thread::spawn(move || {
                        match crate::export::export_separate(&source, &segments, tx.clone()) {
                            Ok(outputs) => { let _ = tx.send(ExportMsg::Done(outputs)); }
                            Err(e) => { let _ = tx.send(ExportMsg::Failed(e)); }
                        }
                    });
                    self.status_message = Some("Exporting separate clips...".to_string());
                } else {
                    self.status_message = Some("No source media file known for export. Run a local player or pass media via CLI arg.".to_string());
                }
            }
            AppAction::ExportMerged => {
                if self.segments.is_empty() {
                    self.status_message = Some("Create at least one segment to export".to_string());
                    return;
                }
                if let Some(path) = self.source_path.as_ref().and_then(|p| p.to_str()) {
                    let (tx, rx) = mpsc::channel();
                    self.export_receiver = Some(rx);
                    self.mode = AppMode::Export;
                    self.is_exporting = true;
                    let source = path.to_string();
                    let segments = self.segments.clone();
                    std::thread::spawn(move || {
                        match crate::export::export_merged(&source, &segments, tx.clone()) {
                            Ok(output) => { let _ = tx.send(ExportMsg::MergedDone(output)); }
                            Err(e) => { let _ = tx.send(ExportMsg::Failed(e)); }
                        }
                    });
                    self.status_message = Some("Exporting merged clip...".to_string());
                } else {
                    self.status_message = Some("No source media file known for export. Run a local player or pass media via CLI arg.".to_string());
                }
            }
            AppAction::ExportSelected => {
                if let Some(idx) = self.selected_segment {
                    if let Some(path) = self.source_path.as_ref().and_then(|p| p.to_str()) {
                        let (tx, rx) = std::sync::mpsc::channel();
                        self.export_receiver = Some(rx);
                        self.mode = AppMode::Export;
                        self.is_exporting = true;
                        let source = path.to_string();
                        let segment = self.segments[idx].clone();
                        std::thread::spawn(move || {
                            match crate::export::export_separate(&source, &[segment], tx.clone()) {
                                Ok(outputs) => { let _ = tx.send(crate::app::ExportMsg::Done(outputs)); }
                                Err(e) => { let _ = tx.send(crate::app::ExportMsg::Failed(e)); }
                            }
                        });
                        self.status_message = Some(format!("Exporting segment {}...", idx + 1));
                    } else {
                        self.status_message = Some("No source media file known for export.".to_string());
                    }
                } else {
                    self.status_message = Some("Select a segment first to export it individually".to_string());
                }
            }
            AppAction::SwitchPlayer => {
                self.cycle_player();
            }
            AppAction::MousePress { button, row, col } => {
                if self.is_inside_timeline(*row, *col) {
                    let pixel_x = col.saturating_sub(self.timeline_rect.0);
                    let target_time = self.timeline_state.pixel_to_time(pixel_x, self.timeline_rect.2);

                    if button == &MouseButton::Left {
                        self.is_dragging_timeline = true;
                        self.drag_last_col = *col;
                        self.drag_was_playing = self.player_playing;
                        if self.player_playing {
                            if let Some(p) = self.active_player_mut() {
                                let _ = p.pause();
                            }
                        }
                        if let Some(p) = self.active_player_mut() {
                            let _ = p.seek_absolute(target_time);
                        }
                        self.current_time = target_time.clamp(0.0, self.timeline_state.duration);
                        self.last_seek_time = std::time::Instant::now();
                        self.sync_auto_selection();
                    } else if button == &MouseButton::Right {
                        self.is_panning_timeline = true;
                        self.drag_last_col = *col;
                    }
                } else if self.is_inside_segments(*row, *col) {
                    if button == &MouseButton::Left {
                        let click_row = row.saturating_sub(self.segments_rect.1);
                        let idx = click_row as usize;
                        if idx < self.segments.len() {
                            self.selected_segment = Some(idx);
                            let start = self.segments[idx].start_seconds;
                            if let Some(player) = self.active_player_mut() {
                                let _ = player.seek_absolute(start);
                            }
                            self.current_time = start;
                            self.last_seek_time = std::time::Instant::now();
                        }
                    }
                }
            }
            AppAction::MouseDrag { row: _, col } => {
                if self.is_dragging_timeline {
                    let pixel_x = col.saturating_sub(self.timeline_rect.0);
                    let target_time = self.timeline_state.pixel_to_time(pixel_x, self.timeline_rect.2);
                    self.current_time = target_time.clamp(0.0, self.timeline_state.duration);
                    
                    let now = std::time::Instant::now();
                    if now.duration_since(self.last_seek_time) >= std::time::Duration::from_millis(50) {
                        if let Some(p) = self.active_player_mut() {
                            let _ = p.seek_absolute(target_time);
                        }
                        self.last_seek_time = now;
                    }
                    self.sync_auto_selection();
                } else if self.is_panning_timeline {
                    let delta = *col as i16 - self.drag_last_col as i16;
                    // Pan timeline
                    self.timeline_state.pan(-delta, self.timeline_rect.2);
                    self.drag_last_col = *col;
                }
            }
            AppAction::MouseRelease { row: _, col } => {
                if self.is_dragging_timeline {
                    self.is_dragging_timeline = false;
                    let pixel_x = col.saturating_sub(self.timeline_rect.0);
                    let target_time = self.timeline_state.pixel_to_time(pixel_x, self.timeline_rect.2);
                    let drag_was_playing = self.drag_was_playing;
                    if let Some(p) = self.active_player_mut() {
                        let _ = p.seek_absolute(target_time);
                        if drag_was_playing {
                            let _ = p.play();
                        }
                    }
                    self.current_time = target_time.clamp(0.0, self.timeline_state.duration);
                    self.last_seek_time = std::time::Instant::now();
                    self.sync_auto_selection();
                } else if self.is_panning_timeline {
                    self.is_panning_timeline = false;
                }
            }
            AppAction::RestoreSession | AppAction::DiscardSession => {}
            AppAction::MouseScroll { up, row: _, col } => {
                // Zoom anchored at mouse position X
                let pixel_x = col.saturating_sub(self.timeline_rect.0);
                let anchor_time = self.timeline_state.pixel_to_time(pixel_x, self.timeline_rect.2);
                if *up {
                    self.timeline_state.zoom_in(1.2, anchor_time);
                } else {
                    self.timeline_state.zoom_out(1.2, anchor_time);
                }
            }
            AppAction::Char(_) | AppAction::Backspace => {}
        }
    }

    fn save_session(&self) {
        if let Some(source_path) = &self.source_path {
            let _ = crate::session::save(source_path, &self.segments);
        }
    }

    fn is_inside_timeline(&self, row: u16, col: u16) -> bool {
        let (tx, ty, tw, th) = self.timeline_rect;
        row >= ty && row < ty + th && col >= tx && col < tx + tw
    }

    fn is_inside_segments(&self, row: u16, col: u16) -> bool {
        let (sx, sy, sw, sh) = self.segments_rect;
        row >= sy && row < sy + sh && col >= sx && col < sx + sw
    }

    pub fn push_undo(&mut self) {
        let snapshot = AppStateSnapshot {
            segments: self.segments.clone(),
            selected_segment: self.selected_segment,
            current_time: self.current_time,
        };
        self.undo_stack.push(snapshot);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(snapshot) = self.undo_stack.pop() {
            let current = AppStateSnapshot {
                segments: self.segments.clone(),
                selected_segment: self.selected_segment,
                current_time: self.current_time,
            };
            self.redo_stack.push(current);
            self.segments = snapshot.segments;
            self.selected_segment = snapshot.selected_segment;
            self.current_time = snapshot.current_time;
            let current_time = self.current_time;
            if let Some(p) = self.active_player_mut() {
                let _ = p.seek_absolute(current_time);
            }
        }
    }

    pub fn redo(&mut self) {
        if let Some(snapshot) = self.redo_stack.pop() {
            let current = AppStateSnapshot {
                segments: self.segments.clone(),
                selected_segment: self.selected_segment,
                current_time: self.current_time,
            };
            self.undo_stack.push(current);
            self.segments = snapshot.segments;
            self.selected_segment = snapshot.selected_segment;
            self.current_time = snapshot.current_time;
            let current_time = self.current_time;
            if let Some(p) = self.active_player_mut() {
                let _ = p.seek_absolute(current_time);
            }
        }
    }

    pub fn timecode(&self) -> String {
        self.format_time(self.current_time)
    }

    pub fn duration_timecode(&self) -> String {
        self.format_time(self.timeline_state.duration)
    }

    pub fn format_time(&self, seconds: f64) -> String {
        if !seconds.is_finite() || seconds < 0.0 {
            return "00:00.000".to_string();
        }
        let h = (seconds / 3600.0).floor() as u32;
        let m = ((seconds % 3600.0) / 60.0).floor() as u32;
        let s = (seconds % 60.0).floor() as u32;
        let ms = ((seconds.fract()) * 1000.0).round() as u32;
        if h > 0 {
            format!("{h:02}:{m:02}:{s:02}.{ms:03}")
        } else {
            format!("{m:02}:{s:02}.{ms:03}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_app_state_session_restore_flow() {
        let temp_dir = std::env::temp_dir();
        let test_video = temp_dir.join("test_app_video.mp4");
        fs::write(&test_video, "dummy content").unwrap();

        let segments = vec![
            Segment {
                id: "uuid-123".to_string(),
                start_seconds: 5.0,
                end_seconds: 15.0,
                label: Some("Restore me".to_string()),
            }
        ];
        crate::session::save(&test_video, &segments).unwrap();

        let mut state = AppState::new();
        state.source_path = Some(test_video.clone());

        let metadata = MediaMetadata {
            source_path: test_video.to_string_lossy().into_owned(),
            duration_seconds: 60.0,
            format_name: Some("mp4".to_string()),
            keyframes_seconds: vec![0.0, 10.0, 20.0],
        };

        state.metadata = Some(metadata.clone());
        state.timeline_state = TimelineState::new(metadata.duration_seconds);

        if let Some(source_path) = &state.source_path {
            if let Ok(session_data) = crate::session::load(source_path) {
                let validation = crate::session::validate(&session_data, source_path);
                if validation.source_exists {
                    state.pending_session = Some(session_data.clone());
                    state.mode = AppMode::SessionRestore;
                    let seg_count = session_data.segments.len();
                    if validation.modified_match {
                        state.status_message = Some(format!(
                            "Session found: {} segments from previous session. Restore? [Y/n]",
                            seg_count
                        ));
                    }
                }
            }
        }

        assert_eq!(state.mode, AppMode::SessionRestore);
        assert!(state.pending_session.is_some());
        assert_eq!(state.pending_session.as_ref().unwrap().segments.len(), 1);

        state.apply_action(AppAction::ZoomIn);
        assert_eq!(state.mode, AppMode::SessionRestore);
        assert_eq!(state.segments.len(), 0);

        state.apply_action(AppAction::RestoreSession);

        assert_eq!(state.mode, AppMode::Normal);
        assert_eq!(state.segments.len(), 1);
        assert_eq!(state.segments[0].label.as_deref(), Some("Restore me"));
        assert!(state.pending_session.is_none());

        let _ = fs::remove_file(&test_video);
        let _ = fs::remove_file(crate::session::session_path(&test_video));
    }

    #[test]
    fn test_app_state_session_discard_flow() {
        let temp_dir = std::env::temp_dir();
        let test_video = temp_dir.join("test_app_video_discard.mp4");
        fs::write(&test_video, "dummy content").unwrap();

        let segments = vec![
            Segment {
                id: "uuid-123".to_string(),
                start_seconds: 5.0,
                end_seconds: 15.0,
                label: Some("Restore me".to_string()),
            }
        ];
        crate::session::save(&test_video, &segments).unwrap();

        let mut state = AppState::new();
        state.source_path = Some(test_video.clone());

        let metadata = MediaMetadata {
            source_path: test_video.to_string_lossy().into_owned(),
            duration_seconds: 60.0,
            format_name: Some("mp4".to_string()),
            keyframes_seconds: vec![0.0, 10.0, 20.0],
        };

        state.metadata = Some(metadata.clone());
        state.timeline_state = TimelineState::new(metadata.duration_seconds);

        if let Some(source_path) = &state.source_path {
            if let Ok(session_data) = crate::session::load(source_path) {
                let validation = crate::session::validate(&session_data, source_path);
                if validation.source_exists {
                    state.pending_session = Some(session_data.clone());
                    state.mode = AppMode::SessionRestore;
                }
            }
        }

        assert_eq!(state.mode, AppMode::SessionRestore);

        state.apply_action(AppAction::DiscardSession);

        assert_eq!(state.mode, AppMode::Normal);
        assert_eq!(state.segments.len(), 0);
        assert!(state.pending_session.is_none());

        let session_file = crate::session::session_path(&test_video);
        assert!(session_file.exists());
        let loaded = crate::session::load(&test_video).unwrap();
        assert_eq!(loaded.segments.len(), 0);

        let _ = fs::remove_file(&test_video);
        let _ = fs::remove_file(&session_file);
    }
}
