// main.rs
mod cli;
mod commands;
mod utils;

use crate::cli::Commands;
use crate::utils::set_up_logging;
use orgtools::org::Position;
use tracing::debug;

use crate::cli::cli;

fn main() {
    set_up_logging();

    let cli = cli();
    debug!("{:?}", cli);
    let config = cli.config();
    match &cli.command {
        Commands::Prune {
            output_file,
            input_file,
        } => {
            commands::prune_done(&config, input_file.as_deref(), output_file.as_deref())
                .expect("prune_done failed");
        }
        Commands::Tree {
            input_file,
            sexp,
            sections,
        } => {
            commands::print_tree(&config, input_file.as_deref(), *sexp, *sections)
                .expect("print_tree failed");
        }
        Commands::List { input_file } => {
            commands::list_headlines(&config, input_file.as_deref())
                .expect("list_headlines failed");
        }
        Commands::Add {
            input_file,
            output_file,
            headline,
            under,
            after,
        } => {
            let (position, search) = if let Some(under) = under {
                (Position::Under, under)
            } else if let Some(after) = after {
                (Position::After, after)
            } else {
                panic!("Either under or after must be provided")
            };
            commands::add_headline(
                &config,
                input_file.as_deref(),
                output_file.as_deref(),
                headline,
                position,
                search,
            )
            .expect("add_headline failed");
        }
    }
}
