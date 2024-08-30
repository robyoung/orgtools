use clap::{Args, Parser, Subcommand};

pub fn parse_keyword(s: &str) -> Result<String, String> {
    Ok(s.trim().to_uppercase())
}

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(long, value_delimiter = ',', value_parser = parse_keyword, default_value = "TODO,DOING,BLOCKED")]
    pub keywords_unfinished: Vec<String>,

    #[clap(long, value_delimiter = ',', value_parser = parse_keyword, default_value = "DONE,ABANDONED")]
    pub keywords_finished: Vec<String>,

    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn config(&self) -> Config {
        Config {
            keywords_unfinished: self.keywords_unfinished.clone(),
            keywords_finished: self.keywords_finished.clone(),
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Prune {
        #[command(flatten)]
        input_file: InputFile,

        #[command(flatten)]
        output_file: OutputFile,
    },
    Tree {
        #[command(flatten)]
        input_file: InputFile,
    },
    List {
        #[command(flatten)]
        input_file: InputFile,
    },
}

#[derive(Args, Debug)]
pub struct InputFile {
    pub input_file: Option<String>,
}

#[derive(Args, Debug)]
pub struct OutputFile {
    #[arg(long)]
    pub output_file: Option<String>,
}

#[derive(Debug, Default)]
pub struct Config {
    pub keywords_unfinished: Vec<String>,
    pub keywords_finished: Vec<String>,
}
