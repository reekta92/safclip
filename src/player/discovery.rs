use mpris::PlayerFinder;
use crate::player::PlayerController;

pub struct PlayerDiscovery {
    finder: PlayerFinder,
}

impl PlayerDiscovery {
    pub fn new() -> Result<Self, String> {
        let finder = PlayerFinder::new().map_err(|e| e.to_string())?;
        Ok(Self { finder })
    }
    pub fn list_players(&self) -> Result<Vec<Box<dyn PlayerController>>, String> {
        let players = self.finder.find_all().map_err(|e| e.to_string())?;
        Ok(players.into_iter()
            .map(|p| Box::new(crate::player::BackgroundPlayer::new(p.identity().to_string())) as Box<dyn PlayerController>)
            .collect())
    }
}
