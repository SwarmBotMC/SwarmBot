use std::cell::RefCell;
use std::rc::Rc;

use tokio::task::JoinHandle;

use crate::bootstrap::Connection;
use crate::protocol::{McProtocol, ClientProtocol};

pub struct Runner<T: McProtocol> {
    clients: Rc<RefCell<Vec<ClientProtocol<T>>>>,
    handles: Vec<JoinHandle<()>>,
}

impl<T: McProtocol> Drop for Runner<T> {
    fn drop(&mut self) {
        for handle in &self.handles {
            handle.abort();
        }
    }
}

impl<T: McProtocol + 'static> Runner<T> {
    pub async fn run(connections: Vec<Connection>) -> ! {
        let mut runner = Runner::<T>::new(connections);
        runner.game_loop().await
    }


    fn new(connections: Vec<Connection>) -> Runner<T> {
        let conn_len = connections.len();
        let clients = Rc::new(RefCell::new(Vec::with_capacity(conn_len)));
        let mut handles = Vec::with_capacity(conn_len);

        for connection in connections {
            let clients = clients.clone();
            let handle = tokio::task::spawn_local(async move {
                let res = ClientProtocol::login(connection).await.unwrap();
                clients.borrow_mut().push(res);
            });
            handles.push(handle);
        }

        Runner {
            handles,
            clients,
        }
    }


    pub async fn game_loop(&mut self) -> ! {
        loop {
            self.game_iter();
        }
    }

    fn game_iter(&mut self) {

        // process packets from game loop
        {
            let mut clients = self.clients.borrow_mut();
            for cp in clients.iter_mut() {
                cp.protocol.apply_packets(&mut cp.client)
            }
        }

        {
            let clients = self.clients.borrow();
            let states: Vec<_> = clients.iter().map(|x| &x.client).collect();

            rayon::scope(|s| {
                for client in states {
                    s.spawn(move |inner_scope| {
                        client.run(inner_scope);
                    });
                }
            });
        }
    }
}
