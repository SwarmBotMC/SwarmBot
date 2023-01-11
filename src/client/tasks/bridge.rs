use interfaces::types::BlockLocation;

use crate::{
    client::{
        pathfind::moves::CardinalDirection,
        physics::{speed::Speed, Line},
        state::{global::GlobalState, local::LocalState},
        tasks::TaskTrait,
    },
    protocol::{Face, InterfaceOut},
    types::{Direction, Displacement},
};

pub struct BridgeTask {
    count: u32,
    place_against: BlockLocation,
    direction: CardinalDirection,
}

impl BridgeTask {
    #[allow(unused)]
    pub fn new(count: u32, direction: CardinalDirection, local: &LocalState) -> Self {
        let start = BlockLocation::from(local.physics.location()).below();
        Self {
            count,
            place_against: start,
            direction,
        }
    }
}

impl TaskTrait for BridgeTask {
    fn tick(
        &mut self,
        _out: &mut impl InterfaceOut,
        local: &mut LocalState,
        _global: &mut GlobalState,
    ) -> bool {
        let displacement = Displacement::from(self.direction.unit_change());

        let direction = Direction::from(-displacement);

        local.physics.look(direction);
        local.physics.line(Line::Backward);
        local.physics.speed(Speed::WALK);

        let target_loc = self.place_against.true_center();
        let current_loc = local.physics.location();

        let place = match self.direction {
            CardinalDirection::North => target_loc.x - current_loc.x < (-0.6),
            CardinalDirection::South => target_loc.x - current_loc.x > (-0.4 + 0.5),
            CardinalDirection::West => target_loc.z - current_loc.z > (0.4 - 0.5),
            CardinalDirection::East => target_loc.z + current_loc.z > (0.4 + 0.5),
        };

        if place {
            let face = Face::from(self.direction);
            local.physics.place_hand_face(self.place_against, face);
            let change = BlockLocation::from(self.direction.unit_change());
            self.place_against = self.place_against + change;
            self.count -= 1;
        }

        self.count == 0
    }
}
