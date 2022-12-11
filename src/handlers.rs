
use axum::{
    extract::*,
    http,
};

const ADDR: &str = "127.0.0.1:3000";


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
    State(db): State<crate::database::Database>,
) -> axum::response::Html<String> {

    let articles = db.fetch_articles();

    let markup = maud::html! {
        h1 { "Welcome to my blog!" }

        div style="display: flex; flex-direction: column" {
            @for (added, title) in articles {
                a href=(format!("http://{ADDR}/blog/{title}")) {
                    (format!("{added} - {title}\n"))
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

    let (article_title, article_body) = db.fetch_article(&title);

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
    State(st): State<crate::database::Database>,
    Path(title): Path<String>,
    headers: axum::http::header::HeaderMap,
    body: String,
) -> http::StatusCode {

    let Some(hv) = headers.get("authorization") else {
        return http::StatusCode::FORBIDDEN
    };
    let Ok("Basic YWRtaW46YWRtaW4=") = hv.to_str() else {
        return http::StatusCode::FORBIDDEN
    };

    st.create_article(&title, &body);

    http::StatusCode::ACCEPTED
}
