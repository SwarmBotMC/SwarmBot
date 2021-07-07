/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/7/21, 12:17 AM
 */

use crate::client::tasks::stream::TaskStream;
use crate::protocol::InterfaceOut;
use crate::client::state::local::LocalState;
use crate::client::state::global::GlobalState;
use crate::client::tasks::{Task, TaskTrait};
use std::time::Instant;

pub struct LazyStream<T: TaskStream> {
    current: Option<Box<Task>>,
    create_task: T,
}

impl<T: TaskStream> From<T> for LazyStream<T> {
    fn from(create_task: T) -> Self {
        Self { create_task, current: None }
    }
}

impl<T: TaskStream> LazyStream<T> {
    fn get(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> Option<&mut Task> {
        if self.current.is_none() {
            let next = self.create_task.poll(out, local, global)?;
            self.current = Some(box next)
        }

        self.current.as_deref_mut()
    }
}

impl<T: TaskStream> TaskTrait for LazyStream<T> {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {
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
        let current = match self.current.as_mut() {
            None => return,
            Some(inner) => inner
        };
        current.expensive(end_by, local, global);
    }
}
