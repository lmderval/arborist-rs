use std::fs;
use std::path::PathBuf;
use std::process::exit;

use clap::{Parser, Subcommand};

use crate::config::parser::ConfigParser;
use crate::config::pretty_printer::PrettyPrinter;
use crate::render::renderer::Renderer;
use crate::typing::exp_resolver::ExpResolver;
use crate::typing::name_resolver::NameResolver;
use crate::typing::type_checker::TypeChecker;
use crate::utils::error::{ErrorKind, MultiError};

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

macro_rules! run_passes {
    (($config:expr, $errors:ident)) => {
        $config
    };

    (
        ($config:expr, $errors:ident)
        => $first_pass:ident$(($($first_arg:expr),+$(,)?))?
        $(=> $pass:ident$(($($arg:expr),+$(,)?))?)*
    ) => {
        run_passes!(
            ($first_pass::do_pass($config, &mut $errors $($(, $first_arg)+)?), $errors)
            $(=> $pass$(($($arg),+))?)*
        )
    };
}

struct FileReader;

impl FileReader {
    fn do_pass(file: &PathBuf, errors: &mut MultiError) -> String {
        match fs::read_to_string(file) {
            Ok(source) => source,
            Err(err) => {
                errors.push(
                    ErrorKind::Cli,
                    &format!("unable to read `{}`: {}", file.to_string_lossy(), err),
                );
                String::new()
            }
        }
    }
}

fn main() {
    let mut errors = MultiError::new();
    let cli = Cli::parse();

    let config = run_passes!(
        (&cli.config, errors)
        => FileReader
        => ConfigParser(&cli.config)
        => ExpResolver
        => NameResolver
        => TypeChecker
    );

    for error in errors.iter() {
        println!("{}", error);
    }

    if let Some(error_kind) = errors.error_kind() {
        exit(error_kind as i32);
    }

    match cli.command {
        Commands::PrettyPrint => PrettyPrinter::print(&config),
        Commands::Render { template, in_place } => Renderer::render(&config, &template, in_place),
    }
}
