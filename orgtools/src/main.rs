// main.rs
mod cli;
mod commands;
mod org;
mod utils;

use cli::Commands;
use tracing::debug;
use utils::set_up_logging;

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
            crate::commands::prune_done(&config, input_file.as_deref(), output_file.as_deref())
                .expect("prune_done failed");
        }
        Commands::Tree { input_file, sexp } => {
            crate::commands::print_tree(&config, input_file.as_deref(), *sexp)
                .expect("print_tree failed");
        }
        Commands::List { input_file } => {
            crate::commands::list_headlines(&config, input_file.as_deref())
                .expect("list_headlines failed");
        }
    }
}
