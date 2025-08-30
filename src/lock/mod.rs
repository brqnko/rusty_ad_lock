#[cfg(feature = "sqlx-mysql")]
mod sqlx;

#[cfg(feature = "sqlx-mysql")]
pub use sqlx::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "sqlx-mysql")]
    #[error(transparent)]
    Sqlx(#[from] ::sqlx::Error),

    #[cfg(feature = "sqlx-mysql")]
    #[error("MySQL returned null(insufficient memory or thread interrupted?)")]
    MySqlReturnedNull,
    #[cfg(feature = "sqlx-mysql")]
    #[error("unknown MySQL signal: {0}")]
    MySqlUnknownSignal(i32),

    #[error("failed to get lock: {0}")]
    FailedToGetLock(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Locker {
    type DB: ::sqlx::Database;

    fn with_locking<T, F>(
        pool: &::sqlx::Pool<Self::DB>,
        key: &str,
        timeout: Option<std::time::Duration>,
        f: F,
    ) -> impl Future<Output = Result<()>>
    where
        // FIXME: 長過ぎるわけだけど、トレイトエイリアスパターンを使ってみても微妙だったのでこれでいく
        F: AsyncFnOnce(&mut ::sqlx::Transaction<'static, Self::DB>) -> T;
}
