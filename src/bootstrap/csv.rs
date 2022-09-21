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
