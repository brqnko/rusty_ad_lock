#[cfg(feature = "sqlx-mysql")]
mod sqlx;

#[cfg(feature = "sqlx-mysql")]
pub use sqlx::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "sqlx-mysql")]
    #[error(transparent)]
    Sqlx(#[from] ::sqlx::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct LockerGuard<'s, 'p, DB: ::sqlx::Database> {
    pub(crate) text: &'s str,
    pub(crate) pool: &'p ::sqlx::Pool<DB>,
}

impl<'s, 'p, DB: ::sqlx::Database> Drop for LockerGuard<'s, 'p, DB> {
    fn drop(&mut self) {
        //
    }
}

pub trait Locker: Sized {
    type DB: ::sqlx::Database;

    fn lock<'s, 'p>(
        pool: &::sqlx::Pool<Self::DB>,
        text: &str,
        timeout: Option<std::time::Duration>,
    ) -> impl Future<Output = Result<Option<LockerGuard<'s, 'p, Self::DB>>>> + Send;

    fn unlock(pool: &::sqlx::Pool<Self::DB>, text: &str) -> impl Future<Output = Result<()>>;
}
