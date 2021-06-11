use std::fs::File;

use crate::csv::{read_proxies, read_users, User};
use crate::error::{HasContext, Res, ResContext};
use crate::opts::Opts;

mod opts;
mod error;
mod csv;

fn main() {
    match run() {
        Ok(_) => println!("Program exited without errors"),
        Err(err) => println!("{}", err)
    };
}

fn run() -> ResContext {

    let Opts { users_file, proxy, proxies_file, .. } = Opts::get();

    let users = {
        let file = File::open(&users_file).context(|| format!("opening users ({})", users_file))?;
        read_users(file).context(|| format!("reading users ({})", users_file))?
    };

    let proxies = if proxy {
        let file = File::open(&proxies_file).context(|| format!("opening proxy ({})", users_file))?;
        read_proxies(file).context(|| format!("opening proxies ({})", proxies_file))?
    } else {
        vec![]
    };



    Ok(())
}
