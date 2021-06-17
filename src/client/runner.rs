use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::time::Duration;

use rayon::Scope;
use tokio::task::JoinHandle;

use crate::bootstrap::Connection;
use crate::client::instance::{Client, State, run_threaded};
use crate::error::Error;
use crate::protocol::{Login, McProtocol};
use crate::storage::world::WorldBlocks;
use crate::types::Location;
use crate::client::pathfind::context::{GlobalContext, PathConfig};
use crate::storage::block::BlockLocation;

#[derive(Default)]
pub struct GlobalState{
    pub world_blocks: WorldBlocks,
    pub travel_config: PathConfig,
}

pub struct Runner<T: McProtocol> {
    pending_logins: Rc<RefCell<Vec<Login<T>>>>,
    global_state: GlobalState,
    clients: Vec<Client<T>>,
}

pub struct RunnerOptions {
    pub delay_millis: u64,
}

impl<T: McProtocol + 'static> Runner<T> {
    pub async fn run(connections: tokio::sync::mpsc::Receiver<Connection>, opts: RunnerOptions) {
        let blocks = WorldBlocks::default();
        let mut runner = Runner::<T>::new(connections, opts).await;
        runner.game_loop().await;
    }


    async fn new(mut connections: tokio::sync::mpsc::Receiver<Connection>, opts: RunnerOptions) -> Runner<T> {
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
            clients: Vec::new(),
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
        let old_count = self.clients.len();
        // first step: removing disconnected clients
        {
            self.clients.retain(|client| !client.protocol.disconnected());
        }

        // second step: turning pending logins into clients
        {
            let mut logins = self.pending_logins.borrow_mut();

            // TODO: why couldnt use iter
            for login in logins.drain(..) {
                let Login { protocol, info } = login;

                let client = Client {
                    state: State {
                        ticks: 0,
                        alive: true,
                        follower: None,
                        info,
                        location: Location::default(),
                        destination: BlockLocation(119, 72, 226),
                        finder_problem: None
                    },
                    protocol,
                };
                self.clients.push(client);
            }
        }


        let new_count = self.clients.len();

        if new_count != old_count {
            println!("{} clients", new_count);
        }

        // third step: process packets from game loop
        for client in &mut self.clients {
            // protocol-specific logic
            client.protocol.apply_packets(&mut client.state, &mut self.global_state);

            // general sync logic that isnt dependent on protocol implementation
            client.run_sync(&mut self.global_state);
        }

        // fourth step: run threaded
        for client in &mut self.clients {
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
