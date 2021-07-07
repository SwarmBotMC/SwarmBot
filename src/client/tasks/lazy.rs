/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/7/21, 12:19 AM
 */

use crate::client::tasks::{Task, TaskTrait};
use crate::client::state::local::LocalState;
use crate::client::state::global::GlobalState;
use crate::protocol::InterfaceOut;
use std::time::Instant;

pub struct LazyTask<T: Lazy> {
    inner: Option<Box<Task>>,
    create_task: Option<T>,
}

pub trait Lazy {
    fn create(&self, local: &mut LocalState, global: &GlobalState) -> Task;
}

impl <T: Lazy> From<T> for LazyTask<T> {
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
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {
        let task = self.get(local, global);
        task.tick(out, local, global)
    }

    fn expensive(&mut self, end_at: Instant, local: &mut LocalState, global: &GlobalState) {
        let task = self.get(local, global);
        task.expensive(end_at, local, global);
    }
}
