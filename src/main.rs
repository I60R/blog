

use axum::{routing, extract};
use redb::ReadableTable;
use std::{
    net::SocketAddr,
    sync::Arc
};


#[derive(Clone)]
struct State {
    db: Arc<redb::Database>
}

const BLOGS: redb::TableDefinition<&str, &str> = redb::TableDefinition::new("blogs");



#[tokio::main]
async fn main() {
    let db = unsafe {
        redb::Database::create("blogs.redb")
            .expect("Database Error")
    };
    let db = Arc::new(db);

    let state = State {
        db,
    };

    let app: axum::Router<_, axum::body::Body> = axum::Router::new()
        .route("/", routing::get(get_about))
        .route("/blog", routing::get(get_articles))
        .route("/blog/:title", routing::post(create_article))
        .route("/blog/:title", routing::get(get_article))
        .route("/image/:title", routing::get(get_article))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Server Error");
}



async fn get_articles(
    extract::State(st): extract::State<State>,
) -> String {
    let db = Arc::clone(&st.db);
    let txn = db.begin_read().expect("Txn read error");

    let mut articles = vec![];
    for t in txn.list_tables() {
        for a in t {
            articles.push(a)
        }
    }

    let x = articles.join("\nhttp://127.0.0.1:300/blog/");
    x
}

async fn get_about() -> &'static str {
    "Hello world: about"
}

async fn get_article(
    extract::Path(title): extract::Path<String>,
    extract::State(st): extract::State<State>,
) -> String {
    let db = Arc::clone(&st.db);
    let txn = db.begin_read().expect("Txn read error");
    {
        let table = txn.open_table(BLOGS).expect("Open read error");
        if let Some(article) = table
            .get(&title)
            .expect("Get read error")
        {
            return String::from(article)
        } else {
            return title
        }
    }
}

async fn create_article(
    extract::Path(title): extract::Path<String>,
    extract::State(st): extract::State<State>,
    payload: String,
) -> String {
    let db = Arc::clone(&st.db);
    let txn = db.begin_write().expect("Txn error");
    {
        let mut table = txn.open_table(BLOGS).expect("Open error");
        table.insert(&title, &payload).expect("Insert error");
    }
    txn.commit().expect("Txn commit error");


    format!("title: {title}, payload: {payload}")
}
