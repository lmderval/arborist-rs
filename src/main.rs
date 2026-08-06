use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

use crate::config::parser::parse;
use crate::config::pretty_printer::PrettyPrinter;
use crate::render::renderer;

pub mod config;
pub mod render;

#[derive(Parser)]
#[command(version, about = "An AST generator for any language", long_about = None)]
struct Cli {
    /// Path to the arborist configuration file
    #[arg(short, long, value_name = "FILE")]
    config: PathBuf,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Pretty print the configuration file
    PrettyPrint,
    /// Render a template from the configuration
    Render {
        #[arg(short, long, value_name = "FILE")]
        template: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    let unparsed_config = fs::read_to_string(cli.config).expect("cannot read file");
    let config = parse(&unparsed_config).expect("unsuccessful parse");

    match cli.command {
        Commands::PrettyPrint => PrettyPrinter::print_config(&config),
        Commands::Render { template } => renderer::render(&config, &Some(template)),
    }
}
