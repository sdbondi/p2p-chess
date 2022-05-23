use std::path::PathBuf;

use clap::Parser;

pub fn init() -> Cli {
    Cli::parse()
}

#[derive(clap::Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(short = 't', long, alias = "local-tor-port")]
    pub local_tor_control_port: Option<u16>,
    #[clap(short = 'b', long, alias = "base-dir")]
    pub base_dir: Option<PathBuf>,
}
