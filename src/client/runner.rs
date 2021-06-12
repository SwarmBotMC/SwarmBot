use std::cell::RefCell;
use std::rc::Rc;

use tokio::task::JoinHandle;

use crate::bootstrap::Connection;
use crate::client::instance::{Client, State};
use crate::protocol::{Login, McProtocol};
use crate::storage::world::WorldBlocks;
use rayon::Scope;

pub struct Runner<'a, T: McProtocol> {
    pending_logins: Rc<RefCell<Vec<Login<T>>>>,
    world_blocks: &'a WorldBlocks,
    clients: Vec<Client<'a, T>>,
    handles: Vec<JoinHandle<()>>,
}

impl<'a, T: McProtocol> Drop for Runner<'a, T> {
    fn drop(&mut self) {
        for handle in &self.handles {
            handle.abort();
        }
    }
}

impl<'a, T: McProtocol + 'static> Runner<'a, T> {
    pub async fn run(connections: Vec<Connection>) -> ! {
        let blocks = WorldBlocks::default();
        let mut runner = Runner::<T>::new(connections, &blocks);
        runner.game_loop().await
    }


    fn new(connections: Vec<Connection>, world_blocks: &WorldBlocks) -> Runner<T> {
        let conn_len = connections.len();
        let logins = Rc::new(RefCell::new(Vec::with_capacity(conn_len)));
        let mut handles = Vec::with_capacity(conn_len);

        for connection in connections {
            let logins = logins.clone();
            let handle = tokio::task::spawn_local(async move {
                println!("starting login of {}", connection.user.email);
                let login = T::login(connection).await.unwrap();
                logins.borrow_mut().push(login);
            });
            handles.push(handle);
        }

        Runner {
            pending_logins: logins,
            handles,
            world_blocks,
            clients: Vec::with_capacity(conn_len),
        }
    }


    pub async fn game_loop(&mut self) -> ! {
        loop {
            self.game_iter();
            // TODO: delay 50 ms
            tokio::time::sleep(core::time::Duration::from_millis(50)).await;
        }
    }

    fn game_iter(&mut self) {

        // first step: turning pending logins into clients
        {
            let mut logins = self.pending_logins.borrow_mut();

            // TODO: why couldnt use iter
            for login in logins.drain(..){
                let Login { protocol, info } = login;

                let client = Client {
                    info,
                    state: Default::default(),
                    protocol,
                    world_blocks: self.world_blocks,
                };
                self.clients.push(client);
            }

        }

        // process packets from game loop
        {
            for client in &mut self.clients {
                client.protocol.apply_packets(&mut client.state)
            }
        }

        {
            let states: Vec<_> = self.clients.iter().map(|x| &x.state).collect();

            rayon::scope(|s| {
                for state in states {
                    s.spawn(move |inner_scope| {
                        run_client(inner_scope, state);
                    });
                }
            });
        }
    }
}

fn run_client(scope: &Scope, state: &State){

}
