use std::cell::RefCell;
use std::rc::Rc;

use tokio::task::JoinHandle;

use crate::client::instance::ClientProtocol;
use crate::connections::Connection;
use crate::packet::McProtocol;

struct ClientRunner<T: McProtocol + Send> {
    clients: Rc<RefCell<Vec<ClientProtocol<T>>>>,
    handles: Vec<JoinHandle<()>>,
}

impl<T: McProtocol + Send + 'static> ClientRunner<T> {
    fn new(connections: Vec<Connection>) -> ClientRunner<T> {
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

        ClientRunner {
            handles,
            clients,
        }
    }

    fn game_loop(&mut self) {

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
