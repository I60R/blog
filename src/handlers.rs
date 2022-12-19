
use axum::{
    extract::*,
    http,
};

const ADDR: &str = "127.0.0.1:3000";


pub async fn get_articles(
    State(db): State<crate::database::Database>,
) -> axum::response::Html<String> {

    let articles = db.fetch_articles();
    let articles = articles
        .iter()
        .map(|(added, title)| {

            let title = urlencoding::decode(title).unwrap();

            (added, title)
        });


    let markup = maud::html! {
        div style="display: flex; flex-direction: column; padding: 5em; width: 60em" {
            h1 { "Welcome to 160R's blog!" }
            h2 { "Software" }
            a href="https://github.com/I60R/page" style="font-weight: bold" {
                "page"
            }
            a href="https://github.com/I60R/javelin" style="font-weight: bold" {
                "javelin"
            }
            h2 { "Articles" }
            div style="display: flex; flex-direction: column" {
                @for (added, title) in articles {
                    a
                        href=(format!("http://{ADDR}/blog/{title}"))
                        style="padding-top: 0.5em"
                    {
                        (format!("{added} - {title}\n"))
                    }
                }
            }
        }
    };

    let markup = markup.into_string();
    axum::response::Html::from(markup)
}



pub async fn get_article(
    State(db): State<crate::database::Database>,
    Path(title): Path<String>,
) -> axum::response::Html<String> {

    let title = urlencoding::encode(&title);
    let (article_title, article_body) = db.fetch_article(&title);

    let article_title = urlencoding::decode(&article_title).unwrap();

    let article_body = base64::decode(article_body).unwrap();
    let article_body = String::from_utf8(article_body).unwrap();

    use syntect::{
        highlighting::ThemeSet,
        html::highlighted_html_for_string,
        parsing::SyntaxSet
    };
    use pulldown_cmark::{html, Event, Parser, Tag, Options, CowStr};


    // Setup for pulldown_cmark to read (only) from stdin
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
                    let html = highlighted_html_for_string(&to_highlight, &ss, &syntax, &theme)
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

    let markup = maud::html! {
        div style="padding: 5em; width: 60em; font-family: Helvetica" {
            h1 { (article_title) }

            (maud::PreEscaped(output))
        }
    };

    let markup = markup.into_string();
    axum::response::Html::from(markup)
}



pub async fn create_article(
    State(st): State<crate::database::Database>,
    Path(title): Path<String>,
    headers: axum::http::header::HeaderMap,
    body: String,
) -> http::StatusCode {

    let Some(hv) = headers.get("authorization") else {
        return http::StatusCode::UNAUTHORIZED
    };
    let Ok("Basic YWRtaW46YWRtaW4=") = hv.to_str() else {
        return http::StatusCode::UNAUTHORIZED
    };

    let title = urlencoding::encode(&title);
    let body = base64::encode(body);

    if st.create_article(&title, &body) {
        http::StatusCode::CREATED
    } else {
        http::StatusCode::CONFLICT
    }
}

pub async fn delete_article(
    State(st): State<crate::database::Database>,
    Path(title): Path<String>,
    headers: axum::http::header::HeaderMap,
) -> http::StatusCode {

    let Some(hv) = headers.get("authorization") else {
        return http::StatusCode::UNAUTHORIZED
    };
    let Ok("Basic YWRtaW46YWRtaW4=") = hv.to_str() else {
        return http::StatusCode::UNAUTHORIZED
    };

    let title = urlencoding::encode(&title);

    if st.delete_article(&title) {
        http::StatusCode::OK
    } else {
        http::StatusCode::NO_CONTENT
    }
}
