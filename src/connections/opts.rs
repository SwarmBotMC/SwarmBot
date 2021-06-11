use clap::{AppSettings, Clap};

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Andrew Gazelka")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    pub host: String,

    #[clap(short, long, default_value = "1")]
    pub count: usize,

    #[clap(short, long)]
    pub online: bool,

    #[clap(long, default_value = "users.csv")]
    pub users_file: String,

    #[clap(long, default_value = "proxies.csv")]
    pub proxies_file: String,

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
