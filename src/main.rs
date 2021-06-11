use clap::{Clap, AppSettings};

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Andrew Gazelka")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {

    host: String,

    #[clap(short,long, default_value = "1")]
    count: usize,

    #[clap(short, long)]
    online: bool,

    #[clap(short, long)]
    socks5: bool,
}

fn main() {
    println!("Hello, world!");
    let opts: Opts = Opts::parse();

    println!("opts {:?}", opts);
}
