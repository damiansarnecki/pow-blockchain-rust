use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    #[arg(long, default_value_t = 7878)]
    pub port: u16,

    #[arg(long, default_value_t = true)]
    pub mine: bool,
}
