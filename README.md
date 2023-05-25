# Test Stand

A crate for spinning up temporary databases in testing.

## Usage

The Test Stand crate ships with a `derive` macro to make it seamless to add
to your existing database type. Simply derive `TestStand` and provide the
`database` and `migration_path` attributes.

```rust
use rocket_db_pools::Database;
use test_stand::TestStand;

#[derive(Database, TestStand)]
#[database("my_database")]
#[migration_path("./migrations")]
pub struct MyDatabase(sqlx::PgPool);

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .attach(MyDatabase::test_stand())
        .attach(MyDatabase::init())
        .mount(...)
}
```

## Why?

I've long needed a solution for creating temporary databases when integration
testing [Rocket](https://rocket.rs) endpoints. The solution, up to this point,
has been to roll your own Fairing (middleware), run tests serially in a single-
threaded mode, or clean up the database after an integration test.

The first solution is onerous and time consuming to set up with every project.
The second solution slows down the development and CI/CD cycle because the tests
run slower. The final solution is onerous when writing tests because you always
have to clean things up.

This crate provides a drop-in fairing that allows for creating a temporary
database using your existing, declared database credentials. From there, it creates
a temporary database, prints it on bootup, and edits the Rocket configuration to
use the new database. By default, the crate does not clean up the databases after
the fact so you can inspect the data should something fail.

### Why is it called "Test Stand?"

I chose this name because the motivating framework is Rocket. Since this is a utility
to make testing easier, the test stand that rocket engines are test fired on seemed
to be a suitable name for such a crate.
