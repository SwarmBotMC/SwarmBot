use std::time::Instant;

use bridge::BridgeTask;
use center::CenterTask;
use compound::CompoundTask;
use delay::DelayTask;
use eat::EatTask;
use fall_bucket::FallBucketTask;
use hit_entity::HitEntityTask;
use lazy::LazyTask;
use mine::MineTask;
use mine_column::MineColumnTask;
use mine_goto::GoMineTop;
use mine_layer::MineLayerTask;
use mine_region::MineRegion;
use navigate::BlockTravelNearTask;
use pillar::PillarTask;
use pillar_and_mine::PillarAndMineTask;

use crate::{
    client::{
        state::{global::GlobalState, local::LocalState},
        tasks::{
            attack_entity::AttackEntity,
            lazy_stream::LazyStream,
            navigate::{BlockTravelTask, ChunkTravelTask},
            safe_mine_coord::SafeMineRegion,
        },
    },
    protocol::InterfaceOut,
};

pub mod attack_entity;
pub mod bridge;
pub mod center;
pub mod compound;
pub mod delay;
pub mod eat;
pub mod fall_bucket;
pub mod hit_entity;
pub mod lazy;
pub mod lazy_stream;
pub mod mine;
pub mod mine_column;
pub mod mine_goto;
pub mod mine_layer;
pub mod mine_region;
pub mod navigate;
pub mod pillar;
pub mod pillar_and_mine;
pub mod safe_mine_coord;
pub mod stream;

#[enum_dispatch]
pub trait TaskTrait {
    /// return true if done
    fn tick(
        &mut self,
        out: &mut impl InterfaceOut,
        local: &mut LocalState,
        global: &mut GlobalState,
    ) -> bool;

    /// Do an expensive part of the task. This is done in a multi-threaded
    /// environment. An example of This has a default implementation of
    /// nothing. However, tasks like pathfinding use this. The task MUST end
    /// by the given {`end_by`} duration else the game loop is held up. This is
    /// called every game loop cycle so if the task hasn't finished it by
    /// {`end_by`} it should instead until this function is called again.
    fn expensive(&mut self, _end_by: Instant, _local: &mut LocalState, _global: &GlobalState) {}
}

pub type GoMineTopTask = LazyTask<GoMineTop>;
pub type MineRegionTask = LazyStream<MineRegion>;
pub type SafeMineRegionTask = LazyTask<SafeMineRegion>;
pub type AttackEntityTask = LazyStream<AttackEntity>;

#[allow(clippy::enum_variant_names)]
#[enum_dispatch(TaskTrait)]
pub enum Task {
    CompoundTask,
    AttackEntityTask,
    HitEntityTask,
    EatTask,
    MineRegionTask,
    SafeMineRegionTask,
    CenterTask,
    BridgeTask,
    GoMineTopTask,
    MineColumnTask,
    MineTask,
    BlockTravelNearTask,
    BlockTravelTask,
    ChunkTravelTask,
    PillarTask,
    DelayTask,
    PillarAndMineTask,
    MineLayerTask,
    FallBucketTask,
}
