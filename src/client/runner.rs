use std::cell::RefCell;
use std::rc::Rc;

use rayon::Scope;
use tokio::task::JoinHandle;

use crate::bootstrap::Connection;
use crate::client::instance::{Client, State};
use crate::protocol::{Login, McProtocol};
use crate::storage::world::WorldBlocks;
use std::marker::PhantomData;
use crate::error::Error;
use std::time::Duration;


#[derive(Default)]
pub struct GlobalState {
    world_blocks: WorldBlocks
}

pub struct Runner<T: McProtocol> {
    pending_logins: Rc<RefCell<Vec<Login<T>>>>,
    global_state: GlobalState,
    clients: Vec<Client<T>>,
    // handles: Rc<RefCell<Vec<JoinHandle<()>>>>
}

// impl<T: McProtocol> Drop for Runner<T> {
//     fn drop(&mut self) {
//         for handle in &self.handles {
//             handle.abort();
//         }
//     }
// }

impl<T: McProtocol + 'static> Runner<T> {

    pub async fn run(connections: tokio::sync::mpsc::UnboundedReceiver<Connection>) {
        let blocks = WorldBlocks::default();
        let mut runner = Runner::<T>::new(connections).await;
        runner.game_loop().await;
    }


    async fn new(mut connections: tokio::sync::mpsc::UnboundedReceiver<Connection>) -> Runner<T> {

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
                    info,
                    state: Default::default(),
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
        {
            for client in &mut self.clients {
                client.protocol.apply_packets(&mut client.state, &mut self.global_state)
            }
        }

        {
            let global_state = &self.global_state;
            let states: Vec<_> = self.clients.iter_mut().map(|x| &mut x.state).collect();

            rayon::scope(|s| {
                for state in states {
                    s.spawn(move |inner_scope| {
                        run_client(inner_scope, state, global_state);
                    });
                }
            });
        }
    }
}

fn run_client(scope: &Scope, state: &mut State, global: &GlobalState) {}
