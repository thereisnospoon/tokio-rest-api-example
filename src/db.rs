use std::future::Future;

use deadpool_postgres::{Config, ManagerConfig, Pool, PoolError, RecyclingMethod};
use deadpool_postgres::tokio_postgres::{NoTls, Row};
use deadpool_postgres::tokio_postgres::error::Error as PgError;
use deadpool_postgres::tokio_postgres::types::ToSql;

use crate::AppResult;
use crate::entities::User;
use crate::errors::AppError;

pub struct DbService {
    pool: Pool,
}

impl DbService {
    pub fn new() -> DbService {
        let mut pool_config = Config::new();
        pool_config.dbname = Some("tokio".to_string());
        pool_config.host = Some("localhost".to_string());
        pool_config.user = Some("postgres".to_string());
        pool_config.password = Some("password".to_string());
        pool_config.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast
        });
        let pool = pool_config.create_pool(NoTls).unwrap();
        DbService {
            pool
        }
    }

    pub async fn insert_user(&self, user: &User) -> AppResult<()> {
        let pg_client = self.pool.get().await.map_err(map_pool_error)?;
        let insert_sql = "\
        insert into users(id, name, age, sex)\
        values ($1, $2, $3, $4)
        ";
        let statement = pg_client.prepare(insert_sql).await.map_err(map_pg_error)?;
        let sex = &user.sex;
        let sex_as_str: &str = sex.into();
        let params: &[&(dyn ToSql + Sync)] = &[&user.id, &user.name, &user.age, &sex_as_str];
        pg_client.execute(&statement, params).await.map_err(map_pg_error)?;
        Ok(())
    }

    pub async fn init_db(&self) -> AppResult<()> {
        let schema = std::include_str!("resources/schema.sql");
        let pg_client = self.pool.get().await.map_err(map_pool_error)?;
        pg_client.batch_execute(schema).await.map_err(map_pg_error)?;
        Ok(())
    }

    fn fetch_many<'a, T: From<Row>, P: ToSql + Sync + 'a>(&'a self, sql: &'static str, param: P) -> impl Future<Output=AppResult<Vec<T>>> + 'a {
        async move {
            let pg_client = self.pool.get().await.map_err(map_pool_error)?;
            let rows = pg_client.query(sql, &[&param]).await.map_err(map_pg_error)?;
            let parsed = rows.into_iter().map(|row| { T::from(row) }).collect();
            Ok(parsed)
        }
    }

    fn fetch_one<'a, T: From<Row>>(&'a self, sql: &'static str, id: &'a str) -> impl Future<Output=AppResult<Option<T>>> + 'a {
        async move {
            let pg_client = self.pool.get().await.map_err(map_pool_error)?;
            let row = pg_client.query_opt(sql, &[&id]).await.map_err(map_pg_error)?;
            Ok(row.map(|r| { T::from(r) }))
        }
    }

    pub async fn get_user_by_id(&self, user_id: &str) -> AppResult<Option<User>> {
        let res = self.fetch_one("select * from users where id = $1", user_id).await?;
        Ok(res)
    }

    pub async fn get_users_by_name(&self, name: &str) -> AppResult<Vec<User>> {
        let res: Vec<User> = self.fetch_many("select * from users where name = $1", name).await?;
        Ok(res)
    }
}

fn map_pool_error(pool_error: PoolError) -> AppError {
    AppError {
        message: "DB Pool error".to_string()
    }
}

fn map_pg_error(error: PgError) -> AppError {
    eprintln!("{:?}", error);
    AppError {
        message: error.to_string()
    }
}