use std::fs::File;

use serde::de::DeserializeOwned;

use crate::error::Res;
use crate::bootstrap::{User, Proxy};
use serde::Serialize;

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

fn write_csv<T: Serialize>(file: File, elements: Vec<T>) -> Res {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b':')
        .has_headers(false)
        .from_writer(file);

    for element in elements {
        writer.serialize(element)?;
    }

    Ok(())
}

pub fn read_users(file: File) -> Res<Vec<User>> {
    read_csv(file)
}

pub fn read_proxies(file: File) -> Res<Vec<Proxy>> {
    read_csv(file)
}
