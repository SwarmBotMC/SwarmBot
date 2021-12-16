// Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::fs::File;

use serde::de::DeserializeOwned;

use crate::{
    bootstrap::{CSVUser, Proxy},
    error::Res,
};

fn read_csv<T: DeserializeOwned>(file: File) -> Res<Vec<T>> {
    csv::ReaderBuilder::new()
        .delimiter(b':')
        .has_headers(false)
        .from_reader(file)
        .deserialize()
        .map(|res| {
            let elem: T = res?;
            Ok(elem)
        })
        .collect()
}

pub fn read_users(file: File) -> Res<Vec<CSVUser>> {
    read_csv(file)
}

pub fn read_proxies(file: File) -> Res<Vec<Proxy>> {
    read_csv(file)
}
