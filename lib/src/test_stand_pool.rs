use rocket::figment;
use rocket_db_pools::Pool;

// ? If the `connection_type` attribute is used, this should be impl'd on that instead
// ? Use a custom `Error` type instead of piggybacking off `Pool`'s?
#[rocket::async_trait]
pub trait TestStandPool: Pool + Send + Sync + 'static {
    type Error: std::fmt::Debug + std::fmt::Display;
    async fn create_database(
        figment: &figment::Figment,
    ) -> Result<String, <Self as TestStandPool>::Error>;

    async fn migrate_database(
        database_name: &str,
        migration_path: &str,
        figment: &figment::Figment,
    ) -> Result<String, <Self as TestStandPool>::Error>;
}

pub(crate) fn get_database_name(config: &rocket_db_pools::Config) -> Option<&str> {
    let Some(idx) = config.url.rfind('/') else {
        return None;
    };
    Some(&config.url[idx + 1..])
}

pub(crate) fn update_connection_url(url: &mut String, new_database: &str) -> Option<()> {
    let Some(idx) = url.rfind('/') else {
        return None;
    };
    url.replace_range(idx + 1.., new_database);
    Some(())
}

// ! This needs to be behind a feature flag
// ! Improve error handling
#[cfg(feature = "sqlx")]
mod sqlx {
    use crate::error::Error;
    use rocket::{
        figment::{self, providers},
        info,
    };
    use rocket_db_pools::{Config, Error as DbPoolsError, Pool};
    use uuid::Uuid;

    #[rocket::async_trait]
    impl<DB> crate::TestStandPool for sqlx::Pool<DB>
    where
        DB: sqlx::Database,
        <DB as sqlx::Database>::Connection: sqlx::migrate::Migrate,
        for<'e> &'e mut <DB as sqlx::Database>::Connection: sqlx::Executor<'e, Database = DB>,
        for<'args> <DB as sqlx::database::HasArguments<'args>>::Arguments:
            sqlx::IntoArguments<'args, DB>,
    {
        type Error = Error<sqlx::Error, sqlx::migrate::MigrateError, String>;
        async fn create_database(
            figment: &figment::Figment,
        ) -> Result<String, <Self as super::TestStandPool>::Error> {
            let parsed_config = figment.extract::<Config>()?;
            let database = super::get_database_name(&parsed_config).ok_or(Error::Config(
                "could not parse database name from url".to_string(),
            ))?;
            let pool = <Self as Pool>::init(figment).await.map_err(Error::DbPool)?;

            let temp_db = format!("{}_{}", database, Uuid::new_v4());
            info!("creating temporary database: {}", temp_db);
            sqlx::query(&format!(r#"CREATE DATABASE "{}""#, temp_db))
                .execute(&pool)
                .await
                .map_err(|e| Error::DbPool(DbPoolsError::Init(e)))?;

            pool.close().await;

            Ok(temp_db)
        }

        async fn migrate_database(
            database_name: &str,
            migration_path: &str,
            figment: &figment::Figment,
        ) -> Result<String, <Self as super::TestStandPool>::Error> {
            let mut url: String = figment
                .extract_inner("url")
                .map_err(|e| Error::DbPool(DbPoolsError::from(e)))?;
            super::update_connection_url(&mut url, database_name).ok_or(Error::Config(
                "could not create a new connection string".to_string(),
            ))?;
            let new_config = figment
                .clone()
                .merge(providers::Serialized::default("url", &url));
            let pool = <Self as Pool>::init(&new_config)
                .await
                .map_err(Error::DbPool)?;
            sqlx::migrate::Migrator::new(std::path::Path::new(migration_path))
                .await
                .map_err(Error::Migrate)?
                .run(&pool)
                .await
                .map_err(Error::Migrate)?;
            Ok(url)
        }
    }
}
