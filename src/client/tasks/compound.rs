use std::{collections::VecDeque, time::Instant};

use crate::{
    client::{
        state::{global::GlobalState, local::LocalState},
        tasks::{Task, TaskTrait},
    },
    protocol::InterfaceOut,
};

#[derive(Default)]
pub struct CompoundTask {
    tasks: VecDeque<Task>,
}

impl CompoundTask {
    pub fn add<T: Into<Task>>(&mut self, task: T) -> &mut Self {
        self.tasks.push_back(task.into());
        self
    }

    pub fn prepend(&mut self, task: impl Into<Task>) {
        self.tasks.push_front(task.into());
    }
}

impl TaskTrait for CompoundTask {
    fn tick(
        &mut self,
        out: &mut impl InterfaceOut,
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
