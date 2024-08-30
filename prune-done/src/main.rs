// main.rs
mod cli;
mod tree_sitter;

use clap::Parser;
use std::{
    fs,
    io::{self, Read},
};
use tracing::{debug, Level};
use tracing_subscriber::FmtSubscriber;

use crate::cli::Cli;
use crate::tree_sitter::modify_content;

fn main() {
    set_up_logging();

    let cli = Cli::parse();
    debug!("{:?}", cli);
    let config = cli.config();
    let content = cli.read_input().expect("Error reading file.");
    let output = modify_content(&config, &content);
    cli.write_output(&output).expect("Error writing output");
}

fn set_up_logging() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .with_writer(io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}
