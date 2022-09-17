// Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crossterm::style::Stylize;
use std::{
    fmt::{Display, Formatter},
    num::ParseIntError,
    time::Instant,
};

use float_ord::FloatOrd;
use interfaces::types::BlockLocation;
use itertools::Itertools;

use crate::{
    client::{
        state::{global::GlobalState, local::LocalState},
        tasks::{
            compound::CompoundTask, eat::EatTask, fall_bucket::FallBucketTask, mine::MineTask,
            navigate::BlockTravelTask, Task, TaskTrait,
        },
    },
    protocol::{EventQueue, Face, InterfaceOut},
    types::Displacement,
};

#[derive(Default)]
pub struct ActionState {
    task: Option<Task>,
}

impl ActionState {
    pub fn schedule<T: Into<Task>>(&mut self, task: T) {
        self.task = Some(task.into());
    }
    pub fn clear(&mut self) {
        self.task = None;
    }
}

pub struct Bot<Queue: EventQueue, Out: InterfaceOut> {
    pub state: LocalState,
    pub actions: ActionState,
    pub queue: Queue,
    pub out: Out,
}

impl<Queue: EventQueue, Out: InterfaceOut> Bot<Queue, Out> {
    pub fn run_sync(&mut self, global: &mut GlobalState) {
        match self.actions.task.as_mut() {
            None => {}
            Some(task) => {
                if task.tick(&mut self.out, &mut self.state, global) {
                    self.actions.task = None;
                }
            }
        }
        let actions = self
            .state
            .physics
            .tick(&mut global.blocks, &self.state.inventory);
        let physics = &self.state.physics;
        self.out
            .teleport_and_look(physics.location(), physics.direction(), physics.on_ground());

        // if self.actions.task.is_none() {
        //     // let mut vel = self.state.physics.velocity();
        //     // vel.dy = 0.;
        //     // if vel.mag2() > 0.01 {
        //     //     vel *= -1.;
        //     //     self.state.physics.look(Direction::from(vel));
        //     //     self.state.physics.speed(Speed::SPRINT);
        //     //     self.state.physics.line(Line::Forward);
        //     // }
        // }
        //
        //
        if let Some(place) = actions.block_placed.as_ref() {
            self.out.swing_arm();
            self.out.place_block(place.location, place.face);
        }

        // this should be after everything else as actions depend on the previous
        // location

        self.state.ticks += 1;
    }
}

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error(transparent)]
    Parse(#[from] ParseIntError),

    #[error(transparent)]
    Count(#[from] WrongArgCount),
}

#[derive(Debug)]
pub struct WrongArgCount {
    required: u32,
}

impl std::error::Error for WrongArgCount {}

impl WrongArgCount {
    pub fn new(required: u32) -> Self {
        Self { required }
    }
}

impl Display for WrongArgCount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("wrong arg count")
    }
}

/// Always returns None.
#[allow(clippy::many_single_char_names)]
pub fn process_command(
    name: &str,
    args: &[&str],
    local: &mut LocalState,
    global: &mut GlobalState,
    actions: &mut ActionState,
    out: &mut impl InterfaceOut,
) -> Result<(), ProcessError> {
    macro_rules! msg {
        () => {{
            println!();
        }};
        ($($msg: expr),*) => {{
            let to_print_raw = format!($($msg),*);
            let to_print = to_print_raw.bold().to_string();
            println!("{}", to_print);
        }};
    }

    match name {
        // "pillar" => {
        //     if let [a] = args {
        //         let y = a.parse()?;
        //         actions.schedule(PillarTask::new(y));
        //     }
        // }
        // "bridge" => {
        //     if let [a] = args {
        //         let amount = a.parse()?;
        //         actions.schedule(BridgeTask::new(amount, CardinalDirection::North, local));
        //     }
        // }
        // "block" => {
        //     local.inventory.switch_block(out);
        // }
        // "gotoc" => { // goto chunk
        //     if let [a, b] = args {
        //         let x = a.parse()?;
        //         let z = b.parse()?;
        //         let goal = ChunkLocation(x, z);
        //         actions.schedule(ChunkTravelTask::new(goal, local));
        //     }
        // }
        // "jump" => {
        //     local.physics.jump();
        // }
        "health" => {
            println!("Health: {}, Food: {}", local.health, local.food);
        }
        "follow" => {
            local.follow_closest = true;
        }
        "kys" => {
            // TODO: try to kill themself by fall damage/lava/etc
        }
        "eat" => {
            let eat_task = EatTask::default();
            actions.task = Some(eat_task.into());
        }
        "slot" => {
            if let [number] = args {
                let number: u8 = number.parse().unwrap();
                local.inventory.change_slot(number, out);
            }
        }
        "fall" => {
            let below = BlockLocation::from(local.physics.location()).below();

            let mine = MineTask::new(below, out, local, global);
            let fall = FallBucketTask::default();
            let mut compound = CompoundTask::default();
            compound.add(mine).add(fall);
            actions.schedule(compound);
        }
        "drop" => {
            local.inventory.drop_hotbar(out);
        }
        "goto" => {
            // if let [id] = args {
            //     let id: u32 = id.parse().unwrap();
            //     let kind = BlockKind::from(id);
            //
            //     let loc = BlockLocation::from(local.physics.location());
            //
            //     let closest = global.blocks.closest(loc, usize::MAX, |state| state.kind()
            // == kind);
            //
            //     if let Some(closest) = closest {
            //         actions.schedule(BlockTravelTask::new(closest, local));
            //     } else {
            //         msg!("There is no block {} by me", id);
            //     }
            // }

            if let [a, b, c] = args {
                let x = a.parse()?;
                let y = b.parse()?;
                let z = c.parse()?;
                let dest = BlockLocation::new(x, y, z);
                actions.schedule(BlockTravelTask::new(dest, local));
            }
        }
        "stop" => {
            actions.task = None;
        }
        "loc" => {
            msg!(
                "My location is {} in {}",
                local.physics.location(),
                local.dimension
            );
        }
        "state" => {
            if let [name] = args {
                if name == &local.info.username {
                    msg!("location {}", local.physics.location());
                    msg!("on ground {}", local.physics.on_ground());
                    let below_loc =
                        BlockLocation::from(local.physics.location() - Displacement::EPSILON_Y);
                    msg!("below kind {:?}", global.blocks.get_block_kind(below_loc));
                    msg!("inventory slots {:?}", local.inventory.hotbar());
                }
            }
        }
        "get" => {
            if let [a, b, c] = args {
                let x = a.parse()?;
                let y = b.parse()?;
                let z = c.parse()?;
                let location = BlockLocation::new(x, y, z);

                msg!("The block is {:?}", global.blocks.get_block(location));
            }
        }
        "place" => {
            if let [a, b, c] = args {
                let x = a.parse()?;
                let y = b.parse()?;
                let z = c.parse()?;

                let origin = local.physics.location() + Displacement::EYE_HEIGHT;

                let location = BlockLocation::new(x, y, z);
                let faces = location.faces();
                let best_loc_idx = IntoIterator::into_iter(faces)
                    .position_min_by_key(|loc| FloatOrd(loc.dist2(origin)))
                    .unwrap();

                local.physics.look_at(faces[best_loc_idx]);
                out.use_item();
                out.place_block(location, Face::from(best_loc_idx as u8));
            }
        }
        // "mine" => {
        //     let origin = local.physics.location() + Displacement::EYE_HEIGHT;
        //
        //     let closest = global.blocks.closest_in_chunk(origin.into(), |state|
        // state.kind().mineable(&global.block_data));
        //
        //     if let Some(closest) = closest {
        //         let mine_task = MineTask::new(closest, local, global);
        //         actions.schedule(mine_task);
        //     }
        // }
        _ => {}
    }

    Ok(())
}

pub fn run_threaded(
    _: &rayon::Scope,
    local: &mut LocalState,
    actions: &mut ActionState,
    global: &GlobalState,
    end_by: Instant,
) {
    if let Some(task) = actions.task.as_mut() {
        task.expensive(end_by, local, global);
    }
}
