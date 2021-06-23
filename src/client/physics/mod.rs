use std::lazy::SyncLazy;

use crate::storage::block::{BlockLocation, SimpleType};
use crate::storage::blocks::WorldBlocks;
use crate::types::{Direction, Displacement, Location};

mod bounding_box;

const JUMP_UPWARDS_MOTION: f64 = 0.42;

const SPRINT_SPEED: f64 = 0.30000001192092896;
const WALK_SPEED: f64 = SPRINT_SPEED * 0.6;

const FALL_FACTOR: f64 = 0.02;

const FALL_TIMES: f64 = 0.9800000190734863;
const FALL_OFF_LAND: f64 = 0.5;

// const LIQUID_MOTION_Y: f64 = 0.30000001192092896;

fn jump_factor(jump_boost: Option<u32>) -> f64 {
    JUMP_UPWARDS_MOTION + match jump_boost {
        None => 0.0,
        Some(level) => (level as f64 + 1.0) * 0.1
    }
}

const JUMP_WATER: f64 = 0.03999999910593033;
const ACC_G: f64 = 0.08;
const VEL_MULT: f64 = 0.9800000190734863;

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
        if self.on_ground {
            self.on_ground = false;
            self.velocity.dy = jump_factor(None);
        }
    }

    pub fn look(&mut self, direction: Direction) {
        self.look = direction;
        self.horizontal = direction.horizontal().unit_vector();
    }

    pub fn direction(&self) -> Direction {
        self.look
    }

    pub fn walk(&mut self, walk: Walk) {
        let mut velocity = self.horizontal;
        velocity *= WALK_SPEED;
        if let Walk::Backward = walk {
            velocity *= -1.0;
        }

        self.velocity.dx = velocity.dx;
        self.velocity.dz = velocity.dz;
    }

    pub fn strafe(&mut self, strafe: Strafe) {
        let mut velocity = self.horizontal.cross(*UNIT_Y);


        velocity *= WALK_SPEED;
        if let Strafe::Left = strafe {
            velocity *= -1.0
        }

        self.velocity.dx = velocity.dx;
        self.velocity.dz = velocity.dz;
    }

    pub fn tick(&mut self, world: &WorldBlocks) {
        // move y, x, z

        let prev_loc = self.location;

        let mut below_loc = prev_loc;
        below_loc.y -= 0.001;

        let below_loc: BlockLocation = below_loc.into();
        if world.get_block_simple(below_loc) == Some(SimpleType::WalkThrough) {
            self.on_ground = false;
        }

        {
            let dx = self.velocity.dx;
            let extra_dx = if dx == 0.0 { 0.0 } else { dx.signum() * PLAYER_WIDTH_2 };
            let end_dx = dx + extra_dx;

            let dz = self.velocity.dz;
            let extra_dz = if dz == 0.0 { 0.0 } else { dz.signum() * PLAYER_WIDTH_2 };
            let end_dz = dz + extra_dz;

            let dy = self.velocity.dy;

            let end_vel = Displacement::new(end_dx, dy, end_dz);
            let test_loc = prev_loc + end_vel;

            let legs: BlockLocation = test_loc.into();
            let head: BlockLocation = {
                let mut head_loc =test_loc;
                head_loc.y += PLAYER_HEIGHT;
                head_loc.into()
            };

            if world.get_block_simple(legs) == Some(SimpleType::Solid) || world.get_block_simple(head) == Some(SimpleType::Solid) {
                let new_dx = 0.0;
                let new_dz = 0.0;
                self.velocity.dx = new_dx;
                self.velocity.dz = new_dz;
            }
        }

        let mut new_loc = prev_loc + self.velocity;

        if !self.on_ground {
            let prev_block_loc: BlockLocation = prev_loc.into();
            let next_block_loc: BlockLocation = new_loc.into();

            let mut head_loc = new_loc;
            head_loc.y += PLAYER_HEIGHT;

            let head_loc = new_loc.into();

            if self.velocity.dy > 0.0 {// we are moving up
                if world.get_block_simple(head_loc) == Some(SimpleType::Solid) {
                    // we hit our heads!
                    println!("hit head");
                    self.velocity.dy = 0.0;
                    new_loc.y = (head_loc.1 as f64) - PLAYER_HEIGHT;
                } else {
                    // we can decelerate normally
                    self.velocity.dy -= ACC_G;
                }
            }else { // we are moving down
                match world.get_block_simple(next_block_loc) {
                    Some(SimpleType::Solid) => {
                        new_loc = prev_block_loc.centered();
                        self.velocity.dy = 0.0;
                        self.on_ground = true;
                    }
                    // we are falling
                    Some(SimpleType::WalkThrough) => {
                        self.velocity.dy -= ACC_G;
                    }
                    Some(kind) => {
                        self.velocity.dy -= ACC_G;
                        // we are not going to do anything
                        // panic!("unsupported physics block {:?}", kind);
                    }
                    // the chunk hasn't loaded, let's not apply physics
                    _ => {}
                }
            }
        }

        self.location = new_loc;

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
