pub struct Player {
    pub name: String,
    pub uuid: u128,
}

#[derive(Default)]
pub struct WorldPlayers {
    players: Vec<Player>,
}

impl WorldPlayers {
    pub fn add(&mut self, player: Player) {
        self.players.push(player);
    }

    pub fn by_name(&mut self, name: &str) -> Option<&Player> {
        self.players.iter().find(|player| player.name == name)
    }

    #[allow(unused)]
    pub fn by_uuid(&mut self, uuid: u128) -> Option<&Player> {
        self.players.iter().find(|player| player.uuid == uuid)
    }

    /// return true if successfully removed
    pub fn remove(&mut self, uuid: u128) -> bool {
        let mut action = || {
            let idx = self.players.iter().position(|player| player.uuid == uuid)?;
            self.players.swap_remove(idx);
            Some(())
        };

        action().is_some()
    }
}
