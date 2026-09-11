use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::config::parser::ConfigParser;
use crate::config::pretty_printer::PrettyPrinter;
use crate::render::renderer::Renderer;
use crate::typing::exp_resolver::ExpResolver;
use crate::typing::inheritance_checker::InheritanceChecker;
use crate::typing::name_resolver::NameResolver;
use crate::utils::error::MultiError;
use crate::utils::file_reader::FileReader;

pub mod config;
pub mod render;
pub mod typing;
pub mod utils;

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
        template: Option<PathBuf>,
        #[arg(short, long)]
        in_place: bool,
    },
}

fn main() {
    let mut errors = MultiError::new();
    let cli = Cli::parse();

    let config = run_passes!(
        (&cli.config, errors)
        => FileReader | exit_on_error
        => ConfigParser(&cli.config)
        => ExpResolver
        => NameResolver
        => InheritanceChecker | exit_on_error
    );

    match cli.command {
        Commands::PrettyPrint => PrettyPrinter::print(&config),
        Commands::Render { template, in_place } => Renderer::render(&config, &template, in_place),
    }
}
