use std::alloc::Global;
use std::collections::HashMap;

use crate::data::{AIR, BlockState, Chunk};
use crate::packet::chunk::ChunkDataPkt;
use crate::pathfind::BlockLocation;
use crate::pathfind::moves::{CardinalDirection3D, Change};
use crate::utils::oneshot;

#[derive(Debug)]
enum Msg {
    GetBlock {
        loc: BlockLocation,
        resp: tokio::sync::oneshot::Sender<Option<BlockState>>,
    },
    GetBlocks {
        locs: Vec<BlockLocation>,
        resp: tokio::sync::oneshot::Sender<Vec<Option<BlockState>>>,
    },
    SetBlock {
        loc: BlockLocation,
        value: BlockState,
    },
    SetChunk {
        x: i32,
        z: i32,
        value: Chunk,
    },
}

#[derive(Clone)]
pub struct WorldBlocks {
    tx: tokio::sync::mpsc::UnboundedSender<Msg>,
}

impl Default for WorldBlocks {
    fn default() -> WorldBlocks {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(async move {
            let mut map: HashMap<_, Chunk> = HashMap::new();

            'msg_loop: while let Some(msg) = rx.recv().await {
                match msg {
                    Msg::GetBlock { loc, resp } => {
                        let mut to_send = || {
                            let BlockLocation(x, y, z) = loc;
                            let cx = (x >> 4) as i32;
                            let cz = (z >> 4) as i32;

                            if y < 0 || y > 255 {
                                return Some(AIR);
                            }

                            let chunk = match map.get_mut(&(cx, cz)) {
                                None => {
                                    return None;
                                }
                                Some(inner) => inner,
                            };

                            let relative_x = (x - (cx << 4) as i64) as u8;
                            let relative_z = (z - (cz << 4) as i64) as u8;

                            assert!((0..16).contains(&relative_x));
                            assert!((0..16).contains(&relative_z));

                            let y = y as usize;

                            Some(chunk.get_rel_block(relative_x, y, relative_z))
                        };

                        let to_send = to_send();
                        match resp.send(to_send) {
                            Ok(x) => {}
                            Err(x) => {
                                println!("")
                            }
                        };
                    }
                    Msg::SetChunk { x, z: y, value } => {
                        map.insert((x, y), value);
                    }
                    Msg::SetBlock { loc, value } => {
                        let BlockLocation(x, y, z) = loc;

                        let cx = (x >> 4) as i32;
                        let cz = (z >> 4) as i32;

                        if y < 0 {
                            continue 'msg_loop;
                        }

                        // unwrap_or!()
                        let chunk = map.get_mut(&(cx, cz));

                        let chunk = if let Some(inner) = chunk {
                            inner
                        } else {
                            continue;
                        };

                        let relative_x = (x - (cx << 4) as i64) as u8;
                        let relative_z = (z - (cz << 4) as i64) as u8;

                        assert!((0..16).contains(&relative_x));
                        assert!((0..16).contains(&relative_z));

                        let y = y as usize;

                        *chunk.get_rel_block_ref(relative_x, y, relative_z) = value;
                    }
                    Msg::GetBlocks { locs: locs, resp } => {
                        let vec: Vec<_> = locs
                            .into_iter()
                            .map(|BlockLocation(x, y, z)| {
                                let cx = (x >> 4) as i32;
                                let cz = (z >> 4) as i32;

                                if y < 0 {
                                    return None;
                                }

                                let chunk = match map.get_mut(&(cx, cz)) {
                                    None => {
                                        return None;
                                    }
                                    Some(inner) => inner,
                                };

                                let relative_x = (x - (cx << 4) as i64) as u8;
                                let relative_z = (z - (cz << 4) as i64) as u8;

                                assert!((0..16).contains(&relative_x));
                                assert!((0..16).contains(&relative_z));

                                let y = y as usize;

                                Some(chunk.get_rel_block(relative_x, y, relative_z))
                            })
                            .collect();
                        resp.send(vec).unwrap();
                    }
                };
            }
        });

        WorldBlocks { tx }
    }
}

impl WorldBlocks {
    pub async fn get_block(&self, location: BlockLocation) -> Option<BlockState> {
        let (tx, rx) = oneshot();

        let msg = Msg::GetBlock {
            loc: BlockLocation(x, y, z),
            resp: tx,
        };

        self.tx.send(msg).unwrap();
        return rx.await.expect("receive works");
    }

    pub async fn adjacent_blocks_not_up(&self, location: BlockLocation) -> Vec<Option<BlockState>> {
        let BlockLocation(x,y,z) = location;
        let test_locs: Vec<_> = CardinalDirection3D::ALL_BUT_UP.into_iter().map(|dir|{
            let Change{dx, dy ,dz} = dir.unit_change();
            BlockLocation(x + dx, y + dy, z + dz)
        }).collect();
        
        self.get_blocks(test_locs).await
    }

    pub async fn get_blocks(&self, locs: Vec<BlockLocation>) -> Vec<Option<BlockState>> {
        let (tx, rx) = oneshot();

        let msg = Msg::GetBlocks { locs, resp: tx };
        self.tx.send(msg).await.expect("send works");
        return rx.await.expect("receive works");
    }

    pub async fn set_block(&self, x: i64, y: i64, z: i64, value: BlockState) {
        let msg = Msg::SetBlock {
            loc: BlockLocation(x, y, z),
            value,
        };
        self.tx.send(msg).await.unwrap();
    }

    pub async fn process_chunk_pkt(&mut self, packet: ChunkDataPkt) {
        let x = packet.chunk_x;
        let z = packet.chunk_z;
        match self
            .tx
            .send(Msg::SetChunk {
                x,
                z,
                value: Chunk::new(packet.data),
            })
            .await
        {
            Ok(x) => {}
            Err(e) => {
                println!("and I oop {}", e)
            }
        }
    }

    pub async fn get_relative(
        &self,
        origin: BlockLocation,
        difference: BlockLocation,
    ) -> Option<BlockState> {
        let BlockLocation(x, y, z) = origin;
        let BlockLocation(dx, dy, dz) = difference;

        let (x, y, z) = (x + dx, y + dy, z + dz);
        self.get_block(x, y, z).await
    }
}
