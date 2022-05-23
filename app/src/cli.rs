use clap::Parser;

pub fn init() -> Cli {
    Cli::parse()
}

#[derive(clap::Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    /// Use a local instance of Tor instead of starting one. Tari usually uses 9051
    #[clap(short = 't', long, alias = "local-tor-port")]
    pub local_tor_control_port: Option<u16>,
}
