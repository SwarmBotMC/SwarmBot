//! Module for interacting with Mojang

use std::convert::TryFrom;

use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha1::{Digest, Sha1};
use swarm_bot_packets::types::UUID;

use crate::{bootstrap::Proxy, default};

#[derive(Debug)]
pub struct MojangClient {
    client: reqwest::Client,
}

impl Default for MojangClient {
    fn default() -> Self {
        Self { client: default() }
    }
}

impl TryFrom<&Proxy> for MojangClient {
    type Error = anyhow::Error;

    fn try_from(proxy: &Proxy) -> Result<Self, Self::Error> {
        let address = proxy.address();
        let user = &proxy.user;
        let pass = &proxy.pass;
        let full_address = format!("socks5://{address}");

        let proxy = reqwest::Proxy::https(full_address)?.basic_auth(user, pass);

        let client = reqwest::Client::builder().proxy(proxy).build()?;

        Ok(Self { client })
    }
}

impl TryFrom<Option<&Proxy>> for MojangClient {
    type Error = anyhow::Error;

    fn try_from(value: Option<&Proxy>) -> Result<Self, Self::Error> {
        value.map_or_else(|| Ok(Self::default()), Self::try_from)
    }
}

pub fn calc_hash(server_id: &str, shared_secret: &[u8], public_key_encoded: &[u8]) -> String {
    let ascii = server_id.as_bytes();

    let mut hasher = Sha1::new();
    hasher.update(ascii);
    hasher.update(shared_secret);
    hasher.update(public_key_encoded);
    let result = hasher.finalize();
    hexdigest(result.as_slice())
}

fn hexdigest(bytes: &[u8]) -> String {
    let bigint = BigInt::from_signed_bytes_be(bytes);
    format!("{bigint:x}")
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SelectedProfile {
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct RawAuthResponse {
    pub access_token: String,
    pub client_token: String,
    pub selected_profile: SelectedProfile,
}

#[derive(Default)]
pub struct AuthResponse {
    pub access_token: String,
    pub client_token: String,
    pub username: String,
    pub uuid: UUID,
}

impl MojangClient {
    pub async fn authenticate(&self, email: &str, password: &str) -> anyhow::Result<AuthResponse> {
        let payload = json!({
            "agent": {
                "name": "minecraft",
                "version": 1
            },
            "username": email, // this is not a mistake... the username now takes in email
            "password": password,
            "requestuser": false
        });

        let payload = payload.to_string();

        let res = self
            .client
            .post("https://authserver.mojang.com/authenticate")
            .body(payload)
            .send()
            .await?;

        let status = res.status();
        if status != 200 {
            let info = res.text().await.unwrap_or_default();
            anyhow::bail!("Invalid credentials! Error code: {status}, info: {info}");
        }

        let auth: RawAuthResponse = res.json().await?;
        let auth = AuthResponse {
            access_token: auth.access_token,
            client_token: auth.client_token,
            username: auth.selected_profile.name,
            uuid: UUID::from(&auth.selected_profile.id),
        };
        Ok(auth)
    }

    pub async fn refresh(
        &self,
        access_token: &str,
        client_token: &str,
    ) -> anyhow::Result<AuthResponse> {
        let payload = json!({
            "accessToken": access_token, // this is not a mistake... the username now takes in email
            "clientToken": client_token,
            "requestUser": false,
        })
        .to_string();

        let res = self
            .client
            .post("https://authserver.mojang.com/refresh")
            .body(payload)
            .send()
            .await?;

        let _status = res.status();
        let auth: RawAuthResponse = res.json().await?;
        let auth = AuthResponse {
            access_token: auth.access_token,
            client_token: auth.client_token,
            username: auth.selected_profile.name,
            uuid: UUID::from(&auth.selected_profile.id),
        };
        Ok(auth)
    }

    pub async fn validate(&self, access_token: &str, client_token: &str) -> anyhow::Result<bool> {
        let payload = json!({
            "accessToken": access_token, // this is not a mistake... the username now takes in email
            "clientToken": client_token,
        })
        .to_string();

        let res = self
            .client
            .post("https://authserver.mojang.com/validate")
            .body(payload)
            .send()
            .await?;

        let status = res.status();
        Ok(status == 204)
    }

    pub async fn join(
        &self,
        uuid: UUID,
        server_hash: &str,
        access_token: &str,
    ) -> anyhow::Result<()> {
        let uuid_str = uuid.to_string();

        let payload = json!({
            "accessToken": access_token,
            "selectedProfile": uuid_str,
            "serverId": server_hash
        });

        let payload = payload.to_string();

        let res = self
            .client
            .post("https://sessionserver.mojang.com/session/minecraft/join")
            .body(payload)
            .send()
            .await?;

        let status = res.status();

        if status != 204 {
            anyhow::bail!("UUID invalid {uuid_str}! Error code: {status}");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sha1::{Digest, Sha1};

    use crate::bootstrap::mojang::hexdigest;

    fn sha1(input: &[u8]) -> String {
        let mut sha1 = Sha1::new();
        sha1.update(input);
        hexdigest(&sha1.finalize().as_slice())
    }

    #[test]
    fn test_hash() {
        assert_eq!(sha1(b"jeb_"), "-7c9d5b0044c130109a5d7b5fb5c317c02b4e28c1");
        assert_eq!(sha1(b"simon"), "88e16a1019277b15d58faf0541e11910eb756f6");
        assert_eq!(sha1(b"Notch"), "4ed1f46bbe04bc756bcb17c0c7ce3e4632f06a48");
    }
}
