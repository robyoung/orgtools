use std::io;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use tree_sitter::{Language, Parser};

pub(crate) fn get_parser() -> Parser {
    get_parser_and_language().0
}

pub(crate) fn get_parser_and_language() -> (Parser, Language) {
    let mut parser = Parser::new();
    let language = get_language();
    parser
        .set_language(language)
        .expect("Error loading Org language");
    (parser, language)
}

pub(crate) fn get_language() -> tree_sitter::Language {
    tree_sitter_org::language()
}

pub fn set_up_logging() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .with_writer(io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

pub mod fs {
    use std::{
        fs,
        io::{self, Read, Write},
    };

    pub fn read_input(input_file: Option<&str>) -> io::Result<String> {
        if let Some(input_file) = input_file {
            fs::read_to_string(input_file)
        } else {
            let mut content = String::new();
            io::stdin().read_to_string(&mut content)?;
            Ok(content)
        }
    }

    pub fn write_output(
        input_file: Option<&str>,
        output_file: Option<&str>,
        content: &str,
    ) -> io::Result<()> {
        if let Some(output_file) = output_file {
            fs::write(output_file, content)
        } else if let Some(input_file) = input_file {
            fs::write(input_file, content)
        } else {
            io::stdout().write_all(content.as_bytes())
        }
    }
}
