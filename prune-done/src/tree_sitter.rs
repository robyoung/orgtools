use crate::cli::Config;
use tracing::info;
use tree_sitter::{Node, TreeCursor};

pub fn modify_content(config: &Config, content: &str) -> String {
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
        let node = cursor.node();

        if !prune_done(config, cursor, node, content, output, start_byte, edited) {
            if cursor.goto_first_child() {
                traverse_and_modify(config, cursor, content, output, start_byte, edited);
                cursor.goto_parent();
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

fn prune_done(
    config: &Config,
    cursor: &mut TreeCursor,
    node: Node,
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
    false
}

fn get_stars(node: Node, content: &str) -> String {
    node.child_by_field_name("stars")
        .expect("Error getting stars")
        .utf8_text(content.as_bytes())
        .expect("Error getting stars text")
        .to_owned()
}

fn get_headline_text(node: Node, content: &str) -> Option<String> {
    if let Some(item) = node.child_by_field_name("item") {
        Some(item.utf8_text(content.as_bytes()).ok()?.to_owned())
    } else {
        None
    }
}

pub fn print_tree(node: Node, source_code: &str, indent: usize) {
    if node.kind() == "headline" {
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

    for child in node.children(&mut node.walk()) {
        print_tree(child, source_code, indent + 2);
    }
}
