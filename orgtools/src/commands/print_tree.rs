use std::io;

use tree_sitter::Node;

use crate::{
    cli::Config,
    utils::{fs::read_input, get_parser},
};

pub fn print_tree(_config: &Config, input_file: Option<&str>, sexp: bool) -> io::Result<()> {
    let input = read_input(input_file)?;
    let mut parser = get_parser();
    let tree = parser.parse(&input, None).unwrap();
    if sexp {
        let formatted_sexp = format_sexp(&tree.root_node().to_sexp());
        println!("{}", formatted_sexp);

        fn format_sexp(sexp: &str) -> String {
            let mut result = String::new();
            let mut indent = 0;
            let mut in_string = false;

            for c in sexp.chars() {
                match c {
                    '(' if !in_string => {
                        if !result.is_empty() {
                            result.push('\n');
                            result.push_str(&" ".repeat(indent));
                        }
                        result.push(c);
                        indent += 2;
                    }
                    ')' if !in_string => {
                        indent = indent.saturating_sub(2);
                        result.push('\n');
                        result.push_str(&" ".repeat(indent));
                        result.push(c);
                    }
                    '"' => {
                        in_string = !in_string;
                        result.push(c);
                    }
                    ' ' if !in_string => {
                        if !result.ends_with(' ') && !result.ends_with('\n') {
                            result.push(' ');
                        }
                    }
                    _ => result.push(c),
                }
            }
            result
        }
    } else {
        inner_print_tree(tree.root_node(), &input, 0);
    }

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
