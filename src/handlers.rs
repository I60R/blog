
use axum::{
    extract::*,
    http,
};
use std::sync::Arc;


pub async fn get_about() -> axum::response::Html<String> {

    let markup = maud::html! {
        div style="display: flex; flex-direction: column" {
            h1 { "Welcome to my blog!" }
            a href="blog" {
                "list articles"
            }
        }
    };

    let markup = markup.into_string();
    axum::response::Html::from(markup)
}




pub async fn get_articles(
    State(st): State<crate::Database>,
) -> axum::response::Html<String> {

    let db = Arc::clone(&st.db);
    let db = db.lock().unwrap();

    let q = "
        SELECT added, title FROM blogs
    ";

    let mut resp = vec![];

    db.iterate(q, |pairs| {
        let mut added = None;

        for &(column, value) in pairs {
            match (column, value) {
                ("added", date_added) => {
                    added = date_added;
                },
                ("title", title) => {
                    resp.push((
                        added.unwrap().to_string(),
                        title.unwrap().to_string()
                    ));
                },
                _ => { }
            }
        }

        true
    }).unwrap();

    let markup = maud::html! {
        h1 { "Welcome to my blog!" }

        div style="display: flex; flex-direction: column" {
            @for (added, title) in resp {
                a href=(format!("http://127.0.0.1:3000/blog/{title}")) {
                    (format!("{added} - {title}\n"))
                }
            }
        }
    };

    let markup = markup.into_string();
    axum::response::Html::from(markup)
}




pub async fn get_article(
    State(st): State<crate::Database>,
    Path(title): Path<String>,
) -> axum::response::Html<String> {
    let db = Arc::clone(&st.db);
    let db = db.lock().unwrap();
    let q = format!("
        SELECT title, body FROM blogs WHERE title = '{title}'
    ");

    let mut article_title = String::new();
    let mut article_body = String::new();

    db.iterate(q, |pairs| {
        for &(column, value) in pairs {
            match (column, value) {
                ("title", Some(title)) => {
                    article_title.push_str(title);
                },
                ("body", Some(body)) => {
                    article_body.push_str(body);
                    return true;
                },
                _ => {
                    article_title = String::from("invalid format");
                }
            }
        }
        true
    }).unwrap();

    let mut output = String::new();
    let parser = pulldown_cmark::Parser::new(&article_body);
    pulldown_cmark::html::push_html(&mut output, parser);

    let markup = maud::html! {
        h1 { (article_title) }

        (maud::PreEscaped(output))
    };

    let markup = markup.into_string();
    axum::response::Html::from(markup)
}




pub async fn create_article(
    State(st): State<crate::Database>,
    Path(title): Path<String>,
    headers: axum::http::header::HeaderMap,
    payload: String,
) -> http::StatusCode {
    let Some(hv) = headers.get("authorization") else {
        return http::StatusCode::FORBIDDEN
    };
    let Ok("Basic YWRtaW46YWRtaW4=") = hv.to_str() else {
        return http::StatusCode::FORBIDDEN
    };

    let db = Arc::clone(&st.db);
    let db = db.lock().unwrap();

    let q = format!("
         INSERT INTO blogs (title, body, added)
            VALUES ('{title}', '{payload}', DATE('now'));
    ");
    db.execute(q).unwrap();

    http::StatusCode::ACCEPTED
}
