use std::{collections::HashSet, default::default};

use float_ord::FloatOrd;
use interfaces::types::{BlockApprox, BlockKind, BlockLocation, BlockState, SimpleType};
use itertools::Itertools;
use num::traits::Pow;

use crate::{
    client::{physics::speed::Speed, state::local::inventory::PlayerInventory},
    protocol::Face,
    storage::blocks::WorldBlocks,
    types::{Direction, Displacement, Location},
};

pub mod speed;
pub mod tools;

const ACC_G: f64 = 0.08;

// player width divided by 2
const PLAYER_WIDTH_2: f64 = 0.6 / 2.0; // + 0.001;
                                       // const PLAYER_WIDTH_2_REG: f64 = 0.6 / 2.0;

// remove 0.1
const PLAYER_HEIGHT: f64 = 1.79999;
const PLAYER_HEIGHT_Y: Displacement = Displacement::new(0., PLAYER_HEIGHT, 0.);

const UNIT_Y: Displacement = Displacement::new(0., 1., 0.);
const EPSILON_Y: Displacement = Displacement::new(0., 0.001, 0.);

#[derive(Debug, Default)]
struct Pending {
    strafe: Option<Strafe>,
    pub place: Option<BlockPlaced>,
    jump: bool,
    line: Option<Line>,
    speed: Speed,
}

fn effects_multiplier(speed: f64, slowness: f64) -> f64 {
    (1.0 + 0.2 * speed) * (1. - 0.15 * slowness)
}

fn initial_ver(jump_boost: u32) -> f64 {
    0.42 + 0.1 * f64::from(jump_boost)
}

fn ver_speed(prev_speed: f64) -> f64 {
    const GRAV: f64 = 0.08;
    const DRAG: f64 = 0.98;
    // if tentative_speed.abs() < 0.003 { 0. } else { tentative_speed }
    (prev_speed - GRAV) * DRAG
}

fn ground_speed(
    prev_speed: f64,
    prev_slip: f64,
    move_mult: f64,
    effect_mult: f64,
    slip: f64,
) -> f64 {
    let momentum = prev_speed * prev_slip * 0.91;
    let acc = 0.1 * move_mult * effect_mult * (0.6 / slip).pow(3);
    momentum + acc
}

fn air_speed(prev_speed: f64, prev_slip: f64, move_mult: f64) -> f64 {
    let momentum = prev_speed * prev_slip * 0.91;
    let acc = 0.02 * move_mult;
    momentum + acc
}

#[derive(Debug)]
struct MovementState {
    speeds: [f64; 2],
    y_vel: f64,
    slip: f64,
    falling: bool,
}

impl Default for MovementState {
    fn default() -> Self {
        Self {
            speeds: default(),
            y_vel: 0.0,
            slip: BlockKind::DEFAULT_SLIP,
            falling: false,
        }
    }
}

#[derive(Debug)]
pub struct BlockPlaced {
    pub location: BlockLocation,
    pub face: Face,
}

pub struct Actions {
    pub block_placed: Option<BlockPlaced>,
}

/// # Purpose
/// Used to simulate a player position. Takes in movement events (such as
/// jumping and strafing) and allows polling the resulting player data---for
/// instance location. # Resources
/// - [Minecaft Parkour](https://www.mcpk.wiki/wiki/Movement_Formulas)
#[derive(Debug, Default)]
pub struct Physics {
    location: Location,
    look: Direction,
    prev: MovementState,
    horizontal: Displacement,
    pending: Pending,
    in_water: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[allow(unused)]
pub enum Strafe {
    Left,
    Right,
}

#[derive(Debug)]
pub enum Line {
    Forward,
    Backward,
}

pub fn mot_xz(mut strafe: f64, mut forward: f64, movement_factor: f64) -> [f64; 2] {
    let dist2 = strafe * strafe + forward * forward;
    if dist2 >= 1.0E-4 {
        let dist = dist2.sqrt().max(1.0);

        let multiplier = movement_factor / dist;

        strafe *= multiplier;
        forward *= multiplier;

        [strafe, forward]
    } else {
        [0., 0.]
    }
}

impl Physics {
    /// move to location and zero out velocity
    pub fn teleport(&mut self, location: Location) {
        // sometimes the server tells us that should be in a block *_*
        self.location = location;

        // so we don't glitch into ground
        self.location.y += 0.001;
        self.prev = MovementState::default();
        self.prev.falling = true;
    }

    pub fn jump(&mut self) {
        self.pending.jump = true;
    }

    /// if the bot is at the highest location of a jump
    #[allow(unused)]
    pub fn at_apex(&self) -> bool {
        self.prev.falling && self.prev.y_vel >= 0.0 && self.prev.y_vel - ACC_G < 0.0
    }

    pub fn look(&mut self, direction: Direction) {
        self.look = direction;
        self.horizontal = direction.horizontal().unit_vector();
    }

    pub fn look_at(&mut self, loc: Location) {
        let current = self.location + Displacement::EYE_HEIGHT;
        let displacement = loc - current;
        self.look(displacement.into());
    }

    pub const fn direction(&self) -> Direction {
        self.look
    }

    pub fn line(&mut self, line: Line) {
        self.pending.line = Some(line);
    }

    // pub fn strafe(&mut self, strafe: Strafe) {
    //     self.pending.strafe = Some(strafe);
    // }

    pub fn place_hand_face(&mut self, against: BlockLocation, face: Face) {
        let locations = against.faces();
        let face_idx = face as usize;

        // we know this is valid because face_idx < 6
        #[allow(clippy::indexing_slicing)]
        let place_loc = locations[face_idx];

        self.look_at(place_loc);

        self.pending.place = Some(BlockPlaced {
            location: against,
            face,
        });
    }

    pub fn place_hand(&mut self, against: BlockLocation) {
        let faces = against.faces();
        let eye_loc = self.location + Displacement::EYE_HEIGHT;

        // we know there is at least one value
        #[allow(clippy::unwrap_used)]
        let face_idx = IntoIterator::into_iter(faces)
            .position_min_by_key(|&location| FloatOrd(location.dist2(eye_loc)))
            .unwrap();

        // we know this is within bounds
        #[allow(clippy::cast_possible_truncation)]
        let face = Face::from(face_idx as u8);

        self.place_hand_face(against, face);
    }

    pub fn speed(&mut self, speed: Speed) {
        self.pending.speed = speed;
    }

    /// if we are on the edge of a block
    #[allow(unused)]
    pub fn on_edge(&self) -> bool {
        let block_loc = BlockLocation::from(self.location());
        let centered = block_loc.center_bottom();
        let dx = (centered.x - self.location.x).abs();
        let dz = (centered.z - self.location.z).abs();
        let dist = dx.max(dz);
        dist > 0.35
    }

    pub fn in_cross_section(loc: Location, world: &WorldBlocks, set: &mut HashSet<BlockLocation>) {
        let dif_x = [-PLAYER_WIDTH_2, PLAYER_WIDTH_2];
        let dif_z = [-PLAYER_WIDTH_2, PLAYER_WIDTH_2];

        for dx in dif_x {
            for dz in dif_z {
                let test_loc = loc + Displacement::new(dx, 0., dz);
                let test_block_loc = BlockLocation::from(test_loc);
                // let's also count avoid as most avoid blocks are semi-full
                let solid = matches!(
                    world.get_block_simple(test_block_loc),
                    Some(SimpleType::Solid | SimpleType::Avoid)
                );
                if solid {
                    set.insert(test_block_loc);
                }
            }
        }
    }

    pub fn cross_section_empty(loc: Location, world: &WorldBlocks) -> bool {
        let dif_x = [-PLAYER_WIDTH_2, PLAYER_WIDTH_2];
        let dif_z = [-PLAYER_WIDTH_2, PLAYER_WIDTH_2];

        for dx in dif_x {
            for dz in dif_z {
                let test_loc = loc + Displacement::new(dx, 0., dz);
                let test_block_loc = BlockLocation::from(test_loc);
                let solid = matches!(
                    world.get_block_simple(test_block_loc),
                    Some(SimpleType::Solid)
                );
                if solid {
                    return false;
                }
            }
        }
        true
    }

    pub fn tick(&mut self, world: &mut WorldBlocks, inventory: &PlayerInventory) -> Actions {
        if let Some(place) = self.pending.place.as_ref() {
            let against = place.location;
            let actual_loc = against + place.face.unit_location();

            if let Some(current) = inventory.current() {
                world.set_block(
                    actual_loc,
                    BlockState::from(current.kind.id(), current.damage),
                );
            } else {
                eprintln!("tried to place air");
                self.pending.place = None;
            };
        }

        let in_block_loc = BlockLocation::from(self.location);

        let in_block = world.get_block_simple(in_block_loc) == Some(SimpleType::Solid);

        #[allow(clippy::use_debug)]
        if in_block {
            println!(
                "was in block at {in_block_loc} of type {:?}",
                world.get_block(in_block_loc)
            );

            // auto jump if we are stuck in a block
            self.pending.jump = true;
        };

        let below_loc = self.location - EPSILON_Y;
        let below_block_loc = BlockLocation::from(below_loc);

        let current_block_loc = BlockLocation::from(self.location + EPSILON_Y);

        // first check is because we can be counted as falling if we are inside a block
        // (Minecraft is weird)
        let mut falling =
            below_block_loc.y == current_block_loc.y || Self::cross_section_empty(below_loc, world);

        let mut slip = match world.get_block(below_block_loc) {
            Some(BlockApprox::Realized(block)) => block.kind().slip(),
            // we might be on the edge of a block
            _ => BlockKind::DEFAULT_SLIP,
        };

        // the horizontal direction we are moving determined from the look direction
        let horizontal = self.horizontal;

        let [strafe_change, forward_change] = {
            let strafe_factor = match self.pending.strafe {
                None => 0.0,
                Some(Strafe::Right) => 1.0,
                Some(Strafe::Left) => -1.0,
            };

            let line_factor = match self.pending.line {
                None => 0.0,
                Some(Line::Forward) => 1.0,
                Some(Line::Backward) => -1.0,
            };

            let move_factor = self.pending.speed.multiplier();

            mot_xz(strafe_factor, line_factor, move_factor)
        };

        let sideways = horizontal.cross(UNIT_Y);

        let move_mults = [
            horizontal.dx * forward_change + sideways.dx * strafe_change,
            horizontal.dz * forward_change + sideways.dz * strafe_change,
        ];

        let effect_mult = effects_multiplier(0.0, 0.0);

        let mut speeds = [0.0, 0.0];

        let MovementState {
            speeds: prev_speeds,
            slip: prev_slip,
            ..
        } = self.prev;

        let mut y_vel = if self.in_water {
            const WATER_SLOW_DOWN: f64 = 0.8;

            for i in 0..2 {
                let momentum = prev_speeds[i] * 0.8;
                let acc = 0.02 * move_mults[i];
                speeds[i] = momentum + acc;
            }

            let feet_loc = BlockLocation::from(self.location);
            let head_loc = BlockLocation::from(self.location + PLAYER_HEIGHT_Y);

            let feet_flowing = world.get_block_kind(feet_loc) == Some(BlockKind::FLOWING_WATER);
            let head_flowing = world.get_block_kind(head_loc) == Some(BlockKind::FLOWING_WATER);

            let downwards_force = feet_flowing || head_flowing;

            // TODO: remove 0.014
            let res = self.prev.y_vel * WATER_SLOW_DOWN - 0.02
                + if self.pending.jump { 0.04 } else { 0. }
                - if downwards_force { 0.014 } else { 0. };

            if falling {
                res
            } else {
                // we can't go down if there is a block below us.
                res.max(0.0)
            }
        } else if falling {
            // when falling the slip of air is 1.0
            slip = 1.0;

            for i in 0..2 {
                speeds[i] = air_speed(prev_speeds[i], prev_slip, move_mults[i]);
            }

            ver_speed(self.prev.y_vel)
        } else if self.pending.jump {
            for i in 0..2 {
                speeds[i] =
                    ground_speed(prev_speeds[i], prev_slip, move_mults[i], effect_mult, slip);
            }
            if self.pending.speed == Speed::SPRINT {
                let move_displacement =
                    Displacement::new(move_mults[0], 0., move_mults[1]).normalize();
                speeds[0] += move_displacement.dx * 0.2;
                speeds[1] += move_displacement.dz * 0.2;
            }
            falling = true;
            initial_ver(0)
        } else {
            // we are not falling and not jumping
            for i in 0..2 {
                speeds[i] =
                    ground_speed(prev_speeds[i], prev_slip, move_mults[i], effect_mult, slip);
            }
            0.0
        };

        let mut new_loc_first = self.location + Displacement::new(0., y_vel, 0.);

        if y_vel < 0.0 {
            if !Self::cross_section_empty(new_loc_first - EPSILON_Y, world) {
                new_loc_first.y = new_loc_first.y.round();
                y_vel = 0.0;
                falling = false;
            }
        } else if y_vel >= 0.0 {
            // we are moving up

            let mut head_loc = new_loc_first + EPSILON_Y;
            head_loc.y += PLAYER_HEIGHT;

            if !Self::cross_section_empty(head_loc, world) {
                new_loc_first.y = head_loc.y.round() - PLAYER_HEIGHT - 0.0001;
                y_vel = 0.0;
            }
        }

        let prev_loc = self.location;

        {
            let mut new_loc = new_loc_first;
            new_loc.x += speeds[0];
            new_loc.z += speeds[1];

            let mut locs = HashSet::new();

            Self::in_cross_section(new_loc + EPSILON_Y, world, &mut locs);
            Self::in_cross_section(new_loc + UNIT_Y, world, &mut locs);
            Self::in_cross_section(new_loc + PLAYER_HEIGHT_Y, world, &mut locs);

            let mut stop_x: bool = false;
            let mut stop_z: bool = false;

            for loc in locs {
                let difference = loc.center_bottom() - prev_loc;
                let change_x = difference.dx.abs();
                let change_z = difference.dz.abs();
                if change_x <= change_z {
                    stop_z = true;
                }
                if change_z <= change_x {
                    stop_x = true;
                }
            }

            let prev_legs: BlockLocation = prev_loc.into();
            let legs: BlockLocation = new_loc.into();
            let head: BlockLocation = {
                let mut head_loc = new_loc;
                head_loc.y += PLAYER_HEIGHT;
                head_loc.into()
            };

            let prev_legs_block = world.get_block_simple(prev_legs);
            let leg_block = world.get_block_simple(legs);
            let head_block = world.get_block_simple(head);

            let against_block = stop_x || stop_z;
            if against_block {
                if stop_x {
                    speeds[0] = 0.0;
                }

                if stop_z {
                    speeds[1] = 0.0;
                }

                self.in_water = prev_legs_block == Some(SimpleType::Water)
                    || head_block == Some(SimpleType::Water);
            } else {
                self.in_water =
                    leg_block == Some(SimpleType::Water) || head_block == Some(SimpleType::Water);
            }

            // let leg_kind = world.get_block_kind(legs);
            // if leg_kind == Some(BlockKind::LADDER) {
            //     // yea this is jank
            //     self.in_water = true;
            // }
        }

        new_loc_first.x += speeds[0];
        new_loc_first.z += speeds[1];

        self.location = new_loc_first;

        let actions = Actions {
            block_placed: self.pending.place.take(),
        };

        self.pending = Pending::default();

        self.prev = MovementState {
            speeds,
            y_vel,
            slip,
            falling,
        };

        actions
    }

    pub const fn on_ground(&self) -> bool {
        !self.prev.falling
    }

    pub const fn location(&self) -> Location {
        self.location
    }

    pub const fn velocity(&self) -> Displacement {
        Displacement::new(self.prev.speeds[0], self.prev.y_vel, self.prev.speeds[1])
    }
}

#[cfg(test)]
mod tests {
    use more_asserts::*;

    use crate::{
        client::{
            physics::{speed::Speed, Line, Physics},
            state::local::inventory::PlayerInventory,
        },
        storage::blocks::WorldBlocks,
        types::{Direction, Displacement, Location},
    };

    #[test]
    fn test_run() {
        let mut world = WorldBlocks::flat();
        let mut physics = Physics::default();
        physics.teleport(Location::new(0., 1., 0.));

        let disp = Displacement::new(1., 0., 0.);
        let dir = Direction::from(disp);

        physics.look(dir);

        let mut ticks = 0;
        loop {
            physics.line(Line::Forward);
            physics.speed(Speed::SPRINT);
            physics.tick(&mut world, &PlayerInventory::default());

            ticks += 1;

            if physics.location.x >= 100.0 {
                break;
            }
        }

        // I got 17.95 seconds when I did this in Minecraft = 359 ticks which is close
        // enough to what I got which was 358
        assert_eq!(358, ticks);
    }

    #[test]
    fn test_sprint_jump() {
        let mut world = WorldBlocks::flat();
        let mut physics = Physics::default();
        physics.teleport(Location::new(0., 1., 0.));

        let disp = Displacement::new(1., 0., 0.);
        let dir = Direction::from(disp);

        physics.look(dir);

        let mut ticks = 0;
        loop {
            physics.line(Line::Forward);
            physics.speed(Speed::SPRINT);
            physics.jump();
            physics.tick(&mut world, &PlayerInventory::default());

            ticks += 1;

            if physics.location.x >= 100.0 {
                break;
            }
        }

        // I got 14.11 seconds when I did this in Minecraft = 282.2 ticks which is close
        // enough to what I got in the simulation which was 286
        assert_eq!(286, ticks);
    }

    #[allow(unused)]
    fn test_multiple_jumps() {
        let mut world = WorldBlocks::flat();
        let mut physics = Physics::default();
        physics.teleport(Location::new(0., 1., 0.));

        let mut zero_count = 0;
        for _ in 0..12 * 10 {
            physics.jump();
            physics.tick(&mut world, &PlayerInventory::default());
            if physics.location.y == 0.0 {
                zero_count += 1;
            }
        }

        assert_eq!(10, zero_count);
    }

    #[test]
    fn test_jump() {
        let mut world = WorldBlocks::flat();

        let mut physics = Physics::default();
        physics.teleport(Location::new(0., 1., 0.));

        physics.jump();

        let mut ticks_in_air = 0;
        let mut highest_y = 0_f64;
        loop {
            physics.tick(&mut world, &PlayerInventory::default());
            ticks_in_air += 1;
            if physics.on_ground() {
                break;
            }

            highest_y = highest_y.max(physics.location.y);

            // 1.25220 is what the Minecraft client gives me
            assert_le!(physics.location.y, 1. + 1.254);
            assert!(physics.location.y > 0.);
        }

        assert_le!((highest_y - 2.25221_f64).abs(), 0.001);

        // 12 is the number of blocks a player should be in the air
        assert_eq!(12, ticks_in_air);
    }
}
