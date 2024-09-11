//! Remove completed tasks from an org file.

use crate::cli::Config;
use crate::org::{Keyword, Org, OutputBuilder, Section};
use crate::utils::fs::{read_input, write_output};
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
    let org = Org::new(config, input);
    let mut builder = OutputBuilder::new(input);
    for section in org.sections() {
        prune_done_inner2(&section, &mut builder);
    }
    builder.append_to_end()
}

fn prune_done_inner2(section: &Section, builder: &mut OutputBuilder) {
    if let Keyword::Finished(_) = section.keyword() {
        builder.append_to(section.start_byte());
        builder.skip_to(section.end_byte());
    } else {
        for subsection in section.subsections() {
            prune_done_inner2(&subsection, builder);
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
}
