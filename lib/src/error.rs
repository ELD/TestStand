use std::fmt;

#[derive(Debug)]
pub enum Error<A, B, C, D = A> {
    DbPool(rocket_db_pools::Error<A, D>),
    Migrate(B),
    Config(C),
}

impl<A, B, C, D> fmt::Display for Error<A, B, C, D>
where
    A: fmt::Display,
    B: fmt::Display,
    C: fmt::Display,
    D: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DbPool(db_pool) => db_pool.fmt(f),
            Error::Migrate(migrate) => write!(f, "migration error: {}", migrate),
            Error::Config(config) => write!(f, "configuration error: {}", config),
        }
    }
}

impl<A, B, C, D> std::error::Error for Error<A, B, C, D>
where
    A: fmt::Debug + fmt::Display,
    B: fmt::Debug + fmt::Display,
    C: fmt::Debug + fmt::Display,
    D: fmt::Debug + fmt::Display,
{
}

impl<A, B, C, D> From<rocket::figment::Error> for Error<A, B, C, D> {
    fn from(e: rocket::figment::Error) -> Self {
        Self::DbPool(rocket_db_pools::Error::from(e))
    }
}
