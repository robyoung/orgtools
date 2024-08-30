use std::{
    fs,
    io::{self, Read, Write},
};

use clap::Parser;
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;
use tree_sitter::TreeCursor;
use tree_sitter_org;

fn parse_keyword(s: &str) -> Result<String, String> {
    Ok(s.trim().to_uppercase())
}

#[derive(Parser, Debug)]
struct Cli {
    #[clap(value_parser)]
    input_file: Option<String>,

    #[clap(long)]
    output_file: Option<String>,

    #[clap(long, value_delimiter = ',', value_parser = parse_keyword, default_value = "TODO,DOING,BLOCKED")]
    keywords_unfinished: Vec<String>,

    #[clap(long, value_delimiter = ',', value_parser = parse_keyword, default_value = "DONE,ABANDONED")]
    keywords_finished: Vec<String>,
}

impl Cli {
    fn config(&self) -> Config {
        Config {
            keywords_unfinished: self.keywords_unfinished.clone(),
            keywords_finished: self.keywords_finished.clone(),
        }
    }

    fn write_output(&self, output: &str) -> io::Result<()> {
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
struct Config {
    keywords_unfinished: Vec<String>,
    keywords_finished: Vec<String>,
}

fn main() {
    // set up tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .with_writer(io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cli = Cli::parse();
    debug!("{:?}", cli);
    let config = cli.config();

    let content = if let Some(input_file) = cli.input_file.clone() {
        fs::read_to_string(&input_file).expect("Error reading file")
    } else {
        let mut content = String::new();
        io::stdin()
            .read_to_string(&mut content)
            .expect("Error reading from stdin");
        content
    };
    let output = modify_content(&config, &content);
    cli.write_output(&output).expect("Error writing output");
}

fn modify_content(config: &Config, content: &str) -> String {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(tree_sitter_org::language())
        .expect("Error loading Org language");
    let tree = parser.parse(&content, None).unwrap();

    let mut output = String::new();
    let mut start_byte = 0;
    let mut edited = false;

    let mut cursor = tree.walk();
    traverse_and_modify(
        &config,
        &mut cursor,
        &content,
        &mut output,
        &mut start_byte,
        &mut edited,
    );

    output.push_str(&content[start_byte..]);
    output
}

fn traverse_and_modify(
    config: &Config,
    cursor: &mut TreeCursor,
    content: &str,
    output: &mut String,
    start_byte: &mut usize,
    edited: &mut bool,
) {
    loop {
        // Mark TODO as DONE
        let node = cursor.node();

        if !prune_done(config, cursor, node, content, output, start_byte, edited) {
            // Recurse into children
            if cursor.goto_first_child() {
                traverse_and_modify(config, cursor, content, output, start_byte, edited);
                cursor.goto_parent();
            }

            // Move to next sibling
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

/// Mark TODO as DONE
fn mark_done(
    node: tree_sitter::Node,
    content: &str,
    output: &mut String,
    start_byte: &mut usize,
    edited: &mut bool,
) {
    if node.kind() == "headline" {
        if let Some(item) = node.child_by_field_name("item") {
            if let Ok(headline_text) = item.utf8_text(content.as_bytes()) {
                if headline_text.starts_with("TODO") {
                    output.push_str(&content[*start_byte..item.start_byte()]);
                    output.push_str(&&headline_text.replace("TODO", "DONE"));
                    println!("{:?}", headline_text);
                    *start_byte = item.end_byte();
                    *edited = true;
                }
            }
        }
    }
}

fn prune_done(
    config: &Config,
    cursor: &mut TreeCursor,
    node: tree_sitter::Node,
    content: &str,
    output: &mut String,
    start_byte: &mut usize,
    edited: &mut bool,
) -> bool {
    if node.kind() == "headline" {
        let stars = get_stars(node, content);
        if let Some(headline_text) = get_headline_text(node, content) {
            if config
                .keywords_finished
                .iter()
                .any(|keyword| headline_text.starts_with(keyword))
            {
                info!("found finished {}", headline_text);
                output.push_str(&content[*start_byte..node.start_byte()]);
                *edited = true;
                let mut subnode = cursor.node();
                loop {
                    // move to next sibling
                    if !cursor.goto_next_sibling() {
                        info!("no next sibling: {}", node.child_count());
                        *start_byte = subnode.end_byte();
                        return true;
                    }
                    subnode = cursor.node();
                    if subnode.kind() == "headline"
                        && get_stars(subnode, content).len() <= stars.len()
                    {
                        info!(
                            "found next headline: {:?}",
                            get_headline_text(subnode, content)
                        );
                        *start_byte = subnode.start_byte();
                        return true;
                    } else {
                        info!("skipping sub node");
                    }
                }
            }
        }
    }
    return false;
}

fn get_stars(node: tree_sitter::Node, content: &str) -> String {
    node.child_by_field_name("stars")
        .expect("Error getting stars")
        .utf8_text(content.as_bytes())
        .expect("Error getting stars text")
        .to_owned()
}

fn get_headline_text(node: tree_sitter::Node, content: &str) -> Option<String> {
    if let Some(item) = node.child_by_field_name("item") {
        Some(item.utf8_text(content.as_bytes()).ok()?.to_owned())
    } else {
        None
    }
}
fn print_tree(node: tree_sitter::Node, source_code: &str, indent: usize) {
    if node.kind() == "headline" {
        // Print the current node with indentation
        println!(
            "{:indent$}{} [{}]",
            "",
            node.kind(),
            node.utf8_text(source_code.as_bytes()).unwrap_or("").trim(),
            indent = indent
        );
    } else {
        println!("{:indent$}{}", "", node.kind(), indent = indent);
    }

    // Recursively print children with increased indentation
    for child in node.children(&mut node.walk()) {
        print_tree(child, source_code, indent + 2);
    }
}
