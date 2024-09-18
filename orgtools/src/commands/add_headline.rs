use std::io;

use crate::{
    cli::Config,
    org::{Org, OutputBuilder, Position, Section},
    utils::fs::{read_input, write_output},
};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};

pub fn add_headline(
    config: &Config,
    input_file: Option<&str>,
    output_file: Option<&str>,
    headline: &str,
    position: Position,
    search: &str,
) -> Result<()> {
    let input = read_input(input_file)?;
    let output = add_headline_to_input(config, &input, headline, position, search)?;

    write_output(input_file, output_file, &output)?;

    Ok(())
}

fn add_headline_to_input(
    config: &Config,
    input: &str,
    headline: &str,
    position: Position,
    search: &str,
) -> Result<String> {
    let org = Org::new(config, input);
    let mut builder = OutputBuilder::new(input);
    if let Some(section) = org.find_section(search) {
        add_headline_to_section(&section, &mut builder, headline, position)?;
        Ok(builder.append_to_end())
    } else {
        Err(anyhow!("Could not find section with headline: {}", search))
    }
}

fn add_headline_to_section(
    section: &Section,
    builder: &mut OutputBuilder,
    headline: &str,
    position: Position,
) -> Result<()> {
    builder.append_to(section.end_byte());
    let num_stars = match position {
        Position::After => section.stars(),
        Position::Under => section.stars() + 1,
    };

    // :PROPERTIES:
    // :CREATED: [2021-08-15 Sun 14:00]
    // :END:
    builder.insert_text(&make_headline(num_stars, headline));
    Ok(())
}

fn make_headline(num_stars: usize, headline: &str) -> String {
    let now: DateTime<Utc> = Utc::now();
    let stamp = now.format("%Y-%m-%d %a %H:%M").to_string();
    let stars = "*".repeat(num_stars);

    format!(
        r#"
{stars} {headline}
:PROPERTIES:
:CREATED: [{stamp}]
:END:
"#
    )
    .trim_start()
    .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_add_headline(
        input: &str,
        search: &str,
        headline: &str,
        position: Position,
        expected_output: &str,
    ) {
        let config = Config {
            keywords_finished: vec!["DONE".to_string(), "CANCELLED".to_string()],
            keywords_unfinished: vec!["TODO".to_string()],
        };
        let result = add_headline_to_input(&config, input, headline, position, search).unwrap();
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_add_headline_after() {
        let input = "* TODO Task 1\n** DONE Subtask 1\n** Subtask 2\n* CANCELLED Task 2\n* Task 3";
        let search = "Task 1";
        let headline = "New Task";
        let expected_headline = make_headline(1, headline);
        let expected_output =
            format!("* TODO Task 1\n** DONE Subtask 1\n** Subtask 2\n{expected_headline}* CANCELLED Task 2\n* Task 3");
        assert_add_headline(input, search, headline, Position::After, &expected_output);
    }

    #[test]
    fn test_add_headline_under() {
        let input = "* TODO Task 1\n** DONE Subtask 1\n** Subtask 2\n* CANCELLED Task 2\n* Task 3";
        let search = "Task 1";
        let headline = "New Task";
        let expected_headline = make_headline(2, headline);
        let expected_output = format!("* TODO Task 1\n** DONE Subtask 1\n** Subtask 2\n{expected_headline}* CANCELLED Task 2\n* Task 3");
        assert_add_headline(input, search, headline, Position::Under, &expected_output);
    }
}
