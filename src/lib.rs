use std::ops::Deref;
use std::rc::Rc;

pub mod model;
pub mod mysql;
pub mod postgresql;
pub mod sqlite;
pub mod xml;

#[derive(Debug)]
pub struct Book<DB, RAW>
where
    DB: sqlx::database::Database,
{
    pool: either::Either<Rc<sqlx::Pool<DB>>, Rc<RAW>>,
}

#[derive(Debug)]
pub struct Item<T, DB, RAW>
where
    DB: sqlx::database::Database,
{
    content: T,
    pool: either::Either<Rc<sqlx::Pool<DB>>, Rc<RAW>>,
}

impl<T, DB, RAW> Item<T, DB, RAW>
where
    DB: sqlx::database::Database,
{
    fn new(content: T, pool: &either::Either<Rc<sqlx::Pool<DB>>, Rc<RAW>>) -> Self {
        match pool {
            either::Left(pool) => Self {
                content,
                pool: either::Left(Rc::clone(&pool)),
            },
            either::Right(raw) => Self {
                content,
                pool: either::Right(Rc::clone(&raw)),
            },
        }
    }
}

impl<T, DB, RAW> Deref for Item<T, DB, RAW>
where
    DB: sqlx::database::Database,
{
    type Target = T;

    fn deref(&self) -> &T {
        &self.content
    }
}

#[derive(Debug)]
pub struct Ignore;
