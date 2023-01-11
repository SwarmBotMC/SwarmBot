//! module for modifying CSVs

use std::fs::File;

use serde::de::DeserializeOwned;

use crate::bootstrap::{CSVUser, Proxy};

/// read CSV file into `Vec` of the desired type
fn read_csv<T: DeserializeOwned>(file: File) -> anyhow::Result<Vec<T>> {
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

/// reade users from a CSV file
pub fn read_users(file: File) -> anyhow::Result<Vec<CSVUser>> {
    read_csv(file)
}

/// read proxies from a CSV file
pub fn read_proxies(file: File) -> anyhow::Result<Vec<Proxy>> {
    read_csv(file)
}
