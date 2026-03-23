use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Debug, Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        
        // Create table if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS xrp_5min (
                id SERIAL PRIMARY KEY,
                title VARCHAR(255) NOT NULL,
                start_ts BIGINT NOT NULL,
                end_ts BIGINT NOT NULL,
                initial_up_price DOUBLE PRECISION NOT NULL,
                initial_down_price DOUBLE PRECISION NOT NULL,
                low_up_price DOUBLE PRECISION NOT NULL,
                low_down_price DOUBLE PRECISION NOT NULL,
                last_up_price DOUBLE PRECISION NOT NULL,
                last_down_price DOUBLE PRECISION NOT NULL,
                result VARCHAR(10) NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await?;
        
        Ok(Database { pool })
    }
    
    pub async fn save_market_result(
        &self,
        title: &str,
        start_ts: i64,
        end_ts: i64,
        initial_up: f64,
        initial_down: f64,
        low_up: f64,
        low_down: f64,
        last_up: f64,
        last_down: f64,
        result: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO xrp_5min (
                title, start_ts, end_ts, initial_up_price, initial_down_price,
                low_up_price, low_down_price, last_up_price, last_down_price, result
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(title)
        .bind(start_ts)
        .bind(end_ts)
        .bind(initial_up)
        .bind(initial_down)
        .bind(low_up)
        .bind(low_down)
        .bind(last_up)
        .bind(last_down)
        .bind(result)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}