//! API for interacting with Org mode files

use std::{cell::RefCell, rc::Rc};

use tree_sitter::{Node, Tree};

use crate::{cli::Config, utils::get_parser};

pub struct Org<'a> {
    config: &'a Config,
    input: &'a str,
    #[allow(dead_code)]
    tree: Rc<RefCell<Tree>>,
    root: Node<'a>,
}

/// The main interface for interacting with Org mode files
impl<'a> Org<'a> {
    pub fn new(config: &'a Config, input: &'a str) -> Self {
        let mut parser = get_parser();
        let tree = Rc::new(RefCell::new(
            parser.parse(input, None).expect("Error parsing Org file."),
        ));

        let root = {
            let tree_ref = tree.borrow();
            let root = tree_ref.root_node();
            // Safety: Transmuting the lifetime of the tree and cursor to 'static is safe because
            //          tree, root and cursor are all owned by the Org struct and will not outlive
            //          it.
            unsafe { std::mem::transmute::<Node, Node<'static>>(root) }
        };

        Org {
            config,
            input,
            tree,
            root,
        }
    }

    pub fn subsections(&'a self) -> Vec<Section<'a>> {
        get_subsections(self.config, self.input, self.root)
    }

    pub fn find_section(&'a self, search: &str) -> Option<Section<'a>> {
        find_section(self.config, self.input, self.root, search)
    }
}

pub struct Section<'a> {
    config: &'a Config,
    input: &'a str,
    node: Node<'a>,
}

impl<'a> Section<'a> {
    pub fn headline(&self) -> Option<Node<'a>> {
        self.node.child_by_field_name("headline")
    }

    /// Returns the full headline text including the keyword.
    ///
    /// Does not include the stars.
    pub fn headline_text_full(&self) -> Option<&'a str> {
        let headline_text = self.headline()?.child_by_field_name("item")?;
        Some(headline_text.utf8_text(self.input.as_bytes()).unwrap())
    }

    /// Returns the headline text without any keyword.
    pub fn headline_text(&self) -> Option<&'a str> {
        let headline = match self.keyword() {
            Keyword::Finished(keyword) | Keyword::Unfinished(keyword) => {
                let keyword_len = keyword.len();
                self.headline_text_full()
                    .map(|text| text[keyword_len..].trim())
            }
            Keyword::None => self.headline_text_full(),
        };
        headline.map(|text| text.trim())
    }

    pub fn subsections(&self) -> Vec<Section<'a>> {
        get_subsections(self.config, self.input, self.node)
    }

    pub fn start_byte(&self) -> usize {
        self.node.start_byte()
    }

    pub fn end_byte(&self) -> usize {
        self.node.end_byte()
    }

    pub fn node(&self) -> Node<'a> {
        self.node
    }

    pub fn keyword(&self) -> Keyword {
        if let Some(headline_text) = self.headline_text_full() {
            if let Some(keyword) = self.find_keyword(&self.config.keywords_finished, headline_text)
            {
                return Keyword::Finished(keyword);
            }

            if let Some(keyword) =
                self.find_keyword(&self.config.keywords_unfinished, headline_text)
            {
                return Keyword::Unfinished(keyword);
            }
        }
        Keyword::None
    }

    fn find_keyword(&self, keywords: &[String], headline_text: &str) -> Option<String> {
        keywords
            .iter()
            .find(|&keyword| headline_text.starts_with(keyword))
            .cloned()
    }

    pub fn stars(&self) -> usize {
        let stars = self
            .headline()
            .expect("Error getting headline")
            .child_by_field_name("stars")
            .expect("Error getting stars");
        stars.end_byte() - stars.start_byte()
    }
}

fn get_subsections<'a>(config: &'a Config, input: &'a str, node: Node<'a>) -> Vec<Section<'a>> {
    let mut cursor = node.walk();
    node.children_by_field_name("subsection", &mut cursor)
        .map(|node| Section {
            config,
            input,
            node,
        })
        .collect()
}

fn find_section<'a>(
    config: &'a Config,
    input: &'a str,
    node: Node<'a>,
    search: &str,
) -> Option<Section<'a>> {
    for section in get_subsections(config, input, node) {
        let headline_text = section.headline_text()?;
        if headline_text.eq(search) {
            return Some(section);
        }
        if let Some(subsection) = find_section(config, input, section.node, search) {
            return Some(subsection);
        }
    }
    None
}

#[derive(Debug, PartialEq)]
pub enum Keyword {
    Finished(String),
    Unfinished(String),
    None,
}

#[derive(Debug)]
pub enum Position {
    Under,
    After,
}

pub struct OutputBuilder<'a> {
    input: &'a str,
    output: String,
    start_byte: usize,
}

impl<'a> OutputBuilder<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            output: String::new(),
            start_byte: 0,
        }
    }

    /// Append input up to a given byte position and advance the start byte
    pub fn append_to(&mut self, end_byte: usize) {
        self.output.push_str(&self.input[self.start_byte..end_byte]);
        self.start_byte = end_byte;
    }

    /// Append input up to the end of the input.
    ///
    /// Append from the current position up to the end of the input.
    /// This consumes the `OutputBuilder` there is no more input to
    /// append.
    pub fn append_to_end(mut self) -> String {
        self.output.push_str(&self.input[self.start_byte..]);
        self.output
    }

    pub fn skip_to(&mut self, byte: usize) {
        self.start_byte = byte;
    }

    pub fn insert_text(&mut self, text: &str) {
        self.output.push_str(text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterate_flat_sections() {
        // Given
        let input = r#"
* Hedline 1
* Hedline 2
"#
        .trim();
        let config = Config::default();

        // When
        let org = Org::new(&config, input);
        let headlines = org
            .subsections()
            .into_iter()
            .filter_map(|section| section.headline_text_full())
            .collect::<Vec<_>>();
        assert_eq!(
            headlines,
            vec![String::from("Hedline 1"), String::from("Hedline 2")]
        );
    }

    #[test]
    fn test_iterate_nested_sections() {
        // Given
        let input = r#"
* Hedline 1
** Hedline 1.1
"#
        .trim();
        let config = Config::default();

        // When
        let org = Org::new(&config, input);
        let headlines = org
            .subsections()
            .into_iter()
            .filter_map(|section| section.headline_text_full())
            .collect::<Vec<_>>();
        assert_eq!(headlines, vec![String::from("Hedline 1")]);
    }

    #[test]
    fn test_get_section_headline_keyword() {
        // Given
        let input = "* DONE Headline 1\n* TODO Headline 2\n* Headline 3\n* \n";
        let config = Config {
            keywords_finished: vec![String::from("DONE")],
            keywords_unfinished: vec![String::from("TODO")],
        };

        // When
        let org = Org::new(&config, input);
        let sections = org.subsections();
        let keywords = sections
            .iter()
            .map(|section| section.keyword())
            .collect::<Vec<_>>();
        assert_eq!(
            keywords,
            vec![
                Keyword::Finished(String::from("DONE")),
                Keyword::Unfinished(String::from("TODO")),
                Keyword::None,
                Keyword::None,
            ],
        );
    }

    #[test]
    fn test_get_section_stars() {
        // Given
        let input = "* Headline 1\n** Headline 1.1\n* Headline 2\n";
        let config = Config::default();

        // When
        let org = Org::new(&config, input);
        let sections = org.subsections();
        assert_eq!(sections[0].stars(), 1);
        let subsections = sections[0].subsections();
        assert_eq!(subsections[0].stars(), 2);
    }

    #[test]
    fn test_find_section() {
        // Given
        let input = "* Headline 1\n** Headline 1.1\n* Headline 2\n";
        let config = Config::default();

        // When
        let org = Org::new(&config, input);
        let section = org.find_section("Headline 1").unwrap();

        // Then
        assert_eq!(section.headline_text_full().unwrap(), "Headline 1");
    }
}
