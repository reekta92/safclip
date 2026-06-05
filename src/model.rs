use serde::{Serialize, Deserialize};

/// A marked segment of the source video.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    pub id: String,
    pub start_seconds: f64,
    pub end_seconds: f64,
    pub label: Option<String>,
}

impl Segment {
    /// Duration of this segment in seconds.
    pub fn duration_seconds(&self) -> f64 {
        self.end_seconds - self.start_seconds
    }

    /// Validate that segment bounds are sane.
    pub fn validate_bounds(&self) -> Result<(), String> {
        if !self.start_seconds.is_finite() || !self.end_seconds.is_finite() {
            return Err("Segment bounds must be finite numbers".to_string());
        }
        if self.start_seconds < 0.0 {
            return Err("Segment start must be greater than or equal to 0".to_string());
        }
        if self.end_seconds <= self.start_seconds {
            return Err("Segment end must be greater than start".to_string());
        }
        Ok(())
    }
}

/// Probed metadata from a source media file.
#[derive(Debug, Clone)]
pub struct MediaMetadata {
    pub source_path: String,
    pub duration_seconds: f64,
    pub format_name: Option<String>,
    pub keyframes_seconds: Vec<f64>,
}

impl MediaMetadata {
    /// Validate metadata consistency.
    pub fn validate(&self) -> Result<(), String> {
        if !self.duration_seconds.is_finite() || self.duration_seconds <= 0.0 {
            return Err("Media duration must be a positive finite number".to_string());
        }
        if self.keyframes_seconds.windows(2).any(|pair| pair[0] > pair[1]) {
            return Err("Keyframes must be sorted in ascending order".to_string());
        }
        Ok(())
    }
}

/// Active mode of the application.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Normal,
    EditLabel,
    Export,
    Help,
    SessionRestore,
}

/// Snapshot of app state for undo/redo.
#[derive(Debug, Clone)]
pub struct AppStateSnapshot {
    pub segments: Vec<Segment>,
    pub selected_segment: Option<usize>,
    pub current_time: f64,
}

pub fn format_time(seconds: f64) -> String {
    if !seconds.is_finite() || seconds < 0.0 {
        return "00:00.000".to_string();
    }
    let minutes = (seconds / 60.0).floor() as u64;
    let secs = (seconds % 60.0).floor() as u64;
    let millis = ((seconds % 1.0) * 1000.0).floor() as u64;
    format!("{:02}:{:02}.{:03}", minutes, secs, millis)
}
