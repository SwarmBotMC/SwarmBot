use interfaces::types::{BlockLocation, BlockState};

use crate::{
    client::{
        state::{global::GlobalState, local::LocalState},
        tasks::TaskTrait,
    },
    protocol::{Face, InterfaceOut, Mine},
};

pub struct MineTask {
    ticks: usize,
    first: bool,
    location: BlockLocation,
    face: Face,
}

impl MineTask {
    pub fn new(
        location: BlockLocation,
        out: &mut impl InterfaceOut,
        local: &mut LocalState,
        global: &GlobalState,
    ) -> Self {
        let kind = global.blocks.get_block_kind(location).unwrap();

        let tool = local.inventory.switch_tool(kind, &global.block_data, out);

        let ticks = tool.wait_time(kind, false, true, &global.block_data) + 1;

        Self {
            ticks,
            location,
            face: Face::PosY,
            first: true,
        }
    }

    pub fn set_face(&mut self, face: Face) {
        self.face = face;
    }
}

impl TaskTrait for MineTask {
    fn tick(
        &mut self,
        out: &mut impl InterfaceOut,
        local: &mut LocalState,
        global: &mut GlobalState,
    ) -> bool {
        let look_loc = self.location.faces()[self.face as usize];
        local.physics.look_at(look_loc);

        if self.first {
            out.swing_arm();
            self.first = false;
            out.mine(self.location, Mine::Start, self.face);
        }

        // println!("mining {}", self.location);

        out.swing_arm();
        if self.ticks == 0 {
            out.mine(self.location, Mine::Finished, self.face);
            global.blocks.set_block(self.location, BlockState::AIR);
            true
        } else {
            self.ticks -= 1;
            false
        }
    }
}
