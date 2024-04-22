use crate::error;
use error::Result;
use sqlx::PgPool;

#[derive(Debug)]
pub struct PgStorage {
    pool: PgPool,
}

impl PgStorage {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn get_tip(&self) -> Result<u64> {
        Ok(0)
    }
    pub async fn upsert_tip(&self, height: i64) -> Result<()> {
        sqlx::query(
            "insert into btc_last_height values($1,$2) on conflict(tip) do update set height=$2",
        )
        .bind("tip")
        .bind(height)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    pub async fn upsert_header(&self, block_id: &str) -> Result<()> {
        todo!()
    }
    pub async fn get_header(&self, block_id: &str) -> Result<()> {
        todo!()
    }
    pub async fn upsert_block(&self, block_id: &str) -> Result<()> {
        todo!()
    }
    pub async fn get_block(&self, block_id: &str) -> Result<()> {
        todo!()
    }
}
