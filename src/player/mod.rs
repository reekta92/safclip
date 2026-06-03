pub mod discovery;
pub mod mpris;

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
