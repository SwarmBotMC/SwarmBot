use std::{collections::VecDeque, time::Instant};

use crate::{
    client::{
        state::{global::GlobalState, local::LocalState},
        tasks::Task,
    },
    protocol::InterfaceOut,
};

#[derive(Default)]
pub struct CompoundTask {
    tasks: VecDeque<Box<dyn Task>>,
}

impl CompoundTask {
    pub fn add<T: Task + 'static>(&mut self, task: T) -> &mut Self {
        self.tasks.push_back(Box::new(task));
        self
    }

    #[allow(unused)]
    pub fn prepend(&mut self, task: impl Task + 'static) {
        self.tasks.push_front(Box::new(task));
    }
}

impl Task for CompoundTask {
    fn tick(
        &mut self,
        out: &mut dyn InterfaceOut,
        local: &mut LocalState,
        global: &mut GlobalState,
    ) -> bool {
        // this is so we can do multiple 0-tick tasks in a gametick
        while let Some(front) = self.tasks.front_mut() {
            let finished = front.tick(out, local, global);

            if finished {
                self.tasks.pop_front();
            } else {
                return false;
            }
        }

        true
    }

    fn expensive(&mut self, end_at: Instant, local: &mut LocalState, global: &GlobalState) {
        match self.tasks.front_mut() {
            None => {}
            Some(res) => res.expensive(end_at, local, global),
        };
    }
}
