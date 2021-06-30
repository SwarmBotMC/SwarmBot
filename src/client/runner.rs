/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::TryRecvError;
use std::time::{Duration, Instant};
use tokio::sync::Notify;

use crate::bootstrap::blocks::BlockData;
use crate::bootstrap::Connection;
use crate::client::bot::{Bot, process_command, run_threaded};
use crate::client::physics::Physics;
use crate::client::processor::SimpleInterfaceIn;
use crate::client::state::global::GlobalState;
use crate::client::state::inventory::Inventory;
use crate::client::state::local::LocalState;
use crate::protocol::{EventQueue, Login, Minecraft};
use crate::types::Dimension;
use crate::term::Term;

struct SyncGlobal(*const GlobalState);

struct SyncLocal(*mut LocalState);

unsafe impl Sync for SyncGlobal {}

unsafe impl Send for SyncGlobal {}

unsafe impl Sync for SyncLocal {}

unsafe impl Send for SyncLocal {}


pub type Logins<T> = Rc<RefCell<Vec<Login<<T as Minecraft>::Queue, <T as Minecraft>::Interface>>>>;

/// Runs the game loop and holds all bots.
pub struct Runner<T: Minecraft> {
    /// logins that are about to be established
    pending_logins: Logins<T>,

    /// the global state of the program containing chunks and global config
    global_state: GlobalState,

    term: Term,

    /// the bots created by pending logins
    bots: Vec<Bot<T::Queue, T::Interface>>,

    id_on: u32,
}

/// Runner launch options
pub struct RunnerOptions {
    pub delay_millis: u64,
}

impl<T: Minecraft + 'static> Runner<T> {
    /// Start the runner process
    pub async fn run(connections: tokio::sync::mpsc::Receiver<Connection>, opts: RunnerOptions) {
        let mut runner = Runner::<T>::init(connections, opts);
        runner.game_loop().await;
    }


    fn init(mut connections: tokio::sync::mpsc::Receiver<Connection>, opts: RunnerOptions) -> Runner<T> {
        let RunnerOptions { delay_millis } = opts;
        let pending_logins = Rc::new(RefCell::new(Vec::new()));
        // let handles = Rc::new(RefCell::new(Vec::new()));

        {
            let pending_logins = pending_logins.clone();
            tokio::task::spawn_local(async move {
                while let Some(connection) = connections.recv().await {
                    let logins = pending_logins.clone();
                    tokio::task::spawn_local(async move {
                        println!("starting login of {}", connection.user.username);
                        let login = match T::login(connection).await {
                            Ok(res) => res,
                            Err(err) => {
                                println!("Error logging in {}", err);
                                return;
                            }
                        };
                        logins.borrow_mut().push(login);
                    });

                    tokio::time::sleep(Duration::from_millis(delay_millis)).await;
                }
            });
        }

        Runner {
            pending_logins,
            global_state: GlobalState::init(),
            term: Term::init(),
            bots: Vec::new(),
            id_on: 0,
        }
    }


    pub async fn game_loop(&mut self) {
        let mut previous_goal = Instant::now();
        loop {
            let end_by = previous_goal + Duration::from_millis(50);
            self.game_iter(end_by).await;
            tokio::time::sleep_until(tokio::time::Instant::from_std(end_by)).await;
            let now = Instant::now();
            let difference = now - end_by;
            let millis_off = difference.as_millis();
            if millis_off > 100 {
                println!("off by {}ms", millis_off);
            }
            previous_goal = end_by;
        }
    }

    async fn game_iter(&mut self, end_by: Instant) {
        let old_count = self.bots.len();
        // first step: removing disconnected clients
        {
            self.bots.retain(|client| !client.state.disconnected);
        }

        // second step: turning pending logins into clients
        {
            let mut logins = self.pending_logins.borrow_mut();

            // TODO: why couldnt use iter
            for login in logins.drain(..) {
                let Login { queue, out, info } = login;

                let client = Bot {
                    state: LocalState::new(self.id_on, info),
                    queue,
                    out,
                };
                self.id_on += 1;
                self.bots.push(client);
            }
        }


        let new_count = self.bots.len();

        if new_count != old_count {
            println!("{} clients", new_count);
        }

        // third step: process commands
        self.process_commands();

        // fourth step: process packets from game loop
        for bot in &mut self.bots {
            // protocol-specific logic

            let mut processor = SimpleInterfaceIn::new(&mut bot.state, &mut self.global_state, &mut bot.out, &self.term);
            bot.queue.flush(&mut processor);

            // fifth step: general sync logic that isn't dependent on protocol implementation
            bot.run_sync(&mut self.global_state);
        }

        // sixth step: run threaded
        let thread_loop_end = Notify::new();

        {
            let global_state_sync = SyncGlobal(&self.global_state);
            let states_sync: Vec<_> = self.bots.iter_mut()
                .map(|bot| &mut bot.state)
                .map(|state| state as *mut LocalState)
                .map(SyncLocal)
                .collect();

            rayon::spawn(move || {
                let global_state = unsafe { &*global_state_sync.0 };
                let states_sync = states_sync;
                rayon::scope(|s| {
                    for state_sync in states_sync {
                        let state = unsafe { &mut *state_sync.0 };
                        s.spawn(move |inner_scope| {
                            run_threaded(inner_scope, state, global_state, end_by);
                        });
                    }
                });

                thread_loop_end.notify_one();
            });
        }
    }

    fn process_commands(&mut self) {
        match self.term.input.try_recv() {
            Ok(command) => {
                let parts: Vec<_> = command.split(' ').collect();

                if parts.is_empty() {
                    return;
                }
                let name = parts[0];
                let args = &parts[1..];

                for bot in &mut self.bots {
                    process_command(name, args, &mut bot.state, &mut self.global_state, &mut bot.out, &self.term);
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(_e) => {
                println!("receive err");
            }
        }
    }
}
