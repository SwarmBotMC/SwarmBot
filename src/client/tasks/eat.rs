use crate::{
    client::{
        state::{global::GlobalState, local::LocalState},
        tasks::TaskTrait,
    },
    protocol::InterfaceOut,
};

pub struct EatTask {
    ticks: usize,
}

const TICKS: usize = 40;

impl Default for EatTask {
    fn default() -> Self {
        // shouldn't need to be 40 (32... but because of lag I guess it sometimes does)
        Self { ticks: TICKS }
    }
}

impl TaskTrait for EatTask {
    fn tick(
        &mut self,
        out: &mut impl InterfaceOut,
        _: &mut LocalState,
        _: &mut GlobalState,
    ) -> bool {
        // start eating
        if self.ticks == TICKS {
            out.use_item();
        }

        if self.ticks == 0 {
            out.finish_eating();
            true
        } else {
            self.ticks -= 1;
            false
        }
    }
}
