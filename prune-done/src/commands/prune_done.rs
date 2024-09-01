use tracing::info;
use tree_sitter::{Node, TreeCursor};

use crate::cli::Config;
use crate::utils::fs::{read_input, write_output};
use crate::utils::{get_headline_text, get_parser, get_stars, is_done};
use std::io;

pub fn prune_done(
    config: &Config,
    input_file: Option<&str>,
    output_file: Option<&str>,
) -> io::Result<()> {
    let input = read_input(input_file)?;
    let output = modify_content(&config, &input);
    write_output(input_file, output_file, &output)?;

    Ok(())
}

pub fn modify_content(config: &Config, content: &str) -> String {
    let mut parser = get_parser();
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
        0,
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
    depth: usize,
) {
    loop {
        let node = cursor.node();

        if !inner_prune_done(config, cursor, node, content, output, start_byte, edited) {
            if cursor.goto_first_child() {
                traverse_and_modify(
                    config,
                    cursor,
                    content,
                    output,
                    start_byte,
                    edited,
                    depth + 1,
                );
                cursor.goto_parent();
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

fn inner_prune_done(
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
            if is_done(config, &headline_text) {
                info!("found finished {}", headline_text);
                output.push_str(&content[*start_byte..node.start_byte()]);
                *edited = true;
                let mut subnode = cursor.node();
                loop {
                    if !cursor.goto_next_sibling() {
                        info!("no next sibling: {}", node.child_count());
                        *start_byte = subnode.end_byte();
                        return false;
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_modify_content() {
        let config = Config {
            keywords_finished: vec!["DONE".to_string(), "CANCELLED".to_string()],
            ..Default::default()
        };

        let input = "* TODO Task 1\n** DONE Subtask 1\n** Subtask 2\n* CANCELLED Task 2\n* Task 3";
        let expected_output = "* TODO Task 1\n** Subtask 2\n* Task 3";

        let result = modify_content(&config, input);
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_remove_subtasks() {
        let config = Config {
            keywords_finished: vec!["DONE".to_string(), "CANCELLED".to_string()],
            ..Default::default()
        };

        let input = "* TODO Task 1\n** DONE Subtask 1\n*** Subtask 2\n* CANCELLED Task 2\n* Task 3";
        let expected_output = "* TODO Task 1\n* Task 3";

        let result = modify_content(&config, input);
        assert_eq!(result, expected_output);
    }
}
