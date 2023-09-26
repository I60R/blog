pub mod database;
pub mod repository;
pub mod article;

// Connects to a database using `DATABASE_URL` from .env
pub async fn connect_database() -> Result<database::Database, Box<dyn std::error::Error>> {
    let database_url = &std::env::var("DATABASE_URL")
        .map_err(|_| "No DATABASE_URL set, check README.md")?;

    let mysql = sqlx::mysql::MySqlPool::connect(database_url).await?;

    let db = database::Database::new_migrate(mysql).await;

    Ok(db)
}
