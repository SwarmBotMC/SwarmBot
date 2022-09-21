use std::time::Instant;

use crate::{
    client::{
        state::{global::GlobalState, local::LocalState},
        tasks::{Task, TaskTrait},
    },
    protocol::InterfaceOut,
};

pub struct LazyTask<T: Lazy> {
    inner: Option<Box<Task>>,
    create_task: Option<T>,
}

pub trait Lazy {
    fn create(&self, local: &mut LocalState, global: &GlobalState) -> Task;
}

impl<T: Lazy> From<T> for LazyTask<T> {
    fn from(block: T) -> Self {
        Self {
            inner: None,
            create_task: Some(block),
        }
    }
}

impl<T: Lazy> LazyTask<T> {
    fn get(&mut self, local: &mut LocalState, global: &GlobalState) -> &mut Task {
        if self.inner.is_none() {
            let f = self.create_task.take().unwrap();
            self.inner = Some(Box::new(f.create(local, global)));
        }

        self.inner.as_mut().unwrap()
    }
}

impl<T: Lazy> TaskTrait for LazyTask<T> {
    fn tick(
        &mut self,
        out: &mut impl InterfaceOut,
        local: &mut LocalState,
        global: &mut GlobalState,
    ) -> bool {
        let task = self.get(local, global);
        task.tick(out, local, global)
    }

    fn expensive(&mut self, end_at: Instant, local: &mut LocalState, global: &GlobalState) {
        let task = self.get(local, global);
        task.expensive(end_at, local, global);
    }
}
