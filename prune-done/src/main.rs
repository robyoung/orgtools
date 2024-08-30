// main.rs
mod cli;
mod commands;
mod utils;

use clap::Parser;
use cli::{Commands, InputFile, OutputFile};
use tracing::debug;
use utils::set_up_logging;

use crate::cli::Cli;

fn main() {
    set_up_logging();

    let cli = Cli::parse();
    debug!("{:?}", cli);
    let config = cli.config();
    match &cli.command {
        Commands::Prune {
            output_file: OutputFile { output_file },
            input_file: InputFile { input_file },
        } => {
            crate::commands::prune_done(&config, input_file.as_deref(), output_file.as_deref())
                .expect("prune_done failed");
        }
        Commands::Tree {
            input_file: InputFile { input_file },
        } => {
            crate::commands::print_tree(&config, input_file.as_deref()).expect("print_tree failed");
        }
        _ => unimplemented!(),
    }
}
