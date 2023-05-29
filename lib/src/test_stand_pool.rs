use rocket::figment;
use rocket_db_pools::Pool;

// ? If the `connection_type` attribute is used, this should be impl'd on that instead
// ? Use a custom `Error` type instead of piggybacking off `Pool`'s?
#[rocket::async_trait]
pub trait TestStandPool: Pool + Send + Sync + 'static {
    async fn create_database(config: &figment::Figment) -> Result<String, <Self as Pool>::Error>;

    async fn migrate_database(
        database_name: &str,
        migration_path: &str,
        config: &figment::Figment,
    ) -> Result<String, <Self as Pool>::Error>;
}

// ! This needs to be behind a feature flag
// ! Improve error handling
#[cfg(feature = "sqlx")]
mod sqlx {
    use rocket::{
        figment::{self, providers},
        info,
    };
    use rocket_db_pools::{Config, Error, Pool};
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
        async fn create_database(
            config: &figment::Figment,
        ) -> Result<String, <Self as Pool>::Error> {
            let parsed_config = config.extract::<Config>()?;
            let database =
                &parsed_config.url[parsed_config.url.rfind('/').unwrap_or_default() + 1..];
            let pool = <Self as Pool>::init(config).await?;

            let temp_db = format!("{}_{}", database, Uuid::new_v4());
            info!("creating temporary database: {}", temp_db);
            sqlx::query(&format!(r#"CREATE DATABASE "{}""#, temp_db))
                .execute(&pool)
                .await
                .map_err(Error::Init)?;

            pool.close().await;

            Ok(temp_db)
        }

        async fn migrate_database(
            database_name: &str,
            migration_path: &str,
            config: &figment::Figment,
        ) -> Result<String, <Self as Pool>::Error> {
            let mut url: String = config.extract_inner("url").unwrap_or_default();
            url.replace_range(url.rfind('/').unwrap_or_default() + 1.., database_name);
            let new_config = config
                .clone()
                .merge(providers::Serialized::default("url", &url));
            let pool = <Self as Pool>::init(&new_config).await?;
            sqlx::migrate::Migrator::new(std::path::Path::new(migration_path))
                .await
                .unwrap()
                .run(&pool)
                .await
                .unwrap();
            Ok(url)
        }
    }
}
