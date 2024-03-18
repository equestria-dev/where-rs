use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "where", version, about)]
pub struct Args {
    /// Generate a config file when none is available
    #[arg(short = 'c', long)]
    pub generate_config: bool,
}
