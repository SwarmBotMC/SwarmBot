use std::time::Instant;

use crate::{
    client::{
        state::{global::GlobalState, local::LocalState},
        tasks::{stream::TaskStream, Task},
    },
    protocol::InterfaceOut,
};

pub struct LazyStream<T: TaskStream> {
    current: Option<Box<dyn Task>>,
    create_task: T,
}

impl<T: TaskStream> From<T> for LazyStream<T> {
    fn from(create_task: T) -> Self {
        Self {
            create_task,
            current: None,
        }
    }
}

impl<T: TaskStream> LazyStream<T> {
    fn get(
        &mut self,
        out: &mut dyn InterfaceOut,
        local: &mut LocalState,
        global: &mut GlobalState,
    ) -> Option<&mut Box<dyn Task>> {
        if self.current.is_none() {
            let next = self.create_task.poll(out, local, global)?;
            self.current = Some(next);
        }

        self.current.as_mut()
    }
}

impl<T: TaskStream + Send> Task for LazyStream<T> {
    fn tick(
        &mut self,
        out: &mut dyn InterfaceOut,
        local: &mut LocalState,
        global: &mut GlobalState,
    ) -> bool {
        while let Some(task) = self.get(out, local, global) {
            let finished_subtask: bool = task.tick(out, local, global);
            if finished_subtask {
                self.current = None;
            } else {
                return false;
            }
        }

        true
    }

    fn expensive(&mut self, end_by: Instant, local: &mut LocalState, global: &GlobalState) {
        let Some(current) = self.current.as_mut() else {
            return;
        };
        current.expensive(end_by, local, global);
    }
}
