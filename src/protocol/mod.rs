/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use packets::types::UUID;

use crate::bootstrap::Connection;
use crate::client::processor::InterfaceIn;
use crate::error::Res;
use crate::storage::block::BlockLocation;
use crate::types::{Direction, Location};

pub mod v340;

mod io;
mod transform;
mod encrypt;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Mine {
    Start,
    Cancel,
    Finished,
}

pub trait InterfaceOut {
    fn send_chat(&mut self, message: &str);
    fn left_click(&mut self);
    fn finish_eating(&mut self);
    fn right_click(&mut self);
    fn change_slot(&mut self, number: u8);
    fn mine(&mut self, location: BlockLocation, mine: Mine);
    fn respawn(&mut self);
    fn teleport(&mut self, location: Location);
    fn look(&mut self, direction: Direction);
    fn teleport_and_look(&mut self, location: Location, direction: Direction, on_ground: bool);
}

#[async_trait::async_trait]
pub trait Minecraft: Sized {
    type Queue: EventQueue;
    type Interface: InterfaceOut;
    async fn login(conn: Connection) -> Res<Login<Self::Queue, Self::Interface>>;
}

pub trait EventQueue {
    fn flush(&mut self, processor: &mut impl InterfaceIn);
}

#[derive(Debug)]
pub struct ClientInfo {
    pub username: String,
    pub uuid: UUID,
    pub entity_id: u32,
}


/// login for a given bot. Holds
pub struct Login<E: EventQueue, I: InterfaceOut> {
    pub queue: E,
    pub out: I,
    pub info: ClientInfo,
}
