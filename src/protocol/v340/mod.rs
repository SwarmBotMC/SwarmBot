use packets::types::{PacketState, UUID, VarInt};
use packets::types::Packet;
use tokio::time::{Duration, sleep};

use crate::bootstrap::{Connection, User};
use crate::bootstrap::mojang::{AuthResponse, calc_hash};
use crate::client::instance::{ClientInfo, State};
use crate::error::Error::WrongPacket;
use crate::error::Res;
use crate::protocol::{Login, McProtocol};
use crate::protocol::encrypt::{rand_bits, RSA};
use crate::protocol::io::reader::PacketReader;
use crate::protocol::io::writer::{PacketWriteChannel, PacketWriter};
use crate::protocol::types::PacketData;
use crate::protocol::v340::clientbound::{JoinGame, LoginSuccess};
use crate::protocol::v340::serverbound::HandshakeNextState;

mod clientbound;
mod serverbound;
mod types;

pub struct Protocol {
    rx: std::sync::mpsc::Receiver<PacketData>,
    tx: PacketWriteChannel,
}

#[async_trait::async_trait]
impl McProtocol for Protocol {
    async fn login(conn: Connection) -> Res<Login<Self>> {
        let Connection { user, online, mojang, port, read, write, host } = conn;

        let User { email, password, online } = user;


        let mut reader = PacketReader::from(read);
        let mut writer = PacketWriter::from(write);

        let AuthResponse { access_token, username, uuid } = if online {
            mojang.authenticate(&email, &password).await?
        } else {
            let mut response = AuthResponse::default();
            response.username = email;
            response
        };

        // START: handshake
        writer.write(serverbound::Handshake {
            protocol_version: VarInt(340),
            host,
            port,
            next_state: HandshakeNextState::Login,
        }).await;


        // START: login
        writer.write(serverbound::LoginStart {
            username: username.clone()
        }).await;

        // writer.flush().await;

        if online {
            let clientbound::EncryptionRequest { public_key_der, verify_token, server_id } = reader.read_exact_packet().await?;

            let rsa = RSA::from_der(&public_key_der);

            let shared_secret = rand_bits();

            let encrypted_ss = rsa.encrypt(&shared_secret).unwrap();
            let encrypted_verify = rsa.encrypt(&verify_token).unwrap();

            // Mojang online mode requests
            if online {
                let hash = calc_hash(&server_id, &shared_secret, &public_key_der);
                mojang.join(uuid, &hash, &access_token).await?;
            }

            // why sleep?
            sleep(Duration::from_secs(1)).await;

            // id = 1
            writer.write(serverbound::EncryptionResponse {
                shared_secret: encrypted_ss,
                verify_token: encrypted_verify,
            }).await;

            // writer.flush().await;

            // we now do everything encrypted
            writer.encryption(&shared_secret);
            reader.encryption(&shared_secret);
        }


        // set compression or login success
        {
            let mut data = reader.read().await;

            let LoginSuccess { username, uuid } = match data.id {
                clientbound::SetCompression::ID => {
                    let clientbound::SetCompression { threshold } = data.read();

                    reader.compression(threshold.into());
                    writer.compression(threshold.into());

                    reader.read_exact_packet().await?
                }
                clientbound::LoginSuccess::ID => {
                    data.reader.read()
                }
                // clientbound::Disconnect::ID => {
                //     let clientbound::Disconnect { reason } = data.reader.read();
                //     return Err(Disconnect(reason));
                // }
                actual => {
                    return Err(WrongPacket {
                        state: PacketState::Login,
                        expected: LoginSuccess::ID,
                        actual,
                    });
                }
            };
        }

        let (tx, rx) = std::sync::mpsc::channel();
        let (os_tx, os_rx) = tokio::sync::oneshot::channel();

        tokio::task::spawn_local(async move {
            let mut oneshot = Some(os_tx);
            loop {
                let packet = reader.read().await;
                if packet.id == clientbound::JoinGame::ID {
                    if let Some(os_tx) = oneshot.take() {
                        let mut packet = packet.clone();
                        let processed: JoinGame = packet.read();
                        os_tx.send(processed.entity_id);
                    }
                }
                tx.send(packet);
            }
        });

        let tx = writer.into_channel();

        let entity_id = os_rx.await.unwrap();

        let protocol = Protocol {
            rx,
            tx,
        };

        let login = Login {
            protocol,
            info: ClientInfo {
                username,
                uuid,
                entity_id,
            },
        };

        Ok(login)
    }

    fn apply_packets(&self, client: &mut State) {
        while let Ok(data) = self.rx.try_recv() {
            self.process_packet(data, client);
        }
    }

    fn teleport(&mut self) {}
}

impl Protocol {
    fn process_packet(&self, data: PacketData, client: &mut State) {
        use clientbound::*;
        match data.id {
            JoinGame::ID => println!("player joined"),
            KeepAliveCb::ID => println!("keep alive"),
            _ => {}
        }
    }
}
