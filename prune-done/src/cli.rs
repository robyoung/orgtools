use clap::Parser;
use std::{
    fs,
    io::{self, Read, Write},
};

pub fn parse_keyword(s: &str) -> Result<String, String> {
    Ok(s.trim().to_uppercase())
}

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(value_parser)]
    pub input_file: Option<String>,

    #[clap(long)]
    pub output_file: Option<String>,

    #[clap(long, value_delimiter = ',', value_parser = parse_keyword, default_value = "TODO,DOING,BLOCKED")]
    pub keywords_unfinished: Vec<String>,

    #[clap(long, value_delimiter = ',', value_parser = parse_keyword, default_value = "DONE,ABANDONED")]
    pub keywords_finished: Vec<String>,
}

impl Cli {
    pub fn config(&self) -> Config {
        Config {
            keywords_unfinished: self.keywords_unfinished.clone(),
            keywords_finished: self.keywords_finished.clone(),
        }
    }

    /// Read input from the input file or stdin.
    pub fn read_input(&self) -> io::Result<String> {
        if let Some(input_file) = &self.input_file {
            fs::read_to_string(input_file)
        } else {
            let mut content = String::new();
            io::stdin().read_to_string(&mut content)?;
            Ok(content)
        }
    }

    /// Write output to the output file or stdout.
    pub fn write_output(&self, output: &str) -> io::Result<()> {
        if self.input_file.is_none() && self.output_file.is_none() {
            io::stdout().write_all(output.as_bytes())
        } else {
            let output_file = self
                .output_file
                .as_ref()
                .unwrap_or(&self.input_file.as_ref().unwrap());
            fs::write(output_file, output)
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub keywords_unfinished: Vec<String>,
    pub keywords_finished: Vec<String>,
}
