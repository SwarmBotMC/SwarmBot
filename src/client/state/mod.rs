use std::rc::Rc;

use tokio::sync::Notify;

use crate::client::follow::Follower;
use crate::client::instance::{ClientInfo, TravelProblem};
use crate::client::pathfind::context::{Costs, MoveContext};
use crate::client::pathfind::incremental::AStar;
use crate::client::pathfind::progress_checker::{NoVehicleGoalCheck, NoVehicleHeuristic};
use crate::client::state::inventory::Inventory;
use crate::storage::block::{BlockLocation, BlockState};
use crate::types::Location;
use std::collections::{HashMap, HashSet};

pub mod inventory;
pub mod global;
pub mod local;

pub enum Dimension {
    Overworld,
    Nether,
    End,
}
