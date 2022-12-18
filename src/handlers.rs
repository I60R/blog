
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

    let mut output = String::new();
    let parser = pulldown_cmark::Parser::new(&article_body);
    pulldown_cmark::html::push_html(&mut output, parser);

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
