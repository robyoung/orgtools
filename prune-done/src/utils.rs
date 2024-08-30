use std::io;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use tree_sitter::{Node, Parser};

pub(crate) fn get_parser() -> Parser {
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_org::language())
        .expect("Error loading Org language");
    parser
}

pub fn get_stars(node: Node, content: &str) -> String {
    node.child_by_field_name("stars")
        .expect("Error getting stars")
        .utf8_text(content.as_bytes())
        .expect("Error getting stars text")
        .to_owned()
}

pub fn get_headline_text(node: Node, content: &str) -> Option<String> {
    if let Some(item) = node.child_by_field_name("item") {
        Some(item.utf8_text(content.as_bytes()).ok()?.to_owned())
    } else {
        None
    }
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
