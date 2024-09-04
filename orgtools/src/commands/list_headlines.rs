use std::io;

use crate::{
    cli::Config,
    org::Headlines,
    utils::{fs::read_input, get_parser},
};

pub fn list_headlines(config: &Config, input_file: Option<&str>) -> io::Result<()> {
    let input = read_input(input_file)?;
    let mut parser = get_parser();
    let tree = parser.parse(&input, None).unwrap();

    for headline in Headlines::new(config, &input, tree.root_node()) {
        if headline.is_todo() {
            print!("{}", headline.get_full_text());
        }
    }

    Ok(())
}
