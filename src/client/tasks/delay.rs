use crate::{
    client::{
        state::{global::GlobalState, local::LocalState},
        tasks::TaskTrait,
    },
    protocol::InterfaceOut,
};

pub struct DelayTask(pub u32);

impl DelayTask {
    #[allow(unused)]
    pub const fn new(ticks: u32) -> Self {
        Self(ticks)
    }
}

impl TaskTrait for DelayTask {
    fn tick(
        &mut self,
        _out: &mut impl InterfaceOut,
        _local: &mut LocalState,
        _global: &mut GlobalState,
    ) -> bool {
        let ticks_left = self.0;
        if ticks_left == 0 {
            true
        } else {
            self.0 -= 1;
            false
        }
    }
}
