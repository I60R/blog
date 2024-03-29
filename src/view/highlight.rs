use syntect::{
    highlighting::ThemeSet,
    html::highlighted_html_for_string,
    parsing::SyntaxSet
};
use pulldown_cmark::{html, Event, Parser, Tag, Options, CowStr};

pub fn code_blocks(article_body: &str) -> String {
    let opts = Options::empty();
    let mut output = String::with_capacity(article_body.len() * 3 / 2);
    let parser = Parser::new_ext(&article_body, opts);

    // Setup for syntect to highlight (specifically) Rust code
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ss.find_syntax_by_extension("rs").unwrap();
    let theme = &ts.themes["InspiredGitHub"];

    // We'll build a new vector of events since we can only consume the parser once
    let mut new_parser = Vec::new();
    // As we go along, we'll want to highlight code in bundles, not lines
    let mut to_highlight = String::new();
    // And track a little bit of state
    let mut in_code_block = false;

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(_)) => {
                // In actual use you'd probably want to keep track of what language this code is
                in_code_block = true;
            }
            Event::End(Tag::CodeBlock(_)) => {
                if in_code_block {
                    // Format the whole multi-line code block as HTML all at once
                    let html = highlighted_html_for_string(&to_highlight, &ss, syntax, theme)
                        .expect("cannot highlight");
                    // And put it into the vector
                    new_parser.push(Event::Html(CowStr::from(html)));
                    to_highlight = String::new();
                    in_code_block = false;
                }
            }
            Event::Text(t) => {
                if in_code_block {
                    // If we're in a code block, build up the string of text
                    to_highlight.push_str(&t);
                } else {
                    new_parser.push(Event::Text(t))
                }
            }

            e => {
                new_parser.push(e);
            }
        }
    }

    // Now we send this new vector of events off to be transformed into HTML
    html::push_html(&mut output, new_parser.into_iter());

    output
}
