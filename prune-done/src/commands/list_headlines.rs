use std::io;

use tracing::info;
use tree_sitter::{Query, QueryCursor};

use crate::{
    cli::Config,
    utils::{fs::read_input, get_headline_text, get_parser_and_language, is_todo},
};

pub fn list_headlines(config: &Config, input_file: Option<&str>) -> io::Result<()> {
    let input = read_input(input_file)?;
    let (mut parser, language) = get_parser_and_language();

    // Parse source
    let tree = parser.parse(&input, None).unwrap();
    let root_node = tree.root_node();

    // Define query
    let query = Query::new(language, "(headline) @headline").expect("Invalid query");
    let mut query_cursor = QueryCursor::new();

    // Execute query
    for (i, (m, capture_ix)) in query_cursor
        .captures(&query, root_node, input.as_bytes())
        .enumerate()
    {
        for (j, node) in m.nodes_for_capture_index(capture_ix as u32).enumerate() {
            if let Some(headline_text) = get_headline_text(node, &input) {
                if is_todo(config, &headline_text) {
                    let headline = node.utf8_text(input.as_bytes()).unwrap();
                    print!("{} {} {} {}", i, j, capture_ix, headline);
                }
            }
        }
    }

    Ok(())
}
