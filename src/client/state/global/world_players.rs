// Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
