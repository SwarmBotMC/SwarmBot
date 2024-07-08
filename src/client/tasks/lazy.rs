use std::time::Instant;

use crate::{
    client::{
        state::{global::GlobalState, local::LocalState},
        tasks::Task,
    },
    protocol::InterfaceOut,
};

pub struct LazyTask<T: Lazy> {
    inner: Option<Box<dyn Task>>,
    create_task: Option<T>,
}

pub trait Lazy {
    fn create(&self, local: &mut LocalState, global: &GlobalState) -> Box<dyn Task>;
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
    fn get(&mut self, local: &mut LocalState, global: &GlobalState) -> &mut dyn Task {
        if self.inner.is_none() {
            let f = self.create_task.take().unwrap();
            let task = f.create(local, global);
            self.inner = Some(task);
        }

        self.inner.as_deref_mut().unwrap()
    }
}

impl<T: Lazy + Send> Task for LazyTask<T> {
    fn tick(
        &mut self,
        out: &mut dyn InterfaceOut,
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
