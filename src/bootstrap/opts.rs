//! Module to interact with cargo options

use clap::Parser;

/// Options parsed from CLI
#[derive(Parser, Debug)]
#[command(about, author, version)]
pub struct CliOptions {
    /// The host which the bot will connect to
    pub host: String,

    /// The number of bots that will be launched
    #[clap(short, long, default_value = "1")]
    pub count: usize,

    /// If a proxy will be used to log the bots in and join the server.
    /// This is recommended as Mojang API has rate limits as do most
    /// servers. Once all proxies are used, bots will go back to the
    /// first proxy.
    ///
    /// If using proxies, make sure that they are not IP banned from Mojang's
    /// API, as alt accounts can easily get locked.
    #[clap(short)]
    pub proxy: bool,

    /// The port of the server which is being connected to
    #[clap(long, default_value = "25565")]
    pub port: u16,

    /// The port of the web socket that is used to communicate bot commands
    /// to. This is used to interface with the SwarmBot mod, although it
    /// can be used for anything.
    #[clap(long, default_value = "8080")]
    pub ws_port: u16,

    /// The delay for launching the bots
    #[clap(short, long, default_value = "500")]
    pub delay_ms: u64,

    /// The file that the users will be read from. This is a CSV file of
    /// the form of
    ///
    /// email@gmail.com:password
    ///
    /// Note, instead of commas as a delimiter, colons are used
    #[clap(long, default_value = "users.csv")]
    pub users_file: String,

    /// The file that the proxies will be read from. This is a CSV file of
    /// the form of
    ///
    /// 111.111.11.11:3333:username:password
    ///
    /// Note, instead of commas as a delimiter, colons are used
    #[clap(long, default_value = "proxies.csv")]
    pub proxies_file: String,

    /// The version number that the bots will be launched on. To see a list
    /// of versions see
    ///
    /// <https://minecraft.fandom.com/wiki/Protocol_version>
    #[clap(short, long, default_value = "340")]
    pub version: usize,

    /// if we are launching in offline mode
    #[clap(long)]
    pub offline: bool
}

impl CliOptions {
    pub fn get() -> Self {
        Self::parse()
    }
}
