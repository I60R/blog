
use axum::{
    extract::*,
    http,
    response,
};
use axum_auth::AuthBasic;
use crate::{view, ADDR, database};



pub async fn get_articles(
    State(db): State<database::Database>,
) -> response::Html<String> {

    let mut articles = db.fetch_articles().await;

    for article_list_item in &mut articles {
        article_list_item.title = urlencoding::decode(&article_list_item.title)
            .unwrap()
            .to_string();
    }

    let v = view::display_articles(articles);
    response::Html::from(v)
}

pub async fn get_article(
    State(db): State<database::Database>,
    Path(title): Path<String>,
) -> impl response::IntoResponse {

    let title = urlencoding::encode(&title);
    let article_item = db.fetch_article(&title).await;

    if let Some(article_item) = article_item {
        let v = view::display_article(article_item);
        Ok(response::Html::from(v))
    } else {
        Err(response::Redirect::permanent(&format!("{ADDR}/blog")))
    }
}

pub async fn next_article(
    State(db): State<database::Database>,
    Path(id): Path<i64>,
) -> response::Redirect {
    let article_title = db
        .fetch_next_article_title_after_id(id)
        .await;
    response::Redirect::permanent(&format!("{ADDR}/blog/{article_title}"))
}

pub async fn prev_article(
    State(db): State<database::Database>,
    Path(id): Path<i64>,
) -> response::Redirect {
    let article_title = db
        .fetch_prev_article_title_before_id(id)
        .await;
    response::Redirect::permanent(&format!("{ADDR}/blog/{article_title}"))
}


pub async fn create_article(
    State(db): State<database::Database>,
    Path(title): Path<String>,
    AuthBasic((id, password)): AuthBasic,
    body: String,
) -> http::StatusCode {
    let ("admin", Some("admin")) = (id.as_ref(), password.as_deref()) else {
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
    State(db): State<database::Database>,
    Path(title): Path<String>,
    AuthBasic((id, password)): AuthBasic,
) -> http::StatusCode {
    let ("admin", Some("admin")) = (id.as_ref(), password.as_deref()) else {
        return http::StatusCode::UNAUTHORIZED
    };

    let title = urlencoding::encode(&title);

    if db.delete_article(&title).await {
        http::StatusCode::OK
    } else {
        http::StatusCode::NO_CONTENT
    }
}
