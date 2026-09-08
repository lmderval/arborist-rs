use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::config::parser::parse;
use crate::config::pretty_printer::PrettyPrinter;
use crate::render::renderer::Renderer;
use crate::typing::exp_resolver::ExpResolver;
use crate::typing::name_resolver::NameResolver;
use crate::typing::type_checker::TypeChecker;

pub mod config;
pub mod render;
pub mod typing;

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
    ($config:expr) => {
        $config
    };

    (
        $config:expr
        => $first_pass:ident$(($($first_arg:expr),+$(,)?))?
        $(=> $pass:ident$(($($arg:expr),+$(,)?))?)*
    ) => {
        run_passes!(
            $first_pass::do_pass($config $($(, $first_arg)+)?)
            $(=> $pass$(($($arg),+))?)*
        )
    };
}

fn main() {
    let cli = Cli::parse();

    let unparsed_config = fs::read_to_string(&cli.config).expect("cannot read file");
    let mut config = parse(&unparsed_config).expect("unsuccessful parse");
    config.set_path(&cli.config);

    let typed_config = run_passes!(
        config
        => ExpResolver
        => NameResolver
        => TypeChecker
    );

    match cli.command {
        Commands::PrettyPrint => PrettyPrinter::print(&typed_config),
        Commands::Render { template, in_place } => {
            Renderer::render(&typed_config, &template, in_place)
        }
    }
}
