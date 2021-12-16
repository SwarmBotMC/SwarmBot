// Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::{
    cell::RefCell,
    default::default,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::Notify;

use crate::{
    bootstrap::Connection,
    client::{
        bot::{run_threaded, ActionState, Bot},
        commands::{Command, Commands, Selection2D},
        processor::SimpleInterfaceIn,
        state::{
            global::{mine_alloc::MinePreference, GlobalState},
            local::LocalState,
        },
        tasks::{
            attack_entity::AttackEntity, lazy_stream::LazyStream, mine_region::MineRegion,
            navigate::BlockTravelTask,
        },
    },
};

use crate::{
    error::{Res, ResBox},
    protocol::{EventQueue, Login, Minecraft},
};

struct SyncGlobal(*const GlobalState);

struct SyncLocal((*mut LocalState, *mut ActionState));

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

    commands: Commands,

    /// the bots created by pending logins
    bots: Vec<Bot<T::Queue, T::Interface>>,

    /// An id counter that increases for each bot. Used as a unique identifier.
    id_on: u32,
}

/// Runner launch options
pub struct RunnerOptions {
    /// The amount of milliseconds to wait between logging in successive users
    pub delay_millis: u64,
    pub ws_port: u16,
}

impl<T: Minecraft + 'static> Runner<T> {
    /// Start the runner process
    pub async fn run(
        connections: tokio::sync::mpsc::Receiver<Connection>,
        opts: RunnerOptions,
    ) -> Res {
        let mut runner = Runner::<T>::init(connections, opts).await?;
        runner.game_loop().await;
        Ok(())
    }

    /// Initialize the runner. Go through the handshake process for each
    /// [`Connection`]
    async fn init(
        mut connections: tokio::sync::mpsc::Receiver<Connection>,
        opts: RunnerOptions,
    ) -> Res<Runner<T>> {
        let commands = Commands::init(opts.ws_port).await?;

        let RunnerOptions { delay_millis, .. } = opts;
        let pending_logins = Rc::new(RefCell::new(Vec::new()));

        {
            let pending_logins = pending_logins.clone();

            // login task for all users
            tokio::task::spawn_local(async move {
                while let Some(connection) = connections.recv().await {
                    let logins = pending_logins.clone();

                    // login task for an individual user
                    tokio::task::spawn_local(async move {
                        println!("Starting login of {}", connection.user.username);
                        let username = connection.user.username.clone();
                        let login = match T::login(connection).await {
                            Ok(res) => {
                                println!("Finished logging in {}", username);
                                res
                            }
                            Err(err) => {
                                println!("Error logging in {} -- {}", username, err);
                                return;
                            }
                        };
                        logins.borrow_mut().push(login);
                    });

                    // if we want a delay between logging in
                    tokio::time::sleep(Duration::from_millis(delay_millis)).await;
                }
            });
        }

        Ok(Runner {
            pending_logins,
            global_state: GlobalState::init(),
            commands,
            bots: Vec::new(),
            id_on: 0,
        })
    }

    pub async fn game_loop(&mut self) {
        let mut previous_goal = Instant::now();

        // a game loop repeating every 50 ms
        loop {
            let end_by = previous_goal + Duration::from_millis(50);
            self.game_iter(end_by).await;
            tokio::time::sleep_until(tokio::time::Instant::from_std(end_by)).await;
            let now = Instant::now();
            let difference = now - end_by;
            let millis_off = difference.as_millis();

            // log if we are wayyyy off
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

            for login in logins.drain(..) {
                let Login { queue, out, info } = login;

                let client = Bot {
                    state: LocalState::new(self.id_on, info),
                    actions: default(),
                    queue,
                    out,
                };
                self.id_on += 1;
                self.bots.push(client);
            }
        }

        let new_count = self.bots.len();

        // log clients if they have changed
        if new_count != old_count {
            println!("{} clients", new_count);
        }

        // process pending commands (from forge mod)
        while let Ok(command) = self.commands.pending.try_recv() {
            if let Err(err) = self.process_command(command) {
                println!("Error processing command: {}", err)
            }
        }

        // fourth step: process packets from game loop
        for bot in &mut self.bots {
            let mut processor = SimpleInterfaceIn::new(
                &mut bot.state,
                &mut bot.actions,
                &mut self.global_state,
                &mut bot.out,
            );

            // protocol-specific logic. Translates input packets and sends to processor
            bot.queue.flush(&mut processor);

            // fifth step: general sync logic that isn't dependent on protocol
            // implementation
            bot.run_sync(&mut self.global_state);
        }

        // sixth step: run multi-threaded environment for the rest of the game loop.
        // GlobalState will be read-only and LocalState will be mutable
        let thread_loop_end = Arc::new(Notify::new());

        {
            let thread_loop_end = thread_loop_end.clone();

            // We have to do unsafe stuff here because Rust requires a 'static lifetime for
            // threads. However, in this case we know that the thread (task)
            // will stop by the end of this function, so we can coerce the
            // lifetimes of &GlobalState and &mut LocalState to be 'static. This is overall
            // pretty safe as it still requires the states to be Send+Sync, so
            // it is hard to make errors.
            let global_state_sync = SyncGlobal(&self.global_state);
            let states_sync: Vec<_> = self
                .bots
                .iter_mut()
                .map(|bot| (&mut bot.state, &mut bot.actions))
                .map(|(state, actions)| (state as *mut LocalState, actions as *mut ActionState))
                .map(SyncLocal)
                .collect();

            rayon::spawn(move || {
                let global_state = unsafe { &*global_state_sync.0 };
                let states_sync = states_sync;
                rayon::scope(|s| {
                    for state_sync in states_sync {
                        let (state, actions) = state_sync.0;
                        let (state, actions) = unsafe { (&mut *state, &mut *actions) };

                        s.spawn(move |inner_scope| {
                            run_threaded(inner_scope, state, actions, global_state, end_by);
                        });
                    }
                });

                // when all tasks are finished allow us to go to the beginning of the loop and
                // mutate GlobalState again
                thread_loop_end.notify_one();
            });
        }

        // wait until all threaded activities have finished
        thread_loop_end.notified().await;
    }

    fn process_command(&mut self, command: Command) -> ResBox {
        let global = &mut self.global_state;
        let bots = &mut self.bots;

        match command {
            Command::Mine(mine) => {
                let Selection2D { from, to } = mine.sel.normalize();
                global.mine.mine(from, to, Some(MinePreference::FromDist));

                for bot in bots {
                    bot.actions.schedule(LazyStream::from(MineRegion))
                }
            }
            Command::GoTo(goto) => {
                for bot in bots {
                    bot.actions
                        .schedule(BlockTravelTask::new(goto.location, &bot.state));
                }
            }
            Command::Attack(attack) => {
                let player = self
                    .global_state
                    .players
                    .by_name(&attack.name)
                    .ok_or("player does not exist")?;
                let entity_id = self
                    .global_state
                    .entities
                    .by_player_uuid(player.uuid)
                    .ok_or("could not find entity id for player")?;

                for bot in bots {
                    let task = LazyStream::from(AttackEntity::new(entity_id));
                    bot.actions.schedule(task)
                }
            }
        }

        Ok(())
    }
}
