

mod clientbound;
mod serverbound;


// struct PacketData {
//     id: u32,
//     data: Vec<u8>
// }

use crate::error::Res;
use crate::bootstrap::{Connection, User};
use crate::client::instance::{State};
use crate::protocol::{McProtocol, Login};
use crate::protocol::io::reader::PacketReader;
use crate::protocol::io::writer::PacketWriter;
use crate::protocol::types::PacketData;
use crate::protocol::v340::serverbound::HandshakeNextState;
use packets::types::{VarInt, PacketState, UUID};
use crate::bootstrap::mojang::{AuthResponse, calc_hash};
use crate::protocol::encrypt::{RSA, rand_bits};
use tokio::time::{sleep, Duration};
use packets::types::Packet;
use crate::protocol::v340::clientbound::LoginSuccess;
use crate::error::Error::WrongPacket;

pub struct Protocol {
    reader: PacketReader,
    writer: PacketWriter
}

#[async_trait::async_trait]
impl McProtocol for Protocol {
    async fn login(conn: Connection) -> Res<Login<Self>> {

        let Connection{user, online, mojang, port, read, write, host} = conn;

        let User {email, password, online} = user;


        let mut reader = PacketReader::from(read);
        let mut writer = PacketWriter::from(write);

        let AuthResponse {access_token, name, uuid} = if online {
            mojang.authenticate(&email, &password).await?
        } else {
            let mut response = AuthResponse::default();
            response.name = email;
            response
        };

        // START: handshake
        writer.write(serverbound::Handshake {
            protocol_version: VarInt(340),
            host,
            port,
            next_state: HandshakeNextState::Login
        }).await;

        // START: login
        writer.write(serverbound::LoginStart {
            username: name
        }).await;


        if online {
            let clientbound::EncryptionRequest { public_key_der, verify_token, server_id } = reader.read_exact_packet().await?;

            let rsa = RSA::from_der(&public_key_der);

            let shared_secret = rand_bits();

            let encrypted_ss = rsa.encrypt(&shared_secret).unwrap();
            let encrypted_verify = rsa.encrypt(&verify_token).unwrap();

            // Mojang online mode requests
            if online {
                let hash = calc_hash(&server_id, &shared_secret, &public_key_der);
                mojang.join(UUID::from(&uuid), &hash, &access_token).await?;
            }

            // why sleep?
            sleep(Duration::from_secs(1)).await;

            // id = 1
            writer.write(serverbound::EncryptionResponse {
                shared_secret: encrypted_ss,
                verify_token: encrypted_verify,
            }).await;

            // we now do everything encrypted
            writer.encryption(&shared_secret);
            reader.encryption(&shared_secret);
        }


        // set compression or login success
        {
            let mut data = reader.read().await;

            let LoginSuccess { username, uuid } = match data.id {
                clientbound::SetCompression::ID => {
                    let clientbound::SetCompression{ threshold } = data.read();

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

            println!("successfully logged in {}", username);
        }


        todo!()


    }

    fn apply_packets(&self, client: &mut State) {
        todo!()
    }

    fn teleport(&mut self) {
        todo!()
    }
}
