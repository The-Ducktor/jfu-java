use lazy_static::lazy_static;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

lazy_static! {
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref THEME_SET: ThemeSet = ThemeSet::load_defaults();
}

pub fn highlight_java_code(code: &str) -> String {
    let syntax = SYNTAX_SET
        .find_syntax_by_extension("java")
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
    let mut h = HighlightLines::new(syntax, &THEME_SET.themes["base16-ocean.dark"]);

    let mut highlighted = String::new();
    for line in LinesWithEndings::from(code) {
        let ranges = h.highlight_line(line, &SYNTAX_SET).unwrap();
        highlighted.push_str(&syntect::util::as_24_bit_terminal_escaped(&ranges, true));
    }
    highlighted
}
