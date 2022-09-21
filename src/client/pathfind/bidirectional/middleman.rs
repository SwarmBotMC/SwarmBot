use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub enum Msg<T: Debug> {
    Node(T),
    Finished { forward: bool },
}


pub struct Middleman<T: Debug> {
    pub node_sender: tokio::sync::mpsc::Sender<Msg<T>>,
    pub vec_receiver: tokio::sync::oneshot::Receiver<Option<T>>,
    handle: JoinHandle<()>,
}

impl<T: Eq + Hash + Copy + Clone + Send + 'static + Debug> Middleman<T> {
    pub fn new() -> Middleman<T> {
        let (send_node, mut receive_node) = tokio::sync::mpsc::channel(32);
        let (send_vec, receive_vec) = tokio::sync::oneshot::channel();


        let handle = tokio::spawn(async move {
            let mut traversed_set = HashSet::new();

            while let Some(elem) = receive_node.recv().await {
                match elem {
                    Msg::Node(elem) => {
                        let was_empty = traversed_set.insert(elem);
                        if !was_empty {
                            send_vec.send(Some(elem)).expect("expected could send");
                            return;
                        }
                    }
                    Msg::Finished { forward } => {
                        if forward {
                            send_vec.send(None).expect("expected could send");
                            return;
                        }
                    }
                }
            }
        });

        Middleman {
            node_sender: send_node,
            vec_receiver: receive_vec,
            handle,
        }
    }

    pub async fn get_split(self) -> Option<T> {
        match self.vec_receiver.await {
            Ok(Some(split_point)) => Some(split_point),
            _ => None
        }
    }
}
