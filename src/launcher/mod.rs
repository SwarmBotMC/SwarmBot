// use crate::connections::Connection;
// use tokio::task::JoinHandle;
// use tokio::io::AsyncReadExt;
//
// struct ConnectionPackets {
//     packets: Vec<u8>,
//     rx: std::sync::mpsc::Receiver<Packet>,
//     handle: JoinHandle<()>
//
// }
//
// impl ConnectionPackets {
//     fn new(connection: Connection) -> Self {
//
//         let mut read = connection.read;
//         let (tx,rx) = std::sync::mpsc::channel();
//         let handle = tokio::task::spawn_local(async move {
//             let x = read.read().await;
//         });
//         ConnectionPackets {
//             packets: vec![],
//             rx,
//             handle
//         }
//     }
//
//     async fn reader(&self){
//         self.rx.
//     }
// }
//
//
// struct Launcher {
//     connections: Vec<ConnectionPackets>,
// }
//
// impl Launcher {
//     fn new(connections: Vec<Connection>) -> Launcher {
//
//         let connections = connections.
//             into_iter().
//             map(|connection| ConnectionPackets::new(connection))
//             .collect();
//
//         Self {
//             connections
//         }
//     }
//
//     fn run(&self, rt: &tokio::runtime::Runtime){
//         let local = tokio::task::LocalSet::new();
//
//         local.block_on(rt, async move {
//             tokio::task::spawn_local()
//         })
//     }
// }
