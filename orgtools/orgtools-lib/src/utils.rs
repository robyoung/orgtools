use tree_sitter::{Language, Parser};

pub(crate) fn get_parser() -> Parser {
    get_parser_and_language().0
}

pub(crate) fn get_parser_and_language() -> (Parser, Language) {
    let mut parser = Parser::new();
    let language = get_language();
    parser
        .set_language(language)
        .expect("Error loading Org language");
    (parser, language)
}

pub(crate) fn get_language() -> tree_sitter::Language {
    tree_sitter_org::language()
}
