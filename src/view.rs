use crate::{
    article,
    ADDR
};


pub fn display_articles(articles: Vec<article::ListItem>) -> String {
    let markup = maud::html! {
        style {
            (include_str!("../css/articles.css"))
        }

        title { "160R blog"  }
        link rel="icon" href=(
            format!("data:image/svg+xml,{}", urlencoding::encode(include_str!("../assets/favicon.svg")))
        ) { }

        body {

            h1 { "Welcome to 160R's blog!" }

            h2 { "Software" }

            a .software href="https://github.com/I60R/page" { "page" }

            a .software href="https://github.com/I60R/javelin" { "javelin" }

            h2 { "Articles" }

            main {

                @for article::ListItem { added, title } in articles {
                    a .article href=(format!("http://{ADDR}/blog/{title}")) {
                        (format!("{added}  •  {title}\n"))
                    }
                }
            }
        }
    };

    markup.into_string()
}


pub fn display_article(article_item: article::Item) -> String {
    let article_title_decoded = urlencoding::decode(&article_item.title)
        .unwrap();

    let article_body = base64::decode(article_item.body)
        .unwrap();
    let article_body = String::from_utf8(article_body)
        .unwrap();

    let markdown_render = display_article::parse_markdown_with_code_blocks(&article_body);

    let markup = maud::html! {
        style {
            (include_str!("../css/article.css"))
        }

        title { (article_title_decoded) }
        link rel="icon" href=(
            format!("data:image/svg+xml,{}", urlencoding::encode(include_str!("../assets/favicon.svg")))
        ) { }

        body {
            main {
                h1 { (article_title_decoded) }

                (maud::PreEscaped(markdown_render))
            }

            footer {
                @if !article_item.is_last {
                    a href=(
                        format!("http://{ADDR}/blog/next/{}", article_item.id)
                    ) { "⇧" }
                } @else {
                    a { "⏺" }
                }

                a href=(format!("http://{ADDR}")) { "⌂" }

                @if !article_item.is_first {
                    a href=(
                        format!("http://{ADDR}/blog/prev/{}", article_item.id)
                    ) { "⇩" }
                } @else {
                    a { "X" }
                }
           }
        }
    };

    markup.into_string()
}

mod display_article {
    use syntect::{
        highlighting::ThemeSet,
        html::highlighted_html_for_string,
        parsing::SyntaxSet
    };
    use pulldown_cmark::{html, Event, Parser, Tag, Options, CowStr};

    pub fn parse_markdown_with_code_blocks(article_body: &str) -> String {

        let opts = Options::empty();
        let mut output = String::with_capacity(article_body.len() * 3 / 2);
        let parser = Parser::new_ext(&article_body, opts);

        // Setup for syntect to highlight (specifically) Rust code
        let ss = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let syntax = ss.find_syntax_by_extension("rs").unwrap();
        let theme = &ts.themes["InspiredGitHub"];

        // We'll build a new vector of events since we can only consume the parser once
        let mut new_p = Vec::new();
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
                        new_p.push(Event::Html(CowStr::from(html)));
                        to_highlight = String::new();
                        in_code_block = false;
                    }
                }
                Event::Text(t) => {
                    if in_code_block {
                        // If we're in a code block, build up the string of text
                        to_highlight.push_str(&t);
                    } else {
                        new_p.push(Event::Text(t))
                    }
                }

                e => {
                    new_p.push(e);
                }
            }
        }

        // Now we send this new vector of events off to be transformed into HTML
        html::push_html(&mut output, new_p.into_iter());

        output
    }
}
