use std::ops::Deref;
use std::rc::Rc;

pub mod model;
pub mod sqlite;

#[derive(Debug)]
pub struct Book<DB>
where
    DB: sqlx::database::Database,
{
    pool: Rc<sqlx::Pool<DB>>,
}

#[derive(Debug)]
pub struct Item<T, DB>
where
    DB: sqlx::database::Database,
{
    content: T,
    pool: Rc<sqlx::Pool<DB>>,
}

impl<T, DB> Item<T, DB>
where
    DB: sqlx::database::Database,
{
    fn new(content: T, pool: Rc<sqlx::Pool<DB>>) -> Self {
        Self {
            content,
            pool: Rc::clone(&pool),
        }
    }
}

impl<T, DB> Deref for Item<T, DB>
where
    DB: sqlx::database::Database,
{
    type Target = T;

    fn deref(&self) -> &T {
        &self.content
    }
}
