#[cfg(feature = "sqlx-mysql")]
mod sqlx;

#[cfg(feature = "sqlx-mysql")]
pub use sqlx::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub struct LockerGuard {}

pub trait Locker {
    type Pool;

    fn lock(
        pool: &Self::Pool,
        text: &str,
        timeout: Option<std::time::Duration>,
    ) -> impl Future<Output = Result<Option<LockerGuard>>> + Send;

    fn unlock(pool: &Self::Pool, text: &str) -> impl Future<Output = Result<()>>;
}
