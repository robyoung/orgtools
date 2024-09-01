//! Remove completed tasks from an org file.
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

struct State<'a> {
    config: &'a Config,
    input: &'a str,
    output: String,
    start_byte: usize,
    edited: bool,
    depth: usize,
}

impl<'a> State<'a> {
    fn new(config: &'a Config, input: &'a str) -> Self {
        State {
            config,
            input,
            output: String::new(),
            start_byte: 0,
            edited: false,
            depth: 0,
        }
    }

    fn push_output(&mut self, to_byte: usize) {
        self.output.push_str(&self.input[self.start_byte..to_byte]);
        self.edited = true;
    }

    fn push_output_to_end(&mut self) {
        self.output.push_str(&self.input[self.start_byte..]);
    }
}

pub fn walk_tree(config: &Config, input: &str) -> String {
    let mut parser = get_parser();
    let tree = parser.parse(&input, None).unwrap();
    let mut cursor = tree.walk();

    let mut state = State::new(config, input);

    walk(&mut state, &mut cursor);
    state.push_output_to_end();

    state.output
}

fn walk(state: &mut State, cursor: &mut TreeCursor) {
    loop {
        let node = cursor.node();
        let should_progress = do_prune(state, cursor, node);

        if should_progress {
            if cursor.goto_first_child() {
                state.depth += 1;
                walk(state, cursor);
                state.depth -= 1;
                cursor.goto_parent();
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

fn do_prune(state: &mut State, cursor: &mut TreeCursor, node: Node) -> bool {
    if node.kind() == "headline" {
        let stars = get_stars(node, state.input);
        if let Some(headline_text) = get_headline_text(node, state.input) {
            if is_done(state.config, &headline_text) {
                info!("found finished {}", headline_text);
                state.push_output(node.start_byte());
                let mut subnode = cursor.node();
                loop {
                    if !cursor.goto_next_sibling() {
                        info!("no next sibling: {}", node.child_count());
                        state.start_byte = subnode.end_byte();
                        return true;
                    }
                    subnode = cursor.node();
                    if subnode.kind() == "headline"
                        && get_stars(subnode, state.input).len() <= stars.len()
                    {
                        info!(
                            "found next headline: {:?}",
                            get_headline_text(subnode, state.input)
                        );
                        state.start_byte = subnode.start_byte();
                        return false;
                    } else {
                        info!("skipping sub node");
                    }
                }
            }
        }
    }
    true
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

    use crate::utils::set_up_logging;

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

    #[test]
    fn test_walk_tree_1() {
        let config = Config {
            keywords_finished: vec!["DONE".to_string(), "CANCELLED".to_string()],
            ..Default::default()
        };

        let input = "* TODO Task 1\n** DONE Subtask 1\n** Subtask 2\n* CANCELLED Task 2\n* Task 3";
        let expected_output = "* TODO Task 1\n** Subtask 2\n* Task 3";

        let result = walk_tree(&config, input);
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_walk_tree_2() {
        let config = Config {
            keywords_finished: vec!["DONE".to_string(), "CANCELLED".to_string()],
            ..Default::default()
        };

        let input = "* TODO Task 1\n** DONE Subtask 1\n*** Subtask 2\n* CANCELLED Task 2\n* Task 3";
        let expected_output = "* TODO Task 1\n* Task 3";

        let result = walk_tree(&config, input);
        assert_eq!(result, expected_output);
    }
}
