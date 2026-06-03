use std::time::Duration;
use mpris::{Player, PlaybackStatus};
use crate::player::PlayerController;

pub struct MprisPlayer {
    player: Player,
}

impl MprisPlayer {
    pub fn new(player: Player) -> Self {
        Self { player }
    }

    pub fn inner(&self) -> &Player {
        &self.player
    }
}

impl PlayerController for MprisPlayer {
    fn play(&mut self) -> Result<(), String> {
        self.player.play().map_err(|e| e.to_string())
    }

    fn pause(&mut self) -> Result<(), String> {
        self.player.pause().map_err(|e| e.to_string())
    }

    fn toggle_play(&mut self) -> Result<(), String> {
        self.player.play_pause().map_err(|e| e.to_string())
    }

    fn seek(&mut self, offset: f64) -> Result<(), String> {
        let offset_us = (offset * 1_000_000.0) as i64;
        // In mpris crate, player.seek takes microseconds offset
        self.player.seek(offset_us).map_err(|e| e.to_string())
    }

    fn seek_absolute(&mut self, position: f64) -> Result<(), String> {
        let position_us = (position * 1_000_000.0) as u64; // SetPosition expects microseconds
        let metadata = self.player.get_metadata().map_err(|e| e.to_string())?;
        let track_id = metadata.track_id().ok_or_else(|| "No track ID available".to_string())?;
        // SetPosition takes TrackId and position (in microseconds)
        self.player.set_position(track_id, &Duration::from_micros(position_us)).map_err(|e| e.to_string())
    }

    fn position(&self) -> f64 {
        // get_position returns Duration or similar
        self.player.get_position().map(|d| d.as_secs_f64()).unwrap_or(0.0)
    }

    fn duration(&self) -> f64 {
        self.player.get_metadata()
            .ok()
            .and_then(|m| m.length())
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0)
    }

    fn is_paused(&self) -> bool {
        self.player.get_playback_status()
            .map(|s| s == PlaybackStatus::Paused)
            .unwrap_or(true)
    }

    fn track_title(&self) -> Option<String> {
        self.player.get_metadata()
            .ok()
            .and_then(|m| m.title().map(|t| t.to_string()))
    }

    fn source_path(&self) -> Option<String> {
        let metadata = self.player.get_metadata().ok()?;
        let url = metadata.url()?;
        let path = if let Some(stripped) = url.strip_prefix("file://") {
            percent_decode(stripped).unwrap_or_else(|| stripped.to_string())
        } else {
            url.to_string()
        };
        Some(path)
    }

    fn identity(&self) -> String {
        self.player.identity().to_string()
    }
}

fn percent_decode(s: &str) -> Option<String> {
    let mut bytes = Vec::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let h1 = chars.next()?;
            let h2 = chars.next()?;
            let hex = format!("{}{}", h1, h2);
            let b = u8::from_str_radix(&hex, 16).ok()?;
            bytes.push(b);
        } else {
            bytes.push(c as u8);
        }
    }
    String::from_utf8(bytes).ok()
}
