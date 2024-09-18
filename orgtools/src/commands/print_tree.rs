use std::io;

use tree_sitter::{Node, Point};

use crate::{
    cli::Config,
    org::{Org, Section},
    utils::{fs::read_input, get_parser},
};

pub fn print_tree(
    config: &Config,
    input_file: Option<&str>,
    sexp: bool,
    sections: bool,
) -> io::Result<()> {
    let input = read_input(input_file)?;
    let mut parser = get_parser();
    let tree = parser.parse(&input, None).unwrap();
    if sexp {
        print_sexp_tree(tree.root_node());
    } else if sections {
        let org = Org::new(config, &input);
        print_sections(&org);
    } else {
        print_manual_tree(tree.root_node(), &input, 0);
    }

    Ok(())
}

fn print_sexp_tree(node: Node) {
    let sexp = node.to_sexp();
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
    println!("{}", result);
}

fn print_sections(org: &Org) {
    for section in org.subsections() {
        print_section(&section, 0);
    }
}

fn print_section(section: &Section, indent: usize) {
    println!(
        "{:indent$}{} {} - {}  ::  ({} - {})",
        "",
        section.headline_text_full().unwrap_or(""),
        format_point(section.node().start_position()),
        format_point(section.node().end_position()),
        section.node().start_byte(),
        section.node().end_byte(),
        indent = indent
    );
    for subsection in section.subsections() {
        print_section(&subsection, indent + 2);
    }
}

fn format_point(point: Point) -> String {
    format!("[{}, {}]", point.row, point.column)
}

fn print_manual_tree(node: Node, source_code: &str, indent: usize) {
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
        print_manual_tree(child, source_code, indent + 2);
    }
}
