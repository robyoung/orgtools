use std::{fs, io};

use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use tree_sitter::TreeCursor;
use tree_sitter_org;

struct Config {
    keywords_unfinished: Vec<String>,
    keywords_finished: Vec<String>,
}

fn main() {
    // set up tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let config = Config {
        keywords_unfinished: vec![
            "TODO".to_string(),
            "DOING".to_string(),
            "BLOCKED".to_string(),
        ],
        keywords_finished: vec!["DONE".to_string(), "ABANDONED".to_string()],
    };

    // set up tree-sitter parser
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(tree_sitter_org::language())
        .expect("Error loading Org language");
    let content = fs::read_to_string("./Current.org").expect("Error reading file");
    let tree = parser.parse(&content, None).unwrap();
    // print_tree(tree.root_node(), &content, 0);
    // println!("{}", tree.root_node().to_sexp());
    // return;
    let mut edited = false;
    let mut output = String::new();
    let mut start_byte = 0;

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
    print!("{}", output);
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
