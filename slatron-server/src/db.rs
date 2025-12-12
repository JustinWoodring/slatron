use anyhow::Result;
use diesel::r2d2::{self, ConnectionManager};
use diesel::sqlite::SqliteConnection;

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;

pub fn create_pool(database_url: &str) -> Result<DbPool> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)?;
    Ok(pool)
}

pub fn run_migrations(conn: &mut DbConnection) -> Result<()> {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

    conn.run_pending_migrations(MIGRATIONS)
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!("Migration error: {}", e))
}
