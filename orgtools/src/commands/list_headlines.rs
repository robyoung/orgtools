use std::io;

use crate::{
    cli::Config,
    org::{Keyword, Org, Section},
    utils::fs::read_input,
};

pub fn list_headlines(config: &Config, input_file: Option<&str>) -> io::Result<()> {
    let input = read_input(input_file)?;
    let org = Org::new(config, &input);
    // TODO refactor so both Org and Section implement the same sections trait
    for section in org.sections() {
        print_section_headlines(&section);
    }

    Ok(())
}

fn print_section_headlines(section: &Section) {
    if let Keyword::Unfinished(_) = section.keyword() {
        if let Some(headline) = section.headline_text() {
            print!("{}", headline);
        }
    }
    for subsection in section.subsections() {
        print_section_headlines(&subsection);
    }
}
