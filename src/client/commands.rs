/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/8/21, 8:37 PM
 */


use tokio::net::{TcpSocket, TcpListener};
use crate::error::Res;
use hyper::{Server, Request, Body, Method, Response, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use crate::storage::block::BlockLocation2D;
use std::net::SocketAddr;
use std::convert::TryFrom;

pub struct Commands;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Selection2D {
    from: BlockLocation2D,
    to: BlockLocation2D
}

#[derive(Serialize, Deserialize, Debug)]
struct Mine {
    sel: Selection2D
}



async fn process(req: Request<Body>) -> Result<Response<Body>, hyper::Error>{
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/mine") => {

            let (_, body) = req.into_parts();
            let bytes = hyper::body::to_bytes(body).await?;
            let mine: Mine = serde_json::from_slice(&bytes).unwrap();

            println!("mine {:?}", mine);

            Ok(Response::default())
        }
        (_, path)=> {
            println!("path {} does not exist", path);
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

impl Commands {
    pub fn init() -> Res {
        let addr = "127.0.0.1:8080".parse().unwrap();

        let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(process)) });

        let server = Server::bind(&addr).serve(service);

        println!("starting http service on {}", addr);

        tokio::task::spawn_local(async move {
            server.await.unwrap();
        });

        Ok(())
    }
}
