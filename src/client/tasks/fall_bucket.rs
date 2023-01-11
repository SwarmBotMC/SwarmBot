use interfaces::types::{BlockLocation, BlockState, SimpleType};

use crate::{
    client::{
        state::{global::GlobalState, local::LocalState},
        tasks::TaskTrait,
    },
    protocol::InterfaceOut,
};

#[derive(Default)]
pub struct FallBucketTask {
    placed: bool,
    ticks_since_place: u32,
    iter: bool,
    place_loc: Option<BlockLocation>,
}

impl TaskTrait for FallBucketTask {
    fn tick(
        &mut self,
        out: &mut impl InterfaceOut,
        local: &mut LocalState,
        global: &mut GlobalState,
    ) -> bool {
        const BUCKET_LEAVE_TICKS: u32 = 10;

        if self.placed {
            let place_loc = self.place_loc.unwrap();
            local.physics.look_at(place_loc.center_bottom());
            self.ticks_since_place += 1;

            // we msut wait or else a) anti cheat might be flagged b) we might remove the
            // water before we land
            if self.ticks_since_place == BUCKET_LEAVE_TICKS {
                out.use_item();
                // out.place_block(place_loc, Face::PosY);
                global.blocks.set_block(place_loc.above(), BlockState::AIR);
            }

            // this is so we don't have any conflicts with other tasks placing stuff and
            // potentially triggering anti-cheat
            if self.ticks_since_place > BUCKET_LEAVE_TICKS + 1 {
                return true;
            }

            return false;
        }

        let current_loc = local.physics.location();
        let below = global.blocks.first_below(BlockLocation::from(current_loc));
        match below {
            None => {}
            Some((location, _)) => {
                if self.iter {
                    local.inventory.switch_bucket(out);
                } else {
                    let height = local.physics.location().y;
                    if height - (f64::from(location.y) + 1.0) < 3.0 {
                        return true;
                    }
                }

                local.physics.look_at(location.center_bottom());
                let dy = current_loc.y - (f64::from(location.y) + 1.0);
                if dy < 3.4 {
                    // we don't have to place when going into water
                    if global.blocks.get_block_simple(location) == Some(SimpleType::Water) {
                        return true;
                    }

                    out.use_item();
                    // out.place_block(location, Face::PosY);
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
