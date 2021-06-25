use std::lazy::SyncLazy;

use crate::storage::block::{BlockLocation, SimpleType};
use crate::storage::blocks::WorldBlocks;
use crate::types::{Direction, Displacement, Location};

pub mod tools;


const SPRINT_SPEED: f64 = 0.2806;
const WALK_SPEED: f64 = 0.21585;
const SWIM_SPEED: f64 = 0.11;

const FALL_FACTOR: f64 = 0.02;

const FALL_TIMES: f64 = 0.9800000190734863;
const FALL_OFF_LAND: f64 = 0.5;

const LIQUID_MOTION_Y: f64 = 0.095;

fn jump_factor(jump_boost: Option<u32>) -> f64 {
    JUMP_UPWARDS_MOTION + match jump_boost {
        None => 0.0,
        Some(level) => (level as f64 + 1.0) * 0.1
    }
}

const JUMP_WATER: f64 = 0.03999999910593033;

const JUMP_UPWARDS_MOTION: f64 = 0.42;

const WATER_DECEL: f64 = 0.2;

const DRAG_MULT: f64 = 0.9800000190734863;
const ACC_G: f64 = 0.08;

// player width divided by 2
const PLAYER_WIDTH_2: f64 = 0.6 / 2.0;

// remove 0.1
const PLAYER_HEIGHT: f64 = 1.79;


// 1/2 at^2 + vt = 0
// t(1/2 at + v) = 0
// 1/2 at + v = 0
// t = -2v / a
// const JUMP_SECS: f64 = {
//     2.0 * jump_factor(None) / ACC_G
// };

/// Takes in normal Minecraft controls and tracks/records information
#[derive(Default, Debug)]
pub struct Physics {
    location: Location,
    look: Direction,
    horizontal: Displacement,
    velocity: Displacement,
    on_ground: bool,
    just_jumped: bool,
    just_descended: bool,
    pub in_water: bool,
}

pub enum Strafe {
    Left,
    Right,
}

pub enum Walk {
    Forward,
    Backward,
}

static UNIT_Y: SyncLazy<Displacement> = SyncLazy::new(|| {
    Displacement::new(0.0, 1.0, 0.0)
});

struct MovementProc {}

impl Physics {
    /// move to location and zero out velocity
    pub fn teleport(&mut self, location: Location) {
        self.location = location;
        self.velocity = Displacement::default();
    }

    pub fn jump(&mut self) {
        self.just_jumped = true;
    }

    pub fn descend(&mut self) {
        self.just_descended = true;
    }

    pub fn look(&mut self, direction: Direction) {
        self.look = direction;
        self.horizontal = direction.horizontal().unit_vector();
    }

    pub fn direction(&self) -> Direction {
        self.look
    }

    fn speed(&self) -> f64 {
        if self.in_water {
            SWIM_SPEED
        } else {
            WALK_SPEED
        }
    }

    pub fn walk(&mut self, walk: Walk) {
        let mut velocity = self.horizontal;
        velocity *= self.speed();
        if let Walk::Backward = walk {
            velocity *= -1.0;
        }

        self.velocity.dx = velocity.dx;
        self.velocity.dz = velocity.dz;
    }

    pub fn strafe(&mut self, strafe: Strafe) {
        let mut velocity = self.horizontal.cross(*UNIT_Y);


        velocity *= self.speed();
        if let Strafe::Left = strafe {
            velocity *= -1.0
        }

        self.velocity.dx = velocity.dx;
        self.velocity.dz = velocity.dz;
    }

    pub fn tick(&mut self, world: &WorldBlocks) {
        self.velocity.dy = (self.velocity.dy - ACC_G) * DRAG_MULT;

        if self.just_jumped {
            self.just_jumped = false;

            if self.on_ground {
                self.velocity.dy = jump_factor(None);
            }
        }
        // move y, x, z

        let prev_loc = self.location;
        let next_loc = prev_loc + self.velocity;

        let future_feet_bl = BlockLocation::from(next_loc);

        match world.get_block_simple(future_feet_bl) {
            None => return,
            Some(SimpleType::Solid) => {
                self.on_ground = true;
                self.velocity.dy = 0.;
                self.location = future_feet_bl.add_y(1).center_bottom();
                return;
            }
            _ => {}
        };

        if self.velocity.dy > 0.0 {
            let future_head_bl = BlockLocation::from(next_loc.add_y(PLAYER_HEIGHT));

            match world.get_block_simple(future_head_bl){
                None => return,
                Some(SimpleType::Solid) => { // we hit our head
                    self.velocity.dy = 0.0;
                    self.location.y = future_feet_bl.y as f64 - PLAYER_HEIGHT;
                }
                _ => {}
            }
        }

        self.location = next_loc;

        // reset walk
        self.velocity.dx = 0.0;
        self.velocity.dz = 0.0;
    }
    pub fn location(&self) -> Location {
        self.location
    }


    pub fn velocity(&self) -> Displacement {
        self.velocity
    }
    pub fn on_ground(&self) -> bool {
        self.on_ground
    }
}
