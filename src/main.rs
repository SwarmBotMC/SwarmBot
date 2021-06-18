#![allow(unused)]
#![deny(unused_must_use)]
#![feature(async_closure)]
#![feature(never_type)]
#![feature(in_band_lifetimes)]
#![feature(drain_filter)]

use tokio::runtime::Runtime;
use tokio::task;

use crate::bootstrap::Output;
use crate::client::runner::{Runner, RunnerOptions};
use crate::error::{ResContext, Error};
use std::fs::File;
use crate::error::Error::Mojang;
use crate::bootstrap::mojang::AuthResponse;
use std::time::Duration;

mod error;
mod bootstrap;
mod protocol;
mod client;
mod storage;
mod db;
mod types;


fn main() {
    auth();
    // match run() {
    //     Ok(_) => println!("Program exited without errors somehow"),
    //     Err(err) => println!("{}", err)
    // };
}

fn auth(){

    let mut rt = Runtime::new().unwrap();
    let local = task::LocalSet::new();
    local.block_on(&rt, async move {
        let file = File::open("users.csv").unwrap();
        let users = bootstrap::csv::read_users(file).unwrap();

        let file = File::open("proxies.csv").unwrap();
        let proxies = bootstrap::csv::read_proxies(file).unwrap();

        println!("proxy count {}", proxies.len());
        let mut handles = Vec::with_capacity(users.len());
        for (user, proxy) in users.iter().zip(proxies) {

            let email = user.email.clone();
            let pass = user.password.clone();

            let mojang = {
                let address = proxy.address();
                let user = proxy.user;
                let pass = proxy.pass;

                bootstrap::mojang::Mojang::socks5(&address, &user, &pass).unwrap()
            };

            let handle = tokio::task::spawn_local(async move{
                match mojang.authenticate(&email, &pass).await {
                    Ok(auth) => {
                        println!("successfully authenticated {}", email);
                        1
                    }
                    Err(err) => {
                        println!("could not auth {} ... {}", email, err);
                        0
                    }
                }
            });
            handles.push(handle);
        }

        let mut success_count = 0;
        for handle in handles {
            success_count += handle.await.unwrap();
        }

        println!("success {}, fail {}", success_count, users.len() - success_count);

    });

}

fn run() -> ResContext<()> {
    let mut rt = Runtime::new().unwrap();
    let local = task::LocalSet::new();

    local.block_on(&rt, async move {
        let Output { version, delay_millis, connections } = bootstrap::init().await?;
        let opts = RunnerOptions {delay_millis};

        match version {
            340 => Runner::<protocol::v340::Protocol>::run(connections, opts).await,
            _ => { panic!("version {} does not exist", version) }
        }

        Ok(())
    })
}
