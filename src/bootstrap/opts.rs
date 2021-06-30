/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use clap::{AppSettings, Clap};

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Andrew Gazelka")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    pub host: String,

    #[clap(long)]
    pub load: bool,

    #[clap(short, long, default_value = "1")]
    pub count: usize,

    #[clap(long, default_value = "25565")]
    pub port: u16,

    #[clap(short, long, default_value = "500")]
    pub delay: u64,

    #[clap(short, long)]
    pub online: bool,

    #[clap(long, default_value = "users.csv")]
    pub users_file: String,

    #[clap(long, default_value = "proxies.csv")]
    pub proxies_file: String,

    #[clap(long, default_value = "mcbot")]
    pub db: String,

    #[clap(short, long)]
    pub proxy: bool,

    #[clap(short, long, default_value = "340")]
    pub version: usize,
}

impl Opts {
    pub fn get() -> Opts {
        Opts::parse()
    }
}
