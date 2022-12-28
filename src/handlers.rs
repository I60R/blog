
use axum::{
    extract::*,
    http,
};
use crate::database::{
    ArticleListItem,
    ArticleItem,
};

const ADDR: &str = "127.0.0.1:3000";


pub async fn get_articles(
    State(db): State<crate::database::Database>,
) -> axum::response::Html<String> {

    let mut articles = db.fetch_articles().await;
    let articles = articles
        .iter_mut()
        .map(|mut article_list_item| {
            article_list_item.title = urlencoding::decode(&article_list_item.title)
                .unwrap()
                .to_string();
            article_list_item
        });


    let markup = maud::html! {
        style {
            (include_str!("articles.css"))
        }

        title { "160R blog"  }
        link rel="icon" href=(
            format!("data:image/svg+xml,{}", urlencoding::encode(include_str!("favicon.svg")))
        ) { }

        body {

            h1 { "Welcome to 160R's blog!" }

            h2 { "Software" }

            a .software href="https://github.com/I60R/page" { "page" }

            a .software href="https://github.com/I60R/javelin" { "javelin" }

            h2 { "Articles" }

            main {

                @for ArticleListItem { added, title } in articles {
                    a .article href=(format!("http://{ADDR}/blog/{title}")) {
                        (format!("{added}  •  {title}\n"))
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
    let aricle_item = db.fetch_article(&title).await;

    display_article(aricle_item)
}

pub async fn next_article(
    State(db): State<crate::database::Database>,
    Path(id): Path<i64>,
) -> axum::response::Redirect {
    let article_title = db
        .fetch_next_article_title_after_id(id)
        .await;
    println!("{article_title}");
    axum::response::Redirect::permanent(&format!("http://{ADDR}/blog/{article_title}"))
}

pub async fn prev_article(
    State(db): State<crate::database::Database>,
    Path(id): Path<i64>,
) -> axum::response::Redirect {
    let article_title = db
        .fetch_prev_article_title_before_id(id)
        .await;
    println!("{article_title}");
    axum::response::Redirect::permanent(&format!("http://{ADDR}/blog/{article_title}"))
}

fn display_article(article_item: ArticleItem) -> axum::response::Html<String> {
    let article_title_decoded = urlencoding::decode(&article_item.title)
        .unwrap();

    let article_body = base64::decode(article_item.body)
        .unwrap();
    let article_body = String::from_utf8(article_body)
        .unwrap();

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
        style {
            (include_str!("article.css"))
        }

        title { (article_title_decoded) }
        link rel="icon" href=(
            format!("data:image/svg+xml,{}", urlencoding::encode(include_str!("favicon.svg")))
        ) { }

        body {
            main {
                h1 { (article_title_decoded) }

                (maud::PreEscaped(output))
            }

            footer {
                @if !article_item.is_first {
                    a href=(
                        format!("http://{ADDR}/blog/prev/after_id={}", article_item.id)
                    ) { "prev" }
                } @else {
                    a { "end" }
                }

                a href=(format!("http://{ADDR}")) { "⌂" }

                @if !article_item.is_last {
                    a href=(
                        format!("http://{ADDR}/blog/next/after_id={}", article_item.id)
                    ) { "next" }
                } @else {
                    a { "end" }
                }
            }
        }
    };

    let markup = markup.into_string();
    axum::response::Html::from(markup)
}



pub async fn create_article(
    State(db): State<crate::database::Database>,
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

    if db.create_article(&title, &body).await {
        http::StatusCode::CREATED
    } else {
        http::StatusCode::CONFLICT
    }
}

pub async fn delete_article(
    State(db): State<crate::database::Database>,
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

    if db.delete_article(&title).await {
        http::StatusCode::OK
    } else {
        http::StatusCode::NO_CONTENT
    }
}
