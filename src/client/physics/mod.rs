use crate::storage::world::WorldBlocks;
use crate::types::{Displacement, Location};
use crate::client::physics::bounding_box::BoundingBox;
use crate::storage::block::{BlockLocation, SimpleType};


mod bounding_box;

const JUMP_UPWARDS_MOTION: f64 = 0.42;

const SPRINT_SPEED: f64 = 0.30000001192092896;

const FALL_FACTOR: f64 = 0.02;

const FALL_TIMES: f64 = 0.9800000190734863;
const FALL_OFF_LAND: f64 = 0.5;

// const LIQUID_MOTION_Y: f64 = 0.30000001192092896;

// const fn jump_factor(jump_boost: Option<u32>) -> f64 {
//     JUMP_UPWARDS_MOTION + match jump_boost {
//         None => 0.0,
//         Some(level) => (level as f64 + 1.0) * 0.1
//     }
// }

const JUMP_WATER: f64 = 0.03999999910593033;
const ACC_G: f64 = 0.08;
const VEL_MULT: f64 = 0.9800000190734863;


/// 1/2 at^2 + vt = 0
/// t(1/2 at + v) = 0
/// 1/2 at + v = 0
/// t = -2v / a
// const JUMP_SECS: f64 = {
//     2.0 * jump_factor(None) / ACC_G
// };

struct PhysicsPlayer {
    location: Location,
    velocity: Displacement,
    on_ground: bool,
}

enum Movement {
    Fall
}

impl PhysicsPlayer {
    /// move to location and zero out velocity
    pub fn teleport(&mut self, location: Location) {
        self.location = location;
        self.velocity = Displacement::default();
    }

    // pub fn jump(&mut self) {
    //     if self.on_ground {
    //         self.on_ground = false;
    //         self.velocity = Displacement::new(0.0, jump_factor(None), 0.0)
    //     }
    // }

    pub fn tick(&mut self, world: &WorldBlocks) -> Option<Movement> {
        // move y, x, z


        if !self.on_ground {
            let prev_loc = self.location;
            let mut new_loc = prev_loc + self.velocity;

            let prev_block_loc: BlockLocation = prev_loc.into();
            let next_block_loc: BlockLocation = new_loc.into();


            if world.get_block_simple(next_block_loc).unwrap() == SimpleType::Solid {
                new_loc = prev_block_loc.centered();
                self.velocity.dy = 0.0;
                self.on_ground = true;
            } else {
                self.velocity.dy -= ACC_G;
            }

            self.location = new_loc;

            Some(Movement::Fall)
        }
        else {
            None
        }

    }
}
