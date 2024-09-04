use std::io;

use tree_sitter::Node;

use crate::{
    cli::Config,
    utils::{fs::read_input, get_parser},
};

pub fn print_tree(_config: &Config, input_file: Option<&str>) -> io::Result<()> {
    let input = read_input(input_file)?;
    let mut parser = get_parser();
    let tree = parser.parse(&input, None).unwrap();
    inner_print_tree(tree.root_node(), &input, 0);

    Ok(())
}

pub fn inner_print_tree(node: Node, source_code: &str, indent: usize) {
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
        inner_print_tree(child, source_code, indent + 2);
    }
}
