use std::marker::PhantomData;

use rocket::{fairing, figment::providers, Build, Rocket};
use rocket_db_pools::Database;

use crate::TestStandPool;

pub trait TestStand: Database {
    const NAME: &'static str;
    const MIGRATION_PATH: &'static str;

    type TestStand: TestStandPool;

    fn test_stand() -> Initializer<Self> {
        Initializer::new()
    }
}

pub struct Initializer<T: TestStand>(Option<&'static str>, PhantomData<fn() -> T>);

impl<T: TestStand> Initializer<T> {
    pub fn new() -> Self {
        Initializer(None, PhantomData)
    }

    pub fn with_name(name: &'static str) -> Self {
        Initializer(Some(name), PhantomData)
    }
}

impl<T: TestStand> Default for Initializer<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[rocket::async_trait]
impl<T: TestStand> fairing::Fairing for Initializer<T> {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: self.0.unwrap_or(std::any::type_name::<Self>()),
            kind: fairing::Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        let config = rocket.figment();
        let workers: usize = config
            .extract_inner(rocket::Config::WORKERS)
            .unwrap_or_else(|_| rocket::Config::default().workers);

        let figment = config
            .focus(&format!("databases.{}", <T as TestStand>::NAME))
            .merge(providers::Serialized::default(
                "max_connections",
                workers * 4,
            ))
            .merge(providers::Serialized::default("connect_timeout", 5));

        let database_name =
            <<T as TestStand>::TestStand as TestStandPool>::create_database(&figment)
                .await
                .unwrap();

        let connection_url = <<T as TestStand>::TestStand as TestStandPool>::migrate_database(
            &database_name,
            T::MIGRATION_PATH,
            &figment,
        )
        .await
        .unwrap();

        let new_conf = config.clone().merge(providers::Serialized::default(
            &format!("databases.{}.url", <T as TestStand>::NAME),
            connection_url,
        ));

        Ok(rocket.configure(new_conf))
    }
}
