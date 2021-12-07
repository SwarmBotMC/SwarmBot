/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use clap::{Parser};

#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Andrew Gazelka")]
pub struct Opts {
    pub host: String,

    #[clap(long)]
    pub load: bool,

    #[clap(short, long, default_value = "1")]
    pub count: usize,

    #[clap(long, default_value = "25565")]
    pub port: u16,

    #[clap(long, default_value = "8080")]
    pub ws_port: u16,

    #[clap(short, long, default_value = "500")]
    pub delay: u64,

    #[clap(long, default_value = "users.csv")]
    pub users_file: String,

    #[clap(long, default_value = "proxies.csv")]
    pub proxies_file: String,

    #[clap(short, long, default_value = "340")]
    pub version: usize,
}

impl Opts {
    pub fn get() -> Opts {
        Opts::parse()
    }
}
