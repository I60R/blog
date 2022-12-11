

use axum::{
    routing as r,
    response as s,
    extract as e,
    http as h,
};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use maud::html;


#[derive(Clone)]
struct State {
    db: Arc<Mutex<sqlite::Connection>>
}


#[tokio::main]
async fn main() {
    let db = sqlite::open("blogs.sqlite")
        .expect("Cannot open DB");

    let q = "
        CREATE TABLE blogs (
            title TEXT NOT NULL PRIMARY_KEY,
            body TEXT NOT NULL,
            added TEXT NOT NULL,
        );
    ";
    if let Err(sqlite::Error { message: Some(message), .. }) = db
        .execute(q)
    {
        if message != "table blogs already exists" {
            panic!("{message}")
        }
    }

    let db = Arc::new(Mutex::new(db));
    let state = State {
        db,
    };

    let app: axum::Router<_, axum::body::Body> = axum::Router::new()
        .route("/", r::get(get_about))
        .route("/blog", r::get(get_articles))
        .route("/blog/:title", r::post(create_article))
        .route("/blog/:title", r::get(get_article))
        .route("/image/:title", r::get(get_article))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Server Error");
}


async fn get_about() -> &'static str {
    "Hello world: about"
}

async fn get_articles(
    e::State(st): e::State<State>,
) -> s::Html<String> {
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

    let markup = html! {
        h1 { "Welcome to my blog!" }

        div style="display: flex; flex-direction: column" {
            @for (added, title) in resp {
                a href=(format!("http://127.0.0.1:3000/blog/{title}")) {
                    (format!("{added} - {title}\n"))
                }
            }
        }
    };

    s::Html::from(markup.into_string())
}

async fn get_article(
    e::State(st): e::State<State>,
    e::Path(title): e::Path<String>,
) -> String {
    let db = Arc::clone(&st.db);
    let db = db.lock().unwrap();
    let q = format!("
        SELECT title, body FROM blogs WHERE title = '{title}'
    ");

    let mut resp = String::new();
    db.iterate(q, |pairs| {
        for &(column, value) in pairs {
            match (column, value) {
                ("title", Some(title)) => {
                    resp.push_str(title);
                    resp.push_str("\n");
                },
                ("body", Some(body)) => {
                    resp.push_str(body);
                    return true;
                },
                _ => {
                    resp = String::from("invalid format");
                }
            }
        }
        true
    }).unwrap();

    resp
}

async fn create_article(
    e::State(st): e::State<State>,
    e::Path(title): e::Path<String>,
    headers: h::header::HeaderMap,
    payload: String,
) -> h::StatusCode {
    let Some(hv) = headers.get("authorization") else {
        return h::StatusCode::FORBIDDEN
    };
    let Ok("Basic YWRtaW46YWRtaW4=") = hv.to_str() else {
        return h::StatusCode::FORBIDDEN
    };

    let db = Arc::clone(&st.db);
    let db = db.lock().unwrap();

    let q = format!("
         INSERT INTO blogs (title, body, added)
            VALUES ('{title}', '{payload}', DATE('now'));
    ");
    db.execute(q).unwrap();

    h::StatusCode::ACCEPTED
}
