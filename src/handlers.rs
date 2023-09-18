
use axum::{
    extract::*,
    http,
    response,
};
use axum_auth::AuthBasic;
use crate::{view, repository, ADDR, article};


pub async fn get_articles(
    State(mut repo): State<repository::ArticlesRepository>,
) -> response::Html<String> {
    let articles = repo.fetch_articles().await;
    let articles = (&articles).iter().map(|a| {
        article::ListItem {
            id: a.id,
            added: a.added.clone(),
            title: urlencoding::decode(&a.title)
                .unwrap()
                .to_string(),
        }
    });
    let v = view::display_articles(articles);
    response::Html::from(v)
}

pub async fn get_article(
    State(mut repo): State<repository::ArticlesRepository>,
    Path(title): Path<String>,
) -> impl response::IntoResponse {
    let title = urlencoding::encode(&title);
    let article_item = repo.fetch_article(&title).await;

    if let Some(article_item) = article_item.as_ref() {
        let v = view::display_article(article_item);
        Ok(response::Html::from(v))
    } else {
        Err(response::Redirect::permanent(&format!("{ADDR}/blog")))
    }
}

pub async fn next_article(
    State(mut repo): State<repository::ArticlesRepository>,
    Path(id): Path<u32>,
) -> response::Redirect {
    let article_title = repo
        .fetch_next_article_title_after_id(id)
        .await;
    response::Redirect::permanent(&format!("{ADDR}/blog/{article_title}"))
}

pub async fn prev_article(
    State(repo): State<repository::ArticlesRepository>,
    Path(id): Path<u32>,
) -> response::Redirect {
    let article_title = repo
        .fetch_prev_article_title_before_id(id).await;
    response::Redirect::permanent(&format!("{ADDR}/blog/{article_title}"))
}


pub async fn create_article(
    State(mut repo): State<repository::ArticlesRepository>,
    Path(title): Path<String>,
    AuthBasic((id, password)): AuthBasic,
    body: String,
) -> http::StatusCode {
    let ("admin", Some("admin")) = (id.as_ref(), password.as_deref()) else {
        return http::StatusCode::UNAUTHORIZED
    };

    let title = urlencoding::encode(&title);
    let body = base64::encode(body);

    if repo.create_article(&title, &body).await {
        http::StatusCode::CREATED
    } else {
        http::StatusCode::CONFLICT
    }
}

pub async fn delete_article(
    State(mut repo): State<repository::ArticlesRepository>,
    Path(title): Path<String>,
    AuthBasic((id, password)): AuthBasic,
) -> http::StatusCode {
    let ("admin", Some("admin")) = (id.as_ref(), password.as_deref()) else {
        return http::StatusCode::UNAUTHORIZED
    };

    let title = urlencoding::encode(&title);

    if repo.delete_article(&title).await {
        http::StatusCode::OK
    } else {
        http::StatusCode::NO_CONTENT
    }
}

pub async fn admin_login() -> impl response::IntoResponse {
    let v = view::admin_login();
    response::Html::from(v)
}

pub async fn admin_panel(
    State(mut repo): State<repository::ArticlesRepository>,
    // AuthBasic((id, password)): AuthBasic,
) -> impl response::IntoResponse {
    // let ("admin", Some("admin")) = (id.as_ref(), password.as_deref()) else {

    //     let status_code = http::StatusCode::UNAUTHORIZED.as_str()
    //         .to_string();
    //     return response::Html::from(status_code)
    // };

    let articles = repo.fetch_articles().await;
    let articles = (&articles).iter().map(|a| {
        article::ListItem {
            id: a.id,
            added: a.added.clone(),
            title: urlencoding::decode(&a.title)
                .unwrap()
                .to_string(),
        }
    });
    let v = view::admin_panel(articles);
    response::Html::from(v)
}