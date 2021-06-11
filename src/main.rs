use std::fs::File;

use crate::error::ContextTrait;
use crate::opts::Opts;

mod opts;
mod error;

pub type Res<T = ()> = Result<T, error::Error>;

fn main() {
    match run() {
        Ok(_) => println!("Program exited without errors"),
        Err(err) => println!("{}", err)
    };
}

fn run() -> Res {
    println!("Hello, world!");
    let Opts { users_file, .. } = Opts::get();

    let users = File::open(&users_file).context(|| format!("reading {}", users_file))?;


    Ok(())
}
