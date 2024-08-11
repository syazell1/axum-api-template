use sqlx::{Executor, FromRow, PgPool, Postgres, Transaction};
use async_trait::async_trait;
use sqlx::postgres::{PgArguments, PgRow};
use sqlx::query::{Query, QueryAs};
#[cfg(test)]
use mockall::{automock, predicate::*};
use crate::errors::AppError;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait DbContext {
    async fn get_transaction<'a>(&self) -> Result<Tx<'a>, AppError>;
    async fn execute_query<'a>(&self, query : Query<'a, Postgres, PgArguments>) -> Result<(), AppError>;
    async fn fetch_optional<T>(&self, query : QueryAs<'static, Postgres, T, PgArguments>) -> Result<Option<T>, AppError>
        where T : for<'a> FromRow<'a, PgRow> + Send + Sync + Unpin + 'static;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait TxContext {
    async fn execute_query<'a>(&mut self, query : Query<'a, Postgres, PgArguments>) -> Result<(), AppError>;
    async fn fetch_optional<'a>(&mut self, query : Query<'a, Postgres, PgArguments>) -> Result<Option<PgRow>, AppError>;
    async fn execute_transaction(self) -> Result<(), AppError>;
}

pub struct Tx<'a> {
    pub tx : Transaction<'a, Postgres>
}

pub struct DbPool {
    pub pool : PgPool
}

#[async_trait]
impl DbContext for DbPool {
    async fn get_transaction<'a>(&self) -> Result<Tx<'a>,AppError> {
        let tx = self.pool.begin().await?;

        let tx = Tx {tx};

        Ok(tx)
    } 
    
    async fn execute_query<'a>(&self, query: Query<'a, Postgres, PgArguments>) -> Result<(), AppError> {
        self.pool.execute(query).await?;

        Ok(())
    }

    async fn fetch_optional<T>(&self, query: QueryAs<'static, Postgres, T, PgArguments>) -> Result<Option<T>, AppError>
    where
        T: for<'a> FromRow<'a, PgRow> + Send + Sync + Unpin + 'static
    {
        let result = query.fetch_optional(&self.pool).await?;

        Ok(result)
    }
}

#[async_trait]
impl <'a>TxContext for Tx<'a> {
    async fn execute_query<'b>(&mut self, query : Query<'b, Postgres, PgArguments>) -> Result<(), AppError> {
        self.tx.execute(query).await?;

        Ok(())
    }


    async fn fetch_optional<'b>(&mut self, query : Query<'b, Postgres, PgArguments>) -> Result<Option<PgRow>, AppError> {
            let result = self.tx.fetch_optional(query).await?;

            Ok(result)
        }

    async fn execute_transaction(self) -> Result<(), AppError> {
        self.tx.commit().await?;

        Ok(())
    }
}