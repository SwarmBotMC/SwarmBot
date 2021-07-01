/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::protocol::{Face, InterfaceOut, Mine};
use crate::storage::block::BlockLocation;

#[enum_dispatch]
pub trait TaskTrait {
    /// return true if done
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool;
}

#[enum_dispatch(TaskTrait)]
pub enum Task {
    EatTask,
    MineTask,
    FallBucketTask,
}


pub struct EatTask {
    pub ticks: usize,
}

impl TaskTrait for EatTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, _: &mut LocalState, _: &mut GlobalState) -> bool {
        if self.ticks == 0 {
            out.finish_eating();
            true
        } else {
            self.ticks -= 1;
            false
        }
    }
}

pub struct MineTask {
    pub ticks: usize,
    pub location: BlockLocation,
    pub face: Face,
}

impl TaskTrait for MineTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, _: &mut LocalState, _: &mut GlobalState) -> bool {
        out.left_click();
        if self.ticks == 0 {
            out.mine(self.location, Mine::Finished, self.face);
            true
        } else {
            self.ticks -= 1;
            false
        }
    }
}

#[derive(Default)]
pub struct FallBucketTask {
    placed: bool,
    ticks_since_place: u32,
    location: Option<BlockLocation>,
}

impl TaskTrait for FallBucketTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {
        if self.placed {
            self.ticks_since_place += 1;
            if self.ticks_since_place > 40 {
                out.right_click();
                out.place_block(self.location.unwrap(), Face::PosY);
                return true;
            }
            return false;
        }

        let current_loc = local.physics.location();
        let below = global.world_blocks.first_below(current_loc.into());
        match below {
            None => {},
            Some((location, _)) => {
                local.physics.look_at(location.center_bottom());
                let dy = current_loc.y - (location.y as f64 + 1.0);
                if dy < 4.0 {
                    out.right_click();
                    out.place_block(location, Face::PosY);
                    self.location = Some(location);
                    self.placed = true;
                    self.ticks_since_place = 0;
                }
            }
        }

        false
    }
}
