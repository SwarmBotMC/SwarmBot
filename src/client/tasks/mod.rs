/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */
use crate::protocol::{InterfaceOut, Face, Mine};
use crate::client::state::local::LocalState;
use crate::storage::block::{BlockLocation};
use crate::client::state::global::GlobalState;


#[enum_dispatch]
pub trait TaskTrait {
    /// return true if done
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool;
}

#[enum_dispatch(TaskTrait)]
pub enum Task {
    EatTask,
    MineTask,
    FallBucketTask
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

pub struct FallBucketTask;

impl TaskTrait for FallBucketTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {
        let current_loc = local.physics.location();
        let below = global.world_blocks.first_below(current_loc.into());
        match below {
            None => true,
            Some((location, _)) => {
                let dy = current_loc.y - (location.y as f64 + 1.0);
                if dy < 4.0 {
                    out.place_block(location, Face::PosY);
                    true
                } else {
                    false
                }
            }
        }
    }
}
