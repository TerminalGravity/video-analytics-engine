use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .acquire_timeout(Duration::from_secs(30))
            .connect(database_url)
            .await?;

        Ok(Database { pool })
    }

    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        // In a real implementation, you'd use sqlx-cli migrations
        // For now, we'll just verify the connection works
        let _result = sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        
        tracing::info!("Database migration check completed");
        Ok(())
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn health_check(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }
} 