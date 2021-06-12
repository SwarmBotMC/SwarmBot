use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncWrite};
use packets::types::VarInt;
use std::future::Future;
