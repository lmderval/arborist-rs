use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    println!("Value for config: {}", cli.config.display());
}
