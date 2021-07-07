/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/7/21, 12:21 AM
 */


use crate::storage::block::{BlockLocation, BlockState};
use crate::client::tasks::TaskTrait;
use crate::protocol::{InterfaceOut, Face};
use crate::client::state::local::LocalState;
use crate::client::state::global::GlobalState;

#[derive(Default)]
pub struct FallBucketTask {
    placed: bool,
    ticks_since_place: u32,
    iter: bool,
    place_loc: Option<BlockLocation>,
}

impl TaskTrait for FallBucketTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {
        const BUCKET_LEAVE_TICKS: u32 = 5;

        if self.placed {
            self.ticks_since_place += 1;
            if self.ticks_since_place > BUCKET_LEAVE_TICKS {
                out.use_item();
                let place_loc = self.place_loc.unwrap();
                out.place_block(place_loc, Face::PosY);
                global.blocks.set_block(place_loc.above(), BlockState::AIR);
                return true;
            }
            return false;
        }

        let current_loc = local.physics.location();
        let below = global.blocks.first_below(BlockLocation::from(current_loc));
        match below {
            None => {}
            Some((location, _)) => {
                if !self.iter {
                    let height = local.physics.location().y;
                    if height - (location.y as f64 + 1.0) < 3.0 {
                        return true;
                    }
                } else {
                    local.inventory.switch_bucket(out);
                }

                local.physics.look_at(location.center_bottom());
                let dy = current_loc.y - (location.y as f64 + 1.0);
                if dy < 3.4 {
                    out.use_item();
                    out.place_block(location, Face::PosY);
                    global.blocks.set_block(location.above(), BlockState::WATER);
                    self.place_loc = Some(location);
                    self.placed = true;
                    self.ticks_since_place = 0;
                }
            }
        }

        self.iter = true;

        false
    }
}
