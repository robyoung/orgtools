//! Remove completed tasks from an org file.

use crate::cli::Config;
use crate::org::{Headline, Headlines, Keyword, Org, OutputBuilder, Section};
use crate::utils::fs::{read_input, write_output};
use crate::utils::get_parser;
use std::io;

pub fn prune_done(
    config: &Config,
    input_file: Option<&str>,
    output_file: Option<&str>,
) -> io::Result<()> {
    let input = read_input(input_file)?;

    let output = prune_done_inner(config, &input);

    write_output(input_file, output_file, &output)?;

    Ok(())
}

fn prune_done_inner(config: &Config, input: &str) -> String {
    let mut parser = get_parser();
    let tree = parser.parse(&input, None).unwrap();
    let mut output = OutputBuilder::new(&input);

    let headlines = Headlines::new(config, &input, tree.root_node());
    let mut current_done_headline: Option<Headline<'_>> = None;
    for headline in headlines {
        if current_done_headline.is_some() {
            if headline.is_child_of(&current_done_headline.as_ref().unwrap()) {
                continue;
            } else {
                // all children handled move start byet to start of this headline
                current_done_headline = None;
                output.skip_to_headline(&headline);
            }
        }
        if headline.is_done() {
            // append everything from previous point to the start of this headline
            output.append_to_headline(&headline);

            current_done_headline = Some(headline);
        } else {
            output.skip_to_headline(&headline);
        }
    }

    output.append_to_end()
}

pub fn prune_done2(
    config: &Config,
    input_file: Option<&str>,
    output_file: Option<&str>,
) -> io::Result<()> {
    let input = read_input(input_file)?;

    let output = prune_done2_inner(&config, &input);

    write_output(input_file, output_file, &output)?;

    Ok(())
}

fn prune_done2_inner(config: &Config, input: &str) -> String {
    let org = Org::new(config, input);
    let mut builder = OutputBuilder::new(input);
    for section in org.sections() {
        prune_done2_inner2(&section, &mut builder);
    }
    builder.append_to_end()
}

fn prune_done2_inner2(section: &Section, builder: &mut OutputBuilder) {
    if let Keyword::Finished(_) = section.keyword() {
        builder.append_to(section.start_byte());
        builder.skip_to(section.end_byte());
    } else {
        for subsection in section.subsections() {
            prune_done2_inner2(&subsection, builder);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_prune_done() {
        let config = Config {
            keywords_finished: vec!["DONE".to_string(), "CANCELLED".to_string()],
            ..Default::default()
        };

        let input = "* TODO Task 1\n** DONE Subtask 1\n** Subtask 2\n* CANCELLED Task 2\n* Task 3";
        let expected_output = "* TODO Task 1\n** Subtask 2\n* Task 3";

        let result = prune_done_inner(&config, input);
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_prune_done_sub_tasks() {
        let config = Config {
            keywords_finished: vec!["DONE".to_string(), "CANCELLED".to_string()],
            ..Default::default()
        };

        let input = "* TODO Task 1\n** DONE Subtask 1\n*** Subtask 2\n* CANCELLED Task 2\n* Task 3";
        let expected_output = "* TODO Task 1\n* Task 3";

        let result = prune_done_inner(&config, input);
        assert_eq!(result, expected_output);
    }

    // TODO: Add tests that include properties, drawers, file preamble and body text.
    #[test]
    fn test_prune_done2() {
        let config = Config {
            keywords_finished: vec!["DONE".to_string(), "CANCELLED".to_string()],
            ..Default::default()
        };

        let input = "* TODO Task 1\n** DONE Subtask 1\n** Subtask 2\n* CANCELLED Task 2\n* Task 3";
        let expected_output = "* TODO Task 1\n** Subtask 2\n* Task 3";

        let result = prune_done2_inner(&config, input);
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_prune_done2_sub_tasks() {
        let config = Config {
            keywords_finished: vec!["DONE".to_string(), "CANCELLED".to_string()],
            ..Default::default()
        };

        let input = "* TODO Task 1\n** DONE Subtask 1\n*** Subtask 2\n* CANCELLED Task 2\n* Task 3";
        let expected_output = "* TODO Task 1\n* Task 3";

        let result = prune_done2_inner(&config, input);
        assert_eq!(result, expected_output);
    }
}
