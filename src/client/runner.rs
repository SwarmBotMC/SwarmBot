use std::cell::RefCell;

use std::rc::Rc;
use std::time::Duration;




use crate::bootstrap::Connection;
use crate::client::bot::{Bot, run_threaded};

use crate::protocol::{Login, Minecraft, EventQueue};
use crate::storage::blocks::WorldBlocks;
use crate::types::Location;
use crate::client::pathfind::context::{Costs};
use crate::storage::block::BlockLocation;
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::client::state::Dimension;
use crate::client::state::inventory::Inventory;
use crate::client::processor::SimpleInterfaceIn;
use crate::client::physics::Physics;


/// Runs the game loop and holds all bots.
pub struct Runner<T: Minecraft> {

    /// logins that are about to be established
    pending_logins: Rc<RefCell<Vec<Login<T::Queue, T::Interface>>>>,

    /// the global state of the program containing chunks and global config
    global_state: GlobalState,

    /// the bots created by pending logins
    bots: Vec<Bot<T::Queue, T::Interface>>,

    id_on: u32
}

/// Runner launch options
pub struct RunnerOptions {
    pub delay_millis: u64,
}

impl<T: Minecraft + 'static> Runner<T> {

    /// Start the runner process
    pub async fn run(connections: tokio::sync::mpsc::Receiver<Connection>, opts: RunnerOptions) {
        let _blocks = WorldBlocks::default();
        let mut runner = Runner::<T>::init(connections, opts);
        runner.game_loop().await;
    }


    fn init(mut connections: tokio::sync::mpsc::Receiver<Connection>, opts: RunnerOptions) -> Runner<T> {
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

                    tokio::time::sleep(Duration::from_millis(opts.delay_millis)).await;
                }
            });
        }


        Runner {
            pending_logins,
            global_state: GlobalState::default(),
            bots: Vec::new(),
            id_on: 0
        }
    }


    pub async fn game_loop(&mut self) {
        loop {
            self.game_iter();
            // TODO: delay 50 ms
            tokio::time::sleep(core::time::Duration::from_millis(50)).await;
        }
    }

    fn game_iter(&mut self) {
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
                    state: LocalState {
                        ticks: 0,
                        bot_id: self.id_on,
                        physics: Physics::default(),
                        disconnected: false,
                        inventory: Inventory {},
                        alive: true,
                        dimension: Dimension::Overworld,
                        follower: None,
                        info,
                        destination: BlockLocation(119, 72, 226),
                        travel_problem: None,
                        costs: Costs {
                            block_walk: 1.0,
                            ascend: 1.0,
                            fall: 1.0,
                            block_place: 1.0
                        }
                    },
                    queue,
                    out
                };
                self.id_on += 1;
                self.bots.push(client);
            }
        }


        let new_count = self.bots.len();

        if new_count != old_count {
            println!("{} clients", new_count);
        }

        // third step: process packets from game loop
        for bot in &mut self.bots {
            // protocol-specific logic

            let mut processor = SimpleInterfaceIn::new(&mut bot.state, &mut self.global_state, &mut bot.out);
            bot.queue.flush(&mut processor);

            // general sync logic that isn't dependent on protocol implementation
            bot.run_sync(&mut self.global_state);
        }

        // fourth step: run threaded
        for client in &mut self.bots {
            run_threaded(&mut client.state, &self.global_state);
        }


        // {
        //     let global_state = &self.global_state;
        //     let states: Vec<_> = self.clients.iter_mut().map(|x| &mut x.state).collect();
        //
        //     rayon::scope(|s| {
        //         for state in states {
        //             s.spawn(move |inner_scope| {
        //                 run_client(inner_scope, state, global_state);
        //             });
        //         }
        //     });
        // }
    }
}
