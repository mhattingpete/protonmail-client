pub mod queries;
pub mod schema;

use sqlx::SqlitePool;

pub async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let migration = include_str!("../../migrations/001_initial.sql");
    for statement in migration.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(pool).await?;
        }
    }
    Ok(())
}
