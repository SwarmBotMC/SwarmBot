/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::client::bot::{ActionState, process_command};
use crate::client::state::global::GlobalState;
use crate::client::state::local::inventory::ItemStack;
use crate::client::state::local::LocalState;
use crate::client::tasks::eat::EatTask;
use crate::protocol::InterfaceOut;
use crate::storage::block::{BlockLocation, BlockState};
use crate::storage::blocks::ChunkLocation;
use crate::storage::chunk::ChunkColumn;
use crate::types::{Chat, Dimension, Location, LocationOrigin, PlayerMessage};
use crate::storage::entities::EntityKind;

pub trait InterfaceIn {
    fn on_chat(&mut self, message: Chat);
    fn on_pickup_item(&mut self, idx: usize, item: ItemStack);
    fn on_lose_item(&mut self, idx: usize);
    fn on_death(&mut self);
    fn on_update_health(&mut self, health: f32, food: u8);
    fn on_dimension_change(&mut self, dimension: Dimension);
    fn on_join(&mut self);
    fn on_move(&mut self, location: Location);
    fn on_recv_chunk(&mut self, location: ChunkLocation, column: ChunkColumn, new: bool);
    fn on_entity_move(&mut self, id: u32, location: LocationOrigin);
    fn on_block_change(&mut self, location: BlockLocation, state: BlockState);
    fn on_entity_destroy(&mut self, id: u32);
    fn on_entity_spawn(&mut self, id: u32, location: Location, kind: EntityKind);
    fn on_disconnect(&mut self, reason: &str);
    fn on_socket_close(&mut self);
}

pub struct SimpleInterfaceIn<'a, I: InterfaceOut> {
    global: &'a mut GlobalState,
    local: &'a mut LocalState,
    actions: &'a mut ActionState,
    out: &'a mut I,
}

impl<I: InterfaceOut> SimpleInterfaceIn<'a, I> {
    pub fn new(local: &'a mut LocalState, actions: &'a mut ActionState, global: &'a mut GlobalState, out: &'a mut I) -> SimpleInterfaceIn<'a, I> {
        SimpleInterfaceIn {
            local,
            global,
            out,
            actions,
        }
    }
}


impl<'a, I: InterfaceOut> InterfaceIn for SimpleInterfaceIn<'a, I> {
    fn on_chat(&mut self, message: Chat) {
        println!("{}", message.clone().colorize());

        let mut process = |msg: PlayerMessage| {
            if let Some(cmd) = msg.into_cmd() {
                let name = cmd.command;
                let args_str: Vec<&str> = cmd.args.iter().map(|x| x.as_str()).collect();
                if let Err(err) = process_command(&name, &args_str, self.local, self.global, self.actions, self.out) {
                    println!("could not process command. Reason: {}", err);
                }
            }
        };

        if let Some(msg) = message.player_message() {
            process(msg);
        } else if let Some(msg) = message.player_dm() {
            process(msg);
        }
    }

    fn on_pickup_item(&mut self, idx: usize, item: ItemStack) {
        self.local.inventory.add(idx, item);
    }

    fn on_lose_item(&mut self, idx: usize) {
        self.local.inventory.remove(idx);
    }

    fn on_death(&mut self) {
        self.actions.clear();
        self.out.respawn();
        self.out.send_chat("I died... oof... well I guess I should respawn");
    }

    fn on_update_health(&mut self, health: f32, food: u8) {
        self.local.health = health;
        self.local.food = food;

        println!("updated health {} food is {}", health, food);

        // we should probably eat something
        if food < 10 {
            // if we could switch to food
            if self.local.inventory.switch_food(&self.global.block_data, self.out) {
                self.actions.schedule(EatTask::default());
            }
        }
    }

    fn on_dimension_change(&mut self, dimension: Dimension) {
        self.local.dimension = dimension;
    }

    fn on_join(&mut self) {
        // always start with slot 0
        self.out.change_slot(0);
    }

    fn on_move(&mut self, location: Location) {
        println!("moved {} -> {}", self.local.physics.location(), location);
        self.local.physics.teleport(location);
    }

    fn on_recv_chunk(&mut self, location: ChunkLocation, column: ChunkColumn, new: bool) {
        if new {
            self.global.blocks.add_column(location, column);
        } else {
            self.global.blocks.modify_column(location, column);
        }
    }

    fn on_entity_move(&mut self, id: u32, location: LocationOrigin) {
        self.global.entities.update_entity(id, self.local.bot_id, location);
    }

    fn on_block_change(&mut self, location: BlockLocation, state: BlockState) {
        self.global.blocks.set_block(location, state);
    }


    fn on_entity_destroy(&mut self, id: u32) {
        self.global.entities.remove_entity(id, self.local.bot_id);
    }

    fn on_entity_spawn(&mut self, id: u32, location: Location, kind: EntityKind) {
        self.global.entities.put_entity(id, self.local.bot_id, location, kind);
    }

    fn on_disconnect(&mut self, _reason: &str) {
        self.local.disconnected = true;
    }

    fn on_socket_close(&mut self) {}
}
