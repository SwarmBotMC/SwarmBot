use interfaces::types::{BlockLocation, SimpleType};

use crate::{
    client::{
        state::{global::GlobalState, local::LocalState},
        tasks::TaskTrait,
    },
    protocol::InterfaceOut,
    types::{Direction, Displacement},
};

pub struct PillarTask {
    dest_y: u32,
}

impl PillarTask {
    pub fn new(dest_y: u32) -> Self {
        println!("pillar dest {dest_y}");
        Self { dest_y }
    }
}

impl TaskTrait for PillarTask {
    fn tick(
        &mut self,
        out: &mut impl InterfaceOut,
        local: &mut LocalState,
        global: &mut GlobalState,
    ) -> bool {
        local.inventory.switch_block(out);

        // equal OR GREATER because we don't want to pillar if we are higher than we
        // need to be
        if u32::try_from(local.physics.location().y as i64).unwrap_or_default() >= self.dest_y {
            let below_loc = BlockLocation::from(local.physics.location()).below();

            // return true if block below us is solid
            if global.blocks.get_block_simple(below_loc) == Some(SimpleType::Solid) {
                return true;
            }
        }

        local.physics.jump();

        local.physics.look(Direction::DOWN);

        // subtract a little so we can be conservative with placements
        let location = local.physics.location() - Displacement::new(0., 0.1, 0.);

        let below_block = BlockLocation::from(location).below();
        let two_below = below_block.below();

        let below_type = global.blocks.get_block_simple(below_block);
        let below_valid = matches!(
            below_type,
            Some(SimpleType::Water | SimpleType::WalkThrough)
        );

        let two_below_valid = matches!(
            global.blocks.get_block_simple(two_below),
            Some(SimpleType::Solid)
        );

        if below_valid && two_below_valid {
            let below = BlockLocation::from(local.physics.location()).below();
            let against = below.below();
            if global.blocks.get_block_simple(against) == Some(SimpleType::Solid) {
                local.physics.place_hand(against);
            }
        }

        false
    }
}
