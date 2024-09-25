//! API for interacting with Org mode files

use std::{cell::RefCell, rc::Rc};

use tree_sitter::{Node, Tree};

use crate::config::Config;
use crate::utils::get_parser;

pub struct Org {
    config: Config,
}

impl Org {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    pub fn from_config(config: Config) -> Self {
        Self { config }
    }

    pub fn keywords_unfinished(mut self, keywords: &[&str]) -> Self {
        self.config.keywords_unfinished = keywords.iter().map(|s| s.to_string()).collect();
        self
    }
    pub fn keywords_finished(mut self, keywords: &[&str]) -> Self {
        self.config.keywords_finished = keywords.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn load<'a>(&self, input: &'a str) -> OrgFile<'a> {
        OrgFile::new(self.config.clone(), input)
    }
}

pub struct OrgFile<'a> {
    config: Config,
    input: &'a str,
    #[allow(dead_code)]
    tree: Rc<RefCell<Tree>>,
    pub root: Node<'a>,
}

/// The main interface for interacting with Org mode files
impl<'a> OrgFile<'a> {
    pub fn new(config: Config, input: &'a str) -> Self {
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

        Self {
            config,
            input,
            tree,
            root,
        }
    }

    pub fn subsections(&'a self) -> Vec<Section<'a>> {
        get_subsections(&self.config, self.input, self.root)
    }

    pub fn find_section(&'a self, search: &str) -> Option<Section<'a>> {
        find_section(&self.config, self.input, self.root, search)
    }

    pub fn output_builder(&self) -> OutputBuilder {
        OutputBuilder::new(&self.config, self.input)
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

    pub fn body(&self) -> Option<Node<'a>> {
        self.node.child_by_field_name("body")
    }

    pub fn body_text(&self) -> Option<&'a str> {
        Some(self.body()?.utf8_text(self.input.as_bytes()).unwrap())
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
    config: &'a Config,
    input: &'a str,
    output: String,
    start_byte: usize,
}

impl<'a> OutputBuilder<'a> {
    pub fn new(config: &'a Config, input: &'a str) -> Self {
        Self {
            config,
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

    pub fn append_up_to_section(&mut self, section: &Section) {
        self.append_to(section.start_byte());
    }

    pub fn append_to_end_of_section(&mut self, section: &Section) {
        self.append_to(section.end_byte());
    }

    /// Append input up to the end of the input.
    ///
    /// Append from the current position up to the end of the input.
    /// This consumes the `OutputBuilder` there is no more input to
    /// append.
    pub fn append_to_end_of_input(mut self) -> String {
        self.output.push_str(&self.input[self.start_byte..]);
        self.output
    }

    pub fn skip_to(&mut self, byte: usize) {
        self.start_byte = byte;
    }

    pub fn insert_text(&mut self, text: &str) {
        self.output.push_str(text);
    }

    pub fn new_section(&mut self) -> SectionBuilder {
        SectionBuilder::new(self.config)
    }

    pub fn insert_section(&mut self, section: SectionBuilder) {
        self.output.push_str(&section.render());
    }
}

pub struct SectionBuilder<'a> {
    config: &'a Config,
    stars: usize,
    headline: String,
    keyword: Option<String>,
    body: Option<String>,
}

impl<'a> SectionBuilder<'a> {
    fn new(config: &'a Config) -> Self {
        Self {
            config,
            stars: 1,
            headline: String::new(),
            keyword: None,
            body: None,
        }
    }

    pub fn render(&self) -> String {
        let stars = "*".repeat(self.stars);
        let keyword = self
            .keyword
            .as_ref()
            .map(|keyword| format!(" {}", keyword))
            .unwrap_or_default();
        let headline: &str = self.headline.as_ref();
        let mut text = format!(
            r#"
{stars}{keyword} {headline}
"#
        );

        if let Some(body) = self.body.as_ref() {
            text.push_str(body);
        }
        text
    }

    pub fn stars(mut self, stars: usize) -> Self {
        self.stars = stars;
        self
    }

    pub fn keyword(mut self, keyword: &str) -> Self {
        let keyword = keyword.to_uppercase();
        if self.config.keywords_finished.contains(&keyword)
            || self.config.keywords_unfinished.contains(&keyword)
        {
            self.keyword = Some(keyword);
        }
        self
    }

    pub fn headline(mut self, headline: &str) -> Self {
        self.headline = headline.to_string();
        self
    }

    pub fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
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

        // When
        let org = Org::new().load(input);
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

        // When
        let org = Org::new().load(input);
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

        // When
        let org = Org::new()
            .keywords_finished(&["DONE"])
            .keywords_unfinished(&["TODO"])
            .load(input);
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

        // When
        let org = Org::new().load(input);
        let sections = org.subsections();
        assert_eq!(sections[0].stars(), 1);
        let subsections = sections[0].subsections();
        assert_eq!(subsections[0].stars(), 2);
    }

    #[test]
    fn test_find_section() {
        // Given
        let input = "* Headline 1\n** Headline 1.1\n* Headline 2\n";

        // When
        let org = Org::new().load(input);
        let section = org.find_section("Headline 1").unwrap();

        // Then
        assert_eq!(section.headline_text_full().unwrap(), "Headline 1");
    }
}
