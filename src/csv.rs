use std::fs::File;

use csv::{Reader, StringRecord};

use crate::error::{ContextTrait, CSVIndex};
use crate::Res;

fn read_csv(file: File) -> Reader<File> {
    csv::ReaderBuilder::new()
        .delimiter(b':')
        .has_headers(false)
        .from_reader(file)
}

pub struct User {
    pub email: String,
    pub password: String,
}

pub fn read_users(file: File) -> Res<Vec<User>> {
    read_csv(file)
        .records()
        .map(|record| {
            let record = record.context_str("reading users")?;
            Ok(User {
                email: record.get(0).ok_or(CSVIndex(0)).context_str("reading users")?.to_string(),
                password: record.get(1).ok_or(CSVIndex(1)).context_str("reading users")?.to_string(),
            })
        })
        .collect()
}
