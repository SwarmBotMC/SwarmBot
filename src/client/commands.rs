/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/8/21, 8:37 PM
 */


use std::convert::TryFrom;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;

use hyper::{Body, Error, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpSocket};

use crate::error::Res;
use crate::storage::block::BlockLocation2D;

pub struct Commands {
    pub pending: Receiver<Command>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Selection2D {
    pub from: BlockLocation2D,
    pub to: BlockLocation2D,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mine {
    pub sel: Selection2D,
}

pub enum Command {
    Mine(Mine)
}


async fn process(tx: Arc<Mutex<std::sync::mpsc::Sender<Command>>>, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/mine") => {
            let (_, body) = req.into_parts();
            let bytes = hyper::body::to_bytes(body).await?;
            let mine: Mine = serde_json::from_slice(&bytes).unwrap();

            tx.lock().unwrap().send(Command::Mine(mine)).unwrap();

            Ok(Response::default())
        }
        (_, path) => {
            println!("path {} does not exist", path);
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}


impl Commands {
    pub fn init() -> Res<Self> {
        let (tx, rx) = std::sync::mpsc::channel();
        let addr = "127.0.0.1:8080".parse().unwrap();


        let tx = Arc::new(Mutex::new(tx));

        let service = make_service_fn(move |_| {
            let tx = tx.clone();

            // shouldn't be a requirement for 'async move' but Server takes in an async closure so we need an async block
            async move {

                // show that it is moved
                let tx = tx.clone();

                Ok::<_, hyper::Error>({
                    service_fn(move |req: Request<Body>| {
                        process(tx.clone(), req)
                    })
                })
            }
        });

        let server = Server::bind(&addr).serve(service);

        println!("starting http service on {}", addr);

        tokio::task::spawn_local(async move {
            server.await.unwrap();
        });

        Ok(Self {
            pending: rx
        })
    }
}
