use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "whered", version, about)]
pub struct Args {
    /// Specify a custom listen address from the default 0.0.0.0:15
    #[arg(short = 'l', long)]
    pub listen_addr: Option<String>,
}
