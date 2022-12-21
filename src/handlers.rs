
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
        style {
            (include_str!("articles.css"))
        }

        body {

            h1 { "Welcome to 160R's blog!" }

            h2 { "Software" }

            a .software href="https://github.com/I60R/page" { "page" }

            a .software href="https://github.com/I60R/javelin" { "javelin" }

            h2 { "Articles" }

            main {

                @for (added, title) in articles {
                    a .article href=(format!("http://{ADDR}/blog/{title}")) {
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
    let (article_id, article_title, article_body, is_last) = db.fetch_article(&title);

    display_article(&article_id, &article_title, &article_body, is_last)
}

pub async fn next_article(
    State(db): State<crate::database::Database>,
    Path(id): Path<String>,
) -> axum::response::Redirect {
    let article_title = db.fetch_article_title_by_id(&format!("({id} + 1)"));
    axum::response::Redirect::permanent(&format!("http://{ADDR}/blog/{article_title}"))
}

pub async fn prev_article(
    State(db): State<crate::database::Database>,
    Path(id): Path<String>,
) -> axum::response::Redirect {
    let article_title = db.fetch_article_title_by_id(&format!("({id} - 1)"));
    axum::response::Redirect::permanent(&format!("http://{ADDR}/blog/{article_title}"))
}

fn display_article(article_id: &str, article_title: &str, article_body: &str, is_last: bool) -> axum::response::Html<String> {

    let article_title_decoded = urlencoding::decode(&article_title).unwrap();

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

    let next_link = format!("http://127.0.0.1:3000/blog/next/{article_id}");
    let prev_link = format!("http://127.0.0.1:3000/blog/prev/{article_id}");

    let markup = maud::html! {
        style {
          (include_str!("article.css"))
        }

        body {
            main {
                h1 { (article_title_decoded) }

                (maud::PreEscaped(output))
            }

            footer {
                @if article_id != "1" {
                    a href=(prev_link) { "prev" }
                } @else {
                    a style="color: transparent" { "prev" }
                }

                a href=(format!("http://{ADDR}")) { "⌂" }

                @if !is_last {
                    a href=(next_link) { "next" }
                } @else {
                    a style="color: transparent" { "next" }
                }
            }
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
